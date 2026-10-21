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
}

export interface KnowledgeBaseEntry {
  id: string
  title: string
  content: string
  sourceType: string
  sourceUrl?: string | null
  fileName?: string | null
  createdAt: string
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
    request<Draft>(`/drafts/generate?conversation_id=${conversationId}`, { method: 'POST' }),

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

  // Knowledge Base & Omni-Source Learning ("Learns")
  listKnowledgeBase: () => request<KnowledgeBaseEntry[]>('/knowledge-base'),
  createKnowledgeBaseEntry: (data: { title: string; content: string; source_type?: string }) =>
    request<KnowledgeBaseEntry>('/knowledge-base', { method: 'POST', body: JSON.stringify(data) }),
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

  listContacts: (tag?: string) => request<ContactRecord[]>(`/contacts${tag ? `?tag=${tag}` : ''}`),
  listContactTags: () => request<{ id: string; name: string }[]>('/contacts/tags'),
  addContactTag: (contactId: string, tag: string) =>
    request(`/contacts/${contactId}/tags`, { method: 'POST', body: JSON.stringify({ tag }) }),
  removeContactTag: (contactId: string, tag: string) =>
    request(`/contacts/${contactId}/tags/${encodeURIComponent(tag)}`, { method: 'DELETE' }),
  importContacts: async (file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    const res = await fetch(`${BASE}/contacts/import`, { method: 'POST', body: formData })
    if (!res.ok) throw new Error(`${res.status} ${res.statusText}: ${await res.text()}`)
    return res.json() as Promise<{ created: number; skipped: number }>
  },

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
  listBroadcastRecipients: (id: string) =>
    request<BroadcastRecipient[]>(`/broadcasts/${id}/recipients`),
  getWhatsAppPricing: (region: string, category: string, volume: number) =>
    request<PricingEstimate>(
      `/broadcasts/pricing-calculator?region=${encodeURIComponent(region)}&category=${encodeURIComponent(category)}&volume=${volume}`
    ),
  seedDefaultCampaigns: () =>
    request<{ status: string; count?: number }>('/broadcasts/seed-defaults', { method: 'POST' }),
}
