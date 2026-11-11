# Communicator - Database Restore Script
# Restores databases from a backup created by backup-databases.ps1.

param(
    [string]$BackupDir,
    [string]$Databases,
    [switch]$PostgresOnly,
    [switch]$SqliteOnly,
    [switch]$ListBackups
)

$ErrorActionPreference = "Stop"

$PgBin = "C:\Program Files\PostgreSQL\16\bin"
$PgHost = "127.0.0.1"
$PgPort = "5432"
$PgUser = "postgres"

$SqliteTargets = @(
    @{ Name = "communicator.db"; Path = Join-Path $PSScriptRoot "communicator.db" },
    @{ Name = "desktop-v2.db"; Path = "C:\Users\nommi\Downloads\scraper\AegisDesk\.freebuff\desktop-v2.db" }
)

# List available backups
if ($ListBackups) {
    $BackupRoot = Join-Path $PSScriptRoot "backups"

    if (-not (Test-Path $BackupRoot)) {
        Write-Host "No backups found." -ForegroundColor Red
        exit 1
    }

    $Backups = Get-ChildItem -Path $BackupRoot -Directory | Sort-Object Name -Descending

    if ($Backups.Count -eq 0) {
        Write-Host "No backups found." -ForegroundColor Yellow
        exit 0
    }

    Write-Host "Available backups (newest first):" -ForegroundColor Cyan
    Write-Host ""

    foreach ($Bk in $Backups) {
        $PgCount = (Get-ChildItem -Path (Join-Path $Bk.FullName "postgresql") -Filter "*.sql.gz" -ErrorAction SilentlyContinue).Count
        $SqliteCount = (Get-ChildItem -Path (Join-Path $Bk.FullName "sqlite") -Filter "*.db" -ErrorAction SilentlyContinue).Count
        $TotalSize = [math]::Round((Get-ChildItem -Path $Bk.FullName -Recurse -File | Measure-Object -Property Length -Sum).Sum / 1MB, 2)

        Write-Host "  $($Bk.Name)" -ForegroundColor White
        Write-Host "    PostgreSQL: $PgCount databases | SQLite: $SqliteCount databases | Total: $TotalSize MB" -ForegroundColor Gray
    }

    exit 0
}

# Validate backup directory
if (-not $BackupDir) {
    Write-Host "Error: -BackupDir is required." -ForegroundColor Red
    Write-Host "Use -ListBackups to see available backups." -ForegroundColor Yellow
    exit 1
}

if (-not ([System.IO.Path]::IsPathRooted($BackupDir))) {
    $BackupDir = Join-Path $PSScriptRoot $BackupDir
}

if (-not (Test-Path $BackupDir)) {
    Write-Host "Error: Backup directory not found: $BackupDir" -ForegroundColor Red
    exit 1
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Database Restore" -ForegroundColor Cyan
Write-Host "  Source: $BackupDir" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# PostgreSQL Restore
if (-not $SqliteOnly) {
    $PgBackupDir = Join-Path $BackupDir "postgresql"

    if (-not (Test-Path $PgBackupDir)) {
        Write-Host "[PostgreSQL] No PostgreSQL backup found in this folder." -ForegroundColor Yellow
    }
    else {
        Write-Host "[PostgreSQL] Restoring databases..." -ForegroundColor Yellow

        $GzFiles = Get-ChildItem -Path $PgBackupDir -Filter "*.sql.gz"

        if ($Databases) {
            $DbNames = $Databases -split "," | ForEach-Object { $_.Trim() }
            $GzFiles = $GzFiles | Where-Object {
                $Name = [System.IO.Path]::GetFileNameWithoutExtension($_.Name) -replace '\.sql$', ''
                $DbNames -contains $Name
            }
        }

        $PgSuccess = 0
        $PgFailed = 0

        foreach ($GzFile in $GzFiles) {
            $DbName = [System.IO.Path]::GetFileNameWithoutExtension($GzFile.Name) -replace '\.sql$', ''

            Write-Host "  Restoring: $DbName ..." -NoNewline

            try {
                $fs = [System.IO.File]::OpenRead($GzFile.FullName)
                $gzipStream = [System.IO.Compression.GzipStream]::new($fs, [System.IO.Compression.CompressionMode]::Decompress)
                $streamReader = New-Object System.IO.StreamReader($gzipStream, [System.Text.Encoding]::UTF8)
                $sqlContent = $streamReader.ReadToEnd()
                $streamReader.Close()
                $gzipStream.Close()
                $fs.Close()

                & "$PgBin\psql.exe" -U $PgUser -h $PgHost -p $PgPort -d postgres -c "DROP DATABASE IF EXISTS $DbName;" 2>&1 | Out-Null
                & "$PgBin\psql.exe" -U $PgUser -h $PgHost -p $PgPort -d postgres -c "CREATE DATABASE $DbName;" 2>&1 | Out-Null
                $sqlContent | & "$PgBin\psql.exe" -U $PgUser -h $PgHost -p $PgPort -d $DbName 2>&1 | Out-Null

                Write-Host " Done" -ForegroundColor Green
                $PgSuccess++
            }
            catch {
                Write-Host " FAILED: $_" -ForegroundColor Red
                $PgFailed++
            }
        }

        Write-Host ""
        Write-Host "  PostgreSQL summary: $PgSuccess succeeded, $PgFailed failed" -ForegroundColor Cyan
    }
}

# SQLite Restore
if (-not $PostgresOnly) {
    $SqliteBackupDir = Join-Path $BackupDir "sqlite"

    if (-not (Test-Path $SqliteBackupDir)) {
        Write-Host "[SQLite] No SQLite backup found in this folder." -ForegroundColor Yellow
    }
    else {
        Write-Host ""
        Write-Host "[SQLite] Restoring databases..." -ForegroundColor Yellow

        $SqliteSuccess = 0
        $SqliteFailed = 0

        foreach ($Db in $SqliteTargets) {
            $SrcFile = Join-Path $SqliteBackupDir $Db.Name

            if (-not (Test-Path $SrcFile)) {
                Write-Host "  SKIPPED: $($Db.Name) (not in backup)" -ForegroundColor DarkYellow
                continue
            }

            Write-Host "  Restoring: $($Db.Name) ..." -NoNewline

            try {
                $CurrentPath = $Db.Path
                if (Test-Path $CurrentPath) {
                    $BackupName = "$($Db.Name).pre-restore-$(Get-Date -Format 'yyyyMMdd-HHmmss').db"
                    Copy-Item $CurrentPath (Join-Path (Split-Path $CurrentPath -Parent) $BackupName) -Force
                }

                Copy-Item $SrcFile $CurrentPath -Force
                Write-Host " Done" -ForegroundColor Green
                $SqliteSuccess++
            }
            catch {
                Write-Host " FAILED: $_" -ForegroundColor Red
                $SqliteFailed++
            }
        }

        Write-Host ""
        Write-Host "  SQLite summary: $SqliteSuccess succeeded, $SqliteFailed failed" -ForegroundColor Cyan
    }
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Restore complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan