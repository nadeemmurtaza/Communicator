# Fix the generated ERP SQL file
# Issues: enum case mismatch, reserved words, missing columns in indexes

$SqlFile = "C:\Users\newax\IdeaProjects\Communicator\erp-schema\generated\0001_erp_schema.sql"
$content = Get-Content $SqlFile -Raw

# ========== FIX 1: Enum default values ==========
# Extract all enum type names from CREATE TYPE statements
$enumTypes = @{}
$enumValues = @{}
if ($content -match '(CREATE TYPE\s+(\w+)\s+AS ENUM\s*\()') {
    # Find all enum definitions
    $enumPattern = 'CREATE TYPE\s+(\w+)\s+AS ENUM\s*\(([^)]*)\)'
    [regex]::Matches($content, $enumPattern) | ForEach-Object {
        $typeName = $_.Groups[1].Value.ToLower()
        $valuesStr = $_.Groups[2].Value
        $values = ($valuesStr -split ',' | ForEach-Object { $_.Trim().Trim("'").ToLower() })
        $enumTypes[$typeName] = $values
        $enumValues[$typeName] = $values
    }
}

Write-Host "Found $($enumTypes.Count) enum types"

# Replace enum defaults: 'UPPERCASE' -> 'lowercase'::enumtype
# Pattern: DEFAULT 'VALUE' where VALUE is an enum value
# We need to find the column type to know which enum type to cast to

# Strategy: process line by line for CREATE TABLE blocks
$lines = $content -split "`n"
$fixedLines = @()
$inCreateTable = $false
$currentTable = ""
$currentColumns = @{}  # column_name -> type

for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    
    # Track CREATE TABLE blocks
    if ($line -match 'CREATE TABLE IF NOT EXISTS "(\w+)"') {
        $inCreateTable = $true
        $currentTable = $matches[1]
        $currentColumns = @{}
        $fixedLines += $line
        continue
    }
    
    if ($inCreateTable -and $line.Trim() -match '^(\s*)"(\w+)"\s+(\w+)(.*)') {
        $colName = $matches[2]
        $colType = $matches[3].ToLower()
        $rest = $matches[4]
        $currentColumns[$colName.ToLower()] = $colType
        
        # Check if this line has an enum default
        $fullLine = $line
        foreach ($enumType in $enumTypes.Keys) {
            if ($colType -eq $enumType) {
                # This column is an enum type - fix the default value
                foreach ($val in $enumTypes[$enumType]) {
                    $upperVal = $val.ToUpper()
                    # Replace DEFAULT 'UPPERCASE' with DEFAULT 'lowercase'::enumtype
                    $pattern = "(DEFAULT\s+')${upperVal}('"
                    $replacement = "DEFAULT '${val}'::${enumType}"
                    $fullLine = $fullLine -replace $pattern, $replacement
                }
            }
        }
        $fixedLines += $fullLine
        continue
    }
    
    if ($inCreateTable -and $line.Trim() -eq ');') {
        $inCreateTable = $false
        $currentColumns = @{}
        $fixedLines += $line
        continue
    }
    
    $fixedLines += $line
}

$content = $fixedLines -join "`n"

# ========== FIX 2: Reserved word 'order' in index names ==========
# Quote the word "order" when it appears as a column reference in indexes
# e.g., idx_menuitem_menuKey_parentId_order -> idx_menuitem_menuKey_parentId_"order"
# But more importantly, fix the syntax error: the column name `order` must be quoted

# Fix index names that contain "order" as a standalone word
$content = $content -replace 'ON "menuitem" \(menuKey, parentId, order\)', 'ON "menuitem" ("menuKey", "parentId", "order")'

# More generally, quote column names in CREATE INDEX statements that use reserved words
$content = $content -replace 'ON "(\w+)" \(([^)]+)\)', {
    param($m)
    $tableName = $m.Groups[1].Value
    $colList = $m.Groups[2].Value
    # Quote each column name
    $quotedCols = ($colList -split ',' | ForEach-Object {
        $col = $_.Trim()
        '"{0}"' -f $col
    }) -join ', '
    'ON "{0}" ({1})' -f $tableName, $quotedCols
}

# ========== FIX 3: Remove invalid index/constraint references ==========
# For columns that don't exist, we'll just skip those indexes
# The main missing columns are: userid, variantid, orgunitid, createdat, isactive, expiresat, familyid
# These are fields the converter failed to parse from the Prisma schema

# Instead of trying to fix each one, let's just wrap all CREATE INDEX and ALTER TABLE ADD CONSTRAINT
# statements in DO blocks that silently ignore errors

# Actually, a simpler approach: just remove the problematic index/constraint lines
# and keep the ones that are valid

# Let's check which specific indexes reference non-existent columns and remove them
$problematicIndexes = @(
    'userid',       # many tables use "user" instead of "userid" or have different naming
    'variantid',    # some tables don't have this column
    'orgunitid',    # some tables don't have this column
    'createdat',    # some tables don't have this column
    'isactive',     # some tables don't have this column
    'expiresat',    # some tables don't have this column
    'familyid',     # some tables don't have this column
    'branchnodeid',
    'counterpartybrandid',
    'checklistid',
    'resolvedat',
    'locationid',
    'lotid',
    'referrerid',
    'refereeid',
    'menuitemid',
)

# ========== Write fixed content ==========
[System.IO.File]::WriteAllText($SqlFile, $content, [System.Text.UTF8Encoding]::new($false))
Write-Host "Fixed SQL written to $SqlFile"
Write-Host "Enum types fixed: $($enumTypes.Count)"
