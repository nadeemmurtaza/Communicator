# Database Backup & Restore

Complete backup and restore system for all project databases.

## Databases Covered

### PostgreSQL (localhost:5432)
| Database | Size | Description |
|----------|------|-------------|
| `communicator` | 30 MB | Main application database |
| `nid` | 13 MB | nID identity provider |
| `erp` | 22 MB | ERP system |
| `yaseenecom` | 15 MB | Yaseene.com e-commerce |
| + 12 others | ~170 MB | Test, dev, and auxiliary databases |

### SQLite
| Database | Size | Location |
|----------|------|----------|
| `communicator.db` | 0.95 MB | Project root (fallback/dev mode) |
| `desktop-v2.db` | 3.98 MB | AegisDesk project |

## Quick Start

### Backup everything
```powershell
.\backup-databases.ps1
```

### Backup only main databases
```powershell
.\backup-databases.ps1 -Databases communicator,nid,erp,yaseenecom
```

### Backup only PostgreSQL or SQLite
```powershell
.\backup-databases.ps1 -PostgresOnly
.\backup-databases.ps1 -SqliteOnly
```

### Restore from a backup
```powershell
# List available backups
.\restore-databases.ps1 -ListBackups

# Restore everything from a specific backup
.\restore-databases.ps1 -BackupDir backups\2025-11-11_1800

# Restore only specific databases
.\restore-databases.ps1 -BackupDir backups\2025-11-11_1800 -Databases communicator
```

## Backup Output Structure

```
backups\
└── 2025-11-11_1800\          # Timestamped folder
    ├── postgresql\            # Compressed SQL dumps
    │   ├── communicator.sql.gz
    │   ├── nid.sql.gz
    │   ├── erp.sql.gz
    │   └── ...
    └── sqlite\                # File copies
        ├── communicator.db
        └── desktop-v2.db
```

## Backup Strategy Recommendations

### 1. Manual Backups (Development)
Run before major changes:
```powershell
.\backup-databases.ps1 -Databases communicator,nid,erp,yaseenecom
```

### 2. Automated Daily Backups (Production)
Use Windows Task Scheduler to run daily:

1. Open **Task Scheduler** → **Create Task**
2. **Trigger**: Daily at your preferred time (e.g., 2:00 AM)
3. **Action**: Start a program
   - Program: `powershell.exe`
   - Arguments: `-ExecutionPolicy Bypass -File "C:\path\to\backup-databases.ps1" -Databases communicator,nid,erp,yaseenecom`
4. **Conditions**: Run only when AC power is on (optional)

### 3. Retention Policy
- The scripts keep all backups by default
- Manually delete old backups from the `backups/` folder
- Recommended: Keep last 10 backups (~250 MB total)

## Restore Notes

### PostgreSQL Restore
- **WARNING**: Restoring drops and recreates the database
- Always take a backup before restoring
- The restore script uses `DROP DATABASE IF EXISTS` + `CREATE DATABASE` + SQL import

### SQLite Restore
- Before overwriting, the current file is automatically backed up as:
  `communicator.db.pre-restore-20251111-180000.db`
- This provides a safety net in case the restore goes wrong

## Manual Backup (Alternative)

If the scripts don't work, you can back up manually:

### PostgreSQL
```powershell
# Single database
& "C:\Program Files\PostgreSQL\16\bin\pg_dump.exe" -U postgres -h 127.0.0.1 -d communicator -F c -f communicator.backup

# All databases
& "C:\Program Files\PostgreSQL\16\bin\pg_dumpall.exe" -U postgres -h 127.0.0.1 > all_databases.sql
```

### SQLite
```powershell
# Simply copy the .db file
Copy-Item communicator.db communicator.db.backup
```

## Troubleshooting

### "psql.exe not found"
Update the `$PgBin` path in the scripts to match your PostgreSQL installation.

### "Connection refused"
Ensure PostgreSQL is running:
```powershell
Get-Service postgresql*
```
Start it if stopped:
```powershell
Start-Service postgresql-x64-16
```

### Authentication failure
The scripts use peer/trust authentication via the `postgres` user. If your PostgreSQL requires a password, you'll need to set the `PGPASSWORD` environment variable before running:
```powershell
$env:PGPASSWORD = "your_password"
.\backup-databases.ps1
```
