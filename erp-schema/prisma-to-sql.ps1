# Prisma to SQL converter for ERP schema - v2
# Parses merged.prisma and generates SQL CREATE TABLE statements
# Handles: enums, models, fields, relations, indexes, unique constraints, defaults

$SchemaFile = "C:\Users\newax\IdeaProjects\Communicator\erp-schema\merged.prisma"
$OutputFile = "C:\Users\newax\IdeaProjects\Communicator\erp-schema\generated\0001_erp_schema.sql"

$null = New-Item -ItemType Directory -Path "C:\Users\newax\IdeaProjects\Communicator\erp-schema\generated" -Force

$rawContent = Get-Content $SchemaFile -Raw

# Strip block comments (/** ... */)
$rawContent = $rawContent -replace '/\*[\s\S]*?\*/', ''

# Strip single-line comments
$cleanContent = ($rawContent -split "`n" | ForEach-Object {
    $line = $_
    # Remove // comments but be careful with URLs
    $idx = $line.IndexOf('//')
    if ($idx -ge 0) {
        $line = $line.Substring(0, $idx)
    }
    $line
}) -join "`n"

$lines = $cleanContent -split "`n"

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
$modelMap = @{}  # Prisma @@map to table name

# ========== PASS 1: Extract enums ==========
$inEnum = $false
$currentEnum = ""
for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i].Trim()
    if ($line -match '^enum\s+(\w+)\s*\{') {
        $inEnum = $true
        $currentEnum = $matches[1]
        $enumValues[$currentEnum] = @()
        continue
    }
    if ($inEnum -and $line -eq '}') {
        $inEnum = $false
        $currentEnum = ""
        continue
    }
    if ($inEnum -and $line -match '^(\w+)') {
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
$inBlockComment = $false

for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i].Trim()
    if ([string]::IsNullOrWhiteSpace($line)) { continue }

    if ($line -match '^model\s+(\w+)\s*\{') {
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
        $opens = ($line -replace '[^{]', '').Length
        $closes = ($line -replace '[^}]', '').Length
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

        if ($line -match '^@@index\((\[.*?\])\)') {
            $currentModelIndexes += $matches[1]
            continue
        }
        if ($line -match '^@@unique\((\[.*?\])\)') {
            $currentModelUniques += $matches[1]
            continue
        }
        if ($line -match '^@@unique\(\[(.*?)\]\)') {
            $currentModelUniques += "[$matches[1]]"
            continue
        }
        if ($line -match '^@@map\("([^"]+)"\)') {
            $currentModelMap = $matches[1]
            continue
        }

        # Multi-line field: join continuation lines
        if ($line -match '^\w') {
            $currentFieldLines += $line
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

# Collect all table definitions first
$tableDefs = @{}
$fkConstraints = @{}

foreach ($modelName in $models.Keys) {
    $tableName = Get-TableName $modelName
    $columns = @()
    $fks = @()

    foreach ($fieldLine in $modelFields[$modelName]) {
        # Clean up the field line - remove trailing commas, extra whitespace
        $fieldLine = $fieldLine.Trim().TrimEnd(',').Trim()
        if ([string]::IsNullOrWhiteSpace($fieldLine)) { continue }

        # Skip lines that start with @@ (already handled)
        if ($fieldLine -match '^@@') { continue }

        # Parse: fieldName fieldType? [attributes...]
        # Handle types like String[], DateTime?, etc.
        # Also handle relation fields like: user User @relation(...)
        
        # Match field name and type
        if ($fieldLine -match '^(\w+)\s+(\w+)(\[\])?\??\s*(.*)') {
            $fieldName = $matches[1]
            $fieldType = $matches[2]
            $isArray = $matches[3] -ne ''
            $isOptional = $fieldLine -match '\?'
            $attrs = if ($matches[4]) { $matches[4].Trim() } else { '' }
        } else {
            continue
        }

        # Skip if type is a model name (relation field)
        if ($models.ContainsKey($fieldType)) {
            # Check if this relation field has `fields: [x]` — it means the FK column
            # is defined by another scalar field, not this one
            # Skip relation fields — they don't create columns themselves
            continue
        }

        # Convert type
        $sqlType = $typeMap[$fieldType]
        if (-not $sqlType) {
            if ($enumValues.ContainsKey($fieldType)) {
                $sqlType = $fieldType.ToLower()
            } else {
                # Unknown type — could be a custom type or Unsupported()
                # Default to TEXT for safety
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

        if ($attrs -match '@id') {
            $isPrimaryKey = $true
        }
        if ($attrs -match '@unique') {
            $isUnique = $true
        }

        # Handle @default
        if ($attrs -match '@default\(([^)]+)\)') {
            $defaultExpr = $matches[1].Trim()

            switch -Regex ($defaultExpr) {
                '^now\(\)$' {
                    $defaultValue = "CURRENT_TIMESTAMP"
                }
                '^true$' {
                    $defaultValue = "TRUE"
                }
                '^false$' {
                    $defaultValue = "FALSE"
                }
                '^\[.*\]$' {
                    $defaultValue = "'{}'"
                }
                '^"([^"]+)"$' {
                    $defaultValue = "'" + $matches[1].Replace("'", "''") + "'"
                }
                '^(\d+)$' {
                    $defaultValue = $matches[1]
                }
                '^(\d+\.\d+)$' {
                    $defaultValue = $matches[1]
                }
                'cuid\(\)' {
                    $defaultValue = "gen_random_uuid()"
                    $sqlType = "UUID"
                }
                'uuid\(\)' {
                    $defaultValue = "gen_random_uuid()"
                    $sqlType = "UUID"
                }
                Default {
                    # Check if it's an enum value
                    if ($enumValues.ContainsKey($defaultExpr)) {
                        $defaultValue = "'" + $defaultExpr.ToLower() + "'::$($fieldType.ToLower())"
                    } else {
                        $defaultValue = "'" + $defaultExpr.Replace("'", "''") + "'"
                    }
                }
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

    # @@index
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

    # @@unique
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
