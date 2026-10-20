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
  messages: Message[]
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
}
