# Fix all SQL migrations for PostgreSQL compatibility
$migrationsDir = "C:\Users\nommi\Downloads\scraper\communicator\backend-rs\migrations"
$fixed = 0

Get-ChildItem -Path $migrationsDir -Filter "*.sql" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $original = $content

    # 1. Fix duplicate DEFAULT: "DEFAULT NOW() DEFAULT (NOW())" -> "DEFAULT NOW()"
    $content = $content -replace 'DEFAULT NOW\(\) DEFAULT \(NOW\(\)\)', 'DEFAULT NOW()'

    # 2. Fix BLOB -> BYTEA
    $content = $content -replace '\bBLOB\b', 'BYTEA'

    # 3. Fix SQLite randomblob UUID generation -> gen_random_uuid()::TEXT
    # Pattern: DEFAULT (lower(hex(randomblob(4)) || '-' || ... ))
    $uuidPattern = 'DEFAULT \(lower\(hex\(randomblob\(4\)\) \|\| .*? \|\| hex\(randomblob\(6\)\)\)\)'
    $content = [regex]::Replace($content, $uuidPattern, 'DEFAULT gen_random_uuid()::TEXT', 'IgnoreCase')

    if ($content -ne $original) {
        Set-Content $_.FullName -Value $content -NoNewline -Encoding UTF8
        $fixed++
        Write-Host "Fixed: $($_.Name)"
    }
}

Write-Host "`nFixed $fixed migration files."
