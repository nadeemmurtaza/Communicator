# Communicator - Database Backup Script
# Backs up all PostgreSQL and SQLite databases to a timestamped folder.

param(
    [switch]$PostgresOnly,
    [switch]$SqliteOnly,
    [string]$Databases,
    [string]$BackupRoot = "backups"
)

$ErrorActionPreference = "Stop"
$Timestamp = Get-Date -Format "yyyy-MM-dd_HHmm"
$BackupDir = Join-Path $PSScriptRoot (Join-Path $BackupRoot $Timestamp)

$PgBin = "C:\Program Files\PostgreSQL\16\bin"
$PgHost = "127.0.0.1"
$PgPort = "5432"
$PgUser = "postgres"

$SqliteFiles = @(
    @{ Name = "communicator.db"; Path = Join-Path $PSScriptRoot "communicator.db" },
    @{ Name = "desktop-v2.db"; Path = "C:\Users\nommi\Downloads\scraper\AegisDesk\.freebuff\desktop-v2.db" }
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Database Backup - $Timestamp" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$PgBackupDir = Join-Path $BackupDir "postgresql"
$SqliteBackupDir = Join-Path $BackupDir "sqlite"

if (-not $SqliteOnly) {
    New-Item -ItemType Directory -Path $PgBackupDir -Force | Out-Null
}
if (-not $PostgresOnly) {
    New-Item -ItemType Directory -Path $SqliteBackupDir -Force | Out-Null
}

# PostgreSQL Backup
if (-not $SqliteOnly) {
    Write-Host "[PostgreSQL] Backing up databases..." -ForegroundColor Yellow

    if ($Databases) {
        $DbList = $Databases -split "," | ForEach-Object { $_.Trim() }
    } else {
        $DbListResult = & "$PgBin\psql.exe" -U $PgUser -h $PgHost -t -c "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname;"
        $DbList = $DbListResult.Trim() -split "`n" | Where-Object { $_.Trim() -ne "" }
    }

    $PgSuccess = 0
    $PgFailed = 0

    foreach ($DbName in $DbList) {
        $DbName = $DbName.Trim()
        if ($DbName -eq "") { continue }

        $GzFile = Join-Path $PgBackupDir "$DbName.sql.gz"

        Write-Host "  Backing up: $DbName ..." -NoNewline

        try {
            $dumpResult = & "$PgBin\pg_dump.exe" -U $PgUser -h $PgHost -p $PgPort --format=plain --clean --if-exists --create --no-owner --no-privileges -d $DbName 2>&1

            $sqlBytes = [System.Text.Encoding]::UTF8.GetBytes($dumpResult)
            $fs = [System.IO.File]::Open($GzFile, [System.IO.FileMode]::Create)
            $gzipStream = [System.IO.Compression.GzipStream]::new($fs, [System.IO.Compression.CompressionLevel]::Optimal)
            $gzipStream.Write($sqlBytes, 0, $sqlBytes.Length)
            $gzipStream.Close()
            $gzipStream.Dispose()
            $fs.Close()
            $fs.Dispose()

            $Size = [math]::Round((Get-Item $GzFile).Length / 1MB, 2)
            Write-Host " Done ($Size MB compressed)" -ForegroundColor Green
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

# SQLite Backup
if (-not $PostgresOnly) {
    Write-Host ""
    Write-Host "[SQLite] Backing up databases..." -ForegroundColor Yellow

    $SqliteSuccess = 0
    $SqliteFailed = 0

    foreach ($Db in $SqliteFiles) {
        $SrcPath = $Db.Path
        $DstFile = Join-Path $SqliteBackupDir $Db.Name

        Write-Host "  Backing up: $($Db.Name) ..." -NoNewline

        try {
            if (Test-Path $SrcPath) {
                Copy-Item $SrcPath $DstFile -Force
                $Size = [math]::Round((Get-Item $DstFile).Length / 1MB, 2)
                Write-Host " Done ($Size MB)" -ForegroundColor Green
                $SqliteSuccess++
            } else {
                Write-Host " SKIPPED (file not found)" -ForegroundColor DarkYellow
                $SqliteFailed++
            }
        }
        catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
            $SqliteFailed++
        }
    }

    Write-Host ""
    Write-Host "  SQLite summary: $SqliteSuccess succeeded, $SqliteFailed failed" -ForegroundColor Cyan
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Backup complete!" -ForegroundColor Green
Write-Host "  Location: $BackupDir" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$Backups = Get-ChildItem -Path (Join-Path $PSScriptRoot $BackupRoot) -Directory -ErrorAction SilentlyContinue | Sort-Object Name
if ($Backups.Count -gt 10) {
    $ToRemove = $Backups.Count - 10
    Write-Host "  [INFO] $ToRemove old backup(s) exist. Consider cleaning up." -ForegroundColor DarkYellow
}