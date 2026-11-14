#![allow(dead_code)]

use sqlx::PgPool;

use crate::config::Config;
use crate::services::whatsapp::WhatsAppService;

// ---------------------------------------------------------------------------
// WhatsApp Automation Engine
// ---------------------------------------------------------------------------

pub struct WhatsAppAutomationEngine {
    pool: PgPool,
    config: Config,
    whatsapp: WhatsAppService,
}

impl WhatsAppAutomationEngine {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self {
            whatsapp: WhatsAppService::new(pool.clone(), config.clone()),
            pool,
            config,
        }
    }

    // ---------------------------------------------------------------------------
    // Agent commands (#bot, #resolve, #close, #csat, #tag)
    // ---------------------------------------------------------------------------

    pub async fn handle_agent_commands(
        &self,
        brand_id: &str,
        conversation_id: &str,
        contact_id: &str,
        text: &str,
    ) -> Option<String> {
        let lower = text.to_lowercase().trim().to_string();

        match lower.split_whitespace().next() {
            Some("#bot") => {
                let _ = self.whatsapp
                    .upsert_automation_state(brand_id, conversation_id, false, false, None, Some("ai_active"), None)
                    .await;
                Some("AI assistant is now active. How can I help you?".to_string())
            }
            Some("#resolve") => {
                let _ = sqlx::query(
                    "UPDATE conversations SET state = 'resolved' WHERE id = $1"
                )
                .bind(conversation_id)
                .execute(&self.pool)
                .await;
                Some("Conversation marked as resolved.".to_string())
            }
            Some("#close") => {
                let _ = sqlx::query(
                    "UPDATE conversations SET state = 'closed' WHERE id = $1"
                )
                .bind(conversation_id)
                .execute(&self.pool)
                .await;
                Some("Conversation closed.".to_string())
            }
            Some("#csat") => {
                let result = self.send_csat_survey(brand_id, contact_id).await.ok()?;
                Some(result)
            }
            Some("#tag") => {
                let parts: Vec<&str> = lower.split_whitespace().collect();
                if parts.len() >= 2 {
                    let tag = parts[1..].join(" ");
                    let _ = sqlx::query(
                        "INSERT OR IGNORE INTO contact_tags (id, contact_id, tag, created_at) VALUES ($1, $2, $3, NOW()::TEXT)"
                    )
                    .bind(uuid::Uuid::new_v4().to_string())
                    .bind(contact_id)
                    .bind(&tag)
                    .execute(&self.pool)
                    .await;
                    Some(format!("Tagged contact with: {}", tag))
                } else {
                    Some("Usage: #tag <tagname>".to_string())
                }
            }
            _ => None,
        }
    }

    // ---------------------------------------------------------------------------
    // CSAT survey
    // ---------------------------------------------------------------------------

    async fn send_csat_survey(&self, brand_id: &str, contact_id: &str) -> Result<String, String> {
        // Get brand name for personalization
        let brand_name: Option<(String,)> = sqlx::query_as(
            "SELECT name FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let name = brand_name.map(|(n,)| n).unwrap_or_else(|| "our team".to_string());

        let buttons = vec![
            serde_json::json!({"id": "csat_5", "title": "?????"}),
            serde_json::json!({"id": "csat_4", "title": "????"}),
            serde_json::json!({"id": "csat_3", "title": "???"}),
            serde_json::json!({"id": "csat_2", "title": "??"}),
            serde_json::json!({"id": "csat_1", "title": "?"}),
        ];

        // Only send up to 3 buttons for WhatsApp
        let _ = self.whatsapp.send_interactive_buttons(
            brand_id,
            contact_id,
            &format!("How would you rate your experience with {}?", name),
            &buttons[..3],
            Some("Customer Satisfaction"),
            None,
        ).await?;

        Ok("CSAT survey sent".to_string())
    }

    pub async fn handle_csat_response(
        &self,
        brand_id: &str,
        conversation_id: &str,
        contact_id: &str,
        payload: &str,
    ) -> Option<String> {
        let score = match payload {
            "csat_5" => 5,
            "csat_4" => 4,
            "csat_3" => 3,
            "csat_2" => 2,
            "csat_1" => 1,
            _ => {
                // Try to parse numeric
                if let Ok(n) = payload.parse::<i32>() {
                    n
                } else {
                    return None;
                }
            }
        };

        // Update conversation CSAT score
        let _ = sqlx::query(
            "UPDATE conversations SET csat_score = $1 WHERE id = $2"
        )
        .bind(score)
        .bind(conversation_id)
        .execute(&self.pool)
        .await;

        // Tag contact based on score
        let tag = if score >= 4 { "csat_promoter" } else if score >= 3 { "csat_passive" } else { "csat_detractor" };
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO contact_tags (id, contact_id, tag, created_at) VALUES ($1, $2, $3, NOW()::TEXT)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(contact_id)
        .bind(tag)
        .execute(&self.pool)
        .await;

        // Mark CSAT as sent
        let _ = self.whatsapp.upsert_automation_state(
            brand_id, conversation_id, false, true, None, None, None
        ).await;

        if score >= 4 {
            Some("Thank you for your feedback! We're glad you had a great experience.".to_string())
        } else if score >= 3 {
            Some("Thank you for your feedback. We'll work to improve your experience.".to_string())
        } else {
            Some("We're sorry to hear about your experience. A team member will follow up shortly.".to_string())
        }
    }

    // ---------------------------------------------------------------------------
    // Agent handoff
    // ---------------------------------------------------------------------------

    pub async fn execute_handoff(
        &self,
        brand_id: &str,
        conversation_id: &str,
    ) -> Result<String, String> {
        // Find available agent using round-robin
        let agent: Option<(String, String)> = sqlx::query_as(
            "SELECT id, name FROM team_members WHERE availability_status = 'online' ORDER BY RANDOM() LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if let Some((agent_id, agent_name)) = agent {
            let _ = sqlx::query(
                "UPDATE conversations SET assigned_to_id = $1, assigned_to_name = $2, human_takeover = 1 WHERE id = $3"
            )
            .bind(&agent_id)
            .bind(&agent_name)
            .bind(conversation_id)
            .execute(&self.pool)
            .await;

            // Update automation state
            let _ = self.whatsapp.upsert_automation_state(
                brand_id, conversation_id, false, false, None, Some("handoff"), None
            ).await;

            Ok(format!("Transferred to {}.", agent_name))
        } else {
            Ok("No agents are currently available. Please hold.".to_string())
        }
    }

    // ---------------------------------------------------------------------------
    // Navigation menu
    // ---------------------------------------------------------------------------

    pub async fn handle_navigation_menu(
        &self,
        brand_id: &str,
        contact_id: &str,
        payload: &str,
    ) -> Option<String> {
        match payload {
            "menu_main" => Some(self.send_main_menu(brand_id, contact_id).await.ok()?),
            "menu_services" => Some(self.send_services_menu(brand_id, contact_id).await.ok()?),
            "menu_pricing" => Some(self.send_pricing_menu(brand_id, contact_id).await.ok()?),
            "menu_location" => Some(self.send_location_info(brand_id, contact_id).await.ok()?),
            "menu_agent" => Some(self.execute_handoff(brand_id, contact_id).await.ok()?),
            _ => None,
        }
    }

    async fn send_main_menu(&self, brand_id: &str, contact_id: &str) -> Result<String, String> {
        let brand_name: Option<(String,)> = sqlx::query_as(
            "SELECT name FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let name = brand_name.map(|(n,)| n).unwrap_or_else(|| "us".to_string());

        let buttons = vec![
            serde_json::json!({"id": "menu_services", "title": "?? Services"}),
            serde_json::json!({"id": "menu_pricing", "title": "?? Pricing"}),
            serde_json::json!({"id": "menu_location", "title": "?? Location"}),
        ];

        self.whatsapp.send_interactive_buttons(
            brand_id,
            contact_id,
            &format!("Welcome to {}! How can we help you?", name),
            &buttons,
            Some("Main Menu"),
            None,
        ).await?;

        // Update automation state
        let _ = self.whatsapp.upsert_automation_state(
            brand_id, contact_id, false, false, Some("main_menu"), None, None
        ).await;

        Ok("Main menu sent".to_string())
    }

    async fn send_services_menu(&self, brand_id: &str, contact_id: &str) -> Result<String, String> {
        // Get services from brand profile
        let services: Option<(String,)> = sqlx::query_as(
            "SELECT services FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let body = if let Some((svc_json,)) = services {
            if let Ok(services) = serde_json::from_str::<Vec<serde_json::Value>>(&svc_json) {
                if !services.is_empty() {
                    let list = services.iter().take(10).map(|s| {
                        s["name"].as_str().unwrap_or("Service").to_string()
                    }).collect::<Vec<_>>().join("\n");
                    format!("Our services:\n\n{}", list)
                } else {
                    "We offer a variety of services. Contact an agent for details.".to_string()
                }
            } else {
                "We offer a variety of services. Contact an agent for details.".to_string()
            }
        } else {
            "We offer a variety of services. Contact an agent for details.".to_string()
        };

        let buttons = vec![
            serde_json::json!({"id": "menu_pricing", "title": "?? Pricing"}),
            serde_json::json!({"id": "menu_agent", "title": "?? Talk to Agent"}),
            serde_json::json!({"id": "menu_main", "title": "?? Main Menu"}),
        ];

        self.whatsapp.send_interactive_buttons(
            brand_id,
            contact_id,
            &body,
            &buttons,
            Some("Our Services"),
            None,
        ).await?;

        Ok("Services menu sent".to_string())
    }

    async fn send_pricing_menu(&self, brand_id: &str, contact_id: &str) -> Result<String, String> {
        // Get prices from brand profile
        let prices: Option<(String,)> = sqlx::query_as(
            "SELECT prices FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let body = if let Some((price_json,)) = prices {
            if let Ok(prices) = serde_json::from_str::<Vec<serde_json::Value>>(&price_json) {
                if !prices.is_empty() {
                    let list = prices.iter().take(10).map(|p| {
                        let name = p["name"].as_str().unwrap_or("Item");
                        let price = p["price"].as_str().unwrap_or("");
                        format!("{}: {}", name, price)
                    }).collect::<Vec<_>>().join("\n");
                    format!("Our pricing:\n\n{}", list)
                } else {
                    "Contact us for pricing details.".to_string()
                }
            } else {
                "Contact us for pricing details.".to_string()
            }
        } else {
            "Contact us for pricing details.".to_string()
        };

        let buttons = vec![
            serde_json::json!({"id": "menu_services", "title": "?? Services"}),
            serde_json::json!({"id": "menu_agent", "title": "?? Talk to Agent"}),
            serde_json::json!({"id": "menu_main", "title": "?? Main Menu"}),
        ];

        self.whatsapp.send_interactive_buttons(
            brand_id,
            contact_id,
            &body,
            &buttons,
            Some("Pricing"),
            None,
        ).await?;

        Ok("Pricing menu sent".to_string())
    }

    async fn send_location_info(&self, brand_id: &str, contact_id: &str) -> Result<String, String> {
        let address: Option<(String,)> = sqlx::query_as(
            "SELECT address FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if let Some((addr,)) = address {
            if !addr.is_empty() {
                let buttons = vec![
                    serde_json::json!({"id": "menu_main", "title": "?? Main Menu"}),
                    serde_json::json!({"id": "menu_agent", "title": "?? Talk to Agent"}),
                ];
                self.whatsapp.send_interactive_buttons(
                    brand_id,
                    contact_id,
                    &format!("?? Our location:\n\n{}", addr),
                    &buttons,
                    Some("Location"),
                    None,
                ).await?;
                return Ok("Location sent".to_string());
            }
        }

        let buttons = vec![
            serde_json::json!({"id": "menu_main", "title": "?? Main Menu"}),
        ];
        self.whatsapp.send_interactive_buttons(
            brand_id,
            contact_id,
            "We don't have a physical location listed. Contact an agent for more info.",
            &buttons,
            Some("Location"),
            None,
        ).await?;

        Ok("Location info sent".to_string())
    }

    // ---------------------------------------------------------------------------
    // Welcome greeting
    // ---------------------------------------------------------------------------

    pub async fn should_send_welcome_greeting(
        &self,
        brand_id: &str,
        conversation_id: &str,
    ) -> Result<bool, String> {
        let state = self.whatsapp.get_automation_state(brand_id, conversation_id).await?;

        if let Some((_, greeting_sent, _, _, _, _)) = state {
            Ok(!greeting_sent)
        } else {
            // Check if this is a new conversation
            let msg_count: Option<(i64,)> = sqlx::query_as(
                "SELECT COUNT(*) FROM messages WHERE conversation_id = $1"
            )
            .bind(conversation_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

            Ok(msg_count.map(|(c,)| c <= 1).unwrap_or(true))
        }
    }

    pub async fn send_welcome_greeting(
        &self,
        brand_id: &str,
        conversation_id: &str,
        contact_id: &str,
    ) -> Result<String, String> {
        let brand_name: Option<(String,)> = sqlx::query_as(
            "SELECT name FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let name = brand_name.map(|(n,)| n).unwrap_or_else(|| "us".to_string());

        // Get business hours from brand profile
        let bio: Option<(String,)> = sqlx::query_as(
            "SELECT bio FROM brands WHERE id = $1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let body = if let Some((b,)) = bio {
            if !b.is_empty() {
                format!("Welcome to {}!\n\n{}", name, b)
            } else {
                format!("Welcome to {}! How can we help you today?", name)
            }
        } else {
            format!("Welcome to {}! How can we help you today?", name)
        };

        let buttons = vec![
            serde_json::json!({"id": "menu_services", "title": "?? Services"}),
            serde_json::json!({"id": "menu_pricing", "title": "?? Pricing"}),
            serde_json::json!({"id": "menu_agent", "title": "?? Agent"}),
        ];

        self.whatsapp.send_interactive_buttons(
            brand_id,
            contact_id,
            &body,
            &buttons,
            Some(&name),
            None,
        ).await?;

        // Mark greeting as sent
        self.whatsapp.upsert_automation_state(
            brand_id, conversation_id, true, false, None, None, None
        ).await?;

        Ok("Welcome greeting sent".to_string())
    }

    // ---------------------------------------------------------------------------
    // Automation rules matching
    // ---------------------------------------------------------------------------

    pub async fn match_advanced_rule(
        &self,
        brand_id: &str,
        text: &str,
    ) -> Result<Option<(String, String, Option<String>)>, String> {
        let rules: Vec<(String, String, Option<String>, String, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, trigger_type, trigger_value, action_type, action_config, name FROM whatsapp_automation_rules WHERE brand_id = $1 AND is_active = 1 ORDER BY priority DESC"
            )
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        let lower = text.to_lowercase();

        for (rule_id, trigger_type, trigger_value, action_type, action_config, _name) in rules {
            let matched = match trigger_type.as_str() {
                "keyword" => {
                    if let Some(kw) = trigger_value {
                        lower.contains(&kw.to_lowercase())
                    } else {
                        false
                    }
                }
                "regex" => {
                    if let Some(re) = trigger_value {
                        match regex::Regex::new(&re) {
                            Ok(r) => r.is_match(&lower),
                            Err(_) => false,
                        }
                    } else {
                        false
                    }
                }
                "button" => {
                    if let Some(btn) = trigger_value {
                        text == btn
                    } else {
                        false
                    }
                }
                "intent" => {
                    if let Some(intent) = trigger_value {
                        lower.contains(&intent.to_lowercase())
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if matched {
                return Ok(Some((rule_id, action_type, action_config)));
            }
        }

        Ok(None)
    }

    pub async fn execute_rule_action(
        &self,
        brand_id: &str,
        contact_id: &str,
        conversation_id: &str,
        action_type: &str,
        action_config: Option<&str>,
    ) -> Result<Option<String>, String> {
        let config = action_config.unwrap_or("{}");
        let cfg: serde_json::Value = serde_json::from_str(config).unwrap_or_else(|_| serde_json::json!({}));

        match action_type {
            "auto_reply" => {
                let reply = cfg["reply"].as_str().unwrap_or("Thank you for your message.");
                let _ = self.whatsapp.send_text(brand_id, contact_id, reply, false).await?;
                Ok(Some(reply.to_string()))
            }
            "buttons" => {
                let body = cfg["body"].as_str().unwrap_or("Choose an option:");
                let buttons: Vec<serde_json::Value> = cfg["buttons"].as_array().cloned().unwrap_or_default();
                let _ = self.whatsapp.send_interactive_buttons(brand_id, contact_id, body, &buttons, None, None).await?;
                Ok(None)
            }
            "list" => {
                let body = cfg["body"].as_str().unwrap_or("Choose an option:");
                let button_title = cfg["button_title"].as_str().unwrap_or("Options");
                let sections: Vec<serde_json::Value> = cfg["sections"].as_array().cloned().unwrap_or_default();
                let _ = self.whatsapp.send_interactive_list(brand_id, contact_id, body, button_title, &sections, None, None).await?;
                Ok(None)
            }
            "handoff" => {
                let result = self.execute_handoff(brand_id, conversation_id).await?;
                Ok(Some(result))
            }
            "csat" => {
                let result = self.send_csat_survey(brand_id, contact_id).await?;
                Ok(Some(result))
            }
            "tag" => {
                let tag = cfg["tag"].as_str().unwrap_or("");
                if !tag.is_empty() {
                    let _ = sqlx::query(
                        "INSERT OR IGNORE INTO contact_tags (id, contact_id, tag, created_at) VALUES ($1, $2, $3, NOW()::TEXT)"
                    )
                    .bind(uuid::Uuid::new_v4().to_string())
                    .bind(contact_id)
                    .bind(tag)
                    .execute(&self.pool)
                    .await;
                }
                Ok(None)
            }
            "template" => {
                let template_name = cfg["template_name"].as_str().unwrap_or("");
                let language = cfg["language"].as_str().unwrap_or("en_US");
                if !template_name.is_empty() {
                    let _ = self.whatsapp.send_template(brand_id, contact_id, template_name, language, None).await?;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    // ---------------------------------------------------------------------------
    // Inactivity nudges
    // ---------------------------------------------------------------------------

    pub async fn run_inactivity_nudges(&self, brand_id: &str) -> Result<usize, String> {
        // Find expired session windows
        let now = chrono::Utc::now().to_rfc3339();
        let expired: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, contact_id FROM whatsapp_session_windows WHERE brand_id = $1 AND is_active = 1 AND expires_at < $2"
        )
        .bind(brand_id)
        .bind(&now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let mut count = 0;
        for (_id, contact_id) in expired {
            // Deactivate session
            let _ = sqlx::query(
                "UPDATE whatsapp_session_windows SET is_active = 0 WHERE brand_id = $1 AND contact_id = $2"
            )
            .bind(brand_id)
            .bind(&contact_id)
            .execute(&self.pool)
            .await;

            // Send nudge message
            let _ = self.whatsapp.send_text(
                brand_id,
                &contact_id,
                "We noticed you left earlier. Is there anything else we can help you with?",
                false,
            ).await;

            count += 1;
        }

        Ok(count)
    }

    // ---------------------------------------------------------------------------
    // Customer profile sync from Meta
    // ---------------------------------------------------------------------------

    pub async fn sync_customer_profile_from_meta(
        &self,
        brand_id: &str,
        wa_id: &str,
        profile_name: Option<&str>,
        profile_picture_url: Option<&str>,
        about: Option<&str>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();

        let _ = sqlx::query(
            "INSERT INTO whatsapp_customer_profiles (id, brand_id, wa_id, profile_name, profile_picture_url, about, created_at, updated_at, last_fetched_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $7)
             ON CONFLICT(wa_id) DO UPDATE SET
             profile_name = COALESCE($4, profile_name),
             profile_picture_url = COALESCE($5, profile_picture_url),
             about = COALESCE($6, about),
             updated_at = $7,
             last_fetched_at = $7"
        )
        .bind(&id)
        .bind(brand_id)
        .bind(wa_id)
        .bind(profile_name)
        .bind(profile_picture_url)
        .bind(about)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }
}
