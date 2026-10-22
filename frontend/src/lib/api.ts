const BASE = '/api/v1'

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...init,
    headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) },
  })
  if (!res.ok) {
    const detail = await res.text()
    throw new Error(`${res.status} ${res.statusText}: ${detail}`)
  }
  if (res.status === 204) return undefined as T
  return res.json() as Promise<T>
}

export interface Draft {
  id: string
  conversationId: string
  body: string
  confidence: number
  citations: string[]
  status: 'pending_review' | 'auto_approved' | 'approved' | 'rejected' | 'sent'
  createdAt: string
}

export interface Conversation {
  id: string
  contactId: string
  contactName?: string
  contactPhone?: string
  channel?: string
  state: string
  resolutionType?: string | null
  broadcastId?: string | null
  broadcastName?: string | null
  humanTakeover: boolean
  assignedToId: string | null
  csatScore?: number | null
  csatComment?: string | null
  customAttributes?: Record<string, any>
  updatedAt: string
}

export interface Message {
  id: string
  direction: 'inbound' | 'outbound'
  channel: string
  body: string
  intent: string
  status: string
  sentAt: string
  providerMetadata?: Record<string, any>
  translation?: TranslationResult
}

export interface ConversationDetail {
  id: string
  state: string
  humanTakeover: boolean
  assignedToId: string | null
  contactId?: string
  contactName?: string
  contactPhone?: string
  channel?: string
  language?: string
  customAttributes?: Record<string, any>
  resolutionType?: string | null
  csatScore?: number | null
  csatComment?: string | null
  campaign?: {
    id: string
    name: string
    templateName: string
    sentAt: string
    status: string
  } | null
  messages: Message[]
}

export interface Service {
  id: string
  name: string
  summary: string
  targetInstitutionTypes: string[]
}

export interface Sequence {
  id: string
  name: string
  isActive: boolean
}

export interface VariantStats {
  variants: Array<{
    variant: string
    enrolled: number
    replied: number
    closedWon: number
  }>
}

export interface TeamMember {
  id: string
  name: string
  email: string
  role?: string
  custom_permissions?: string[]
  availability_status?: string
  max_concurrent_conversations?: number
}

export interface RoleItem {
  id: string
  name: string
  display_name: string
  description: string
  is_system_role: boolean
  permissions: string[]
  permissions_count: number
  assigned_members_count: number
  created_at: string
  updated_at: string
}

export interface PermissionCatalogItem {
  key: string
  label: string
  description: string
  category: string
}

export interface KnowledgeBaseEntry {
  id: string
  title: string
  content: string
  topic?: string
  sourceType: string
  source_type?: string
  sourceUrl?: string | null
  source_url?: string | null
  fileName?: string | null
  file_name?: string | null
  targetTouchpoints?: string[]
  target_touchpoints?: string[]
  status?: string
  resolutionRate?: number
  resolution_rate?: number
  usageCount?: number
  usage_count?: number
  helpfulCount?: number
  helpful_count?: number
  unhelpfulCount?: number
  unhelpful_count?: number
  createdAt: string
  created_at?: string
  updatedAt?: string
  updated_at?: string
}

export interface KnowledgeRecommendation {
  id: string
  gapType: 'content_gap' | 'customer_data_gap' | 'action_gap' | string
  gap_type?: string
  topic: string
  title: string
  description: string
  impactOnResolution: number
  impact_on_resolution?: number
  suggestedAction: string
  suggested_action?: string
  suggestedContent: string
  suggested_content?: string
  status: 'open' | 'applied' | 'dismissed'
  createdAt: string
  created_at?: string
  resolvedAt?: string | null
  resolved_at?: string | null
}

export interface KnowledgeHubTopic {
  name: string
  icon: string
  sourcesCount: number
  avgResolutionRate: number
  sources: KnowledgeBaseEntry[]
}

export interface KnowledgeHubOverview {
  totalSources: number
  overallResolutionRate: number
  topics: KnowledgeHubTopic[]
  touchpointBreakdown: {
    agent: {
      title: string
      sourcesDeployed: number
      resolutionRate: number
      deflectionsCount: number
      status: string
    }
    copilot: {
      title: string
      sourcesDeployed: number
      resolutionRate: number
      insertionsCount: number
      status: string
    }
    help_center: {
      title: string
      sourcesDeployed: number
      resolutionRate: number
      satisfactionPct: number
      status: string
    }
  }
}

export interface KnowledgeGap {
  id: string
  question: string
  suggestedAnswer: string
  sourceQuery: string
  frequency: number
  status: 'suggested' | 'approved' | 'dismissed'
  createdAt: string
}

export interface ContactMemory {
  id: string
  contactId: string
  fact: string
  category: string
  confidence: number
  notedAt: string
  createdAt?: string
}

export interface AutomationRule {
  id: string
  name: string
  description?: string
  triggerKeyword: string
  matchType: string
  actionType: string
  actionValue: string
  priority: number
  isActive: boolean
  executionCount?: number
  lastTriggeredAt?: string | null
  createdAt?: string
}

export interface AutomationStats {
  totalRules: number
  activeRules: number
  totalExecutions: number
  csatAverage: number | null
  csatResponsesCount: number
}

export interface RuleTestResult {
  matched: boolean
  message?: string
  rule?: AutomationRule
  actionType?: string
  actionValue?: string
  preview?: Record<string, unknown>
}

export interface ContactRecord {
  id: string
  value: string
  personName: string
  channel: string
  language: string
  customFields: Record<string, unknown>
  tags: string[]
  createdAt?: string
}

export interface ContactSegment {
  id: string
  name: string
  description: string
  filters: {
    channels?: string[]
    tags?: string[]
    tag_match_mode?: 'any' | 'all'
    search?: string
    created_after?: string
    created_before?: string
  }
  memberCount: number
  createdAt: string
  updatedAt: string
}

export interface ImportPreviewResult {
  totalRows: number
  detectedColumns: string[]
  detectedChannels: string[]
  sampleRows: Array<{
    name: string
    channel: string
    value: string
    tags: string[]
    valid: boolean
  }>
}

export interface MultiChannelImportResult {
  status: string
  created: number
  updated: number
  skipped: number
  total: number
  channels: Record<string, number>
}

export interface Broadcast {
  id: string
  name: string
  templateName: string
  templateLanguage: string
  bodyPreview?: string
  templateParams?: Record<string, string>
  tagName: string
  status: 'draft' | 'scheduled' | 'sending' | 'completed' | 'failed'
  scheduledAt: string | null
  audienceCount?: number
  sentCount: number
  deliveredCount: number
  readCount: number
  failedCount: number
  skippedCount?: number
  createdAt: string
}

export interface BroadcastRecipient {
  id: string
  contactId: string
  contactName?: string
  contactPhone?: string
  status: string
  error: string
  sentAt?: string | null
  deliveredAt?: string | null
  readAt?: string | null
}

export interface CampaignAnalytics {
  campaign: Broadcast
  metrics: {
    audience: number
    submitted: number
    submittedPct: number
    delivered: number
    deliveredPct: number
    read: number
    readPct: number
    failed: number
    failedPct: number
    skipped: number
    skippedPct: number
  }
  deliveryBreakdown: {
    deliveredRate: number
    readRate: number
    failureRate: number
  }
  recipients: BroadcastRecipient[]
}

export interface AudiencePreview {
  tags: string[]
  totalSelected: number
  dailyLimit: number
  sampleContacts: Array<{
    id: string
    name: string
    phone: string
    tags: string[]
  }>
}

export interface Inbox {
  id: string
  name: string
  channelType: string
  phoneNumber: string
  avatarUrl: string
  isActive: boolean
  greetingEnabled: boolean
  greetingMessage: string
  outOfOfficeEnabled: boolean
  outOfOfficeMessage: string
  csatEnabled: boolean
  autoReadReceipts: boolean
  workingHoursEnabled: boolean
  workingHoursStart: number
  workingHoursEnd: number
  timezone: string
  credentials: Record<string, string>
  createdAt: string
  updatedAt: string
}

export interface WhatsAppChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    verifyToken: string
    subscribedFields: string[]
  }
  health: {
    isConnected: boolean
    verifiedAt: string
    displayName: string
    displayPhoneNumber: string
    qualityRating: string
    messagingLimitTier: string
    codeVerificationStatus: string
  }
}

export interface WhatsAppBusinessProfile {
  about: string
  address: string
  description: string
  email: string
  vertical: string
  websites: string[]
  profilePictureUrl?: string
}

export interface WhatsAppTemplate {
  id: string
  name: string
  category: string
  language: string
  status: string
  bodyText: string
  components: any[]
  lastUpdated: string
}

export interface LiveChatChannelData {
  inbox: Inbox
  websiteToken: string
  websiteDomain: string
  embedScript: string
  branding: {
    widgetColor: string
    greetingTitle: string
    greetingTagline: string
    widgetPosition: 'right' | 'left'
    avatarUrl: string
  }
  preChatForm: {
    enabled: boolean
    fields: Array<{ name: string; label: string; required: boolean }>
  }
  security: {
    hmacSecret: string
    identityValidation: boolean
  }
  health: {
    isOnline: boolean
    activeVisitors: number
    averageResponseTime: string
    resolutionRate: string
  }
}

export interface LiveChatUpdatePayload {
  name?: string
  website_domain?: string
  widget_color?: string
  greeting_title?: string
  greeting_tagline?: string
  widget_position?: 'right' | 'left'
  avatar_url?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  pre_chat_form_enabled?: boolean
  pre_chat_fields?: Array<{ name: string; label: string; required: boolean }>
  identity_validation?: boolean
  hmac_secret?: string
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface WidgetStyling {
  widget_color: string
  theme_mode: 'light' | 'dark' | 'system'
  launcher_style: 'round_bubble' | 'pill_with_text' | 'minimal_tab'
  launcher_icon: 'chat_bubble' | 'sparkle_ai' | 'headset_support' | 'envelope'
  launcher_text: string
  widget_position: 'right' | 'left'
  greeting_title: string
  greeting_tagline: string
  avatar_url: string
  border_radius: 'rounded_lg' | 'rounded_2xl' | 'rounded_full'
  play_sound: boolean
  show_branding: boolean
  allowed_domains: string
  pre_chat_form_enabled: boolean
  identity_validation: boolean
  hmac_secret: string
}

export interface WidgetEmbedCodes {
  html_script: string
  react_nextjs: string
  google_tag_manager: string
  raw_init: string
}

export interface WidgetCustomizerConfig {
  inbox_name: string
  website_domain: string
  website_token: string
  styling: WidgetStyling
  embed_codes: WidgetEmbedCodes
  preview_url: string
}

export interface UpdateWidgetCustomizerPayload {
  inbox_name?: string
  website_domain?: string
  widget_color?: string
  theme_mode?: string
  launcher_style?: string
  launcher_icon?: string
  launcher_text?: string
  widget_position?: string
  greeting_title?: string
  greeting_tagline?: string
  avatar_url?: string
  border_radius?: string
  play_sound?: boolean
  show_branding?: boolean
  allowed_domains?: string
  pre_chat_form_enabled?: boolean
  identity_validation?: boolean
  hmac_secret?: string
}

export interface AuditLogItem {
  id: string
  created_at: string
  action: string
  actor: string
  actor_email?: string
  actor_name?: string
  actor_role?: string
  entity_type: string
  entity_id: string
  severity: 'info' | 'notice' | 'warning' | 'critical'
  compliance_category: string
  description: string
  ip_address: string
  user_agent?: string
  details: Record<string, any>
}

export interface AuditLogsResponse {
  items: AuditLogItem[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export interface AuditStatistics {
  total_events: number
  sample_analyzed: number
  severity_distribution: Record<string, number>
  category_distribution: Record<string, number>
  top_actors: Array<{ actor: string; count: number }>
  top_actions: Array<{ action: string; count: number }>
  soc2_compliance_status: string
  last_audit_event_at: string | null
}

export interface PreChatField {
  id: string
  name: string
  label: string
  type: 'text' | 'email' | 'phone' | 'select' | 'textarea' | 'number' | 'checkbox'
  placeholder?: string
  required: boolean
  options?: string[]
  order: number
}

export interface PreChatRoutingRule {
  id: string
  field: string
  operator: 'equals' | 'contains' | 'not_equals'
  value: string
  assign_priority: 'low' | 'medium' | 'high' | 'urgent'
  assign_tag: string
  team_target?: string
}

export interface PreChatFormConfig {
  website_token: string
  website_domain: string
  enabled: boolean
  title: string
  description: string
  submit_button_text: string
  require_email: boolean
  fields: PreChatField[]
  routing_rules: PreChatRoutingRule[]
}

export interface PreChatTestIntakeResult {
  valid: boolean
  errors: string[]
  routing_result: {
    priority: string
    tags: string[]
    team_target?: string
    applied_rules: string[]
  }
  evaluated_answers: Record<string, any>
}

export interface LiveChatTestVisitorPayload {
  name: string
  email: string
  phone?: string
  company?: string
  initial_message: string
}

export interface LiveChatEvent {
  id: string
  conversationId: string
  direction: 'inbound' | 'outbound'
  status: string
  body: string
  provider: string
  providerMessageId?: string
  senderName: string
  senderType: string
  sentAt: string
}

export interface SMSChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    httpMethod: string
    signatureValidation: boolean
    fallbackUrl: string
  }
  compliance10dlc: {
    status: string
    campaignId: string
    brandName: string
    brandTier: string
    useCase: string
    optOutKeywords: string[]
    optInKeywords: string[]
    optOutMessage: string
    optInMessage: string
    carrierFiltering: string
  }
  health: {
    isConnected: boolean
    phoneConfigured: boolean
    displayPhoneNumber: string
    capabilities: string[]
    dailyVolumeQuota: string
    deliverabilityRate: string
  }
}

export interface SMSUpdatePayload {
  name?: string
  account_sid?: string
  auth_token?: string
  phone_number?: string
  messaging_service_sid?: string
  a2p_10dlc_status?: string
  brand_name?: string
  brand_tier?: string
  opt_out_message?: string
  opt_in_message?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface TicketActivity {
  id: string
  author_id?: string | null
  author_name: string
  activity_type: string
  message: string
  metadata?: Record<string, any>
  created_at: string
}

export interface ChildTicketSummary {
  id: string
  ticket_number: string
  title: string
  status: string
  priority: string
  department: string
  contact_id?: string | null
  contact_name?: string | null
  conversation_id?: string | null
  created_at: string
}

export interface Ticket {
  id: string
  ticket_number: string
  title: string
  description: string
  ticket_type: 'customer_issue' | 'back_office' | 'tracker'
  status: 'open' | 'in_progress' | 'waiting_on_back_office' | 'waiting_on_customer' | 'resolved' | 'closed'
  priority: 'urgent' | 'high' | 'medium' | 'low'
  department: 'support' | 'finance' | 'logistics' | 'engineering' | 'warehouse' | 'legal'
  conversation_id?: string | null
  contact_id?: string | null
  contact_name?: string | null
  contact_identifier?: string | null
  tracker_id?: string | null
  tracker_number?: string | null
  tracker_title?: string | null
  assigned_to_id?: string | null
  assigned_to_name?: string | null
  resolution_summary?: string | null
  customer_broadcast_sent?: boolean
  resolved_at?: string | null
  created_at: string
  updated_at: string
  child_count?: number
  resolved_child_count?: number
  activities?: TicketActivity[]
  child_tickets?: ChildTicketSummary[]
}

export interface TicketStats {
  total_tickets: number
  open_tickets: number
  customer_issues: number
  back_office_tickets: number
  tracker_tickets: number
  resolved_tickets: number
  department_breakdown: Record<string, number>
  impacted_customers_count: number
}

export interface AgentPolicy {
  id: string
  name: string
  category: string
  description: string
  version: string
  status: string
  process_steps: Array<{ step: number; title: string; instruction: string }>
  rules: Record<string, any>
  guardrails: string[]
  created_at: string
  updated_at: string
}

export interface SimulationScenario {
  id: string
  name: string
  description: string
  category: string
  policy_id?: string | null
  policy_name?: string | null
  customer_persona: {
    name: string
    sentiment: string
    tier: string
    patience?: string
  }
  expected_outcome: string
  customer_messages: string[]
  created_at: string
  latest_run?: {
    id: string
    verdict: string
    overall_score: number
    policy_adherence_score: number
    created_at: string
  } | null
}

export interface SimulationRun {
  id: string
  scenario_id: string
  status: string
  verdict: 'passed' | 'failed' | 'escalated' | 'warning'
  policy_adherence_score: number
  empathy_score: number
  accuracy_score: number
  overall_score: number
  turn_count: number
  transcript: Array<{
    turn: number
    role: 'customer' | 'agent'
    speaker: string
    content: string
    sentiment?: string
    agent_thoughts?: string[]
    policy_check?: string
  }>
  evaluation_summary: string
  recommendations: string[]
  created_at: string
}

export interface SimulationStats {
  total_policies: number
  total_scenarios: number
  total_simulation_runs: number
  pass_rate_percentage: number
  average_overall_score: number
  agent_name: string
}

export interface SandboxChatResult {
  reply: string
  policy_matched: string
  matched_rules: string[]
  guardrails_applied: string[]
  escalated_for_approval: boolean
  timestamp: string
}

export interface AgentInsight {
  id: string
  title: string
  category: 'human_exemplar' | 'trend' | 'policy_refinement' | 'content_gap' | 'team_coaching'
  description: string
  human_rep_id?: string | null
  human_rep_name?: string | null
  exemplar_content?: string | null
  impact_score: number
  estimated_resolution_lift: number
  suggested_action: string
  action_payload: Record<string, any>
  status: 'open' | 'applied' | 'dismissed'
  applied_at?: string | null
  created_at: string
}

export interface AgentRetrainingEvent {
  id: string
  title: string
  trigger_type: string
  version_from: string
  version_to: string
  changes_applied: Array<Record<string, any>>
  status: string
  baseline_resolution_rate: number
  post_training_resolution_rate: number
  resolution_lift: number
  created_at: string
}

export interface SelfImprovingStats {
  total_insights: number
  open_suggestions: number
  applied_retrainings: number
  cumulative_resolution_lift: number
  current_agent_version: string
  human_exemplars_learned: number
  team_coaching_active: number
  agent_name: string
}

export interface AIConnection {
  id: string
  name: string
  provider: 'anthropic' | 'openai' | 'google' | 'deepseek' | 'groq' | 'ollama' | 'custom'
  model_id: string
  api_key_masked: string
  has_api_key: boolean
  base_url?: string | null
  temperature: number
  max_tokens: number
  is_active: boolean
  assigned_jobs: string[]
  test_status: 'connected' | 'failed' | 'untested'
  latency_ms?: number | null
  last_tested_at?: string | null
  created_at: string
}

export interface AICustomPolicy {
  id: string
  job_type: string
  tone: string
  guidelines: string
  rules: Array<{ id: string; rule: string; enabled: boolean; category?: string }>
  guardrails: string[]
  auto_approval_mode: string
  min_csat_learning_threshold: number
  max_autonomous_refund: number
  updated_at: string
}

export type JobModelMapping = Record<
  string,
  {
    connection_id?: string | null
    model_name: string
    provider: string
    model_id: string
  }
>

export interface ChannelGateway {
  id: string
  gateway_type: 'sms' | 'email' | 'voice_sip'
  provider: string
  name: string
  is_default: boolean
  is_active: boolean
  credentials: Record<string, string>
  config: Record<string, any>
  test_status: 'connected' | 'error' | 'untested'
  latency_ms: number
  last_tested_at: string | null
  created_at: string
  updated_at: string
}

export interface SupportedProviderItem {
  provider: string
  name: string
  fields: string[]
}

export interface SupportedProvidersCatalog {
  sms: SupportedProviderItem[]
  email: SupportedProviderItem[]
  voice_sip: SupportedProviderItem[]
}

export interface WebhookEndpoint {
  id: string
  name: string
  url: string
  secret_masked: string
  has_secret: boolean
  events: string[]
  headers: Record<string, string>
  is_active: boolean
  retry_count: number
  timeout_seconds: number
  total_delivered: number
  total_failed: number
  last_status_code: number | null
  last_delivered_at: string | null
  created_at: string
  updated_at: string
}

export interface WebhookDeliveryLog {
  id: string
  endpoint_id: string
  event: string
  payload: Record<string, any>
  status_code: number
  response_headers: Record<string, string>
  response_body: string
  duration_ms: number
  attempt: number
  success: boolean
  error_message: string | null
  created_at: string
}

export interface WebhookEventCatalogItem {
  event: string
  description: string
  category: string
}

export interface StripeSubscription {
  id: string
  status: 'active' | 'past_due' | 'trialing' | 'canceled' | 'unpaid'
  plan_name: string
  amount_cents: number
  amount_formatted: string
  interval: string
  current_period_end: string
  cancel_at_period_end: boolean
  created_at?: string
}

export interface StripePaymentMethod {
  brand: string
  last4: string
  exp_month: number
  exp_year: number
}

export interface StripeBillingProfile {
  contact_id?: string | null
  customer_id: string
  name: string
  email: string
  currency: string
  delinquent: boolean
  has_active_subscription: boolean
  subscription: StripeSubscription
  mrr: number
  mrr_formatted: string
  ltv: number
  ltv_formatted: string
  payment_method: StripePaymentMethod
  billing_portal_enabled: boolean
  test_mode: boolean
}

export interface StripeInvoice {
  id: string
  number: string
  amount_paid_cents: number
  amount_paid_formatted: string
  amount_due_cents: number
  currency: string
  status: 'paid' | 'open' | 'void' | 'uncollectible'
  created_at: string
  period_start: string
  period_end: string
  hosted_invoice_url: string
  invoice_pdf_url: string
  refundable_amount_cents: number
  refunded: boolean
  charge_id?: string
}

export interface StripeBillingLog {
  id: string
  contact_id?: string | null
  customer_identifier: string
  action_type: string
  amount_cents: number
  amount_formatted: string
  currency: string
  status: string
  stripe_id: string
  actor_email: string
  reason: string
  details: Record<string, any>
  created_at: string
}

export interface StripeConfig {
  id: string
  account_name: string
  api_key_masked: string
  publishable_key: string
  webhook_secret_masked: string
  has_api_key: boolean
  has_webhook_secret: boolean
  is_enabled: boolean
  test_mode: boolean
  default_currency: string
  last_tested_at?: string | null
  last_test_status: string
  created_at?: string
  updated_at?: string
}

export interface IssueTrackerConfig {
  id: string
  provider: 'linear' | 'github' | 'jira' | 'slack'
  account_name: string
  is_enabled: boolean
  test_mode: boolean
  credentials: Record<string, any>
  config: Record<string, any>
  last_tested_at?: string | null
  last_test_status: string
  created_at?: string
  updated_at?: string
}

export interface LinkedIssue {
  id: string
  conversation_id?: string | null
  ticket_id?: string | null
  provider: 'linear' | 'github' | 'jira'
  external_issue_id: string
  issue_key: string
  issue_url: string
  title: string
  status: string
  priority: string
  assigned_to: string
  created_by_email: string
  sync_status: string
  metadata_json: Record<string, any>
  created_at: string
  updated_at?: string
}

export interface SlackSyncStatus {
  id?: string
  conversation_id: string
  slack_channel_id?: string
  slack_channel_name?: string
  slack_thread_ts?: string
  thread_url?: string
  is_synced: boolean
  last_synced_at?: string
}

export interface SMSTestMessagePayload {
  recipient_phone: string
  message: string
}

export interface SMSEvent {
  id: string
  conversationId: string
  direction: 'inbound' | 'outbound'
  status: string
  body: string
  provider: string
  providerMessageId?: string
  senderName: string
  senderType: string
  sentAt: string
}

export interface FacebookChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    verifyToken: string
    signatureAlgorithm: string
    fields: string[]
  }
  pageProfile: {
    pageId: string
    pageName: string
    category: string
    verificationTier: string
  }
  icebreakers: Array<{ question: string; payload: string }>
  health: {
    isConnected: boolean
    pageConnected: boolean
    messagingWindow: string
    graphApiVersion: string
    messageLatency: string
  }
}

export interface FacebookUpdatePayload {
  name?: string
  page_id?: string
  page_name?: string
  page_access_token?: string
  app_secret?: string
  webhook_verify_token?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  icebreakers?: Array<{ question: string; payload: string }>
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface FacebookTestMessagePayload {
  recipient_psid: string
  recipient_name?: string
  message: string
}

export interface InstagramChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    verifyToken: string
    signatureAlgorithm: string
    fields: string[]
  }
  accountProfile: {
    igUsername: string
    igId: string
    connectedFacebookPage: string
    accountType: string
  }
  storyMentions: {
    enabled: boolean
    replyMessage: string
  }
  icebreakers: Array<{ question: string; payload: string }>
  health: {
    isConnected: boolean
    accountConnected: boolean
    storyMentionReplyActive: boolean
    graphApiVersion: string
    directMessagesQuota: string
  }
}

export interface InstagramUpdatePayload {
  name?: string
  ig_username?: string
  ig_id?: string
  connected_page_id?: string
  access_token?: string
  app_secret?: string
  webhook_verify_token?: string
  story_mentions_enabled?: boolean
  story_reply_message?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  icebreakers?: Array<{ question: string; payload: string }>
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface InstagramTestMessagePayload {
  recipient_igsid: string
  recipient_username?: string
  message: string
}

export interface SocialEvent {
  id: string
  conversationId: string
  direction: 'inbound' | 'outbound'
  status: string
  body: string
  provider: string
  providerMessageId?: string
  senderName: string
  senderType: string
  sentAt: string
}

export interface TelegramChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    secretToken: string
    security: string
  }
  botProfile: {
    botUsername: string
    botName: string
    canJoinGroups: boolean
    canReadGroupMessages: boolean
  }
  commands: Array<{ command: string; description: string }>
  health: {
    isConnected: boolean
    botConfigured: boolean
    telegramApi: string
    status: string
  }
}

export interface TelegramUpdatePayload {
  name?: string
  bot_token?: string
  bot_username?: string
  secret_token?: string
  commands?: Array<{ command: string; description: string }>
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface TelegramTestMessagePayload {
  recipient_chat_id: string
  recipient_name?: string
  message: string
}

export interface LineChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    signatureValidation: boolean
    signatureHeader: string
  }
  accountProfile: {
    channelId: string
    lineAccountId: string
    accountType: string
  }
  quickReplies: Array<{ label: string; text: string }>
  health: {
    isConnected: boolean
    accountConnected: boolean
    lineApi: string
    status: string
  }
}

export interface LineUpdatePayload {
  name?: string
  channel_id?: string
  channel_secret?: string
  channel_access_token?: string
  line_account_id?: string
  quick_replies?: Array<{ label: string; text: string }>
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface LineTestMessagePayload {
  recipient_user_id: string
  recipient_name?: string
  message: string
}

export interface TikTokChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    verifyToken: string
    signatureHeader: string
    supportedEvents: string[]
  }
  businessProfile: {
    appId: string
    businessAccountId: string
    accountHandle: string
    accountType: string
  }
  commentAutoReply: {
    enabled: boolean
    template: string
  }
  health: {
    isConnected: boolean
    businessAccountConnected: boolean
    tiktokApi: string
    status: string
  }
}

export interface TikTokUpdatePayload {
  name?: string
  app_id?: string
  app_secret?: string
  access_token?: string
  business_account_id?: string
  webhook_verify_token?: string
  comment_auto_reply_enabled?: boolean
  comment_auto_reply_text?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface TikTokTestMessagePayload {
  recipient_open_id: string
  recipient_name?: string
  message: string
}

export interface TikTokTestCommentPayload {
  comment_id: string
  video_id: string
  recipient_open_id: string
  recipient_name?: string
  reply_text: string
}


export interface VoiceChannelData {
  inbox: Inbox
  sipConfig: {
    sipDomain: string
    twimlAppSid: string
    callerId: string
    fallbackNumber: string
    status: string
  }
  webrtcConfig: {
    enabled: boolean
    clientType: string
    codec: string
    iceServers: string[]
  }
  ivrConfig: {
    enabled: boolean
    prompt: string
    menu: Array<{ digit: string; label: string; action: string; target: string }>
  }
  recordingConfig: {
    recordingEnabled: boolean
    recordingChannels: string
    transcriptionEnabled: boolean
    transcriptionProvider: string
  }
  health: {
    status: string
    activeCalls: number
    concurrentChannelLimit: number
    lastPingAt: string
  }
}

export interface VoiceUpdatePayload {
  name?: string
  account_sid?: string
  auth_token?: string
  voice_number?: string
  caller_id?: string
  twiml_app_sid?: string
  sip_domain?: string
  recording_enabled?: boolean
  recording_channels?: string
  transcription_enabled?: boolean
  ivr_enabled?: boolean
  ivr_prompt?: string
  ivr_menu?: Array<{ digit: string; label: string; action: string; target: string }>
  webrtc_enabled?: boolean
  fallback_number?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface VoiceTestCallPayload {
  recipient_phone: string
  agent_name?: string
  message?: string
}

export interface VoiceCallLogItem {
  id: string
  conversationId: string
  callSid: string
  direction: 'inbound' | 'outbound'
  from: string
  to: string
  status: string
  duration: number
  recordingUrl?: string
  recordingDuration?: number
  transcriptionText?: string
  summary: string
  sentAt: string
}

export interface VoiceWebRtcTokenResponse {
  identity: string
  token: string
  sipDomain: string
  twimlAppSid: string
  callerId: string
  expiresIn: number
}

export interface EmailChannelData {
  inbox: Inbox
  smtpConfig: {
    host: string
    port: number
    username: string
    encryption: string
    fromAddress: string
    fromName: string
    replyTo: string
    status: string
  }
  imapConfig: {
    host: string
    port: number
    username: string
    ssl: boolean
    folder: string
    pollingIntervalMinutes: number
    status: string
  }
  webhookConfig: {
    inboundUrl: string
    supportedProviders: string[]
  }
  templateConfig: {
    signature: string
    footer: string
    csatLink: boolean
  }
  health: {
    status: string
    spfConfigured: boolean
    dkimConfigured: boolean
    dmarcConfigured: boolean
    lastSyncAt: string
  }
}

export interface EmailUpdatePayload {
  name?: string
  email_from_address?: string
  email_from_name?: string
  email_reply_to?: string
  smtp_host?: string
  smtp_port?: number
  smtp_username?: string
  smtp_password?: string
  smtp_encryption?: string
  imap_host?: string
  imap_port?: number
  imap_username?: string
  imap_password?: string
  imap_ssl?: boolean
  imap_folder?: string
  email_signature?: string
  email_footer?: string
  greeting_enabled?: boolean
  greeting_message?: string
  out_of_office_enabled?: boolean
  out_of_office_message?: string
  csat_enabled?: boolean
  working_hours_enabled?: boolean
  working_hours_start?: number
  working_hours_end?: number
  timezone?: string
}

export interface EmailTestMessagePayload {
  recipient_email: string
  recipient_name?: string
  subject?: string
  message: string
}

export interface EmailSyncImapResponse {
  status: string
  syncedAt: string
  mailbox: string
  messagesFetched: number
  statusMessage: string
}

export interface Team {
  id: string
  name: string
  description: string
  autoAssign: boolean
  membersCount: number
  members: Array<{ id: string; name: string; email: string }>
  createdAt: string
}

export interface CannedResponse {
  id: string
  shortCode: string
  short_code?: string
  content: string
  category: string
  createdAt: string
  created_at?: string
}

export interface ContactNote {
  id: string
  contactId: string
  authorName: string
  body: string
  createdAt: string
}

export interface CopilotApprovalRequest {
  id: string
  conversationId: string
  conversation_id?: string
  contactId?: string | null
  contact_id?: string | null
  requestType: string
  request_type?: string
  status: 'pending' | 'approved' | 'rejected' | 'amended'
  title: string
  proposedReply: string
  proposed_reply?: string
  amendedReply?: string | null
  amended_reply?: string | null
  actionData: Record<string, any>
  action_data?: Record<string, any>
  internalNote: string
  internal_note?: string
  questionToTeammate?: string | null
  question_to_teammate?: string | null
  teammateAnswer?: string | null
  teammate_answer?: string | null
  assignedToId?: string | null
  reviewerName?: string | null
  reviewer_name?: string | null
  reviewerFeedback?: string | null
  reviewer_feedback?: string | null
  dispatchedMessageId?: string | null
  createdAt: string
  created_at?: string
  updatedAt: string
  updated_at?: string
  resolvedAt?: string | null
  resolved_at?: string | null
}

export interface PricingEstimate {
  region: string
  regionName: string
  category: string
  volume: number
  ratePerConversation: number
  estimatedCostUsd: number
  estimatedRoiUsd: number
  freeServiceTierIncluded: string
  supportedRegions: Array<{ code: string; name: string }>
}

export interface Template {
  id?: string
  name: string
  status: string
  category: string
  language: string
  components: Array<{
    type: string
    text?: string
    format?: string
  }>
}

export interface IntegrationSettings {
  whatsapp: {
    provider: string
    phoneNumberId: string
    wabaId: string
    accessTokenMasked: string | null
    appSecretMasked: string | null
    webhookVerifyTokenMasked: string | null
    webhookCallbackPath: string
    verifiedAt: string | null
    verifiedName: string
    displayNumber: string
  }
  business: {
    name: string
    description: string
    agentGuidelines: string
    autoReplyEnabled: boolean
    businessHoursEnabled: boolean
    businessHoursStart: number
    businessHoursEnd: number
    businessTimezone: string
    awayMessage: string
  }
}

export interface WhatsAppSettingsInput {
  provider?: string
  access_token?: string
  phone_number_id?: string
  waba_id?: string
  app_secret?: string
  webhook_verify_token?: string
}

export interface BusinessSettingsInput {
  name?: string
  description?: string
  agent_guidelines?: string
  auto_reply_enabled?: boolean
  business_hours_enabled?: boolean
  business_hours_start?: number
  business_hours_end?: number
  business_timezone?: string
  away_message?: string
}

export interface VerifyResult {
  valid: boolean
  displayPhoneNumber?: string
  displayNumber?: string
  verifiedName?: string
  qualityRating?: string
  error?: string
}

// Reporting & Analytics Suite Types
export interface LiveReportData {
  status: string
  liveMetrics: {
    openConversations: number
    unassignedConversations: number
    pendingReplies: number
    resolvedToday: number
    avgWaitTimeSeconds: number
    avgWaitTimeFormatted: string
    lastRefreshedAt: string
  }
  channelBreakdown: Record<string, number>
  agentsOnline: Array<{
    id: string
    name: string
    email: string
    status: 'online' | 'busy' | 'offline'
    activeChats: number
    maxCapacity: number
  }>
}

export interface ConversationReportData {
  status: string
  period: string
  summary: {
    totalConversations: number
    totalMessages: number
    inboundMessages: number
    outboundMessages: number
    resolutionRatePercent: number
    avgFirstResponseMinutes: number
    avgResolutionHours: number
  }
  timeSeries: Array<{
    date: string
    conversations: number
    inboundMessages: number
    outboundMessages: number
    resolutions: number
  }>
  channelDistribution: Record<string, number>
}

export interface AgentReportItem {
  id: string
  name: string
  email: string
  avatar: string
  status: string
  conversationsAssigned: number
  conversationsResolved: number
  resolutionRate: string
  avgFirstResponseMinutes: number
  avgResolutionHours: number
  csatRating: number
  csatResponsesCount: number
}

export interface TeamReportItem {
  id: string
  name: string
  description: string
  conversationsCount: number
  unassignedCount: number
  resolutionRate: string
  avgResponseMinutes: string
  slaAdherencePercent: string
}

export interface CsatFeedbackItem {
  conversationId: string
  contactName: string
  channel: string
  score: number
  comment: string
  assignedAgent: string
  date: string
}

export interface CsatReportData {
  status: string
  period: string
  metrics: {
    averageScore: number
    satisfactionPercentage: number
    totalResponses: number
    distribution: Record<string, number>
  }
  recentFeedback: CsatFeedbackItem[]
}

// Captain AI Copilot, Triage & Monitors Types
export interface CaptainSuggestedReply {
  conversationId: string
  suggestedReply: string
  confidence: number
  groundedSources: string[]
  suggestedActions: string[]
}

export interface CaptainSummary {
  conversationId: string
  summary: string
  bulletPoints: string[]
  sentiment: 'positive' | 'neutral' | 'negative' | 'urgent'
  keyTopics: string[]
  actionRequired: boolean
}

export interface CaptainRewriteResult {
  original: string
  rewritten: string
  tone: string
}

export interface CaptainTriageResult {
  conversationId: string
  intent: string
  priority: 'urgent' | 'high' | 'medium' | 'low'
  suggestedTeam: string
  suggestedTeamId?: string | null
  suggestedTags: string[]
  applied: boolean
  reasoning: string
}

export interface CaptainMonitorItem {
  id: string
  name: string
  topic: string
  keywords: string[]
  threshold: number
  currentHitsToday: number
  status: 'healthy' | 'triggered' | 'warning'
  severity: string
  insight: string
}

export interface EmergingThemeItem {
  theme: string
  volume: number
  growth: string
  sentiment: string
  sampleQuote: string
  recommendedAction: string
}

export interface CaptainMonitorsData {
  status: string
  activeMonitors: CaptainMonitorItem[]
  emergingThemes: EmergingThemeItem[]
  sentimentTelemetry: Record<string, number>
  totalMonitoredConversations: number
  lastAnalyzedAt: string
}

// Real-Time Collision Detection & Agent Presence Types
export interface AgentPresenceItem {
  agentId: string
  agentName: string
  agentAvatar: string
  action: 'viewing' | 'typing' | 'left'
  lastSeenAt: number
  conversationId: string
}

export interface CollisionCheckResult {
  conversationId: string
  hasCollision: boolean
  severity: 'none' | 'warning' | 'danger'
  warningMessage: string | null
  otherAgentsCount: number
  typers: AgentPresenceItem[]
  viewers: AgentPresenceItem[]
  allPresences: AgentPresenceItem[]
}

export interface MacroAction {
  type: string
  [key: string]: any
}

export interface MacroItem {
  id: string
  name: string
  description: string
  actions: MacroAction[]
  createdAt: string
  updatedAt: string
}

export interface MacroExecutionResult {
  status: string
  macroId: string
  macroName: string
  conversationId: string
  executedActionsCount: number
  executedActions: Array<{
    action: string
    [key: string]: any
  }>
  conversationState: string
  conversationPriority: string
  conversationTags: string[]
  executedAt: string
}

export interface SLAPolicy {
  id: string
  name: string
  description: string
  firstResponseTimeThreshold: number
  resolutionTimeThreshold: number
  onlyDuringBusinessHours: boolean
  priorities: string[]
  inboxIds: string[]
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface SLAData {
  policyId: string
  policyName: string
  firstResponseThresholdMinutes: number
  resolutionThresholdMinutes: number
  firstResponseTargetAt: string
  resolutionTargetAt: string
  firstResponseAt: string | null
  firstResponseStatus: 'active' | 'fulfilled' | 'breached'
  firstResponseBreached: boolean
  resolutionAt: string | null
  resolutionStatus: 'active' | 'fulfilled' | 'breached'
  resolutionBreached: boolean
  isBreached: boolean
  nearingBreach: boolean
  minutesToFirstResponseBreach: number
  minutesToResolutionBreach: number
  evaluatedAt: string
}

export interface SLAEvaluationResult {
  conversationId: string
  sla: SLAData
  tags: string[]
}

export interface SLAMetrics {
  totalTracked: number
  firstResponseFulfillmentRate: number
  firstResponseFulfilledCount: number
  firstResponseBreachedCount: number
  resolutionFulfillmentRate: number
  resolutionFulfilledCount: number
  resolutionBreachedCount: number
  activeBreachesCount: number
  nearingBreachesCount: number
  policyBreakdown: Array<{
    policyId: string
    policyName: string
    trackedCount: number
    breachedCount: number
    fulfilledCount: number
  }>
}

export interface SLABreachedConversation {
  conversationId: string
  contactName: string
  contactIdentifier: string
  priority: string
  channel: string
  state: string
  policyName: string
  firstResponseStatus: string
  firstResponseTargetAt: string
  resolutionStatus: string
  resolutionTargetAt: string
  isBreached: boolean
  nearingBreach: boolean
  minutesToFirstResponseBreach: number
  minutesToResolutionBreach: number
  updatedAt: string | null
}

export interface RoutingAgent {
  id: string
  name: string
  email: string
  availabilityStatus: 'online' | 'busy' | 'offline'
  maxCapacity: number
  activeConversationsCount: number
  availableSlots: number
  utilizationRatePct: number
  lastAssignedAt: string | null
}

export interface CapacityOverview {
  totalAgents: number
  onlineAgents: number
  busyAgents: number
  offlineAgents: number
  totalMaxCapacity: number
  onlineCapacity: number
  currentActiveConversations: number
  availableOnlineSlots: number
  utilizationRatePct: number
  agents: RoutingAgent[]
}

export interface AutoAssignResult {
  status: 'assigned' | 'overflow'
  assigned: boolean
  conversationId: string
  agentId?: string
  agentName?: string
  strategy?: string
  activeCount?: number
  maxCapacity?: number
  assignedAt?: string
  reason?: string
  tags?: string[]
}


export interface SSOConfigItem {
  id: string
  is_enabled: boolean
  provider_type: 'saml' | 'okta' | 'azure_ad' | 'google_workspace' | 'onelogin'
  idp_entity_id: string
  idp_sso_url: string
  idp_x509_cert: string
  sp_entity_id: string
  sp_acs_url: string
  enforced_domains: string[]
  jit_provisioning_enabled: boolean
  default_role: string
  allow_unlinked_passwords: boolean
  attribute_mapping: Record<string, string>
  last_tested_at: string | null
  last_test_status: string
  last_test_message: string
  created_at: string
  updated_at: string
}

export interface SSOTestResult {
  status: 'healthy' | 'warning' | 'error'
  message: string
  tested_at: string
  checks: Array<{ name: string; passed: boolean; message: string }>
  sample_parsed?: Record<string, any>
}


export interface TranslationResult {
  original_text: string
  translated_text: string
  source_language: string
  target_language: string
  confidence: number
  provider: string
  translated_at?: string
}


export interface CustomAttributeDefinitionItem {
  id: string
  name: string
  display_name: string
  description: string
  attribute_model: 'contact_attribute' | 'conversation_attribute'
  attribute_type: 'text' | 'number' | 'currency' | 'boolean' | 'list' | 'date' | 'link'
  options: string[]
  default_value: string | null
  is_required: boolean
  regex_pattern: string | null
  regex_cue: string | null
  created_at: string
  updated_at: string
}

export const api = {
  listDrafts: () => request<Draft[]>('/drafts?status=pending_review'),
  approveDraft: (id: string) => request<Draft>(`/drafts/${id}/approve`, { method: 'POST' }),
  rejectDraft: (id: string, reason?: string) =>
    request<Draft>(`/drafts/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify({ reason: reason ?? '' }),
    }),
  editDraft: (id: string, body: string) =>
    request<Draft>(`/drafts/${id}/edit`, { method: 'POST', body: JSON.stringify({ body }) }),
  generateDraft: (conversationId: string) =>
    request<Draft>(`/drafts/generate/${conversationId}`, { method: 'POST' }),

  listConversations: () => request<Conversation[]>('/conversations'),
  getConversation: (id: string) => request<ConversationDetail>(`/conversations/${id}`),
  sendMessage: (conversationId: string, body: string) =>
    request<Message>(`/conversations/${conversationId}/messages`, {
      method: 'POST',
      body: JSON.stringify({ body }),
    }),
  resolveConversation: (id: string) =>
    request<{ id: string; state: string; resolutionType: string }>(`/conversations/${id}/resolve`, {
      method: 'POST',
    }),
  enrollSequence: (conversationId: string, sequenceId: string) =>
    request(`/conversations/${conversationId}/enroll-sequence?sequence_id=${sequenceId}`, {
      method: 'POST',
    }),

  listServices: () => request<Service[]>('/services'),
  createService: (data: { name: string; summary: string }) =>
    request<Service>('/services', { method: 'POST', body: JSON.stringify(data) }),

  listSequences: () => request<Sequence[]>('/sequences'),
  sequenceVariantStats: (id: string) => request<VariantStats>(`/sequences/${id}/variant-stats`),

  getIntegrationSettings: () => request<IntegrationSettings>('/settings'),
  updateIntegrationSettings: (payload: {
    whatsapp?: WhatsAppSettingsInput
    business?: BusinessSettingsInput
  }) => request<{ status: string }>('/settings', { method: 'PUT', body: JSON.stringify(payload) }),
  verifyWhatsApp: () => request<VerifyResult>('/settings/whatsapp/verify', { method: 'POST' }),
  listTemplates: () => request<Template[]>('/settings/whatsapp/templates'),
  createTemplate: (data: {
    name: string
    category: string
    language: string
    body_text: string
  }) =>
    request('/settings/whatsapp/templates', { method: 'POST', body: JSON.stringify(data) }),
  deleteTemplate: (name: string) =>
    request(`/settings/whatsapp/templates/${encodeURIComponent(name)}`, { method: 'DELETE' }),

  // Centralized Knowledge Hub & Content Optimization
  getKnowledgeHubOverview: (params?: { q?: string; topic?: string; touchpoint?: string }) => {
    const sp = new URLSearchParams()
    if (params?.q) sp.set('q', params.q)
    if (params?.topic) sp.set('topic', params.topic)
    if (params?.touchpoint) sp.set('touchpoint', params.touchpoint)
    const qs = sp.toString()
    return request<KnowledgeHubOverview>(`/knowledge-base/hub${qs ? `?${qs}` : ''}`)
  },
  listKnowledgeRecommendations: (status?: string, gap_type?: string) => {
    const sp = new URLSearchParams()
    if (status) sp.set('status', status)
    if (gap_type) sp.set('gap_type', gap_type)
    const qs = sp.toString()
    return request<KnowledgeRecommendation[]>(`/knowledge-base/recommendations${qs ? `?${qs}` : ''}`)
  },
  applyKnowledgeRecommendation: (id: string, data?: { custom_content?: string }) =>
    request<{ status: string; recommendation: KnowledgeRecommendation; createdEntry: KnowledgeBaseEntry }>(
      `/knowledge-base/recommendations/${id}/apply`,
      { method: 'POST', body: JSON.stringify(data || {}) }
    ),
  dismissKnowledgeRecommendation: (id: string) =>
    request<{ status: string; recommendation: KnowledgeRecommendation }>(
      `/knowledge-base/recommendations/${id}/dismiss`,
      { method: 'POST' }
    ),

  // Knowledge Base & Omni-Source Learning ("Learns")
  listKnowledgeBase: () => request<KnowledgeBaseEntry[]>('/knowledge-base'),
  createKnowledgeBaseEntry: (data: {
    title: string
    content: string
    source_type?: string
    topic?: string
    target_touchpoints?: string[]
  }) => request<KnowledgeBaseEntry>('/knowledge-base', { method: 'POST', body: JSON.stringify(data) }),
  ingestUrl: (url: string) =>
    request<KnowledgeBaseEntry>('/knowledge-base/ingest-url', {
      method: 'POST',
      body: JSON.stringify({ url }),
    }),
  uploadKnowledgeDocument: async (file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    const res = await fetch(`${BASE}/knowledge-base/upload`, { method: 'POST', body: formData })
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}: ${await res.text()}`)
    return res.json() as Promise<KnowledgeBaseEntry>
  },
  deleteKnowledgeBaseEntry: (id: string) =>
    request(`/knowledge-base/${id}`, { method: 'DELETE' }),

  // Knowledge Gaps & Suggested FAQs ("Fills gaps")
  listKnowledgeGaps: (status?: string) =>
    request<KnowledgeGap[]>(`/knowledge-base/gaps${status ? `?status=${status}` : ''}`),
  scanKnowledgeGaps: () =>
    request<{ status: string; detectedCount: number; gaps: KnowledgeGap[] }>(
      '/knowledge-base/gaps/scan',
      { method: 'POST' }
    ),
  approveKnowledgeGap: (gapId: string, customAnswer?: string) =>
    request<KnowledgeBaseEntry>(`/knowledge-base/gaps/${gapId}/approve`, {
      method: 'POST',
      body: JSON.stringify(customAnswer ? { custom_answer: customAnswer } : {}),
    }),
  dismissKnowledgeGap: (gapId: string) =>
    request<{ status: string }>(`/knowledge-base/gaps/${gapId}/dismiss`, { method: 'POST' }),

  // Contact Memories & Preferences ("Remembers")
  listContactMemories: (contactId: string) =>
    request<ContactMemory[]>(`/contacts/${contactId}/memories`),
  createContactMemory: (contactId: string, data: { fact: string; category?: string }) =>
    request<ContactMemory>(`/contacts/${contactId}/memories`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  deleteContactMemory: (contactId: string, memoryId: string) =>
    request<{ status: string }>(`/contacts/${contactId}/memories/${memoryId}`, {
      method: 'DELETE',
    }),

  listTeam: () => request<TeamMember[]>('/team'),
  createTeamMember: (data: { name: string; email?: string }) =>
    request<{ id: string }>('/team', { method: 'POST', body: JSON.stringify(data) }),
  deleteTeamMember: (id: string) => request(`/team/${id}`, { method: 'DELETE' }),
  assignConversation: (conversationId: string, memberId: string | null) =>
    request(
      `/team/conversations/${conversationId}/assign${memberId ? `?member_id=${memberId}` : ''}`,
      { method: 'POST' },
    ),
  setHumanTakeover: (conversationId: string, enabled: boolean) =>
    request<{ id: string; humanTakeover: boolean }>(
      `/team/conversations/${conversationId}/takeover?enabled=${enabled}`,
      { method: 'POST' },
    ),

  listAutomationRules: () => request<AutomationRule[]>('/automation-rules'),
  createAutomationRule: (data: {
    name: string
    description?: string
    trigger_keyword: string
    match_type: string
    action_type: string
    action_value: string
    priority: number
    is_active: boolean
  }) => request<AutomationRule>('/automation-rules', { method: 'POST', body: JSON.stringify(data) }),
  updateAutomationRule: (id: string, data: Record<string, unknown>) =>
    request<AutomationRule>(`/automation-rules/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  deleteAutomationRule: (id: string) => request(`/automation-rules/${id}`, { method: 'DELETE' }),
  seedDefaultAutomations: () =>
    request<{ status: string; installedCount: number; message: string }>(
      '/automation-rules/seed-defaults',
      { method: 'POST' }
    ),
  testAutomationRule: (data: { text: string; payload_id?: string }) =>
    request<RuleTestResult>('/automation-rules/test', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  getAutomationStats: () => request<AutomationStats>('/automation-rules/stats'),
  runInactivityNudges: () =>
    request<{ status: string; nudgedCount: number }>('/automation-rules/run-nudges', {
      method: 'POST',
    }),

  listContacts: (params?: { tag?: string; channel?: string; search?: string; segment_id?: string } | string) => {
    if (typeof params === 'string') {
      return request<ContactRecord[]>(`/contacts?tag=${encodeURIComponent(params)}`)
    }
    const q = new URLSearchParams()
    if (params?.tag) q.set('tag', params.tag)
    if (params?.channel) q.set('channel', params.channel)
    if (params?.search) q.set('search', params.search)
    if (params?.segment_id) q.set('segment_id', params.segment_id)
    const qs = q.toString()
    return request<ContactRecord[]>(`/contacts${qs ? `?${qs}` : ''}`)
  },
  listContactTags: () => request<{ id: string; name: string }[]>('/contacts/tags'),
  addContactTag: (contactId: string, tag: string) =>
    request(`/contacts/${contactId}/tags`, { method: 'POST', body: JSON.stringify({ tag }) }),
  removeContactTag: (contactId: string, tag: string) =>
    request(`/contacts/${contactId}/tags/${encodeURIComponent(tag)}`, { method: 'DELETE' }),
  previewContactImport: async (file: File, default_channel?: string) => {
    const formData = new FormData()
    formData.append('file', file)
    if (default_channel) formData.append('default_channel', default_channel)
    const res = await fetch(`${BASE}/contacts/import/preview`, { method: 'POST', body: formData })
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}: ${await res.text()}`)
    return res.json() as Promise<ImportPreviewResult>
  },
  importContacts: async (file: File, default_channel?: string, default_tags?: string) => {
    const formData = new FormData()
    formData.append('file', file)
    if (default_channel) formData.append('default_channel', default_channel)
    if (default_tags) formData.append('default_tags', default_tags)
    const res = await fetch(`${BASE}/contacts/import`, { method: 'POST', body: formData })
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}: ${await res.text()}`)
    return res.json() as Promise<MultiChannelImportResult>
  },
  exportContactsUrl: (params?: { channel?: string; tag?: string; search?: string; segment_id?: string }) => {
    const q = new URLSearchParams()
    if (params?.channel) q.set('channel', params.channel)
    if (params?.tag) q.set('tag', params.tag)
    if (params?.search) q.set('search', params.search)
    if (params?.segment_id) q.set('segment_id', params.segment_id)
    const qs = q.toString()
    return `${BASE}/contacts/export${qs ? `?${qs}` : ''}`
  },

  // Saved Dynamic Contact Segments
  listSegments: () => request<ContactSegment[]>('/segments'),
  createSegment: (data: { name: string; description?: string; filters?: any }) =>
    request<ContactSegment>('/segments', { method: 'POST', body: JSON.stringify(data) }),
  getSegment: (id: string) => request<ContactSegment & { contacts: any[] }>(`/segments/${id}`),
  updateSegment: (id: string, data: Partial<ContactSegment>) =>
    request<ContactSegment>(`/segments/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  deleteSegment: (id: string) => request<{ status: string; deleted: boolean }>(`/segments/${id}`, { method: 'DELETE' }),
  exportSegmentUrl: (id: string) => `${BASE}/segments/${id}/export`,

  // WhatsApp Campaigns Suite
  listBroadcasts: () => request<Broadcast[]>('/broadcasts'),
  createBroadcast: (data: {
    name: string
    template_name: string
    template_language?: string
    body_preview?: string
    template_params?: Record<string, string>
    tag_name: string
    scheduled_at?: string | null
  }) => request<Broadcast>('/broadcasts', { method: 'POST', body: JSON.stringify(data) }),
  getAudiencePreview: (tags: string) =>
    request<AudiencePreview>(`/broadcasts/audience-preview?tags=${encodeURIComponent(tags)}`),
  getCampaignAnalytics: (campaignId: string) =>
    request<CampaignAnalytics>(`/broadcasts/${campaignId}/analytics`),
  rescheduleCampaign: (campaignId: string, scheduledAt: string | null) =>
    request<Broadcast>(`/broadcasts/${campaignId}/reschedule`, {
      method: 'POST',
      body: JSON.stringify({ scheduled_at: scheduledAt }),
    }),
  dispatchBroadcast: (id: string) =>
    request<Broadcast>(`/broadcasts/${id}/dispatch`, { method: 'POST' }),
  processScheduledCampaigns: () =>
    request<{ processed: number; campaigns: Broadcast[] }>('/broadcasts/process-scheduled', {
      method: 'POST',
    }),
  runAllSchedulers: () =>
    request<{
      followupsSent: number
      draftsAutoSent: number
      inactivityNudges: number
      campaignsDispatched: number
    }>('/scheduler/run-all', { method: 'POST' }),
  listBroadcastRecipients: (id: string) =>
    request<BroadcastRecipient[]>(`/broadcasts/${id}/recipients`),
  getWhatsAppPricing: (region: string, category: string, volume: number) =>
    request<PricingEstimate>(
      `/broadcasts/pricing-calculator?region=${encodeURIComponent(region)}&category=${encodeURIComponent(category)}&volume=${volume}`
    ),
  seedDefaultCampaigns: () =>
    request<{ status: string; count?: number }>('/broadcasts/seed-defaults', { method: 'POST' }),

  // Channels & Inboxes API (Starting with WhatsApp)
  listInboxes: () => request<Inbox[]>('/inboxes'),
  getWhatsAppChannel: () => request<WhatsAppChannelData>('/inboxes/channels/whatsapp'),
  updateWhatsAppChannel: (data: any) =>
    request<WhatsAppChannelData>('/inboxes/channels/whatsapp', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyWhatsAppConnection: () =>
    request<{ status: string; health: any }>('/inboxes/channels/whatsapp/verify', {
      method: 'POST',
    }),
  listWhatsAppTemplates: () => request<WhatsAppTemplate[]>('/inboxes/channels/whatsapp/templates'),
  syncWhatsAppTemplates: () =>
    request<{ status: string; message: string }>('/inboxes/channels/whatsapp/sync-templates', {
      method: 'POST',
    }),
  sendWhatsAppTestMessage: (recipient_phone: string, message?: string) =>
    request<any>('/inboxes/channels/whatsapp/test-message', {
      method: 'POST',
      body: JSON.stringify({ recipient_phone, message }),
    }),
  listWhatsAppEvents: () => request<any[]>('/inboxes/channels/whatsapp/events'),
  getWhatsAppProfile: () => request<WhatsAppBusinessProfile>('/inboxes/channels/whatsapp/profile'),
  updateWhatsAppProfile: (data: WhatsAppBusinessProfile) =>
    request<{ status: string; profile: WhatsAppBusinessProfile; metaSynced: boolean }>(
      '/inboxes/channels/whatsapp/profile',
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    ),
  registerWhatsAppPhone: (pin: string) =>
    request<{ status: string; message: string }>('/inboxes/channels/whatsapp/register', {
      method: 'POST',
      body: JSON.stringify({ pin }),
    }),

  // Channel 2: Website Live Chat Integration
  getLiveChatChannel: () => request<LiveChatChannelData>('/inboxes/channels/live-chat'),
  updateLiveChatChannel: (data: LiveChatUpdatePayload) =>
    request<LiveChatChannelData>('/inboxes/channels/live-chat', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  regenerateLiveChatToken: () =>
    request<{ websiteToken: string; embedScript: string; message: string }>(
      '/inboxes/channels/live-chat/regenerate-token',
      { method: 'POST' }
    ),
  sendLiveChatTestVisitor: (data: LiveChatTestVisitorPayload) =>
    request<{
      status: string
      conversationId: string
      contactId: string
      messageId: string
      message: string
    }>('/inboxes/channels/live-chat/test-visitor', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listLiveChatEvents: () => request<LiveChatEvent[]>('/inboxes/channels/live-chat/events'),

  // Channel 3: SMS Messaging (Twilio)
  getSMSChannel: () => request<SMSChannelData>('/inboxes/channels/sms'),
  updateSMSChannel: (data: SMSUpdatePayload) =>
    request<SMSChannelData>('/inboxes/channels/sms', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifySMSConnection: () =>
    request<{
      status: string
      verified: boolean
      accountSid: string
      phoneNumber: string
      a2p10dlcStatus: string
      message: string
    }>('/inboxes/channels/sms/verify', { method: 'POST' }),
  sendSMSTestMessage: (data: SMSTestMessagePayload) =>
    request<{
      status: string
      sid: string
      recipient: string
      message: string
    }>('/inboxes/channels/sms/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listSMSEvents: () => request<SMSEvent[]>('/inboxes/channels/sms/events'),

  // Channel 4: Socials (Facebook Messenger & Instagram Direct Messages)
  getFacebookChannel: () => request<FacebookChannelData>('/inboxes/channels/facebook'),
  updateFacebookChannel: (data: FacebookUpdatePayload) =>
    request<FacebookChannelData>('/inboxes/channels/facebook', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyFacebookConnection: () =>
    request<{
      status: string
      verified: boolean
      pageId: string
      pageName: string
      graphApiVersion: string
      message: string
    }>('/inboxes/channels/facebook/verify', { method: 'POST' }),
  sendFacebookTestMessage: (data: FacebookTestMessagePayload) =>
    request<{
      status: string
      mid: string
      recipient: string
      message: string
    }>('/inboxes/channels/facebook/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listFacebookEvents: () => request<SocialEvent[]>('/inboxes/channels/facebook/events'),

  getInstagramChannel: () => request<InstagramChannelData>('/inboxes/channels/instagram'),
  updateInstagramChannel: (data: InstagramUpdatePayload) =>
    request<InstagramChannelData>('/inboxes/channels/instagram', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyInstagramConnection: () =>
    request<{
      status: string
      verified: boolean
      igId: string
      igUsername: string
      graphApiVersion: string
      message: string
    }>('/inboxes/channels/instagram/verify', { method: 'POST' }),
  sendInstagramTestMessage: (data: InstagramTestMessagePayload) =>
    request<{
      status: string
      mid: string
      recipient: string
      message: string
    }>('/inboxes/channels/instagram/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listInstagramEvents: () => request<SocialEvent[]>('/inboxes/channels/instagram/events'),

  // Channel 5: Telegram & LINE Messaging APIs
  getTelegramChannel: () => request<TelegramChannelData>('/inboxes/channels/telegram'),
  updateTelegramChannel: (data: TelegramUpdatePayload) =>
    request<TelegramChannelData>('/inboxes/channels/telegram', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyTelegramConnection: () =>
    request<{
      status: string
      verified: boolean
      botUsername: string
      telegramApi: string
      message: string
    }>('/inboxes/channels/telegram/verify', { method: 'POST' }),
  sendTelegramTestMessage: (data: TelegramTestMessagePayload) =>
    request<{
      status: string
      messageId: string
      recipient: string
      message: string
    }>('/inboxes/channels/telegram/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listTelegramEvents: () => request<SocialEvent[]>('/inboxes/channels/telegram/events'),

  getLineChannel: () => request<LineChannelData>('/inboxes/channels/line'),
  updateLineChannel: (data: LineUpdatePayload) =>
    request<LineChannelData>('/inboxes/channels/line', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyLineConnection: () =>
    request<{
      status: string
      verified: boolean
      channelId: string
      lineAccountId: string
      lineApi: string
      message: string
    }>('/inboxes/channels/line/verify', { method: 'POST' }),
  sendLineTestMessage: (data: LineTestMessagePayload) =>
    request<{
      status: string
      messageId: string
      recipient: string
      message: string
    }>('/inboxes/channels/line/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listLineEvents: () => request<SocialEvent[]>('/inboxes/channels/line/events'),

  // Voice & WebRTC Channel
  getVoiceChannel: () => request<VoiceChannelData>('/inboxes/channels/voice'),
  updateVoiceChannel: (data: VoiceUpdatePayload) =>
    request<VoiceChannelData>('/inboxes/channels/voice', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyVoiceChannel: () =>
    request<{
      status: string
      verified: boolean
      sipDomainReachable: boolean
      twimlAppConfigured: boolean
      callerIdVerified: boolean
      message: string
    }>('/inboxes/channels/voice/verify', { method: 'POST' }),
  getVoiceWebRtcToken: (identity: string = 'agent-inbox') =>
    request<VoiceWebRtcTokenResponse>('/inboxes/channels/voice/token', {
      method: 'POST',
      body: JSON.stringify({ identity }),
    }),
  initiateVoiceCall: (data: VoiceTestCallPayload) =>
    request<{
      status: string
      callSid: string
      conversationId: string
      to: string
      callerId: string
      message: string
    }>('/inboxes/channels/voice/call', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  hangupVoiceCall: (callSid: string) =>
    request<{
      status: string
      callSid: string
      result: any
    }>('/inboxes/channels/voice/hangup', {
      method: 'POST',
      body: JSON.stringify({ call_sid: callSid }),
    }),
  getVoiceCallLogs: () => request<VoiceCallLogItem[]>('/inboxes/channels/voice/call-logs'),
  listVoiceEvents: () => request<SocialEvent[]>('/inboxes/channels/voice/events'),

  // Support Email Channel
  getEmailChannel: () => request<EmailChannelData>('/inboxes/channels/email'),
  updateEmailChannel: (data: EmailUpdatePayload) =>
    request<EmailChannelData>('/inboxes/channels/email', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyEmailChannel: () =>
    request<{
      status: string
      verified: boolean
      smtpVerified: boolean
      imapVerified: boolean
      spfDkimVerified: boolean
      details: any
      message: string
    }>('/inboxes/channels/email/verify', { method: 'POST' }),
  sendEmailTestMessage: (data: EmailTestMessagePayload) =>
    request<{
      status: string
      messageId: string
      conversationId: string
      recipient: string
      subject: string
      message: string
    }>('/inboxes/channels/email/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  syncEmailImap: () => request<EmailSyncImapResponse>('/inboxes/channels/email/sync-imap', { method: 'POST' }),
  listEmailEvents: () => request<SocialEvent[]>('/inboxes/channels/email/events'),

  // TikTok Channel Integration
  getTikTokChannel: () => request<TikTokChannelData>('/inboxes/channels/tiktok'),
  updateTikTokChannel: (data: TikTokUpdatePayload) =>
    request<TikTokChannelData>('/inboxes/channels/tiktok', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyTikTokConnection: () =>
    request<{
      status: string
      verified: boolean
      appId: string
      businessAccountId: string
      tiktokApi: string
      message: string
    }>('/inboxes/channels/tiktok/verify', { method: 'POST' }),
  sendTikTokTestMessage: (data: TikTokTestMessagePayload) =>
    request<{
      status: string
      messageId: string
      recipient: string
      message: string
    }>('/inboxes/channels/tiktok/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  sendTikTokTestCommentReply: (data: TikTokTestCommentPayload) =>
    request<{
      status: string
      replyId: string
      commentId: string
      videoId: string
      recipient: string
      message: string
    }>('/inboxes/channels/tiktok/test-comment-reply', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listTikTokEvents: () => request<SocialEvent[]>('/inboxes/channels/tiktok/events'),


  // Teams & Queues
  listTeams: () => request<Team[]>('/teams'),
  createTeam: (data: { name: string; description?: string; auto_assign?: boolean }) =>
    request<Team>('/teams', { method: 'POST', body: JSON.stringify(data) }),
  deleteTeam: (id: string) => request(`/teams/${id}`, { method: 'DELETE' }),

  // Canned Responses (Saved Replies)
  listCannedResponses: (params?: { q?: string; category?: string }) => {
    const searchParams = new URLSearchParams()
    if (params?.q) searchParams.set('q', params.q)
    if (params?.category) searchParams.set('category', params.category)
    const qs = searchParams.toString()
    return request<CannedResponse[]>(`/canned-responses${qs ? `?${qs}` : ''}`)
  },
  createCannedResponse: (data: { short_code: string; content: string; category?: string }) =>
    request<CannedResponse>('/canned-responses', { method: 'POST', body: JSON.stringify(data) }),
  updateCannedResponse: (id: string, data: { short_code?: string; content?: string; category?: string }) =>
    request<CannedResponse>(`/canned-responses/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  deleteCannedResponse: (id: string) => request(`/canned-responses/${id}`, { method: 'DELETE' }),
  expandCannedResponse: (data: { short_code: string; conversation_id?: string; custom_vars?: Record<string, string> }) =>
    request<{ shortCode: string; rawContent: string; renderedContent: string; variables: Record<string, string> }>(
      '/canned-responses/expand',
      { method: 'POST', body: JSON.stringify(data) }
    ),

  // Copilot Teammate Approvals & Human-in-the-Loop Workflows
  listCopilotApprovals: (params?: { conversation_id?: string; status?: string; request_type?: string }) => {
    const sp = new URLSearchParams()
    if (params?.conversation_id) sp.set('conversation_id', params.conversation_id)
    if (params?.status) sp.set('status', params.status)
    if (params?.request_type) sp.set('request_type', params.request_type)
    const qs = sp.toString()
    return request<CopilotApprovalRequest[]>(`/copilot/approvals${qs ? `?${qs}` : ''}`)
  },
  createCopilotApproval: (data: {
    conversation_id: string
    title: string
    proposed_reply: string
    request_type?: string
    action_data?: Record<string, any>
    internal_note?: string
    question_to_teammate?: string
    assigned_to_id?: string
  }) => request<CopilotApprovalRequest>('/copilot/approvals', { method: 'POST', body: JSON.stringify(data) }),
  getCopilotApproval: (id: string) => request<CopilotApprovalRequest>(`/copilot/approvals/${id}`),
  approveCopilotRequest: (id: string, data: { reviewer_name?: string; amended_reply?: string; execute_send?: boolean }) =>
    request<CopilotApprovalRequest>(`/copilot/approvals/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  rejectCopilotRequest: (id: string, data: { reviewer_name?: string; feedback?: string }) =>
    request<CopilotApprovalRequest>(`/copilot/approvals/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  provideCopilotInfo: (id: string, data: { reviewer_name?: string; teammate_answer: string }) =>
    request<CopilotApprovalRequest>(`/copilot/approvals/${id}/provide-info`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  evaluateCopilotTrigger: (data: { conversation_id: string; customer_message: string }) =>
    request<{ triggered: boolean; approval: CopilotApprovalRequest | null }>('/copilot/approvals/evaluate-trigger', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // Shared Inbox Collaboration & Actions
  sendConversationMessage: (id: string, body: string, is_private?: boolean, sender_name?: string) =>
    request<any>(`/conversations/${id}/messages`, {
      method: 'POST',
      body: JSON.stringify({ body, is_private, sender_name }),
    }),
  updatePriority: (id: string, priority: string) =>
    request<any>(`/conversations/${id}/priority`, {
      method: 'POST',
      body: JSON.stringify({ priority }),
    }),
  assignTeam: (id: string, team_id: string | null) =>
    request<any>(`/conversations/${id}/assign-team`, {
      method: 'POST',
      body: JSON.stringify({ team_id }),
    }),
  assignAgent: (id: string, agent_id: string | null) =>
    request<any>(`/conversations/${id}/assign-agent`, {
      method: 'POST',
      body: JSON.stringify({ agent_id }),
    }),
  snoozeConversation: (id: string, snoozed_until: string | null) =>
    request<any>(`/conversations/${id}/snooze`, {
      method: 'POST',
      body: JSON.stringify({ snoozed_until }),
    }),
  updateCustomAttributes: (id: string, attributes: Record<string, any>) =>
    request<any>(`/conversations/${id}/custom-attributes`, {
      method: 'POST',
      body: JSON.stringify({ attributes }),
    }),
  listContactNotes: (contactId: string) =>
    request<ContactNote[]>(`/conversations/contacts/${contactId}/notes`),
  addContactNote: (contactId: string, body: string, authorName?: string) =>
    request<ContactNote>(`/conversations/contacts/${contactId}/notes`, {
      method: 'POST',
      body: JSON.stringify({ body, author_name: authorName }),
    }),

  // Reporting & Analytics Suite
  getLiveReport: () => request<LiveReportData>('/reports/live'),
  getConversationReports: (period: '7d' | '30d' | '90d' = '7d', channel?: string) =>
    request<ConversationReportData>(`/reports/conversations?period=${period}${channel ? `&channel=${encodeURIComponent(channel)}` : ''}`),
  getAgentReports: (period: '7d' | '30d' | '90d' = '7d') =>
    request<AgentReportItem[]>(`/reports/agents?period=${period}`),
  getTeamReports: (period: '7d' | '30d' | '90d' = '7d') =>
    request<TeamReportItem[]>(`/reports/teams?period=${period}`),
  getCsatReport: (period: '7d' | '30d' | '90d' = '7d') =>
    request<CsatReportData>(`/reports/csat?period=${period}`),

  // Captain AI Copilot, Triage & Monitors
  getCaptainSuggestedReply: (conversationId: string, instruction?: string) =>
    request<CaptainSuggestedReply>(`/conversations/${conversationId}/ai-copilot/suggested-reply`, {
      method: 'POST',
      body: JSON.stringify({ instruction }),
    }),
  summarizeConversationThread: (conversationId: string) =>
    request<CaptainSummary>(`/conversations/${conversationId}/ai-copilot/summarize`, {
      method: 'POST',
    }),
  rewriteDraftText: (text: string, tone: 'professional' | 'friendly' | 'shorten' | 'fix_spelling' | 'expand' = 'professional') =>
    request<CaptainRewriteResult>('/ai-copilot/rewrite', {
      method: 'POST',
      body: JSON.stringify({ text, tone }),
    }),
  triageConversation: (conversationId: string, apply: boolean = false) =>
    request<CaptainTriageResult>(`/conversations/${conversationId}/ai-copilot/triage`, {
      method: 'POST',
      body: JSON.stringify({ apply }),
    }),
  getCaptainMonitors: () => request<CaptainMonitorsData>('/captain/monitors'),
  createCaptainMonitor: (name: string, topic: string, keywords: string[], threshold: number = 5) =>
    request<CaptainMonitorItem>('/captain/monitors', {
      method: 'POST',
      body: JSON.stringify({ name, topic, keywords, threshold }),
    }),

  // Real-Time Collision Detection & Live Agent Presence
  updateConversationPresence: (
    conversationId: string,
    agentName: string,
    action: 'viewing' | 'typing' | 'left',
    agentAvatar?: string
  ) =>
    request<{ status: string; presence: AgentPresenceItem }>(`/conversations/${conversationId}/presence`, {
      method: 'POST',
      body: JSON.stringify({ agent_name: agentName, action, agent_avatar: agentAvatar }),
    }),
  getConversationPresence: (conversationId: string) =>
    request<AgentPresenceItem[]>(`/conversations/${conversationId}/presence`),
  checkConversationCollision: (conversationId: string, currentAgent: string = 'Riley Brooks') =>
    request<CollisionCheckResult>(
      `/conversations/${conversationId}/collision-check?current_agent=${encodeURIComponent(currentAgent)}`
    ),
  leaveConversationPresence: (conversationId: string, agentName: string) =>
    request<{ status: string; removed: boolean }>(
      `/conversations/${conversationId}/presence?agent_name=${encodeURIComponent(agentName)}`,
      { method: 'DELETE' }
    ),

  // Macros & One-Click Workflows
  listMacros: () => request<MacroItem[]>('/macros'),
  getMacro: (macroId: string) => request<MacroItem>(`/macros/${macroId}`),
  createMacro: (payload: { name: string; description?: string; actions: MacroAction[] }) =>
    request<MacroItem>('/macros', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  updateMacro: (macroId: string, payload: { name?: string; description?: string; actions?: MacroAction[] }) =>
    request<MacroItem>(`/macros/${macroId}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),
  deleteMacro: (macroId: string) =>
    request<{ status: string; deleted: boolean }>(`/macros/${macroId}`, {
      method: 'DELETE',
    }),
  executeMacro: (conversationId: string, macroId: string, agentName: string = 'Riley Brooks') =>
    request<MacroExecutionResult>(`/conversations/${conversationId}/macros/${macroId}/execute`, {
      method: 'POST',
      body: JSON.stringify({ agent_name: agentName }),
    }),

  // Dynamic SLA Management & Breach Detection
  listSLAPolicies: () => request<SLAPolicy[]>('/sla/policies'),
  getSLAPolicy: (policyId: string) => request<SLAPolicy>(`/sla/policies/${policyId}`),
  createSLAPolicy: (payload: {
    name: string
    description?: string
    first_response_time_threshold?: number
    resolution_time_threshold?: number
    only_during_business_hours?: boolean
    priorities?: string[]
    inbox_ids?: string[]
  }) =>
    request<SLAPolicy>('/sla/policies', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  updateSLAPolicy: (policyId: string, payload: Partial<SLAPolicy>) =>
    request<SLAPolicy>(`/sla/policies/${policyId}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),
  deleteSLAPolicy: (policyId: string) =>
    request<{ status: string; deleted: boolean }>(`/sla/policies/${policyId}`, {
      method: 'DELETE',
    }),
  getSLAMetrics: () => request<SLAMetrics>('/sla/metrics'),
  getSLABreaches: () => request<SLABreachedConversation[]>('/sla/breaches'),
  evaluateConversationSLA: (conversationId: string, policyId?: string) =>
    request<SLAEvaluationResult>(`/conversations/${conversationId}/sla/evaluate`, {
      method: 'POST',
      body: JSON.stringify({ policy_id: policyId }),
    }),

  // Auto-Assignment & Agent Capacity Routing
  listRoutingAgents: () => request<RoutingAgent[]>('/routing/agents'),
  updateAgentStatus: (agentId: string, status: 'online' | 'busy' | 'offline') =>
    request<{ id: string; name: string; availabilityStatus: string; maxCapacity: number }>(
      `/routing/agents/${agentId}/status`,
      { method: 'PUT', body: JSON.stringify({ status }) }
    ),
  updateAgentCapacity: (agentId: string, max_capacity: number) =>
    request<{ id: string; name: string; availabilityStatus: string; maxCapacity: number }>(
      `/routing/agents/${agentId}/capacity`,
      { method: 'PUT', body: JSON.stringify({ max_capacity }) }
    ),
  getCapacityOverview: (teamId?: string) =>
    request<CapacityOverview>(`/routing/capacity-overview${teamId ? `?team_id=${teamId}` : ''}`),
  autoAssignConversation: (
    conversationId: string,
    strategy: 'capacity_based' | 'round_robin' = 'capacity_based',
    teamId?: string
  ) =>
    request<AutoAssignResult>(`/conversations/${conversationId}/auto-assign`, {
      method: 'POST',
      body: JSON.stringify({ strategy, team_id: teamId }),
    }),
  batchAutoAssign: (strategy: 'capacity_based' | 'round_robin' = 'capacity_based', teamId?: string) =>
    request<{ totalProcessed: number; assignedCount: number; overflowCount: number; results: AutoAssignResult[] }>(
      '/routing/auto-assign-batch',
      { method: 'POST', body: JSON.stringify({ strategy, team_id: teamId }) }
    ),

  // Unified Ticketing & Back-Office Collaboration
  listTickets: (params?: {
    ticket_type?: string
    status?: string
    department?: string
    tracker_id?: string
    conversation_id?: string
    search?: string
  }) => {
    const searchParams = new URLSearchParams()
    if (params?.ticket_type) searchParams.set('ticket_type', params.ticket_type)
    if (params?.status) searchParams.set('status', params.status)
    if (params?.department) searchParams.set('department', params.department)
    if (params?.tracker_id) searchParams.set('tracker_id', params.tracker_id)
    if (params?.conversation_id) searchParams.set('conversation_id', params.conversation_id)
    if (params?.search) searchParams.set('search', params.search)
    const qs = searchParams.toString()
    return request<Ticket[]>(`/tickets${qs ? `?${qs}` : ''}`)
  },
  getTicketStats: () => request<TicketStats>('/tickets/stats'),
  getTrackers: () => request<Ticket[]>('/tickets/trackers'),
  getTicket: (id: string) => request<Ticket>(`/tickets/${id}`),
  createTicket: (data: {
    title: string
    description: string
    ticket_type?: string
    department?: string
    priority?: string
    status?: string
    conversation_id?: string | null
    contact_id?: string | null
    tracker_id?: string | null
    assigned_to_id?: string | null
    author_name?: string
  }) =>
    request<Ticket>('/tickets', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateTicket: (
    id: string,
    data: {
      title?: string
      description?: string
      status?: string
      priority?: string
      department?: string
      assigned_to_id?: string | null
      resolution_summary?: string | null
      author_name?: string
    }
  ) =>
    request<Ticket>(`/tickets/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),
  addTicketActivity: (
    id: string,
    data: {
      message: string
      activity_type?: string
      author_id?: string | null
      author_name?: string
      metadata?: Record<string, any>
    }
  ) =>
    request<TicketActivity>(`/tickets/${id}/activities`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  linkTicketToTracker: (ticketId: string, trackerId: string, authorName?: string) =>
    request<Ticket>(`/tickets/${ticketId}/link-tracker`, {
      method: 'POST',
      body: JSON.stringify({ tracker_id: trackerId, author_name: authorName }),
    }),
  broadcastTrackerUpdate: (
    trackerId: string,
    data: {
      broadcast_message: string
      resolve_all_children?: boolean
      send_customer_message?: boolean
      author_name?: string
    }
  ) =>
    request<{
      tracker_id: string
      tracker_number: string
      impacted_tickets: number
      messages_sent: number
      resolved_tickets: number
      tracker_resolved: boolean
    }>(`/tickets/${trackerId}/broadcast`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // AI Agent Policy Training & Simulations
  listAgentPolicies: (category?: string) =>
    request<AgentPolicy[]>(`/simulations/policies${category ? `?category=${category}` : ''}`),
  createAgentPolicy: (data: Partial<AgentPolicy>) =>
    request<AgentPolicy>('/simulations/policies', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listSimulationScenarios: (category?: string) =>
    request<SimulationScenario[]>(`/simulations/scenarios${category ? `?category=${category}` : ''}`),
  runSimulation: (scenario_id: string, policy_id?: string) =>
    request<SimulationRun>('/simulations/run', {
      method: 'POST',
      body: JSON.stringify({ scenario_id, policy_id }),
    }),
  runBatchSimulations: () =>
    request<{
      total_scenarios_tested: number
      passed_count: number
      pass_rate_percentage: number
      average_score: number
      results: Array<{
        scenario_id: string
        scenario_name: string
        verdict: string
        overall_score: number
        policy_adherence: number
      }>
    }>('/simulations/batch', {
      method: 'POST',
    }),
  sandboxChat: (message: string, history?: any[], policy_id?: string) =>
    request<SandboxChatResult>('/simulations/sandbox/chat', {
      method: 'POST',
      body: JSON.stringify({ message, history: history || [], policy_id }),
    }),
  getSimulationStats: () => request<SimulationStats>('/simulations/stats'),

  // Self-Improving AI & Continuous Retraining
  listInsights: (params?: { category?: string; status?: string }) => {
    const searchParams = new URLSearchParams()
    if (params?.category) searchParams.set('category', params.category)
    if (params?.status) searchParams.set('status', params.status)
    const qs = searchParams.toString()
    return request<AgentInsight[]>(`/self-improving/insights${qs ? `?${qs}` : ''}`)
  },
  applyInsightRetraining: (insightId: string, authorName?: string) =>
    request<{
      retraining_event_id: string
      title: string
      version_from: string
      version_to: string
      baseline_resolution_rate: number
      post_training_resolution_rate: number
      resolution_lift: number
      status: string
    }>(`/self-improving/insights/${insightId}/apply`, {
      method: 'POST',
      body: JSON.stringify({ author_name: authorName || 'Manager' }),
    }),
  dismissInsight: (insightId: string) =>
    request<{ status: string; insight_id: string }>(`/self-improving/insights/${insightId}/dismiss`, {
      method: 'POST',
    }),
  listRetrainingHistory: () =>
    request<AgentRetrainingEvent[]>('/self-improving/retraining-history'),
  triggerAutoLearningScan: () =>
    request<{
      conversations_scanned: number
      high_csat_patterns_analyzed: number
      new_insights_surfaced: number
      status: string
    }>('/self-improving/scan', {
      method: 'POST',
    }),
  getSelfImprovingStats: () =>
    request<SelfImprovingStats>('/self-improving/stats'),

  // Connect Any AI Model & Custom Policies
  listAIConnections: () => request<AIConnection[]>('/ai/connections'),
  createAIConnection: (data: {
    name: string
    provider: string
    model_id: string
    api_key?: string
    base_url?: string | null
    temperature?: number
    max_tokens?: number
    is_active?: boolean
    assigned_jobs?: string[]
  }) =>
    request<{ id: string; name: string; provider: string; model_id: string; test_status: string; is_active: boolean }>(
      '/ai/connections',
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    ),
  testAIConnection: (connectionId: string) =>
    request<{ connection_id: string; name: string; provider: string; model_id: string; status: string; latency_ms: number; message: string }>(
      `/ai/connections/${connectionId}/test`,
      { method: 'POST' }
    ),
  deleteAIConnection: (connectionId: string) =>
    request<{ status: string; connection_id: string }>(`/ai/connections/${connectionId}`, {
      method: 'DELETE',
    }),
  getJobModelMapping: () => request<JobModelMapping>('/ai/jobs/mapping'),
  assignModelToJob: (job_type: string, connection_id: string) =>
    request<{ job_type: string; assigned_to: string; model_id: string; provider: string; status: string }>(
      '/ai/jobs/mapping',
      {
        method: 'POST',
        body: JSON.stringify({ job_type, connection_id }),
      }
    ),
  getCustomPolicy: (job_type: string) => request<AICustomPolicy>(`/ai/policies/${job_type}`),
  updateCustomPolicy: (job_type: string, data: Partial<AICustomPolicy>) =>
    request<AICustomPolicy>(`/ai/policies/${job_type}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  getEffectivePrompt: (job_type: string) =>
    request<{ job_type: string; prompt: string }>(`/ai/prompts/${job_type}`),

  // Universal Multi-Provider Gateways (Any SMS, Any Email, Any VoIP/SIP)
  listGateways: (gateway_type?: string) =>
    request<ChannelGateway[]>(gateway_type ? `/gateways?gateway_type=${gateway_type}` : '/gateways'),
  getSupportedGatewayProviders: () =>
    request<SupportedProvidersCatalog>('/gateways/providers/supported'),
  createGateway: (data: {
    gateway_type: string
    provider: string
    name: string
    credentials: Record<string, any>
    config: Record<string, any>
    is_default?: boolean
  }) =>
    request<ChannelGateway>('/gateways', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateGateway: (
    id: string,
    data: {
      name?: string
      credentials?: Record<string, any>
      config?: Record<string, any>
      is_active?: boolean
    }
  ) =>
    request<ChannelGateway>(`/gateways/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  deleteGateway: (id: string) =>
    request<{ success: boolean; deleted_id: string }>(`/gateways/${id}`, {
      method: 'DELETE',
    }),
  testGatewayConnection: (id: string) =>
    request<{
      success: boolean
      gateway_id: string
      gateway_name: string
      provider: string
      test_status: string
      latency_ms: number
      error?: string | null
      checked_at: string
    }>(`/gateways/${id}/test`, {
      method: 'POST',
    }),
  setDefaultGateway: (id: string) =>
    request<ChannelGateway>(`/gateways/${id}/set-default`, {
      method: 'POST',
    }),
  sendGatewayPing: (id: string, target: string, message?: string) =>
    request<{
      success: boolean
      provider: string
      gateway_type: string
      target: string
      message_id: string
      latency_ms: number
      simulated: boolean
      timestamp: string
    }>(`/gateways/${id}/ping`, {
      method: 'POST',
      body: JSON.stringify({ target, message: message || 'Test transmission' }),
    }),

  // Outgoing Webhooks Engine & Event Dispatcher
  listWebhookEndpoints: () => request<WebhookEndpoint[]>('/outgoing-webhooks'),
  getWebhookCatalog: () => request<WebhookEventCatalogItem[]>('/outgoing-webhooks/events/catalog'),
  getWebhookDeliveries: (endpointId?: string, limit?: number) =>
    request<WebhookDeliveryLog[]>(
      endpointId
        ? `/outgoing-webhooks/${endpointId}/deliveries?limit=${limit || 50}`
        : `/outgoing-webhooks/deliveries?limit=${limit || 50}`
    ),
  createWebhookEndpoint: (data: {
    name: string
    url: string
    events: string[]
    secret?: string
    headers?: Record<string, string>
    is_active?: boolean
    retry_count?: number
    timeout_seconds?: number
  }) =>
    request<WebhookEndpoint>('/outgoing-webhooks', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateWebhookEndpoint: (
    id: string,
    data: {
      name?: string
      url?: string
      events?: string[]
      headers?: Record<string, string>
      is_active?: boolean
      retry_count?: number
      timeout_seconds?: number
      rotate_secret?: boolean
    }
  ) =>
    request<WebhookEndpoint>(`/outgoing-webhooks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  deleteWebhookEndpoint: (id: string) =>
    request<{ success: boolean; deleted_id: string }>(`/outgoing-webhooks/${id}`, {
      method: 'DELETE',
    }),
  testWebhookPing: (id: string) =>
    request<{
      success: boolean
      delivery_id: string
      endpoint_id: string
      url: string
      status_code: number
      duration_ms: number
      response_body: string
      error?: string | null
      tested_at: string
    }>(`/outgoing-webhooks/${id}/test`, {
      method: 'POST',
    }),
  resendWebhookDelivery: (deliveryId: string) =>
    request<{
      success: boolean
      original_delivery_id: string
      new_delivery_id: string
      status_code: number
      duration_ms: number
      attempt: number
    }>(`/outgoing-webhooks/deliveries/${deliveryId}/resend`, {
      method: 'POST',
    }),

  // Stripe Billing Integration
  getStripeBilling: (identifier: string) =>
    request<StripeBillingProfile>(`/integrations/stripe/billing/${encodeURIComponent(identifier)}`),
  getStripeInvoices: (identifier: string, limit: number = 10) =>
    request<StripeInvoice[]>(
      `/integrations/stripe/invoices/${encodeURIComponent(identifier)}?limit=${limit}`
    ),
  processStripeRefund: (data: {
    charge_or_invoice_id: string
    amount_cents?: number
    reason?: string
    actor_email?: string
    contact_id?: string
    customer_identifier?: string
  }) =>
    request<{
      id: string
      charge_id: string
      amount_cents: number
      amount_formatted: string
      currency: string
      status: string
      reason: string
      actor_email: string
      created_at: string
      log_id: string
    }>('/integrations/stripe/refund', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  createStripePortalSession: (data: {
    identifier: string
    return_url?: string
    actor_email?: string
  }) =>
    request<{
      session_id: string
      url: string
      customer_identifier: string
      created_at: string
    }>('/integrations/stripe/portal-session', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  getStripeBillingLogs: (identifier: string, limit: number = 25) =>
    request<StripeBillingLog[]>(
      `/integrations/stripe/logs/${encodeURIComponent(identifier)}?limit=${limit}`
    ),
  getStripeConfig: () => request<StripeConfig>('/integrations/stripe/config'),
  updateStripeConfig: (data: Partial<StripeConfig>) =>
    request<StripeConfig>('/integrations/stripe/config', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  testStripeConnection: () =>
    request<{
      status: string
      mode: string
      message: string
      tested_at: string
    }>('/integrations/stripe/test', {
      method: 'POST',
    }),

  // Issue Trackers & Slack Sync (Gap 3)
  listTrackerConfigs: () => request<IssueTrackerConfig[]>('/integrations/trackers/configs'),
  getTrackerConfig: (provider: string) =>
    request<IssueTrackerConfig>(`/integrations/trackers/configs/${provider}`),
  updateTrackerConfig: (provider: string, data: Partial<IssueTrackerConfig>) =>
    request<IssueTrackerConfig>(`/integrations/trackers/configs/${provider}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  testTrackerConnection: (provider: string) =>
    request<{
      status: string
      mode: string
      message: string
    }>(`/integrations/trackers/test/${provider}`, {
      method: 'POST',
    }),
  listLinkedIssues: (conversationId?: string, ticketId?: string) => {
    const params = new URLSearchParams()
    if (conversationId) params.append('conversation_id', conversationId)
    if (ticketId) params.append('ticket_id', ticketId)
    const qs = params.toString() ? `?${params.toString()}` : ''
    return request<LinkedIssue[]>(`/integrations/trackers/linked${qs}`)
  },
  createLinkedIssue: (data: {
    provider: 'linear' | 'github' | 'jira'
    title: string
    description?: string
    priority?: string
    issue_type?: string
    conversation_id?: string
    ticket_id?: string
    actor_email?: string
    assigned_to?: string
  }) =>
    request<LinkedIssue>('/integrations/trackers/create-issue', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  unlinkIssue: (issueId: string) =>
    request<{ success: boolean; deleted_id: string }>(`/integrations/trackers/linked/${issueId}`, {
      method: 'DELETE',
    }),
  syncIssueStatus: (issueId: string) =>
    request<LinkedIssue>(`/integrations/trackers/linked/${issueId}/sync`, {
      method: 'POST',
    }),
  syncConversationToSlack: (data: {
    conversation_id: string
    channel_name?: string
    note?: string
    author?: string
  }) =>
    request<{
      id: string
      conversation_id: string
      slack_channel_name: string
      slack_thread_ts: string
      thread_url: string
      is_synced: boolean
      last_synced_at: string
      dispatched_note?: string
    }>('/integrations/trackers/slack/sync', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  getSlackSyncStatus: (conversationId: string) =>
    request<SlackSyncStatus>(`/integrations/trackers/slack/status/${conversationId}`),

  // Gap 4: Live Chat Widget Customizer
  getWidgetCustomizer: () => request<WidgetCustomizerConfig>('/widget/customizer'),
  updateWidgetCustomizer: (data: UpdateWidgetCustomizerPayload) =>
    request<WidgetCustomizerConfig>('/widget/customizer', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  resetWidgetCustomizer: () =>
    request<WidgetCustomizerConfig>('/widget/customizer/reset', {
      method: 'POST',
    }),
  getWidgetEmbedCode: (token?: string) =>
    request<{ website_token: string; base_url: string; embed_codes: WidgetEmbedCodes }>(
      `/widget/customizer/embed-code${token ? `?token=${encodeURIComponent(token)}` : ''}`
    ),
// Gap 5: Pre-Chat Intake Forms
  getPreChatConfig: () => request<PreChatFormConfig>('/inboxes/channels/live-chat/prechat'),
  updatePreChatConfig: (data: Partial<PreChatFormConfig>) =>
    request<PreChatFormConfig>('/inboxes/channels/live-chat/prechat', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  resetPreChatConfig: () =>
    request<PreChatFormConfig>('/inboxes/channels/live-chat/prechat/reset', {
      method: 'POST',
    }),
  testPreChatIntake: (answers: Record<string, any>) =>
    request<PreChatTestIntakeResult>('/inboxes/channels/live-chat/prechat/test', {
      method: 'POST',
      body: JSON.stringify({ answers }),
    }),
// Gap 6: Audit Logs & SOC 2 Compliance
  getAuditLogs: (params?: {
    action?: string
    actor?: string
    entity_type?: string
    severity?: string
    compliance_category?: string
    search?: string
    page?: number
    page_size?: number
  }) => {
    const qs = new URLSearchParams()
    if (params?.action) qs.append('action', params.action)
    if (params?.actor) qs.append('actor', params.actor)
    if (params?.entity_type) qs.append('entity_type', params.entity_type)
    if (params?.severity) qs.append('severity', params.severity)
    if (params?.compliance_category) qs.append('compliance_category', params.compliance_category)
    if (params?.search) qs.append('search', params.search)
    if (params?.page) qs.append('page', String(params.page))
    if (params?.page_size) qs.append('page_size', String(params.page_size))
    const qStr = qs.toString() ? `?${qs.toString()}` : ''
    return request<AuditLogsResponse>(`/audit-logs${qStr}`)
  },
  getAuditStats: () => request<AuditStatistics>('/audit-logs/stats'),
  recordAuditEvent: (data: {
    action: string
    entity_type: string
    entity_id?: string
    actor?: string
    actor_email?: string
    actor_name?: string
    actor_role?: string
    severity?: string
    compliance_category?: string
    description?: string
    details?: Record<string, any>
  }) =>
    request<{ success: boolean; id: string; action: string }>('/audit-logs', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
// Gap 7: Granular Role-Based Access Control (RBAC)
  getRoles: () => request<RoleItem[]>('/roles'),
  getPermissionsCatalog: () => request<PermissionCatalogItem[]>('/roles/permissions'),
  getRoleDetail: (roleIdOrName: string) => request<RoleItem>(`/roles/${roleIdOrName}`),
  createRole: (data: {
    name: string
    display_name: string
    description?: string
    permissions: string[]
  }) =>
    request<{ success: boolean; id: string; name: string; display_name: string; permissions: string[] }>(
      '/roles',
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    ),
  updateRole: (
    roleIdOrName: string,
    data: { display_name?: string; description?: string; permissions?: string[] }
  ) =>
    request<{ success: boolean; id: string; name: string; display_name: string; permissions: string[] }>(
      `/roles/${roleIdOrName}`,
      {
        method: 'PUT',
        body: JSON.stringify(data),
      }
    ),
  deleteRole: (roleIdOrName: string) =>
    request<{ success: boolean; deleted_role: string }>(`/roles/${roleIdOrName}`, {
      method: 'DELETE',
    }),
  assignMemberRole: (
    memberId: string,
    data: { role: string; custom_permissions?: string[]; actor?: string }
  ) =>
    request<{ success: boolean; member_id: string; name: string; role: string; custom_permissions: string[] }>(
      `/roles/members/${memberId}/assign`,
      {
        method: 'POST',
        body: JSON.stringify(data),
      }
    ),
  checkMemberPermission: (memberId: string, permission: string) =>
    request<{ member_id: string; permission: string; allowed: boolean }>(
      `/roles/members/${memberId}/check?permission=${encodeURIComponent(permission)}`
    ),
  // Gap 8: Enterprise SSO & SAML 2.0
  getSSOConfig: () =>
    request<{ sso_config: SSOConfigItem }>('/sso/config').then((r) => r.sso_config),
  updateSSOConfig: (data: Partial<SSOConfigItem>) =>
    request<{ sso_config: SSOConfigItem }>('/sso/config', {
      method: 'PUT',
      body: JSON.stringify(data),
    }).then((r) => r.sso_config),
  testSSOConnection: (sampleXml?: string) =>
    request<SSOTestResult>('/sso/test', {
      method: 'POST',
      body: JSON.stringify({ sample_xml: sampleXml }),
    }),
  checkSSOEmail: (email: string) =>
    request<{
      sso_required: boolean
      is_enabled: boolean
      provider_type: string
      domain: string
      redirect_url: string | null
      allow_unlinked_passwords: boolean
    }>('/sso/check', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),
  // Gap 9: Bi-Directional Real-Time Auto-Translation
  getTranslationLanguages: () =>
    request<{ languages: Array<{ code: string; name: string; native: string; dir: string }>; total: number }>('/translation/languages'),
  getTranslationSettings: () =>
    request<{ settings: Record<string, any> }>('/translation/settings').then((r) => r.settings),
  updateTranslationSettings: (data: Record<string, any>) =>
    request<{ settings: Record<string, any> }>('/translation/settings', {
      method: 'PUT',
      body: JSON.stringify(data),
    }).then((r) => r.settings),
  detectLanguage: (text: string) =>
    request<{ language: string; name: string; confidence: number; script: string }>('/translation/detect', {
      method: 'POST',
      body: JSON.stringify({ text }),
    }),
  translateMessage: (messageId: string, targetLanguage = 'en') =>
    request<{ success: boolean; message_id: string; translation: TranslationResult }>(
      `/translation/messages/${messageId}/translate?target_language=${encodeURIComponent(targetLanguage)}`,
      { method: 'POST' }
    ),
  prepareOutboundTranslation: (conversationId: string, text: string, targetLanguage?: string) =>
    request<{
      conversation_id: string
      customer_language: string
      original_text: string
      translated_text: string
      source_language: string
      target_language: string
      confidence: number
    }>(`/translation/conversations/${conversationId}/prepare-outbound`, {
      method: 'POST',
      body: JSON.stringify({ text, target_language: targetLanguage }),
    }),
  // Gap 10: Dynamic Custom Attributes & CRM Schema Builder
  getCustomAttributes: (attributeModel?: string) =>
    request<CustomAttributeDefinitionItem[]>(
      attributeModel ? `/custom-attributes?attribute_model=${encodeURIComponent(attributeModel)}` : '/custom-attributes'
    ),
  createCustomAttribute: (data: {
    name: string
    display_name: string
    description?: string
    attribute_model: 'contact_attribute' | 'conversation_attribute'
    attribute_type: 'text' | 'number' | 'currency' | 'boolean' | 'list' | 'date' | 'link'
    options?: string[]
    default_value?: string
    is_required?: boolean
  }) =>
    request<CustomAttributeDefinitionItem>('/custom-attributes', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateCustomAttribute: (
    idOrName: string,
    data: {
      display_name?: string
      description?: string
      options?: string[]
      default_value?: string
      is_required?: boolean
    }
  ) =>
    request<CustomAttributeDefinitionItem>(`/custom-attributes/${idOrName}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  deleteCustomAttribute: (idOrName: string) =>
    request<{ success: boolean; deleted: string }>(`/custom-attributes/${idOrName}`, {
      method: 'DELETE',
    }),
  updateContactAttributes: (contactId: string, attributes: Record<string, any>) =>
    request<{ success: boolean; contact_id: string; custom_fields: Record<string, any> }>(
      `/custom-attributes/contacts/${contactId}/values`,
      {
        method: 'POST',
        body: JSON.stringify({ attributes }),
      }
    ),
  updateConversationAttributes: (conversationId: string, attributes: Record<string, any>) =>
    request<{ success: boolean; conversation_id: string; custom_attributes: Record<string, any> }>(
      `/custom-attributes/conversations/${conversationId}/values`,
      {
        method: 'POST',
        body: JSON.stringify({ attributes }),
      }
    ),
}