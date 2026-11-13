# Comprehensive SQLite → PostgreSQL Migration Fix Script
# Fixes all remaining SQLite-specific syntax in SQL migrations

$migrationsDir = "C:\Users\nommi\Downloads\scraper\communicator\backend-rs\migrations"
$fixed = 0

Get-ChildItem -Path $migrationsDir -Filter "*.sql" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $original = $content
    
    # 1. INTEGER DEFAULT 1 → BOOLEAN DEFAULT true (must be before INTEGER DEFAULT 0)
    $content = $content -replace 'INTEGER\s+DEFAULT\s+1\b', 'BOOLEAN DEFAULT true'
    
    # 2. INTEGER DEFAULT 0 → BOOLEAN DEFAULT false
    $content = $content -replace 'INTEGER\s+DEFAULT\s+0\b', 'BOOLEAN DEFAULT false'
    
    # 3. INTEGER NOT NULL DEFAULT 1 → BOOLEAN NOT NULL DEFAULT true
    $content = $content -replace 'INTEGER\s+NOT\s+NULL\s+DEFAULT\s+1\b', 'BOOLEAN NOT NULL DEFAULT true'
    
    # 4. INTEGER NOT NULL DEFAULT 0 → BOOLEAN NOT NULL DEFAULT false
    $content = $content -replace 'INTEGER\s+NOT\s+NULL\s+DEFAULT\s+0\b', 'BOOLEAN NOT NULL DEFAULT false'
    
    # 5. strftime('%Y-%m-%dT%H:%M:%fZ', 'now') → NOW()
    $content = $content -replace "strftime\('%Y-%m-%dT%H:%M:%fZ',\s*'now'\)", 'NOW()'
    
    # 6. strftime('%Y-%m-%d', → DATE( (for date-only patterns)
    $content = $content -replace "strftime\('%Y-%m-%d'", 'DATE('
    
    # 7. INSERT OR IGNORE → INSERT (will add ON CONFLICT per-file if needed)
    $content = $content -replace 'INSERT\s+OR\s+IGNORE', 'INSERT'
    
    # 8. TEXT NOT NULL (for timestamp columns) → TIMESTAMPTZ NOT NULL DEFAULT NOW()
    # Only fix columns named created_at, updated_at, synced_at, etc.
    $content = $content -replace '(\s+)(created_at|updated_at|synced_at|verified_at|paid_at|issued_at|due_at|period_start|period_end|current_period_start|current_period_end|last_synced_at|recorded_at|resolved_at|paid_at|expires_at|starts_at|ends_at)\s+TEXT\s+NOT\s+NULL', '$1$2 TIMESTAMPTZ NOT NULL DEFAULT NOW()'
    
    # 9. TEXT (for nullable timestamp columns)
    $content = $content -replace '(\s+)(verified_at|paid_at|issued_at|due_at|resolved_at|last_synced_at|recorded_at|transcribed_at|started_at|stopped_at)\s+TEXT(?!\s+NOT\s+NULL)', '$1$2 TIMESTAMPTZ'
    
    # 10. sqlite_master → information_schema.tables
    $content = $content -replace "sqlite_master\s+WHERE\s+type\s*=\s*'table'\s+AND\s+name\s*=\s*'", "information_schema.tables WHERE table_schema = 'public' AND table_name = '"
    
    # 11. WITHOUT ROWID (SQLite-specific, remove for PostgreSQL)
    $content = $content -replace '\s*WITHOUT\s+ROWID\b', ''
    
    # 12. AUTOINCREMENT (SQLite-specific, PostgreSQL uses SERIAL)
    $content = $content -replace 'INTEGER\s+PRIMARY\s+KEY\s+AUTOINCREMENT', 'SERIAL PRIMARY KEY'
    
    if ($content -ne $original) {
        Set-Content $_.FullName -Value $content -NoNewline
        $fixed++
        Write-Host "Fixed: $($_.Name)"
    }
}

Write-Host "`nFixed $fixed migration files."
