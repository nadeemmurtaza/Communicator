# ============================================================
# Run ALL migrations for Communicator and nID databases
# ============================================================
# This script applies all SQL migrations in order to PostgreSQL,
# then runs Prisma migrations for nID.
# ============================================================

$ErrorActionPreference = "Stop"
$psql = "C:\Program Files\PostgreSQL\16\bin\psql.exe"
$migrationsDir = "C:\Users\nommi\Downloads\scraper\communicator\backend-rs\migrations"
$tempDir = "C:\Users\nommi\AppData\Local\Temp\junie\migration_temp"
$nidDir = "C:\Users\nommi\Downloads\scraper\nID"

# Create temp directory
if (-not (Test-Path $tempDir)) {
    New-Item -ItemType Directory -Path $tempDir | Out-Null
}

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  COMMUNICATOR + nID MIGRATION RUNNER" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# --------------------------------------------------------
# Part 1: Check PostgreSQL is running
# --------------------------------------------------------
Write-Host "[1/4] Checking PostgreSQL connection..." -ForegroundColor Yellow
$testResult = & $psql -h 127.0.0.1 -U nommi -d postgres -c "SELECT 1;" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Cannot connect to PostgreSQL. Is it running?" -ForegroundColor Red
    Write-Host "Start PostgreSQL: pg_ctl start -D 'C:/Program Files/PostgreSQL/16/data'" -ForegroundColor Red
    exit 1
}
Write-Host "  PostgreSQL is running." -ForegroundColor Green
Write-Host ""

# --------------------------------------------------------
# Part 2: Apply Communicator migrations
# --------------------------------------------------------
Write-Host "[2/4] Applying Communicator migrations..." -ForegroundColor Yellow

# Get all migration files sorted by name
$files = Get-ChildItem -Path $migrationsDir -Filter "00*.sql" | Sort-Object Name
$applied = 0
$skipped = 0
$failed = 0

foreach ($file in $files) {
    Write-Host "  Applying: $($file.Name) ..." -NoNewline
    
    # Read content and make ALTER TABLE ADD COLUMN safe (idempotent)
    $content = Get-Content $file.FullName -Raw

    # Wrap ALTER TABLE ADD COLUMN in safe DO blocks
    $safeContent = [regex]::Replace($content,
        'ALTER TABLE (\w+) ADD COLUMN (\w+) ([^;]+);',
        {
            param($m)
            $tbl = $m.Groups[1].Value
            $col = $m.Groups[2].Value
            $def = $m.Groups[3].Value
            return @"
DO `$$`$$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = '$tbl' AND column_name = '$col'
    ) THEN
        ALTER TABLE $tbl ADD COLUMN $col $def;
    END IF;
END `$$`$$;
"@
        },
        'IgnoreCase'
    )

    # Also wrap CREATE TABLE in IF NOT EXISTS if not already
    $safeContent = [regex]::Replace($safeContent,
        'CREATE TABLE (\w+) \(',
        'CREATE TABLE IF NOT EXISTS $1 (',
        'IgnoreCase'
    )

    $tempFile = Join-Path $tempDir "migration_safe.sql"
    Set-Content -Path $tempFile -Value $safeContent -NoNewline -Encoding UTF8

    # Run psql capturing all output (stdout+stderr). Use Start-Process for proper redirect.
    $errFile = Join-Path $tempDir "migration_err.log"
    $outFile = Join-Path $tempDir "migration_out.log"
    
    $proc = Start-Process -FilePath $psql -ArgumentList "-h", "127.0.0.1", "-U", "nommi", "-d", "communicator", "-f", $tempFile, "-o", $outFile, "--no-psqlrc" -NoNewWindow -Wait -RedirectStandardError $errFile -PassThru
    $exitCode = $proc.ExitCode
    
    $stdout = ""
    $stderr = ""
    if (Test-Path $outFile) { $stdout = Get-Content $outFile -Raw }
    if (Test-Path $errFile) { $stderr = Get-Content $errFile -Raw }
    $allOutput = "$stdout $stderr"

    if ($exitCode -eq 0) {
        Write-Host " OK" -ForegroundColor Green
        $applied++
    } else {
        # Check common "already exists" errors
        if ($allOutput -match "already exists|does not exist.*column|column.*already exists|relation.*already exists") {
            Write-Host " SKIP (already applied)" -ForegroundColor DarkGray
            $skipped++
        } else {
            Write-Host " FAIL" -ForegroundColor Red
            $errLines = $stderr -split "`n" | Where-Object { $_ -match "ERROR|FATAL" } | Select-Object -First 3
            if ($errLines) {
                foreach ($line in $errLines) {
                    Write-Host "    $line" -ForegroundColor Red
                }
            } else {
                Write-Host "    $allOutput" -ForegroundColor Red
            }
            $failed++
        }
    }
    
    Remove-Item $errFile -ErrorAction SilentlyContinue
    Remove-Item $outFile -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "  Communicator migrations: $applied applied, $skipped skipped, $failed failed" -ForegroundColor Cyan
Write-Host ""

# --------------------------------------------------------
# Part 3: Apply nID Prisma migrations
# --------------------------------------------------------
Write-Host "[3/4] Applying nID Prisma migrations..." -ForegroundColor Yellow

# Check if nID directory exists
if (Test-Path "$nidDir\packages\db\prisma\schema.prisma") {
    Write-Host "  Running prisma generate..." -NoNewline
    Push-Location "$nidDir\packages\db"
    $genResult = npx prisma generate 2>&1
    $genCode = $LASTEXITCODE
    Pop-Location
    if ($genCode -eq 0) {
        Write-Host " OK" -ForegroundColor Green
    } else {
        Write-Host " FAIL" -ForegroundColor Red
        Write-Host "    $($genResult -join ' ')" -ForegroundColor Red
    }

    Write-Host "  Running prisma migrate deploy..." -NoNewline
    Push-Location "$nidDir\packages\db"
    $migrateResult = npx prisma migrate deploy 2>&1
    $migrateCode = $LASTEXITCODE
    Pop-Location
    if ($migrateCode -eq 0) {
        Write-Host " OK" -ForegroundColor Green
        Write-Host "    $($migrateResult | Select-Object -Last 2)" -ForegroundColor DarkGray
    } else {
        Write-Host " FAIL" -ForegroundColor Red
        Write-Host "    $($migrateResult -join ' ')" -ForegroundColor Red
    }

    # Run seed if needed
    Write-Host "  Running prisma db seed..." -NoNewline
    Push-Location "$nidDir\packages\db"
    $seedResult = npx prisma db seed 2>&1
    $seedCode = $LASTEXITCODE
    Pop-Location
    if ($seedCode -eq 0) {
        Write-Host " OK" -ForegroundColor Green
    } else {
        Write-Host " WARN (may already be seeded)" -ForegroundColor Yellow
    }
} else {
    Write-Host "  nID schema not found at: $nidDir\packages\db\prisma\schema.prisma" -ForegroundColor Yellow
    Write-Host "  Skipping nID migrations." -ForegroundColor Yellow
}

Write-Host ""

# --------------------------------------------------------
# Part 4: Summary
# --------------------------------------------------------
Write-Host "[4/4] Migration Summary" -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Communicator DB: communicator (PostgreSQL)" -ForegroundColor White
Write-Host "  Migrations applied: $applied" -ForegroundColor Green
Write-Host "  Migrations skipped: $skipped" -ForegroundColor DarkGray
Write-Host "  Migrations failed:  $failed" -ForegroundColor $(if ($failed -gt 0) { 'Red' } else { 'Green' })
Write-Host ""
Write-Host "  nID DB: nid (PostgreSQL via Prisma)" -ForegroundColor White
Write-Host "  Schema: $nidDir\packages\db\prisma\schema.prisma" -ForegroundColor White
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# Clean up temp files
Remove-Item $tempFile -ErrorAction SilentlyContinue

if ($failed -gt 0) {
    Write-Host "WARNING: $failed migration(s) failed. Check errors above." -ForegroundColor Red
    exit 1
} else {
    Write-Host "All migrations completed successfully!" -ForegroundColor Green
}
