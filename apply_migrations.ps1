# Apply all pending migrations to the communicator PostgreSQL database
$psql = "C:\Program Files\PostgreSQL\16\bin\psql.exe"
$migrationsDir = "C:\Users\nommi\Downloads\scraper\communicator\backend-rs\migrations"
$applied = 0
$failed = 0

# Get migration files in order, skip 0001 (already applied)
$files = Get-ChildItem -Path $migrationsDir -Filter "00*.sql" | Sort-Object Name | Where-Object { $_.Name -notlike "0001_*" }

foreach ($file in $files) {
    Write-Host "Applying: $($file.Name) ..."

    # Read the file and wrap ALTER TABLE ADD COLUMN in safe DO blocks
    $content = Get-Content $file.FullName -Raw

    # Replace "ALTER TABLE X ADD COLUMN Y" with a safe version that checks first
    # Pattern: ALTER TABLE <table> ADD COLUMN <col> <rest of definition>;
    $safeContent = [regex]::Replace($content,
        'ALTER TABLE (\w+) ADD COLUMN (\w+) ([^;]+);',
        {
            param($match)
            $table = $match.Groups[1].Value
            $col = $match.Groups[2].Value
            $def = $match.Groups[3].Value
            return @"
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = '$table' AND column_name = '$col'
    ) THEN
        ALTER TABLE $table ADD COLUMN $col $def;
    END IF;
END $$;
"@
        },
        'IgnoreCase'
    )

    # Write the safe content to a temp file
    $tempFile = "C:\Users\nommi\AppData\Local\Temp\junie\session-261106-070848-n18a\migration_safe.sql"
    Set-Content -Path $tempFile -Value $safeContent -NoNewline -Encoding UTF8

    # Execute the migration
    $result = & $psql -h 127.0.0.1 -U nommi -d communicator -f $tempFile 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host "  ✓ Applied successfully" -ForegroundColor Green
        $applied++
    } else {
        # Check if it's just a "column already exists" error (we handled it, so this is something else)
        if ($result -match "column.*already exists") {
            Write-Host "  ⚠ Column already exists (skipped)" -ForegroundColor Yellow
            $applied++
        } else {
            Write-Host "  ✗ Failed:" -ForegroundColor Red
            Write-Host $result -ForegroundColor Red
            $failed++
        }
    }

    # Clean up temp file
    Remove-Item $tempFile -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "Applied: $applied | Failed: $failed"
