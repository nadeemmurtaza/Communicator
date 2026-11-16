# Fix enum defaults in generated ERP SQL
$SqlFile = "C:\Users\newax\IdeaProjects\Communicator\erp-schema\generated\0001_erp_schema.sql"

# Read all lines
$lines = [System.IO.File]::ReadAllLines($SqlFile)
Write-Host "Total lines: $($lines.Count)"

# Build enum map from CREATE TYPE statements
$enums = @{}
for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    if ($line -match 'CREATE TYPE\s+(\w+)\s+AS ENUM\s*\(([^)]*)\)') {
        $name = $matches[1].ToLower()
        $vals = ($matches[2] -split ',' | ForEach-Object { $_.Trim().Trim("'") })
        $enums[$name] = @{}
        foreach ($v in $vals) {
            $enums[$name][$v.ToUpper()] = $v
        }
    }
}
Write-Host "Enum types: $($enums.Count)"

# Fix DEFAULT 'UPPERCASE' -> DEFAULT 'lowercase'::enumtype
$count = 0
for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    if ($line -match '"(\w+)"\s+(\w+)\s+DEFAULT\s+''([A-Z_]+)''') {
        $col = $matches[1]
        $type = $matches[2].ToLower()
        $upperVal = $matches[3]
        if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) {
            $lowerVal = $enums[$type][$upperVal]
            $lines[$i] = $line -replace "DEFAULT\s+'${upperVal}'", "DEFAULT '$lowerVal'::$type"
            $count++
        }
    }
}
Write-Host "Fixed $count enum defaults"

# Write back
[System.IO.File]::WriteAllLines($SqlFile, $lines, [System.Text.UTF8Encoding]::new($false))
Write-Host "Done!"
