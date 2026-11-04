use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub contact_id: Option<String>,
    pub contact_name: Option<String>,
    pub customer_name: Option<String>,
    pub lead_name: Option<String>,
    pub contact_phone: Option<String>,
    pub phone: Option<String>,
    pub channel: Option<String>,
    pub state: String,
    pub priority: Option<String>,
    pub resolution_type: Option<String>,
    pub broadcast_id: Option<String>,
    pub broadcast_name: Option<String>,
    pub inbox_id: Option<String>,
    pub team_id: Option<String>,
    pub team_name: Option<String>,
    pub csat_score: Option<i32>,
    pub human_takeover: bool,
    pub assigned_to_id: Option<String>,
    pub assigned_to_name: Option<String>,
    pub snoozed_until: Option<String>,
    pub language: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: String,
    pub custom_attributes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub direction: String, // "inbound" | "outbound"
    pub channel: String,
    pub body: String,
    pub intent: Option<String>,
    pub status: String,
    pub sent_at: String,
    pub provider_message_id: Option<String>,
    pub provider_metadata: Option<String>,
    pub sender_name: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub value: String, // phone or email
    pub channel: String,
    pub person_name: Option<String>,
    pub role: Option<String>,
    pub language: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessProfile {
    /// True only when Graph API answered 200 for the phone-number node.
    pub is_connected: bool,
    /// Meta's own error text when the call failed, so the UI can show the real
    /// reason instead of an invented status.
    pub error: Option<String>,
    pub display_phone_number: String,
    pub verified_name: String,
    pub quality_rating: String,
    pub messaging_limit_tier: String,
    pub official_business_account: bool,
    pub is_verified: bool,
    /// None when Meta did not report a quality rating -- distinct from "has
    /// spam flags", which an unknown rating must not be allowed to imply.
    pub zero_spam_flags: Option<bool>,
    pub account_mode: String,
    pub about: Option<String>,
    pub address: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub profile_picture_url: Option<String>,
    pub vertical: Option<String>,
    pub websites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelHealth {
    pub channel: String,
    pub is_connected: bool,
    pub status: String,
    pub status_label: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: String,
    pub conversation_id: String,
    pub body: String,
    pub confidence: f64,
    pub citations: Option<String>, // json array string
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: Option<String>,
    pub availability_status: Option<String>,
    pub max_concurrent_conversations: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub auto_assign: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBaseEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub topic: Option<String>,
    pub source_type: Option<String>,
    pub source_url: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_keyword: String,
    pub match_type: String,
    pub action_type: String,
    pub action_value: String,
    pub priority: i32,
    pub is_active: bool,
    pub execution_count: Option<i32>,
    pub last_triggered_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CannedResponse {
    pub id: String,
    pub short_code: String,
    pub content: String,
    pub category: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: String,
    pub ticket_number: String,
    pub title: String,
    pub description: String,
    pub ticket_type: String,
    pub department: String,
    pub priority: String,
    pub status: String,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub assigned_to_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RoleItem {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_system_role: Option<bool>,
    pub permissions: Option<String>,
    pub permissions_count: Option<i32>,
    pub assigned_members_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SlaPolicy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i32>,
    pub resolution_time_threshold: Option<i32>,
    pub only_during_business_hours: Option<bool>,
    pub priorities: Option<String>,
    pub inbox_ids: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
