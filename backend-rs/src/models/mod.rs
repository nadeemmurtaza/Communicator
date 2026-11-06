#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Email,
    Sms,
    Whatsapp,
    Website,
    Facebook,
    Instagram,
    Telegram,
    Line,
    Voice,
    Tiktok,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AIProvider {
    Anthropic,
    Openai,
    Google,
    Deepseek,
    Groq,
    Ollama,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AIJobType {
    SelfImproving,
    AutonomousAgent,
    CopilotDrafts,
    Simulations,
    QualityTriage,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    Unknown,
    OptedIn,
    OptedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BroadcastStatus {
    Draft,
    Scheduled,
    Sending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipientStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    Refund,
    Discount,
    PolicyException,
    InfoRequest,
    ReplyApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Amended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InquiryStatus {
    DiscoveryInProgress,
    DiscoveryComplete,
    AwaitingReview,
    Qualified,
    MeetingPending,
    MeetingBooked,
    ProposalPending,
    Won,
    Lost,
    Nurture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayType {
    Sms,
    Email,
    VoiceSip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveProofStatus {
    Pending,
    InProgress,
    Captured,
    Verified,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofMediaType {
    LivePhoto,
    LiveVideo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Active,
    Draft,
    Testing,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationVerdict {
    Passed,
    Failed,
    Escalated,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightCategory {
    HumanExemplar,
    Trend,
    PolicyRefinement,
    ContentGap,
    TeamCoaching,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightStatus {
    Open,
    Applied,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Sent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Queued,
    Sent,
    Delivered,
    Read,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationState {
    Open,
    Pending,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    Open,
    InProgress,
    Pending,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketType {
    Support,
    Bug,
    Feature,
    Complaint,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketDepartment {
    Support,
    Sales,
    Engineering,
    Billing,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Comment,
    StatusChange,
    Assignment,
    Created,
    Updated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueTrackerProvider {
    Linear,
    Github,
    Jira,
    Slack,
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

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
    /// Whether the provider actually responded to a live API probe.
    /// `None` means the probe was not attempted (e.g. credentials missing).
    pub real_connection: Option<bool>,
    /// Error from the last live API probe, if it failed.
    pub connection_error: Option<String>,
    /// Timestamp of the last successful live API probe.
    pub last_verified_at: Option<String>,
    /// Additional details from the provider (e.g. Meta quality rating, Twilio account status).
    pub provider_info: Option<serde_json::Value>,
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

// ---------------------------------------------------------------------------
// Additional Models (translated from Python)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub owner_account_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Lead {
    pub id: String,
    pub tenant_key: String,
    pub harvester_institution_id: String,
    pub name: String,
    pub institution_type: String,
    pub city: String,
    pub qualification_rationale: String,
    pub synced_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub id: String,
    pub tenant_key: String,
    pub lead_id: Option<String>,
    pub display_name: String,
    pub customer_id: Option<String>,
    pub brand_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub designation: String,
    pub role: String,
    pub industry: String,
    pub avatar_url: String,
    pub timezone: String,
    pub language: String,
    pub customer_type: String,
    pub customer_segment: String,
    pub customer_tier: String,
    pub lifetime_value: f64,
    pub total_orders: i64,
    pub total_spent: f64,
    pub last_order_date: Option<String>,
    pub first_contact_date: Option<String>,
    pub engagement_score: f64,
    pub sentiment_score: f64,
    pub notes: String,
    pub tags: String,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ConsentRecord {
    pub id: String,
    pub contact_id: String,
    pub status: String,
    pub source: String,
    pub note: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ContactMemory {
    pub id: String,
    pub contact_id: String,
    pub fact: String,
    pub category: String,
    pub confidence: f64,
    pub source_message_id: Option<String>,
    pub noted_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ContactNote {
    pub id: String,
    pub contact_id: String,
    pub author_name: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Inbox {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub phone_number: String,
    pub avatar_url: String,
    pub is_active: bool,
    pub credentials: Option<String>,
    pub greeting_enabled: bool,
    pub greeting_message: String,
    pub out_of_office_enabled: bool,
    pub out_of_office_message: String,
    pub csat_enabled: bool,
    pub auto_read_receipts: bool,
    pub working_hours_enabled: bool,
    pub working_hours_start: i32,
    pub working_hours_end: i32,
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSettings {
    pub id: String,
    pub tenant_key: String,
    pub whatsapp_provider: String,
    pub meta_access_token: String,
    pub meta_phone_number_id: String,
    pub meta_waba_id: String,
    pub meta_app_secret: String,
    pub meta_webhook_verify_token: String,
    pub whatsapp_verified_at: Option<String>,
    pub whatsapp_verified_name: String,
    pub whatsapp_display_number: String,
    pub business_name: String,
    pub business_description: String,
    pub agent_guidelines: String,
    pub auto_reply_enabled: bool,
    pub business_hours_enabled: bool,
    pub business_hours_start: i32,
    pub business_hours_end: i32,
    pub business_timezone: String,
    pub away_message: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChannelGatewayConnection {
    pub id: String,
    pub gateway_type: String,
    pub provider: String,
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub credentials: Option<String>,
    pub config: Option<String>,
    pub test_status: String,
    pub latency_ms: i32,
    pub last_tested_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Broadcast {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub campaign_type: String,
    pub objective: String,
    pub campaign_template_id: Option<String>,
    pub parent_campaign_id: Option<String>,
    pub step_index: i32,
    pub step_delay_days: i32,
    pub stop_on_reply: bool,
    pub template_name: String,
    pub template_language: String,
    pub body_preview: String,
    pub template_params: Option<String>,
    pub tag_name: String,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub audience_count: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub read_count: i32,
    pub failed_count: i32,
    pub skipped_count: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastRecipient {
    pub id: String,
    pub broadcast_id: String,
    pub contact_id: String,
    pub status: String,
    pub provider_message_id: String,
    pub error: String,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTemplate {
    pub id: String,
    pub tenant_key: String,
    pub key: String,
    pub channel: String,
    pub name: String,
    pub purpose: String,
    pub meta_template_name: String,
    pub meta_language: String,
    pub meta_category: String,
    pub meta_status: String,
    pub body_preview: String,
    pub variables: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AIConnection {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub temperature: f64,
    pub max_tokens: i32,
    pub is_active: bool,
    pub assigned_jobs: Option<String>,
    pub last_tested_at: Option<String>,
    pub test_status: String,
    pub latency_ms: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AICustomPolicy {
    pub id: String,
    pub job_type: String,
    pub tone: String,
    pub guidelines: String,
    pub rules: Option<String>,
    pub guardrails: Option<String>,
    pub auto_approval_mode: String,
    pub min_csat_learning_threshold: f64,
    pub max_autonomous_refund: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CopilotApprovalRequest {
    pub id: String,
    pub conversation_id: String,
    pub contact_id: Option<String>,
    pub request_type: String,
    pub status: String,
    pub title: String,
    pub proposed_reply: String,
    pub amended_reply: Option<String>,
    pub action_data: Option<String>,
    pub internal_note: String,
    pub question_to_teammate: Option<String>,
    pub teammate_answer: Option<String>,
    pub assigned_to_id: Option<String>,
    pub reviewer_name: Option<String>,
    pub reviewer_feedback: Option<String>,
    pub dispatched_message_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub actor: String,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Inquiry {
    pub id: String,
    pub tenant_key: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub lead_id: Option<String>,
    pub organization_name: String,
    pub organization_status: String,
    pub industry: String,
    pub contact_role: String,
    pub org_size_range: String,
    pub locations_count: String,
    pub problem_description: String,
    pub problem_categories: Option<String>,
    pub reported_impact: Option<String>,
    pub frequency: String,
    pub impact_level: String,
    pub current_workaround: String,
    pub limitations: Option<String>,
    pub desired_outcome: String,
    pub existing_systems: Option<String>,
    pub integration_required: String,
    pub capability_mapping: Option<String>,
    pub timeline: String,
    pub decision_process: String,
    pub budget_range: String,
    pub qualification_score: Option<i32>,
    pub qualification_factors: Option<String>,
    pub status: String,
    pub assigned_owner: String,
    pub next_action: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryFlowTemplate {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub is_active: bool,
    pub steps: Option<String>,
    pub problem_category_keywords: Option<String>,
    pub capability_keywords: Option<String>,
    pub capability_labels: Option<String>,
    pub scoring_weights: Option<String>,
    pub intro_message: String,
    pub closing_message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub description: String,
    pub actions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ContactSegment {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub description: String,
    pub filters: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomAttributeDefinition {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub attribute_model: String,
    pub attribute_type: String,
    pub default_value: Option<String>,
    pub options: Option<String>,
    pub is_required: bool,
    pub regex_pattern: Option<String>,
    pub regex_cue: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LiveProofSession {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub order_id: Option<String>,
    pub customer_identifier: String,
    pub claim_type: String,
    pub session_token: String,
    pub status: String,
    pub anti_spoof_nonce: String,
    pub max_video_seconds: i32,
    pub require_video: bool,
    pub require_photos: bool,
    pub expires_at: String,
    pub completed_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LiveProofEvidence {
    pub id: String,
    pub session_id: String,
    pub tenant_id: String,
    pub media_type: String,
    pub file_path: String,
    pub file_size_bytes: i32,
    pub mime_type: String,
    pub sha256_hash: String,
    pub duration_seconds: Option<f64>,
    pub resolution: Option<String>,
    pub device_fingerprint: Option<String>,
    pub watermark_verified: bool,
    pub verification_score: f64,
    pub captured_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeGap {
    pub id: String,
    pub question: String,
    pub suggested_answer: String,
    pub source_query: String,
    pub frequency: i32,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRecommendation {
    pub id: String,
    pub gap_type: String,
    pub topic: String,
    pub title: String,
    pub description: String,
    pub impact_on_resolution: f64,
    pub suggested_action: String,
    pub suggested_content: String,
    pub status: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FollowUpSequence {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SequenceStep {
    pub id: String,
    pub sequence_id: String,
    pub step_number: i32,
    pub delay_days: i32,
    pub channel: String,
    pub template: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub summary: String,
    pub details: String,
    pub target_institution_types: Option<String>,
    pub faq: Option<String>,
    pub pricing_notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentPolicy {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub process_steps: Option<String>,
    pub rules: Option<String>,
    pub guardrails: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SimulationScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub policy_id: Option<String>,
    pub customer_persona: Option<String>,
    pub expected_outcome: String,
    pub customer_messages: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRun {
    pub id: String,
    pub scenario_id: String,
    pub policy_id: Option<String>,
    pub status: String,
    pub verdict: String,
    pub policy_adherence_score: f64,
    pub empathy_score: f64,
    pub accuracy_score: f64,
    pub overall_score: f64,
    pub turn_count: i32,
    pub transcript: Option<String>,
    pub evaluation_summary: String,
    pub recommendations: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentInsight {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub human_rep_id: Option<String>,
    pub human_rep_name: Option<String>,
    pub exemplar_content: Option<String>,
    pub impact_score: f64,
    pub estimated_resolution_lift: f64,
    pub suggested_action: String,
    pub action_payload: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentRetrainingEvent {
    pub id: String,
    pub title: String,
    pub trigger_type: String,
    pub version_from: String,
    pub version_to: String,
    pub insight_id: Option<String>,
    pub changes_applied: Option<String>,
    pub status: String,
    pub baseline_resolution_rate: f64,
    pub post_training_resolution_rate: f64,
    pub resolution_lift: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SSOConfig {
    pub id: String,
    pub is_enabled: bool,
    pub provider_type: String,
    pub idp_entity_id: String,
    pub idp_sso_url: String,
    pub idp_x509_cert: String,
    pub sp_entity_id: String,
    pub sp_acs_url: String,
    pub enforced_domains: Option<String>,
    pub allow_unlinked_passwords: bool,
    pub jit_provisioning_enabled: bool,
    pub default_role: String,
    pub attribute_mapping: Option<String>,
    pub last_tested_at: Option<String>,
    pub last_test_status: String,
    pub last_test_message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StripeConfig {
    pub id: String,
    pub account_name: String,
    pub api_key: String,
    pub publishable_key: String,
    pub webhook_secret: String,
    pub is_enabled: bool,
    pub test_mode: bool,
    pub default_currency: String,
    pub last_tested_at: Option<String>,
    pub last_test_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StripeBillingActionLog {
    pub id: String,
    pub contact_id: Option<String>,
    pub customer_identifier: String,
    pub action_type: String,
    pub amount_cents: i32,
    pub currency: String,
    pub status: String,
    pub stripe_id: String,
    pub actor_email: String,
    pub reason: String,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TranslationSettings {
    pub id: String,
    pub is_enabled: bool,
    pub auto_translate_inbound: bool,
    pub auto_translate_outbound: bool,
    pub agent_primary_language: String,
    pub preferred_engine: String,
    pub confidence_threshold: f64,
    pub enabled_languages: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub secret: String,
    pub events: Option<String>,
    pub headers: Option<String>,
    pub is_active: bool,
    pub retry_count: i32,
    pub timeout_seconds: i32,
    pub total_delivered: i32,
    pub total_failed: i32,
    pub last_status_code: Option<i32>,
    pub last_delivered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDeliveryLog {
    pub id: String,
    pub endpoint_id: String,
    pub event: String,
    pub payload: Option<String>,
    pub status_code: i32,
    pub response_headers: Option<String>,
    pub response_body: String,
    pub duration_ms: i32,
    pub attempt: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IssueTrackerConfig {
    pub id: String,
    pub provider: String,
    pub account_name: String,
    pub is_enabled: bool,
    pub test_mode: bool,
    pub credentials: Option<String>,
    pub config: Option<String>,
    pub last_tested_at: Option<String>,
    pub last_test_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LinkedIssue {
    pub id: String,
    pub conversation_id: Option<String>,
    pub ticket_id: Option<String>,
    pub provider: String,
    pub external_issue_id: String,
    pub issue_key: String,
    pub issue_url: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: String,
    pub created_by_email: String,
    pub sync_status: String,
    pub metadata_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SlackConversationSync {
    pub id: String,
    pub conversation_id: String,
    pub slack_channel_id: String,
    pub slack_channel_name: String,
    pub slack_thread_ts: String,
    pub is_synced: bool,
    pub last_synced_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TicketActivity {
    pub id: String,
    pub ticket_id: String,
    pub activity_type: String,
    pub actor_name: String,
    pub content: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Brand hierarchy models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: String,
    pub tenant_key: String,
    pub name: String,
    pub description: String,
    pub industry: String,
    pub website: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub timezone: String,
    pub settings: String,
    pub external_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub logo_url: String,
    pub tagline: String,
    pub color_scheme: String,
    pub bio: String,
    pub website: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub social_links: String,
    pub products: String,
    pub services: String,
    pub prices: String,
    pub policies: String,
    pub rules: String,
    pub tone_guidelines: String,
    pub keywords: String,
    pub external_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandCredentials {
    pub id: String,
    pub brand_id: String,
    pub channel_type: String,
    pub credentials: String,
    pub is_active: bool,
    pub last_verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandAIConfig {
    pub id: String,
    pub brand_id: String,
    pub ai_provider: String,
    pub api_key: String,
    pub model_id: String,
    pub base_url: String,
    pub temperature: f64,
    pub max_tokens: i64,
    pub system_prompt: String,
    pub rag_enabled: bool,
    pub embedding_provider: String,
    pub embedding_api_key: String,
    pub embedding_model: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandKnowledgeEntry {
    pub id: String,
    pub brand_id: String,
    pub title: String,
    pub content: String,
    pub source_type: String,
    pub topic: String,
    pub tags: String,
    pub embedding_vector: Option<Vec<u8>>,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Data Source Tracking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandProfileSource {
    pub id: String,
    pub brand_id: String,
    pub field_name: String,
    pub source_platform: String,
    pub source_user_id: String,
    pub source_user_name: String,
    pub source_record_id: String,
    pub source_url: String,
    pub synced_at: Option<String>,
    pub last_verified_at: Option<String>,
    pub is_verified: bool,
    pub confidence_score: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrgProfileSource {
    pub id: String,
    pub organization_id: String,
    pub field_name: String,
    pub source_platform: String,
    pub source_user_id: String,
    pub source_user_name: String,
    pub source_record_id: String,
    pub source_url: String,
    pub synced_at: Option<String>,
    pub last_verified_at: Option<String>,
    pub is_verified: bool,
    pub confidence_score: f64,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Live Sync Status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandSyncStatus {
    pub id: String,
    pub brand_id: String,
    pub platform: String,
    pub is_connected: bool,
    pub last_synced_at: Option<String>,
    pub next_sync_at: Option<String>,
    pub sync_interval_seconds: i64,
    pub last_error: String,
    pub connection_status: String,
    pub platform_data: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrgSyncStatus {
    pub id: String,
    pub organization_id: String,
    pub platform: String,
    pub is_connected: bool,
    pub last_synced_at: Option<String>,
    pub next_sync_at: Option<String>,
    pub sync_interval_seconds: i64,
    pub last_error: String,
    pub connection_status: String,
    pub platform_data: String,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Conflict Detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfileConflict {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub conflict_type: String,
    pub source_a_platform: String,
    pub source_a_value: String,
    pub source_a_last_updated: Option<String>,
    pub source_b_platform: String,
    pub source_b_value: String,
    pub source_b_last_updated: Option<String>,
    pub resolution_status: String,
    pub resolved_value: String,
    pub resolved_by: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Custom Sections
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCustomSection {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub section_name: String,
    pub section_type: String,
    pub display_order: i64,
    pub is_active: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCustomSectionData {
    pub id: String,
    pub section_id: String,
    pub row_key: String,
    pub field_key: String,
    pub field_value: String,
    pub field_type: String,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Pricing, Packages, Promotions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandPackage {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub duration_type: String,
    pub duration_value: i64,
    pub features: String,
    pub is_active: bool,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandPromotion {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub description: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub start_date: String,
    pub end_date: String,
    pub applicable_packages: String,
    pub is_active: bool,
    pub usage_limit: i64,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BrandVoucherSettings {
    pub id: String,
    pub brand_id: String,
    pub auto_generate: bool,
    pub voucher_prefix: String,
    pub voucher_length: i64,
    pub discount_type: String,
    pub discount_value: f64,
    pub validity_days: i64,
    pub trigger_event: String,
    pub max_uses_per_customer: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Message Triggers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageTrigger {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub description: String,
    pub trigger_type: String,
    pub trigger_event: String,
    pub trigger_schedule: String,
    pub trigger_conditions: String,
    pub preferred_channel: String,
    pub message_template: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageTriggerLog {
    pub id: String,
    pub trigger_id: String,
    pub contact_id: String,
    pub executed_at: String,
    pub channel_used: String,
    pub message_sent: String,
    pub status: String,
    pub error_message: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - Platform Data Cache
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PlatformDataCache {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub platform: String,
    pub data_key: String,
    pub data_value: String,
    pub synced_at: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Enhanced profile models - RAG Section Mapping
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RagSectionMapping {
    pub id: String,
    pub section_id: String,
    pub is_indexed: bool,
    pub embedding_status: String,
    pub last_indexed_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Unified Customer ID
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerChannelId {
    pub id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub channel_identifier: String,
    pub channel_profile_name: String,
    pub is_primary: bool,
    pub verified_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Data Sources
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientProfileSource {
    pub id: String,
    pub customer_id: String,
    pub field_name: String,
    pub source_type: String,
    pub source_user_id: String,
    pub source_user_name: String,
    pub source_record_id: String,
    pub source_url: String,
    pub synced_at: Option<String>,
    pub confidence_score: f64,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Self-Learning Engine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientLearningInsight {
    pub id: String,
    pub customer_id: String,
    pub insight_type: String,
    pub category: String,
    pub insight: String,
    pub confidence: f64,
    pub evidence_count: i64,
    pub source_conversation_id: Option<String>,
    pub source_message_id: Option<String>,
    pub extracted_at: String,
    pub is_verified: bool,
    pub verified_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientLearningPattern {
    pub id: String,
    pub customer_id: String,
    pub pattern_type: String,
    pub pattern_data: String,
    pub confidence: f64,
    pub sample_size: i64,
    pub last_updated: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Preferences
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientPreference {
    pub id: String,
    pub customer_id: String,
    pub preference_type: String,
    pub preference_value: String,
    pub confidence: f64,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Transactions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientOrder {
    pub id: String,
    pub customer_id: String,
    pub order_number: String,
    pub order_date: String,
    pub status: String,
    pub total_amount: f64,
    pub currency: String,
    pub items: String,
    pub payment_status: String,
    pub payment_method: String,
    pub channel: String,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientInvoice {
    pub id: String,
    pub customer_id: String,
    pub invoice_number: String,
    pub order_id: Option<String>,
    pub invoice_date: String,
    pub due_date: Option<String>,
    pub total_amount: f64,
    pub paid_amount: f64,
    pub remaining_amount: f64,
    pub currency: String,
    pub status: String,
    pub payment_method: String,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Payment Details
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientPaymentDetail {
    pub id: String,
    pub customer_id: String,
    pub detail_type: String,
    pub account_holder: String,
    pub account_number: String,
    pub bank_name: String,
    pub routing_number: String,
    pub card_last_four: String,
    pub is_default: bool,
    pub is_verified: bool,
    pub metadata: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Contact Points
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientContactPoint {
    pub id: String,
    pub customer_id: String,
    pub contact_type: String,
    pub contact_value: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Order Stage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientOrderStage {
    pub id: String,
    pub customer_id: String,
    pub stage_name: String,
    pub stage_value: i64,
    pub probability: f64,
    pub expected_close_date: Option<String>,
    pub current_order_id: Option<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Task Checklists
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientTaskChecklist {
    pub id: String,
    pub customer_id: String,
    pub checklist_name: String,
    pub checklist_type: String,
    pub is_active: bool,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientTaskItem {
    pub id: String,
    pub checklist_id: String,
    pub task_name: String,
    pub task_description: String,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub completed_by: String,
    pub due_date: Option<String>,
    pub priority: String,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Interaction History
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientInteractionHistory {
    pub id: String,
    pub customer_id: String,
    pub interaction_type: String,
    pub channel: String,
    pub direction: String,
    pub summary: String,
    pub sentiment: String,
    pub duration_seconds: i64,
    pub interaction_date: String,
    pub metadata: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - Custom Sections
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientCustomSection {
    pub id: String,
    pub customer_id: String,
    pub section_name: String,
    pub section_type: String,
    pub section_source: String,
    pub display_order: i64,
    pub is_active: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientCustomSectionData {
    pub id: String,
    pub section_id: String,
    pub row_key: String,
    pub field_key: String,
    pub field_value: String,
    pub field_type: String,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Client Profile models - RAG Mapping
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClientRagMapping {
    pub id: String,
    pub customer_id: String,
    pub section_type: String,
    pub section_id: String,
    pub is_indexed: bool,
    pub embedding_status: String,
    pub last_indexed_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// WhatsApp Channel models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppTemplate {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub category: Option<String>,
    pub language: Option<String>,
    pub status: String,
    pub body: String,
    pub components: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppCustomerProfile {
    pub id: String,
    pub brand_id: String,
    pub wa_id: String,
    pub profile_name: Option<String>,
    pub profile_picture_url: Option<String>,
    pub about: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_fetched_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppSessionWindow {
    pub id: String,
    pub brand_id: String,
    pub contact_id: String,
    pub started_at: String,
    pub expires_at: String,
    pub is_active: bool,
    pub message_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppAutomationState {
    pub id: String,
    pub brand_id: String,
    pub conversation_id: String,
    pub greeting_sent: bool,
    pub csat_sent: bool,
    pub last_menu: Option<String>,
    pub current_flow: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppTemplateLog {
    pub id: String,
    pub brand_id: String,
    pub template_name: String,
    pub template_id: Option<String>,
    pub contact_id: Option<String>,
    pub conversation_id: Option<String>,
    pub message_id: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppAutomationRule {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_value: Option<String>,
    pub action_type: String,
    pub action_config: Option<String>,
    pub is_active: bool,
    pub priority: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppBusinessProfileCache {
    pub id: String,
    pub brand_id: String,
    pub verified_name: Option<String>,
    pub about: Option<String>,
    pub address: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub websites: Option<String>,
    pub vertical: Option<String>,
    pub profile_picture_url: Option<String>,
    pub messaging_limit_tier: Option<String>,
    pub quality_rating: Option<String>,
    pub account_mode: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_synced_at: Option<String>,
}

// ---------------------------------------------------------------------------
// WhatsApp Campaigns, Flows, and Industries models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Industry {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TemplateGroup {
    pub id: String,
    pub brand_id: String,
    pub industry_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppTemplateFlow {
    pub id: String,
    pub brand_id: String,
    pub group_id: Option<String>,
    pub industry_id: Option<String>,
    pub name: String,
    pub r#type: String,
    pub category: Option<String>,
    pub body: String,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub buttons: Option<String>,
    pub sections: Option<String>,
    pub language: String,
    pub status: String,
    pub is_active: bool,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppFlowStep {
    pub id: String,
    pub flow_id: String,
    pub step_number: i64,
    pub condition: Option<String>,
    pub action_type: String,
    pub action_config: Option<String>,
    pub next_step_id: Option<String>,
    pub delay_seconds: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub r#type: String,
    pub template_flow_id: Option<String>,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub total_contacts: i64,
    pub sent_count: i64,
    pub delivered_count: i64,
    pub read_count: i64,
    pub failed_count: i64,
    pub retry_count: i64,
    pub progress: Option<String>,
    pub metadata: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CampaignContact {
    pub id: String,
    pub campaign_id: String,
    pub contact_id: String,
    pub status: String,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
    pub replied_at: Option<String>,
    pub retry_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageLimit {
    pub id: String,
    pub brand_id: String,
    pub period_start: String,
    pub period_end: String,
    pub utility_count: i64,
    pub marketing_count: i64,
    pub authentication_count: i64,
    pub customer_service_count: i64,
    pub total_limit: i64,
    pub remaining: i64,
    pub quality_rating: Option<String>,
    pub messaging_tier: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CampaignRetryQueue {
    pub id: String,
    pub campaign_id: String,
    pub contact_id: String,
    pub template_flow_id: Option<String>,
    pub retry_count: i64,
    pub max_retries: i64,
    pub next_retry_at: String,
    pub error: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
