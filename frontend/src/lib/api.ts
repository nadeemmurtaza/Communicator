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
  state: string
  humanTakeover: boolean
  assignedToId: string | null
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
  messages: Message[]
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
  createdAt: string
}

export interface AutomationRule {
  id: string
  name: string
  triggerKeyword: string
  matchType: 'contains' | 'exact'
  actionType: 'auto_reply' | 'handoff' | 'add_tag'
  actionValue: string
  priority: number
  isActive: boolean
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
  tagName: string
  status: 'draft' | 'scheduled' | 'sending' | 'completed' | 'failed'
  scheduledAt: string | null
  sentCount: number
  deliveredCount: number
  readCount: number
  failedCount: number
  createdAt: string
}

export interface BroadcastRecipient {
  id: string
  contactId: string
  status: string
  error: string
}

export interface Service {
  id: string
  name: string
  summary: string
  details?: string
  targetInstitutionTypes: string[]
  faq?: { question: string; answer: string }[]
  pricingNotes?: string
}

export interface Sequence {
  id: string
  name: string
  isActive: boolean
}

export interface VariantStats {
  sequenceId: string
  variants: { variant: string; enrolled: number; replied: number; closedWon: number }[]
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

export interface Template {
  id: string
  name: string
  status: string
  category: string
  language: string
  bodyText: string
}

export interface VerifyResult {
  status: string
  verifiedName: string
  displayNumber: string
  qualityRating: string | null
}

export const api = {
  listDrafts: (status = 'pending_review') => request<Draft[]>(`/drafts?status=${status}`),
  approveDraft: (id: string) =>
    request<{ id: string; status: string }>(`/drafts/${id}/approve`, { method: 'POST' }),
  rejectDraft: (id: string, feedback = '') =>
    request<{ id: string; status: string }>(
      `/drafts/${id}/reject${feedback ? `?feedback=${encodeURIComponent(feedback)}` : ''}`,
      { method: 'POST' },
    ),
  generateDraft: (conversationId: string) =>
    request<{ id: string; status: string; body: string }>(`/drafts/generate/${conversationId}`, {
      method: 'POST',
    }),

  listConversations: () => request<Conversation[]>('/conversations'),
  getConversation: (id: string) => request<ConversationDetail>(`/conversations/${id}`),
  enrollSequence: (id: string, sequenceId: string) =>
    request(`/conversations/${id}/enroll-sequence?sequence_id=${sequenceId}`, { method: 'POST' }),

  listServices: () => request<Service[]>('/services'),
  getService: (id: string) => request<Service>(`/services/${id}`),
  createService: (data: {
    name: string
    summary?: string
    details?: string
    target_institution_types?: string[]
  }) => request<{ id: string }>('/services', { method: 'POST', body: JSON.stringify(data) }),

  listSequences: () => request<Sequence[]>('/sequences'),
  sequenceVariantStats: (id: string) => request<VariantStats>(`/sequences/${id}/variant-stats`),

  runFollowups: () => request('/scheduler/run-followups', { method: 'POST' }),
  dispatchAutoApproved: () => request('/scheduler/dispatch-auto-approved', { method: 'POST' }),

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

  listKnowledgeBase: () => request<KnowledgeBaseEntry[]>('/knowledge-base'),
  createKnowledgeBaseEntry: (data: { title: string; content: string; source_type?: string }) =>
    request<{ id: string }>('/knowledge-base', { method: 'POST', body: JSON.stringify(data) }),
  deleteKnowledgeBaseEntry: (id: string) =>
    request(`/knowledge-base/${id}`, { method: 'DELETE' }),

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

  listBroadcasts: () => request<Broadcast[]>('/broadcasts'),
  createBroadcast: (data: {
    name: string
    template_name: string
    template_language: string
    body_preview?: string
    tag_name: string
  }) => request<Broadcast>('/broadcasts', { method: 'POST', body: JSON.stringify(data) }),
  dispatchBroadcast: (id: string) =>
    request<Broadcast>(`/broadcasts/${id}/dispatch`, { method: 'POST' }),
  listBroadcastRecipients: (id: string) =>
    request<BroadcastRecipient[]>(`/broadcasts/${id}/recipients`),
}
