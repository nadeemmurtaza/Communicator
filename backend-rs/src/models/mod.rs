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
    pub media_type: Option<String>,
    pub media_url: Option<String>,
    pub media_filename: Option<String>,
    pub media_mimetype: Option<String>,
    pub voice_duration: Option<i32>,
    pub transcription_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageFeedback {
    pub id: String,
    pub message_id: String,
    pub conversation_id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub author_type: String,
    pub author_id: Option<String>,
    pub created_at: String,
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

// ---------------------------------------------------------------------------
// Commerce Engine - Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => OrderStatus::Pending,
            "confirmed" => OrderStatus::Confirmed,
            "processing" => OrderStatus::Processing,
            "shipped" => OrderStatus::Shipped,
            "delivered" => OrderStatus::Delivered,
            "cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Draft,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Draft => "draft",
            OrderStatus::Pending => "pending",
            OrderStatus::Confirmed => "confirmed",
            OrderStatus::Processing => "processing",
            OrderStatus::Shipped => "shipped",
            OrderStatus::Delivered => "delivered",
            OrderStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Paid,
    PartiallyPaid,
    Refunded,
}

impl PaymentStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "paid" => PaymentStatus::Paid,
            "partially_paid" => PaymentStatus::PartiallyPaid,
            "refunded" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::PartiallyPaid => "partially_paid",
            PaymentStatus::Refunded => "refunded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPlatform {
    Shopify,
    WooCommerce,
    Wix,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipmentStatus {
    Pending,
    InTransit,
    Delivered,
    Returned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryEventType {
    PickedUp,
    InTransit,
    OutForDelivery,
    Delivered,
    Returned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryTransactionType {
    Inbound,
    Outbound,
    Adjustment,
    Transfer,
    Reserved,
    Released,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Product
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub brand_id: String,
    pub category_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub currency: String,
    pub images: Option<String>,
    pub variants: Option<String>,
    pub inventory_count: i32,
    pub track_inventory: bool,
    pub visibility: Option<String>,
    pub metadata: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariant {
    pub id: String,
    pub name: String,
    pub options: serde_json::Value,
    pub price_adjustment: f64,
    pub sku: Option<String>,
    pub inventory_count: i32,
    pub image: Option<String>,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Product Channel Visibility
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductChannelVisibility {
    pub id: String,
    pub product_id: String,
    pub channel_type: String,
    pub is_visible: bool,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Cart
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Cart {
    pub id: String,
    pub customer_id: String,
    pub brand_id: String,
    pub channel_type: Option<String>,
    pub status: String,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CartItem {
    pub id: String,
    pub cart_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Orders
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CommerceOrder {
    pub id: String,
    pub brand_id: String,
    pub customer_id: String,
    pub order_number: String,
    pub channel_type: Option<String>,
    pub status: String,
    pub total_amount: f64,
    pub subtotal: f64,
    pub tax: f64,
    pub shipping: f64,
    pub currency: String,
    pub items_snapshot: Option<String>,
    pub shipping_address: Option<String>,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub metadata: Option<String>,
    pub source: String,
    pub external_order_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CommerceOrderItem {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub product_name: String,
    pub sku: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub metadata: Option<String>,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Payments
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CommercePayment {
    pub id: String,
    pub order_id: String,
    pub amount: f64,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub transaction_id: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Sync
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncConnection {
    pub id: String,
    pub brand_id: String,
    pub platform: String,
    pub credentials: Option<String>,
    pub sync_status: String,
    pub last_sync_at: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncLog {
    pub id: String,
    pub connection_id: String,
    pub direction: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Discounts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Coupon {
    pub id: String,
    pub brand_id: String,
    pub code: String,
    pub r#type: String,
    pub value: f64,
    pub min_order_amount: Option<f64>,
    pub max_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub used_count: i32,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub applicable_products: Option<String>,
    pub applicable_categories: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CouponUsage {
    pub id: String,
    pub coupon_id: String,
    pub order_id: String,
    pub customer_id: String,
    pub discount_amount: f64,
    pub used_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MembershipTier {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub discount_percent: f64,
    pub monthly_price: Option<f64>,
    pub benefits: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerMembership {
    pub id: String,
    pub customer_id: String,
    pub membership_tier_id: String,
    pub started_at: String,
    pub ends_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Analytics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDailyStat {
    pub id: String,
    pub brand_id: String,
    pub date: String,
    pub orders_count: i32,
    pub revenue: f64,
    pub avg_order_value: f64,
    pub unique_customers: i32,
    pub products_sold: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsEvent {
    pub id: String,
    pub brand_id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub properties: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Delivery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShippingMethod {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub r#type: String,
    pub rate: f64,
    pub free_shipping_threshold: Option<f64>,
    pub regions: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Shipment {
    pub id: String,
    pub order_id: String,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub status: String,
    pub estimated_delivery: Option<String>,
    pub actual_delivery: Option<String>,
    pub shipping_label_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryEvent {
    pub id: String,
    pub shipment_id: String,
    pub event_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Storefront
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Storefront {
    pub id: String,
    pub brand_id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub theme: Option<String>,
    pub hero_image: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StorefrontSection {
    pub id: String,
    pub storefront_id: String,
    pub section_type: String,
    pub sort_order: i32,
    pub content: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StorefrontVisit {
    pub id: String,
    pub storefront_id: String,
    pub visitor_ip: String,
    pub user_agent: Option<String>,
    pub path: String,
    pub visited_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Inventory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Warehouse {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub address: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InventoryByWarehouse {
    pub id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub reorder_point: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTransaction {
    pub id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub transaction_type: String,
    pub quantity: i32,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LowStockAlert {
    pub id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub current_quantity: i32,
    pub reorder_point: i32,
    pub alerted_at: String,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BackInStockRequest {
    pub id: String,
    pub product_id: String,
    pub customer_id: String,
    pub channel_type: Option<String>,
    pub notified_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Engine - Sync Mapping (External ID tracking)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ExternalProductMapping {
    pub id: String,
    pub product_id: String,
    pub platform: String,
    pub connection_id: String,
    pub external_id: String,
    pub external_sku: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ExternalOrderMapping {
    pub id: String,
    pub order_id: String,
    pub platform: String,
    pub connection_id: String,
    pub external_id: String,
    pub external_order_number: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncQueueItem {
    pub id: String,
    pub connection_id: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub direction: String,
    pub payload: Option<String>,
    pub status: String,
    pub priority: i32,
    pub retries: i32,
    pub max_retries: i32,
    pub error: Option<String>,
    pub scheduled_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEvent {
    pub id: String,
    pub platform: String,
    pub event_type: String,
    pub payload: String,
    pub processed: bool,
    pub error: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Product Options/Add-ons (Gap #1)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductOption {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_adjustment: f64,
    pub required: bool,
    pub max_selections: i32,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductOptionValue {
    pub id: String,
    pub option_id: String,
    pub name: String,
    pub price_adjustment: f64,
    pub sku: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Delivery Fee Calculator (Gap #2)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryFeeRule {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub r#type: String,
    pub base_fee: f64,
    pub per_km_fee: f64,
    pub per_gram_fee: f64,
    pub max_distance: Option<f64>,
    pub free_delivery_threshold: Option<f64>,
    pub min_order_value: Option<f64>,
    pub postal_codes: Option<String>,
    pub regions: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Payment Proof Upload (Gap #3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentProof {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub file_url: String,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
    pub status: String,
    pub merchant_notes: Option<String>,
    pub verified_at: Option<String>,
    pub verified_by: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Order Notifications (Gap #4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderNotificationTemplate {
    pub id: String,
    pub brand_id: String,
    pub channel_type: String,
    pub trigger_event: String,
    pub template_name: String,
    pub template_body: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrderNotificationLog {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub trigger_event: String,
    pub message_id: Option<String>,
    pub status: String,
    pub sent_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Local Payment Methods (Gap #6)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentGateway {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub provider: String,
    pub credentials: Option<String>,
    pub currency: String,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Service Types (Gap #7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServiceType {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Delivery Schedules & Time Slots (Gap #8)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeliverySchedule {
    pub id: String,
    pub brand_id: String,
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
    pub cutoff_time: Option<String>,
    pub max_capacity: Option<i32>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryTimeSlot {
    pub id: String,
    pub brand_id: String,
    pub day_of_week: i32,
    pub slot_start: String,
    pub slot_end: String,
    pub label: Option<String>,
    pub max_orders: i32,
    pub is_active: bool,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Invoices (Gap #9)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: String,
    pub order_id: String,
    pub invoice_number: String,
    pub pdf_url: Option<String>,
    pub issued_at: String,
    pub due_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - CSV Import/Export (Gap #10)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportJob {
    pub id: String,
    pub brand_id: String,
    pub entity_type: String,
    pub file_url: String,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub error_count: i32,
    pub errors: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Customer Accounts (Gap #11)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerAccount {
    pub id: String,
    pub customer_id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub is_verified: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Customer Reviews (Gap #12)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerReview {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub product_id: Option<String>,
    pub rating: i32,
    pub comment: Option<String>,
    pub is_public: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Loyalty/Rewards (Gap #13)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LoyaltyProgram {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub points_per_currency: f64,
    pub points_expiry_days: i32,
    pub redemption_rate: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerPoints {
    pub id: String,
    pub customer_id: String,
    pub brand_id: String,
    pub points_balance: i32,
    pub points_earned: i32,
    pub points_redeemed: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PointsTransaction {
    pub id: String,
    pub customer_id: String,
    pub brand_id: String,
    pub points: i32,
    pub transaction_type: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Payment Reminders (Gap #14)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentReminder {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub reminder_type: String,
    pub scheduled_at: Option<String>,
    pub sent_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Delivery Reminders (Gap #15)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryReminder {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub reminder_type: String,
    pub scheduled_at: Option<String>,
    pub sent_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Staff Management (Gap #16)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StaffMember {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub permissions: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - POS Sessions (Gap #17)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PosSession {
    pub id: String,
    pub brand_id: String,
    pub staff_id: Option<String>,
    pub session_token: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Receipts (Gap #18)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub id: String,
    pub order_id: String,
    pub receipt_number: String,
    pub pdf_url: Option<String>,
    pub sent_via: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Commerce Inbox Threads (Gap #19)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CommerceInboxThread {
    pub id: String,
    pub brand_id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub last_message_at: Option<String>,
    pub unread_count: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Wholesale Pricing (Gap #20)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WholesalePricing {
    pub id: String,
    pub product_id: String,
    pub customer_group: String,
    pub price: f64,
    pub min_quantity: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Payment Fees (Gap #21)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentFee {
    pub id: String,
    pub order_id: String,
    pub gateway_id: Option<String>,
    pub fee_amount: f64,
    pub fee_percentage: f64,
    pub passed_to_customer: bool,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Payment Recovery (Gap #22)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRecovery {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub attempt_count: i32,
    pub last_attempt_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Automated Follow-ups (Gap #23)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AutomatedFollowup {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub trigger_type: String,
    pub trigger_days: i32,
    pub channel_type: String,
    pub template_body: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FollowupLog {
    pub id: String,
    pub followup_id: String,
    pub customer_id: String,
    pub channel_type: String,
    pub sent_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Custom Domains (Gap #24)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomDomain {
    pub id: String,
    pub brand_id: String,
    pub domain: String,
    pub storefront_id: Option<String>,
    pub ssl_enabled: bool,
    pub verified: bool,
    pub dns_records: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Storefront Pages (Gap #25)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StorefrontPage {
    pub id: String,
    pub storefront_id: String,
    pub slug: String,
    pub title: String,
    pub content: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Meta Pixels (Gap #27)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MetaPixel {
    pub id: String,
    pub brand_id: String,
    pub pixel_id: String,
    pub access_token: Option<String>,
    pub events: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Checkout Questions (Gap #29)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutQuestion {
    pub id: String,
    pub brand_id: String,
    pub question: String,
    pub question_type: String,
    pub options: Option<String>,
    pub is_required: bool,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutQuestionAnswer {
    pub id: String,
    pub order_id: String,
    pub question_id: String,
    pub answer: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Commerce Gaps - Delivery Proofs (Gap #30)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryProof {
    pub id: String,
    pub shipment_id: String,
    pub order_id: String,
    pub file_url: String,
    pub file_name: Option<String>,
    pub uploaded_by: Option<String>,
    pub uploaded_at: String,
}

// ---------------------------------------------------------------------------
// ERP Catalog Module (Port of catalog.prisma — 790 lines)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErpProductKind {
    Goods,
    Service,
    Digital,
    Stitched,
    Unstitched,
    StitchingService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErpProductStatus {
    Draft,
    Active,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErpOptionDisplay {
    Button,
    Dropdown,
    Swatch,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErpCollectionSortStrategy {
    Manual,
    BestSelling,
    Newest,
    PriceAsc,
    PriceDesc,
    Margin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErpAiSuggestionKind {
    AltText,
    MetaTitle,
    MetaDescription,
    Description,
    CareInstructions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErpAiSuggestionStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCategory {
    pub id: String,
    pub slug: String,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCategoryTranslation {
    pub id: String,
    pub category_id: String,
    pub locale: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpBrand {
    pub id: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpBrandTranslation {
    pub id: String,
    pub brand_id: String,
    pub locale: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProduct {
    pub id: String,
    pub slug: String,
    pub kind: String,
    pub status: String,
    pub category_id: Option<String>,
    pub base_price: i64,
    pub compare_at_price: Option<i64>,
    pub brand_id: Option<String>,
    pub fabric: Option<String>,
    pub season: Option<String>,
    pub tags: String,
    pub code: Option<String>,
    pub noindex: bool,
    pub canonical_override: Option<String>,
    pub piece_count: Option<i32>,
    pub fabric_meters: Option<f64>,
    pub stitching_available: bool,
    pub is_consumable: bool,
    pub garment_type_id: Option<String>,
    pub shipping_class_id: Option<String>,
    pub featured_rank: i32,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductTranslation {
    pub id: String,
    pub product_id: String,
    pub locale: String,
    pub name: String,
    pub description: Option<String>,
    pub care_instructions: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub focus_keyword: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductVariant {
    pub id: String,
    pub product_id: String,
    pub sku: String,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub label: Option<String>,
    pub price_override: Option<i64>,
    pub stock: i32,
    pub low_stock_threshold: i32,
    pub is_active: bool,
    pub cost_price: i64,
    pub reorder_point: Option<i32>,
    pub reorder_quantity: Option<i32>,
    pub preferred_supplier_id: Option<String>,
    pub barcode: Option<String>,
    pub sku_is_manual: bool,
    pub produced_in_house: bool,
    pub lot_tracked: bool,
    pub tracks_serials: bool,
    pub warranty_months: Option<i32>,
    pub tax_class_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductImage {
    pub id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub url: String,
    pub alt: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductImageTranslation {
    pub id: String,
    pub image_id: String,
    pub locale: String,
    pub alt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductOption {
    pub id: String,
    pub product_id: String,
    pub key: String,
    pub display: String,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductOptionTranslation {
    pub id: String,
    pub option_id: String,
    pub locale: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductOptionValue {
    pub id: String,
    pub option_id: String,
    pub key: String,
    pub swatch_hex: Option<String>,
    pub swatch_url: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductOptionValueTranslation {
    pub id: String,
    pub value_id: String,
    pub locale: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantOptionValue {
    pub variant_id: String,
    pub option_id: String,
    pub value_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpOptionTemplate {
    pub id: String,
    pub key: String,
    pub display: String,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpOptionTemplateValue {
    pub id: String,
    pub template_id: String,
    pub key: String,
    pub swatch_hex: Option<String>,
    pub swatch_url: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCollection {
    pub id: String,
    pub slug: String,
    pub rule: Option<String>,
    pub sort_strategy: String,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCollectionTranslation {
    pub id: String,
    pub collection_id: String,
    pub locale: String,
    pub name: String,
    pub description: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCollectionProduct {
    pub collection_id: String,
    pub product_id: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpRedirect {
    pub id: String,
    pub from_path: String,
    pub to_path: String,
    pub status_code: i32,
    pub is_automatic: bool,
    pub hit_count: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockAdjustment {
    pub id: String,
    pub variant_id: String,
    pub delta: i32,
    pub resulting_stock: i32,
    pub reason: String,
    pub note: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductSizeBias {
    pub product_id: String,
    pub too_small_pct: f64,
    pub true_to_size_pct: f64,
    pub too_large_pct: f64,
    pub sample_size: i32,
    pub shift: i32,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductCrossSell {
    pub source_product_id: String,
    pub target_product_id: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductAiSuggestion {
    pub id: String,
    pub product_id: String,
    pub image_id: Option<String>,
    pub kind: String,
    pub locale: String,
    pub suggested_text: String,
    pub status: String,
    pub created_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCategoryPreset {
    pub id: String,
    pub key: String,
    pub label: String,
    pub tree_json: String,
    pub description: Option<String>,
}

// Request/Response DTOs for catalog API

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub slug: String,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryRequest {
    pub slug: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    pub slug: String,
    pub kind: String,
    pub category_id: Option<String>,
    pub base_price: i64,
    pub compare_at_price: Option<i64>,
    pub brand_id: Option<String>,
    pub fabric: Option<String>,
    pub season: Option<String>,
    pub tags: Option<Vec<String>>,
    pub code: Option<String>,
    pub piece_count: Option<i32>,
    pub fabric_meters: Option<f64>,
    pub stitching_available: Option<bool>,
    pub is_consumable: Option<bool>,
    pub featured_rank: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    pub slug: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<String>,
    pub base_price: Option<i64>,
    pub compare_at_price: Option<i64>,
    pub brand_id: Option<String>,
    pub fabric: Option<String>,
    pub season: Option<String>,
    pub tags: Option<Vec<String>>,
    pub code: Option<String>,
    pub piece_count: Option<i32>,
    pub fabric_meters: Option<f64>,
    pub stitching_available: Option<bool>,
    pub is_consumable: Option<bool>,
    pub featured_rank: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantRequest {
    pub product_id: String,
    pub sku: String,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub label: Option<String>,
    pub price_override: Option<i64>,
    pub stock: Option<i32>,
    pub low_stock_threshold: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVariantRequest {
    pub sku: Option<String>,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub label: Option<String>,
    pub price_override: Option<i64>,
    pub stock: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductImageRequest {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub url: String,
    pub alt: Option<String>,
    pub sort_order: Option<i32>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductOptionRequest {
    pub product_id: String,
    pub key: String,
    pub display: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOptionValueRequest {
    pub option_id: String,
    pub key: String,
    pub swatch_hex: Option<String>,
    pub swatch_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionRequest {
    pub slug: String,
    pub rule: Option<String>,
    pub sort_strategy: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAdjustmentRequest {
    pub variant_id: String,
    pub delta: i32,
    pub reason: String,
    pub note: Option<String>,
}

// ============================================================
// ERP Orders Module (orders.prisma — 369 lines)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrder {
    pub id: String,
    pub number: String,
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub status: String,
    pub currency: String,
    pub channel: String,
    pub location_id: Option<String>,
    pub terminal_id: Option<String>,
    pub shift_id: Option<String>,
    pub origin_node_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
    pub source_cart_id: Option<String>,
    pub shipping_address: serde_json::Value,
    pub billing_address: Option<serde_json::Value>,
    pub items_subtotal: i64,
    pub stitching_subtotal: i64,
    pub item_discount_total: i64,
    pub order_discount: i64,
    pub shipping_fee: i64,
    pub cod_fee: i64,
    pub tax_amount: i64,
    pub grand_total: i64,
    pub shipping_zone: Option<String>,
    pub customer_note: Option<String>,
    pub staff_note: Option<String>,
    pub cancel_reason: Option<String>,
    pub placed_at: String,
    pub confirmed_at: Option<String>,
    pub delivered_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub review_request_sent_at: Option<String>,
    pub cod_risk_score: Option<i32>,
    pub cod_risk_action: Option<String>,
    pub cod_risk_computed_at: Option<String>,
    pub cod_risk_reason_en: Option<String>,
    pub source_type: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrderItem {
    pub id: String,
    pub order_id: String,
    pub product_id: Option<String>,
    pub variant_id: Option<String>,
    pub kind: String,
    pub name_en: String,
    pub name_ur: Option<String>,
    pub sku: Option<String>,
    pub image_url: Option<String>,
    pub variant_label: Option<String>,
    pub unit_price: i64,
    pub unit_cost: i64,
    pub quantity: i32,
    pub stitching_fee_unit: i64,
    pub discount_unit: i64,
    pub line_total: i64,
    pub tax_amount: i64,
    pub tax_rate_bps: i64,
    pub stitching_service_id: Option<String>,
    pub stitching_service_name: Option<String>,
    pub garment_type_key: Option<String>,
    pub measurement_snapshot: Option<serde_json::Value>,
    pub options_snapshot: Option<serde_json::Value>,
    pub stitching_notes: Option<String>,
    pub measurement_values: Option<serde_json::Value>,
    pub custom_design_text: Option<serde_json::Value>,
    pub size_token: Option<String>,
    pub replenishment_reminder_sent_at: Option<String>,
    pub fulfillment_status: String,
    pub awaiting_since: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrderStatusEvent {
    pub id: String,
    pub order_id: String,
    pub from_status: Option<String>,
    pub to_status: String,
    pub note: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCart {
    pub id: String,
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCartItem {
    pub id: String,
    pub cart_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: i32,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShipment {
    pub id: String,
    pub order_id: String,
    pub courier: String,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub status: String,
    pub cost: Option<i64>,
    pub dispatched_at: Option<String>,
    pub delivered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub method_id: Option<String>,
    pub weight_grams: Option<i32>,
    pub package_count: i32,
    pub label_url: Option<String>,
    pub manifest_id: Option<String>,
    pub meta: Option<serde_json::Value>,
    pub is_rto: bool,
    pub rto_at: Option<String>,
    pub failed_attempts: i32,
    pub last_polled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShipmentItem {
    pub id: String,
    pub shipment_id: String,
    pub order_item_id: String,
    pub quantity: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShipmentEvent {
    pub id: String,
    pub shipment_id: String,
    pub event_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpCityCodRiskStat {
    pub city: String,
    pub total_shipments: i32,
    pub rto_shipments: i32,
    pub avg_order_value: i64,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrderNumberCounter {
    pub year: i32,
    pub counter: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpBackInStockRequest {
    pub id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub customer_email: String,
    pub status: String,
    pub notified_at: Option<String>,
    pub created_at: String,
}

// ============================================================
// ERP Orders Request/Response DTOs
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
    pub source_cart_id: Option<String>,
    pub shipping_address: serde_json::Value,
    pub billing_address: Option<serde_json::Value>,
    pub currency: Option<String>,
    pub shipping_zone: Option<String>,
    pub customer_note: Option<String>,
    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderItemRequest {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub kind: Option<String>,
    pub name_en: String,
    pub name_ur: Option<String>,
    pub sku: Option<String>,
    pub image_url: Option<String>,
    pub variant_label: Option<String>,
    pub unit_price: i64,
    pub unit_cost: i64,
    pub quantity: i32,
    pub stitching_fee_unit: Option<i64>,
    pub discount_unit: Option<i64>,
    pub tax_rate_bps: Option<i64>,
    pub stitching_service_id: Option<String>,
    pub stitching_service_name: Option<String>,
    pub garment_type_key: Option<String>,
    pub measurement_snapshot: Option<serde_json::Value>,
    pub options_snapshot: Option<serde_json::Value>,
    pub stitching_notes: Option<String>,
    pub measurement_values: Option<serde_json::Value>,
    pub custom_design_text: Option<serde_json::Value>,
    pub size_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderStatusRequest {
    pub status: String,
    pub note: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCartRequest {
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCartItemRequest {
    pub cart_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCartItemRequest {
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutCartRequest {
    pub cart_id: String,
    pub shipping_address: serde_json::Value,
    pub billing_address: Option<serde_json::Value>,
    pub customer_note: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShipmentRequest {
    pub order_id: String,
    pub courier: Option<String>,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub weight_grams: Option<i32>,
    pub package_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShipmentStatusRequest {
    pub status: String,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShipmentEventRequest {
    pub shipment_id: String,
    pub event_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackInStockRequestRequest {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub customer_email: String,
}

// ============================================================
// ERP Payments Module (payments.prisma — 55 lines)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpPayment {
    pub id: String,
    pub order_id: String,
    pub provider: String,
    pub status: String,
    pub amount: i64,
    pub currency: String,
    pub reference: Option<String>,
    pub provider_ref: Option<String>,
    pub meta: Option<serde_json::Value>,
    pub receipt_url: Option<String>,
    pub failure_reason: Option<String>,
    pub approved_by_id: Option<String>,
    pub paid_at: Option<String>,
    pub refunded_at: Option<String>,
    pub refund_amount: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpPaymentMethod {
    pub id: String,
    pub name: String,
    pub code: String,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================
// ERP Payments Request/Response DTOs
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentRequest {
    pub order_id: String,
    pub provider: String,
    pub amount: i64,
    pub currency: Option<String>,
    pub reference: Option<String>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePaymentStatusRequest {
    pub status: String,
    pub provider_ref: Option<String>,
    pub receipt_url: Option<String>,
    pub failure_reason: Option<String>,
    pub approved_by_id: Option<String>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundPaymentRequest {
    pub refund_amount: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadReceiptRequest {
    pub receipt_url: String,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentMethodRequest {
    pub name: String,
    pub code: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePaymentMethodRequest {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

// ============================================================
// ERP Fulfilment Module (fulfilment.prisma — 281 lines)
// Returns/RMA, shipping zones, methods, rates, manifests
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShippingZone {
    pub id: String,
    pub key: String,
    pub sort_order: i32,
    pub cities: String,
    pub is_active: bool,
    pub is_catch_all: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShippingMethod {
    pub id: String,
    pub key: String,
    pub courier_key: Option<String>,
    pub eta_min_days: i32,
    pub eta_max_days: i32,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShippingRate {
    pub id: String,
    pub zone_id: String,
    pub method_id: String,
    pub rate_type: String,
    pub fee: i32,
    pub base_fee: i32,
    pub included_units: i32,
    pub free_above: Option<i32>,
    pub min_value: Option<i32>,
    pub max_value: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpShippingClass {
    pub id: String,
    pub key: String,
    pub handling_fee: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpManifest {
    pub id: String,
    pub number: String,
    pub courier_key: String,
    pub status: String,
    pub dispatched_at: Option<String>,
    pub created_by_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpReturnReasonCode {
    pub id: String,
    pub key: String,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpReturnRequest {
    pub id: String,
    pub number: String,
    pub order_id: String,
    pub user_id: Option<String>,
    pub status: String,
    pub resolution: Option<String>,
    pub customer_note: Option<String>,
    pub staff_note: Option<String>,
    pub photo_urls: String,
    pub requested_at: String,
    pub approved_at: Option<String>,
    pub received_at: Option<String>,
    pub inspected_at: Option<String>,
    pub refunded_at: Option<String>,
    pub refund_amount: Option<i32>,
    pub restocking_fee_minor: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpReturnRequestItem {
    pub id: String,
    pub return_id: String,
    pub order_item_id: String,
    pub reason_id: String,
    pub quantity: i32,
    pub restockable: Option<bool>,
    pub restocked: bool,
}

// ERP Fulfilment Request DTOs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShippingZoneRequest {
    pub key: String,
    pub cities: Option<Vec<String>>,
    pub is_catch_all: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShippingZoneRequest {
    pub cities: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub is_catch_all: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShippingMethodRequest {
    pub key: String,
    pub courier_key: Option<String>,
    pub eta_min_days: Option<i32>,
    pub eta_max_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShippingMethodRequest {
    pub courier_key: Option<String>,
    pub eta_min_days: Option<i32>,
    pub eta_max_days: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShippingRateRequest {
    pub zone_id: String,
    pub method_id: String,
    pub rate_type: Option<String>,
    pub fee: i32,
    pub base_fee: Option<i32>,
    pub included_units: Option<i32>,
    pub free_above: Option<i32>,
    pub min_value: Option<i32>,
    pub max_value: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReturnRequestRequest {
    pub order_id: String,
    pub user_id: Option<String>,
    pub customer_note: Option<String>,
    pub photo_urls: Option<Vec<String>>,
    pub items: Vec<CreateReturnItemRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReturnItemRequest {
    pub order_item_id: String,
    pub reason_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReturnStatusRequest {
    pub status: String,
    pub resolution: Option<String>,
    pub staff_note: Option<String>,
    pub refund_amount: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectReturnItemRequest {
    pub return_item_id: String,
    pub restockable: bool,
    pub restocked: bool,
}

// ============================================================
// ERP Inventory Module (inventory.prisma — 535 lines)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLocation {
    pub id: String,
    pub name: String,
    pub code: String,
    pub r#type: String,
    pub retail_brand_id: Option<String>,
    pub origin_node_id: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub address: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantLocationStock {
    pub variant_id: String,
    pub location_id: String,
    pub quantity: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockMovement {
    pub id: String,
    pub variant_id: String,
    pub delta: i32,
    pub resulting_stock: i32,
    pub reason: String,
    pub location_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub unit_cost: Option<i32>,
    pub note: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockVariance {
    pub id: String,
    pub variant_id: String,
    pub location_id: String,
    pub overshoot_qty: i32,
    pub stock_movement_id: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub status: String,
    pub resolved_by_user_id: Option<String>,
    pub resolved_at: Option<String>,
    pub resolution_note: Option<String>,
    pub pos_shift_id: Option<String>,
    pub serialized_unit_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSpotCheckPolicy {
    pub id: String,
    pub location_id: Option<String>,
    pub min_skus_per_check: i32,
    pub frequency_days: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSpotCheck {
    pub id: String,
    pub policy_id: String,
    pub location_id: String,
    pub status: String,
    pub scheduled_for: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSpotCheckLine {
    pub id: String,
    pub spot_check_id: String,
    pub variant_id: String,
    pub expected: i32,
    pub counted: Option<i32>,
    pub variance: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantSalesStat {
    pub variant_id: String,
    pub units_30: i32,
    pub units_90: i32,
    pub last_sold_at: Option<String>,
    pub days_of_cover: Option<f64>,
    pub age_days: Option<i32>,
    pub is_slow_mover: bool,
    pub is_dead_stock: bool,
    pub last_low_stock_alert_at: Option<String>,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantForecast {
    pub variant_id: String,
    pub forecasted_daily_demand: f64,
    pub seasonal_factor: Option<f64>,
    pub projected_stockout_at: Option<String>,
    pub reason_en: Option<String>,
    pub computed_at: String,
    pub safety_stock_units: Option<i32>,
    pub suggested_reorder_point: Option<i32>,
    pub service_level_percent: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpMarkdownSuggestion {
    pub id: String,
    pub variant_id: String,
    pub discount_id: Option<String>,
    pub suggested_bps: i32,
    pub estimated_clearance_pct: f64,
    pub estimated_clearance_days: i32,
    pub margin_before_paisa: i32,
    pub margin_after_paisa: i32,
    pub status: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLot {
    pub id: String,
    pub product_variant_id: String,
    pub lot_code: String,
    pub expires_on: Option<String>,
    pub received_on: String,
    pub source_receipt_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLotStock {
    pub id: String,
    pub lot_id: String,
    pub location_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLotAllocation {
    pub id: String,
    pub lot_id: String,
    pub stock_movement_id: String,
    pub quantity: i32,
    pub allocated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSerializedUnit {
    pub id: String,
    pub product_variant_id: String,
    pub serial_number: String,
    pub status: String,
    pub lot_id: Option<String>,
    pub location_id: Option<String>,
    pub order_id: Option<String>,
    pub stock_movement_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockCount {
    pub id: String,
    pub count_number: String,
    pub status: String,
    pub note: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub created_by: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockCountLine {
    pub id: String,
    pub stock_count_id: String,
    pub variant_id: String,
    pub expected: Option<i32>,
    pub counted: Option<i32>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================
// ERP Inventory Request/Response DTOs
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
    pub name: String,
    pub code: String,
    pub r#type: Option<String>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub is_active: Option<bool>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockMovementRequest {
    pub variant_id: String,
    pub delta: i32,
    pub reason: String,
    pub location_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub unit_cost: Option<i32>,
    pub note: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockVarianceRequest {
    pub variant_id: String,
    pub location_id: String,
    pub overshoot_qty: i32,
    pub stock_movement_id: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveVarianceRequest {
    pub resolved_by_user_id: String,
    pub resolution_note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpotCheckPolicyRequest {
    pub location_id: Option<String>,
    pub min_skus_per_check: i32,
    pub frequency_days: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpotCheckRequest {
    pub policy_id: String,
    pub location_id: String,
    pub scheduled_for: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpotCheckLineRequest {
    pub spot_check_id: String,
    pub variant_id: String,
    pub expected: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSpotCheckLineRequest {
    pub counted: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLotRequest {
    pub product_variant_id: String,
    pub lot_code: String,
    pub expires_on: Option<String>,
    pub source_receipt_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLotStockRequest {
    pub lot_id: String,
    pub location_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLotAllocationRequest {
    pub lot_id: String,
    pub stock_movement_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSerializedUnitRequest {
    pub product_variant_id: String,
    pub serial_number: String,
    pub lot_id: Option<String>,
    pub location_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSerializedUnitStatusRequest {
    pub status: String,
    pub order_id: Option<String>,
    pub location_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockCountRequest {
    pub count_number: Option<String>,
    pub note: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockCountLineRequest {
    pub stock_count_id: String,
    pub variant_id: String,
    pub expected: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStockCountLineRequest {
    pub counted: i32,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantForecastRequest {
    pub variant_id: String,
    pub forecasted_daily_demand: f64,
    pub seasonal_factor: Option<f64>,
    pub reason_en: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarkdownSuggestionRequest {
    pub variant_id: String,
    pub suggested_bps: i32,
    pub estimated_clearance_pct: f64,
    pub estimated_clearance_days: i32,
    pub margin_before_paisa: i32,
    pub margin_after_paisa: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMarkdownSuggestionStatusRequest {
    pub status: String,
    pub discount_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Phase 35: Embed Widget
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmbedWidgetConfig {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub widget_type: String,
    pub status: String,
    pub settings: serde_json::Value,
    pub styles: serde_json::Value,
    pub trigger_config: serde_json::Value,
    pub domain_whitelist: Option<serde_json::Value>,
    pub install_count: i32,
    pub last_installed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmbedWidgetRequest {
    pub name: String,
    pub widget_type: String,
    pub settings: Option<serde_json::Value>,
    pub styles: Option<serde_json::Value>,
    pub trigger_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWidgetEventRequest {
    pub instance_id: String,
    pub event_type: String,
    pub payload: Option<serde_json::Value>,
}
