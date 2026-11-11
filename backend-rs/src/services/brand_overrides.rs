//! Brand Overrides in Workflows — Phase 27
//!
//! Enables dynamic brand switching mid-conversation or mid-workflow execution.
//! Mirrors Intercom's "Brand overrides in workflows" capability where a
//! conversation can switch brands mid-flow (e.g., from Coke to Pepsi sub-brand).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrandOverrideRule {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: Option<Value>,
    pub from_brand_id: Option<String>,
    pub from_brand_name: Option<String>,
    pub to_brand_id: String,
    pub to_brand_name: Option<String>,
    pub conditions: Option<Value>,
    pub is_active: bool,
    pub auto_switch_kb: bool,
    pub auto_switch_ai_config: bool,
    pub auto_switch_messenger: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrandSwitchLog {
    pub id: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub from_brand_id: String,
    pub from_brand_name: Option<String>,
    pub to_brand_id: String,
    pub to_brand_name: Option<String>,
    pub trigger_type: String,
    pub trigger_source: Option<String>,
    pub rule_id: Option<String>,
    pub switched_by: Option<String>,
    pub switched_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrandOverrideSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub allow_cross_org_brands: bool,
    pub require_approval: bool,
    pub log_all_switches: bool,
    pub default_fallback_brand_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrandHierarchy {
    pub id: String,
    pub parent_brand_id: String,
    pub parent_brand_name: Option<String>,
    pub sub_brand_id: String,
    pub sub_brand_name: Option<String>,
    pub relationship_type: String,
    pub priority: i32,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrandOverrideSummary {
    pub total_rules: i32,
    pub active_rules: i32,
    pub total_switches_7d: i32,
    pub total_switches_30d: i32,
    pub enabled: bool,
    pub top_switched_brands: Vec<Value>,
    pub recent_switches: Vec<BrandSwitchLog>,
    pub hierarchy_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwitchBrandResult {
    pub switch_id: String,
    pub conversation_id: String,
    pub from_brand_id: String,
    pub to_brand_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuleResult {
    pub rule_id: String,
    pub name: String,
    pub message: String,
}

// ——— Core Functions ———

/// Get brand override summary dashboard data
pub async fn get_brand_override_summary(state: &AppState) -> Result<BrandOverrideSummary, sqlx::Error> {
    // Total rules
    let total_rules: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM brand_override_rules")
        .fetch_one(&state.db)
        .await?;

    // Active rules
    let active_rules: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM brand_override_rules WHERE is_active = true")
        .fetch_one(&state.db)
        .await?;

    // Total switches in last 7 days
    let total_switches_7d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM brand_switch_log WHERE switched_at >= NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total switches in last 30 days
    let total_switches_30d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM brand_switch_log WHERE switched_at >= NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&state.db)
    .await?;

    // Settings
    let settings_row = sqlx::query(
        "SELECT enabled FROM brand_override_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;
    let enabled = settings_row
        .map(|r| r.try_get::<bool, _>(0).unwrap_or(true))
        .unwrap_or(true);

    // Hierarchy count
    let hierarchy_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM brand_hierarchy")
        .fetch_one(&state.db)
        .await?;

    // Top switched-to brands
    let top_brands_rows = sqlx::query(
        "SELECT b.name, COUNT(bsl.id) as cnt FROM brand_switch_log bsl \
         JOIN brands b ON bsl.to_brand_id = b.id \
         GROUP BY b.id, b.name ORDER BY cnt DESC LIMIT 5"
    )
    .fetch_all(&state.db)
    .await?;

    let top_switched_brands: Vec<Value> = top_brands_rows
        .iter()
        .map(|row| {
            json!({
                "brand_name": row.try_get::<String, _>(0).unwrap_or_default(),
                "switch_count": row.try_get::<i64, _>(1).unwrap_or(0i64),
            })
        })
        .collect();

    // Recent switches
    let switch_rows = sqlx::query(
        "SELECT bsl.id, bsl.conversation_id, bsl.contact_id, bsl.from_brand_id, \
         fb.name as from_brand_name, bsl.to_brand_id, tb.name as to_brand_name, \
         bsl.trigger_type, bsl.trigger_source, bsl.rule_id, bsl.switched_by, bsl.switched_at \
         FROM brand_switch_log bsl \
         LEFT JOIN brands fb ON bsl.from_brand_id = fb.id \
         LEFT JOIN brands tb ON bsl.to_brand_id = tb.id \
         ORDER BY bsl.switched_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_switches: Vec<BrandSwitchLog> = switch_rows
        .iter()
        .map(|row| BrandSwitchLog {
            id: row.try_get(0).unwrap_or_default(),
            conversation_id: row.try_get(1).unwrap_or_default(),
            contact_id: row.try_get(2).unwrap_or_default(),
            from_brand_id: row.try_get(3).unwrap_or_default(),
            from_brand_name: row.try_get(4).ok(),
            to_brand_id: row.try_get(5).unwrap_or_default(),
            to_brand_name: row.try_get(6).ok(),
            trigger_type: row.try_get(7).unwrap_or_default(),
            trigger_source: row.try_get(8).ok(),
            rule_id: row.try_get(9).ok(),
            switched_by: row.try_get(10).ok(),
            switched_at: row.try_get(11).unwrap_or_default(),
        })
        .collect();

    Ok(BrandOverrideSummary {
        total_rules,
        active_rules,
        total_switches_7d,
        total_switches_30d,
        enabled,
        top_switched_brands,
        recent_switches,
        hierarchy_count,
    })
}

/// List brand override rules
pub async fn list_brand_override_rules(
    state: &AppState,
    organization_id: Option<&str>,
    is_active: Option<bool>,
    limit: i32,
    offset: i32,
) -> Result<Vec<BrandOverrideRule>, sqlx::Error> {
    let limit = limit.max(1).min(100);
    let offset = offset.max(0);

    let mut conditions = Vec::new();
    let mut param_count = 1;

    if let Some(org) = organization_id {
        conditions.push(format!("organization_id = ${}", param_count));
        param_count += 1;
    }
    if let Some(active) = is_active {
        conditions.push(format!("is_active = ${}", param_count));
        param_count += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query = format!(
        "SELECT bor.id, bor.organization_id, bor.name, bor.description, bor.trigger_type, \
         bor.trigger_config, bor.from_brand_id, fb.name as from_brand_name, \
         bor.to_brand_id, tb.name as to_brand_name, bor.conditions, bor.is_active, \
         bor.auto_switch_kb, bor.auto_switch_ai_config, bor.auto_switch_messenger, \
         bor.created_at, bor.updated_at \
         FROM brand_override_rules bor \
         LEFT JOIN brands fb ON bor.from_brand_id = fb.id \
         LEFT JOIN brands tb ON bor.to_brand_id = tb.id \
         {} \
         ORDER BY bor.created_at DESC \
         LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );

    let mut query_builder = sqlx::query(&query);

    if let Some(org) = organization_id {
        query_builder = query_builder.bind(org);
    }
    if let Some(active) = is_active {
        query_builder = query_builder.bind(active);
    }
    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(&state.db).await?;

    Ok(rows
        .iter()
        .map(|row| BrandOverrideRule {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            trigger_type: row.try_get(4).unwrap_or_default(),
            trigger_config: row.try_get(5).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            from_brand_id: row.try_get(6).ok(),
            from_brand_name: row.try_get(7).ok(),
            to_brand_id: row.try_get(8).unwrap_or_default(),
            to_brand_name: row.try_get(9).ok(),
            conditions: row.try_get(10).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            is_active: row.try_get(11).unwrap_or(true),
            auto_switch_kb: row.try_get(12).unwrap_or(true),
            auto_switch_ai_config: row.try_get(13).unwrap_or(true),
            auto_switch_messenger: row.try_get(14).unwrap_or(true),
            created_at: row.try_get(15).unwrap_or_default(),
            updated_at: row.try_get(16).unwrap_or_default(),
        })
        .collect())
}

/// Create a brand override rule
pub async fn create_brand_override_rule(
    state: &AppState,
    organization_id: &str,
    name: &str,
    trigger_type: &str,
    from_brand_id: Option<&str>,
    to_brand_id: &str,
    auto_switch_kb: bool,
    auto_switch_ai_config: bool,
    auto_switch_messenger: bool,
) -> Result<RuleResult, String> {
    // Verify to_brand exists
    let brand_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM brands WHERE id = $1)"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if !brand_exists {
        return Err(format!("Target brand {} not found", to_brand_id));
    }

    let rule_id = format!("brand-override-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO brand_override_rules (id, organization_id, name, trigger_type, \
         from_brand_id, to_brand_id, auto_switch_kb, auto_switch_ai_config, auto_switch_messenger) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&rule_id)
    .bind(organization_id)
    .bind(name)
    .bind(trigger_type)
    .bind(from_brand_id)
    .bind(to_brand_id)
    .bind(auto_switch_kb)
    .bind(auto_switch_ai_config)
    .bind(auto_switch_messenger)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to create rule: {}", e))?;

    Ok(RuleResult {
        rule_id,
        name: name.to_string(),
        message: "Brand override rule created".to_string(),
    })
}

/// Update a brand override rule
pub async fn update_brand_override_rule(
    state: &AppState,
    rule_id: &str,
    name: Option<&str>,
    is_active: Option<bool>,
    to_brand_id: Option<&str>,
) -> Result<(), String> {
    // Simpler approach with explicit string interpolation (values escaped)
    let mut parts = Vec::new();

    if let Some(n) = name {
        parts.push(format!("name = '${}'", n.replace('\'', "''")));
    }
    if let Some(active) = is_active {
        parts.push(format!("is_active = {}", active));
    }
    if let Some(to_brand) = to_brand_id {
        parts.push(format!("to_brand_id = '${}'", to_brand.replace('\'', "''")));
    }
    parts.push("updated_at = NOW()".to_string());

    if parts.is_empty() {
        return Ok(());
    }

    let query_str = format!(
        "UPDATE brand_override_rules SET {} WHERE id = '${}'",
        parts.join(", "),
        rule_id.replace('\'', "''")
    );

    sqlx::query(&query_str)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to update rule: {}", e))?;

    Ok(())
}

/// Delete a brand override rule
pub async fn delete_brand_override_rule(state: &AppState, rule_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM brand_override_rules WHERE id = $1")
        .bind(rule_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to delete rule: {}", e))?;

    Ok(())
}

/// Switch brand for a conversation
pub async fn switch_brand(
    state: &AppState,
    conversation_id: &str,
    contact_id: &str,
    from_brand_id: &str,
    to_brand_id: &str,
    trigger_type: &str,
    trigger_source: &str,
    rule_id: Option<&str>,
    switched_by: &str,
) -> Result<SwitchBrandResult, String> {
    // Check settings
    let settings = get_brand_override_settings(state).await;
    if !settings.as_ref().map(|s| s.enabled).unwrap_or(true) {
        return Err("Brand overrides are disabled".to_string());
    }

    // Verify conversation exists and get current brand
    let conv_row = sqlx::query(
        "SELECT brand_id FROM conversations WHERE id = $1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if conv_row.is_none() {
        return Err(format!("Conversation {} not found", conversation_id));
    }

    // Verify to_brand exists
    let brand_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM brands WHERE id = $1)"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if !brand_exists {
        return Err(format!("Target brand {} not found", to_brand_id));
    }

    // Update conversation's brand
    let switch_id = format!("brand-switch-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "UPDATE conversations SET brand_id = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(to_brand_id)
    .bind(conversation_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to switch brand: {}", e))?;

    // Log the switch
    sqlx::query(
        "INSERT INTO brand_switch_log (id, conversation_id, contact_id, from_brand_id, \
         to_brand_id, trigger_type, trigger_source, rule_id, switched_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&switch_id)
    .bind(conversation_id)
    .bind(contact_id)
    .bind(from_brand_id)
    .bind(to_brand_id)
    .bind(trigger_type)
    .bind(trigger_source)
    .bind(rule_id)
    .bind(switched_by)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to log switch: {}", e))?;

    Ok(SwitchBrandResult {
        switch_id,
        conversation_id: conversation_id.to_string(),
        from_brand_id: from_brand_id.to_string(),
        to_brand_id: to_brand_id.to_string(),
        message: format!("Brand switched from {} to {}", from_brand_id, to_brand_id),
    })
}

/// Get switch log for a conversation
pub async fn get_switch_log_for_conversation(
    state: &AppState,
    conversation_id: &str,
) -> Result<Vec<BrandSwitchLog>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT bsl.id, bsl.conversation_id, bsl.contact_id, bsl.from_brand_id, \
         fb.name as from_brand_name, bsl.to_brand_id, tb.name as to_brand_name, \
         bsl.trigger_type, bsl.trigger_source, bsl.rule_id, bsl.switched_by, bsl.switched_at \
         FROM brand_switch_log bsl \
         LEFT JOIN brands fb ON bsl.from_brand_id = fb.id \
         LEFT JOIN brands tb ON bsl.to_brand_id = tb.id \
         WHERE bsl.conversation_id = $1 \
         ORDER BY bsl.switched_at ASC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .iter()
        .map(|row| BrandSwitchLog {
            id: row.try_get(0).unwrap_or_default(),
            conversation_id: row.try_get(1).unwrap_or_default(),
            contact_id: row.try_get(2).unwrap_or_default(),
            from_brand_id: row.try_get(3).unwrap_or_default(),
            from_brand_name: row.try_get(4).ok(),
            to_brand_id: row.try_get(5).unwrap_or_default(),
            to_brand_name: row.try_get(6).ok(),
            trigger_type: row.try_get(7).unwrap_or_default(),
            trigger_source: row.try_get(8).ok(),
            rule_id: row.try_get(9).ok(),
            switched_by: row.try_get(10).ok(),
            switched_at: row.try_get(11).unwrap_or_default(),
        })
        .collect())
}

/// Get brand override settings
pub async fn get_brand_override_settings(
    state: &AppState,
) -> Result<Option<BrandOverrideSettings>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, allow_cross_org_brands, require_approval, \
         log_all_switches, default_fallback_brand_id, created_at, updated_at \
         FROM brand_override_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    Ok(row.map(|r| BrandOverrideSettings {
        id: r.try_get(0).unwrap_or_default(),
        organization_id: r.try_get(1).unwrap_or_default(),
        enabled: r.try_get(2).unwrap_or(true),
        allow_cross_org_brands: r.try_get(3).unwrap_or(false),
        require_approval: r.try_get(4).unwrap_or(false),
        log_all_switches: r.try_get(5).unwrap_or(true),
        default_fallback_brand_id: r.try_get(6).ok(),
        created_at: r.try_get(7).unwrap_or_default(),
        updated_at: r.try_get(8).unwrap_or_default(),
    }))
}

/// Update brand override settings
pub async fn update_brand_override_settings(
    state: &AppState,
    enabled: Option<bool>,
    allow_cross_org_brands: Option<bool>,
    require_approval: Option<bool>,
    log_all_switches: Option<bool>,
) -> Result<BrandOverrideSettings, String> {
    let mut sets = Vec::new();

    if let Some(v) = enabled {
        sets.push(format!("enabled = {}", v));
    }
    if let Some(v) = allow_cross_org_brands {
        sets.push(format!("allow_cross_org_brands = {}", v));
    }
    if let Some(v) = require_approval {
        sets.push(format!("require_approval = {}", v));
    }
    if let Some(v) = log_all_switches {
        sets.push(format!("log_all_switches = {}", v));
    }
    sets.push("updated_at = NOW()".to_string());

    if sets.is_empty() {
        return get_brand_override_settings(state).await.map_err(|e| e.to_string())?
            .ok_or("Settings not found".to_string());
    }

    let query = format!(
        "UPDATE brand_override_settings SET {} WHERE organization_id = 'org-default'",
        sets.join(", ")
    );

    sqlx::query(&query)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    get_brand_override_settings(state).await.map_err(|e| e.to_string())?
        .ok_or("Settings not found after update".to_string())
}

/// List brand hierarchy
pub async fn list_brand_hierarchy(state: &AppState) -> Result<Vec<BrandHierarchy>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT bh.id, bh.parent_brand_id, pb.name as parent_brand_name, \
         bh.sub_brand_id, sb.name as sub_brand_name, \
         bh.relationship_type, bh.priority, bh.created_at \
         FROM brand_hierarchy bh \
         LEFT JOIN brands pb ON bh.parent_brand_id = pb.id \
         LEFT JOIN brands sb ON bh.sub_brand_id = sb.id \
         ORDER BY bh.priority DESC, bh.created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .iter()
        .map(|row| BrandHierarchy {
            id: row.try_get(0).unwrap_or_default(),
            parent_brand_id: row.try_get(1).unwrap_or_default(),
            parent_brand_name: row.try_get(2).ok(),
            sub_brand_id: row.try_get(3).unwrap_or_default(),
            sub_brand_name: row.try_get(4).ok(),
            relationship_type: row.try_get(5).unwrap_or_default(),
            priority: row.try_get(6).unwrap_or(0i32),
            created_at: row.try_get(7).unwrap_or_default(),
        })
        .collect())
}

/// Add a brand hierarchy relationship
pub async fn add_brand_hierarchy(
    state: &AppState,
    parent_brand_id: &str,
    sub_brand_id: &str,
    relationship_type: &str,
    priority: i32,
) -> Result<(), String> {
    // Verify both brands exist
    let parent_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM brands WHERE id = $1)"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if !parent_exists {
        return Err(format!("Parent brand {} not found", parent_brand_id));
    }

    let sub_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM brands WHERE id = $1)"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if !sub_exists {
        return Err(format!("Sub-brand {} not found", sub_brand_id));
    }

    let id = format!("brand-hierarchy-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO brand_hierarchy (id, parent_brand_id, sub_brand_id, relationship_type, priority) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (parent_brand_id, sub_brand_id) \
         DO UPDATE SET relationship_type = $4, priority = $5"
    )
    .bind(&id)
    .bind(parent_brand_id)
    .bind(sub_brand_id)
    .bind(relationship_type)
    .bind(priority)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to add hierarchy: {}", e))?;

    Ok(())
}

/// Remove a brand hierarchy relationship
pub async fn remove_brand_hierarchy(state: &AppState, parent_brand_id: &str, sub_brand_id: &str) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM brand_hierarchy WHERE parent_brand_id = $1 AND sub_brand_id = $2"
    )
    .bind(parent_brand_id)
    .bind(sub_brand_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to remove hierarchy: {}", e))?;

    Ok(())
}

/// Get sub-brands for a parent brand
pub async fn get_sub_brands(
    state: &AppState,
    parent_brand_id: &str,
) -> Result<Vec<BrandHierarchy>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT bh.id, bh.parent_brand_id, pb.name as parent_brand_name, \
         bh.sub_brand_id, sb.name as sub_brand_name, \
         bh.relationship_type, bh.priority, bh.created_at \
         FROM brand_hierarchy bh \
         LEFT JOIN brands pb ON bh.parent_brand_id = pb.id \
         LEFT JOIN brands sb ON bh.sub_brand_id = sb.id \
         WHERE bh.parent_brand_id = $1 \
         ORDER BY bh.priority DESC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .iter()
        .map(|row| BrandHierarchy {
            id: row.try_get(0).unwrap_or_default(),
            parent_brand_id: row.try_get(1).unwrap_or_default(),
            parent_brand_name: row.try_get(2).ok(),
            sub_brand_id: row.try_get(3).unwrap_or_default(),
            sub_brand_name: row.try_get(4).ok(),
            relationship_type: row.try_get(5).unwrap_or_default(),
            priority: row.try_get(6).unwrap_or(0i32),
            created_at: row.try_get(7).unwrap_or_default(),
        })
        .collect())
}

/// Find matching override rules for a conversation
pub async fn find_matching_rules(
    state: &AppState,
    from_brand_id: &str,
    organization_id: &str,
) -> Result<Vec<BrandOverrideRule>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT bor.id, bor.organization_id, bor.name, bor.description, bor.trigger_type, \
         bor.trigger_config, bor.from_brand_id, fb.name as from_brand_name, \
         bor.to_brand_id, tb.name as to_brand_name, bor.conditions, bor.is_active, \
         bor.auto_switch_kb, bor.auto_switch_ai_config, bor.auto_switch_messenger, \
         bor.created_at, bor.updated_at \
         FROM brand_override_rules bor \
         LEFT JOIN brands fb ON bor.from_brand_id = fb.id \
         LEFT JOIN brands tb ON bor.to_brand_id = tb.id \
         WHERE bor.is_active = true \
         AND bor.organization_id = $1 \
         AND (bor.from_brand_id IS NULL OR bor.from_brand_id = $2) \
         ORDER BY bor.created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .iter()
        .map(|row| BrandOverrideRule {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            description: row.try_get(3).ok(),
            trigger_type: row.try_get(4).unwrap_or_default(),
            trigger_config: row.try_get(5).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            from_brand_id: row.try_get(6).ok(),
            from_brand_name: row.try_get(7).ok(),
            to_brand_id: row.try_get(8).unwrap_or_default(),
            to_brand_name: row.try_get(9).ok(),
            conditions: row.try_get(10).ok().and_then(|s: String| serde_json::from_str(&s).ok()),
            is_active: row.try_get(11).unwrap_or(true),
            auto_switch_kb: row.try_get(12).unwrap_or(true),
            auto_switch_ai_config: row.try_get(13).unwrap_or(true),
            auto_switch_messenger: row.try_get(14).unwrap_or(true),
            created_at: row.try_get(15).unwrap_or_default(),
            updated_at: row.try_get(16).unwrap_or_default(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_serialization() {
        let summary = BrandOverrideSummary {
            total_rules: 5,
            active_rules: 3,
            total_switches_7d: 10,
            total_switches_30d: 45,
            enabled: true,
            top_switched_brands: vec![],
            recent_switches: vec![],
            hierarchy_count: 2,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_rules\":5"));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_switch_result_serialization() {
        let result = SwitchBrandResult {
            switch_id: "switch-123".to_string(),
            conversation_id: "conv-456".to_string(),
            from_brand_id: "brand-a".to_string(),
            to_brand_id: "brand-b".to_string(),
            message: "Brand switched".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"switch_id\":\"switch-123\""));
        assert!(json.contains("\"to_brand_id\":\"brand-b\""));
    }

    #[test]
    fn test_rule_result_serialization() {
        let result = RuleResult {
            rule_id: "rule-123".to_string(),
            name: "Test Rule".to_string(),
            message: "Created".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"rule_id\":\"rule-123\""));
    }
}
