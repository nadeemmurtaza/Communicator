# Prisma to SQL converter for ERP schema - v3 (Complete rewrite)
# Parses merged.prisma and generates SQL CREATE TABLE statements

$SchemaFile = "C:\Users\newax\IdeaProjects\Communicator\erp-schema\merged.prisma"
$OutputFile = "C:\Users\newax\IdeaProjects\Communicator\erp-schema\generated\0001_erp_schema.sql"

$null = New-Item -ItemType Directory -Path "C:\Users\newax\IdeaProjects\Communicator\erp-schema\generated" -Force

$rawContent = Get-Content $SchemaFile -Raw

# Strip block comments (/** ... */)
$rawContent = $rawContent -replace '/\*[\s\S]*?\*/', ''

# Strip single-line comments (but preserve URLs in strings)
$cleanLines = ($rawContent -split "`n" | ForEach-Object {
    $line = $_
    # Simple approach: remove // comments
    $inString = $false
    $result = ""
    for ($i = 0; $i -lt $line.Length; $i++) {
        $ch = $line[$i]
        if ($ch -eq '"') { $inString = -not $inString }
        if ($ch -eq '/' -and -not $inString -and $i + 1 -lt $line.Length -and $line[$i + 1] -eq '/') {
            break
        }
        $result += $ch
    }
    $result
})

# Type mapping from Prisma to PostgreSQL
$typeMap = @{
    "String" = "TEXT"
    "Int" = "INTEGER"
    "Float" = "DOUBLE PRECISION"
    "Boolean" = "BOOLEAN"
    "DateTime" = "TIMESTAMPTZ"
    "Json" = "JSONB"
    "Decimal" = "NUMERIC"
    "Bytes" = "BYTEA"
}

# Data structures
$enumValues = @{}
$models = @{}
$modelFields = @{}
$modelIndexes = @{}
$modelUnique = @{}
$modelMap = @{}

# ========== PASS 1: Extract enums ==========
$inEnum = $false
$currentEnum = ""
foreach ($line in $cleanLines) {
    $trimmed = $line.Trim()
    if ($trimmed -match '^enum\s+(\w+)\s*\{') {
        $inEnum = $true
        $currentEnum = $matches[1]
        $enumValues[$currentEnum] = @()
        continue
    }
    if ($inEnum -and $trimmed -eq '}') {
        $inEnum = $false
        $currentEnum = ""
        continue
    }
    if ($inEnum -and $trimmed -match '^(\w+)') {
        $enumValues[$currentEnum] += $matches[1]
        continue
    }
}

# ========== PASS 2: Extract models ==========
$inModel = $false
$currentModel = ""
$currentFieldLines = @()
$currentModelIndexes = @()
$currentModelUniques = @()
$currentModelMap = ""
$braceDepth = 0

foreach ($line in $cleanLines) {
    $trimmed = $line.Trim()
    if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }

    if ($trimmed -match '^model\s+(\w+)\s*\{') {
        $inModel = $true
        $currentModel = $matches[1]
        $currentFieldLines = @()
        $currentModelIndexes = @()
        $currentModelUniques = @()
        $currentModelMap = ""
        $braceDepth = 1
        continue
    }

    if ($inModel) {
        $opens = ($trimmed -replace '[^{]', '').Length
        $closes = ($trimmed -replace '[^}]', '').Length
        $braceDepth += $opens - $closes

        if ($braceDepth -le 0) {
            $models[$currentModel] = $true
            $modelFields[$currentModel] = $currentFieldLines
            $modelIndexes[$currentModel] = $currentModelIndexes
            $modelUnique[$currentModel] = $currentModelUniques
            $modelMap[$currentModel] = $currentModelMap
            $inModel = $false
            $currentModel = ""
            $braceDepth = 0
            continue
        }

        if ($trimmed -match '^@@index\((\[.*?\])\)') {
            $currentModelIndexes += $matches[1]
            continue
        }
        if ($trimmed -match '^@@unique\((\[.*?\])\)') {
            $currentModelUniques += $matches[1]
            continue
        }
        if ($trimmed -match '^@@map\("([^"]+)"\)') {
            $currentModelMap = $matches[1]
            continue
        }

        # Field lines start with a word character
        if ($trimmed -match '^\w') {
            $currentFieldLines += $trimmed
        }
    }
}

Write-Host "Parsed $($models.Count) models and $($enumValues.Count) enums"

# ========== Helper: get table name ==========
function Get-TableName {
    param($modelName)
    if ($modelMap[$modelName]) { return $modelMap[$modelName] }
    return $modelName.ToLower()
}

# ========== PASS 3: Parse fields and generate SQL ==========
$sql = @()
$sql += "-- ERP Schema generated from Prisma models"
$sql += "-- Generated on $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
$sql += ""
$sql += "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
$sql += ""

# Generate enum types
foreach ($enumName in $enumValues.Keys) {
    $enumSqlName = $enumName.ToLower()
    $values = $enumValues[$enumName] | ForEach-Object { "'" + $_.ToLower() + "'" }
    $sql += "CREATE TYPE $enumSqlName AS ENUM (" + ($values -join ', ') + ");"
    $sql += ""
}

# Collect all table definitions
$tableDefs = @{}

foreach ($modelName in $models.Keys) {
    $tableName = Get-TableName $modelName
    $columns = @()

    foreach ($fieldLine in $modelFields[$modelName]) {
        $fieldLine = $fieldLine.Trim().TrimEnd(',').Trim()
        if ([string]::IsNullOrWhiteSpace($fieldLine)) { continue }
        if ($fieldLine -match '^@@') { continue }

        # Parse field: fieldName fieldType [attributes]
        # fieldType can be: Word, Word[], Word?, Word[]?
        # We need to extract the base type name first
        
        # Step 1: Extract field name (first word)
        $spaceIdx = $fieldLine.IndexOf(' ')
        if ($spaceIdx -lt 0) { continue }
        
        $fieldName = $fieldLine.Substring(0, $spaceIdx)
        $rest = $fieldLine.Substring($spaceIdx + 1).Trim()
        
        # Step 2: Extract type from rest
        # Type can be: Word, Word[], Word?, Word[]?
        # Followed by optional attributes starting with @
        
        $fieldType = ""
        $isArray = $false
        $attrs = ""
        
        if ($rest -match '^(\w+)\[\]\??\s*(.*)') {
            $fieldType = $matches[1]
            $isArray = $true
            $attrs = $matches[2].Trim()
        } elseif ($rest -match '^(\w+)\??\s*(.*)') {
            $fieldType = $matches[1]
            $isArray = $false
            $attrs = $matches[2].Trim()
        } else {
            continue
        }
        
        $isOptional = $false
        # Check if ? is in the type part (before attrs)
        $typePart = $rest.Substring(0, $rest.IndexOf($fieldType) + $fieldType.Length)
        if ($rest -match "^\w+\[\]\?") { $isOptional = $true }
        elseif ($rest -match "^\w+\?\s") { $isOptional = $true }
        elseif ($rest -match "^\w+\?$") { $isOptional = $true }
        
        # Skip relation fields (type is a model name)
        if ($models.ContainsKey($fieldType)) {
            continue
        }

        # Convert type
        $sqlType = $typeMap[$fieldType]
        if (-not $sqlType) {
            if ($enumValues.ContainsKey($fieldType)) {
                $sqlType = $fieldType.ToLower()
            } else {
                $sqlType = "TEXT"
            }
        }

        if ($isArray) {
            $sqlType = $sqlType + "[]"
        }

        # Check attributes
        $isPrimaryKey = $false
        $isUnique = $false
        $defaultValue = $null
        $defaultExpr = $null

        if ($attrs -match '@id') {
            $isPrimaryKey = $true
        }
        if ($attrs -match '@unique') {
            $isUnique = $true
        }

        # Handle @default - need to handle nested parentheses like cuid(), now()
        $defaultPos = $attrs.IndexOf('@default(')
        if ($defaultPos -ge 0) {
            # Find the matching closing paren
            $start = $defaultPos + 9  # after '@default('
            $depth = 1
            $end = $start
            while ($end -lt $attrs.Length -and $depth -gt 0) {
                if ($attrs[$end] -eq '(') { $depth++ }
                elseif ($attrs[$end] -eq ')') { $depth-- }
                if ($depth -gt 0) { $end++ }
            }
            if ($depth -eq 0) {
                $defaultExpr = $attrs.Substring($start, $end - $start).Trim()
            }
        }

        if ($null -ne $defaultExpr) {
            if ($defaultExpr -eq 'now()') {
                $defaultValue = "CURRENT_TIMESTAMP"
            } elseif ($defaultExpr -eq 'true') {
                $defaultValue = "TRUE"
            } elseif ($defaultExpr -eq 'false') {
                $defaultValue = "FALSE"
            } elseif ($defaultExpr -eq '[]') {
                $defaultValue = "'{}'"
            } elseif ($defaultExpr -match '^"([^"]*)"$') {
                $defaultValue = "'" + $matches[1].Replace("'", "''") + "'"
            } elseif ($defaultExpr -match '^(\d+)$') {
                $defaultValue = $matches[1]
            } elseif ($defaultExpr -match '^(\d+\.\d+)$') {
                $defaultValue = $matches[1]
            } elseif ($defaultExpr -eq 'cuid()' -or $defaultExpr -eq 'uuid()') {
                $defaultValue = "gen_random_uuid()"
                $sqlType = "UUID"
                if ($isArray) { $sqlType = "UUID[]" }
            } elseif ($enumValues.ContainsKey($defaultExpr)) {
                $defaultValue = "'" + $defaultExpr.ToLower() + "'::$($fieldType.ToLower())"
            } else {
                $defaultValue = "'" + $defaultExpr.Replace("'", "''") + "'"
            }
        }

        # Handle @updatedAt
        if ($attrs -match '@updatedAt') {
            $defaultValue = "CURRENT_TIMESTAMP"
        }

        # Build column definition
        $colDef = "    `"$fieldName`" $sqlType"
        if ($isPrimaryKey) {
            $colDef += " PRIMARY KEY"
        }
        if ($defaultValue) {
            $colDef += " DEFAULT $defaultValue"
        }
        if (-not $isPrimaryKey -and -not $isOptional) {
            $colDef += " NOT NULL"
        }
        if ($isUnique) {
            $colDef += " UNIQUE"
        }

        $columns += $colDef
    }

    $tableDefs[$tableName] = $columns
}

# ========== Create tables ==========
foreach ($tableName in $tableDefs.Keys) {
    $columns = $tableDefs[$tableName]
    if (-not $columns.Count) { continue }

    $sql += "CREATE TABLE IF NOT EXISTS `"$tableName`" ("
    $sql += ($columns -join ",`n")
    $sql += ");"
    $sql += ""
}

# ========== Create indexes ==========
foreach ($modelName in $models.Keys) {
    $tableName = Get-TableName $modelName

    if ($modelIndexes[$modelName]) {
        $idxNum = 0
        foreach ($idx in $modelIndexes[$modelName]) {
            $colList = ($idx -replace '[\[\]]', '' -split ',' | ForEach-Object { $_.Trim() }) -join ', '
            $colsForName = ($idx -replace '[\[\]]', '' -split ',' | ForEach-Object { $_.Trim() }) -join '_'
            $idxName = "idx_" + $tableName + "_" + $colsForName
            $sql += "CREATE INDEX IF NOT EXISTS `"$idxName`" ON `"$tableName`" ($colList);"
            $idxNum++
        }
        $sql += ""
    }

    if ($modelUnique[$modelName]) {
        $uniqNum = 0
        foreach ($uniq in $modelUnique[$modelName]) {
            $colList = ($uniq -replace '[\[\]]', '' -split ',' | ForEach-Object { $_.Trim() }) -join ', '
            $colsForName = ($uniq -replace '[\[\]]', '' -split ',' | ForEach-Object { $_.Trim() }) -join '_'
            $uniqName = "uniq_" + $tableName + "_" + $colsForName
            $sql += "ALTER TABLE `"$tableName`" ADD CONSTRAINT `"$uniqName`" UNIQUE ($colList);"
            $uniqNum++
        }
        $sql += ""
    }
}

# ========== Write output ==========
$output = $sql -join "`n"
[System.IO.File]::WriteAllText($OutputFile, $output, [System.Text.UTF8Encoding]::new($false))

$tableCount = ($tableDefs.GetEnumerator() | Where-Object { $_.Value.Count -gt 0 }).Count
Write-Host "Generated SQL to $OutputFile"
Write-Host "Tables: $tableCount"
Write-Host "Enums: $($enumValues.Count)"
Write-Host "Total SQL lines: $($sql.Count)"
