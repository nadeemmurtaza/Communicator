# Fix all foreign key type mismatches in migrations
# Tables from 0001 have UUID IDs, but later migrations use TEXT for FK columns
$migrationsDir = "C:\Users\nommi\Downloads\scraper\communicator\backend-rs\migrations"
$fixed = 0

# All tables from migration 0001 that have UUID primary keys
$uuidTables = @(
    "agent_insights", "agent_policies", "agent_retraining_events", "ai_connections",
    "ai_custom_policies", "audit_logs", "automation_rules", "broadcast_recipients",
    "broadcasts", "campaign_templates", "canned_responses", "channel_gateway_connections",
    "consent_records", "contact_memories", "contact_notes", "contact_segments",
    "contacts", "conversations", "copilot_approvals", "custom_attribute_definitions",
    "customers", "discovery_flow_templates", "drafts", "follow_up_sequences",
    "inboxes", "inquiries", "integration_settings", "issue_tracker_configs",
    "knowledge_base_entries", "knowledge_gaps", "knowledge_recommendations",
    "leads", "linked_issues", "macros", "messages", "role_definitions",
    "sequence_steps", "services", "simulation_runs", "simulation_scenarios",
    "sla_policies", "slack_conversation_syncs", "sso_configs", "stripe_billing_action_logs",
    "stripe_configs", "tags", "team_members", "team_memberships", "teams",
    "tenants", "ticket_activities", "tickets", "translation_settings",
    "webhook_delivery_logs", "webhook_endpoints"
)

Get-ChildItem -Path $migrationsDir -Filter "00*.sql" | Where-Object { $_.Name -notlike "0001_*" } | Sort-Object Name | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $original = $content

    foreach ($tbl in $uuidTables) {
        # Fix: REFERENCES <table>(id) where the referencing column is defined as TEXT
        # Pattern: column_name TEXT NOT NULL, ... FOREIGN KEY (column_name) REFERENCES <uuid_table>(id)
        # We need to find the column definition and change TEXT to UUID
        
        # Find all FOREIGN KEY references to this UUID table
        $pattern = "FOREIGN KEY \((\w+)\) REFERENCES $tbl\(id\)"
        $matches = [regex]::Matches($content, $pattern, 'IgnoreCase')
        
        foreach ($match in $matches) {
            $colName = $match.Groups[1].Value
            
            # Find the column definition and change TEXT to UUID
            # Pattern: column_name TEXT (with optional constraints after)
            $colPattern = "(\b$colName\s+)TEXT(\s+NOT\s+NULL\b|\s+DEFAULT\b|\s+UNIQUE\b|,|\))"
            $content = [regex]::Replace($content, $colPattern, "`$1UUID`$2", 'IgnoreCase')
        }
    }

    if ($content -ne $original) {
        Set-Content $_.FullName -Value $content -NoNewline -Encoding UTF8
        $fixed++
        Write-Host "Fixed: $($_.Name)"
    }
}

Write-Host "`nFixed $fixed migration files."
