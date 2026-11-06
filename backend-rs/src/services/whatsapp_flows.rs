#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::config::Config;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Industry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateGroup {
    pub id: String,
    pub brand_id: String,
    pub industry_id: Option<String>,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppTemplateFlow {
    pub id: String,
    pub brand_id: String,
    pub group_id: Option<String>,
    pub industry_id: Option<String>,
    pub name: String,
    pub flow_type: String,
    pub category: String,
    pub body: String,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub buttons: Option<serde_json::Value>,
    pub sections: Option<serde_json::Value>,
    pub language: String,
    pub status: String,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppFlowStep {
    pub id: String,
    pub flow_id: String,
    pub step_number: i32,
    pub condition: Option<String>,
    pub action_type: String,
    pub action_config: Option<serde_json::Value>,
    pub next_step_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowWithSteps {
    pub flow: WhatsAppTemplateFlow,
    pub steps: Vec<WhatsAppFlowStep>,
}

// ---------------------------------------------------------------------------
// Flows Service
// ---------------------------------------------------------------------------

pub struct WhatsAppFlowsService {
    pool: SqlitePool,
    _config: Config,
}

impl WhatsAppFlowsService {
    pub fn new(pool: SqlitePool, config: Config) -> Self {
        Self {
            pool,
            _config: config,
        }
    }

    // ---------------------------------------------------------------------------
    // Industry CRUD
    // ---------------------------------------------------------------------------

    pub async fn create_industry(
        &self,
        name: &str,
        description: &str,
        icon: Option<&str>,
    ) -> Result<Industry, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO industries (id, name, description, icon, is_active, created_at) VALUES (?, ?, ?, ?, 1, ?)"
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(icon)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(Industry {
            id,
            name: name.to_string(),
            description: description.to_string(),
            icon: icon.map(|s| s.to_string()),
            is_active: true,
            created_at: now,
        })
    }

    pub async fn list_industries(&self) -> Result<Vec<Industry>, String> {
        let rows = sqlx::query(
            "SELECT id, name, description, icon, is_active, created_at FROM industries ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| Industry {
                id: row.try_get("id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
                description: row.try_get("description").unwrap_or_default(),
                icon: row.try_get("icon").ok().flatten(),
                is_active: row.try_get("is_active").unwrap_or(0i32) != 0,
                created_at: row.try_get("created_at").unwrap_or_default(),
            })
            .collect())
    }

    pub async fn update_industry(
        &self,
        industry_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        icon: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sets = Vec::new();
        let mut binds: Vec<&str> = Vec::new();

        if let Some(n) = name {
            sets.push("name = ?");
            binds.push(n);
        }
        if let Some(d) = description {
            sets.push("description = ?");
            binds.push(d);
        }
        if let Some(i) = icon {
            sets.push("icon = ?");
            binds.push(i);
        }
        if let Some(a) = is_active {
            sets.push("is_active = ?");
            binds.push(&if a { "1" } else { "0" });
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = ?");
        binds.push(&now);

        let query = format!(
            "UPDATE industries SET {} WHERE id = ?",
            sets.join(", ")
        );

        let mut query_builder = sqlx::query(&query);
        for b in &binds {
            query_builder = query_builder.bind(b);
        }
        query_builder
            .bind(industry_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }

    pub async fn delete_industry(&self, industry_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM industries WHERE id = ?")
            .bind(industry_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Template Group CRUD
    // ---------------------------------------------------------------------------

    pub async fn create_template_group(
        &self,
        brand_id: &str,
        industry_id: Option<&str>,
        name: &str,
        description: &str,
        sort_order: i32,
    ) -> Result<TemplateGroup, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO template_groups (id, brand_id, industry_id, name, description, sort_order, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(brand_id)
        .bind(industry_id)
        .bind(name)
        .bind(description)
        .bind(sort_order)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(TemplateGroup {
            id,
            brand_id: brand_id.to_string(),
            industry_id: industry_id.map(|s| s.to_string()),
            name: name.to_string(),
            description: description.to_string(),
            sort_order,
            created_at: now,
        })
    }

    pub async fn list_template_groups(
        &self,
        brand_id: &str,
        industry_id: Option<&str>,
    ) -> Result<Vec<TemplateGroup>, String> {
        let query_str = if industry_id.is_some() {
            "SELECT id, brand_id, industry_id, name, description, sort_order, created_at FROM template_groups WHERE brand_id = ?1 AND industry_id = ?2 ORDER BY sort_order, name"
        } else {
            "SELECT id, brand_id, industry_id, name, description, sort_order, created_at FROM template_groups WHERE brand_id = ?1 ORDER BY sort_order, name"
        };

        let rows = if let Some(ind) = industry_id {
            sqlx::query(query_str)
                .bind(brand_id)
                .bind(ind)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?
        } else {
            sqlx::query(query_str)
                .bind(brand_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?
        };

        Ok(rows
            .into_iter()
            .map(|row| TemplateGroup {
                id: row.try_get("id").unwrap_or_default(),
                brand_id: row.try_get("brand_id").unwrap_or_default(),
                industry_id: row.try_get("industry_id").ok().flatten(),
                name: row.try_get("name").unwrap_or_default(),
                description: row.try_get("description").unwrap_or_default(),
                sort_order: row.try_get("sort_order").unwrap_or(0i32),
                created_at: row.try_get("created_at").unwrap_or_default(),
            })
            .collect())
    }

    pub async fn update_template_group(
        &self,
        group_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        sort_order: Option<i32>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sets = Vec::new();
        let mut binds: Vec<Box<dyn std::any::Any>> = Vec::new();

        if let Some(n) = name {
            sets.push("name = ?");
            binds.push(Box::new(n));
        }
        if let Some(d) = description {
            sets.push("description = ?");
            binds.push(Box::new(d));
        }
        if let Some(s) = sort_order {
            sets.push("sort_order = ?");
            binds.push(Box::new(s));
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = ?");

        let query = format!(
            "UPDATE template_groups SET {} WHERE id = ?",
            sets.join(", ")
        );

        let mut qb = sqlx::query(&query);
        for b in &binds {
            if let Some(s) = b.downcast_ref::<&str>() {
                qb = qb.bind(s);
            } else if let Some(i) = b.downcast_ref::<i32>() {
                qb = qb.bind(i);
            }
        }
        qb.bind(&now)
            .bind(group_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }

    pub async fn delete_template_group(&self, group_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM template_groups WHERE id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Template Flow CRUD
    // ---------------------------------------------------------------------------

    pub async fn create_template_flow(
        &self,
        brand_id: &str,
        group_id: Option<&str>,
        industry_id: Option<&str>,
        name: &str,
        flow_type: &str,
        category: &str,
        body: &str,
        header: Option<&str>,
        footer: Option<&str>,
        buttons: Option<&str>,
        sections: Option<&str>,
        language: &str,
    ) -> Result<WhatsAppTemplateFlow, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO whatsapp_template_flows (id, brand_id, group_id, industry_id, name, type, category, body, header, footer, buttons, sections, language, status, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'draft', 1, ?, ?)"
        )
        .bind(&id)
        .bind(brand_id)
        .bind(group_id)
        .bind(industry_id)
        .bind(name)
        .bind(flow_type)
        .bind(category)
        .bind(body)
        .bind(header)
        .bind(footer)
        .bind(buttons)
        .bind(sections)
        .bind(language)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(WhatsAppTemplateFlow {
            id,
            brand_id: brand_id.to_string(),
            group_id: group_id.map(|s| s.to_string()),
            industry_id: industry_id.map(|s| s.to_string()),
            name: name.to_string(),
            flow_type: flow_type.to_string(),
            category: category.to_string(),
            body: body.to_string(),
            header: header.map(|s| s.to_string()),
            footer: footer.map(|s| s.to_string()),
            buttons: buttons.map(|s| serde_json::from_str(s).unwrap_or_default()),
            sections: sections.map(|s| serde_json::from_str(s).unwrap_or_default()),
            language: language.to_string(),
            status: "draft".to_string(),
            is_active: true,
            metadata: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn get_template_flow(&self, flow_id: &str) -> Result<FlowWithSteps, String> {
        let row = sqlx::query(
            "SELECT id, brand_id, group_id, industry_id, name, type, category, body,
                    header, footer, buttons, sections, language, status, is_active, metadata, created_at, updated_at
             FROM whatsapp_template_flows WHERE id = ?1 LIMIT 1"
        )
        .bind(flow_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if matches!(&e, sqlx::Error::RowNotFound) {
                "Flow not found".to_string()
            } else {
                format!("DB error: {}", e)
            }
        })?;

        let flow = WhatsAppTemplateFlow {
            id: row.try_get("id").unwrap_or_default(),
            brand_id: row.try_get("brand_id").unwrap_or_default(),
            group_id: row.try_get("group_id").ok().flatten(),
            industry_id: row.try_get("industry_id").ok().flatten(),
            name: row.try_get("name").unwrap_or_default(),
            flow_type: row.try_get("type").unwrap_or_default(),
            category: row.try_get("category").unwrap_or_default(),
            body: row.try_get("body").unwrap_or_default(),
            header: row.try_get("header").ok().flatten(),
            footer: row.try_get("footer").ok().flatten(),
            buttons: row.try_get::<Option<String>, _>("buttons").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
            sections: row.try_get::<Option<String>, _>("sections").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
            language: row.try_get("language").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            is_active: row.try_get("is_active").unwrap_or(0i32) != 0,
            metadata: row.try_get::<Option<String>, _>("metadata").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        };

        let steps = self.list_flow_steps(flow_id).await?;

        Ok(FlowWithSteps { flow, steps })
    }

    pub async fn list_template_flows(
        &self,
        brand_id: &str,
        industry_id: Option<&str>,
        group_id: Option<&str>,
        category: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<WhatsAppTemplateFlow>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, group_id, industry_id, name, type, category, body,
             header, footer, buttons, sections, language, status, is_active, metadata, created_at, updated_at
             FROM whatsapp_template_flows WHERE brand_id = ?1 ORDER BY name"
        )
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            let flow = WhatsAppTemplateFlow {
                id: row.try_get("id").unwrap_or_default(),
                brand_id: row.try_get("brand_id").unwrap_or_default(),
                group_id: row.try_get("group_id").ok().flatten(),
                industry_id: row.try_get("industry_id").ok().flatten(),
                name: row.try_get("name").unwrap_or_default(),
                flow_type: row.try_get("type").unwrap_or_default(),
                category: row.try_get("category").unwrap_or_default(),
                body: row.try_get("body").unwrap_or_default(),
                header: row.try_get("header").ok().flatten(),
                footer: row.try_get("footer").ok().flatten(),
                buttons: row.try_get::<Option<String>, _>("buttons").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
                sections: row.try_get::<Option<String>, _>("sections").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
                language: row.try_get("language").unwrap_or_default(),
                status: row.try_get("status").unwrap_or_default(),
                is_active: row.try_get("is_active").unwrap_or(0i32) != 0,
                metadata: row.try_get::<Option<String>, _>("metadata").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            };

            // Filter in Rust
            if let Some(ind) = industry_id {
                if flow.industry_id.as_deref() != Some(ind) {
                    continue;
                }
            }
            if let Some(grp) = group_id {
                if flow.group_id.as_deref() != Some(grp) {
                    continue;
                }
            }
            if let Some(cat) = category {
                if flow.category != cat {
                    continue;
                }
            }
            if let Some(st) = status {
                if flow.status != st {
                    continue;
                }
            }

            result.push(flow);
        }

        Ok(result)
    }

    pub async fn update_template_flow(
        &self,
        flow_id: &str,
        name: Option<&str>,
        body: Option<&str>,
        header: Option<&str>,
        footer: Option<&str>,
        buttons: Option<&str>,
        sections: Option<&str>,
        status: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sets = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(n) = name {
            sets.push("name = ?");
            binds.push(n.to_string());
        }
        if let Some(b) = body {
            sets.push("body = ?");
            binds.push(b.to_string());
        }
        if let Some(h) = header {
            sets.push("header = ?");
            binds.push(h.to_string());
        }
        if let Some(f) = footer {
            sets.push("footer = ?");
            binds.push(f.to_string());
        }
        if let Some(b) = buttons {
            sets.push("buttons = ?");
            binds.push(b.to_string());
        }
        if let Some(s) = sections {
            sets.push("sections = ?");
            binds.push(s.to_string());
        }
        if let Some(st) = status {
            sets.push("status = ?");
            binds.push(st.to_string());
        }
        if let Some(a) = is_active {
            sets.push("is_active = ?");
            binds.push(if a { "1".to_string() } else { "0".to_string() });
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = ?");
        binds.push(now);

        let query = format!(
            "UPDATE whatsapp_template_flows SET {} WHERE id = ?",
            sets.join(", ")
        );

        let mut qb = sqlx::query(&query);
        for b in &binds {
            qb = qb.bind(b);
        }
        qb.bind(flow_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }

    pub async fn delete_template_flow(&self, flow_id: &str) -> Result<(), String> {
        // Delete steps first
        sqlx::query("DELETE FROM whatsapp_flow_steps WHERE flow_id = ?")
            .bind(flow_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        sqlx::query("DELETE FROM whatsapp_template_flows WHERE id = ?")
            .bind(flow_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }

    pub async fn duplicate_flow(&self, flow_id: &str, new_name: &str) -> Result<String, String> {
        let row = sqlx::query(
            "SELECT id, brand_id, group_id, industry_id, name, type, category, body,
                    header, footer, buttons, sections, language, status, is_active, metadata, created_at, updated_at
             FROM whatsapp_template_flows WHERE id = ?1 LIMIT 1"
        )
        .bind(flow_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if matches!(&e, sqlx::Error::RowNotFound) {
                "Flow not found".to_string()
            } else {
                format!("DB error: {}", e)
            }
        })?;

        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let brand_id = row.try_get::<String, _>("brand_id").unwrap_or_default();
        let group_id: Option<String> = row.try_get("group_id").ok().flatten();
        let industry_id: Option<String> = row.try_get("industry_id").ok().flatten();
        let flow_type = row.try_get::<String, _>("type").unwrap_or_default();
        let category = row.try_get::<String, _>("category").unwrap_or_default();
        let body = row.try_get::<String, _>("body").unwrap_or_default();
        let header: Option<String> = row.try_get("header").ok().flatten();
        let footer: Option<String> = row.try_get("footer").ok().flatten();
        let buttons: Option<String> = row.try_get("buttons").ok().flatten();
        let sections: Option<String> = row.try_get("sections").ok().flatten();
        let language = row.try_get::<String, _>("language").unwrap_or_default();
        let metadata: Option<String> = row.try_get("metadata").ok().flatten();

        sqlx::query(
            "INSERT INTO whatsapp_template_flows (id, brand_id, group_id, industry_id, name, type, category, body, header, footer, buttons, sections, language, status, is_active, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'draft', 0, ?, ?, ?)"
        )
        .bind(&new_id)
        .bind(&brand_id)
        .bind(&group_id)
        .bind(&industry_id)
        .bind(new_name)
        .bind(&flow_type)
        .bind(&category)
        .bind(&body)
        .bind(&header)
        .bind(&footer)
        .bind(&buttons)
        .bind(&sections)
        .bind(&language)
        .bind(&metadata)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(new_id)
    }

    pub async fn validate_flow(&self, flow_id: &str) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        let row = sqlx::query(
            "SELECT name, body, header FROM whatsapp_template_flows WHERE id = ?1 LIMIT 1"
        )
        .bind(flow_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        match row {
            Some(row) => {
                let name: String = row.try_get("name").unwrap_or_default();
                let body: String = row.try_get("body").unwrap_or_default();
                let header: Option<String> = row.try_get("header").ok().flatten();
                if name.is_empty() {
                    errors.push("Flow name is required".to_string());
                }
                if body.is_empty() {
                    errors.push("Flow body is required".to_string());
                }
                if body.len() > 1024 {
                    errors.push("Flow body exceeds 1024 character limit".to_string());
                }
                if let Some(h) = header {
                    if h.len() > 60 {
                        errors.push("Header exceeds 60 character limit".to_string());
                    }
                }
            }
            None => {
                errors.push("Flow not found".to_string());
            }
        }

        Ok(errors)
    }

    // ---------------------------------------------------------------------------
    // Flow Step CRUD
    // ---------------------------------------------------------------------------

    pub async fn create_flow_step(
        &self,
        flow_id: &str,
        step_number: i32,
        condition: Option<&str>,
        action_type: &str,
        action_config: Option<&str>,
        next_step_id: Option<&str>,
    ) -> Result<WhatsAppFlowStep, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO whatsapp_flow_steps (id, flow_id, step_number, condition, action_type, action_config, next_step_id, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(flow_id)
        .bind(step_number)
        .bind(condition)
        .bind(action_type)
        .bind(action_config)
        .bind(next_step_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(WhatsAppFlowStep {
            id,
            flow_id: flow_id.to_string(),
            step_number,
            condition: condition.map(|s| s.to_string()),
            action_type: action_type.to_string(),
            action_config: action_config.and_then(|s| serde_json::from_str(s).ok()),
            next_step_id: next_step_id.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn list_flow_steps(&self, flow_id: &str) -> Result<Vec<WhatsAppFlowStep>, String> {
        let rows = sqlx::query(
            "SELECT id, flow_id, step_number, condition, action_type, action_config, next_step_id, created_at
             FROM whatsapp_flow_steps WHERE flow_id = ?1 ORDER BY step_number"
        )
        .bind(flow_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| WhatsAppFlowStep {
                id: row.try_get("id").unwrap_or_default(),
                flow_id: row.try_get("flow_id").unwrap_or_default(),
                step_number: row.try_get("step_number").unwrap_or(0i32),
                condition: row.try_get("condition").ok().flatten(),
                action_type: row.try_get("action_type").unwrap_or_default(),
                action_config: row.try_get::<Option<String>, _>("action_config").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()),
                next_step_id: row.try_get("next_step_id").ok().flatten(),
                created_at: row.try_get("created_at").unwrap_or_default(),
            })
            .collect())
    }

    pub async fn update_flow_step(
        &self,
        step_id: &str,
        step_number: Option<i32>,
        condition: Option<&str>,
        action_type: Option<&str>,
        action_config: Option<&str>,
        next_step_id: Option<&str>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sets = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(sn) = step_number {
            sets.push("step_number = ?");
            binds.push(sn.to_string());
        }
        if let Some(c) = condition {
            sets.push("condition = ?");
            binds.push(c.to_string());
        }
        if let Some(at) = action_type {
            sets.push("action_type = ?");
            binds.push(at.to_string());
        }
        if let Some(ac) = action_config {
            sets.push("action_config = ?");
            binds.push(ac.to_string());
        }
        if let Some(ns) = next_step_id {
            sets.push("next_step_id = ?");
            binds.push(ns.to_string());
        }

        if sets.is_empty() {
            return Ok(());
        }

        sets.push("updated_at = ?");
        binds.push(now);

        let query = format!(
            "UPDATE whatsapp_flow_steps SET {} WHERE id = ?",
            sets.join(", ")
        );

        let mut qb = sqlx::query(&query);
        for b in &binds {
            qb = qb.bind(b);
        }
        qb.bind(step_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }

    pub async fn delete_flow_step(&self, step_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM whatsapp_flow_steps WHERE id = ?")
            .bind(step_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        Ok(())
    }
}
