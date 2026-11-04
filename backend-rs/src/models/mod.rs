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
    pub created_at: String,
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
