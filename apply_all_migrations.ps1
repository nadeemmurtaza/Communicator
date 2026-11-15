# Apply all migrations to the communicator database
$env:PGPASSWORD = 'MNAnadeem..00'
$psql = "C:\Program Files\PostgreSQL\16\bin\psql.exe"
$migrationsDir = "C:\Users\newax\IdeaProjects\Communicator\backend-rs\migrations"
$db = "communicator"
$user = "communicator_app"

# Get all SQL migration files sorted by name
$migrations = Get-ChildItem -Path $migrationsDir -Filter "*.sql" | Sort-Object Name

$successCount = 0
$failCount = 0

foreach ($migration in $migrations) {
    $name = $migration.Name
    Write-Host "Applying $name..." -ForegroundColor Cyan

    $output = & $psql -U $user -d $db -f $migration.FullName 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host "  [OK] $name applied successfully" -ForegroundColor Green
        $successCount++
    } else {
        Write-Host "  [FAIL] $name FAILED" -ForegroundColor Red
        Write-Host "  Error: $output" -ForegroundColor Red
        $failCount++
    }
}

Write-Host ""
$sep = [string]::new('=', 40)
Write-Host $sep -ForegroundColor Yellow
Write-Host "Migration Summary:" -ForegroundColor Yellow
Write-Host "  Total: $($migrations.Count)" -ForegroundColor Yellow
Write-Host "  Success: $successCount" -ForegroundColor Green
Write-Host "  Failed: $failCount" -ForegroundColor Red
Write-Host $sep -ForegroundColor Yellow
