#![allow(dead_code)]

//! Port of `services/reply_engine.py`.
//!
//! The Rust route that used to sit behind `/drafts/generate` invented the reply
//! text, the confidence and the citations in code. This module does what the
//! Python did instead: assemble real context out of the database, make one real
//! call to the configured AI provider, and store what the model actually said --
//! including its gaps. Anything the model did not answer is stored as absent and
//! reported, never filled in with a plausible number.

use crate::db::AppState;
use crate::services::alerts::{send_hot_lead_alert, HotLeadAlert};
use crate::services::claude_client::ask_json;
use crate::services::draft_actions::record_draft_audit;
use crate::services::integration_settings;
use crate::services::reply_integration::ReplyIntegrationService;
use crate::services::validation::{self, ValidationResult};
use serde_json::{json, Value};
use sqlx::Row;
use std::sync::OnceLock;
use uuid::Uuid;

/// States where a reply must not be drafted: the contact opted out, or the
/// conversation is already decided either way.
const INELIGIBLE_STATES: &[&str] = &["opted_out", "closed_won", "closed_lost"];

/// Intents that page a human instead of letting the AI continue.
const ESCALATING_INTENTS: &[&str] = &["escalate", "complaint", "buying_signal"];
const ALERT_INTENTS: &[&str] = &[
    "buying_signal",
    "complaint",
    "escalate",
    "refund_request",
    "damaged_item",
];
const ALERT_SENTIMENTS: &[&str] = &["negative", "urgent"];
/// Intents that mark the conversation hot, mirroring reply_engine.py:232.
const HOT_LEAD_INTENTS: &[&str] = &["buying_signal", "complaint", "refund_request", "damaged_item"];

const MAX_HISTORY_MESSAGES: i64 = 40;
const MAX_KB_ENTRIES: i64 = 30;
const MAX_MEMORIES: i64 = 12;
const REPLY_MAX_TOKENS: u32 = 1600;

fn claim_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?i)\b(refund|damaged|broken|defective|faulty)\b")
            .expect("static pattern")
    })
}

fn return_phrase_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?i)\b(return item|return product|returning item|send back)\b")
            .expect("static pattern")
    })
}

fn order_id_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(?i)(?:#|ORD-|order\s*#?)([A-Za-z0-9\-]{4,16})").expect("static pattern")
    })
}

pub enum ReplyError {
    /// No conversation with that id.
    NotFound(String),
    /// The conversation exists but is opted out / closed.
    NotEligible { conversation_id: String, state: String },
    /// The conversation has no contact row, so there is nobody to reply to.
    ContactMissing(String),
    /// No AI provider is configured: the reply cannot be generated at all.
    AiNotConfigured(String),
    /// The provider was reached and failed, or answered unusably.
    AiFailed(String),
    /// A database read that the engine cannot proceed without.
    DbFailed { context: String, reason: String },
}

impl ReplyError {
    pub fn status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            ReplyError::NotFound(_) => StatusCode::NOT_FOUND,
            ReplyError::NotEligible { .. } => StatusCode::BAD_REQUEST,
            ReplyError::ContactMissing(_) => StatusCode::BAD_REQUEST,
            ReplyError::AiNotConfigured(_) => StatusCode::SERVICE_UNAVAILABLE,
            ReplyError::AiFailed(_) => StatusCode::BAD_GATEWAY,
            ReplyError::DbFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn json(&self) -> Value {
        use axum::http::StatusCode;
        let status = self.status();
        let mut body = json!({ "saved": false, "databaseError": status == StatusCode::INTERNAL_SERVER_ERROR });
        match self {
            ReplyError::NotFound(id) => {
                body["error"] = json!("no conversation with that id exists; nothing was drafted");
                body["conversationId"] = json!(id);
            }
            ReplyError::NotEligible {
                conversation_id,
                state,
            } => {
                body["error"] = json!(format!(
                    "conversation is in state '{state}', which is not eligible for an AI reply"
                ));
                body["conversationId"] = json!(conversation_id);
                body["conversationState"] = json!(state);
            }
            ReplyError::ContactMissing(id) => {
                body["error"] = json!(format!(
                    "conversation {id} has no contact row, so there is no channel address to reply on"
                ));
                body["conversationId"] = json!(id);
            }
            ReplyError::AiNotConfigured(reason) => {
                body["error"] = json!(format!(
                    "no AI provider is configured, so no reply could be generated: {reason}"
                ));
                body["aiConfigured"] = json!(false);
            }
            ReplyError::AiFailed(reason) => {
                body["error"] = json!(format!("the AI provider could not produce a reply: {reason}"));
                body["aiFailed"] = json!(true);
            }
            ReplyError::DbFailed { context, reason } => {
                body["error"] = json!(format!("could not {context}: {reason}"));
            }
        }
        body
    }
}

/// What the model actually said, with every gap kept as `None`.
#[derive(Debug, Clone)]
pub struct ModelReply {
    pub intent: Option<String>,
    pub sentiment: Option<String>,
    pub language: Option<String>,
    pub confidence: Option<f64>,
    pub reply: Option<String>,
    pub citations: Vec<String>,
    pub ignored_fields: Vec<String>,
}

fn string_field(parsed: &Value, key: &str) -> Option<String> {
    parsed
        .get(key)
        .and_then(Value::as_str)
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn parse_model_reply(parsed: &Value) -> ModelReply {
    let mut ignored_fields = Vec::new();
    let confidence = match parsed.get("confidence") {
        None => None,
        Some(Value::Number(num)) => num.as_f64().filter(|v| (0.0..=1.0).contains(v)),
        Some(Value::String(text)) => text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| (0.0..=1.0).contains(v)),
        Some(other) => {
            ignored_fields.push(format!("confidence ({other})"));
            None
        }
    };
    let citations = match parsed.get("citations") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
            .collect(),
        Some(Value::Null) | None => Vec::new(),
        Some(other) => {
            ignored_fields.push(format!("citations ({other})"));
            Vec::new()
        }
    };
    ModelReply {
        intent: string_field(parsed, "intent"),
        sentiment: string_field(parsed, "sentiment"),
        language: string_field(parsed, "language"),
        confidence,
        reply: string_field(parsed, "reply"),
        citations,
        ignored_fields,
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedDraft {
    pub id: String,
    pub conversation_id: String,
    pub body: String,
    pub status: String,
    pub created_at: String,
    pub model: ModelReply,
    pub stored_confidence: f64,
    pub auto_send_threshold: Option<f64>,
    pub escalation: bool,
    pub language_updated: bool,
    pub conversation_state: Option<String>,
    pub resolution_type: Option<String>,
    pub verification_session_id: Option<String>,
    pub verification_url: Option<String>,
    pub alert: Option<String>,
    pub warnings: Vec<String>,
    pub draft_written: bool,
    /// Result of the validation layer check (Phase 7).
    pub validation: ValidationResult,
}

impl GeneratedDraft {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "draftId": self.id,
            "conversationId": self.conversation_id,
            "body": self.body,
            "status": self.status,
            "draft_status": self.status,
            "confidence": self.model.confidence,
            "storedConfidence": self.stored_confidence,
            "citations": self.model.citations,
            "intent": self.model.intent,
            "sentiment": self.model.sentiment,
            "language": self.model.language,
            "createdAt": self.created_at,
            "escalation": self.escalation,
            "autoSendThreshold": self.auto_send_threshold,
            "autoApproved": self.status == "auto_approved",
            "languageUpdated": self.language_updated,
            "conversationState": self.conversation_state,
            "resolutionType": self.resolution_type,
            "liveProofSessionId": self.verification_session_id,
            "verificationUrl": self.verification_url,
            "salesAlert": self.alert,
            "ignoredModelFields": self.model.ignored_fields,
            "saved": self.draft_written,
            "warnings": self.warnings,
            "validation": self.validation.to_json(),
        })
    }
}

/// Port of `_build_system_prompt`. The business name, description and guidelines
/// come from the tenant's Connections row; when that row is missing the prompt
/// says so rather than naming a company the operator never connected.
async fn build_system_prompt(
    state: &AppState,
    tenant_key: &str,
    warnings: &mut Vec<String>,
) -> String {
    let profile = match integration_settings::load(&state.db, tenant_key).await {
        Ok(Some(row)) => Some(row),
        Ok(None) => None,
        Err(err) => {
            warnings.push(format!(
                "the business profile could not be read, so the prompt was built without it: {err}"
            ));
            None
        }
    };

    let (name_line, description_block, guidelines_block) = match &profile {
        Some(row) => {
            let name = row.business_name.trim();
            let name_line = if name.is_empty() {
                "The connected business has no display name in the Connections profile. Do not \
                 invent or state a company name."
                    .to_string()
            } else {
                format!("You represent {name}.")
            };
            let description = row.business_description.trim();
            let description_block = if description.is_empty() {
                String::new()
            } else {
                format!("\nBUSINESS DESCRIPTION:\n{description}\n")
            };
            let guidelines = row.agent_guidelines.trim();
            let guidelines_block = if guidelines.is_empty() {
                String::new()
            } else {
                format!("\nADDITIONAL GUIDELINES:\n{guidelines}\n")
            };
            (name_line, description_block, guidelines_block)
        }
        None => {
            warnings.push(format!(
                "no Connections profile row exists for tenant '{tenant_key}', so the reply is \
                 unbranded and the model was told not to name a company"
            ));
            (
                "No business profile is configured for this tenant. Do not name a company, offer \
                 pricing, or make claims about services that are not listed below."
                    .to_string(),
                String::new(),
                String::new(),
            )
        }
    };

    format!(
        "You are a sales and support assistant replying to a lead or customer on behalf of the team.\n\
         {name_line}{description_block}\n\
         Only make claims found in the SERVICES context below. Never invent pricing or contractual terms.\n\
         If a CUSTOMER RELATIONSHIP section is present, use it to respond with awareness of the \
         relationship -- acknowledge what was previously discussed instead of restarting the \
         conversation. Never state anything about the relationship that isn't in that section.\n\
         If the customer asks for a return, refund, or reports damaged/defective merchandise, set \
         intent to \"refund_request\" or \"damaged_item\".\n\
         If the lead asks something outside the SERVICES context, or shows a strong buying signal, or \
         is upset/complaining, set intent to \"escalate\" and keep confidence low.\n\
         Detect the language of the lead's most recent message and write \"reply\" in that same \
         language, regardless of what language the rest of this prompt is in.{guidelines_block}\n\
         Respond with strict JSON only, no markdown fences:\n\
         {{\"intent\": \"faq_question|objection|buying_signal|scheduling_request|complaint|refund_request|damaged_item|unrelated|escalate\",\n\
         \"sentiment\": \"positive|neutral|negative|urgent\",\n\
         \"language\": \"BCP-47-ish short code of the lead's message, e.g. en, es, fr\",\n\
         \"confidence\": 0.0-1.0,\n\
         \"reply\": \"the message body to send back to the lead, written in `language`\",\n\
         \"citations\": [\"service names referenced\"]}}\n"
    )
}

struct Thread {
    history: String,
    last_inbound_body: String,
    message_count: i64,
}

/// The lead columns the hot-lead alert and the prompt need. `Default` is all
/// `None`, which renders as "(no lead linked)" instead of a made-up name.
#[derive(Debug, Clone, Default)]
struct LeadFacts {
    name: Option<String>,
    institution_type: Option<String>,
    city: Option<String>,
}

async fn load_history(state: &AppState, conversation_id: &str) -> Result<Thread, ReplyError> {
    let rows = sqlx::query(
        "SELECT direction, body, COALESCE(sender_name, '') AS sender_name FROM messages \
         WHERE conversation_id = ? ORDER BY COALESCE(sent_at, created_at) ASC LIMIT ?",
    )
    .bind(conversation_id)
    .bind(MAX_HISTORY_MESSAGES)
    .fetch_all(&state.db)
    .await
    .map_err(|err| ReplyError::DbFailed {
        context: format!("read the message history of conversation {conversation_id}"),
        reason: err.to_string(),
    })?;

    let message_count = rows.len();
    let mut lines = Vec::with_capacity(message_count);
    let mut last_inbound: Option<String> = None;
    for row in rows {
        let direction: String = row
            .try_get::<Option<String>, _>("direction")
            .ok()
            .flatten()
            .unwrap_or_default();
        let body: String = row
            .try_get::<Option<String>, _>("body")
            .ok()
            .flatten()
            .unwrap_or_default();
        let label = match direction.to_ascii_uppercase().as_str() {
            "INBOUND" => "INBOUND",
            "OUTBOUND" => "OUTBOUND",
            _ => "UNKNOWN DIRECTION",
        };
        lines.push(format!("{label}: {body}"));
        if direction.eq_ignore_ascii_case("inbound") && !body.trim().is_empty() {
            last_inbound = Some(body);
        }
    }

    Ok(Thread {
        history: lines.join("\n"),
        last_inbound_body: last_inbound.unwrap_or_default(),
        message_count: message_count as i64,
    })
}

/// Real services for the lead's institution type, read from the `services` table.
async fn service_context(
    state: &AppState,
    tenant_key: &str,
    institution_type: Option<&str>,
    warnings: &mut Vec<String>,
) -> String {
    if !crate::routes::contacts::column_exists(&state.db, "services", "target_institution_types")
        .await
    {
        warnings.push(
            "services.target_institution_types is not in this schema, so no service context was \
             given to the model"
                .to_string(),
        );
        return String::new();
    }
    let rows = match sqlx::query(
        "SELECT name, COALESCE(summary, '') AS summary, COALESCE(details, '') AS details, \
                COALESCE(target_institution_types, '[]') AS targets \
         FROM services WHERE COALESCE(tenant_key, '') = ? ORDER BY name ASC",
    )
    .bind(tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            warnings.push(format!(
                "the service catalog could not be read, so the model answered without it: {err}"
            ));
            return String::new();
        }
    };

    let mut rendered = Vec::new();
    for row in rows {
        let name: String = row.try_get("name").unwrap_or_default();
        let summary: String = row.try_get("summary").unwrap_or_default();
        let details: String = row.try_get("details").unwrap_or_default();
        let targets_raw: String = row.try_get("targets").unwrap_or_else(|_| "[]".to_string());
        let targets: Vec<String> = serde_json::from_str::<Vec<Value>>(&targets_raw)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .collect();

        let matches_lead = match institution_type {
            // Without a known institution type every catalog entry is offered; a
            // blank target list means the service is general and always applies.
            None => true,
            Some(kind) => {
                targets.is_empty()
                    || kind.trim().is_empty()
                    || targets
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(kind.trim()))
            }
        };
        if !matches_lead {
            continue;
        }
        let detail = if summary.trim().is_empty() {
            details
        } else {
            summary
        };
        rendered.push(format!("- {name}: {}", detail.trim()));
    }

    if rendered.is_empty() {
        warnings.push(format!(
            "tenant '{tenant_key}' has no catalog services matching this lead, so the model had no \
             service grounding and was told to escalate instead of guessing"
        ));
        return "SERVICES: (none configured for this tenant)".to_string();
    }
    format!("SERVICES:\n{}\n", rendered.join("\n"))
}

async fn knowledge_context(
    state: &AppState,
    tenant_key: &str,
    warnings: &mut Vec<String>,
) -> String {
    let rows = match sqlx::query(
        "SELECT title, content FROM knowledge_base_entries WHERE COALESCE(tenant_key, '') = ? \
         ORDER BY updated_at DESC LIMIT ?",
    )
    .bind(tenant_key)
    .bind(MAX_KB_ENTRIES)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            warnings.push(format!(
                "the knowledge base could not be read, so the model answered without it: {err}"
            ));
            return String::new();
        }
    };

    let mut blocks = Vec::new();
    for row in rows {
        let title: String = row.try_get("title").unwrap_or_default();
        let content: String = row.try_get("content").unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }
        blocks.push(format!("### {title}\n{content}"));
    }
    if blocks.is_empty() {
        return String::new();
    }
    format!(
        "BUSINESS KNOWLEDGE BASE:\n{}\n\n",
        blocks.join("\n\n")
    )
}

async fn memory_context(
    state: &AppState,
    contact_id: &str,
    warnings: &mut Vec<String>,
) -> String {
    if !crate::routes::contacts::column_exists(&state.db, "contact_memories", "fact").await {
        return String::new();
    }
    let rows = match sqlx::query(
        "SELECT fact, COALESCE(category, '') AS category, confidence FROM contact_memories \
         WHERE contact_id = ? ORDER BY COALESCE(noted_at, created_at) DESC LIMIT ?",
    )
    .bind(contact_id)
    .bind(MAX_MEMORIES)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            warnings.push(format!(
                "the contact's stored memories could not be read, so they were left out: {err}"
            ));
            return String::new();
        }
    };

    let mut lines = Vec::new();
    for row in rows {
        let fact: String = row.try_get("fact").unwrap_or_default();
        let category: String = row.try_get("category").unwrap_or_default();
        // `confidence` is TEXT in this schema; a value that does not parse stays
        // unknown rather than becoming 0 or 1.
        let confidence: Option<f64> = row
            .try_get::<Option<String>, _>("confidence")
            .ok()
            .flatten()
            .and_then(|raw| raw.trim().parse::<f64>().ok());
        if fact.trim().is_empty() {
            continue;
        }
        let suffix = match confidence {
            Some(value) => format!(" (confidence {value})"),
            None => String::new(),
        };
        let prefix = if category.trim().is_empty() {
            String::new()
        } else {
            format!("[{category}] ")
        };
        lines.push(format!("- {prefix}{fact}{suffix}"));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("KNOWN ABOUT THIS CONTACT:\n{}\n\n", lines.join("\n"))
}

/// Build the lead-facing user prompt from real rows only. Returns the prompt,
/// the thread it was built from, the lead facts (also needed for the
/// hot-lead alert text), the knowledge context string (needed for the
/// validation layer to check grounding), and the knowledge sources used
/// (for Phase 37 tracking) so the caller does not re-query any of these.
async fn build_user_prompt(
    state: &AppState,
    conversation_id: &str,
    contact: &sqlx::postgres::PgRow,
    tenant_key: &str,
    warnings: &mut Vec<String>,
) -> Result<(String, Thread, LeadFacts, String, Vec<crate::services::reply_integration::KnowledgeSourceUsed>), ReplyError> {
    let thread = load_history(state, conversation_id).await?;
    if thread.message_count == 0 {
        warnings.push(
            "this conversation has no stored messages, so the model had no history to answer from"
                .to_string(),
        );
    }

    let contact_id: String = contact.try_get("id").unwrap_or_default();

    let lead_id: Option<String> = contact
        .try_get::<Option<String>, _>("lead_id")
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty());

    let mut lead_line = String::from("LEAD: (no lead row is linked to this contact)\n");
    let mut lead_facts = LeadFacts::default();
    let mut institution_type: Option<String> = None;
    if let Some(lead_id) = lead_id {
        match sqlx::query(
            "SELECT name, COALESCE(institution_type, '') AS institution_type, \
                    COALESCE(city, '') AS city, \
                    COALESCE(qualification_rationale, '') AS qualification_rationale \
             FROM leads WHERE id = ? LIMIT 1",
        )
        .bind(&lead_id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some(row)) => {
                let name: String = row.try_get("name").unwrap_or_default();
                let kind: String = row.try_get("institution_type").unwrap_or_default();
                let city: String = row.try_get("city").unwrap_or_default();
                let rationale: String = row.try_get("qualification_rationale").unwrap_or_default();
                if !kind.trim().is_empty() {
                    institution_type = Some(kind.clone());
                }
                lead_facts = LeadFacts {
                    name: (!name.trim().is_empty()).then(|| name.clone()),
                    institution_type: (!kind.trim().is_empty()).then(|| kind.clone()),
                    city: (!city.trim().is_empty()).then(|| city.clone()),
                };
                lead_line = format!(
                    "LEAD: {} ({}, {})\nQUALIFICATION: {}\n",
                    if name.trim().is_empty() {
                        "(unnamed lead)"
                    } else {
                        name.trim()
                    },
                    if kind.trim().is_empty() {
                        "institution type unknown"
                    } else {
                        kind.trim()
                    },
                    if city.trim().is_empty() {
                        "city unknown"
                    } else {
                        city.trim()
                    },
                    if rationale.trim().is_empty() {
                        "(no qualification rationale recorded)"
                    } else {
                        rationale.trim()
                    }
                );
            }
            Ok(None) => warnings.push(format!(
                "contact links to lead {lead_id} but that lead row does not exist, so no lead \
                 profile was given to the model"
            )),
            Err(err) => warnings.push(format!(
                "the linked lead row could not be read: {err}"
            )),
        }
    } else {
        warnings.push(
            "this contact has no linked lead, so the model answered without lead qualification data"
                .to_string(),
        );
    }

    let services = service_context(state, tenant_key, institution_type.as_deref(), warnings).await;
    let knowledge = knowledge_context(state, tenant_key, warnings).await;
    let memories = memory_context(state, &contact_id, warnings).await;

    // Combine services and knowledge base for the validator's grounding check.
    let knowledge_context = format!("{}{}", services, knowledge);

    // Phase 37: Use ReplyIntegrationService to combine all knowledge sources
    // (KB entries, help center, document chunks, audience scoping, tone/guidance)
    let integration_service = ReplyIntegrationService::new(&state.db);
    let context_package = integration_service.build_context_package(
        Some(&contact_id),
        &thread.last_inbound_body,
        tenant_key,
    ).await;

    let (integration_knowledge, tone_instructions, guidance_instructions, sources_used) = match context_package {
        Ok(pkg) => {
            (pkg.knowledge_context, pkg.tone_instructions, pkg.guidance_instructions, pkg.sources_used)
        }
        Err(e) => {
            warnings.push(format!("reply integration context build failed: {e}, using legacy context only"));
            (String::new(), String::new(), String::new(), Vec::new())
        }
    };

    // Store sources_used in warnings for later tracking (we'll pass them through the return)
    // We return them as part of the tuple so generate_reply can track them

    // Use integration knowledge if available (supersedes legacy knowledge), otherwise fall back
    let integration_knowledge_empty = integration_knowledge.is_empty();
    let final_knowledge = if integration_knowledge_empty {
        knowledge
    } else {
        integration_knowledge.clone()
    };

    // Combine for validator grounding check: use integration knowledge if available
    let final_knowledge_context = if integration_knowledge_empty {
        knowledge_context
    } else {
        integration_knowledge
    };

    let prompt = format!(
        "{lead_line}\n{services}\n{final_knowledge}{memories}{tone_instructions}{guidance_instructions}\
         CONVERSATION HISTORY:\n{}\n\nDraft the next reply to send to this lead.",
        if thread.history.trim().is_empty() {
            "(no messages recorded in this thread)"
        } else {
            &thread.history
        },
    );
    Ok((prompt, thread, lead_facts, final_knowledge_context, sources_used))
}

/// Port of `generate_reply`. Returns the stored draft, or the reason nothing was
/// stored.
pub async fn generate_reply(
    state: &AppState,
    conversation_id: &str,
    request_tenant: Option<&str>,
) -> Result<GeneratedDraft, ReplyError> {
    let conversation = sqlx::query(
        "SELECT id, COALESCE(contact_id, '') AS contact_id, state, \
                COALESCE(resolution_type, '') AS resolution_type \
         FROM conversations WHERE id = ? LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| ReplyError::DbFailed {
        context: format!("read conversation {conversation_id}"),
        reason: err.to_string(),
    })?;

    let Some(conversation) = conversation else {
        return Err(ReplyError::NotFound(conversation_id.to_string()));
    };
    let state_value: String = conversation.try_get("state").unwrap_or_default();
    if INELIGIBLE_STATES
        .iter()
        .any(|s| state_value.eq_ignore_ascii_case(s))
    {
        return Err(ReplyError::NotEligible {
            conversation_id: conversation_id.to_string(),
            state: state_value,
        });
    }

    let contact_id: String = conversation.try_get("contact_id").unwrap_or_default();
    if contact_id.trim().is_empty() {
        return Err(ReplyError::ContactMissing(conversation_id.to_string()));
    }

    let (projection, _missing) = crate::routes::contacts::contact_projection(&state.db).await;
    let select_sql = format!(
        "SELECT {} FROM contacts WHERE id = ? LIMIT 1",
        projection.join(", ")
    );
    let contact = sqlx::query(&select_sql)
        .bind(&contact_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| ReplyError::DbFailed {
            context: format!("read contact {contact_id}"),
            reason: err.to_string(),
        })?;
    let Some(contact) = contact else {
        return Err(ReplyError::ContactMissing(conversation_id.to_string()));
    };

    // Tenant resolution order: the request's own tenant, then the stored rows.
    // Never a compiled-in default: an unscoped reply would read another
    // tenant's knowledge base and catalog.
    let stored_tenant: Option<String> = contact
        .try_get::<Option<String>, _>("tenant_key")
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty());
    let conv_tenant: Option<String> = sqlx::query_scalar::<_, Option<String>>(
        "SELECT tenant_key FROM conversations WHERE id = ? LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten()
    .filter(|v| !v.trim().is_empty());

    let tenant_key = request_tenant
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or(stored_tenant)
        .or(conv_tenant)
        .or_else(|| {
            let configured = state.config.default_tenant_key.trim();
            if configured.is_empty() {
                None
            } else {
                Some(configured.to_string())
            }
        });
    let Some(tenant_key) = tenant_key else {
        return Err(ReplyError::DbFailed {
            context: "resolve which tenant this reply belongs to".to_string(),
            reason: format!(
                "the request carries no tenant, and neither contact {contact_id} nor conversation \
                 {conversation_id} has a stored tenant_key, so reading this tenant's catalog and \
                 knowledge base would be unscoped"
            ),
        });
    };

    let channel: String = contact.try_get("channel").unwrap_or_default();
    let contact_value: String = contact.try_get("value").unwrap_or_default();

    let mut warnings: Vec<String> = Vec::new();
    warnings.push(
        "relationship history across the contact's other conversations (Python \
         services/relationship_profile.py) has no Rust port yet, so that section was left out of \
         the prompt"
            .to_string(),
    );

    let system_prompt = build_system_prompt(state, &tenant_key, &mut warnings).await;
    let (user_prompt, thread, lead_facts, knowledge_context, knowledge_sources) =
        build_user_prompt(state, conversation_id, &contact, &tenant_key, &mut warnings).await?;

    let parsed = match ask_json(state, &system_prompt, &user_prompt, REPLY_MAX_TOKENS).await {
        Ok(parsed) => parsed,
        Err(err) => {
            if err.is_local_refusal() {
                return Err(ReplyError::AiNotConfigured(err.message()));
            }
            return Err(ReplyError::AiFailed(err.message()));
        }
    };

    let model = parse_model_reply(&parsed);
    for field in &model.ignored_fields {
        warnings.push(format!(
            "the model returned a {field} that is not the expected type, so it was stored as absent"
        ));
    }
    if model.intent.is_none() {
        warnings.push(
            "the model did not return an intent, so no escalation or hot-lead routing decision \
             could be made from it"
                .to_string(),
        );
    }

    let Some(body) = model.reply.clone() else {
        return Err(ReplyError::AiFailed(
            "the model answered with valid JSON but no usable `reply` field, so nothing was drafted"
                .to_string(),
        ));
    };

    // Detected language is written back only when the column exists.
    let mut language_updated = false;
    if let Some(detected) = model.language.as_deref() {
        if detected.len() <= 10 {
            let stored_language: Option<String> = contact
                .try_get::<Option<String>, _>("language")
                .ok()
                .flatten();
            if stored_language.as_deref() != Some(detected) {
                let updated = sqlx::query("UPDATE contacts SET language = ? WHERE id = ?")
                    .bind(detected)
                    .bind(&contact_id)
                    .execute(&state.db)
                    .await;
                match updated {
                    Ok(report) if report.rows_affected() == 1 => language_updated = true,
                    Ok(report) => warnings.push(format!(
                        "contacts.language was not updated to '{detected}': the UPDATE affected {} rows",
                        report.rows_affected()
                    )),
                    Err(err) => warnings.push(format!(
                        "contacts.language could not be updated to '{detected}': {err}"
                    )),
                }
            }
        }
    }

    let mut verification_session_id: Option<String> = None;
    let mut verification_url: Option<String> = None;
    let body = maybe_handle_refund_claim(
        state,
        conversation_id,
        &contact_id,
        &contact_value,
        &tenant_key,
        model.intent.as_deref(),
        &thread.last_inbound_body,
        body,
        &mut verification_session_id,
        &mut verification_url,
        &mut warnings,
    )
    .await;

    // --- Validation Layer (Phase 7) ---
    // Before deciding the draft status, run an independent validation check.
    // If validation fails, the draft is forced to pending_review regardless of
    // the confidence threshold.
    let validation_config = validation::ValidationConfig::from_env();
    let validation_result = validation::validate_draft(
        state,
        &validation_config,
        &thread.last_inbound_body,
        &body,
        &knowledge_context,
    )
    .await;

    if validation_result.validated {
        if !validation_result.passed {
            warnings.push(format!(
                "validation failed: {}",
                validation_result.reason.as_deref().unwrap_or("see issues")
            ));
            for issue in &validation_result.issues {
                warnings.push(format!("validation issue: {issue}"));
            }
        } else {
            warnings.push(format!(
                "validation passed (relevance: {:.2}, grounding: {:.2}, latency: {}ms)",
                validation_result.relevance_score,
                validation_result.grounding_score,
                validation_result.latency_ms,
            ));
        }
    } else {
        // Validation was skipped — log the reason but don't block.
        if let Some(reason) = &validation_result.reason {
            warnings.push(format!("validation skipped: {reason}"));
        }
    }

    let escalate = model
        .intent
        .as_deref()
        .is_some_and(|intent| ESCALATING_INTENTS.iter().any(|i| i.eq_ignore_ascii_case(intent)));

    let threshold = state.config.reply_auto_send_confidence;
    let mut status = String::from("pending_review");
    if escalate {
        warnings.push(
            "the model flagged this reply as an escalation, so it stays with a human regardless of \
             confidence"
                .to_string(),
        );
    }
    match (threshold, model.confidence, escalate) {
        (Some(limit), Some(confidence), false) if confidence >= limit => {
            // Even if confidence is high, validation failure forces pending_review.
            if validation_result.validated && !validation_result.passed {
                warnings.push(
                    "confidence exceeds auto-send threshold but validation failed — \
                     draft remains pending_review for human approval"
                        .to_string(),
                );
            } else {
                status = String::from("auto_approved");
            }
        }
        (None, _, false) => warnings.push(
            "REPLY_AUTO_SEND_CONFIDENCE is not set, so nothing was auto-approved; the draft waits \
             for a human"
                .to_string(),
        ),
        (_, None, false) => warnings.push(
            "the model reported no confidence, so the draft cannot be auto-approved"
                .to_string(),
        ),
        _ => {}
    }

    // Conversation bookkeeping, mirroring reply_engine.py:217-233.
    let mut resolution_type: Option<String> = None;
    let mut conversation_state: Option<String> = None;
    if status == "auto_approved" && !escalate && model.confidence.is_some_and(|c| c >= 0.85) {
        resolution_type = Some("autonomous_no_handoff".to_string());
    } else if escalate {
        resolution_type = Some("escalated".to_string());
    }
    if model
        .intent
        .as_deref()
        .is_some_and(|i| HOT_LEAD_INTENTS.iter().any(|h| h.eq_ignore_ascii_case(i)))
    {
        conversation_state = Some("hot_lead".to_string());
    }
    if let Some(new_state) = conversation_state.as_deref() {
        let updated = sqlx::query("UPDATE conversations SET state = ? WHERE id = ?")
            .bind(new_state)
            .bind(conversation_id)
            .execute(&state.db)
            .await;
        match updated {
            Ok(report) if report.rows_affected() == 1 => {}
            Ok(report) => {
                warnings.push(format!(
                    "conversation state was not set to '{new_state}': the UPDATE affected {} rows",
                    report.rows_affected()
                ));
                conversation_state = None;
            }
            Err(err) => {
                warnings.push(format!(
                    "conversation state could not be set to '{new_state}': {err}"
                ));
                conversation_state = None;
            }
        }
    }
    if let Some(resolution) = resolution_type.as_deref() {
        let updated = sqlx::query("UPDATE conversations SET resolution_type = ? WHERE id = ?")
            .bind(resolution)
            .bind(conversation_id)
            .execute(&state.db)
            .await;
        match updated {
            Ok(report) if report.rows_affected() == 1 => {}
            Ok(report) => warnings.push(format!(
                "conversation resolution_type was not set to '{resolution}': the UPDATE affected {} \
                 rows",
                report.rows_affected()
            )),
            Err(err) => warnings.push(format!(
                "conversation resolution_type could not be set to '{resolution}': {err}"
            )),
        }
    }

    let draft_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let stored_confidence = model.confidence.unwrap_or(0.0);
    if model.confidence.is_none() {
        warnings.push(
            "drafts.confidence is a NOT NULL column and the model gave none, so 0.0 was stored as \
             the schema's absent marker; the response reports confidence as null"
                .to_string(),
        );
    }
    let citations_json = serde_json::to_string(&model.citations).unwrap_or_else(|_| "[]".to_string());

    let inserted = sqlx::query(
        "INSERT INTO drafts (id, conversation_id, body, confidence, citations, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&draft_id)
    .bind(conversation_id)
    .bind(&body)
    .bind(stored_confidence)
    .bind(&citations_json)
    .bind(&status)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    let draft_written = match inserted {
        Ok(report) if report.rows_affected() == 1 => true,
        Ok(report) => {
            warnings.push(format!(
                "the drafted reply was NOT stored: the drafts INSERT affected {} rows",
                report.rows_affected()
            ));
            false
        }
        Err(err) => {
            warnings.push(format!(
                "the drafted reply was NOT stored by the database: {err}"
            ));
            false
        }
    };

    let audit_details = json!({
        "intent": model.intent,
        "sentiment": model.sentiment,
        "confidence": model.confidence,
        "language": model.language,
        "citations": model.citations,
        "status": status,
        "resolution_type": resolution_type,
        "liveProofSessionId": verification_session_id,
        "aiModel": state.config.anthropic_model,
    });
    let audit_note = record_draft_audit(
        state,
        &draft_id,
        "draft_generated",
        "ai",
        &format!("AI drafted a reply for conversation {conversation_id}"),
        audit_details,
    )
    .await;
    if let Some(note) = audit_note {
        warnings.push(note);
    }

    // Phase 37: Track knowledge sources used in this reply
    if !knowledge_sources.is_empty() && draft_written {
        let integration_service = ReplyIntegrationService::new(&state.db);
        if let Err(e) = integration_service.track_knowledge_sources(&draft_id, conversation_id, &knowledge_sources).await {
            warnings.push(format!("failed to track knowledge sources: {e}"));
        }
    }

    // Hot-lead paging: reported whether or not it worked.
    let alert: Option<String>;
    let needs_alert = model
        .intent
        .as_deref()
        .is_some_and(|i| ALERT_INTENTS.iter().any(|a| a.eq_ignore_ascii_case(i)))
        || model
            .sentiment
            .as_deref()
            .is_some_and(|s| ALERT_SENTIMENTS.iter().any(|s2| s2.eq_ignore_ascii_case(s)));
    if needs_alert {
        let outcome = send_hot_lead_alert(
            state,
            &HotLeadAlert {
                lead_name: lead_facts.name.as_deref(),
                institution_type: lead_facts.institution_type.as_deref(),
                city: lead_facts.city.as_deref(),
                channel: &channel,
                intent: model.intent.as_deref().unwrap_or("(model returned no intent)"),
                sentiment: model.sentiment.as_deref().unwrap_or("(model returned no sentiment)"),
                conversation_id,
                message_body: &thread.last_inbound_body,
            },
        )
        .await;
        alert = Some(outcome.describe());
    } else {
        alert = Some(
            "no sales alert sent: neither the intent nor the sentiment met the alerting threshold"
                .to_string(),
        );
    }

    Ok(GeneratedDraft {
        id: draft_id,
        conversation_id: conversation_id.to_string(),
        body,
        status,
        created_at: now,
        model,
        stored_confidence,
        auto_send_threshold: threshold,
        escalation: escalate,
        language_updated,
        conversation_state,
        resolution_type,
        verification_session_id,
        verification_url,
        alert,
        warnings,
        draft_written,
        validation: validation_result,
    })
}

/// Port of the LiveProof branch (reply_engine.py:152-209).
async fn maybe_handle_refund_claim(
    state: &AppState,
    conversation_id: &str,
    contact_id: &str,
    contact_value: &str,
    tenant_key: &str,
    intent: Option<&str>,
    last_inbound: &str,
    mut body: String,
    session_id_out: &mut Option<String>,
    url_out: &mut Option<String>,
    warnings: &mut Vec<String>,
) -> String {
    let lowered = last_inbound.to_lowercase();
    let is_policy_query = lowered.contains("policy");
    let has_claim_keyword = claim_regex().is_match(last_inbound)
        || return_phrase_regex().is_match(last_inbound);
    let intent_claims = intent
        .is_some_and(|i| i.eq_ignore_ascii_case("refund_request") || i.eq_ignore_ascii_case("damaged_item"));
    if is_policy_query || !(intent_claims || has_claim_keyword) {
        return body;
    }

    let order_id = order_id_regex()
        .captures(last_inbound)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));

    let claim_type = if intent_claims {
        intent.unwrap_or("refund").to_string()
    } else {
        "refund".to_string()
    };

    // Reuse a pending session for this conversation instead of opening a second.
    let existing = sqlx::query(
        "SELECT id, session_token FROM live_proof_sessions WHERE conversation_id = ? \
         AND status = 'pending' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&state.db)
    .await;

    let created = match existing {
        Ok(Some(row)) => {
            let id = row.try_get::<String, _>("id").ok();
            let token = row.try_get::<String, _>("session_token").ok();
            match (id, token) {
                (Some(id), Some(token)) => Some((id, token)),
                _ => {
                    warnings.push(
                        "a pending LiveProof session exists for this conversation but its id or \
                         token could not be read, so no verification link was added"
                            .to_string(),
                    );
                    return body;
                }
            }
        }
        Ok(None) => {
            let spec = crate::routes::live_proof::NewLiveProofSession {
                tenant_id: tenant_key.to_string(),
                organization_id: None,
                conversation_id: Some(conversation_id.to_string()),
                contact_id: Some(contact_id.to_string()),
                order_id: order_id.clone(),
                customer_identifier: contact_value.to_string(),
                claim_type: claim_type.clone(),
                ttl_minutes: crate::routes::live_proof::DEFAULT_TTL_MINUTES,
                max_video_seconds: crate::routes::live_proof::DEFAULT_MAX_VIDEO_SECONDS,
                require_video: true,
                require_photos: true,
            };
            match crate::routes::live_proof::create_pending_session(state, &spec).await {
                Ok(created) => Some((created.id, created.session_token)),
                Err(reason) => {
                    warnings.push(format!(
                        "this looks like a refund claim but no verification session could be \
                         created: {reason}"
                    ));
                    return body;
                }
            }
        }
        Err(err) => {
            warnings.push(format!(
                "existing pending LiveProof sessions could not be checked, so no verification link \
                 was added: {err}"
            ));
            return body;
        }
    };

    let Some((session_id, token)) = created else {
        return body;
    };

    let base = state.config.public_url.trim().trim_end_matches('/').to_string();
    if base.is_empty() {
        warnings.push(
            "PUBLIC_URL is not configured, so the reply does not contain a verification link even \
             though a LiveProof session was opened"
                .to_string(),
        );
        *session_id_out = Some(session_id);
        return body;
    }

    let url = format!("{base}/verify/proof/{session_id}?token={token}");
    *session_id_out = Some(session_id);
    *url_out = Some(url.clone());
    if body.contains(&url) {
        return body;
    }
    body.push_str(&format!(
        "\n\nTo process your refund claim immediately, please verify your item using our secure \
         live camera portal:\n{url}\n\nNote: for fraud prevention only real-time direct camera \
         capture is accepted; photo gallery uploads are disabled."
    ));
    body
}
