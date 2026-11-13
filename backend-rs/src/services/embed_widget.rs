use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::models::EmbedWidgetConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedWidgetSummary {
    pub total_configs: i64,
    pub active_configs: i64,
    pub total_instances: i64,
    pub active_instances: i64,
    pub total_events: i64,
    pub events_by_type: Vec<EventTypeCount>,
    pub top_domains: Vec<DomainCount>,
    pub recent_events: Vec<EmbedWidgetEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct DomainCount {
    pub domain: String,
    pub instances: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct EmbedWidgetEvent {
    pub id: String,
    pub instance_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct EmbedWidgetSettings {
    pub id: String,
    pub org_id: String,
    pub enabled: bool,
    pub allow_custom_domains: bool,
    pub max_instances_per_domain: i32,
    pub analytics_enabled: bool,
    pub default_language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedScriptResult {
    pub script: String,
    pub config_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateConfigResult {
    pub config: EmbedWidgetConfig,
    pub script: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicWidgetData {
    pub widget_type: String,
    pub settings: serde_json::Value,
    pub styles: serde_json::Value,
    pub trigger_config: serde_json::Value,
    pub help_center: Option<serde_json::Value>,
    pub default_language: String,
}

pub struct EmbedWidgetService {
    pub pool: PgPool,
}

impl EmbedWidgetService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn get_summary(&self) -> Result<EmbedWidgetSummary, anyhow::Error> {
        let total_configs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM embed_widget_configs").fetch_one(&self.pool).await?;
        let active_configs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM embed_widget_configs WHERE status = 'active'").fetch_one(&self.pool).await?;
        let total_instances: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM embed_widget_instances").fetch_one(&self.pool).await?;
        let active_instances: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM embed_widget_instances WHERE status = 'active'").fetch_one(&self.pool).await?;
        let total_events: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM embed_widget_events").fetch_one(&self.pool).await?;

        let events_by_type = sqlx::query_as::<_, EventTypeCount>(
            "SELECT event_type, COUNT(*) as count FROM embed_widget_events GROUP BY event_type ORDER BY count DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;

        let top_domains = sqlx::query_as::<_, DomainCount>(
            "SELECT domain, COUNT(*) as instances FROM embed_widget_instances GROUP BY domain ORDER BY instances DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;

        let recent_events = sqlx::query_as::<_, EmbedWidgetEvent>(
            "SELECT id, instance_id, event_type, payload, created_at FROM embed_widget_events ORDER BY created_at DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;

        Ok(EmbedWidgetSummary {
            total_configs: total_configs.0,
            active_configs: active_configs.0,
            total_instances: total_instances.0,
            active_instances: active_instances.0,
            total_events: total_events.0,
            events_by_type,
            top_domains,
            recent_events,
        })
    }

    pub async fn list_configs(&self, org_id: &str, status: Option<&str>) -> Result<Vec<EmbedWidgetConfig>, anyhow::Error> {
        let configs = if let Some(status) = status {
            sqlx::query_as::<_, EmbedWidgetConfig>(
                "SELECT * FROM embed_widget_configs WHERE org_id = $1 AND status = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(status).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as::<_, EmbedWidgetConfig>(
                "SELECT * FROM embed_widget_configs WHERE org_id = $1 ORDER BY created_at DESC"
            ).bind(org_id).fetch_all(&self.pool).await?
        };
        Ok(configs)
    }

    pub async fn get_config(&self, id: &str) -> Result<Option<EmbedWidgetConfig>, anyhow::Error> {
        let config = sqlx::query_as::<_, EmbedWidgetConfig>(
            "SELECT * FROM embed_widget_configs WHERE id = $1"
        ).bind(id).fetch_optional(&self.pool).await?;
        Ok(config)
    }

    pub async fn create_config(&self, org_id: &str, name: &str, widget_type: &str, settings: &serde_json::Value, styles: &serde_json::Value, trigger_config: &serde_json::Value) -> Result<CreateConfigResult, anyhow::Error> {
        let config = sqlx::query_as::<_, EmbedWidgetConfig>(
            "INSERT INTO embed_widget_configs (org_id, name, widget_type, settings, styles, trigger_config)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        ).bind(org_id).bind(name).bind(widget_type).bind(settings).bind(styles).bind(trigger_config).fetch_one(&self.pool).await?;

        let script = self.generate_embed_script(&config.id);

        Ok(CreateConfigResult {
            config,
            script,
        })
    }

    pub async fn update_config(&self, id: &str, updates: &serde_json::Value) -> Result<EmbedWidgetConfig, anyhow::Error> {
        let mut query = String::from("UPDATE embed_widget_configs SET updated_at = NOW()");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , name = ${}", param_count));
            params.push(name.to_string());
            param_count += 1;
        }
        if let Some(status) = updates.get("status").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , status = ${}", param_count));
            params.push(status.to_string());
            param_count += 1;
        }
        if let Some(settings) = updates.get("settings") {
            query.push_str(&format!(" , settings = ${}", param_count));
            params.push(settings.to_string());
            param_count += 1;
        }
        if let Some(styles) = updates.get("styles") {
            query.push_str(&format!(" , styles = ${}", param_count));
            params.push(styles.to_string());
            param_count += 1;
        }
        if let Some(trigger_config) = updates.get("trigger_config") {
            query.push_str(&format!(" , trigger_config = ${}", param_count));
            params.push(trigger_config.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(id.to_string());

        let config = sqlx::query_as::<_, EmbedWidgetConfig>(&query)
            .bind(&params[..])
            .fetch_one(&self.pool).await?;

        Ok(config)
    }

    pub async fn delete_config(&self, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM embed_widget_configs WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_settings(&self, org_id: &str) -> Result<EmbedWidgetSettings, anyhow::Error> {
        let settings = sqlx::query_as::<_, EmbedWidgetSettings>(
            "SELECT * FROM embed_widget_settings WHERE org_id = $1"
        ).bind(org_id).fetch_optional(&self.pool).await?;

        match settings {
            Some(s) => Ok(s),
            None => {
                let settings = sqlx::query_as::<_, EmbedWidgetSettings>(
                    "INSERT INTO embed_widget_settings (org_id, enabled, analytics_enabled)
                     VALUES ($1, true, true) RETURNING *"
                ).bind(org_id).fetch_one(&self.pool).await?;
                Ok(settings)
            }
        }
    }

    pub async fn update_settings(&self, org_id: &str, updates: &serde_json::Value) -> Result<EmbedWidgetSettings, anyhow::Error> {
        let mut query = String::from("UPDATE embed_widget_settings SET updated_at = NOW()");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(enabled) = updates.get("enabled").and_then(|v| v.as_bool()) {
            query.push_str(&format!(" , enabled = ${}", param_count));
            params.push(enabled.to_string());
            param_count += 1;
        }
        if let Some(allow_custom_domains) = updates.get("allow_custom_domains").and_then(|v| v.as_bool()) {
            query.push_str(&format!(" , allow_custom_domains = ${}", param_count));
            params.push(allow_custom_domains.to_string());
            param_count += 1;
        }
        if let Some(max_instances) = updates.get("max_instances_per_domain").and_then(|v| v.as_i64()) {
            query.push_str(&format!(" , max_instances_per_domain = ${}", param_count));
            params.push(max_instances.to_string());
            param_count += 1;
        }
        if let Some(analytics_enabled) = updates.get("analytics_enabled").and_then(|v| v.as_bool()) {
            query.push_str(&format!(" , analytics_enabled = ${}", param_count));
            params.push(analytics_enabled.to_string());
            param_count += 1;
        }
        if let Some(default_language) = updates.get("default_language").and_then(|v| v.as_str()) {
            query.push_str(&format!(" , default_language = ${}", param_count));
            params.push(default_language.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE org_id = ${} RETURNING *", param_count));
        params.push(org_id.to_string());

        let settings = sqlx::query_as::<_, EmbedWidgetSettings>(&query)
            .bind(&params[..])
            .fetch_one(&self.pool).await?;

        Ok(settings)
    }

    pub async fn record_event(&self, instance_id: &str, event_type: &str, payload: &serde_json::Value) -> Result<String, anyhow::Error> {
        let event = sqlx::query_as::<_, EmbedWidgetEvent>(
            "INSERT INTO embed_widget_events (instance_id, event_type, payload)
             VALUES ($1, $2, $3) RETURNING *"
        ).bind(instance_id).bind(event_type).bind(payload).fetch_one(&self.pool).await?;
        Ok(event.id)
    }

    pub async fn get_public_widget_data(&self, config_id: &str) -> Result<PublicWidgetData, anyhow::Error> {
        let config = self.get_config(config_id).await?
            .ok_or_else(|| anyhow::anyhow!("Widget config not found"))?;

        let help_center = if config.widget_type == "help_center" || config.widget_type == "combined" {
            // Fetch public help center data
            let articles = sqlx::query(r#"
                SELECT id, title, slug, content, category_id, helpful_count, not_helpful_count
                FROM help_center_articles
                WHERE status = 'published'
                ORDER BY pin_order ASC, helpful_count DESC
                LIMIT 20
            "#).fetch_all(&self.pool).await.ok();

            articles.map(|rows| {
                let articles: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.try_get::<String, _>("id").unwrap_or_default(),
                            "title": r.try_get::<String, _>("title").unwrap_or_default(),
                            "slug": r.try_get::<String, _>("slug").unwrap_or_default(),
                            "content": r.try_get::<String, _>("content").unwrap_or_default(),
                            "categoryId": r.try_get::<Option<String>, _>("category_id").unwrap_or_default(),
                            "helpfulCount": r.try_get::<i64, _>("helpful_count").unwrap_or_default(),
                            "notHelpfulCount": r.try_get::<i64, _>("not_helpful_count").unwrap_or_default(),
                        })
                    })
                    .collect();
                serde_json::json!({ "articles": articles })
            })
        } else {
            None
        };

        Ok(PublicWidgetData {
            widget_type: config.widget_type,
            settings: config.settings,
            styles: config.styles,
            trigger_config: config.trigger_config,
            help_center,
            default_language: "en".to_string(),
        })
    }

    pub fn generate_embed_script(&self, config_id: &str) -> String {
        format!(r#"
(function() {{
    var script = document.createElement('script');
    script.src = 'https://communicator.app/widget/embed.js';
    script.setAttribute('data-config-id', '{config_id}');
    script.setAttribute('data-api-url', window.location.origin);
    (document.getElementsByTagName('head')[0] || document.documentElement).appendChild(script);
    script.onload = function() {{
        window.CommunicatorWidget.init('{{{config_id}}}');
    }};
}})();
"#).trim().to_string()
    }

    pub async fn get_events(&self, instance_id: Option<&str>, event_type: Option<&str>, limit: i64) -> Result<Vec<EmbedWidgetEvent>, anyhow::Error> {
        let mut query = String::from("SELECT id, instance_id, event_type, payload, created_at FROM embed_widget_events WHERE 1=1");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(instance_id) = instance_id {
            query.push_str(&format!(" AND instance_id = ${}", param_count));
            params.push(instance_id.to_string());
            param_count += 1;
        }
        if let Some(event_type) = event_type {
            query.push_str(&format!(" AND event_type = ${}", param_count));
            params.push(event_type.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", param_count));
        params.push(limit.to_string());

        let events = sqlx::query_as::<_, EmbedWidgetEvent>(&query)
            .bind(&params[..])
            .fetch_all(&self.pool).await?;

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_serialization() {
        let summary = EmbedWidgetSummary {
            total_configs: 0,
            active_configs: 0,
            total_instances: 0,
            active_instances: 0,
            total_events: 0,
            events_by_type: vec![],
            top_domains: vec![],
            recent_events: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_configs\""));
    }

    #[test]
    fn test_embed_script_generation() {
        let service = EmbedWidgetService { pool: PgPool::connect_lazy("postgres://").unwrap() };
        let script = service.generate_embed_script("test-config-id");
        assert!(script.contains("test-config-id"));
        assert!(script.contains("CommunicatorWidget"));
    }

    #[test]
    fn test_create_config_result_serialization() {
        let result = CreateConfigResult {
            config: EmbedWidgetConfig {
                id: "test".to_string(),
                org_id: "org1".to_string(),
                name: "Test Widget".to_string(),
                widget_type: "help_center".to_string(),
                status: "active".to_string(),
                settings: serde_json::json!({}),
                styles: serde_json::json!({}),
                trigger_config: serde_json::json!({}),
                domain_whitelist: None,
                install_count: 0,
                last_installed_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            script: "<script>test</script>".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Test Widget"));
    }
}
