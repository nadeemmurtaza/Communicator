const BASE = '/api/v1'

/**
 * Every request carries the signed-in session token and the tenant the
 * person has selected (set by AuthContext on login / org switch), so the
 * backend scopes data to the org actually showing in the UI instead of a
 * hardcoded default -- switching organizations must change what every
 * subsequent request can see.
 */
function authHeaders(): Record<string, string> {
  const headers: Record<string, string> = {}
  try {
    const session = localStorage.getItem('communicator.session')
    if (session) {
      const { token } = JSON.parse(session)
      if (token) headers['Authorization'] = `Bearer ${token}`
    }
    const org = localStorage.getItem('communicator.organization')
    if (org) {
      const { tenantKey } = JSON.parse(org)
      if (tenantKey) headers['X-Tenant-Key'] = tenantKey
    }
    const brand = localStorage.getItem('communicator.brand')
    if (brand) {
      const { id: brandId } = JSON.parse(brand)
      if (brandId) headers['X-Brand-ID'] = brandId
    }
  } catch {
    // corrupted storage -- fall back to unauthenticated/default-tenant request
  }
  return headers
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...init,
    headers: { 'Content-Type': 'application/json', ...authHeaders(), ...(init?.headers ?? {}) },
  })
  if (!res.ok) {
    const detail = await res.text()
    let cleanMessage = ''
    try {
      const json = JSON.parse(detail)
      cleanMessage = json.detail || json.message || json.error || detail
      if (typeof cleanMessage === 'object') {
        cleanMessage = JSON.stringify(cleanMessage)
      }
    } catch {
      if (detail.includes('<!DOCTYPE') || detail.includes('<html') || detail.includes('Cloudflare')) {
        cleanMessage = `Service temporarily unreachable or gateway error (${res.status} ${res.statusText}).`
      } else {
        cleanMessage = detail.trim().slice(0, 300)
      }
    }
    throw new Error(`${res.status} ${res.statusText}: ${cleanMessage}`)
  }
  if (res.status === 204) return undefined as T
  return res.json() as Promise<T>
}

export interface OrganizationItem {
  id: string
  tenantKey: string
  name: string
  createdAt: string
}

export interface BrandItem {
  id: string
  organizationId: string
  name: string
  slug: string
  description?: string
  logoUrl?: string
  tagline?: string
  createdAt?: string
}

export interface ValidationResult {
  passed: boolean
  relevanceScore: number
  groundingScore: number
  reason: string | null
  issues: string[]
  validated: boolean
  latencyMs: number
}

export interface Draft {
  id: string
  conversationId: string
  body: string
  confidence: number
  citations: string[]
  status: 'pending_review' | 'auto_approved' | 'approved' | 'rejected' | 'sent'
  createdAt: string
  validation?: ValidationResult
}

export interface Conversation {
  id: string
  contactId: string
  contactName?: string
  customerName?: string
  leadName?: string
  contactPhone?: string
  phone?: string
  channel?: string
  state: string
  resolutionType?: string | null
  broadcastId?: string | null
  broadcastName?: string | null
  humanTakeover: boolean
  assignedToId: string | null
  assignedToName?: string | null
  teamId?: string | null
  teamName?: string | null
  snoozedUntil?: string | null
  priority?: string | null
  csatScore?: number | null
  csatComment?: string | null
  customAttributes?: Record<string, any>
  updatedAt: string
}

export interface MessageFeedback {
  id: string
  rating: number        // 1 = helpful, -1 = not helpful
  comment?: string | null
  authorType: string    // "customer" | "agent"
  createdAt: string
}

// Monitoring & Alerting (Phase 8)

export interface MonitoringMetric {
  id: string
  metricName: string
  metricValue: number
  metricType: string
  labels: string
  collectedAt: string
  createdAt: string
}

export interface MetricSummary {
  metricName: string
  avgValue: number
  minValue: number
  maxValue: number
  latestValue: number
  sampleCount: number
  windowStart: string
  windowEnd: string
}

export interface ComponentHealth {
  name: string
  status: 'healthy' | 'degraded' | 'unhealthy'
  details: string | null
  latencyMs: number
}

export interface HealthCheckResult {
  overallStatus: string
  components: ComponentHealth[]
  checkedAt: string
}

export interface NegativeFeedbackItem {
  messageId: string
  conversationId: string
  rating: number
  comment: string | null
  authorType: string
  createdAt: string
}

export interface FeedbackSummary {
  totalRatings: number
  positiveRatings: number
  negativeRatings: number
  satisfactionRate: number
  recentNegative: NegativeFeedbackItem[]
}

export interface MonitoringDashboard {
  health: HealthCheckResult
  activeAlerts: number
  pendingAlerts: number
  metricsSummary: MetricSummary[]
  feedbackSummary: FeedbackSummary
}

export interface MonitoringAlert {
  id: string
  ruleId: string
  alertName: string
  severity: string
  metricName: string
  metricValue: number
  threshold: number
  condition: string
  message: string
  acknowledged: number
  acknowledgedBy: string | null
  acknowledgedAt: string | null
  resolved: number
  resolvedAt: string | null
  createdAt: string
}

export interface MonitoringAlertRule {
  id: string
  name: string
  description: string
  metricName: string
  condition: string
  threshold: number
  windowMinutes: number
  severity: string
  enabled: number
  notificationWebhook: string | null
  cooldownMinutes: number
  createdAt: string
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
  senderName?: string
  providerMessageId?: string
  providerMetadata?: Record<string, any>
  translation?: TranslationResult
  feedback?: MessageFeedback | null
  // Media & Voice fields (Priority 4)
  mediaType?: string | null       // "image" | "video" | "audio" | "document" | "sticker"
  mediaUrl?: string | null        // URL to the media resource
  mediaFilename?: string | null   // Original filename
  mediaMimetype?: string | null   // MIME type (e.g. image/jpeg)
  voiceDuration?: number | null   // Voice note duration in seconds
  transcriptionText?: string | null // Transcribed text for voice messages
}

export interface WhatsApp24hWindow {
  isOpen: boolean
  isClosingSoon: boolean
  expiresAt: string | null
  remainingSeconds: number
  remainingMinutes: number
}

export interface ConversationDetail {
  id: string
  state: string
  humanTakeover: boolean
  assignedToId: string | null
  assignedToName?: string | null
  teamId?: string | null
  teamName?: string | null
  contactId?: string
  contactName?: string
  customerName?: string
  leadName?: string
  contactPhone?: string
  phone?: string
  channel?: string
  language?: string
  priority?: string | null
  snoozedUntil?: string | null
  customAttributes?: Record<string, any>
  resolutionType?: string | null
  csatScore?: number | null
  csatComment?: string | null
  slaStatus?: string
  sla?: SLAData
  whatsapp24hWindow?: WhatsApp24hWindow | null
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
  customFields?: Record<string, unknown>
  tags: string[]
  createdAt?: string
  optInStatus?: string
  qualityRating?: string
  role?: string
}

export interface ContactProfileData {
  id: string
  value: string
  personName: string | null
  channel: string
  language: string
  createdAt: string
  updatedAt?: string
  tags: string[]
  customFields?: Record<string, any>
  meta?: {
    waId?: string
    isWhatsAppRegistered?: boolean
    qualityRating?: string
    optInStatus?: string
    optInTimestamp?: string
    verified?: boolean
    syncedAt?: string
    nameStatus?: string
    codeVerificationStatus?: string
  }
  conversation?: {
    id?: string
    state?: string
    priority?: string
    resolutionType?: string | null
    totalMessages?: number
    inboundCount?: number
    outboundCount?: number
    lastInboundText?: string
    lastInboundAt?: string
    lastOutboundText?: string
    lastOutboundAt?: string
    lastOutboundStatus?: string
    csatScore?: number | null
  }
  lead?: {
    id?: string
    name?: string
    institutionId?: string
    qualificationRationale?: string
    institutionType?: string
    city?: string
  }
  memories?: Array<{
    id: string
    fact: string
    category: string
    createdAt: string
    confidence?: number
    notedAt?: string
  }>
  recentMessages?: Array<{
    id: string
    direction: 'inbound' | 'outbound'
    body: string
    sentAt: string
    status?: string
    senderName?: string
  }>
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
  templateName: string | null
  templateLanguage: string | null
  bodyPreview?: string | null
  templateParams?: Record<string, string> | string | null
  tagName: string | null
  status: 'draft' | 'scheduled' | 'sending' | 'completed' | 'failed'
  scheduledAt: string | null
  createdAt: string
  audienceCount?: number | null
  sentCount: number
  deliveredCount: number
  readCount: number
  failedCount: number
  skippedCount?: number
  tenantKey?: string | null
  warnings?: string[]
}

/** One recipient decision returned by `/broadcasts/{id}/dispatch`. */
export interface DispatchRecipient {
  contactId: string
  messageId?: string
  status: 'sent' | 'failed' | 'not_sent'
  reason?: string | null
  provider?: string
  httpStatus?: number | null
}

/** What the dispatcher actually did — every count comes from a real send attempt. */
export interface DispatchOutcome extends Broadcast {
  totalRecipients?: number
  templateUsed?: boolean
  recipients?: DispatchRecipient[]
  error?: string
}

export interface BroadcastRecipient {
  id: string
  contactId: string
  messageId?: string | null
  contactName?: string | null
  contactPhone?: string | null
  status: string
  error: string | null
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
  warnings?: string[]
  /** Recomputed from the recipient ledger, not from the stored counters. */
  campaignId?: string
  totalRecipients?: number
}

export interface AudiencePreview {
  tags: string[]
  totalSelected: number
  eligibleCount: number
  optedInCount: number
  optedOutCount: number
  unknownConsentCount: number
  /** Null unless the daily cap was read from Meta's `messaging_limit_tier`. */
  dailyLimit: number | null
  dailyLimitSource: string | null
  tenantKey?: string
  sampleContacts: Array<{
    id: string
    /** Null when the contact has no stored name — the UI must not invent one. */
    name: string | null
    phone: string
    channel?: string
    consent?: string | null
  }>
  warnings?: string[]
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

export interface ChannelHealth {
  channel: string
  isConnected: boolean
  status: string
  statusLabel: string
  warning?: string | null
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
    tier?: string
    verificationStatus?: string
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

export interface SharedBusinessProfile {
  businessName: string
  verifiedName: string
  displayPhone: string
  isVerified: boolean
  verificationStatus: string
  qualityRating: string
  messagingLimit: string
  tier: string
  spamFlags: string
  description: string
  about: string
  email: string
  address: string
  vertical: string
  websites: string[]
  profilePictureUrl: string
  businessHours: {
    enabled: boolean
    start: number
    end: number
    timezone: string
    awayMessage: string
  }
  autoReplyEnabled: boolean
  agentGuidelines: string
  metaIntegration: {
    connected: boolean
    phoneNumberId: string
    wabaId: string
    verifiedAt: string
  }
}

export interface WhatsAppTemplate {
  id: string
  name: string
  category: string
  language: string
  status: string
  body: string
  bodyText?: string
  components: any[] | any
  lastUpdated?: string
  lastSyncedAt?: string
  // Enhanced fields
  headerText?: string | null
  footerText?: string | null
  buttons?: any[]
  variables?: string[]
  qualityScore?: number | null
  qualityRating?: string | null
  sentCount?: number
  lastSentAt?: string | null
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
  /** NULL when no author was stored -- the backend never substitutes a stand-in. */
  author_name?: string | null
  activity_type?: string | null
  message?: string | null
  metadata?: Record<string, any>
  created_at?: string | null
}

export interface ChildTicketSummary {
  id: string
  ticket_number?: string | null
  title?: string | null
  status?: string | null
  priority?: string | null
  department?: string | null
  contact_id?: string | null
  /** NULL when the ticket has no contact_id, so nobody can be messaged for it. */
  contact_name?: string | null
  conversation_id?: string | null
  created_at?: string | null
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
  /** Live count of not-yet-resolved urgent tickets; absent means never counted. */
  urgent_tickets?: number
  department_breakdown: Record<string, number>
  impacted_customers_count: number
  /** NULL until at least one ticket has both created_at and resolved_at. */
  avg_resolution_time_hours?: number | null
  total_activities?: number
}

/**
 * One per-recipient row of a tracker broadcast, exactly as
 * `broadcast_tracker_update` in backend-rs/src/routes/tickets.rs builds it.
 * The keys present depend on which branch produced the row, so every field is
 * optional: a row with no `provider`/`httpStatus` means no provider was ever
 * contacted for that recipient, which is information, not a zero.
 */
export interface BroadcastDelivery {
  /** 'sent' | 'failed' | 'not_sent' | 'error', or missing if unreadable. */
  status?: string | null
  ticketId?: string | null
  ticket_id?: string | null
  conversationId?: string | null
  conversation_id?: string | null
  messageId?: string | null
  message_id?: string | null
  channel?: string | null
  contactId?: string | null
  provider?: string | null
  httpStatus?: number | null
  providerMessageId?: string | null
  simulated?: boolean
  detail?: string | null
  note?: string | null
}

/**
 * Mirror of the JSON the Rust handler returns for POST /tickets/{id}/broadcast.
 * The handler emits both snake_case and camelCase for every scalar, so both are
 * declared here; `messages_sent` counts only sends a provider accepted with a
 * real 2xx, and `messages_not_sent` counts recipients nothing left the process
 * for (no conversation, no addressable contact value, consent/business gates).
 */
export interface BroadcastTrackerUpdateResult {
  tracker_id: string
  trackerId: string
  /** NULL when the tracker row stores no ticket_number. */
  tracker_number: string | null
  trackerNumber: string | null
  impacted_tickets: number
  impactedTickets: number
  messages_sent: number
  messagesSent: number
  messages_failed: number
  messagesFailed: number
  messages_not_sent: number
  messagesNotSent: number
  deliveries: BroadcastDelivery[]
  customer_message_requested: boolean
  customerMessageRequested: boolean
  resolve_all_requested: boolean
  resolveAllRequested: boolean
  resolved_tickets: number
  resolvedTickets: number
  already_resolved: number
  alreadyResolved: number
  resolve_failed: number
  resolveFailed: number
  customers_notified: boolean
  customersNotified: boolean
  tracker_resolved: boolean
  trackerResolved: boolean
  tracker_resolve_error: string | null
  trackerResolveError: string | null
  activity_errors: string[]
  activityErrors: string[]
  note: string
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
  provider?: 'linear' | 'github' | 'jira' | null
  external_issue_id?: string | null
  issue_key?: string | null
  /** NULL when the tracker returned no URL; a missing URL is not a link. */
  issue_url?: string | null
  title?: string | null
  status?: string | null
  priority?: string | null
  assigned_to?: string | null
  /** NULL when the create request carried no signed-in actor email. */
  created_by_email?: string | null
  sync_status?: string | null
  metadata_json?: Record<string, any> | null
  created_at?: string | null
  updated_at?: string | null
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

export interface DiscordChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    callbackUrlNote: string
    interactionSecret: string
  }
  botProfile: {
    botToken: string | null
    clientId: string
    botUsername: string
    botDiscriminator: string
    botAvatar: string
    botId: string
    source: string
  }
  commands: Array<{ id: string; name: string; description: string; isActive: boolean }>
  settings: {
    autoRespond: boolean
    greetingMessage: string
    awayMessage: string
    allowedChannels: string
    blockedWords: string
    logEvents: boolean
  }
  health: {
    isConnected: boolean
    configured: boolean
    inboxActive: boolean
    botId: string
    status: string
    statusLabel: string
    missingCredentials: string[]
    connectionError: string
  }
  warnings: string[]
}

export interface DiscordUpdatePayload {
  name?: string
  bot_token?: string
  client_id?: string
  application_id?: string
  interaction_secret?: string
  settings?: {
    autoRespond?: boolean
    greetingMessage?: string
    awayMessage?: string
  }
}

export interface SlackChannelData {
  inbox: Inbox
  webhookConfig: {
    callbackUrl: string
    callbackUrlNote: string
    signingSecret: string
  }
  botProfile: {
    botToken: string | null
    appToken: string
    appId: string
    botUserId: string
    source: string
  }
  commands: Array<{ id: string; name: string; description: string; isActive: boolean }>
  settings: {
    autoRespond: boolean
    greetingMessage: string
    awayMessage: string
    allowedChannels: string
    blockedWords: string
    logEvents: boolean
  }
  health: {
    isConnected: boolean
    configured: boolean
    inboxActive: boolean
    botUserId: string
    status: string
    statusLabel: string
    missingCredentials: string[]
    connectionError: string
  }
  warnings: string[]
}

export interface SlackUpdatePayload {
  name?: string
  bot_token?: string
  app_token?: string
  signing_secret?: string
  app_id?: string
  settings?: {
    autoRespond?: boolean
    greetingMessage?: string
    awayMessage?: string
  }
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
  category: string
  volume: number
  /** Always from the `whatsapp_pricing_rates` row; the endpoint 503s when none exists. */
  ratePerConversation: number
  ratePerMessage: number | null
  estimatedCost: number
  /** Null when the stored rate is not denominated in USD. */
  estimatedCostUsd: number | null
  currency: string
  rateSource: string
  rateUpdatedAt: string | null
  /** Never computed — kept so the UI cannot invent a multiplier. */
  estimatedRoiUsd: null
  roiNote: string
  warnings?: string[]
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
  createService: (data: { name: string; summary: string; target_institution_types?: string[] }) =>
    request<Service>('/services', { method: 'POST', body: JSON.stringify(data) }),

  listSequences: () => request<Sequence[]>('/sequences'),
  createSequence: (data: { name: string; description?: string; is_active?: boolean }) =>
    request<Sequence>('/sequences', { method: 'POST', body: JSON.stringify(data) }),
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

  // Organization Profile & BYOK
  getOrganizationProfile: () => request<any>('/organization/profile'),
  updateOrganizationProfile: (data: any) =>
    request<any>('/organization/profile', { method: 'PUT', body: JSON.stringify(data) }),
  getOrganizationSettings: () => request<any>('/organization/settings'),
  updateOrganizationSettings: (data: any) =>
    request<any>('/organization/settings', { method: 'PUT', body: JSON.stringify(data) }),
  getOrganizationStats: () => request<any>('/organization/stats'),
  verifyChannelConnection: (channel: string) =>
    request<{ connected: boolean; error?: string }>(`/organization/channels/${channel}/verify`, {
      method: 'POST',
    }),

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
  addContactMemory: (contactId: string, fact: string, category?: string) =>
    request<ContactMemory>(`/contacts/${contactId}/memories`, {
      method: 'POST',
      body: JSON.stringify({ fact, category }),
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
  getContactProfile: (id: string) => request<ContactProfileData>(`/contacts/${id}`),
  toggleContactConsent: (id: string) =>
    request<{ status: string; optInStatus: string; optInTimestamp: string }>(`/contacts/${id}/toggle-consent`, {
      method: 'POST',
    }),
  syncContactMeta: (id: string) =>
    request<{ status: string; syncedAt: string }>(`/contacts/${id}/sync-meta`, {
      method: 'POST',
    }),
  syncMetaContacts: () =>
    request<{ status: string; totalWhatsAppContacts: number; updatedContacts: number; message: string }>(
      '/contacts/sync-meta',
      { method: 'POST' }
    ),
  getOrCreateContactConversation: (id: string) =>
    request<{ id: string; state: string }>(`/contacts/${id}/conversation`, {
      method: 'POST',
    }),
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
  // A plain <a href> can't send the X-Tenant-Key/Authorization headers the
  // export route now requires -- fetch it with those headers and hand the
  // browser a Blob to download instead, so a non-default organization's
  // export doesn't silently fall back to the default tenant's contacts.
  downloadContactsExport: async (
    params?: { channel?: string; tag?: string; search?: string; segment_id?: string },
    filename = 'contacts_export.csv'
  ) => {
    const url = `${BASE}/contacts/export${(() => {
      const q = new URLSearchParams()
      if (params?.channel) q.set('channel', params.channel)
      if (params?.tag) q.set('tag', params.tag)
      if (params?.search) q.set('search', params.search)
      if (params?.segment_id) q.set('segment_id', params.segment_id)
      const qs = q.toString()
      return qs ? `?${qs}` : ''
    })()}`
    const res = await fetch(url, { headers: authHeaders() })
    if (!res.ok) throw new Error('Export failed')
    const blob = await res.blob()
    const blobUrl = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = blobUrl
    a.download = filename
    document.body.appendChild(a)
    a.click()
    a.remove()
    URL.revokeObjectURL(blobUrl)
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
    template_name?: string
    template_language?: string
    body_preview: string
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
    request<DispatchOutcome>(`/broadcasts/${id}/dispatch`, { method: 'POST' }),
  processScheduledCampaigns: () =>
    request<{
      processed: number
      due: number
      tenantKey: string
      campaigns: Array<{
        id: string
        status: string
        sentCount?: number
        failedCount?: number
        skippedCount?: number
        error?: string | null
        httpStatus?: number
      }>
    }>('/broadcasts/process-scheduled', {
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

  // Channels & Inboxes API (Starting with WhatsApp)
  // Shared Global Business Profile
  getBusinessProfile: () => request<SharedBusinessProfile>('/business-profile'),
  updateBusinessProfile: (data: any) =>
    request<SharedBusinessProfile>('/business-profile', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  syncBusinessProfileMeta: () =>
    request<SharedBusinessProfile>('/business-profile/sync-meta', {
      method: 'POST',
    }),

  listInboxes: () => request<Inbox[]>('/inboxes'),
  getChannelsHealth: () => request<ChannelHealth[]>('/channels/health'),
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

  // Channel 6b: Discord
  getDiscordChannel: () => request<DiscordChannelData>('/inboxes/channels/discord'),
  updateDiscordChannel: (data: DiscordUpdatePayload) =>
    request<DiscordChannelData>('/inboxes/channels/discord', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifyDiscordConnection: () =>
    request<{ verified: boolean; status: string; bot?: object; message: string }>('/inboxes/channels/discord/verify', {
      method: 'POST',
    }),
  sendDiscordTestMessage: (data: { channelId: string; message?: string }) =>
    request<{ sent: boolean; status: string; messageId?: string }>('/inboxes/channels/discord/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listDiscordEvents: () => request<SocialEvent[]>('/inboxes/channels/discord/events'),

  // Channel 6c: Slack
  getSlackChannel: () => request<SlackChannelData>('/inboxes/channels/slack'),
  updateSlackChannel: (data: SlackUpdatePayload) =>
    request<SlackChannelData>('/inboxes/channels/slack', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  verifySlackConnection: () =>
    request<{ verified: boolean; status: string; bot?: object; message: string }>('/inboxes/channels/slack/verify', {
      method: 'POST',
    }),
  sendSlackTestMessage: (data: { channelId: string; message?: string }) =>
    request<{ sent: boolean; status: string; timestamp?: string; channelId?: string }>('/inboxes/channels/slack/test-message', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listSlackEvents: () => request<SocialEvent[]>('/inboxes/channels/slack/events'),

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

  // Message Feedback — thumbs up / down on AI messages
  submitMessageFeedback: (
    messageId: string,
    rating: number,
    comment?: string,
    authorType?: string,
    authorId?: string,
  ) =>
    request<any>(`/messages/${messageId}/feedback`, {
      method: 'POST',
      body: JSON.stringify({ rating, comment, author_type: authorType, author_id: authorId }),
    }),
  getFeedbackSummary: () =>
    request<{
      totalFeedback: number
      positiveRatings: number
      negativeRatings: number
      satisfactionRate: number
      recentNegativeFeedback: Array<{
        messageId: string
        body: string
        comment?: string | null
        createdAt: string
      }>
    }>('/feedback/summary'),

  // Monitoring & Alerting (Phase 8)
  getMonitoringDashboard: () => request<MonitoringDashboard>('/monitoring/dashboard'),
  getMonitoringHealth: () => request<HealthCheckResult>('/monitoring/health'),
  getMonitoringMetrics: (name?: string, windowHours?: number) => {
    const params = new URLSearchParams()
    if (name) params.set('name', name)
    if (windowHours) params.set('window_hours', String(windowHours))
    return request<{ metrics: MonitoringMetric[]; window_hours: number; count: number }>(
      `/monitoring/metrics?${params.toString()}`
    )
  },
  getMonitoringMetricsSummary: (windowHours?: number) => {
    const params = new URLSearchParams()
    if (windowHours) params.set('window_hours', String(windowHours))
    return request<{ summaries: MetricSummary[]; window_hours: number; count: number }>(
      `/monitoring/metrics/summary?${params.toString()}`
    )
  },
  collectMonitoringMetrics: () =>
    request<{ collected: number; alerts_fired: number; collected_at: string }>(
      '/monitoring/metrics/collect',
      { method: 'POST' }
    ),
  getMonitoringAlerts: (severity?: string, acknowledged?: boolean, resolved?: boolean) => {
    const params = new URLSearchParams()
    if (severity) params.set('severity', severity)
    if (acknowledged !== undefined) params.set('acknowledged', String(acknowledged))
    if (resolved !== undefined) params.set('resolved', String(resolved))
    return request<{ alerts: MonitoringAlert[]; count: number }>(
      `/monitoring/alerts?${params.toString()}`
    )
  },
  acknowledgeMonitoringAlert: (alertId: string, acknowledgedBy?: string) =>
    request<any>(`/monitoring/alerts/${alertId}/acknowledge`, {
      method: 'POST',
      body: JSON.stringify({ acknowledged_by: acknowledgedBy }),
    }),
  resolveMonitoringAlert: (alertId: string) =>
    request<any>(`/monitoring/alerts/${alertId}/resolve`, {
      method: 'POST',
    }),
  getMonitoringAlertRules: () =>
    request<{ rules: MonitoringAlertRule[]; count: number }>('/monitoring/alert-rules'),
  createMonitoringAlertRule: (payload: {
    name: string
    description?: string
    metric_name: string
    condition: string
    threshold: number
    window_minutes?: number
    severity?: string
    notification_webhook?: string
    cooldown_minutes?: number
  }) =>
    request<any>('/monitoring/alert-rules', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  updateMonitoringAlertRule: (
    ruleId: string,
    payload: {
      name?: string
      description?: string
      metric_name?: string
      condition?: string
      threshold?: number
      window_minutes?: number
      severity?: string
      enabled?: number
      notification_webhook?: string
      cooldown_minutes?: number
    }
  ) =>
    request<any>(`/monitoring/alert-rules/${ruleId}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),
  deleteMonitoringAlertRule: (ruleId: string) =>
    request<any>(`/monitoring/alert-rules/${ruleId}`, {
      method: 'DELETE',
    }),
  evaluateMonitoringAlerts: () =>
    request<{ evaluated: boolean; alerts_fired: number; evaluated_at: string }>(
      '/monitoring/alerts/evaluate',
      { method: 'POST' }
    ),

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
    request<BroadcastTrackerUpdateResult>(`/tickets/${trackerId}/broadcast`, {
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
  // Three-Layer Access Control (Phase 61)
  getAccessGrants: (tenantId?: string) =>
    request<any[]>(tenantId ? `/access/grants?tenant_id=${tenantId}` : '/access/grants'),
  getDepartments: (tenantId?: string) =>
    request<any[]>(tenantId ? `/departments/${tenantId}` : '/departments/tenant'),
  getInvitations: (tenantId?: string) =>
    request<any[]>(tenantId ? `/invitations/${tenantId}` : '/invitations/tenant'),
  getMyPermissions: () =>
    request<any>('/access/my-permissions'),
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
  updateContactAttributeValues: (contactId: string, attributes: Record<string, any>) =>
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
  // Scoped server-side to the signed-in account (Authorization header via
  // authHeaders()) -- account_id is no longer accepted as a client-supplied
  // parameter, so it's not sent here.
  listOrganizations: () => request<OrganizationItem[]>('/organizations'),
  createOrganization: (name: string) =>
    request<OrganizationItem>('/organizations', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),
  // Brand hierarchy
  listBrands: (organizationId?: string) =>
    request<{ brands: BrandItem[] }>(organizationId ? `/brands?organizationId=${organizationId}` : '/brands').then(
      (r) => r.brands
    ),
  getBrand: (id: string) => request<BrandItem>(`/brands/${id}`),
  createBrand: (data: {
    organizationId: string
    name: string
    slug?: string
    description?: string
    logoUrl?: string
    tagline?: string
  }) => request<BrandItem>('/brands', { method: 'POST', body: JSON.stringify(data) }),
  updateBrand: (id: string, data: Record<string, any>) =>
    request<BrandItem>(`/brands/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  deleteBrand: (id: string) => request<{ deleted: boolean }>(`/brands/${id}`, { method: 'DELETE' }),
  getBrandProfile: (id: string) =>
    request<{ brand: any; organization: any; aiConfig: any; knowledgeCount: number }>(`/brands/${id}/profile`),
  // Brand credentials (BYOK)
  listBrandCredentials: (brandId: string) =>
    request<{ credentials: Array<{ id: string; channelType: string; isActive: boolean }> }>(
      `/brands/${brandId}/credentials`
    ).then((r) => r.credentials),
  upsertBrandCredentials: (brandId: string, channel: string, creds: Record<string, string>) =>
    request<{ id: string; channelType: string }>(`/brands/${brandId}/credentials/${channel}`, {
      method: 'PUT',
      body: JSON.stringify(creds),
    }),
  deleteBrandCredentials: (brandId: string, channel: string) =>
    request<{ deleted: boolean }>(`/brands/${brandId}/credentials/${channel}`, { method: 'DELETE' }),
  verifyBrandCredentials: (brandId: string, channel: string) =>
    request<{ connected: boolean; error?: string }>(`/brands/${brandId}/credentials/${channel}/verify`, {
      method: 'POST',
    }),
  // Brand AI config
  getBrandAIConfig: (brandId: string) => request<any>(`/brands/${brandId}/ai-config`),
  updateBrandAIConfig: (brandId: string, data: Record<string, any>) =>
    request<any>(`/brands/${brandId}/ai-config`, { method: 'PUT', body: JSON.stringify(data) }),
  // Brand knowledge base
  listBrandKnowledge: (brandId: string, q?: string, topic?: string) => {
    const params = new URLSearchParams()
    if (q) params.set('q', q)
    if (topic) params.set('topic', topic)
    const qs = params.toString()
    return request<{ entries: any[] }>(`/brands/${brandId}/knowledge${qs ? '?' + qs : ''}`).then((r) => r.entries)
  },
  createBrandKnowledge: (brandId: string, data: { title: string; content: string; topic?: string; tags?: string }) =>
    request<any>(`/brands/${brandId}/knowledge`, { method: 'POST', body: JSON.stringify(data) }),
  updateBrandKnowledge: (brandId: string, entryId: string, data: Record<string, any>) =>
    request<any>(`/brands/${brandId}/knowledge/${entryId}`, { method: 'PUT', body: JSON.stringify(data) }),
  deleteBrandKnowledge: (brandId: string, entryId: string) =>
    request<{ deleted: boolean }>(`/brands/${brandId}/knowledge/${entryId}`, { method: 'DELETE' }),
  // WhatsApp Channel
  getWhatsAppHealth: (brandId: string) => request<any>(`/brands/${brandId}/whatsapp/health`),
  listWhatsAppTemplates: (brandId?: string) =>
    brandId ? request<any[]>(`/brands/${brandId}/whatsapp/templates`) : request<any[]>('/inboxes/channels/whatsapp/templates'),
  syncWhatsAppTemplates: (brandId: string) =>
    request<{ templatesSynced: number; profileSynced: boolean; syncedAt: string }>(
      `/brands/${brandId}/whatsapp/sync-templates`, { method: 'POST' }
    ),
  syncWhatsAppProfile: (brandId: string) =>
    request<{ synced: boolean; verifiedName?: string; messagingLimitTier?: string; qualityRating?: string; syncedAt: string }>(
      `/brands/${brandId}/whatsapp/sync-profile`, { method: 'POST' }
    ),
  listWhatsAppCustomers: (brandId: string) => request<any[]>(`/brands/${brandId}/whatsapp/customers`),
  sendWhatsAppTest: (brandId: string, data: { to: string; messageType?: string; body?: string; templateName?: string; language?: string }) =>
    request<{ success: boolean; messageId: string; messageType: string }>(
      `/brands/${brandId}/whatsapp/send-test`, { method: 'POST', body: JSON.stringify(data) }
    ),
  listWhatsAppRules: (brandId: string) => request<any[]>(`/brands/${brandId}/whatsapp/automation-rules`),
  createWhatsAppRule: (brandId: string, data: { name: string; triggerType: string; triggerValue?: string; actionType: string; actionConfig?: string; priority?: number }) =>
    request<{ id: string; createdAt: string }>(
      `/brands/${brandId}/whatsapp/automation-rules`, { method: 'POST', body: JSON.stringify(data) }
    ),
  updateWhatsAppRule: (brandId: string, ruleId: string, data: { name: string; triggerType: string; triggerValue?: string; actionType: string; actionConfig?: string; priority?: number }) =>
    request<{ id: string; updatedAt: string }>(
      `/brands/${brandId}/whatsapp/automation-rules/${ruleId}`, { method: 'PUT', body: JSON.stringify(data) }
    ),
  deleteWhatsAppRule: (brandId: string, ruleId: string) =>
    request<{ deleted: boolean }>(`/brands/${brandId}/whatsapp/automation-rules/${ruleId}`, { method: 'DELETE' }),
  runWhatsAppNudges: (brandId: string) =>
    request<{ nudgesSent: number; runAt: string }>(`/brands/${brandId}/whatsapp/nudges`, { method: 'POST' }),
  sendWhatsAppTemplate: (brandId: string, data: { contactId: string; templateName: string; language?: string; components?: any }) =>
    request<{ sent: boolean; providerMessageId: string }>(
      `/brands/${brandId}/whatsapp/send-template`, { method: 'POST', body: JSON.stringify({
        contact_id: data.contactId,
        template_name: data.templateName,
        language: data.language,
        components: data.components,
      }) }
    ),
  sendWhatsAppInteractive: (brandId: string, data: { contactId: string; type: string; bodyText: string; headerText?: string; footerText?: string; buttons?: any[]; listButtonTitle?: string; listSections?: any[]; mediaUrl?: string; mediaType?: string; mediaFilename?: string; ctaDisplayText?: string; ctaUrl?: string }) =>
    request<{ sent: boolean; providerMessageId: string }>(
      `/brands/${brandId}/whatsapp/send-interactive`, { method: 'POST', body: JSON.stringify({
        contact_id: data.contactId,
        type: data.type,
        bodyText: data.bodyText,
        headerText: data.headerText,
        footerText: data.footerText,
        buttons: data.buttons,
        listButtonTitle: data.listButtonTitle,
        listSections: data.listSections,
        mediaUrl: data.mediaUrl,
        mediaType: data.mediaType,
        mediaFilename: data.mediaFilename,
        ctaDisplayText: data.ctaDisplayText,
        ctaUrl: data.ctaUrl,
      }) }
    ),
  sendWhatsAppMedia: (brandId: string, data: { conversationId: string; contactId: string; mediaUrl: string; mediaType: string; mediaFilename?: string; mediaMimetype?: string; caption?: string }) =>
    request<{ sent: boolean; providerMessageId: string; messageId: string }>(
      `/brands/${brandId}/whatsapp/send-media`, { method: 'POST', body: JSON.stringify({
        conversation_id: data.conversationId,
        contact_id: data.contactId,
        media_url: data.mediaUrl,
        media_type: data.mediaType,
        media_filename: data.mediaFilename,
        media_mimetype: data.mediaMimetype,
        caption: data.caption,
      }) }
    ),
  sendWhatsAppVoice: (brandId: string, data: { conversationId: string; contactId: string; audioUrl: string; duration?: number; transcription?: string }) =>
    request<{ sent: boolean; providerMessageId: string; messageId: string }>(
      `/brands/${brandId}/whatsapp/send-voice`, { method: 'POST', body: JSON.stringify({
        conversation_id: data.conversationId,
        contact_id: data.contactId,
        audio_url: data.audioUrl,
        duration: data.duration,
        transcription: data.transcription,
      }) }
    ),
  // WhatsApp Quality History (Priority 5)
  getWhatsAppQualityHistory: (brandId: string) =>
    request<{
      brandId: string;
      history: Array<{
        qualityRating?: string;
        qualityScore?: number;
        messagingLimitTier?: string;
        messagingLimitValue?: number;
        blockReasons: string[];
        spamFlags: number;
        matchedConversations: number;
        createdAt?: string;
      }>;
      trend: string;
      currentScore?: number;
      currentRating?: string;
      currentSpamFlags: number;
      currentTier?: string;
      totalSnapshots: number;
      averageScore: number;
      recommendations: Array<{
        type?: string;
        severity?: string;
        title?: string;
        description?: string;
        createdAt?: string;
      }>;
    }>(`/brands/${brandId}/whatsapp/quality-history`),

  // Commerce Engine
  getBrandId: () => {
    try {
      const brand = localStorage.getItem('communicator.brand')
      if (brand) {
        const { id } = JSON.parse(brand)
        return id || null
      }
    } catch {}
    return null
  },
  listCategories: (brandId: string) =>
    request<CommerceCategory[]>(`/commerce/categories?brand_id=${brandId}`),
  createCategory: (data: { brandId: string; name: string; parentId?: string; sortOrder?: number }) =>
    request<CommerceCategory>('/commerce/categories', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name, parent_id: data.parentId, sort_order: data.sortOrder }) }),
  listProducts: (brandId: string) =>
    request<CommerceProduct[]>(`/commerce/products?brand_id=${brandId}`),
  createProduct: (data: { brandId: string; name: string; price: number; sku?: string; categoryId?: string; description?: string }) =>
    request<CommerceProduct>('/commerce/products', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name, price: data.price, sku: data.sku, category_id: data.categoryId, description: data.description }) }),
  deleteProduct: (id: string) =>
    request<void>(`/commerce/products/${id}`, { method: 'DELETE' }),
  getProduct: (id: string) =>
    request<CommerceProduct>(`/commerce/products/${id}`),
  updateProduct: (id: string, data: { name?: string; price?: number; sku?: string; categoryId?: string; description?: string; compareAtPrice?: number; inventoryCount?: number; trackInventory?: boolean; isActive?: boolean }) =>
    request<CommerceProduct>(`/commerce/products/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  listCarts: (brandId: string) =>
    request<CommerceCart[]>(`/commerce/carts?brand_id=${brandId}`),
  getCartItems: (cartId: string) =>
    request<CommerceCartItem[]>(`/commerce/carts/${cartId}/items`),
  addCartItem: (cartId: string, data: { productId: string; quantity: number; unitPrice: number; variantId?: string }) =>
    request<CommerceCartItem>(`/commerce/carts/${cartId}/items`, { method: 'POST', body: JSON.stringify({ product_id: data.productId, quantity: data.quantity, unit_price: data.unitPrice, variant_id: data.variantId }) }),
  removeCartItem: (itemId: string) =>
    request<void>(`/commerce/carts/items/${itemId}`, { method: 'DELETE' }),
  updateCartItem: (itemId: string, quantity: number) =>
    request<CommerceCartItem>(`/commerce/carts/items/${itemId}`, { method: 'PUT', body: JSON.stringify({ quantity }) }),
  checkoutCart: (cartId: string, data?: { shippingAddress?: string; paymentMethod?: string }) =>
    request<CommerceOrder>(`/commerce/carts/${cartId}/checkout`, { method: 'POST', body: JSON.stringify(data || {}) }),
  listOrders: (brandId: string) =>
    request<CommerceOrder[]>(`/commerce/orders?brand_id=${brandId}`),
  getOrder: (orderId: string) =>
    request<CommerceOrder>(`/commerce/orders/${orderId}`),
  updateOrderStatus: (orderId: string, status: string) =>
    request<CommerceOrder>(`/commerce/orders/${orderId}/update-status`, { method: 'POST', body: JSON.stringify({ status }) }),
  listSyncConnections: (brandId: string) =>
    request<SyncConnection[]>(`/commerce/sync/connections?brand_id=${brandId}`),
  createSyncConnection: (data: { brandId: string; platform: string; credentials: Record<string, any> }) =>
    request<SyncConnection>('/commerce/sync/connections', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, platform: data.platform, credentials: data.credentials }) }),
  deleteSyncConnection: (id: string) =>
    request<void>(`/commerce/sync/connections/${id}`, { method: 'DELETE' }),
  triggerSync: (id: string) =>
    request<any>(`/commerce/sync/connections/${id}/sync`, { method: 'POST' }),
  listSyncLogs: (connectionId?: string) => {
    const qs = connectionId ? `?connection_id=${connectionId}` : ''
    return request<SyncLog[]>(`/commerce/sync/logs${qs}`)
  },
}

// ============================================================================
// WhatsApp Campaigns — Standalone exports used by WhatsAppCampaigns.tsx
// ============================================================================

export async function listCampaigns(brandId: string, status?: string) {
  const qs = status ? `?status=${status}` : ''
  return request<any[]>(`/brands/${brandId}/whatsapp/campaigns${qs}`)
}

export async function createCampaign(brandId: string, data: { name: string; campaign_type: string; template_flow_id?: string; scheduled_at?: string }) {
  return request<any>(`/brands/${brandId}/whatsapp/campaigns`, { method: 'POST', body: JSON.stringify(data) })
}

export async function getCampaign(campaignId: string) {
  return request<any>(`/campaigns/${campaignId}`)
}

export async function deleteCampaign(campaignId: string) {
  return request<{ deleted: boolean }>(`/campaigns/${campaignId}`, { method: 'DELETE' })
}

export async function startCampaign(campaignId: string) {
  return request<{ status: string }>(`/campaigns/${campaignId}/start`, { method: 'POST' })
}

export async function pauseCampaign(campaignId: string) {
  return request<{ status: string }>(`/campaigns/${campaignId}/pause`, { method: 'POST' })
}

export async function cancelCampaign(campaignId: string) {
  return request<{ status: string }>(`/campaigns/${campaignId}/cancel`, { method: 'POST' })
}

export async function addCampaignContacts(campaignId: string, contactIds: string[]) {
  return request<{ added: number }>(`/campaigns/${campaignId}/contacts`, { method: 'POST', body: JSON.stringify({ contact_ids: contactIds }) })
}

export async function retryCampaign(campaignId: string) {
  return request<{ retried: number }>(`/campaigns/${campaignId}/retry`, { method: 'POST' })
}

export async function getCampaignContacts(campaignId: string) {
  return request<any[]>(`/campaigns/${campaignId}/contacts`)
}

export async function getCampaignAnalytics(campaignId: string) {
  return request<any>(`/campaigns/${campaignId}/analytics`)
}

// --- Template Flows ---

export async function listFlows(brandId: string, industryId?: string, groupId?: string, category?: string, status?: string) {
  const params = new URLSearchParams()
  if (industryId) params.set('industry_id', industryId)
  if (groupId) params.set('group_id', groupId)
  if (category) params.set('category', category)
  if (status) params.set('status', status)
  const qs = params.toString()
  return request<any[]>(`/brands/${brandId}/whatsapp/flows${qs ? `?${qs}` : ''}`)
}

export async function createFlow(brandId: string, data: { name: string; body: string; category?: string; industry_id?: string; group_id?: string; flow_type?: string; header?: string; footer?: string; language?: string }) {
  return request<any>(`/brands/${brandId}/whatsapp/flows`, { method: 'POST', body: JSON.stringify(data) })
}

export async function updateFlow(flowId: string, data: { name?: string; body?: string; header?: string; footer?: string; status?: string; is_active?: boolean }) {
  return request<{ updated: boolean }>(`/flows/${flowId}`, { method: 'PUT', body: JSON.stringify(data) })
}

export async function deleteFlow(flowId: string) {
  return request<{ deleted: boolean }>(`/flows/${flowId}`, { method: 'DELETE' })
}

export async function duplicateFlow(flowId: string, newName: string) {
  return request<{ id: string }>(`/flows/${flowId}/duplicate`, { method: 'POST', body: JSON.stringify({ name: newName }) })
}

// --- Flow Steps ---

export async function listFlowSteps(flowId: string) {
  return request<any[]>(`/flows/${flowId}/steps`)
}

export async function createFlowStep(flowId: string, data: { step_number: number; action_type: string; condition?: string; action_config?: string; next_step_id?: string }) {
  return request<any>(`/flows/${flowId}/steps`, { method: 'POST', body: JSON.stringify(data) })
}

export async function deleteFlowStep(stepId: string) {
  return request<{ deleted: boolean }>(`/steps/${stepId}`, { method: 'DELETE' })
}

// --- Industries ---

export async function listIndustries() {
  const brandId = localStorage.getItem('selectedBrand') || ''
  return request<any[]>(brandId ? `/brands/${brandId}/whatsapp/industries` : '/industries')
}

export async function createIndustry(name: string, description?: string) {
  const brandId = localStorage.getItem('selectedBrand') || ''
  return request<any>(brandId ? `/brands/${brandId}/whatsapp/industries` : '/industries', { method: 'POST', body: JSON.stringify({ name, description }) })
}

// --- Template Groups ---

export async function listTemplateGroups(brandId: string, industryId?: string) {
  const qs = industryId ? `?industry_id=${industryId}` : ''
  return request<any[]>(`/brands/${brandId}/whatsapp/groups${qs}`)
}

export async function createTemplateGroup(brandId: string, data: { name: string; description?: string; industry_id?: string; sort_order?: number }) {
  return request<any>(`/brands/${brandId}/whatsapp/groups`, { method: 'POST', body: JSON.stringify(data) })
}

// --- Message Limits ---

export async function getMessageLimits(brandId: string) {
  return request<any>(`/brands/${brandId}/whatsapp/message-limits`)
}

export async function syncMessageLimits(brandId: string) {
  return request<{ synced: boolean }>(`/brands/${brandId}/whatsapp/sync-message-limits`, { method: 'POST' })
}

// --- Meta Templates Sync ---

export async function syncTemplates(brandId: string) {
  return request<{ synced: number }>(`/brands/${brandId}/whatsapp/sync-templates-now`, { method: 'POST' })
}

export async function listTemplates(brandId: string) {
  return request<any[]>(`/brands/${brandId}/whatsapp/templates`)
}

// --- AI Campaign ---

export async function aiSuggestCampaign(brandId: string, data: { category?: string; audience?: string }) {
  return request<any>(`/brands/${brandId}/whatsapp/campaigns/ai-suggest`, { method: 'POST', body: JSON.stringify(data) })
}

export async function aiGenerateCampaign(brandId: string, data: { campaign_type?: string; audience?: string; goal?: string }) {
  return request<any>(`/brands/${brandId}/whatsapp/campaigns/ai-generate`, { method: 'POST', body: JSON.stringify(data) })
}

// --- Contacts (for contact selection tab) ---

export async function listContacts(_brandId: string, limit?: number) {
  const qs = limit ? `?limit=${limit}` : ''
  return request<any>(`/contacts${qs}`)
}

// --- Commerce Engine ---

export interface CommerceProduct {
  id: string
  brandId: string
  categoryId?: string
  name: string
  description?: string
  sku?: string
  price?: number
  compareAtPrice?: number
  currency: string
  images?: string
  variants?: string
  inventoryCount?: number
  trackInventory: boolean
  visibility?: string
  metadata?: string
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface CommerceCategory {
  id: string
  brandId: string
  name: string
  parentId?: string
  sortOrder: number
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface CommerceOrder {
  id: string
  brandId: string
  customerId: string
  orderNumber: string
  channelType?: string
  status: string
  totalAmount?: number
  subtotal: number
  tax: number
  shipping: number
  currency: string
  itemsSnapshot?: string
  shippingAddress?: string
  paymentStatus: string
  paymentMethod?: string
  metadata?: string
  source: string
  externalOrderId?: string
  createdAt: string
  updatedAt: string
}

export interface CommerceCart {
  id: string
  customerId: string
  brandId: string
  channelType?: string
  status: string
  metadata?: string
  createdAt: string
  updatedAt: string
}

export interface CommerceCartItem {
  id: string
  cartId: string
  productId: string
  variantId?: string
  quantity: number
  unitPrice: number
  metadata?: string
  createdAt: string
}

export interface SyncConnection {
  id: string
  brandId: string
  platform: string
  credentials?: string
  syncStatus: string
  lastSyncAt?: string
  metadata?: string
  createdAt: string
  updatedAt: string
}

export interface SyncLog {
  id: string
  connectionId: string
  direction: string
  entityType: string
  entityId?: string
  status: string
  error?: string
  createdAt: string
}

export function getBrandId(): string | null {
  try {
    const brand = localStorage.getItem('communicator.brand')
    if (brand) {
      const { id } = JSON.parse(brand)
      return id || null
    }
  } catch {}
  return null
}

// Categories
export async function listCategories(brandId: string) {
  return request<CommerceCategory[]>(`/commerce/categories?brand_id=${brandId}`)
}

export async function createCategory(data: { brandId: string; name: string; parentId?: string; sortOrder?: number }) {
  return request<CommerceCategory>('/commerce/categories', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name, parent_id: data.parentId, sort_order: data.sortOrder }) })
}

// Products
export async function listProducts(brandId: string) {
  return request<CommerceProduct[]>(`/commerce/products?brand_id=${brandId}`)
}

export async function createProduct(data: { brandId: string; name: string; price: number; sku?: string; categoryId?: string; description?: string }) {
  return request<CommerceProduct>('/commerce/products', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name, price: data.price, sku: data.sku, category_id: data.categoryId, description: data.description }) })
}

export async function deleteProduct(id: string) {
  return request<void>(`/commerce/products/${id}`, { method: 'DELETE' })
}

export async function getProduct(id: string) {
  return request<CommerceProduct>(`/commerce/products/${id}`)
}

export async function updateProduct(id: string, data: { name?: string; price?: number; sku?: string; categoryId?: string; description?: string; compareAtPrice?: number; inventoryCount?: number; trackInventory?: boolean; isActive?: boolean }) {
  return request<CommerceProduct>(`/commerce/products/${id}`, { method: 'PUT', body: JSON.stringify(data) })
}

// Carts
export async function listCarts(brandId: string, customerId?: string) {
  const qs = customerId ? `?brand_id=${brandId}&customer_id=${customerId}` : `?brand_id=${brandId}`
  return request<CommerceCart[]>(`/commerce/carts${qs}`)
}

export async function createCart(data: { customerId: string; brandId: string; channelType?: string }) {
  return request<CommerceCart>('/commerce/carts', { method: 'POST', body: JSON.stringify({ customer_id: data.customerId, brand_id: data.brandId, channel_type: data.channelType }) })
}

export async function getCartItems(cartId: string) {
  return request<CommerceCartItem[]>(`/commerce/carts/${cartId}/items`)
}

export async function addCartItem(cartId: string, data: { productId: string; quantity: number; unitPrice: number; variantId?: string }) {
  return request<CommerceCartItem>(`/commerce/carts/${cartId}/items`, { method: 'POST', body: JSON.stringify({ product_id: data.productId, quantity: data.quantity, unit_price: data.unitPrice, variant_id: data.variantId }) })
}

export async function removeCartItem(itemId: string) {
  return request<void>(`/commerce/carts/items/${itemId}`, { method: 'DELETE' })
}

export async function updateCartItem(itemId: string, quantity: number) {
  return request<CommerceCartItem>(`/commerce/carts/items/${itemId}`, { method: 'PUT', body: JSON.stringify({ quantity }) })
}

export async function checkoutCart(cartId: string, data?: { shippingAddress?: string; paymentMethod?: string }) {
  return request<CommerceOrder>(`/commerce/carts/${cartId}/checkout`, { method: 'POST', body: JSON.stringify(data || {}) })
}

// Orders
export async function listOrders(brandId: string) {
  return request<CommerceOrder[]>(`/commerce/orders?brand_id=${brandId}`)
}

export async function updateOrderStatus(orderId: string, status: string) {
  return request<CommerceOrder>(`/commerce/orders/${orderId}/update-status`, { method: 'POST', body: JSON.stringify({ status }) })
}

export async function getOrder(orderId: string) {
  return request<CommerceOrder>(`/commerce/orders/${orderId}`)
}

// Sync
export async function listSyncConnections(brandId: string) {
  return request<SyncConnection[]>(`/commerce/sync/connections?brand_id=${brandId}`)
}

export async function createSyncConnection(data: { brandId: string; platform: string; credentials: Record<string, any> }) {
  return request<SyncConnection>('/commerce/sync/connections', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, platform: data.platform, credentials: data.credentials }) })
}

export async function deleteSyncConnection(id: string) {
  return request<void>(`/commerce/sync/connections/${id}`, { method: 'DELETE' })
}

export async function triggerSync(id: string) {
  return request<any>(`/commerce/sync/connections/${id}/sync`, { method: 'POST' })
}

export async function listSyncLogs(connectionId?: string) {
  const qs = connectionId ? `?connection_id=${connectionId}` : ''
  return request<SyncLog[]>(`/commerce/sync/logs${qs}`)
}

export async function syncAllPlatforms(brandId: string) {
  return request<any[]>(`/commerce/sync/all?brand_id=${brandId}`, { method: 'POST' })
}

export async function getPlatformInfo(connectionId: string) {
  return request<any>(`/commerce/sync/connections/${connectionId}/info`)
}

export async function listProductMappings(connectionId: string) {
  return request<any[]>(`/commerce/sync/connections/${connectionId}/mappings`)
}

export async function processSyncQueue(connectionId?: string) {
  const qs = connectionId ? `?connection_id=${connectionId}` : ''
  return request<any>(`/commerce/sync/queue/process${qs}`, { method: 'POST' })
}

// Sync methods on api namespace
const syncConnections$ = listSyncConnections
const createSyncConnection$ = createSyncConnection
const deleteSyncConnection$ = deleteSyncConnection
const triggerSync$ = triggerSync
const listSyncLogs$ = listSyncLogs

// --- Phase 2: Coupons ---

export interface CommerceCoupon {
  id: string
  brandId: string
  code: string
  type: string
  value: number
  minOrderAmount?: number
  maxDiscount?: number
  usageLimit?: number
  usedCount: number
  startsAt?: string
  endsAt?: string
  applicableProducts?: string
  applicableCategories?: string
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface CouponUsage {
  id: string
  couponId: string
  orderId: string
  customerId: string
  discountAmount: number
  usedAt: string
}

export interface MembershipTier {
  id: string
  brandId: string
  name: string
  discountPercent: number
  monthlyPrice?: number
  benefits?: string
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface CustomerMembership {
  id: string
  customerId: string
  membershipTierId: string
  startedAt: string
  endsAt?: string
  status: string
  createdAt: string
}

export async function listCoupons(brandId: string) {
  return request<CommerceCoupon[]>(`/commerce/coupons?brand_id=${brandId}`)
}

export async function createCoupon(data: { brandId: string; code: string; type?: string; value: number; minOrderAmount?: number; maxDiscount?: number; usageLimit?: number }) {
  return request<CommerceCoupon>('/commerce/coupons', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, code: data.code, type: data.type, value: data.value, min_order_amount: data.minOrderAmount, max_discount: data.maxDiscount, usage_limit: data.usageLimit }) })
}

export async function updateCoupon(id: string, data: { code?: string; type?: string; value?: number; isActive?: boolean }) {
  return request<CommerceCoupon>(`/commerce/coupons/${id}`, { method: 'PUT', body: JSON.stringify(data) })
}

export async function deleteCoupon(id: string) {
  return request<void>(`/commerce/coupons/${id}`, { method: 'DELETE' })
}

export async function validateCoupon(code: string, orderAmount: number) {
  return request<any>(`/commerce/coupons/validate`, { method: 'POST', body: JSON.stringify({ code, order_amount: orderAmount }) })
}

export async function listMembershipTiers(brandId: string) {
  return request<MembershipTier[]>(`/commerce/memberships?brand_id=${brandId}`)
}

export async function createMembershipTier(data: { brandId: string; name: string; discountPercent?: number; monthlyPrice?: number }) {
  return request<MembershipTier>('/commerce/memberships', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name, discount_percent: data.discountPercent, monthly_price: data.monthlyPrice }) })
}

export async function deleteMembershipTier(id: string) {
  return request<void>(`/commerce/memberships/${id}`, { method: 'DELETE' })
}

// --- Phase 2: Analytics ---

export interface AnalyticsOverview {
  totalRevenue: number
  totalOrders: number
  avgOrderValue: number
  totalCustomers: number
  totalProducts: number
  revenueChangePercent: number
  ordersChangePercent: number
}

export interface RevenueTrendPoint {
  date: string
  revenue: number
  orders: number
}

export interface TopProduct {
  productId: string
  productName: string
  sku?: string
  quantitySold: number
  revenue: number
}

export interface TopCustomer {
  customerId: string
  customerName?: string
  email?: string
  totalOrders: number
  totalSpent: number
}

export interface ChannelPerformance {
  channelType: string
  ordersCount: number
  revenue: number
}

export async function getAnalyticsOverview(brandId: string, days?: number) {
  const qs = days ? `?brand_id=${brandId}&days=${days}` : `?brand_id=${brandId}`
  return request<AnalyticsOverview>(`/commerce/analytics/overview${qs}`)
}

export async function getRevenueTrend(brandId: string, days?: number) {
  const qs = days ? `?brand_id=${brandId}&days=${days}` : `?brand_id=${brandId}`
  return request<RevenueTrendPoint[]>(`/commerce/analytics/revenue${qs}`)
}

export async function getTopProducts(brandId: string, limit?: number) {
  const qs = limit ? `?brand_id=${brandId}&limit=${limit}` : `?brand_id=${brandId}`
  return request<TopProduct[]>(`/commerce/analytics/top-products${qs}`)
}

export async function getTopCustomers(brandId: string, limit?: number) {
  const qs = limit ? `?brand_id=${brandId}&limit=${limit}` : `?brand_id=${brandId}`
  return request<TopCustomer[]>(`/commerce/analytics/top-customers${qs}`)
}

export async function getChannelPerformance(brandId: string) {
  return request<ChannelPerformance[]>(`/commerce/analytics/channels?brand_id=${brandId}`)
}

// --- Phase 2: Shipping ---

export interface ShippingMethod {
  id: string
  brandId: string
  name: string
  type: string
  rate: number
  freeShippingThreshold?: number
  regions?: string
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface Shipment {
  id: string
  orderId: string
  trackingNumber?: string
  carrier?: string
  status: string
  estimatedDelivery?: string
  actualDelivery?: string
  shippingLabelUrl?: string
  createdAt: string
  updatedAt: string
}

export interface DeliveryEvent {
  id: string
  shipmentId: string
  eventType: string
  location?: string
  description?: string
  timestamp: string
}

export async function listShippingMethods(brandId: string) {
  return request<ShippingMethod[]>(`/commerce/shipping-methods?brand_id=${brandId}`)
}

export async function createShippingMethod(data: { brandId: string; name: string; type?: string; rate?: number }) {
  return request<ShippingMethod>('/commerce/shipping-methods', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name, type: data.type, rate: data.rate }) })
}

export async function deleteShippingMethod(id: string) {
  return request<void>(`/commerce/shipping-methods/${id}`, { method: 'DELETE' })
}

export async function createShipment(data: { orderId: string; trackingNumber?: string; carrier?: string }) {
  return request<Shipment>('/commerce/shipments', { method: 'POST', body: JSON.stringify({ order_id: data.orderId, tracking_number: data.trackingNumber, carrier: data.carrier }) })
}

export async function updateShipment(id: string, data: { trackingNumber?: string; status?: string; carrier?: string }) {
  return request<Shipment>(`/commerce/shipments/${id}`, { method: 'PUT', body: JSON.stringify({ tracking_number: data.trackingNumber, status: data.status, carrier: data.carrier }) })
}

export async function getShipment(id: string) {
  return request<Shipment>(`/commerce/shipments/${id}`)
}

export async function getDeliveryEvents(shipmentId: string) {
  return request<DeliveryEvent[]>(`/commerce/shipments/${shipmentId}/events`)
}

// --- Phase 2: Storefront ---

export interface Storefront {
  id: string
  brandId: string
  slug: string
  title: string
  description?: string
  theme?: string
  heroImage?: string
  seoTitle?: string
  seoDescription?: string
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface StorefrontSection {
  id: string
  storefrontId: string
  sectionType: string
  sortOrder: number
  content?: string
  createdAt: string
}

export async function listStorefronts(brandId: string) {
  return request<Storefront[]>(`/commerce/storefronts?brand_id=${brandId}`)
}

export async function createStorefront(data: { brandId: string; slug: string; title: string; description?: string; seoTitle?: string; seoDescription?: string }) {
  return request<Storefront>('/commerce/storefronts', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, slug: data.slug, title: data.title, description: data.description, seo_title: data.seoTitle, seo_description: data.seoDescription }) })
}

export async function updateStorefront(id: string, data: { title?: string; description?: string; isActive?: boolean }) {
  return request<Storefront>(`/commerce/storefronts/${id}`, { method: 'PUT', body: JSON.stringify(data) })
}

export async function deleteStorefront(id: string) {
  return request<void>(`/commerce/storefronts/${id}`, { method: 'DELETE' })
}

// --- Phase 2: Inventory ---

export interface Warehouse {
  id: string
  brandId: string
  name: string
  address?: string
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface InventoryByWarehouse {
  id: string
  productId: string
  warehouseId: string
  quantity: number
  reservedQuantity: number
  reorderPoint: number
  createdAt: string
  updatedAt: string
}

export interface LowStockAlert {
  id: string
  productId: string
  warehouseId: string
  currentQuantity: number
  reorderPoint: number
  alertedAt: string
  acknowledged: boolean
}

export async function listWarehouses(brandId: string) {
  return request<Warehouse[]>(`/commerce/warehouses?brand_id=${brandId}`)
}

export async function createWarehouse(data: { brandId: string; name: string }) {
  return request<Warehouse>('/commerce/warehouses', { method: 'POST', body: JSON.stringify({ brand_id: data.brandId, name: data.name }) })
}

export async function deleteWarehouse(id: string) {
  return request<void>(`/commerce/warehouses/${id}`, { method: 'DELETE' })
}

export async function setInventory(data: { productId: string; warehouseId: string; quantity: number; reorderPoint?: number }) {
  return request<InventoryByWarehouse>('/commerce/inventory', { method: 'PUT', body: JSON.stringify({ product_id: data.productId, warehouse_id: data.warehouseId, quantity: data.quantity, reorder_point: data.reorderPoint }) })
}

export async function getInventorySummary(productId: string) {
  return request<any>(`/commerce/inventory/${productId}/summary`)
}

export async function listLowStockAlerts() {
  return request<LowStockAlert[]>('/commerce/inventory/alerts')
}

export async function acknowledgeAlert(id: string) {
  return request<void>(`/commerce/inventory/alerts/${id}/acknowledge`, { method: 'POST' })
}

// --- Phase 9: AI-Powered Suggestions ---

export interface Suggestion {
  id: string
  suggestionType: string  // 'knowledge_gap', 'faq_addition', 'agent_insight'
  title: string
  description: string
  suggestedContent: string | null
  sourceFeedbackId: string | null
  sourceConversationId: string | null
  sourceMessageBody: string | null
  confidence: number
  category: string | null
  status: string  // 'suggested', 'approved', 'dismissed', 'applied'
  approvedBy: string | null
  approvedAt: string | null
  createdAt: string
  updatedAt: string
}

export interface SuggestionTypeCount {
  suggestionType: string
  count: number
}

export interface SuggestionCategoryCount {
  category: string
  count: number
}

export interface SuggestionSummary {
  totalSuggestions: number
  openSuggestions: number
  approvedSuggestions: number
  dismissedSuggestions: number
  byType: SuggestionTypeCount[]
  byCategory: SuggestionCategoryCount[]
  recentSuggestions: Suggestion[]
}

export async function listSuggestions(status?: string, suggestionType?: string, limit?: number) {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (suggestionType) params.set('suggestion_type', suggestionType)
  if (limit) params.set('limit', String(limit))
  const query = params.toString()
  return request<{ suggestions: Suggestion[]; count: number }>(`/suggestions${query ? `?${query}` : ''}`)
}

export async function getSuggestionSummary() {
  return request<SuggestionSummary>('/suggestions/summary')
}

export async function approveSuggestion(id: string, approvedBy?: string) {
  return request<{ suggestion: Suggestion; status: string }>(`/suggestions/${id}/approve`, {
    method: 'POST',
    body: JSON.stringify({ approved_by: approvedBy || 'system' }),
  })
}

export async function dismissSuggestion(id: string) {
  return request<{ suggestion: Suggestion; status: string }>(`/suggestions/${id}/dismiss`, {
    method: 'POST',
  })
}

export async function triggerBatchAnalysis(hoursBack?: number) {
  const hours = hoursBack || 24
  return request<{ status: string; feedbackAnalyzed: number; suggestionsCreated: number; errors: number }>(
    `/suggestions/batch-analyze?hours_back=${hours}`,
    { method: 'POST' }
  )
}

export async function analyzeFeedback(feedbackId: string) {
  return request<{ status: string; feedbackId: string; suggestionsCreated: number; suggestions: Suggestion[] }>(
    `/suggestions/analyze-feedback/${feedbackId}`,
    { method: 'POST' }
  )
}

// --- Phase 11: Topic Categorization ---

export interface ConversationTopic {
  id: string
  conversation_id: string
  topic: string
  subtopic: string | null
  confidence: number
  keywords: string[]
  is_overridden: boolean
  overridden_by: string | null
  categorized_at: string
  created_at: string
  updated_at: string
}

export interface TopicCount {
  topic: string
  count: number
  percentage: number
}

export interface KeywordCount {
  keyword: string
  count: number
}

export interface TrendingTopic {
  topic: string
  current_count: number
  previous_count: number
  change_percentage: number
}

export interface TopicSummary {
  total_conversations: number
  categorized_count: number
  uncategorized_count: number
  topic_counts: TopicCount[]
  top_keywords: KeywordCount[]
  trending_topics: TrendingTopic[]
}

export interface BatchCategorizationResult {
  total: number
  categorized: number
  failed: number
}

export async function getTopicSummary() {
  return request<TopicSummary>('/topics/summary')
}

export async function getConversationTopic(conversationId: string) {
  return request<ConversationTopic>(`/topics/${conversationId}`)
}

export async function categorizeConversation(conversationId: string) {
  return request<ConversationTopic>(`/topics/${conversationId}`, { method: 'POST' })
}

export async function overrideConversationTopic(
  conversationId: string,
  topic: string,
  subtopic?: string,
  overriddenBy?: string
) {
  return request<ConversationTopic>(`/topics/${conversationId}`, {
    method: 'PUT',
    body: JSON.stringify({ topic, subtopic, overridden_by: overriddenBy || 'manager' }),
  })
}

export async function batchCategorize() {
  return request<BatchCategorizationResult>('/topics/batch-categorize', { method: 'POST' })
}

// --- Phase 16: CX Scoring & Automated QA ---

export interface QAScore {
  id: string
  conversation_id: string
  overall_score: number
  accuracy_score: number
  empathy_score: number
  resolution_score: number
  compliance_score: number
  timeliness_score: number
  evaluation_summary: string
  issues: string[]
  recommendations: string[]
  is_auto_scored: boolean
  scored_by: string | null
  scored_at: string
  created_at: string
  updated_at: string
}

export interface IssueCount {
  issue: string
  count: number
}

export interface DimensionTrend {
  dimension: string
  current_avg: number
  previous_avg: number
  change: number
}

export interface CXSummary {
  overall_cx_score: number
  avg_accuracy: number
  avg_empathy: number
  avg_resolution: number
  avg_compliance: number
  avg_timeliness: number
  total_conversations_scored: number
  conversations_last_24h: number
  conversations_last_7d: number
  trend_direction: string
  top_issues: IssueCount[]
  dimension_trend: DimensionTrend[]
}

export interface QAScoringRule {
  id: string
  name: string
  description: string
  dimension: string
  condition: any
  score_impact: number
  is_enabled: boolean
  severity: string
  created_at: string
  updated_at: string
}

export interface BatchScoringResult {
  total: number
  scored: number
  failed: number
  skipped: number
}

export async function getCXSummary() {
  return request<CXSummary>('/cx-qa/summary')
}

export async function getQAScore(conversationId: string) {
  return request<QAScore>(`/cx-qa/${conversationId}`)
}

export async function batchScoreConversations() {
  return request<BatchScoringResult>('/cx-qa/batch-score', { method: 'POST' })
}

export async function listQARules() {
  return request<QAScoringRule[]>('/cx-qa/rules')
}

export async function updateQARule(
  id: string,
  isEnabled?: boolean,
  scoreImpact?: number,
) {
  const body: Record<string, any> = {}
  if (isEnabled !== undefined) body.is_enabled = isEnabled
  if (scoreImpact !== undefined) body.score_impact = scoreImpact
  return request<QAScoringRule>(`/cx-qa/rules/${id}`, {
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

// --- Phase 15: CSAT Surveys ---

export interface CSATSurveyTemplate {
  id: string
  name: string
  description: string | null
  question: string
  scale_type: string
  min_score: number
  max_score: number
  include_comment: boolean
  comment_prompt: string | null
  trigger_type: string
  trigger_delay_minutes: number
  channel_filter: string | null
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface CSATSurvey {
  id: string
  template_id: string
  conversation_id: string
  contact_id: string
  channel: string
  contact_identifier: string
  status: string
  score: number | null
  comment: string | null
  sent_at: string | null
  responded_at: string | null
  expires_at: string | null
  created_at: string
}

export interface CSATComment {
  survey_id: string
  score: number
  comment: string
  contact_identifier: string
  channel: string
  responded_at: string
}

export interface CSATSummary {
  overall_csat: number | null
  overall_average_score: number | null
  total_surveys_sent: number
  total_responses: number
  response_rate: number
  score_distribution: Record<string, number>
  by_channel: Record<string, { avg: number; count: number }>
  trend_7d: number | null
  recent_comments: CSATComment[]
}

export interface CSATSnapshot {
  id: string
  window_start: string
  window_end: string
  total_surveys_sent: number
  total_responses: number
  response_rate: number
  average_score: number | null
  csat_percentage: number | null
  created_at: string
}

export async function listCSATTemplates() {
  return request<{ templates: CSATSurveyTemplate[]; total: number }>('/csat/templates')
}

export async function getCSATTemplate(id: string) {
  return request<CSATSurveyTemplate>(`/csat/templates/${id}`)
}

export async function createCSATTemplate(body: {
  name: string
  question?: string
  scale_type?: string
  include_comment?: boolean
  comment_prompt?: string
  trigger_type?: string
}) {
  return request<CSATSurveyTemplate>('/csat/templates', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function updateCSATTemplate(id: string, body: Record<string, any>) {
  return request<CSATSurveyTemplate>(`/csat/templates/${id}`, {
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

export async function deleteCSATTemplate(id: string) {
  return request<{ message: string }>(`/csat/templates/${id}`, {
    method: 'DELETE',
  })
}

export async function listCSATSurveys(
  status?: string,
  channel?: string,
  limit?: number,
) {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (channel) params.set('channel', channel)
  if (limit) params.set('limit', String(limit))
  return request<{ surveys: CSATSurvey[]; total: number }>(`/csat/surveys?${params.toString()}`)
}

export async function createCSATSurvey(body: {
  conversation_id: string
  contact_id: string
  channel: string
  template_id?: string
  contact_identifier?: string
}) {
  return request<CSATSurvey>('/csat/surveys', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function respondToCSATSurvey(
  id: string,
  score: number,
  comment?: string,
) {
  return request<{ id: string; score: number; responded_at: string }>(`/csat/surveys/${id}/respond`, {
    method: 'POST',
    body: JSON.stringify({ score, comment }),
  })
}

export async function sendCSATSurvey(id: string) {
  return request<{ id: string; status: string }>(`/csat/surveys/${id}/send`, {
    method: 'POST',
  })
}

export async function getCSATSummary(days?: number) {
  const params = new URLSearchParams()
  if (days) params.set('limit', String(days))
  return request<CSATSummary>(`/csat/summary?${params.toString()}`)
}

export async function listCSATSnapshots() {
  return request<{ snapshots: CSATSnapshot[]; total: number }>('/csat/snapshots')
}

export async function generateCSATSnapshot() {
  return request<{ snapshot: CSATSnapshot }>('/csat/snapshots', { method: 'POST' })
}

// ——— Phase 17: Fin Operator ———

export interface OperatorProposal {
  id: string
  proposalType: string
  title: string
  description: string
  rationale: string
  expectedImpact?: Record<string, unknown>
  implementationPlan?: Record<string, unknown>
  status: string
  priority: string
  confidence: number
  source: string
  sourceData?: Record<string, unknown>
  relatedFeedbackIds?: string[]
  relatedConversationIds?: string[]
  createdBy: string
  reviewedBy?: string
  reviewedAt?: string
  deployedAt?: string
  deploymentResult?: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface OperatorConfig {
  id: string
  scanIntervalMinutes: number
  autoDeployEnabled: boolean
  autoDeployMaxRisk: string
  maxProposalsPerScan: number
  minConfidenceThreshold: number
  metricsToMonitor: unknown[]
  alertOnFailure: boolean
  enabled: boolean
  lastScanAt?: string
  lastScanResult?: Record<string, unknown>
  updatedAt: string
}

export interface OperatorMetricsSnapshot {
  id: string
  snapshotTime: string
  metricsData: Record<string, unknown>
  anomalies?: unknown[]
  trendData?: Record<string, unknown>
  createdAt: string
}

export interface OperatorPerformanceLog {
  id: string
  actionType: string
  actionDetail: string
  target?: string
  result: string
  outputData?: Record<string, unknown>
  durationMs?: number
  createdAt: string
}

export interface OperatorSummary {
  config: OperatorConfig
  totalProposals: number
  proposalsByStatus: Record<string, number>
  proposalsByType: Record<string, number>
  recentProposals: OperatorProposal[]
  recentLogs: OperatorPerformanceLog[]
  latestSnapshot?: OperatorMetricsSnapshot
  anomalyCount: number
}

export async function getOperatorSummary() {
  return request<OperatorSummary>('/operator/summary')
}

export async function getOperatorConfig() {
  return request<OperatorConfig>('/operator/config')
}

export async function updateOperatorConfig(updates: Record<string, unknown>) {
  return request<OperatorConfig>('/operator/config', { method: 'PUT', body: JSON.stringify(updates) })
}

export async function runOperatorScan() {
  return request<unknown>('/operator/scan', { method: 'POST' })
}

export async function listOperatorProposals(status?: string, type?: string, limit?: number) {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (type) params.set('proposal_type', type)
  if (limit) params.set('limit', String(limit))
  return request<{ proposals: OperatorProposal[]; total: number }>(`/operator/proposals?${params}`)
}

export async function getOperatorProposal(id: string) {
  return request<OperatorProposal>(`/operator/proposals/${id}`)
}

export async function approveOperatorProposal(id: string, reviewer?: string) {
  return request<OperatorProposal>(`/operator/proposals/${id}/approve`, {
    method: 'POST',
    body: JSON.stringify({ reviewer: reviewer || 'admin' }),
  })
}

export async function dismissOperatorProposal(id: string, reviewer?: string) {
  return request<{ message: string; id: string }>(`/operator/proposals/${id}/dismiss`, {
    method: 'POST',
    body: JSON.stringify({ reviewer: reviewer || 'admin' }),
  })
}

export async function listOperatorLogs(actionType?: string, limit?: number) {
  const params = new URLSearchParams()
  if (actionType) params.set('action_type', actionType)
  if (limit) params.set('limit', String(limit))
  return request<{ logs: OperatorPerformanceLog[]; total: number }>(`/operator/logs?${params}`)
}

export async function listOperatorSnapshots(limit?: number) {
  const params = new URLSearchParams()
  if (limit) params.set('limit', String(limit))
  return request<{ snapshots: OperatorMetricsSnapshot[]; total: number }>(`/operator/snapshots?${params}`)
}

// ——— Phase 18: AI Procedures & Actions ———

export interface ProcedureStep {
  order: number
  name: string
  description: string
  actionType: string
  actionConfig: Record<string, unknown>
  expectedResponse: string
  timeoutSeconds: number
}

export interface Procedure {
  id: string
  name: string
  description?: string
  category: string
  triggerKeywords: string[]
  steps: ProcedureStep[]
  maxIterations: number
  timeoutSeconds: number
  status: string
  brandId?: string
  createdBy?: string
  createdAt: string
  updatedAt: string
}

export interface ProcedureExecution {
  id: string
  procedureId: string
  conversationId: string
  currentStep: number
  state: string
  contextData?: Record<string, unknown>
  resultData?: Record<string, unknown>
  errorMessage?: string
  startedAt: string
  completedAt?: string
}

export interface ProcedureAction {
  id: string
  name: string
  description?: string
  actionType: string
  endpoint?: string
  method: string
  headers: Record<string, string>
  bodyTemplate?: string
  authConfig?: Record<string, unknown>
  timeoutSeconds: number
  retryCount: number
  enabled: boolean
  brandId?: string
  createdBy?: string
  createdAt: string
  updatedAt: string
}

export interface ProcedureSummary {
  totalProcedures: number
  proceduresByStatus: Record<string, number>
  proceduresByCategory: Record<string, number>
  activeProcedures: Procedure[]
  totalExecutions: number
  executionsByState: Record<string, number>
  recentExecutions: ProcedureExecution[]
  totalActions: number
  actionsByType: Record<string, number>
}

export async function getProcedureSummary() {
  return request<ProcedureSummary>('/procedures/summary')
}

export async function listProcedures(status?: string, category?: string) {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (category) params.set('category', category)
  return request<Procedure[]>(`/procedures?${params}`)
}

export async function createProcedure(body: Record<string, unknown>) {
  return request<Procedure>('/procedures', { method: 'POST', body: JSON.stringify(body) })
}

export async function getProcedure(id: string) {
  return request<Procedure>(`/procedures/${id}`)
}

export async function updateProcedure(id: string, updates: Record<string, unknown>) {
  return request<Procedure>(`/procedures/${id}`, { method: 'PUT', body: JSON.stringify(updates) })
}

export async function deleteProcedure(id: string) {
  return request<null>(`/procedures/${id}`, { method: 'DELETE' })
}

export async function matchProcedures(message: string, conversationId: string) {
  return request<unknown[]>('/procedures/match', {
    method: 'POST',
    body: JSON.stringify({ message, conversation_id: conversationId }),
  })
}

export async function startProcedureExecution(procedureId: string, conversationId: string) {
  return request<ProcedureExecution>(`/procedures/${procedureId}/execute`, {
    method: 'POST',
    body: JSON.stringify({ conversation_id: conversationId }),
  })
}

export async function getProcedureExecution(id: string) {
  return request<ProcedureExecution>(`/procedures/executions/${id}`)
}

export async function getRunningExecution(conversationId: string) {
  return request<ProcedureExecution | null>(`/procedures/executions/conversation/${conversationId}`)
}

export async function executeProcedureStep(executionId: string, customerInput?: string) {
  return request<unknown>(`/procedures/executions/${executionId}/step`, {
    method: 'POST',
    body: JSON.stringify({ customer_input: customerInput }),
  })
}

export async function completeProcedureExecution(executionId: string, result: string) {
  return request<null>(`/procedures/executions/${executionId}/complete`, {
    method: 'POST',
    body: JSON.stringify({ result }),
  })
}

export async function failProcedureExecution(executionId: string, error: string) {
  return request<null>(`/procedures/executions/${executionId}/fail`, {
    method: 'POST',
    body: JSON.stringify({ error }),
  })
}

export async function listActions(actionType?: string) {
  const params = new URLSearchParams()
  if (actionType) params.set('action_type', actionType)
  return request<ProcedureAction[]>(`/actions?${params}`)
}

export async function createAction(body: Record<string, unknown>) {
  return request<ProcedureAction>('/actions', { method: 'POST', body: JSON.stringify(body) })
}

export async function getAction(id: string) {
  return request<ProcedureAction>(`/actions/${id}`)
}

export async function updateAction(id: string, updates: Record<string, unknown>) {
  return request<ProcedureAction>(`/actions/${id}`, { method: 'PUT', body: JSON.stringify(updates) })
}

export async function deleteAction(id: string) {
  return request<null>(`/actions/${id}`, { method: 'DELETE' })
}

// ——— Phase 19: Unified Inbox ———

export interface UnifiedCustomer {
  customerId: string
  displayName: string
  email?: string
  phone?: string
  company?: string
  avatarUrl?: string
  channels: string[]
  conversationCount: number
  activeConversations: number
  lastMessageAt?: string
  lastMessagePreview?: string
  lastMessageChannel?: string
  tags: string[]
  customerTier?: string
  lifetimeValue: number
}

export interface UnifiedConversation {
  conversationId: string
  channel: string
  state: string
  subject?: string
  lastMessageAt?: string
  lastMessagePreview?: string
  lastMessageDirection?: string
  unreadCount: number
  messageCount: number
  assignedTo?: string
  priority?: string
  createdAt: string
}

export interface UnifiedMessage {
  id: string
  conversationId: string
  direction: string
  channel: string
  body: string
  senderName?: string
  sentAt: string
  mediaType?: string
  mediaUrl?: string
  isPrivate?: boolean
}

export interface UnifiedInboxEntry {
  customer: UnifiedCustomer
  conversations: UnifiedConversation[]
  mergedTimeline: UnifiedMessage[]
}

export interface UnifiedInboxSummary {
  totalCustomers: number
  activeCustomers: number
  conversationsByChannel: Record<string, number>
  conversationsByState: Record<string, number>
  recentEntries: UnifiedInboxEntry[]
}

export interface UnifiedInboxSettings {
  id: string
  organizationId: string
  enabled: boolean
  groupBy: string
  mergeTimeline: boolean
  showChannelBadges: boolean
  autoResolve: boolean
}

export interface CustomerMapping {
  id: string
  customerId: string
  contactId: string
  channel: string
  channelIdentifier: string
  confidence: number
  resolvedBy: string
}

export async function getUnifiedInbox(
  limit = 20,
  offset = 0,
  channel?: string,
  state?: string,
  search?: string,
) {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  params.set('offset', String(offset))
  if (channel) params.set('channel', channel)
  if (state) params.set('state', state)
  if (search) params.set('search', search)
  return request<UnifiedInboxSummary>(`/unified-inbox?${params}`)
}

export async function getUnifiedCustomer(customerId: string) {
  return request<UnifiedInboxEntry>(`/unified-inbox/${customerId}`)
}

export async function resolveCustomer(channel: string, identifier: string) {
  return request<UnifiedCustomer | { found: boolean }>(`/unified-inbox/resolve`, {
    method: 'POST',
    body: JSON.stringify({ channel, identifier }),
  })
}

export async function createCustomerMapping(body: {
  customerId: string
  contactId: string
  channel: string
  channelIdentifier: string
}) {
  return request<CustomerMapping>('/unified-inbox/mapping', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function getUnifiedInboxSettings(organizationId?: string) {
  const params = new URLSearchParams()
  if (organizationId) params.set('search', organizationId)
  return request<UnifiedInboxSettings>(`/unified-inbox/settings?${params}`)
}

export async function updateUnifiedInboxSettings(body: Record<string, unknown>) {
  return request<UnifiedInboxSettings>('/unified-inbox/settings', {
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

// ——— Phase 20: Workflow Builder ———

export interface WorkflowBuilderWorkflow {
  id: string
  organizationId: string
  name: string
  description?: string
  workflowType: string
  status: string
  triggerType: string
  triggerConfig?: Record<string, unknown>
  stepsJson?: Record<string, unknown>
  executionCount: number
  lastExecutedAt?: string
  createdBy?: string
  createdAt: string
  updatedAt: string
}

export interface WorkflowStep {
  id: string
  workflowId: string
  stepOrder: number
  stepType: string
  name: string
  config?: Record<string, unknown>
  conditionBranch?: string
  parentStepId?: string
  onSuccessNext?: string
  onFailureNext?: string
  createdAt: string
}

export interface WorkflowCondition {
  id: string
  workflowId: string
  stepId: string
  conditionType: string
  field: string
  operator: string
  value?: string
  logicOperator: string
  branchOnMatch: string
  createdAt: string
}

export interface WorkflowExecution {
  id: string
  workflowId: string
  triggerSource?: string
  triggerData?: Record<string, unknown>
  targetContactId?: string
  targetConversationId?: string
  currentStepId?: string
  status: string
  stepResults?: Record<string, unknown>
  errorMessage?: string
  startedAt: string
  completedAt?: string
}

export interface WorkflowTrigger {
  id: string
  workflowId: string
  triggerName: string
  triggerEvent: string
  filterConditions?: Record<string, unknown>
  isActive: boolean
  createdAt: string
}

export interface WorkflowSummary {
  totalWorkflows: number
  activeWorkflows: number
  draftWorkflows: number
  workflowsByType: Record<string, number>
  recentExecutions: WorkflowExecution[]
  totalExecutions24h: number
  failedExecutions24h: number
}

export interface ExecutionResult {
  executionId: string
  status: string
  stepsCompleted: number
  stepsTotal: number
  error?: string
}

export async function getWorkflowSummary() {
  return request<WorkflowSummary>('/workflows/summary')
}

export async function listWorkflows(
  limit = 50,
  offset = 0,
  workflowType?: string,
  status?: string,
  organizationId?: string,
) {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  params.set('offset', String(offset))
  if (workflowType) params.set('workflow_type', workflowType)
  if (status) params.set('status', status)
  if (organizationId) params.set('organization_id', organizationId)
  return request<{ workflows: WorkflowBuilderWorkflow[]; total: number }>(`/workflows?${params}`)
}

export async function getWorkflowById(id: string) {
  return request<{ workflow: WorkflowBuilderWorkflow; steps: WorkflowStep[]; conditions: WorkflowCondition[]; triggers: WorkflowTrigger[] }>(`/workflows/${id}`)
}

export async function createWorkflow(body: {
  organizationId: string
  name: string
  workflowType: string
  triggerType: string
  description?: string
  triggerConfig?: string
  createdBy?: string
}) {
  return request<WorkflowBuilderWorkflow>('/workflows', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function updateWorkflow(id: string, updates: Record<string, unknown>) {
  return request<WorkflowBuilderWorkflow>(`/workflows/${id}`, {
    method: 'PUT',
    body: JSON.stringify(updates),
  })
}

export async function deleteWorkflow(id: string) {
  return request<{ deleted: boolean }>(`/workflows/${id}`, { method: 'DELETE' })
}

export async function addWorkflowStep(workflowId: string, body: {
  stepType: string
  name: string
  stepOrder: number
  config?: string
  parentStepId?: string
  conditionBranch?: string
}) {
  return request<WorkflowStep>(`/workflows/${workflowId}/steps`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function addWorkflowCondition(workflowId: string, body: {
  stepId: string
  conditionType: string
  field: string
  operator: string
  value?: string
  logicOperator: string
}) {
  return request<WorkflowCondition>(`/workflows/${workflowId}/conditions`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function addWorkflowTrigger(workflowId: string, body: {
  triggerName: string
  triggerEvent: string
  filterConditions?: string
}) {
  return request<WorkflowTrigger>(`/workflows/${workflowId}/triggers`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function triggerWorkflowExecution(workflowId: string, body: {
  triggerSource: string
  triggerData?: string
  targetContactId?: string
  targetConversationId?: string
}) {
  return request<ExecutionResult>(`/workflows/${workflowId}/execute`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function listWorkflowExecutions(
  limit = 50,
  offset = 0,
  workflowId?: string,
  status?: string,
) {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  params.set('offset', String(offset))
  if (workflowId) params.set('workflow_id', workflowId)
  if (status) params.set('status', status)
  return request<{ executions: WorkflowExecution[]; total: number }>(`/workflows/executions?${params}`)
}

export async function getAvailableTriggers() {
  return request<{ triggers: Array<{ event: string; label: string; description: string }>; step_types: Array<{ type: string; label: string; description: string }> }>('/workflows/available-triggers')
}

// ——— Phase 21: Public Help Center ———

export interface HelpCenterCategory {
  id: string
  organizationId: string
  name: string
  slug: string
  description?: string
  icon?: string
  displayOrder: number
  isVisible: boolean
  articleCount: number
  createdAt: string
  updatedAt: string
}

export interface HelpCenterArticle {
  id: string
  organizationId: string
  categoryId: string
  title: string
  slug: string
  content: string
  authorId?: string
  status: string
  isPinned: boolean
  viewCount: number
  helpfulCount: number
  notHelpfulCount: number
  tags?: unknown
  seoTitle?: string
  seoDescription?: string
  categoryName?: string
  createdAt: string
  updatedAt: string
  publishedAt?: string
}

export interface HelpCenterSettings {
  id: string
  organizationId: string
  name: string
  tagline?: string
  logoUrl?: string
  primaryColor: string
  enableSearch: boolean
  enableArticleFeedback: boolean
  enableEmailSubscriptions: boolean
  defaultLanguage: string
  customCss?: string
  footerText?: string
  createdAt: string
  updatedAt: string
}

export interface HelpCenterSummary {
  totalArticles: number
  publishedArticles: number
  draftArticles: number
  totalCategories: number
  totalViews7d: number
  totalSearches7d: number
  avgHelpfulRate: number
  topArticles: HelpCenterArticle[]
  popularSearches: Array<{ query: string; count: number }>
}

export interface HelpCenterSearchResult {
  articles: HelpCenterArticle[]
  total: number
  query: string
}

export interface ArticleFeedbackResult {
  articleId: string
  helpful: boolean
  totalHelpful: number
  totalNotHelpful: number
}

export async function getHelpCenterSummary() {
  return request<HelpCenterSummary>('/help-center/summary')
}

export async function getPublicHelpCenter() {
  return request<unknown>('/help-center/public')
}

export async function listHelpCenterCategories(visibleOnly = true) {
  const params = new URLSearchParams()
  params.set('visible_only', String(visibleOnly))
  return request<{ categories: HelpCenterCategory[] }>(`/help-center/categories?${params}`)
}

export async function createHelpCenterCategory(body: {
  organizationId: string
  name: string
  slug: string
  description?: string
  icon?: string
  displayOrder?: number
}) {
  return request<HelpCenterCategory>('/help-center/categories', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function listHelpCenterArticles(
  limit = 50,
  offset = 0,
  categoryId?: string,
  status?: string,
) {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  params.set('offset', String(offset))
  if (categoryId) params.set('category_id', categoryId)
  if (status) params.set('status', status)
  return request<{ articles: HelpCenterArticle[] }>(`/help-center/articles?${params}`)
}

export async function createHelpCenterArticle(body: {
  organizationId: string
  categoryId: string
  title: string
  slug: string
  content: string
  authorId?: string
}) {
  return request<HelpCenterArticle>('/help-center/articles', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function updateHelpCenterArticle(id: string, updates: {
  title?: string
  content?: string
  status?: string
  isPinned?: boolean
  seoTitle?: string
  seoDescription?: string
}) {
  return request<HelpCenterArticle>(`/help-center/articles/${id}`, {
    method: 'PUT',
    body: JSON.stringify(updates),
  })
}

export async function deleteHelpCenterArticle(id: string) {
  return request<{ deleted: boolean }>(`/help-center/articles/${id}`, { method: 'DELETE' })
}

export async function searchHelpCenterArticles(q: string, limit = 20) {
  const params = new URLSearchParams()
  params.set('q', q)
  params.set('limit', String(limit))
  return request<HelpCenterSearchResult>(`/help-center/articles/search?${params}`)
}

export async function getHelpCenterArticleBySlug(slug: string) {
  return request<HelpCenterArticle>(`/help-center/articles/slug/${slug}`)
}

export async function recordArticleFeedback(articleId: string, helpful: boolean) {
  return request<ArticleFeedbackResult>(`/help-center/articles/${articleId}/feedback`, {
    method: 'POST',
    body: JSON.stringify({ helpful }),
  })
}

export async function getHelpCenterSettings() {
  return request<HelpCenterSettings>('/help-center/settings')
}

export async function updateHelpCenterSettings(updates: {
  name?: string
  tagline?: string
  primaryColor?: string
  enableSearch?: boolean
  enableArticleFeedback?: boolean
  footerText?: string
}) {
  return request<HelpCenterSettings>('/help-center/settings', {
    method: 'PUT',
    body: JSON.stringify(updates),
  })
}

// ——— Phase 22: CRM Sync ———

export interface CRMConnection {
  id: string
  organizationId: string
  provider: string
  status: string
  authType: string
  credentials: any
  config: any
  lastSyncAt: string | null
  lastError: string | null
  connectedAt: string | null
  createdAt: string
  updatedAt: string
}

export interface CRMSyncLog {
  id: string
  connectionId: string
  direction: string
  objectType: string
  recordCount: number
  successCount: number
  errorCount: number
  status: string
  errorMessage: string | null
  startedAt: string
  completedAt: string | null
}

export interface CRMFieldMapping {
  id: string
  connectionId: string
  objectType: string
  localField: string
  crmField: string
  direction: string
  transform: any
  isRequired: boolean
  createdAt: string
}

export interface CRMProviderInfo {
  provider: string
  displayName: string
  description: string
  authTypes: string[]
  supportedObjects: string[]
  setupUrl: string
  docsUrl: string
}

export interface CRMSyncSummary {
  totalConnections: number
  activeConnections: number
  disconnectedConnections: number
  providers: Array<{ provider: string; count: number }>
  recentSyncs: CRMSyncLog[]
  totalSyncs7d: number
  totalRecordsSynced7d: number
  pendingRefs: number
  conflictRefs: number
}

export interface CRMSyncResult {
  connectionId: string
  direction: string
  objectType: string
  recordCount: number
  successCount: number
  errorCount: number
  status: string
  details: any[]
}

export async function getCRMSyncSummary() {
  return request<CRMSyncSummary>('/crm-sync/summary')
}

export async function getCRMProviders() {
  return request<CRMProviderInfo[]>('/crm-sync/providers')
}

export async function listCRMConnections(provider?: string, status?: string) {
  const params = new URLSearchParams()
  if (provider) params.set('provider', provider)
  if (status) params.set('status', status)
  return request<CRMConnection[]>(`/crm-sync/connections?${params}`)
}

export async function createCRMConnection(body: {
  organizationId?: string
  provider: string
  authType?: string
  credentials?: any
  config?: any
}) {
  return request<CRMConnection>('/crm-sync/connections', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function updateCRMConnection(id: string, body: {
  status: string
  credentials?: any
  lastError?: string
}) {
  return request<CRMConnection>(`/crm-sync/connections/${id}`, {
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

export async function deleteCRMConnection(id: string) {
  return request<{ deleted: boolean }>(`/crm-sync/connections/${id}`, { method: 'DELETE' })
}

export async function triggerCRMSync(id: string, direction: string, objectType: string) {
  return request<CRMSyncResult>(`/crm-sync/connections/${id}/sync`, {
    method: 'POST',
    body: JSON.stringify({ direction, object_type: objectType }),
  })
}

export async function listSyncLogs(connectionId?: string, limit = 50) {
  const params = new URLSearchParams()
  if (connectionId) params.set('connection_id', connectionId)
  params.set('limit', String(limit))
  return request<CRMSyncLog[]>(`/crm-sync/logs?${params}`)
}

export async function listFieldMappings(connectionId?: string, objectType?: string) {
  const params = new URLSearchParams()
  if (connectionId) params.set('connection_id', connectionId)
  if (objectType) params.set('object_type', objectType)
  return request<CRMFieldMapping[]>(`/crm-sync/mappings?${params}`)
}

// ——— Phase 23: KB Document Upload & URL Crawling ———

export interface KBDocument {
  id: string
  organizationId: string
  fileName: string
  fileType: string
  fileSize: number
  fileHash: string | null
  storagePath: string | null
  status: string
  errorMessage: string | null
  pagesCount: number | null
  chunksCount: number | null
  createdAt: string
  updatedAt: string
}

export interface KBDocumentChunk {
  id: string
  documentId: string
  chunkIndex: number
  title: string
  content: string
  pageNumber: number | null
  kbEntryId: string | null
  createdAt: string
}

export interface KBURLSource {
  id: string
  organizationId: string
  url: string
  title: string | null
  status: string
  errorMessage: string | null
  contentLength: number | null
  chunksCount: number | null
  lastCrawledAt: string | null
  nextCrawlAt: string | null
  crawlIntervalHours: number
  createdAt: string
  updatedAt: string
}

export interface KBURLCrawlLog {
  id: string
  urlSourceId: string
  statusCode: number | null
  contentLength: number | null
  chunksCreated: number
  status: string
  errorMessage: string | null
  startedAt: string
  completedAt: string | null
}

export interface KBImportSettings {
  id: string
  organizationId: string
  maxFileSizeMb: number
  allowedFileTypes: string[]
  chunkSize: number
  chunkOverlap: number
  autoCreateKbEntries: boolean
  crawlIntervalHours: number
  maxUrlDepth: number
  userAgent: string
  createdAt: string
  updatedAt: string
}

export interface KBImportSummary {
  totalDocuments: number
  readyDocuments: number
  processingDocuments: number
  errorDocuments: number
  totalUrlSources: number
  readyUrlSources: number
  pendingUrlSources: number
  totalChunks: number
  totalKbEntriesCreated: number
  recentUploads: KBDocument[]
  recentCrawls: KBURLCrawlLog[]
}

export interface DocumentUploadResult {
  documentId: string
  fileName: string
  chunksCreated: number
  kbEntriesCreated: number
  status: string
  errorMessage: string | null
}

export interface URLCrawlResult {
  urlSourceId: string
  url: string
  title: string | null
  chunksCreated: number
  kbEntriesCreated: number
  status: string
  errorMessage: string | null
}

export async function getKBImportSummary() {
  return request<KBImportSummary>('/kb-import/summary')
}

export async function uploadDocument(body: {
  organizationId?: string
  fileName: string
  fileType: string
  fileData: string
}) {
  return request<DocumentUploadResult>('/kb-import/upload', {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export async function listDocuments(organizationId?: string, status?: string) {
  const params = new URLSearchParams()
  if (organizationId) params.set('organization_id', organizationId)
  if (status) params.set('status', status)
  return request<KBDocument[]>(`/kb-import/documents?${params}`)
}

export async function getDocumentChunks(documentId: string) {
  return request<KBDocumentChunk[]>(`/kb-import/documents/${documentId}/chunks`)
}

export async function deleteDocument(documentId: string) {
  return request<{ deleted: boolean }>(`/kb-import/documents/${documentId}`, { method: 'DELETE' })
}

export async function addURLSource(body: {
  organizationId?: string
  url: string
  crawlIntervalHours?: number
}) {
  return request<KBURLSource>('/kb-import/urls', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      url: body.url,
      crawl_interval_hours: body.crawlIntervalHours,
    }),
  })
}

export async function crawlURL(sourceId: string) {
  return request<URLCrawlResult>(`/kb-import/urls/${sourceId}/crawl`, { method: 'POST' })
}

export async function listURLSources(organizationId?: string, status?: string) {
  const params = new URLSearchParams()
  if (organizationId) params.set('organization_id', organizationId)
  if (status) params.set('status', status)
  return request<KBURLSource[]>(`/kb-import/urls?${params}`)
}

export async function deleteURLSource(sourceId: string) {
  return request<{ deleted: boolean }>(`/kb-import/urls/${sourceId}`, { method: 'DELETE' })
}

export async function getKBImportSettings(organizationId?: string) {
  const params = new URLSearchParams()
  if (organizationId) params.set('organization_id', organizationId)
  return request<KBImportSettings>(`/kb-import/settings?${params}`)
}

export async function updateKBImportSettings(body: {
  organizationId?: string
  settings: any
}) {
  return request<KBImportSettings>('/kb-import/settings', {
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

// --- Commerce Gaps API Functions ---

// Gap #1: Product Options
export async function listProductOptions(productId: string) {
  return request<any[]>(`/commerce/products/${productId}/options`)
}
export async function getProductOption(id: string) {
  return request<any>(`/commerce/options/${id}`)
}
export async function createProductOption(data: any) {
  return request<any>('/commerce/products/options', { method: 'POST', body: JSON.stringify(data) })
}
export async function updateProductOption(id: string, data: any) {
  return request<any>(`/commerce/options/${id}`, { method: 'PUT', body: JSON.stringify(data) })
}
export async function deleteProductOption(id: string) {
  return request<void>(`/commerce/options/${id}`, { method: 'DELETE' })
}
export async function createProductOptionValue(optionId: string, data: any) {
  return request<any>(`/commerce/options/${optionId}/values`, { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteProductOptionValue(id: string) {
  return request<void>(`/commerce/options/values/${id}`, { method: 'DELETE' })
}

// Gap #2: Delivery Fee Rules
export async function listDeliveryFeeRules(brandId: string) {
  return request<any[]>(`/commerce/delivery-fee-rules?brand_id=${brandId}`)
}
export async function createDeliveryFeeRule(data: any) {
  return request<any>('/commerce/delivery-fee-rules', { method: 'POST', body: JSON.stringify(data) })
}
export async function updateDeliveryFeeRule(id: string, data: any) {
  return request<any>(`/commerce/delivery-fee-rules/${id}`, { method: 'PUT', body: JSON.stringify(data) })
}
export async function deleteDeliveryFeeRule(id: string) {
  return request<void>(`/commerce/delivery-fee-rules/${id}`, { method: 'DELETE' })
}

// Gap #3: Payment Proofs
export async function uploadPaymentProof(data: any) {
  return request<any>('/commerce/payment-proofs', { method: 'POST', body: JSON.stringify(data) })
}
export async function listPaymentProofs() {
  return request<any[]>('/commerce/payment-proofs')
}
export async function verifyPaymentProof(id: string, data: any) {
  return request<any>(`/commerce/payment-proofs/${id}/verify`, { method: 'POST', body: JSON.stringify(data) })
}

// Gap #4: Order Notification Templates
export async function listNotificationTemplates(brandId: string) {
  return request<any[]>(`/commerce/notification-templates?brand_id=${brandId}`)
}
export async function createNotificationTemplate(data: any) {
  return request<any>('/commerce/notification-templates', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteNotificationTemplate(id: string) {
  return request<void>(`/commerce/notification-templates/${id}`, { method: 'DELETE' })
}

// Gap #6: Payment Gateways
export async function listPaymentGateways(brandId: string) {
  return request<any[]>(`/commerce/payment-gateways?brand_id=${brandId}`)
}
export async function createPaymentGateway(data: any) {
  return request<any>('/commerce/payment-gateways', { method: 'POST', body: JSON.stringify(data) })
}
export async function deletePaymentGateway(id: string) {
  return request<void>(`/commerce/payment-gateways/${id}`, { method: 'DELETE' })
}

// Gap #7: Service Types
export async function listServiceTypes(brandId: string) {
  return request<any[]>(`/commerce/service-types?brand_id=${brandId}`)
}
export async function createServiceType(data: any) {
  return request<any>('/commerce/service-types', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteServiceType(id: string) {
  return request<void>(`/commerce/service-types/${id}`, { method: 'DELETE' })
}

// Gap #8: Delivery Schedules & Time Slots
export async function listDeliverySchedules(brandId: string) {
  return request<any[]>(`/commerce/delivery-schedules?brand_id=${brandId}`)
}
export async function createDeliverySchedule(data: any) {
  return request<any>('/commerce/delivery-schedules', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteDeliverySchedule(id: string) {
  return request<void>(`/commerce/delivery-schedules/${id}`, { method: 'DELETE' })
}
export async function listTimeSlots(brandId: string) {
  return request<any[]>(`/commerce/time-slots?brand_id=${brandId}`)
}
export async function createTimeSlot(data: any) {
  return request<any>('/commerce/time-slots', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteTimeSlot(id: string) {
  return request<void>(`/commerce/time-slots/${id}`, { method: 'DELETE' })
}

// Gap #9: Invoices
export async function generateInvoice(orderId: string) {
  return request<any>(`/commerce/orders/${orderId}/invoice`, { method: 'POST' })
}

// Gap #10: CSV Import
export async function importCsv(data: any) {
  return request<any>('/commerce/csv/import', { method: 'POST', body: JSON.stringify(data) })
}
export async function listCsvImportJobs(brandId: string) {
  return request<any[]>(`/commerce/csv/jobs?brand_id=${brandId}`)
}

// Gap #11: Customer Accounts
export async function createCustomerAccount(customerId: string, data: any) {
  return request<any>(`/commerce/customers/${customerId}/account`, { method: 'POST', body: JSON.stringify(data) })
}
export async function customerLogin(data: any) {
  return request<any>('/commerce/customer/login', { method: 'POST', body: JSON.stringify(data) })
}

// Gap #12: Customer Reviews
export async function createReview(data: any) {
  return request<any>('/commerce/reviews', { method: 'POST', body: JSON.stringify(data) })
}
export async function listReviews() {
  return request<any[]>('/commerce/reviews')
}

// Gap #13: Loyalty Programs
export async function listLoyaltyPrograms(brandId: string) {
  return request<any[]>(`/commerce/loyalty?brand_id=${brandId}`)
}
export async function createLoyaltyProgram(data: any) {
  return request<any>('/commerce/loyalty', { method: 'POST', body: JSON.stringify(data) })
}
export async function getCustomerPoints(customerId: string, brandId: string) {
  return request<any>(`/commerce/loyalty/points?customer_id=${customerId}&brand_id=${brandId}`)
}

// Gap #16: Staff Management
export async function listStaffMembers(brandId: string) {
  return request<any[]>(`/commerce/staff?brand_id=${brandId}`)
}
export async function createStaffMember(data: any) {
  return request<any>('/commerce/staff', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteStaffMember(id: string) {
  return request<void>(`/commerce/staff/${id}`, { method: 'DELETE' })
}

// Gap #17: POS Sessions
export async function createPosSession(brandId: string) {
  return request<any>(`/commerce/pos/sessions?brand_id=${brandId}`, { method: 'POST' })
}
export async function endPosSession(id: string) {
  return request<void>(`/commerce/pos/sessions/${id}/end`, { method: 'POST' })
}
export async function listPosSessions(brandId: string) {
  return request<any[]>(`/commerce/pos/sessions?brand_id=${brandId}`)
}

// Gap #18: Receipts
export async function generateReceipt(orderId: string) {
  return request<any>(`/commerce/orders/${orderId}/receipt`, { method: 'POST' })
}

// Gap #20: Wholesale Pricing
export async function listWholesalePricing(productId: string) {
  return request<any[]>(`/commerce/products/${productId}/wholesale`)
}
export async function createWholesalePricing(data: any) {
  return request<any>('/commerce/products/wholesale', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteWholesalePricing(id: string) {
  return request<void>(`/commerce/wholesale/${id}`, { method: 'DELETE' })
}

// Gap #23: Automated Follow-ups
export async function listFollowups(brandId: string) {
  return request<any[]>(`/commerce/followups?brand_id=${brandId}`)
}
export async function createFollowup(data: any) {
  return request<any>('/commerce/followups', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteFollowup(id: string) {
  return request<void>(`/commerce/followups/${id}`, { method: 'DELETE' })
}

// Gap #24: Custom Domains
export async function listCustomDomains(brandId: string) {
  return request<any[]>(`/commerce/custom-domains?brand_id=${brandId}`)
}
export async function createCustomDomain(data: any) {
  return request<any>('/commerce/custom-domains', { method: 'POST', body: JSON.stringify(data) })
}
export async function verifyCustomDomain(id: string) {
  return request<any>(`/commerce/custom-domains/${id}/verify`, { method: 'POST' })
}
export async function deleteCustomDomain(id: string) {
  return request<void>(`/commerce/custom-domains/${id}`, { method: 'DELETE' })
}

// Gap #25: Storefront Pages
export async function listStorefrontPages(storefrontId: string) {
  return request<any[]>(`/commerce/storefronts/${storefrontId}/pages`)
}
export async function createStorefrontPage(data: any) {
  return request<any>('/commerce/storefronts/pages', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteStorefrontPage(id: string) {
  return request<void>(`/commerce/storefronts/pages/${id}`, { method: 'DELETE' })
}

// Gap #27: Meta Pixels
export async function listMetaPixels(brandId: string) {
  return request<any[]>(`/commerce/meta-pixels?brand_id=${brandId}`)
}
export async function createMetaPixel(data: any) {
  return request<any>('/commerce/meta-pixels', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteMetaPixel(id: string) {
  return request<void>(`/commerce/meta-pixels/${id}`, { method: 'DELETE' })
}

// Gap #29: Checkout Questions
export async function listCheckoutQuestions(brandId: string) {
  return request<any[]>(`/commerce/checkout-questions?brand_id=${brandId}`)
}
export async function createCheckoutQuestion(data: any) {
  return request<any>('/commerce/checkout-questions', { method: 'POST', body: JSON.stringify(data) })
}
export async function submitCheckoutAnswers(data: any) {
  return request<any[]>('/commerce/checkout/answers', { method: 'POST', body: JSON.stringify(data) })
}
export async function deleteCheckoutQuestion(id: string) {
  return request<void>(`/commerce/checkout-questions/${id}`, { method: 'DELETE' })
}

// Gap #30: Delivery Proofs
export async function uploadDeliveryProof(data: any) {
  return request<any>('/commerce/delivery-proofs', { method: 'POST', body: JSON.stringify(data) })
}
export async function listDeliveryProofs(orderId: string) {
  return request<any[]>(`/commerce/orders/${orderId}/delivery-proofs`)
}

// ============================================================================
// nID Sync System API
// ============================================================================

export interface NidProfile {
  id: string
  nidProfileId: string
  nidAccountId?: string
  nidOrganizationId?: string
  slug: string
  displayName: string
  headline?: string
  category: string
  bio?: string
  photoKey?: string
  phone?: string
  whatsapp?: string
  publicEmail?: string
  city?: string
  countryCode?: string
  timezone: string
  isPublished: boolean
  isPersonal: boolean
}

export interface NidOrganization {
  id: string
  communicatorOrgId: string
  nidOrgId: string
  nidTenantId: string
  nidLegalName: string
  nidRegistrationNumber?: string
  nidCountryCode?: string
  isVerified: boolean
  verifiedLevel?: string
  verifiedAt?: string
}

export interface NidMembership {
  id: string
  teamMemberId: string
  nidOrgId: string
  nidMembershipId: string
  role?: string
  isAdmin: boolean
  status: string
}

export interface NidTenantMapping {
  id: string
  communicatorTenantKey: string
  nidTenantId: string
  nidTenantName: string
  nidTenantSlug: string
  isActive: boolean
}

export interface NidInvoice {
  id: string
  nidInvoiceId: string
  nidTenantId: string
  invoiceNumber: string
  status: string
  currency: string
  subtotalMinor: number
  taxMinor: number
  totalMinor: number
  amountPaidMinor: number
  periodStart?: string
  periodEnd?: string
  dueAt?: string
  issuedAt?: string
  paidAt?: string
}

export interface NidPayment {
  id: string
  nidPaymentId: string
  nidTenantId: string
  provider: string
  providerTransactionId: string
  amountMinor: number
  currency: string
  status: string
  methodType?: string
  paidAt?: string
}

export interface NidSubscription {
  id: string
  nidSubscriptionId: string
  nidTenantId: string
  planKey: string
  planName: string
  planMode: string
  status: string
  currentPeriodStart?: string
  currentPeriodEnd?: string
  cancelAtPeriodEnd: boolean
}

export interface BrandSummary {
  id: string
  name: string
  slug: string
  totalOrders: number
  totalSales: number
  totalCustomers: number
}

export interface PersonalOrder {
  id: string
  orderNumber: string
  status: string
  totalAmount: number
  currency: string
  platformName: string
  platformType: string
  brandName: string
  shopName?: string
  platformLogoUrl?: string
  createdAt: string
}

export interface LaunchpadOverview {
  personalProfile?: NidProfile
  workProfiles: NidProfile[]
  organizations: NidOrganization[]
  memberships: NidMembership[]
  tenants: NidTenantMapping[]
  totalOrders: number
  totalPersonalOrders: number
  totalInvoices: number
  totalPayments: number
  activeSubscriptions: number
}

export interface WorkProfileOverview {
  organization: NidOrganization
  brands: BrandSummary[]
  totalCustomers: number
  totalOrders: number
  totalSales: number
  totalContacts: number
  activeSubscriptions: number
}

export interface PersonalProfileOverview {
  profile: NidProfile
  orders: PersonalOrder[]
  invoices: NidInvoice[]
  payments: NidPayment[]
  subscriptions: NidSubscription[]
  totalSpent: number
}

export interface NidAccountLink {
  id: string
  teamMemberId: string
  nidAccountId: string
  nidPhone: string
  nidEmail?: string
  nidProfileId?: string
  nidProfileSlug?: string
  nidProfileDisplayName?: string
  nidProfilePhotoKey?: string
  syncedAt: string
}

export async function getLaunchpadOverview(teamMemberId: string): Promise<LaunchpadOverview> {
  return request<LaunchpadOverview>(`/nid-sync/launchpad?team_member_id=${encodeURIComponent(teamMemberId)}`)
}

export async function getAccountLink(teamMemberId: string): Promise<NidAccountLink | null> {
  return request<NidAccountLink | null>(`/nid-sync/account-link/${encodeURIComponent(teamMemberId)}`)
}

export async function linkAccount(
  teamMemberId: string,
  data: {
    nidAccountId: string
    nidPhone: string
    nidEmail?: string
    nidProfileSlug?: string
    nidProfileDisplayName?: string
  }
): Promise<NidAccountLink> {
  return request<NidAccountLink>(`/nid-sync/account-link/${encodeURIComponent(teamMemberId)}`, {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

export async function getPersonalProfile(nidAccountId: string): Promise<NidProfile | null> {
  return request<NidProfile | null>(`/nid-sync/personal-profile?nid_account_id=${encodeURIComponent(nidAccountId)}`)
}

export async function getPersonalProfileOverview(nidAccountId: string): Promise<PersonalProfileOverview> {
  return request<PersonalProfileOverview>(`/nid-sync/personal-profile/overview?nid_account_id=${encodeURIComponent(nidAccountId)}`)
}

export async function getWorkProfiles(teamMemberId: string): Promise<NidProfile[]> {
  return request<NidProfile[]>(`/nid-sync/work-profiles?team_member_id=${encodeURIComponent(teamMemberId)}`)
}

export async function getWorkProfileOverview(
  teamMemberId: string,
  orgId: string
): Promise<WorkProfileOverview> {
  return request<WorkProfileOverview>(`/nid-sync/work-profile/${encodeURIComponent(teamMemberId)}/${encodeURIComponent(orgId)}`)
}

export async function getUserOrganizations(teamMemberId: string): Promise<NidOrganization[]> {
  return request<NidOrganization[]>(`/nid-sync/organizations?team_member_id=${encodeURIComponent(teamMemberId)}`)
}

export async function getUserMemberships(teamMemberId: string): Promise<NidMembership[]> {
  return request<NidMembership[]>(`/nid-sync/memberships?team_member_id=${encodeURIComponent(teamMemberId)}`)
}

export async function getInvoices(limit?: number): Promise<NidInvoice[]> {
  return request<NidInvoice[]>(`/nid-sync/invoices${limit ? `?limit=${limit}` : ''}`)
}

export async function getInvoiceByNumber(invoiceNumber: string): Promise<NidInvoice | null> {
  return request<NidInvoice | null>(`/nid-sync/invoices/${encodeURIComponent(invoiceNumber)}`)
}

export async function getPayments(limit?: number): Promise<NidPayment[]> {
  return request<NidPayment[]>(`/nid-sync/payments${limit ? `?limit=${limit}` : ''}`)
}

export async function getSubscriptions(): Promise<NidSubscription[]> {
  return request<NidSubscription[]>('/nid-sync/subscriptions')
}

export async function setOrderPlatform(data: {
  orderId: string
  platformName: string
  platformType: string
  brandId?: string
  brandName: string
  shopName?: string
  platformLogoUrl?: string
}): Promise<void> {
  return request<void>('/nid-sync/order-platform', { method: 'POST', body: JSON.stringify(data) })
}

export async function cacheProfile(profile: NidProfile): Promise<void> {
  return request<void>('/nid-sync/cache-profile', { method: 'POST', body: JSON.stringify(profile) })
}

// --- Fin Vision API Types and Functions (Phase 25) ---

export interface VisionImage {
  id: string
  organizationId: string
  conversationId?: string
  contactId?: string
  channel?: string
  imageUrl: string
  imageData?: string
  mimeType: string
  fileSize: number
  status: string
  createdAt: string
  updatedAt: string
}

export interface VisionAnalysis {
  id: string
  imageId: string
  analysisType: string
  prompt?: string
  result: string
  confidence: number
  objectsDetected: string
  textExtracted?: string
  summary?: string
  processingTimeMs: number
  claudeModel: string
  errorMessage?: string
  createdAt: string
}

export interface VisionSettings {
  id: string
  organizationId: string
  enabled: boolean
  autoAnalyze: boolean
  defaultAnalysisType: string
  claudeModel: string
  maxFileSizeMb: number
  allowedMimeTypes: string
  ocrEnabled: boolean
  objectDetectionEnabled: boolean
  createdAt: string
  updatedAt: string
}

export interface VisionSummary {
  totalImages: number
  totalAnalyses: number
  avgConfidence: number
  avgProcessingTimeMs: number
  analysisTypeBreakdown: { analysisType: string; count: number }[]
  recentAnalyses: VisionAnalysis[]
  enabled: boolean
  autoAnalyze: boolean
}

export interface UploadResult {
  imageId: string
  status: string
  analysisId?: string
  message: string
}

export interface AnalyzeResult {
  analysisId: string
  imageId: string
  analysisType: string
  result: any
  confidence: number
  objectsDetected: string[]
  textExtracted?: string
  summary?: string
  processingTimeMs: number
  claudeModel: string
}

export async function getVisionSummary(): Promise<VisionSummary> {
  return request<VisionSummary>('/vision/summary')
}

export async function uploadVisionImage(body: {
  organizationId?: string
  imageData: string
  mimeType: string
  fileSize: number
  conversationId?: string
  contactId?: string
  channel?: string
}): Promise<UploadResult> {
  return request<UploadResult>('/vision/upload', { method: 'POST', body: JSON.stringify(body) })
}

export async function analyzeVisionImage(
  imageId: string,
  body: { analysisType?: string; prompt?: string; claudeModel?: string }
): Promise<AnalyzeResult> {
  return request<AnalyzeResult>(`/vision/analyze/${imageId}`, { method: 'POST', body: JSON.stringify(body) })
}

export async function listVisionImages(status?: string, limit?: number): Promise<VisionImage[]> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (limit) params.set('limit', String(limit))
  return request<VisionImage[]>(`/vision/images?${params}`)
}

export async function getVisionImage(imageId: string): Promise<{ image: VisionImage; analyses: VisionAnalysis[] }> {
  return request<{ image: VisionImage; analyses: VisionAnalysis[] }>(`/vision/images/${imageId}`)
}

export async function deleteVisionImage(imageId: string): Promise<{ success: boolean }> {
  return request<{ success: boolean }>(`/vision/images/${imageId}`, { method: 'DELETE' })
}

export async function getVisionSettings(): Promise<VisionSettings> {
  return request<VisionSettings>('/vision/settings')
}

export async function updateVisionSettings(body: {
  organizationId?: string
  enabled?: boolean
  autoAnalyze?: boolean
  defaultAnalysisType?: string
  claudeModel?: string
  maxFileSizeMb?: number
  allowedMimeTypes?: string[]
  ocrEnabled?: boolean
  objectDetectionEnabled?: boolean
}): Promise<VisionSettings> {
  return request<VisionSettings>('/vision/settings', { method: 'PUT', body: JSON.stringify(body) })
}

// --- Audience-Based Knowledge Scoping API Types and Functions (Phase 26) ---

export interface AudienceKBLink {
  id: string
  segmentId: string
  segmentName?: string
  kbEntryId: string
  kbTitle?: string
  priority: number
  createdAt: string
}

export interface AudienceKBSettings {
  id: string
  organizationId: string
  enabled: boolean
  fallbackToGlobalKb: boolean
  mergeSegments: boolean
  defaultScope: string
  createdAt: string
  updatedAt: string
}

export interface SegmentContactMembership {
  id: string
  segmentId: string
  segmentName?: string
  contactId: string
  contactName?: string
  membershipType: string
  matchedAt: string
}

export interface AudienceKBUsageLog {
  id: string
  conversationId: string
  contactId: string
  segmentId: string
  kbEntryId: string
  queryText: string
  relevanceScore: number
  usedInResponse: boolean
  createdAt: string
}

export interface AudienceSummary {
  totalSegmentsWithKb: number
  totalLinks: number
  totalMemberships: number
  totalUsage7d: number
  avgRelevanceScore: number
  enabled: boolean
  topSegments: { segmentName: string; linkCount: number }[]
  topKbEntries: { kbTitle: string; usageCount: number }[]
}

export interface ScopedKBResult {
  kbEntryIds: string[]
  segmentIds: string[]
  fallbackIncluded: boolean
  totalEntries: number
}

export interface LinkResult {
  linkId: string
  segmentId: string
  kbEntryId: string
  message: string
}

export interface MembershipResult {
  membershipId: string
  segmentId: string
  contactId: string
  message: string
}

export async function getAudienceSummary(): Promise<AudienceSummary> {
  return request<AudienceSummary>('/audience-kb/summary')
}

export async function getAudienceSettings(): Promise<AudienceKBSettings> {
  return request<AudienceKBSettings>('/audience-kb/settings')
}

export async function updateAudienceSettings(body: {
  enabled?: boolean
  fallbackToGlobalKb?: boolean
  mergeSegments?: boolean
  defaultScope?: string
}): Promise<AudienceKBSettings> {
  return request<AudienceKBSettings>('/audience-kb/settings', { method: 'PUT', body: JSON.stringify(body) })
}

export async function listAudienceLinks(segmentId?: string): Promise<{ links: AudienceKBLink[]; segmentId?: string }> {
  const params = new URLSearchParams()
  if (segmentId) params.set('segment_id', segmentId)
  return request<{ links: AudienceKBLink[]; segmentId?: string }>(`/audience-kb/links?${params}`)
}

export async function linkKbToSegment(body: {
  segmentId: string
  kbEntryId: string
  priority?: number
}): Promise<LinkResult> {
  return request<LinkResult>('/audience-kb/links', { method: 'POST', body: JSON.stringify(body) })
}

export async function unlinkKbFromSegment(segmentId: string, kbEntryId: string): Promise<{ message: string }> {
  return request<{ message: string }>(`/audience-kb/links/${segmentId}/${kbEntryId}`, { method: 'DELETE' })
}

export async function getScopedKbForContact(contactId: string): Promise<ScopedKBResult> {
  return request<ScopedKBResult>(`/audience-kb/scoped?contact_id=${contactId}`)
}

export async function listAudienceMemberships(
  segmentId?: string,
  contactId?: string,
  limit?: number
): Promise<{ memberships: SegmentContactMembership[]; segmentId?: string; contactId?: string }> {
  const params = new URLSearchParams()
  if (segmentId) params.set('segment_id', segmentId)
  if (contactId) params.set('contact_id', contactId)
  if (limit) params.set('limit', String(limit))
  return request<{ memberships: SegmentContactMembership[]; segmentId?: string; contactId?: string }>(
    `/audience-kb/memberships?${params}`
  )
}

export async function addContactToSegment(body: {
  segmentId: string
  contactId: string
  membershipType?: string
}): Promise<MembershipResult> {
  return request<MembershipResult>('/audience-kb/memberships', { method: 'POST', body: JSON.stringify(body) })
}

export async function removeContactFromSegment(
  segmentId: string,
  contactId: string
): Promise<{ message: string }> {
  return request<{ message: string }>(`/audience-kb/memberships/${segmentId}/${contactId}`, { method: 'DELETE' })
}

export async function logKbUsage(body: {
  conversationId: string
  contactId: string
  segmentId: string
  kbEntryId: string
  queryText?: string
  relevanceScore?: number
  usedInResponse?: boolean
}): Promise<{ message: string }> {
  return request<{ message: string }>('/audience-kb/usage-log', { method: 'POST', body: JSON.stringify(body) })
}

export async function getAudienceUsageLogs(
  segmentId: string,
  limit?: number
): Promise<{ logs: AudienceKBUsageLog[]; segmentId: string }> {
  const params = new URLSearchParams()
  if (limit) params.set('limit', String(limit))
  return request<{ logs: AudienceKBUsageLog[]; segmentId: string }>(`/audience-kb/usage/${segmentId}?${params}`)
}

// --- Brand Overrides in Workflows API Types and Functions (Phase 27) ---

export interface BrandOverrideRule {
  id: string
  organizationId: string
  name: string
  description?: string
  triggerType: string
  triggerConfig?: Record<string, unknown>
  fromBrandId?: string
  fromBrandName?: string
  toBrandId: string
  toBrandName?: string
  conditions?: Record<string, unknown>
  isActive: boolean
  autoSwitchKb: boolean
  autoSwitchAiConfig: boolean
  autoSwitchMessenger: boolean
  createdAt: string
  updatedAt: string
}

export interface BrandSwitchLog {
  id: string
  conversationId: string
  contactId: string
  fromBrandId: string
  fromBrandName?: string
  toBrandId: string
  toBrandName?: string
  triggerType: string
  triggerSource?: string
  ruleId?: string
  switchedBy?: string
  switchedAt: string
}

export interface BrandOverrideSettings {
  id: string
  organizationId: string
  enabled: boolean
  allowCrossOrgBrands: boolean
  requireApproval: boolean
  logAllSwitches: boolean
  defaultFallbackBrandId?: string
  createdAt: string
  updatedAt: string
}

export interface BrandHierarchy {
  id: string
  parentBrandId: string
  parentBrandName?: string
  subBrandId: string
  subBrandName?: string
  relationshipType: string
  priority: number
  createdAt: string
}

export interface BrandOverrideSummary {
  totalRules: number
  activeRules: number
  totalSwitches7d: number
  totalSwitches30d: number
  enabled: boolean
  topSwitchedBrands: { brandName: string; switchCount: number }[]
  recentSwitches: BrandSwitchLog[]
  hierarchyCount: number
}

export interface SwitchBrandResult {
  switchId: string
  conversationId: string
  fromBrandId: string
  toBrandId: string
  message: string
}

export async function getBrandOverrideSummary(): Promise<BrandOverrideSummary> {
  return request<BrandOverrideSummary>('/brand-overrides/summary')
}

export async function listBrandOverrideRules(
  organizationId?: string,
  isActive?: boolean
): Promise<BrandOverrideRule[]> {
  const params = new URLSearchParams()
  if (organizationId) params.set('organization_id', organizationId)
  if (isActive !== undefined) params.set('is_active', String(isActive))
  return request<BrandOverrideRule[]>(`/brand-overrides/rules?${params}`)
}

export async function createBrandOverrideRule(body: {
  organizationId: string
  name: string
  triggerType?: string
  fromBrandId?: string
  toBrandId: string
  autoSwitchKb?: boolean
  autoSwitchAiConfig?: boolean
  autoSwitchMessenger?: boolean
}): Promise<{ ruleId: string; name: string; message: string }> {
  return request<{ ruleId: string; name: string; message: string }>('/brand-overrides/rules', {
    method: 'POST',
    body: JSON.stringify(body)
  })
}

export async function updateBrandOverrideRule(
  ruleId: string,
  body: { name?: string; is_active?: boolean; to_brand_id?: string }
): Promise<{ message: string }> {
  return request<{ message: string }>(`/brand-overrides/rules/${ruleId}`, {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

export async function deleteBrandOverrideRule(ruleId: string): Promise<{ message: string }> {
  return request<{ message: string }>(`/brand-overrides/rules/${ruleId}`, { method: 'DELETE' })
}

export async function switchBrand(body: {
  conversationId: string
  contactId: string
  fromBrandId: string
  toBrandId: string
  triggerType?: string
  triggerSource?: string
  ruleId?: string
  switchedBy?: string
}): Promise<SwitchBrandResult> {
  return request<SwitchBrandResult>('/brand-overrides/switch', { method: 'POST', body: JSON.stringify(body) })
}

export async function getSwitchLog(conversationId: string): Promise<BrandSwitchLog[]> {
  return request<BrandSwitchLog[]>(`/brand-overrides/switch-log/${conversationId}`)
}

export async function getBrandOverrideSettings(): Promise<BrandOverrideSettings> {
  return request<BrandOverrideSettings>('/brand-overrides/settings')
}

export async function updateBrandOverrideSettings(body: {
  enabled?: boolean
  allowCrossOrgBrands?: boolean
  requireApproval?: boolean
  logAllSwitches?: boolean
}): Promise<BrandOverrideSettings> {
  return request<BrandOverrideSettings>('/brand-overrides/settings', {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

export async function listBrandHierarchy(): Promise<BrandHierarchy[]> {
  return request<BrandHierarchy[]>('/brand-overrides/hierarchy')
}

export async function addBrandHierarchy(body: {
  parentBrandId: string
  subBrandId: string
  relationshipType?: string
  priority?: number
}): Promise<{ message: string }> {
  return request<{ message: string }>('/brand-overrides/hierarchy', { method: 'POST', body: JSON.stringify(body) })
}

export async function removeBrandHierarchy(body: {
  parentBrandId: string
  subBrandId: string
}): Promise<{ message: string }> {
  return request<{ message: string }>('/brand-overrides/hierarchy/remove', { method: 'DELETE', body: JSON.stringify(body) })
}

export async function getSubBrands(parentBrandId: string): Promise<BrandHierarchy[]> {
  return request<BrandHierarchy[]>(`/brand-overrides/sub-brands/${parentBrandId}`)
}

// --- MCP Protocol Support API Types and Functions (Phase 28) ---

export interface MCPServer {
  id: string
  organizationId: string
  name: string
  description?: string
  serverUrl: string
  transportType: string
  headers?: Record<string, any>
  environmentVars?: Record<string, any>
  isActive: boolean
  connectionStatus: string
  lastConnectedAt?: string
  toolCount?: number
  createdAt: string
  updatedAt: string
}

export interface MCPTool {
  id: string
  serverId: string
  serverName?: string
  toolName: string
  description?: string
  inputSchema?: Record<string, any>
  isEnabled: boolean
  aiAccessible: boolean
  callCount?: number
  createdAt: string
  updatedAt: string
}

export interface MCPCallLog {
  id: string
  serverId: string
  serverName?: string
  toolId: string
  toolName: string
  conversationId?: string
  contactId?: string
  inputData?: Record<string, any>
  outputData?: Record<string, any>
  status: string
  errorMessage?: string
  durationMs?: number
  tokensUsed: number
  calledBy: string
  calledAt: string
  completedAt?: string
}

export interface MCPServerTemplate {
  id: string
  provider: string
  name: string
  description?: string
  defaultUrl: string
  transportType: string
  requiredEnvVars?: string[]
  defaultTools?: string[]
  iconUrl?: string
  documentationUrl?: string
  isActive: boolean
  createdAt: string
}

export interface MCPSettings {
  id: string
  organizationId: string
  enabled: boolean
  autoDiscoverTools: boolean
  logAllCalls: boolean
  maxConcurrentCalls: number
  timeoutSeconds: number
  allowedTools?: string[]
  blockedTools?: string[]
  createdAt: string
  updatedAt: string
}

export interface MCPSummary {
  totalServers: number
  connectedServers: number
  totalTools: number
  enabledTools: number
  activeSessions: number
  totalCalls7d: number
  totalCalls30d: number
  enabled: boolean
  topCalledTools: { toolName: string; callCount: number }[]
  recentCalls: MCPCallLog[]
  availableTemplates: number
}

export async function getMCPSummary(): Promise<MCPSummary> {
  return request<MCPSummary>('/mcp/summary')
}

export async function listMCPServers(
  organizationId?: string,
  isActive?: boolean
): Promise<MCPServer[]> {
  const params = new URLSearchParams()
  if (organizationId) params.set('organization_id', organizationId)
  if (isActive !== undefined) params.set('is_active', String(isActive))
  return request<MCPServer[]>(`/mcp/servers?${params}`)
}

export async function createMCPServer(body: {
  organizationId: string
  name: string
  serverUrl: string
  transportType?: string
}): Promise<{ serverId: string; name: string; message: string }> {
  return request<{ serverId: string; name: string; message: string }>('/mcp/servers', {
    method: 'POST',
    body: JSON.stringify(body)
  })
}

export async function updateMCPServer(
  serverId: string,
  body: { name?: string; is_active?: boolean; server_url?: string }
): Promise<void> {
  return request<void>(`/mcp/servers/${serverId}`, {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

export async function deleteMCPServer(serverId: string): Promise<void> {
  return request<void>(`/mcp/servers/${serverId}`, { method: 'DELETE' })
}

export async function discoverMCPTools(serverId: string): Promise<{
  serverId: string
  toolsDiscovered: number
  tools: string[]
  message: string
}> {
  return request<{ serverId: string; toolsDiscovered: number; tools: string[]; message: string }>(
    `/mcp/servers/${serverId}/discover`,
    { method: 'POST' }
  )
}

export async function listMCPTools(
  serverId?: string,
  isEnabled?: boolean
): Promise<MCPTool[]> {
  const params = new URLSearchParams()
  if (serverId) params.set('server_id', serverId)
  if (isEnabled !== undefined) params.set('is_enabled', String(isEnabled))
  return request<MCPTool[]>(`/mcp/tools?${params}`)
}

export async function callMCPTool(
  serverId: string,
  body: { toolId: string; toolName: string; inputData?: string; conversationId?: string; contactId?: string }
): Promise<{ callId: string; toolName: string; result: Record<string, any>; durationMs: number }> {
  return request<{ callId: string; toolName: string; result: Record<string, any>; durationMs: number }>(
    `/mcp/servers/${serverId}/call`,
    { method: 'POST', body: JSON.stringify(body) }
  )
}

export async function listMCPTemplates(): Promise<MCPServerTemplate[]> {
  return request<MCPServerTemplate[]>('/mcp/templates')
}

export async function getMCPSettings(): Promise<MCPSettings> {
  return request<MCPSettings>('/mcp/settings')
}

export async function updateMCPSettings(body: {
  enabled?: boolean
  autoDiscoverTools?: boolean
  logAllCalls?: boolean
  maxConcurrentCalls?: number
  timeoutSeconds?: number
}): Promise<void> {
  return request<void>('/mcp/settings', {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

export async function listMCPCallLogs(
  serverId?: string,
  status?: string
): Promise<MCPCallLog[]> {
  const params = new URLSearchParams()
  if (serverId) params.set('server_id', serverId)
  if (status) params.set('status', status)
  return request<MCPCallLog[]>(`/mcp/calls?${params}`)
}

// ============================================================================
// Phase 29: Autonomous Resolution
// ============================================================================

export interface ResolutionConfig {
  id: string
  organizationId: string
  enabled: boolean
  autoSendThreshold: number
  requireValidation: boolean
  validationThreshold: number
  maxAutonomousAttempts: number
  escalateOnLowConfidence: boolean
  lowConfidenceThreshold: number
  escalateOnSensitiveTopic: boolean
  sensitiveTopics?: string[]
  allowedChannels?: string[]
  cooldownSeconds: number
  createdAt: string
  updatedAt: string
}

export interface EscalationRule {
  id: string
  organizationId: string
  name: string
  description?: string
  conditionType: string
  conditionValue: number
  conditionTopic?: string
  conditionKeyword?: string
  actionType: string
  assignToAgentId?: string
  assignToTeamId?: string
  notifyWebhook?: string
  notificationMessage?: string
  isActive: boolean
  priority: number
  createdAt: string
  updatedAt: string
}

export interface ResolutionLog {
  id: string
  organizationId: string
  conversationId: string
  contactId: string
  channel: string
  attemptNumber: number
  aiConfidence: number
  validationScore?: number
  validationPassed?: boolean
  resolutionType: string
  escalated: boolean
  escalationRuleId?: string
  escalationReason?: string
  messageSent: boolean
  messageId?: string
  customerResolved?: boolean
  customerFeedbackRating?: number
  outcome: string
  errorMessage?: string
  durationMs?: number
  createdAt: string
  completedAt?: string
}

export interface AgentPolicy {
  id: string
  organizationId: string
  name: string
  description?: string
  policyType: string
  condition: string
  action: string
  isActive: boolean
  priority: number
  metadata?: Record<string, any>
  createdAt: string
  updatedAt: string
}

export interface ResolutionSummary {
  enabled: boolean
  totalAttempts7d: number
  totalAttempts30d: number
  successfulResolutions7d: number
  escalatedCount7d: number
  resolutionRate7d: number
  escalationRate7d: number
  avgConfidence7d: number
  autoSendThreshold: number
  activeEscalationRules: number
  activePolicies: number
  recentLogs: ResolutionLog[]
  topEscalationReasons: { reason: string; count: number }[]
}

export interface ResolutionDecision {
  shouldAutoSend: boolean
  reason: string
  escalationRule?: EscalationRule
  policiesViolated: AgentPolicy[]
}

export interface AnalyticsSnapshot {
  snapshotId: string
  windowStart: string
  windowEnd: string
  resolutionRate: number
  escalationRate: number
}

export async function getResolutionSummary(): Promise<ResolutionSummary> {
  return request<ResolutionSummary>('/autonomous-resolution/summary')
}

export async function getResolutionConfig(): Promise<ResolutionConfig> {
  return request<ResolutionConfig>('/autonomous-resolution/config')
}

export async function updateResolutionConfig(
  body: ResolutionConfig
): Promise<{ configId: string; message: string }> {
  return request<{ configId: string; message: string }>('/autonomous-resolution/config', {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

export async function listEscalationRules(
  isActive?: boolean
): Promise<EscalationRule[]> {
  const params = new URLSearchParams()
  if (isActive !== undefined) params.set('is_active', String(isActive))
  return request<EscalationRule[]>(`/autonomous-resolution/escalation-rules?${params}`)
}

export async function createEscalationRule(
  body: EscalationRule
): Promise<{ ruleId: string; message: string }> {
  return request<{ ruleId: string; message: string }>('/autonomous-resolution/escalation-rules', {
    method: 'POST',
    body: JSON.stringify(body)
  })
}

export async function updateEscalationRule(
  ruleId: string,
  body: EscalationRule
): Promise<{ ruleId: string; message: string }> {
  return request<{ ruleId: string; message: string }>(
    `/autonomous-resolution/escalation-rules/${ruleId}`,
    { method: 'PUT', body: JSON.stringify(body) }
  )
}

export async function deleteEscalationRule(
  ruleId: string
): Promise<{ ruleId: string; message: string }> {
  return request<{ ruleId: string; message: string }>(
    `/autonomous-resolution/escalation-rules/${ruleId}`,
    { method: 'DELETE' }
  )
}

export async function listAgentPolicies(
  isActive?: boolean
): Promise<AgentPolicy[]> {
  const params = new URLSearchParams()
  if (isActive !== undefined) params.set('is_active', String(isActive))
  return request<AgentPolicy[]>(`/autonomous-resolution/policies?${params}`)
}

export async function evaluateResolutionDecision(
  conversationId: string,
  aiConfidence: number,
  validationScore?: number,
  topic?: string
): Promise<ResolutionDecision> {
  const params = new URLSearchParams()
  params.set('conversation_id', conversationId)
  params.set('ai_confidence', String(aiConfidence))
  if (validationScore !== undefined) params.set('validation_score', String(validationScore))
  if (topic) params.set('topic', topic)
  return request<ResolutionDecision>(
    `/autonomous-resolution/evaluate?${params}`,
    { method: 'POST' }
  )
}

export async function logResolutionAttempt(
  body: {
    conversationId: string
    contactId: string
    channel: string
    aiConfidence: number
    validationScore?: number
    validationPassed?: boolean
  }
): Promise<ResolutionLog> {
  return request<ResolutionLog>('/autonomous-resolution/log-attempt', {
    method: 'POST',
    body: JSON.stringify(body)
  })
}

export async function listResolutionLogs(
  outcome?: string,
  escalated?: boolean,
  limit?: number,
  offset?: number
): Promise<ResolutionLog[]> {
  const params = new URLSearchParams()
  if (outcome) params.set('outcome', outcome)
  if (escalated !== undefined) params.set('escalated', String(escalated))
  if (limit) params.set('limit', String(limit))
  if (offset) params.set('offset', String(offset))
  return request<ResolutionLog[]>(`/autonomous-resolution/logs?${params}`)
}

export async function generateAnalyticsSnapshot(
  days?: number
): Promise<AnalyticsSnapshot> {
  const params = new URLSearchParams()
  if (days) params.set('days', String(days))
  return request<AnalyticsSnapshot>(`/autonomous-resolution/snapshot?${params}`, {
    method: 'POST'
  })
}

// ============================================================================
// Phase 30: Agent Performance Analytics
// ============================================================================

export interface AgentPerformanceSnapshot {
  id: string
  organizationId: string
  agentId: string
  agentName: string
  windowStart: string
  windowEnd: string
  conversationsHandled: number
  conversationsResolved: number
  avgResponseTimeSeconds: number
  avgFirstResponseTimeSeconds: number
  avgResolutionTimeSeconds: number
  resolutionRate: number
  customerSatisfactionAvg?: number
  customerSatisfactionCount: number
  positiveFeedbackCount: number
  negativeFeedbackCount: number
  feedbackRate: number
  messagesSent: number
  messagesReceived: number
  draftsApproved: number
  draftsRejected: number
  draftApprovalRate: number
  escalationsReceived: number
  escalationsSent: number
  slaBreaches: number
  slaComplianceRate: number
  cxScoreAvg?: number
  qaScoreAvg?: number
  createdAt: string
}

export interface CoachingInsight {
  id: string
  organizationId: string
  agentId: string
  agentName: string
  insightType: string
  title: string
  description: string
  category: string
  severity: string
  recommendation: string
  relatedConversationIds?: any
  relatedMetrics?: any
  isAddressed: boolean
  addressedAt?: string
  addressedBy?: string
  createdAt: string
}

export interface ComparisonMetrics {
  id: string
  organizationId: string
  windowStart: string
  windowEnd: string
  teamAvgResponseTime: number
  teamAvgResolutionRate: number
  teamAvgSatisfaction?: number
  teamAvgCxScore?: number
  teamAvgQaScore?: number
  topPerformerId?: string
  topPerformerScore?: number
  needsCoachingCount: number
  exceedsBenchmarkCount: number
  totalAgents: number
  createdAt: string
}

export interface AgentPerformanceSettings {
  id: string
  organizationId: string
  enabled: boolean
  autoGenerateInsights: boolean
  insightFrequency: string
  benchmarkResponseTimeSeconds: number
  benchmarkResolutionRate: number
  benchmarkSatisfaction: number
  benchmarkCxScore: number
  coachingSeverityThreshold: string
  showLeaderboard: boolean
  createdAt: string
  updatedAt: string
}

export interface AgentPerformanceSummary {
  enabled: boolean
  totalAgents: number
  teamAvgResolutionRate: number
  teamAvgResponseTime: number
  teamAvgSatisfaction?: number
  needsCoaching: number
  exceedsBenchmark: number
  activeInsights: number
  recentSnapshots: AgentPerformanceSnapshot[]
  recentInsights: CoachingInsight[]
  comparisonMetrics?: ComparisonMetrics
}

export async function getAgentPerformanceSummary(): Promise<AgentPerformanceSummary> {
  return request<AgentPerformanceSummary>('/agent-performance/summary')
}

export async function listAgentSnapshots(
  agentId?: string,
  days?: number
): Promise<AgentPerformanceSnapshot[]> {
  const params = new URLSearchParams()
  if (agentId) params.set('agent_id', agentId)
  if (days) params.set('days', String(days))
  return request<AgentPerformanceSnapshot[]>(`/agent-performance/snapshots?${params}`)
}

export async function createAgentSnapshot(
  body: { agentId: string; agentName: string; [key: string]: any }
): Promise<{ snapshotId: string; agentId: string; message: string }> {
  return request<{ snapshotId: string; agentId: string; message: string }>('/agent-performance/snapshots', {
    method: 'POST',
    body: JSON.stringify(body)
  })
}

export async function listCoachingInsights(
  agentId?: string,
  addressed?: boolean
): Promise<CoachingInsight[]> {
  const params = new URLSearchParams()
  if (agentId) params.set('agent_id', agentId)
  if (addressed !== undefined) params.set('addressed', String(addressed))
  return request<CoachingInsight[]>(`/agent-performance/insights?${params}`)
}

export async function createCoachingInsight(
  body: { agentId: string; agentName: string; title: string; description: string; category: string; severity: string; recommendation: string }
): Promise<{ insightId: string; message: string }> {
  return request<{ insightId: string; message: string }>('/agent-performance/insights', {
    method: 'POST',
    body: JSON.stringify(body)
  })
}

export async function addressCoachingInsight(
  insightId: string,
  addressedBy: string
): Promise<{ insightId: string; message: string }> {
  return request<{ insightId: string; message: string }>(
    `/agent-performance/insights/${insightId}/address`,
    { method: 'PUT', body: JSON.stringify({ addressed_by: addressedBy }) }
  )
}

export async function getComparisonMetrics(days?: number): Promise<ComparisonMetrics[]> {
  const params = new URLSearchParams()
  if (days) params.set('days', String(days))
  return request<ComparisonMetrics[]>(`/agent-performance/comparison?${params}`)
}

export async function generateComparisonSnapshot(): Promise<ComparisonMetrics> {
  return request<ComparisonMetrics>('/agent-performance/comparison/generate', { method: 'POST' })
}

export async function getAgentPerformanceSettings(): Promise<AgentPerformanceSettings> {
  return request<AgentPerformanceSettings>('/agent-performance/settings')
}

export async function updateAgentPerformanceSettings(
  body: Partial<AgentPerformanceSettings>
): Promise<{ settingsId: string; message: string }> {
  return request<{ settingsId: string; message: string }>('/agent-performance/settings', {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

// ——— Phase 31: Real-time Notifications & WebSockets ———

export interface NotificationPreferences {
  id: string
  organizationId: string
  userId: string
  newMessagePush: boolean
  newMessageEmail: boolean
  draftApprovedPush: boolean
  draftRejectedPush: boolean
  alertTriggeredPush: boolean
  alertTriggeredEmail: boolean
  conversationAssignedPush: boolean
  conversationEscalatedPush: boolean
  workflowCompletedPush: boolean
  csatResponsePush: boolean
  suggestionReadyPush: boolean
  operatorProposalPush: boolean
  soundEnabled: boolean
  desktopNotifications: boolean
  quietHoursStart?: string
  quietHoursEnd?: string
  createdAt: string
  updatedAt: string
}

export interface NotificationItem {
  id: string
  organizationId: string
  userId?: string
  notificationType: string
  title: string
  body: string
  data?: any
  channel: string
  status: string
  sentAt?: string
  readAt?: string
  dismissedAt?: string
  createdAt: string
}

export interface WebSocketConnection {
  id: string
  organizationId: string
  userId: string
  sessionId: string
  connectedAt: string
  lastHeartbeat: string
  path: string
  userAgent: string
  ipAddress: string
}

export interface BroadcastLog {
  id: string
  organizationId: string
  notificationType: string
  title: string
  recipientCount: number
  deliveredCount: number
  failedCount: number
  broadcastAt: string
}

export interface NotificationSettings {
  id: string
  organizationId: string
  enabled: boolean
  websocketEnabled: boolean
  emailNotificationsEnabled: boolean
  emailFromAddress: string
  maxNotificationsPerHour: number
  digestMode: boolean
  digestIntervalMinutes: number
  retentionDays: number
  createdAt: string
  updatedAt: string
}

export interface NotificationTypeCount {
  notificationType: string
  count: number
}

export interface NotificationSummary {
  enabled: boolean
  websocketEnabled: boolean
  totalNotifications: number
  pendingNotifications: number
  unreadNotifications: number
  activeConnections: number
  broadcastsToday: number
  byType: NotificationTypeCount[]
  recentNotifications: NotificationItem[]
  recentBroadcasts: BroadcastLog[]
}

export async function getNotificationSummary(): Promise<NotificationSummary> {
  return request<NotificationSummary>('/notifications/summary')
}

export async function listNotifications(
  userId?: string,
  notificationType?: string,
  status?: string,
  limit?: number
): Promise<NotificationItem[]> {
  const params = new URLSearchParams()
  if (userId) params.set('user_id', userId)
  if (notificationType) params.set('notification_type', notificationType)
  if (status) params.set('status', status)
  if (limit) params.set('limit', String(limit))
  return request<NotificationItem[]>(`/notifications?${params}`)
}

export async function queueNotification(
  body: {
    organizationId: string
    userId?: string
    notificationType: string
    title: string
    body: string
    data?: string
    channel?: string
  }
): Promise<{ notificationId: string; message: string }> {
  return request<{ notificationId: string; message: string }>('/notifications/queue', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      user_id: body.userId,
      notification_type: body.notificationType,
      title: body.title,
      body: body.body,
      data: body.data,
      channel: body.channel
    })
  })
}

export async function broadcastNotification(
  body: {
    organizationId: string
    notificationType: string
    title: string
    body: string
  }
): Promise<{ broadcastId: string; recipients: number; message: string }> {
  return request<{ broadcastId: string; recipients: number; message: string }>('/notifications/broadcast', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      notification_type: body.notificationType,
      title: body.title,
      body: body.body
    })
  })
}

export async function markNotificationRead(
  id: string
): Promise<{ notificationId: string; message: string }> {
  return request<{ notificationId: string; message: string }>(`/notifications/${id}/read`, {
    method: 'PUT'
  })
}

export async function dismissNotification(
  id: string
): Promise<{ notificationId: string; message: string }> {
  return request<{ notificationId: string; message: string }>(`/notifications/${id}/dismiss`, {
    method: 'PUT'
  })
}

export async function getNotificationPreferences(
  userId: string
): Promise<NotificationPreferences> {
  return request<NotificationPreferences>(`/notifications/preferences/${userId}`)
}

export async function updateNotificationPreferences(
  userId: string,
  body: Partial<NotificationPreferences>
): Promise<{ preferencesId: string; message: string }> {
  return request<{ preferencesId: string; message: string }>(
    `/notifications/preferences/${userId}`,
    { method: 'PUT', body: JSON.stringify(body) }
  )
}

export async function getNotificationSettings(): Promise<NotificationSettings> {
  return request<NotificationSettings>('/notifications/settings')
}

export async function updateNotificationSettings(
  body: Partial<NotificationSettings>
): Promise<{ settingsId: string; message: string }> {
  return request<{ settingsId: string; message: string }>('/notifications/settings', {
    method: 'PUT',
    body: JSON.stringify(body)
  })
}

export async function listWebSocketConnections(): Promise<WebSocketConnection[]> {
  return request<WebSocketConnection[]>('/notifications/connections')
}

export async function cleanupOldNotifications(): Promise<{ deletedCount: number; message: string }> {
  return request<{ deletedCount: number; message: string }>('/notifications/cleanup', {
    method: 'POST'
  })
}

// ============================================================================
// Phase 32: In-App Messaging & Proactive Triggers
// ============================================================================

export interface InAppMessage {
  id: string
  organizationId: string
  title: string
  body: string
  messageType: string
  style: string
  targetUrl?: string
  buttonText?: string
  buttonUrl?: string
  dismissible: boolean
  autoCloseSeconds?: number
  triggerType: string
  triggerEvent?: string
  triggerPage?: string
  triggerDelaySeconds?: number
  triggerSegmentId?: string
  audienceSegmentId?: string
  frequency: string
  status: string
  startAt?: string
  endAt?: string
  views: number
  clicks: number
  dismisses: number
  createdAt: string
  updatedAt: string
}

export interface InAppMessageEvent {
  id: string
  organizationId: string
  messageId: string
  contactId?: string
  sessionId?: string
  eventType: string
  metadata?: any
  createdAt: string
}

export interface ProactiveTrigger {
  id: string
  organizationId: string
  name: string
  triggerType: string
  condition?: any
  action: string
  messageId?: string
  delaySeconds: number
  enabled: boolean
  maxTriggersPerDay: number
  triggersToday: number
  createdAt: string
  updatedAt: string
}

export interface BehavioralEvent {
  id: string
  organizationId: string
  contactId?: string
  sessionId?: string
  eventName: string
  eventData?: any
  pageUrl?: string
  createdAt: string
}

export interface InAppSettings {
  id: string
  organizationId: string
  enabled: boolean
  maxConcurrentMessages: number
  defaultStyle: string
  allowCustomCss: boolean
  trackBehavioralEvents: boolean
  eventRetentionDays: number
  createdAt: string
  updatedAt: string
}

export interface MessageTypeCount {
  messageType: string
  count: number
}

export interface TriggerTypeCount {
  triggerType: string
  count: number
}

export interface InAppSummary {
  enabled: boolean
  totalMessages: number
  activeMessages: number
  draftMessages: number
  totalTriggers: number
  enabledTriggers: number
  totalEventsToday: number
  totalBehavioralEvents: number
  byMessageType: MessageTypeCount[]
  byTriggerType: TriggerTypeCount[]
  recentMessages: InAppMessage[]
  recentEvents: InAppMessageEvent[]
}

export interface TriggerEvaluationResult {
  matched: boolean
  messageId?: string
  triggerName?: string
  reason: string
}

export async function getInAppSummary(): Promise<InAppSummary> {
  return request<InAppSummary>('/inapp/summary')
}

export async function createInAppMessage(
  body: {
    organizationId: string
    title: string
    body: string
    messageType?: string
    triggerType?: string
    triggerEvent?: string
    triggerPage?: string
    frequency?: string
  }
): Promise<{ messageId: string; message: string }> {
  return request<{ messageId: string; message: string }>('/inapp/messages', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      title: body.title,
      body: body.body,
      message_type: body.messageType,
      trigger_type: body.triggerType,
      trigger_event: body.triggerEvent,
      trigger_page: body.triggerPage,
      frequency: body.frequency
    })
  })
}

export async function listInAppMessages(
  status?: string,
  messageType?: string
): Promise<InAppMessage[]> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (messageType) params.set('message_type', messageType)
  return request<InAppMessage[]>(`/inapp/messages?${params}`)
}

export async function getInAppMessage(id: string): Promise<InAppMessage> {
  return request<InAppMessage>(`/inapp/messages/${id}`)
}

export async function updateInAppMessage(
  id: string,
  body: Partial<InAppMessage>
): Promise<{ messageId: string; message: string }> {
  return request<{ messageId: string; message: string }>(`/inapp/messages/${id}`, {
    method: 'PUT',
    body: JSON.stringify({
      title: body.title,
      body: body.body,
      message_type: body.messageType,
      status: body.status,
      button_text: body.buttonText,
      button_url: body.buttonUrl,
      trigger_type: body.triggerType,
      trigger_event: body.triggerEvent,
      frequency: body.frequency
    })
  })
}

export async function deleteInAppMessage(id: string): Promise<{ messageId: string; message: string }> {
  return request<{ messageId: string; message: string }>(`/inapp/messages/${id}`, {
    method: 'DELETE'
  })
}

export async function createProactiveTrigger(
  body: {
    organizationId: string
    name: string
    triggerType: string
    condition?: string
    action?: string
    messageId?: string
    delaySeconds?: number
  }
): Promise<{ triggerId: string; message: string }> {
  return request<{ triggerId: string; message: string }>('/inapp/triggers', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      name: body.name,
      trigger_type: body.triggerType,
      condition: body.condition,
      action: body.action,
      message_id: body.messageId,
      delay_seconds: body.delaySeconds
    })
  })
}

export async function listProactiveTriggers(): Promise<ProactiveTrigger[]> {
  return request<ProactiveTrigger[]>('/inapp/triggers')
}

export async function updateProactiveTrigger(
  id: string,
  body: Partial<ProactiveTrigger>
): Promise<{ triggerId: string; message: string }> {
  return request<{ triggerId: string; message: string }>(`/inapp/triggers/${id}`, {
    method: 'PUT',
    body: JSON.stringify({
      name: body.name,
      trigger_type: body.triggerType,
      condition: body.condition,
      enabled: body.enabled,
      message_id: body.messageId
    })
  })
}

export async function deleteProactiveTrigger(id: string): Promise<{ triggerId: string; message: string }> {
  return request<{ triggerId: string; message: string }>(`/inapp/triggers/${id}`, {
    method: 'DELETE'
  })
}

export async function recordBehavioralEvent(
  body: {
    organizationId: string
    contactId?: string
    sessionId?: string
    eventName: string
    eventData?: string
    pageUrl?: string
  }
): Promise<{ eventId: string; message: string }> {
  return request<{ eventId: string; message: string }>('/inapp/events/behavioral', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      contact_id: body.contactId,
      session_id: body.sessionId,
      event_name: body.eventName,
      event_data: body.eventData,
      page_url: body.pageUrl
    })
  })
}

export async function listBehavioralEvents(): Promise<BehavioralEvent[]> {
  return request<BehavioralEvent[]>('/inapp/events/behavioral/list')
}

export async function recordMessageEvent(
  body: {
    organizationId: string
    messageId: string
    contactId?: string
    sessionId?: string
    eventType: string
  }
): Promise<{ eventId: string; message: string }> {
  return request<{ eventId: string; message: string }>('/inapp/events/message', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      message_id: body.messageId,
      contact_id: body.contactId,
      session_id: body.sessionId,
      event_type: body.eventType
    })
  })
}

export async function evaluateTriggers(
  body: {
    organizationId: string
    eventName: string
    pageUrl?: string
  }
): Promise<TriggerEvaluationResult[]> {
  return request<TriggerEvaluationResult[]>('/inapp/triggers/evaluate', {
    method: 'POST',
    body: JSON.stringify({
      organization_id: body.organizationId,
      event_name: body.eventName,
      page_url: body.pageUrl
    })
  })
}

export async function getInAppSettings(): Promise<InAppSettings> {
  return request<InAppSettings>('/inapp/settings')
}

export async function updateInAppSettings(
  body: Partial<InAppSettings>
): Promise<{ settingsId: string; message: string }> {
  return request<{ settingsId: string; message: string }>('/inapp/settings', {
    method: 'PUT',
    body: JSON.stringify({
      enabled: body.enabled,
      max_concurrent_messages: body.maxConcurrentMessages,
      default_style: body.defaultStyle,
      track_behavioral_events: body.trackBehavioralEvents,
      event_retention_days: body.eventRetentionDays
    })
  })
}

// ============================================================================
// TONE OF VOICE & GUIDANCE — Phase 33
// ============================================================================

export interface ToneConfig {
  id: string
  organizationId: string
  toneStyle: string
  personality: string
  answerLength: string
  language: string
  formalityLevel: number
  humorEnabled: boolean
  empathyLevel: number
  concisenessLevel: number
  customInstructions?: string
  forbiddenPhrases?: any
  requiredPhrases?: any
  greetingTemplate: string
  closingTemplate: string
  escalationTemplate: string
  enabled: boolean
  createdAt: string
  updatedAt: string
}

export interface GuidanceRule {
  id: string
  organizationId: string
  name: string
  description?: string
  ruleType: string
  instruction: string
  priority: number
  appliesTo: string
  enabled: boolean
  createdAt: string
  updatedAt: string
}

export interface ToneAnalytics {
  id: string
  organizationId: string
  toneStyle: string
  conversationCount: number
  avgSatisfaction: number
  avgResolutionRate: number
  avgResponseLength: number
  thumbsUpCount: number
  thumbsDownCount: number
  windowStart: string
  windowEnd: string
  createdAt: string
}

export interface TonePreset {
  id: string
  name: string
  description?: string
  toneStyle: string
  personality: string
  answerLength: string
  formalityLevel: number
  humorEnabled: boolean
  empathyLevel: number
  concisenessLevel: number
  greetingTemplate: string
  closingTemplate: string
  escalationTemplate: string
  customInstructions?: string
}

export interface ToneSummary {
  enabled: boolean
  currentToneStyle: string
  currentPersonality: string
  currentAnswerLength: string
  totalGuidanceRules: number
  enabledGuidanceRules: number
  totalPresets: number
  recentAnalytics: ToneAnalytics[]
  guidanceRules: GuidanceRule[]
  presets: TonePreset[]
}

export interface PromptAdditions {
  toneInstructions: string
  guidanceInstructions: string
  greetingTemplate: string
  closingTemplate: string
  escalationTemplate: string
}

export async function getToneSummary(): Promise<ToneSummary> {
  return request<ToneSummary>('/tone-voice/summary')
}

export async function getToneConfig(): Promise<ToneConfig> {
  return request<ToneConfig>('/tone-voice/config')
}

export async function updateToneConfig(
  body: Partial<ToneConfig>
): Promise<{ configId: string; message: string }> {
  return request<{ configId: string; message: string }>('/tone-voice/config', {
    method: 'PUT',
    body: JSON.stringify({
      tone_style: body.toneStyle,
      personality: body.personality,
      answer_length: body.answerLength,
      language: body.language,
      formality_level: body.formalityLevel,
      humor_enabled: body.humorEnabled,
      empathy_level: body.empathyLevel,
      conciseness_level: body.concisenessLevel,
      custom_instructions: body.customInstructions,
      greeting_template: body.greetingTemplate,
      closing_template: body.closingTemplate,
      escalation_template: body.escalationTemplate,
      enabled: body.enabled
    })
  })
}

export async function applyTonePreset(
  presetId: string
): Promise<{ configId: string; message: string }> {
  return request<{ configId: string; message: string }>('/tone-voice/apply-preset', {
    method: 'POST',
    body: JSON.stringify({ preset_id: presetId })
  })
}

export async function listGuidanceRules(): Promise<GuidanceRule[]> {
  return request<GuidanceRule[]>('/tone-voice/guidance')
}

export async function createGuidanceRule(
  body: {
    name: string
    instruction: string
    ruleType?: string
    priority?: number
  }
): Promise<{ ruleId: string; message: string }> {
  return request<{ ruleId: string; message: string }>('/tone-voice/guidance', {
    method: 'POST',
    body: JSON.stringify({
      name: body.name,
      instruction: body.instruction,
      rule_type: body.ruleType,
      priority: body.priority
    })
  })
}

export async function updateGuidanceRule(
  id: string,
  body: Partial<GuidanceRule>
): Promise<{ ruleId: string; message: string }> {
  return request<{ ruleId: string; message: string }>(`/tone-voice/guidance/${id}`, {
    method: 'PUT',
    body: JSON.stringify({
      name: body.name,
      description: body.description,
      rule_type: body.ruleType,
      instruction: body.instruction,
      priority: body.priority,
      applies_to: body.appliesTo,
      enabled: body.enabled
    })
  })
}

export async function deleteGuidanceRule(id: string): Promise<{ ruleId: string; message: string }> {
  return request<{ ruleId: string; message: string }>(`/tone-voice/guidance/${id}`, {
    method: 'DELETE'
  })
}

export async function getPromptAdditions(): Promise<PromptAdditions> {
  return request<PromptAdditions>('/tone-voice/prompt-additions')
}

export async function createToneAnalytics(
  body: {
    toneStyle: string
    conversationCount: number
    avgSatisfaction: number
    avgResolutionRate: number
    avgResponseLength: number
    thumbsUp: number
    thumbsDown: number
  }
): Promise<{ analyticsId: string; message: string }> {
  return request<{ analyticsId: string; message: string }>('/tone-voice/analytics', {
    method: 'POST',
    body: JSON.stringify({
      tone_style: body.toneStyle,
      conversation_count: body.conversationCount,
      avg_satisfaction: body.avgSatisfaction,
      avg_resolution_rate: body.avgResolutionRate,
      avg_response_length: body.avgResponseLength,
      thumbs_up: body.thumbsUp,
      thumbs_down: body.thumbsDown
    })
  })
}

// ============================================================================
// Phase 35: Embed Widget
// ============================================================================

export interface EmbedWidgetConfig {
  id: string
  orgId: string
  name: string
  widgetType: string
  status: string
  settings: Record<string, unknown>
  styles: Record<string, unknown>
  triggerConfig: Record<string, unknown>
  domainWhitelist: string[] | null
  installCount: number
  lastInstalledAt: string | null
  createdAt: string
  updatedAt: string
}

export interface EmbedWidgetEvent {
  id: string
  instanceId: string
  eventType: string
  payload: Record<string, unknown>
  createdAt: string
}

export interface EmbedWidgetSettings {
  id: string
  orgId: string
  enabled: boolean
  allowCustomDomains: boolean
  maxInstancesPerDomain: number
  analyticsEnabled: boolean
  defaultLanguage: string
}

export interface EmbedWidgetSummary {
  totalConfigs: number
  activeConfigs: number
  totalInstances: number
  activeInstances: number
  totalEvents: number
  eventsByType: { eventType: string; count: number }[]
  topDomains: { domain: string; instances: number }[]
  recentEvents: EmbedWidgetEvent[]
}

export interface CreateWidgetResult {
  config: EmbedWidgetConfig
  script: string
}

export async function getEmbedWidgetSummary(): Promise<EmbedWidgetSummary> {
  return request<EmbedWidgetSummary>('/embed-widget/summary')
}

export async function listEmbedWidgetConfigs(
  orgId?: string,
  status?: string
): Promise<EmbedWidgetConfig[]> {
  const params = new URLSearchParams()
  if (orgId) params.set('org_id', orgId)
  if (status) params.set('status', status)
  return request<EmbedWidgetConfig[]>(`/embed-widget/configs?${params}`)
}

export async function getEmbedWidgetConfig(id: string): Promise<EmbedWidgetConfig> {
  return request<EmbedWidgetConfig>(`/embed-widget/configs/${id}`)
}

export async function createEmbedWidgetConfig(
  body: {
    name: string
    widgetType: string
    settings?: Record<string, unknown>
    styles?: Record<string, unknown>
    triggerConfig?: Record<string, unknown>
  }
): Promise<CreateWidgetResult> {
  return request<CreateWidgetResult>('/embed-widget/configs', {
    method: 'POST',
    body: JSON.stringify({
      name: body.name,
      widget_type: body.widgetType,
      settings: body.settings,
      styles: body.styles,
      trigger_config: body.triggerConfig
    })
  })
}

export async function updateEmbedWidgetConfig(
  id: string,
  body: {
    name?: string
    status?: string
    settings?: Record<string, unknown>
    styles?: Record<string, unknown>
    triggerConfig?: Record<string, unknown>
  }
): Promise<EmbedWidgetConfig> {
  return request<EmbedWidgetConfig>(`/embed-widget/configs/${id}`, {
    method: 'PUT',
    body: JSON.stringify({
      name: body.name,
      status: body.status,
      settings: body.settings,
      styles: body.styles,
      trigger_config: body.triggerConfig
    })
  })
}

export async function deleteEmbedWidgetConfig(id: string): Promise<void> {
  return request<void>(`/embed-widget/configs/${id}`, { method: 'DELETE' })
}

export async function getEmbedWidgetSettings(): Promise<EmbedWidgetSettings> {
  return request<EmbedWidgetSettings>('/embed-widget/settings')
}

export async function updateEmbedWidgetSettings(
  body: Partial<EmbedWidgetSettings>
): Promise<EmbedWidgetSettings> {
  return request<EmbedWidgetSettings>('/embed-widget/settings', {
    method: 'PUT',
    body: JSON.stringify({
      enabled: body.enabled,
      allow_custom_domains: body.allowCustomDomains,
      max_instances_per_domain: body.maxInstancesPerDomain,
      analytics_enabled: body.analyticsEnabled,
      default_language: body.defaultLanguage
    })
  })
}

export async function recordWidgetEvent(
  body: {
    instanceId: string
    eventType: string
    payload?: Record<string, unknown>
  }
): Promise<{ eventId: string }> {
  return request<{ eventId: string }>('/embed-widget/events', {
    method: 'POST',
    body: JSON.stringify({
      instance_id: body.instanceId,
      event_type: body.eventType,
      payload: body.payload
    })
  })
}

export async function getEmbedWidgetEvents(
  instanceId?: string,
  eventType?: string,
  limit?: number
): Promise<EmbedWidgetEvent[]> {
  const params = new URLSearchParams()
  if (instanceId) params.set('instance_id', instanceId)
  if (eventType) params.set('event_type', eventType)
  if (limit) params.set('limit', String(limit))
  return request<EmbedWidgetEvent[]>(`/embed-widget/events?${params}`)
}

export async function getEmbedScript(configId: string): Promise<{ script: string; configId: string }> {
  return request<{ script: string; configId: string }>(`/embed-widget/script/${configId}`)
}

// ---------------------------------------------------------------------------
// Phase 36: Unified Analytics Dashboard
// ---------------------------------------------------------------------------

export interface UnifiedAnalyticsSummary {
  total_conversations: number
  resolved_conversations: number
  resolution_rate: number
  avg_response_time_seconds: number
  cx_score: number
  csat_score: number
  feedback_satisfaction_rate: number
  validation_pass_rate: number
  total_feedback: number
  positive_feedback: number
  negative_feedback: number
  total_suggestions: number
  approved_suggestions: number
  active_alerts: number
  conversations_by_channel: { channel: string; count: number }[]
  conversations_by_status: { status: string; count: number }[]
  topic_distribution: { topic: string; count: number }[]
  agent_leaderboard: { agent_id: string; agent_name: string; conversations_handled: number; resolution_rate: number }[]
  autonomous_stats: { total_resolutions: number; auto_sent: number; escalated: number; resolution_rate: number }
  procedure_stats: { total_procedures: number; active_executions: number; completed_executions: number; success_rate: number }
  workflow_stats: { total_workflows: number; active_workflows: number; total_executions: number; success_rate: number }
}

export interface SavedReport {
  id: string
  org_id: string
  name: string
  description?: string
  report_type: string
  config: Record<string, unknown>
  time_range: string
  start_date?: string
  end_date?: string
  filters: Record<string, unknown>
  last_run_at?: string
  last_result?: Record<string, unknown>
  created_by?: string
  created_at: string
  updated_at: string
}

export interface ScheduledReport {
  id: string
  report_id: string
  schedule: string
  cron_expression?: string
  next_run_at?: string
  last_run_at?: string
  recipients: string[]
  format: string
  status: string
  created_at: string
  updated_at: string
}

export interface ReportTemplate {
  id: string
  org_id: string
  name: string
  report_type: string
  description?: string
  config: Record<string, unknown>
  is_default: boolean
  usage_count: number
  created_at: string
  updated_at: string
}

export interface DashboardWidget {
  id: string
  org_id: string
  dashboard_id: string
  widget_type: string
  title: string
  config: Record<string, unknown>
  position_x: number
  position_y: number
  width: number
  height: number
  is_visible: boolean
  created_at: string
  updated_at: string
}

export interface AnalyticsSnapshot {
  id: string
  org_id: string
  snapshot_type: string
  period_start: string
  period_end: string
  data: Record<string, unknown>
  created_at: string
}

export interface UnifiedAnalyticsSettings {
  id: string
  org_id: string
  enabled: boolean
  auto_snapshot: boolean
  snapshot_frequency: string
  retention_days: number
  default_dashboard: string
}

export async function getUnifiedAnalyticsSummary(): Promise<UnifiedAnalyticsSummary> {
  return request<UnifiedAnalyticsSummary>('/analytics/summary')
}

export async function listSavedReports(orgId?: string, reportType?: string): Promise<SavedReport[]> {
  const params = new URLSearchParams()
  if (orgId) params.set('org_id', orgId)
  if (reportType) params.set('report_type', reportType)
  return request<SavedReport[]>(`/analytics/reports?${params}`)
}

export async function createSavedReport(body: {
  org_id: string
  name: string
  report_type: string
  config: Record<string, unknown>
  time_range: string
  filters: Record<string, unknown>
  created_by: string
}): Promise<SavedReport> {
  return request<SavedReport>('/analytics/reports', { method: 'POST', body: JSON.stringify(body) })
}

export async function updateSavedReport(id: string, updates: Record<string, unknown>): Promise<SavedReport> {
  return request<SavedReport>(`/analytics/reports/${id}`, { method: 'PUT', body: JSON.stringify(updates) })
}

export async function deleteSavedReport(id: string): Promise<void> {
  return request<void>(`/analytics/reports/${id}`, { method: 'DELETE' })
}

export async function runReport(id: string): Promise<{ report_id: string; result: Record<string, unknown>; run_at: string }> {
  return request<{ report_id: string; result: Record<string, unknown>; run_at: string }>(`/analytics/reports/${id}/run`, { method: 'POST' })
}

export async function listScheduledReports(): Promise<ScheduledReport[]> {
  return request<ScheduledReport[]>('/analytics/scheduled')
}

export async function createScheduledReport(body: {
  report_id: string
  schedule: string
  recipients: string[]
  format: string
}): Promise<ScheduledReport> {
  return request<ScheduledReport>('/analytics/scheduled', { method: 'POST', body: JSON.stringify(body) })
}

export async function updateScheduledReport(id: string, status: string): Promise<ScheduledReport> {
  return request<ScheduledReport>(`/analytics/scheduled/${id}`, { method: 'PUT', body: JSON.stringify({ status }) })
}

export async function deleteScheduledReport(id: string): Promise<void> {
  return request<void>(`/analytics/scheduled/${id}`, { method: 'DELETE' })
}

export async function listReportTemplates(orgId?: string): Promise<ReportTemplate[]> {
  const params = new URLSearchParams()
  if (orgId) params.set('org_id', orgId)
  return request<ReportTemplate[]>(`/analytics/templates?${params}`)
}

export async function createReportTemplate(body: {
  org_id: string
  name: string
  report_type: string
  description: string
  config: Record<string, unknown>
}): Promise<ReportTemplate> {
  return request<ReportTemplate>('/analytics/templates', { method: 'POST', body: JSON.stringify(body) })
}

export async function listDashboardWidgets(orgId: string, dashboardId: string): Promise<DashboardWidget[]> {
  const params = new URLSearchParams()
  params.set('org_id', orgId)
  params.set('dashboard_id', dashboardId)
  return request<DashboardWidget[]>(`/analytics/widgets?${params}`)
}

export async function createDashboardWidget(body: {
  org_id: string
  dashboard_id: string
  widget_type: string
  title: string
  config: Record<string, unknown>
  position_x: number
  position_y: number
  width: number
  height: number
}): Promise<DashboardWidget> {
  return request<DashboardWidget>('/analytics/widgets', { method: 'POST', body: JSON.stringify(body) })
}

export async function updateDashboardWidget(id: string, updates: Record<string, unknown>): Promise<DashboardWidget> {
  return request<DashboardWidget>(`/analytics/widgets/${id}`, { method: 'PUT', body: JSON.stringify(updates) })
}

export async function deleteDashboardWidget(id: string): Promise<void> {
  return request<void>(`/analytics/widgets/${id}`, { method: 'DELETE' })
}

export async function createAnalyticsSnapshot(body: {
  org_id: string
  snapshot_type: string
  period_start: string
  period_end: string
}): Promise<AnalyticsSnapshot> {
  return request<AnalyticsSnapshot>('/analytics/snapshots', { method: 'POST', body: JSON.stringify(body) })
}

export async function listAnalyticsSnapshots(orgId: string, snapshotType?: string, limit?: number): Promise<AnalyticsSnapshot[]> {
  const params = new URLSearchParams()
  params.set('org_id', orgId)
  if (snapshotType) params.set('snapshot_type', snapshotType)
  if (limit) params.set('limit', String(limit))
  return request<AnalyticsSnapshot[]>(`/analytics/snapshots?${params}`)
}

export async function getUnifiedAnalyticsSettings(orgId?: string): Promise<UnifiedAnalyticsSettings> {
  const params = new URLSearchParams()
  if (orgId) params.set('org_id', orgId)
  return request<UnifiedAnalyticsSettings>(`/analytics/settings?${params}`)
}

export async function updateUnifiedAnalyticsSettings(updates: Record<string, unknown>): Promise<UnifiedAnalyticsSettings> {
  return request<UnifiedAnalyticsSettings>('/analytics/settings', { method: 'PUT', body: JSON.stringify(updates) })
}

export async function exportReport(reportId: string, format: string): Promise<{ format: string; data: Record<string, unknown>; exported_at: string }> {
  const params = new URLSearchParams()
  params.set('report_id', reportId)
  params.set('format', format)
  return request<{ format: string; data: Record<string, unknown>; exported_at: string }>(`/analytics/export?${params}`)
}

// ---------------------------------------------------------------------------
// Phase 37: Reply Engine Integration
// ---------------------------------------------------------------------------

export interface ReplyIntegrationSettings {
  id: string
  org_id: string
  use_tone_of_voice: boolean
  use_guidance_rules: boolean
  use_audience_scoping: boolean
  use_help_center_articles: boolean
  use_document_chunks: boolean
  max_knowledge_entries: number
  max_document_chunks: number
  max_help_center_articles: number
  track_knowledge_sources: boolean
}

export interface SourceTypeCount {
  source_type: string
  count: number
}

export interface ReplyIntegrationSummary {
  tone_of_voice_enabled: boolean
  guidance_rules_active: number
  audience_scoping_enabled: boolean
  help_center_enabled: boolean
  document_chunks_enabled: boolean
  total_knowledge_sources_tracked: number
  sources_by_type: SourceTypeCount[]
  settings: ReplyIntegrationSettings
}

export interface KnowledgeSourceUsed {
  source_type: string
  source_id: string | null
  source_title: string
}

export interface ReplyContextPackage {
  knowledge_context: string
  tone_instructions: string
  guidance_instructions: string
  sources_used: KnowledgeSourceUsed[]
  audience_scoped: boolean
  contact_id: string | null
}

export async function getReplyIntegrationSummary(): Promise<ReplyIntegrationSummary> {
  return request<ReplyIntegrationSummary>('/reply-integration/summary')
}

export async function getReplyIntegrationSettings(): Promise<ReplyIntegrationSettings> {
  return request<ReplyIntegrationSettings>('/reply-integration/settings')
}

export async function updateReplyIntegrationSettings(settings: ReplyIntegrationSettings): Promise<ReplyIntegrationSettings> {
  return request<ReplyIntegrationSettings>('/reply-integration/settings', { method: 'PUT', body: JSON.stringify(settings) })
}

export async function buildReplyContext(body: {
  contact_id?: string
  last_inbound: string
  tenant_key: string
}): Promise<ReplyContextPackage> {
  return request<ReplyContextPackage>('/reply-integration/build-context', { method: 'POST', body: JSON.stringify(body) })
}

export async function getKnowledgeSources(draftId?: string, conversationId?: string, sourceType?: string): Promise<{ sources: any[]; total: number }> {
  const params = new URLSearchParams()
  if (draftId) params.set('draft_id', draftId)
  if (conversationId) params.set('conversation_id', conversationId)
  if (sourceType) params.set('source_type', sourceType)
  return request<{ sources: any[]; total: number }>(`/reply-integration/sources?${params}`)
}

// ---------------------------------------------------------------------------
// Phase 38: A/B Testing & Phased Deployments
// ---------------------------------------------------------------------------

export interface ABExperiment {
  id: string
  name: string
  description: string | null
  experiment_type: string
  status: string
  traffic_split: number
  start_at: string | null
  end_at: string | null
  winner_variant_id: string | null
  created_by: string | null
  org_id: string
  created_at: string
  updated_at: string
}

export interface ABVariant {
  id: string
  experiment_id: string
  variant_name: string
  config: Record<string, unknown>
  is_control: boolean
  created_at: string
}

export interface ABAssignment {
  id: string
  experiment_id: string
  variant_id: string
  conversation_id: string
  contact_id: string | null
  assigned_at: string
}

export interface ABResult {
  id: string
  experiment_id: string
  variant_id: string
  assignment_id: string | null
  metric_name: string
  metric_value: number
  metadata: Record<string, unknown>
  created_at: string
}

export interface VariantStats {
  variant_id: string
  variant_name: string
  is_control: boolean
  assignments: number
  results: number
  avg_metric_values: Record<string, number>
  min_metric_values: Record<string, number>
  max_metric_values: Record<string, number>
}

export interface ABExperimentSummary {
  id: string
  experiment_id: string
  total_assignments: number
  total_results: number
  variant_stats: VariantStats[]
  statistical_significance: number
  recommended_winner: string | null
  snapshot_at: string
}

export interface ABSettings {
  id: string
  org_id: string
  enabled: boolean
  auto_detect_winner: boolean
  min_sample_size: number
  significance_threshold: number
  max_concurrent_experiments: number
}

export interface ABTestingSummary {
  total_experiments: number
  running_experiments: number
  completed_experiments: number
  total_assignments: number
  total_results: number
  experiments_by_type: { experiment_type: string; count: number }[]
  recent_experiments: ABExperiment[]
  settings: ABSettings
}

export interface AssignmentResult {
  assignment: ABAssignment
  variant: ABVariant
}

export interface CreateExperimentResult {
  experiment: ABExperiment
  variants: ABVariant[]
}

export async function getABTestingSummary(): Promise<ABTestingSummary> {
  return request<ABTestingSummary>('/ab-testing/summary')
}

export async function listABExperiments(status?: string, experimentType?: string): Promise<ABExperiment[]> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (experimentType) params.set('experiment_type', experimentType)
  return request<ABExperiment[]>(`/ab-testing/experiments?${params}`)
}

export async function createABExperiment(body: {
  name: string
  description?: string
  experiment_type: string
  traffic_split: number
  variants: { variant_name: string; config: Record<string, unknown> }[]
}): Promise<CreateExperimentResult> {
  return request<CreateExperimentResult>('/ab-testing/experiments', { method: 'POST', body: JSON.stringify(body) })
}

export async function getABExperiment(id: string): Promise<ABExperiment> {
  return request<ABExperiment>(`/ab-testing/experiments/${id}`)
}

export async function updateABExperimentStatus(id: string, status: string): Promise<ABExperiment> {
  return request<ABExperiment>(`/ab-testing/experiments/${id}/status`, { method: 'PUT', body: JSON.stringify({ status }) })
}

export async function deleteABExperiment(id: string): Promise<void> {
  return request<void>(`/ab-testing/experiments/${id}`, { method: 'DELETE' })
}

export async function getABVariants(experimentId: string): Promise<ABVariant[]> {
  return request<ABVariant[]>(`/ab-testing/experiments/${experimentId}/variants`)
}

export async function assignConversation(experimentId: string, body: {
  conversation_id: string
  contact_id?: string
}): Promise<AssignmentResult> {
  return request<AssignmentResult>(`/ab-testing/experiments/${experimentId}/assign`, { method: 'POST', body: JSON.stringify(body) })
}

export async function recordABResult(experimentId: string, body: {
  variant_id: string
  assignment_id: string
  metric_name: string
  metric_value: number
}): Promise<ABResult> {
  return request<ABResult>(`/ab-testing/experiments/${experimentId}/results`, { method: 'POST', body: JSON.stringify(body) })
}

export async function getABResults(experimentId: string): Promise<ABResult[]> {
  return request<ABResult[]>(`/ab-testing/experiments/${experimentId}/results`)
}

export async function generateABSummary(experimentId: string): Promise<{ summary: ABExperimentSummary }> {
  return request<{ summary: ABExperimentSummary }>(`/ab-testing/experiments/${experimentId}/summary`, { method: 'POST' })
}

export async function getABSettings(): Promise<ABSettings> {
  return request<ABSettings>('/ab-testing/settings')
}

export async function updateABSettings(settings: ABSettings): Promise<ABSettings> {
  return request<ABSettings>('/ab-testing/settings', { method: 'PUT', body: JSON.stringify(settings) })
}

// ============================================================================
// ERP FULFILMENT — Phase 39
// ============================================================================

export interface FulfilmentSummary {
  total_zones: number
  active_zones: number
  total_methods: number
  active_methods: number
  total_rates: number
  total_returns: number
  pending_returns: number
  refunded_amount: number
  total_manifests: number
  open_manifests: number
}

export interface ErpShippingZone {
  id: string
  key: string
  sort_order: number
  cities: string
  is_active: boolean
  is_catch_all: boolean
  created_at: string
  updated_at: string
}

export interface ErpShippingMethod {
  id: string
  key: string
  courier_key: string | null
  eta_min_days: number
  eta_max_days: number
  is_active: boolean
  sort_order: number
  created_at: string
  updated_at: string
}

export interface ErpShippingRate {
  id: string
  zone_id: string
  method_id: string
  rate_type: string
  fee: number
  base_fee: number
  included_units: number
  free_above: number | null
  min_value: number | null
  max_value: number | null
  is_active: boolean
}

export interface ErpReturnReasonCode {
  id: string
  key: string
  is_active: boolean
  sort_order: number
}

export interface ErpReturnRequest {
  id: string
  number: string
  order_id: string
  user_id: string | null
  status: string
  resolution: string | null
  customer_note: string | null
  staff_note: string | null
  photo_urls: string
  requested_at: string
  approved_at: string | null
  received_at: string | null
  inspected_at: string | null
  refunded_at: string | null
  refund_amount: number | null
  restocking_fee_minor: number | null
}

export interface ErpReturnRequestItem {
  id: string
  return_id: string
  order_item_id: string
  reason_id: string
  quantity: number
  restockable: boolean | null
  restocked: boolean
}

export interface ErpManifest {
  id: string
  number: string
  courier_key: string
  status: string
  dispatched_at: string | null
  created_by_id: string | null
  created_at: string
}

export async function getFulfilmentSummary(): Promise<FulfilmentSummary> {
  return request<FulfilmentSummary>('/erp/fulfilment/summary')
}

export async function listShippingZones(activeOnly = true): Promise<ErpShippingZone[]> {
  return request<ErpShippingZone[]>(`/erp/fulfilment/shipping-zones?activeOnly=${activeOnly}`)
}

export async function createShippingZone(body: { key: string; cities?: string[]; is_catch_all?: boolean }): Promise<ErpShippingZone> {
  return request<ErpShippingZone>('/erp/fulfilment/shipping-zones', { method: 'POST', body: JSON.stringify(body) })
}

export async function getShippingZone(id: string): Promise<ErpShippingZone> {
  return request<ErpShippingZone>(`/erp/fulfilment/shipping-zones/${id}`)
}

export async function updateShippingZone(id: string, body: Record<string, unknown>): Promise<ErpShippingZone> {
  return request<ErpShippingZone>(`/erp/fulfilment/shipping-zones/${id}`, { method: 'PUT', body: JSON.stringify(body) })
}

export async function deleteShippingZone(id: string): Promise<void> {
  return request<void>(`/erp/fulfilment/shipping-zones/${id}`, { method: 'DELETE' })
}

export async function listShippingMethods(activeOnly = true): Promise<ErpShippingMethod[]> {
  return request<ErpShippingMethod[]>(`/erp/fulfilment/shipping-methods?activeOnly=${activeOnly}`)
}

export async function createShippingMethod(body: { key: string; courier_key?: string; eta_min_days?: number; eta_max_days?: number }): Promise<ErpShippingMethod> {
  return request<ErpShippingMethod>('/erp/fulfilment/shipping-methods', { method: 'POST', body: JSON.stringify(body) })
}

export async function getShippingMethod(id: string): Promise<ErpShippingMethod> {
  return request<ErpShippingMethod>(`/erp/fulfilment/shipping-methods/${id}`)
}

export async function updateShippingMethod(id: string, body: Record<string, unknown>): Promise<ErpShippingMethod> {
  return request<ErpShippingMethod>(`/erp/fulfilment/shipping-methods/${id}`, { method: 'PUT', body: JSON.stringify(body) })
}

export async function deleteShippingMethod(id: string): Promise<void> {
  return request<void>(`/erp/fulfilment/shipping-methods/${id}`, { method: 'DELETE' })
}

export async function listShippingRates(): Promise<ErpShippingRate[]> {
  return request<ErpShippingRate[]>('/erp/fulfilment/shipping-rates')
}

export async function createShippingRate(body: { zone_id: string; method_id: string; rate_type?: string; fee: number; base_fee?: number; included_units?: number; free_above?: number; min_value?: number; max_value?: number }): Promise<ErpShippingRate> {
  return request<ErpShippingRate>('/erp/fulfilment/shipping-rates', { method: 'POST', body: JSON.stringify(body) })
}

export async function deleteShippingRate(id: string): Promise<void> {
  return request<void>(`/erp/fulfilment/shipping-rates/${id}`, { method: 'DELETE' })
}

export async function listReturnReasons(): Promise<ErpReturnReasonCode[]> {
  return request<ErpReturnReasonCode[]>('/erp/fulfilment/return-reasons')
}

export async function listReturnRequests(status?: string): Promise<ErpReturnRequest[]> {
  const params = status ? `?status=${status}` : ''
  return request<ErpReturnRequest[]>(`/erp/fulfilment/returns${params}`)
}

export async function getReturnRequest(id: string): Promise<{ request: ErpReturnRequest; items: ErpReturnRequestItem[] }> {
  return request<{ request: ErpReturnRequest; items: ErpReturnRequestItem[] }>(`/erp/fulfilment/returns/${id}`)
}

export async function createReturnRequest(body: { order_id: string; user_id?: string; customer_note?: string; photo_urls?: string[]; items: { order_item_id: string; reason_id: string; quantity: number }[] }): Promise<ErpReturnRequest> {
  return request<ErpReturnRequest>('/erp/fulfilment/returns', { method: 'POST', body: JSON.stringify(body) })
}

export async function updateReturnStatus(id: string, body: { status: string; resolution?: string; staff_note?: string; refund_amount?: number }): Promise<ErpReturnRequest> {
  return request<ErpReturnRequest>(`/erp/fulfilment/returns/${id}/status`, { method: 'PUT', body: JSON.stringify(body) })
}

export async function getReturnStats(): Promise<Record<string, unknown>> {
  return request<Record<string, unknown>>('/erp/fulfilment/returns/stats')
}

export async function listManifests(): Promise<ErpManifest[]> {
  return request<ErpManifest[]>('/erp/fulfilment/manifests')
}

export async function dispatchManifest(id: string): Promise<ErpManifest> {
  return request<ErpManifest>(`/erp/fulfilment/manifests/${id}/dispatch`, { method: 'POST' })
}

// ============================================================================
// SCHEDULED TASKS & CRON JOBS — Phase 40
// ============================================================================

export interface ScheduledTask {
  id: string
  name: string
  description: string | null
  task_type: string
  cron_schedule: string
  next_run_at: string | null
  is_active: boolean
  config: string
  created_at: string
  updated_at: string
}

export interface TaskRun {
  id: string
  task_id: string
  started_at: string
  completed_at: string | null
  status: string
  duration_ms: number | null
  result: string | null
  error_message: string | null
  records_processed: number
}

export interface TaskQueueEntry {
  id: string
  task_id: string
  triggered_by: string
  triggered_at: string
  status: string
  completed_at: string | null
  error_message: string | null
}

export interface SchedulerSettings {
  id: string
  org_id: string
  enabled: boolean
  max_concurrent_tasks: number
  task_timeout_seconds: number
  retry_failed: boolean
  retry_count: number
  retry_delay_seconds: number
  log_retention_days: number
  webhook_url: string | null
  created_at: string
  updated_at: string
}

export interface TaskTypeCount {
  task_type: string
  count: number
}

export interface SchedulerSummary {
  total_tasks: number
  active_tasks: number
  pending_queue: number
  total_runs: number
  successful_runs: number
  failed_runs: number
  scheduler_enabled: boolean
  task_type_breakdown: TaskTypeCount[]
  recent_runs: TaskRun[]
}

export interface TriggerResult {
  queue_id: string
  task_name: string
  message: string
}

export interface CleanupResult {
  deleted_runs: number
  deleted_queue: number
  message: string
}

export async function getSchedulerSummary(): Promise<SchedulerSummary> {
  return request<SchedulerSummary>('/scheduled-tasks/summary')
}

export async function listScheduledTasks(activeOnly?: boolean, taskType?: string): Promise<ScheduledTask[]> {
  const params = new URLSearchParams()
  if (activeOnly) params.set('active_only', 'true')
  if (taskType) params.set('task_type', taskType)
  return request<ScheduledTask[]>(`/scheduled-tasks?${params}`)
}

export async function getScheduledTask(id: string): Promise<ScheduledTask> {
  return request<ScheduledTask>(`/scheduled-tasks/${id}`)
}

export async function createScheduledTask(body: {
  name: string
  description?: string
  task_type: string
  cron_schedule: string
  config?: Record<string, unknown>
}): Promise<ScheduledTask> {
  return request<ScheduledTask>('/scheduled-tasks', { method: 'POST', body: JSON.stringify(body) })
}

export async function updateScheduledTask(id: string, body: {
  description?: string
  task_type?: string
  cron_schedule?: string
  is_active?: boolean
  config?: Record<string, unknown>
}): Promise<ScheduledTask> {
  return request<ScheduledTask>(`/scheduled-tasks/${id}`, { method: 'PUT', body: JSON.stringify(body) })
}

export async function deleteScheduledTask(id: string): Promise<void> {
  return request<void>(`/scheduled-tasks/${id}`, { method: 'DELETE' })
}

export async function triggerScheduledTask(id: string): Promise<TriggerResult> {
  return request<TriggerResult>(`/scheduled-tasks/${id}/trigger`, { method: 'POST' })
}

export async function processTaskQueue(): Promise<string[]> {
  return request<string[]>('/scheduled-tasks/process-queue', { method: 'POST' })
}

export async function listTaskRuns(taskId?: string, status?: string, limit?: number): Promise<TaskRun[]> {
  const params = new URLSearchParams()
  if (taskId) params.set('task_id', taskId)
  if (status) params.set('status', status)
  if (limit) params.set('limit', String(limit))
  return request<TaskRun[]>(`/scheduled-tasks/runs?${params}`)
}

export async function listTaskQueue(status?: string): Promise<TaskQueueEntry[]> {
  const params = status ? `?status=${status}` : ''
  return request<TaskQueueEntry[]>(`/scheduled-tasks/queue${params}`)
}

export async function getSchedulerSettings(): Promise<SchedulerSettings> {
  return request<SchedulerSettings>('/scheduled-tasks/settings')
}

export async function updateSchedulerSettings(settings: SchedulerSettings): Promise<SchedulerSettings> {
  return request<SchedulerSettings>('/scheduled-tasks/settings', { method: 'PUT', body: JSON.stringify(settings) })
}

export async function cleanupOldLogs(retentionDays?: number): Promise<CleanupResult> {
  const params = retentionDays ? `?retention_days=${retentionDays}` : ''
  return request<CleanupResult>(`/scheduled-tasks/cleanup${params}`, { method: 'POST' })
}

export async function getAvailableTaskTypes(): Promise<string[]> {
  return request<string[]>('/scheduled-tasks/task-types')
}

// ============================================================================
// Phase 41: Compliance & Data Privacy Dashboard
// ============================================================================

export interface SuppressionEntry {
  id: string
  contact_id: string
  channel: string
  reason: string
  added_at: string
  added_by: string
  note: string | null
  is_active: boolean
}

export interface GDPRRequest {
  id: string
  contact_id: string | null
  email: string | null
  request_type: string
  status: string
  requested_at: string
  completed_at: string | null
  handled_by: string | null
  notes: string | null
  data_export_url: string | null
  expires_at: string | null
}

export interface CompliancePolicy {
  id: string
  org_id: string
  policy_name: string
  regulation: string
  channel: string
  description: string | null
  rules: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface AuditLogEntry {
  id: string
  action: string
  entity_type: string
  entity_id: string
  details: string
  performed_by: string
  performed_at: string
  severity: string
}

export interface ComplianceSettings {
  id: string
  org_id: string
  gdpr_response_days: number
  auto_expire_gdpr_requests: boolean
  require_consent_before_send: boolean
  check_suppression_before_send: boolean
  log_all_compliance_actions: boolean
  can_spam_unsubscribe_link: boolean
  can_spam_physical_address: string | null
  tcpa_consent_required: boolean
  whatsapp_opt_in_required: boolean
  data_retention_days: number
  created_at: string
  updated_at: string
}

export interface ComplianceReasonCount {
  reason: string
  count: number
}

export interface ComplianceGDPRTypeCount {
  request_type: string
  count: number
}

export interface ComplianceSummary {
  total_suppressed: number
  active_suppressed: number
  pending_gdpr_requests: number
  total_gdpr_requests: number
  active_policies: number
  total_audit_entries: number
  critical_violations: number
  suppression_by_reason: ComplianceReasonCount[]
  gdpr_by_type: ComplianceGDPRTypeCount[]
  recent_audit_logs: AuditLogEntry[]
  upcoming_gdpr_expiries: number
}

export interface SuppressionCheckResult {
  is_suppressed: boolean
  entry: SuppressionEntry | null
  message: string
}

export interface ComplianceCheck {
  check_name: string
  regulation: string
  passed: boolean
  message: string
}

export interface ComplianceCheckResult {
  is_compliant: boolean
  checks: ComplianceCheck[]
  message: string
}

// API Functions

export async function getComplianceSummary(): Promise<ComplianceSummary> {
  return request<ComplianceSummary>('/compliance/summary')
}

export async function listSuppressions(activeOnly?: boolean, channel?: string, reason?: string): Promise<SuppressionEntry[]> {
  const params = new URLSearchParams()
  if (activeOnly) params.set('active_only', 'true')
  if (channel) params.set('channel', channel)
  if (reason) params.set('reason', reason)
  return request<SuppressionEntry[]>(`/compliance/suppressions?${params}`)
}

export async function addSuppression(body: {
  contact_id: string
  channel: string
  reason: string
  note?: string
}): Promise<SuppressionEntry> {
  return request<SuppressionEntry>('/compliance/suppressions', { method: 'POST', body: JSON.stringify(body) })
}

export async function getSuppression(id: string): Promise<SuppressionEntry> {
  return request<SuppressionEntry>(`/compliance/suppressions/${id}`)
}

export async function removeSuppression(id: string): Promise<void> {
  return request<void>(`/compliance/suppressions/${id}`, { method: 'DELETE' })
}

export async function checkSuppressed(contactId: string, channel: string): Promise<SuppressionCheckResult> {
  const params = new URLSearchParams()
  params.set('contact_id', contactId)
  params.set('channel', channel)
  return request<SuppressionCheckResult>(`/compliance/suppressions/check?${params}`)
}

export async function listGDPRRequests(status?: string, requestType?: string): Promise<GDPRRequest[]> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (requestType) params.set('request_type', requestType)
  return request<GDPRRequest[]>(`/compliance/gdpr-requests?${params}`)
}

export async function createGDPRRequest(body: {
  contact_id?: string
  email?: string
  request_type: string
  notes?: string
}): Promise<GDPRRequest> {
  return request<GDPRRequest>('/compliance/gdpr-requests', { method: 'POST', body: JSON.stringify(body) })
}

export async function getGDPRRequest(id: string): Promise<GDPRRequest> {
  return request<GDPRRequest>(`/compliance/gdpr-requests/${id}`)
}

export async function updateGDPRRequest(id: string, body: {
  status?: string
  handled_by?: string
  notes?: string
}): Promise<GDPRRequest> {
  return request<GDPRRequest>(`/compliance/gdpr-requests/${id}`, { method: 'PUT', body: JSON.stringify(body) })
}

export async function listPolicies(activeOnly?: boolean, regulation?: string): Promise<CompliancePolicy[]> {
  const params = new URLSearchParams()
  if (activeOnly) params.set('active_only', 'true')
  if (regulation) params.set('regulation', regulation)
  return request<CompliancePolicy[]>(`/compliance/policies?${params}`)
}

export async function createPolicy(body: {
  policy_name: string
  regulation: string
  channel: string
  description?: string
  rules?: Record<string, unknown>
}): Promise<CompliancePolicy> {
  return request<CompliancePolicy>('/compliance/policies', { method: 'POST', body: JSON.stringify(body) })
}

export async function getPolicy(id: string): Promise<CompliancePolicy> {
  return request<CompliancePolicy>(`/compliance/policies/${id}`)
}

export async function updatePolicy(id: string, isActive?: boolean): Promise<CompliancePolicy> {
  const params = isActive !== undefined ? `?is_active=${isActive}` : ''
  return request<CompliancePolicy>(`/compliance/policies/${id}${params}`, { method: 'PUT' })
}

export async function deletePolicy(id: string): Promise<void> {
  return request<void>(`/compliance/policies/${id}`, { method: 'DELETE' })
}

export async function listAuditLogs(action?: string, severity?: string, limit?: number): Promise<AuditLogEntry[]> {
  const params = new URLSearchParams()
  if (action) params.set('action', action)
  if (severity) params.set('severity', severity)
  if (limit) params.set('limit', String(limit))
  return request<AuditLogEntry[]>(`/compliance/audit-logs?${params}`)
}

export async function checkCompliance(contactId: string, channel: string, body: string): Promise<ComplianceCheckResult> {
  const params = new URLSearchParams()
  params.set('contact_id', contactId)
  params.set('channel', channel)
  params.set('body', body)
  return request<ComplianceCheckResult>(`/compliance/check?${params}`)
}

export async function getComplianceSettings(): Promise<ComplianceSettings> {
  return request<ComplianceSettings>('/compliance/settings')
}

export async function updateComplianceSettings(settings: ComplianceSettings): Promise<ComplianceSettings> {
  return request<ComplianceSettings>('/compliance/settings', { method: 'PUT', body: JSON.stringify(settings) })
}

// ---------------------------------------------------------------------------
// Phase 42: Conversation Search & Discovery
// ---------------------------------------------------------------------------

export interface SearchResult {
  searchType: string
  id: string
  title: string
  snippet: string
  metadata: Record<string, any>
  matchedFields: string[]
  score: number
}

export interface SearchResponse {
  query: string
  totalResults: number
  conversations: SearchResult[]
  messages: SearchResult[]
  contacts: SearchResult[]
  durationMs: number
}

export interface SavedSearch {
  id: string
  orgId: string
  name: string
  query: string
  searchType: string
  filters: string
  createdBy: string
  createdAt: string
  updatedAt: string
  isPinned: boolean
  usageCount: number
}

export interface SearchHistoryEntry {
  id: string
  orgId: string
  query: string
  searchType: string
  resultCount: number
  filters: string
  performedBy: string
  performedAt: string
  durationMs: number
}

export interface SearchSummary {
  totalSavedSearches: number
  pinnedSearches: number
  totalSearchesToday: number
  totalSearches7d: number
  avgResultsPerSearch: number
  zeroResultRate: number
  topQueries: Array<{ query: string; count: number }>
  topSearchTypes: Array<{ searchType: string; count: number }>
  recentHistory: SearchHistoryEntry[]
}

export interface SearchSettings {
  id: string
  orgId: string
  maxResults: number
  searchMessageContent: boolean
  searchContactDetails: boolean
  searchConversationMetadata: boolean
  trackSearchHistory: boolean
  autoSuggest: boolean
  minQueryLength: number
  highlightMatches: boolean
  createdAt: string
  updatedAt: string
}

export interface SearchAnalyticsSnapshot {
  windowStart: string
  windowEnd: string
  totalSearches: number
  avgResultCount: number
  avgDurationMs: number
  zeroResultSearches: number
}

// API Functions

export async function getSearchSummary(): Promise<SearchSummary> {
  return request<SearchSummary>('/search/summary')
}

export async function searchConversations(q: string, searchType?: string, limit?: number): Promise<SearchResponse> {
  const params = new URLSearchParams()
  params.set('q', q)
  if (searchType) params.set('search_type', searchType)
  if (limit) params.set('limit', String(limit))
  return request<SearchResponse>(`/search?${params}`)
}

export async function listSavedSearches(searchType?: string): Promise<SavedSearch[]> {
  const params = new URLSearchParams()
  if (searchType) params.set('search_type', searchType)
  return request<SavedSearch[]>(`/search/saved?${params}`)
}

export async function createSavedSearch(body: {
  name: string
  query: string
  search_type?: string
  filters?: Record<string, any>
}): Promise<SavedSearch> {
  return request<SavedSearch>('/search/saved', { method: 'POST', body: JSON.stringify(body) })
}

export async function getSavedSearch(id: string): Promise<SavedSearch> {
  return request<SavedSearch>(`/search/saved/${id}`)
}

export async function deleteSavedSearch(id: string): Promise<void> {
  return request<void>(`/search/saved/${id}`, { method: 'DELETE' })
}

export async function pinSavedSearch(id: string, pinned: boolean): Promise<void> {
  return request<void>(`/search/saved/${id}/pin`, { method: 'POST', body: JSON.stringify({ pinned }) })
}

export async function useSavedSearch(id: string): Promise<SearchResponse> {
  return request<SearchResponse>(`/search/saved/${id}/use`, { method: 'POST' })
}

export async function getSearchHistory(limit?: number): Promise<SearchHistoryEntry[]> {
  const params = new URLSearchParams()
  if (limit) params.set('limit', String(limit))
  return request<SearchHistoryEntry[]>(`/search/history?${params}`)
}

export async function clearSearchHistory(): Promise<void> {
  return request<void>('/search/history', { method: 'DELETE' })
}

export async function generateSearchAnalyticsSnapshot(): Promise<SearchAnalyticsSnapshot> {
  return request<SearchAnalyticsSnapshot>('/search/analytics/snapshot', { method: 'POST' })
}

export async function getSearchSettings(): Promise<SearchSettings> {
  return request<SearchSettings>('/search/settings')
}

export async function updateSearchSettings(settings: Partial<SearchSettings>): Promise<SearchSettings> {
  return request<SearchSettings>('/search/settings', { method: 'PUT', body: JSON.stringify(settings) })
}

// Phase 44: System Health Check & Onboarding Wizard

export interface OnboardingStep {
  id: string
  category: string
  title: string
  description: string
  route: string
  checkEndpoint: string | null
  displayOrder: number
  isRequired: boolean
}

export interface OnboardingProgress {
  id: string
  organizationId: string
  stepId: string
  isCompleted: boolean
  completedAt: string | null
  completedBy: string | null
  notes: string | null
}

export interface HealthCheckResult {
  id: string
  organizationId: string
  component: string
  status: string
  message: string | null
  details: string | null
  lastCheckedAt: string
  checkDurationMs: number | null
}

export interface OnboardingSettings {
  id: string
  organizationId: string
  showOnboarding: boolean
  autoCheck: boolean
  healthCheckIntervalMinutes: number
}

export interface CategoryProgress {
  category: string
  total: number
  completed: number
  percentage: number
}

export interface OnboardingSummary {
  totalSteps: number
  completedSteps: number
  requiredSteps: number
  requiredCompleted: number
  completionPercentage: number
  requiredCompletionPercentage: number
  categories: CategoryProgress[]
  nextSteps: OnboardingStep[]
  settings: OnboardingSettings
}

export interface HealthCheckSummary {
  totalComponents: number
  healthy: number
  degraded: number
  unhealthy: number
  unknown: number
  overallStatus: string
  results: HealthCheckResult[]
}

export async function getOnboardingSummary(): Promise<OnboardingSummary> {
  return request<OnboardingSummary>('/system-health/summary')
}

export async function listOnboardingSteps(): Promise<OnboardingStep[]> {
  return request<OnboardingStep[]>('/system-health/steps')
}

export async function getOnboardingProgress(): Promise<OnboardingProgress[]> {
  return request<OnboardingProgress[]>('/system-health/progress')
}

export async function completeOnboardingStep(stepId: string, completedBy?: string): Promise<{ success: boolean; progress: OnboardingProgress; summary: OnboardingSummary }> {
  return request<any>('/system-health/complete-step', { method: 'POST', body: JSON.stringify({ stepId, completedBy }) })
}

export async function runHealthCheck(): Promise<{ success: boolean; summary: HealthCheckSummary }> {
  return request<any>('/system-health/health-check', { method: 'POST' })
}

export async function getHealthCheckSummary(): Promise<HealthCheckSummary> {
  return request<HealthCheckSummary>('/system-health/health-summary')
}

export async function getOnboardingSettings(): Promise<OnboardingSettings> {
  return request<OnboardingSettings>('/system-health/settings')
}

export async function updateOnboardingSettings(settings: Partial<OnboardingSettings>): Promise<OnboardingSettings> {
  return request<OnboardingSettings>('/system-health/settings', { method: 'PUT', body: JSON.stringify(settings) })
}

// Phase 45: Email Templates & SMTP Management

export interface SmtpSettingsItem {
  id: string
  brandId: string | null
  host: string
  port: number
  security: string
  authMethod: string
  username: string
  password: string
  fromAddress: string
  fromName: string
  replyToAddress: string | null
  connectionTimeoutSeconds: number
  enabled: boolean
  testSentAt: string | null
  testStatus: string | null
  testError: string | null
  createdAt: string
  updatedAt: string
}

export interface EmailTemplateItem {
  id: string
  brandId: string | null
  name: string
  subject: string
  bodyHtml: string
  bodyText: string
  templateType: string
  category: string
  language: string
  enabled: boolean
  isDefault: boolean
  variables: any[]
  createdAt: string
  updatedAt: string
}

export interface EmailSendLogItem {
  id: string
  brandId: string | null
  templateId: string | null
  templateName: string | null
  toAddress: string
  fromAddress: string
  subject: string
  status: string
  errorMessage: string | null
  smtpResponse: string | null
  messageId: string | null
  sentAt: string | null
  createdAt: string
}

export interface SendEmailRequest {
  toAddress: string
  templateName: string
  brandId: string | null
  variables: Record<string, string>
}

export interface SmtpSummary {
  totalSmtpConfigs: number
  enabledSmtpConfigs: number
  totalTemplates: number
  enabledTemplates: number
  defaultTemplates: number
  totalSendLogs: number
  sentCount: number
  failedCount: number
  bouncedCount: number
  categories: Array<{ category: string; count: number }>
  recentLogs: EmailSendLogItem[]
}

export async function getSmtpSummary(): Promise<SmtpSummary> {
  return request<SmtpSummary>('/smtp/summary')
}

export async function listSmtpSettings(): Promise<SmtpSettingsItem[]> {
  return request<SmtpSettingsItem[]>('/smtp')
}

export async function getSmtpSettings(id: string): Promise<SmtpSettingsItem> {
  return request<SmtpSettingsItem>(`/smtp/${id}`)
}

export async function createSmtpSettings(settings: {
  brandId?: string
  host: string
  port?: number
  security?: string
  authMethod?: string
  username: string
  password: string
  fromAddress: string
  fromName?: string
  replyToAddress?: string
  connectionTimeoutSeconds?: number
}): Promise<SmtpSettingsItem> {
  return request<SmtpSettingsItem>('/smtp', { method: 'POST', body: JSON.stringify(settings) })
}

export async function updateSmtpSettings(id: string, settings: Partial<SmtpSettingsItem>): Promise<SmtpSettingsItem> {
  return request<SmtpSettingsItem>(`/smtp/${id}`, { method: 'PUT', body: JSON.stringify(settings) })
}

export async function deleteSmtpSettings(id: string): Promise<void> {
  return request<void>(`/smtp/${id}`, { method: 'DELETE' })
}

export async function testSmtpConnection(id: string, toAddress: string): Promise<{ success: boolean; message: string; error?: string }> {
  return request<any>(`/smtp/${id}/test`, { method: 'POST', body: JSON.stringify({ toAddress }) })
}

export async function listEmailTemplates(category?: string, templateType?: string): Promise<EmailTemplateItem[]> {
  const params = new URLSearchParams()
  if (category) params.set('category', category)
  if (templateType) params.set('template_type', templateType)
  const qs = params.toString()
  return request<EmailTemplateItem[]>(`/email-templates${qs ? '?' + qs : ''}`)
}

export async function getEmailTemplate(id: string): Promise<EmailTemplateItem> {
  return request<EmailTemplateItem>(`/email-templates/${id}`)
}

export async function createEmailTemplate(template: {
  brandId?: string
  name: string
  subject: string
  bodyHtml: string
  bodyText?: string
  templateType?: string
  category?: string
  language?: string
  enabled?: boolean
  variables?: any[]
}): Promise<EmailTemplateItem> {
  return request<EmailTemplateItem>('/email-templates', { method: 'POST', body: JSON.stringify(template) })
}

export async function updateEmailTemplate(id: string, template: Partial<EmailTemplateItem>): Promise<EmailTemplateItem> {
  return request<EmailTemplateItem>(`/email-templates/${id}`, { method: 'PUT', body: JSON.stringify(template) })
}

export async function deleteEmailTemplate(id: string): Promise<void> {
  return request<void>(`/email-templates/${id}`, { method: 'DELETE' })
}

export async function sendTemplatedEmail(req: SendEmailRequest): Promise<{ success: boolean; logId: string; messageId?: string; error?: string }> {
  return request<any>('/email/send', { method: 'POST', body: JSON.stringify({
    to_address: req.toAddress,
    template_name: req.templateName,
    brand_id: req.brandId,
    variables: req.variables
  }) })
}

export async function getEmailSendLogs(limit?: number): Promise<EmailSendLogItem[]> {
  const params = new URLSearchParams()
  if (limit) params.set('limit', String(limit))
  const qs = params.toString()
  return request<EmailSendLogItem[]>(`/email/logs${qs ? '?' + qs : ''}`)
}

// Phase 46: Omni-Channel Broadcast System

export interface BroadcastItem {
  id: string
  organizationId: string
  brandId: string | null
  title: string
  status: string
  createdBy: string
  scheduledAt: string | null
  completedAt: string | null
  cancelledAt: string | null
  bodyText: string
  bodyHtml: string | null
  subjectLine: string | null
  previewText: string | null
  mediaType: string
  primaryMediaUrl: string | null
  mediaCaption: string | null
  linkUrl: string | null
  linkTitle: string | null
  linkDescription: string | null
  linkImageUrl: string | null
  targetType: string
  targetIds: any[]
  channels: any[]
  channelOverrides: any
  totalRecipients: number
  sentCount: number
  deliveredCount: number
  failedCount: number
  readCount: number
  clickedCount: number
  repliedCount: number
  sendDelaySeconds: number
  retryFailed: boolean
  maxRetries: number
  trackOpens: boolean
  trackClicks: boolean
  metadata: any
  createdAt: string
  updatedAt: string
}

export interface BroadcastMediaItem {
  id: string
  broadcastId: string
  mediaType: string
  fileName: string
  fileSizeBytes: number
  mimeType: string
  fileUrl: string
  thumbnailUrl: string | null
  durationSeconds: number
  width: number
  height: number
  status: string
  errorMessage: string | null
  metadata: any
  createdAt: string
}

export interface BroadcastChannelItem {
  id: string
  broadcastId: string
  channelCode: string
  channelName: string
  status: string
  formattedBody: string | null
  formattedMediaUrl: string | null
  formattedSubject: string | null
  recipientCount: number
  sentCount: number
  failedCount: number
  startedAt: string | null
  completedAt: string | null
  errorMessage: string | null
  metadata: any
  createdAt: string
}

export interface BroadcastRecipientItem {
  id: string
  broadcastId: string
  contactId: string
  channelCode: string
  phoneNumber: string | null
  emailAddress: string | null
  externalId: string | null
  status: string
  messageId: string | null
  sentAt: string | null
  deliveredAt: string | null
  readAt: string | null
  clickedAt: string | null
  repliedAt: string | null
  errorMessage: string | null
  retryCount: number
  metadata: any
  createdAt: string
}

export interface BroadcastLogItem {
  id: string
  broadcastId: string
  eventType: string
  channelCode: string | null
  recipientId: string | null
  details: any
  createdAt: string
}

export interface ChannelFormatItem {
  channelCode: string
  channelName: string
  formattedBody: string
  formattedMediaUrl: string | null
  formattedSubject: string | null
  supportsHtml: boolean
  supportsImages: boolean
  supportsVideo: boolean
  supportsLinks: boolean
  maxLength: number
}

export interface BroadcastChannelCount {
  channelCode: string
  channelName: string
  broadcastCount: number
  recipientCount: number
}

export interface BroadcastSummary {
  totalBroadcasts: number
  draftBroadcasts: number
  sentBroadcasts: number
  failedBroadcasts: number
  scheduledBroadcasts: number
  totalRecipientsSent: number
  totalDelivered: number
  totalFailed: number
  totalRead: number
  totalClicked: number
  totalReplied: number
  byChannel: BroadcastChannelCount[]
  recentBroadcasts: BroadcastItem[]
}

export interface SendBroadcastResult {
  broadcastId: string
  status: string
  channelsSent: string[]
  totalRecipients: number
  message: string
}

export interface FormatPreviewResult {
  broadcastId: string
  formats: ChannelFormatItem[]
}

export interface BroadcastSettingsItem {
  id: string
  organizationId: string
  enabled: boolean
  maxRecipientsPerBroadcast: number
  defaultSendDelaySeconds: number
  rateLimitPerMinute: number
  allowScheduling: boolean
  requireApproval: boolean
  approvedBy: string | null
  autoFormatChannels: boolean
  trackOpens: boolean
  trackClicks: boolean
  defaultChannels: string[]
  createdAt: string
  updatedAt: string
}

export async function getBroadcastSummary(): Promise<BroadcastSummary> {
  return request<BroadcastSummary>('/broadcasts/summary')
}

export async function listBroadcasts(status?: string, limit?: number): Promise<BroadcastItem[]> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (limit) params.set('limit', String(limit))
  const qs = params.toString()
  const res: any = await request<any>(`/broadcasts${qs ? '?' + qs : ''}`)
  return res.broadcasts || []
}

export async function getBroadcast(id: string): Promise<BroadcastItem> {
  return request<BroadcastItem>(`/broadcasts/${id}`)
}

export async function createBroadcast(broadcast: {
  organizationId?: string
  brandId?: string
  title: string
  createdBy?: string
  bodyText: string
  bodyHtml?: string
  subjectLine?: string
  previewText?: string
  mediaType?: string
  primaryMediaUrl?: string
  mediaCaption?: string
  linkUrl?: string
  linkTitle?: string
  linkDescription?: string
  linkImageUrl?: string
  targetType?: string
  targetIds?: string[]
  channels?: string[]
  scheduledAt?: string
}): Promise<BroadcastItem> {
  return request<BroadcastItem>('/broadcasts', { method: 'POST', body: JSON.stringify({
    organization_id: broadcast.organizationId,
    brand_id: broadcast.brandId,
    title: broadcast.title,
    created_by: broadcast.createdBy || 'system',
    body_text: broadcast.bodyText,
    body_html: broadcast.bodyHtml,
    subject_line: broadcast.subjectLine,
    preview_text: broadcast.previewText,
    media_type: broadcast.mediaType || 'text',
    primary_media_url: broadcast.primaryMediaUrl,
    media_caption: broadcast.mediaCaption,
    link_url: broadcast.linkUrl,
    link_title: broadcast.linkTitle,
    link_description: broadcast.linkDescription,
    link_image_url: broadcast.linkImageUrl,
    target_type: broadcast.targetType || 'contacts',
    target_ids: JSON.stringify(broadcast.targetIds || []),
    channels: JSON.stringify(broadcast.channels || []),
    scheduled_at: broadcast.scheduledAt,
  }) })
}

export async function updateBroadcast(id: string, updates: Partial<BroadcastItem>): Promise<BroadcastItem> {
  const payload: any = {}
  if (updates.title !== undefined) payload.title = updates.title
  if (updates.bodyText !== undefined) payload.body_text = updates.bodyText
  if (updates.bodyHtml !== undefined) payload.body_html = updates.bodyHtml
  if (updates.subjectLine !== undefined) payload.subject_line = updates.subjectLine
  if (updates.previewText !== undefined) payload.preview_text = updates.previewText
  if (updates.mediaType !== undefined) payload.media_type = updates.mediaType
  if (updates.primaryMediaUrl !== undefined) payload.primary_media_url = updates.primaryMediaUrl
  if (updates.mediaCaption !== undefined) payload.media_caption = updates.mediaCaption
  if (updates.linkUrl !== undefined) payload.link_url = updates.linkUrl
  if (updates.linkTitle !== undefined) payload.link_title = updates.linkTitle
  if (updates.linkDescription !== undefined) payload.link_description = updates.linkDescription
  if (updates.linkImageUrl !== undefined) payload.link_image_url = updates.linkImageUrl
  if (updates.targetType !== undefined) payload.target_type = updates.targetType
  if (updates.targetIds !== undefined) payload.target_ids = JSON.stringify(updates.targetIds)
  if (updates.channels !== undefined) payload.channels = JSON.stringify(updates.channels)
  if (updates.scheduledAt !== undefined) payload.scheduled_at = updates.scheduledAt
  return request<BroadcastItem>(`/broadcasts/${id}`, { method: 'PUT', body: JSON.stringify(payload) })
}

export async function deleteBroadcast(id: string): Promise<void> {
  return request<void>(`/broadcasts/${id}`, { method: 'DELETE' })
}

export async function updateBroadcastStatus(id: string, status: string): Promise<BroadcastItem> {
  return request<BroadcastItem>(`/broadcasts/${id}/status/${status}`, { method: 'PUT' })
}

export async function sendBroadcast(id: string): Promise<SendBroadcastResult> {
  return request<SendBroadcastResult>(`/broadcasts/${id}/send`, { method: 'POST' })
}

export async function getFormatPreview(id: string): Promise<FormatPreviewResult> {
  return request<FormatPreviewResult>(`/broadcasts/${id}/format-preview`)
}

export async function getAvailableChannels(): Promise<ChannelFormatItem[]> {
  const res: any = await request<any>('/broadcasts/channels')
  return res.channels || []
}

export async function uploadBroadcastMedia(media: {
  broadcastId: string
  mediaType: string
  fileName: string
  fileSizeBytes: number
  mimeType: string
  fileUrl: string
  thumbnailUrl?: string
}): Promise<BroadcastMediaItem> {
  return request<BroadcastMediaItem>('/broadcasts/media', { method: 'POST', body: JSON.stringify(media) })
}

export async function listBroadcastMedia(broadcastId: string): Promise<BroadcastMediaItem[]> {
  const res: any = await request<any>(`/broadcasts/${broadcastId}/media`)
  return res.media || []
}

export async function deleteBroadcastMedia(id: string): Promise<void> {
  return request<void>(`/broadcasts/media/${id}`, { method: 'DELETE' })
}

export async function listBroadcastChannels(broadcastId: string): Promise<BroadcastChannelItem[]> {
  const res: any = await request<any>(`/broadcasts/${broadcastId}/channels`)
  return res.channels || []
}

export async function listBroadcastRecipients(broadcastId: string, status?: string, limit?: number): Promise<BroadcastRecipientItem[]> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  if (limit) params.set('limit', String(limit))
  const qs = params.toString()
  const res: any = await request<any>(`/broadcasts/${broadcastId}/recipients${qs ? '?' + qs : ''}`)
  return res.recipients || []
}

export async function listBroadcastLogs(broadcastId: string, limit?: number): Promise<BroadcastLogItem[]> {
  const params = new URLSearchParams()
  if (limit) params.set('limit', String(limit))
  const qs = params.toString()
  const res: any = await request<any>(`/broadcasts/${broadcastId}/logs${qs ? '?' + qs : ''}`)
  return res.logs || []
}

export async function getBroadcastSettings(): Promise<BroadcastSettingsItem> {
  return request<BroadcastSettingsItem>('/broadcasts/settings')
}

export async function updateBroadcastSettings(settings: Partial<BroadcastSettingsItem>): Promise<BroadcastSettingsItem> {
  const payload: any = {}
  if (settings.enabled !== undefined) payload.enabled = settings.enabled
  if (settings.maxRecipientsPerBroadcast !== undefined) payload.max_recipients_per_broadcast = settings.maxRecipientsPerBroadcast
  if (settings.defaultSendDelaySeconds !== undefined) payload.default_send_delay_seconds = settings.defaultSendDelaySeconds
  if (settings.rateLimitPerMinute !== undefined) payload.rate_limit_per_minute = settings.rateLimitPerMinute
  if (settings.allowScheduling !== undefined) payload.allow_scheduling = settings.allowScheduling
  if (settings.requireApproval !== undefined) payload.require_approval = settings.requireApproval
  if (settings.autoFormatChannels !== undefined) payload.auto_format_channels = settings.autoFormatChannels
  if (settings.trackOpens !== undefined) payload.track_opens = settings.trackOpens
  if (settings.trackClicks !== undefined) payload.track_clicks = settings.trackClicks
  if (settings.defaultChannels !== undefined) payload.default_channels = settings.defaultChannels
  return request<BroadcastSettingsItem>('/broadcasts/settings', { method: 'PUT', body: JSON.stringify(payload) })
}

export interface ChannelFormatPreview {
  channelCode: string
  channelName: string
  formattedBody: string
  formattedMediaUrl: string | null
  formattedSubject: string | null
  supportsHtml: boolean
  supportsImages: boolean
  supportsVideo: boolean
  supportsLinks: boolean
  maxLength: number
}

export async function previewHtmlForChannels(
  html: string,
  bodyText: string,
  channels?: string[]
): Promise<ChannelFormatPreview[]> {
  const res = await request<{ formats: ChannelFormatPreview[] }>('/broadcasts/html-preview', {
    method: 'POST',
    body: JSON.stringify({ html, bodyText, channels }),
  })
  return res.formats
}