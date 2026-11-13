# Apply all pending migrations to the communicator PostgreSQL database
$psql = "C:\Program Files\PostgreSQL\16\bin\psql.exe"
$migrationsDir = "C:\Users\nommi\Downloads\scraper\communicator\backend-rs\migrations"
$tempDir = "C:\Users\nommi\AppData\Local\Temp\junie\session-261106-070848-n18a"
$applied = 0
$failed = 0

$files = Get-ChildItem -Path $migrationsDir -Filter "00*.sql" | Sort-Object Name | Where-Object { $_.Name -notlike "0001_*" }

foreach ($file in $files) {
    Write-Host "Applying: $($file.Name) ..."

    $content = Get-Content $file.FullName -Raw

    # Wrap ALTER TABLE ADD COLUMN in safe DO blocks
    $safeContent = [regex]::Replace($content,
        'ALTER TABLE (\w+) ADD COLUMN (\w+) ([^;]+);',
        {
            param($m)
            $tbl = $m.Groups[1].Value
            $col = $m.Groups[2].Value
            $def = $m.Groups[3].Value
            $sql = "DO `$$`$$ BEGIN IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = '$tbl' AND column_name = '$col') THEN ALTER TABLE $tbl ADD COLUMN $col $def; END IF; END `$$`$$;"
            return $sql
        },
        'IgnoreCase'
    )

    $tempFile = Join-Path $tempDir "migration_safe.sql"
    Set-Content -Path $tempFile -Value $safeContent -NoNewline -Encoding UTF8

    $result = & $psql -h 127.0.0.1 -U nommi -d communicator -f $tempFile 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host "  OK" -ForegroundColor Green
        $applied++
    } else {
        if ($result -match "column.*already exists") {
            Write-Host "  WARN: column exists" -ForegroundColor Yellow
            $applied++
        } else {
            Write-Host "  FAIL:" -ForegroundColor Red
            Write-Host $result -ForegroundColor Red
            $failed++
        }
    }

    Remove-Item $tempFile -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "Applied: $applied, Failed: $failed"
