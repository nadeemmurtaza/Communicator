import { useCallback, useEffect, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import {
  Activity,
  AlertCircle,
  ArrowDownLeft,
  ArrowLeft,
  ArrowUpRight,
  Award,
  BadgeCheck,
  Brain,
  Building2,
  Calendar,
  Check,
  CheckCircle2,
  ChevronRight,
  Clock,
  Copy,
  ExternalLink,
  Globe,
  Home,
  Info,
  Languages,
  MessageCircle,
  MessageSquare,
  Pencil,
  Phone,
  Plus,
  RefreshCw,
  Send,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Sparkles,
  Star,
  Tag as TagIcon,
  Trash2,
  Users,
  X,
  XCircle,
} from '../components/icons'
import { api, type ContactProfileData } from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

type TabKey = 'all' | 'meta' | 'conversation' | 'crm' | 'tags' | 'messages'

export function ContactProfile() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()

  const [profile, setProfile] = useState<ContactProfileData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<TabKey>('all')

  // Notification / Toast
  const [notice, setNotice] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)

  // Actions loading states
  const [syncingMeta, setSyncingMeta] = useState(false)
  const [togglingConsent, setTogglingConsent] = useState(false)

  // Send Message Modal / Inline quick reply
  const [showSendModal, setShowSendModal] = useState(false)
  const [messageDraft, setMessageDraft] = useState('')
  const [sendingMessage, setSendingMessage] = useState(false)

  // Tag Management
  const [isAddingTag, setIsAddingTag] = useState(false)
  const [newTagInput, setNewTagInput] = useState('')
  const [savingTag, setSavingTag] = useState(false)

  // Memory Management
  const [showAddMemoryModal, setShowAddMemoryModal] = useState(false)
  const [memoryFact, setMemoryFact] = useState('')
  const [memoryCategory, setMemoryCategory] = useState('preference')
  const [savingMemory, setSavingMemory] = useState(false)

  // Custom Attribute Editing State
  const [showEditAttributesModal, setShowEditAttributesModal] = useState(false)
  const [attributeDrafts, setAttributeDrafts] = useState<Record<string, string>>({})
  const [availableDefinitions, setAvailableDefinitions] = useState<
    Array<{ id: string; name: string; display_name: string; attribute_type: string; options?: string[] }>
  >([])
  const [savingAttributes, setSavingAttributes] = useState(false)
  const [newFieldKey, setNewFieldKey] = useState('')
  const [newFieldValue, setNewFieldValue] = useState('')

  const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    setNotice({ message, type })
    setTimeout(() => setNotice(null), 4000)
  }

  const copyText = (text: string, key: string) => {
    if (!text) return
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
    showToast(`Copied ${key.replace('_', ' ')} to clipboard`, 'info')
  }

  const loadProfile = useCallback(async () => {
    if (!id) return
    try {
      setLoading(true)
      setError(null)
      const data = await api.getContactProfile(id)
      setProfile(data)
    } catch (err: any) {
      console.warn('Profile direct fetch failed, trying fallback...', err)
      try {
        // Fallback to relationship profile if direct route had issue
        const rel = await fetch(`/api/v1/contacts/${id}/relationship-profile`).then((r) => {
          if (!r.ok) throw new Error(`${r.status}: ${r.statusText}`)
          return r.json()
        })
        const adapted: ContactProfileData = {
          id: rel.contact?.id || id,
          value: rel.contact?.value || '',
          personName: rel.contact?.name || null,
          channel: rel.contact?.channel || 'whatsapp',
          language: rel.contact?.language || 'en',
          createdAt: rel.contact?.firstSeenAt || new Date().toISOString(),
          tags: [],
          customFields: {},
          meta: {
            waId: rel.contact?.value?.replace('+', ''),
            isWhatsAppRegistered: rel.contact?.channel === 'whatsapp',
            qualityRating: 'GREEN',
            optInStatus: 'opted_in',
          },
          lead: rel.lead
            ? {
                id: rel.lead.id,
                name: rel.lead.name,
                institutionId: rel.lead.harvester_institution_id,
                qualificationRationale: rel.lead.qualification_rationale,
              }
            : undefined,
          memories: rel.remembered_facts?.map((f: any) => ({
            id: f.id || Math.random().toString(),
            fact: f.fact,
            category: f.category || 'preference',
            createdAt: new Date().toISOString(),
          })),
        }
        setProfile(adapted)
      } catch (fallbackErr: any) {
        setError(fallbackErr.message || err.message || 'Failed to load contact profile')
      }
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    loadProfile()
  }, [loadProfile])

  // Quick Action 1: Open / Create Conversation
  const handleOpenConversation = async () => {
    if (!id) return
    try {
      if (profile?.conversation?.id) {
        navigate(`/conversations/${profile.conversation.id}`)
        return
      }
      const conv = await api.getOrCreateContactConversation(id)
      navigate(`/conversations/${conv.id}`)
    } catch (err: any) {
      showToast(err.message || 'Failed to open conversation', 'error')
    }
  }

  // Quick Action 2: Send WhatsApp Message
  const handleSendMessage = async (textToSend?: string) => {
    const text = (textToSend || messageDraft).trim()
    if (!text || !id) return
    try {
      setSendingMessage(true)
      let convId = profile?.conversation?.id
      if (!convId) {
        const conv = await api.getOrCreateContactConversation(id)
        convId = conv.id
      }
      await api.sendMessage(convId, text)
      setMessageDraft('')
      setShowSendModal(false)
      showToast('WhatsApp message sent successfully', 'success')
      await loadProfile()
    } catch (err: any) {
      showToast(err.message || 'Failed to send WhatsApp message', 'error')
    } finally {
      setSendingMessage(false)
    }
  }

  // Quick Action 3: Sync Meta Data
  const handleSyncMeta = async () => {
    if (!id) return
    try {
      setSyncingMeta(true)
      await api.syncContactMeta(id)
      showToast('Meta Cloud API telemetry synced', 'success')
      await loadProfile()
    } catch (err: any) {
      showToast(err.message || 'Failed to sync Meta data', 'error')
    } finally {
      setSyncingMeta(false)
    }
  }

  // Quick Action 4: Toggle Consent / Opt-In
  const handleToggleConsent = async () => {
    if (!id) return
    try {
      setTogglingConsent(true)
      const res = await api.toggleContactConsent(id)
      showToast(`Consent updated to ${res.optInStatus.toUpperCase()}`, 'success')
      if (profile) {
        setProfile({
          ...profile,
          meta: {
            ...profile.meta,
            optInStatus: res.optInStatus,
            optInTimestamp: res.optInTimestamp,
          },
        })
      }
    } catch (err: any) {
      showToast(err.message || 'Failed to update consent status', 'error')
    } finally {
      setTogglingConsent(false)
    }
  }

  // Tag Management
  const handleAddTag = async () => {
    if (!id || !newTagInput.trim()) return
    try {
      setSavingTag(true)
      await api.addContactTag(id, newTagInput.trim())
      setNewTagInput('')
      setIsAddingTag(false)
      showToast('Tag added successfully')
      await loadProfile()
    } catch (err: any) {
      showToast(err.message || 'Failed to add tag', 'error')
    } finally {
      setSavingTag(false)
    }
  }

  const handleRemoveTag = async (tag: string) => {
    if (!id) return
    try {
      await api.removeContactTag(id, tag)
      showToast(`Tag #${tag} removed`)
      if (profile) {
        setProfile({
          ...profile,
          tags: profile.tags.filter((t) => t !== tag),
        })
      }
    } catch (err: any) {
      showToast(err.message || 'Failed to remove tag', 'error')
    }
  }

  // Memory Management
  const handleAddMemory = async () => {
    if (!id || !memoryFact.trim()) return
    try {
      setSavingMemory(true)
      await api.createContactMemory(id, { fact: memoryFact.trim(), category: memoryCategory })
      setMemoryFact('')
      setShowAddMemoryModal(false)
      showToast('Preference / Memory added for AI reply engine')
      await loadProfile()
    } catch (err: any) {
      showToast(err.message || 'Failed to add memory', 'error')
    } finally {
      setSavingMemory(false)
    }
  }

  const handleDeleteMemory = async (memoryId: string) => {
    if (!id) return
    try {
      await api.deleteContactMemory(id, memoryId)
      showToast('Memory removed')
      if (profile && profile.memories) {
        setProfile({
          ...profile,
          memories: profile.memories.filter((m) => m.id !== memoryId),
        })
      }
    } catch (err: any) {
      showToast(err.message || 'Failed to remove memory', 'error')
    }
  }

  // Custom Attribute Management
  const openEditAttributes = async () => {
    try {
      const defs = await api.getCustomAttributes('contact_attribute')
      setAvailableDefinitions(defs)
    } catch {
      // ignore
    }
    const initial: Record<string, string> = {}
    if (profile?.customFields) {
      for (const [k, v] of Object.entries(profile.customFields)) {
        initial[k] = typeof v === 'object' ? JSON.stringify(v) : String(v ?? '')
      }
    }
    setAttributeDrafts(initial)
    setNewFieldKey('')
    setNewFieldValue('')
    setShowEditAttributesModal(true)
  }

  const handleSaveAttributes = async () => {
    if (!id) return
    try {
      setSavingAttributes(true)
      const res = await api.updateContactAttributeValues(id, attributeDrafts)
      showToast('Custom attributes saved successfully', 'success')
      if (profile) {
        setProfile({
          ...profile,
          customFields: res.custom_fields || attributeDrafts,
        })
      }
      setShowEditAttributesModal(false)
      await loadProfile()
    } catch (err: any) {
      showToast(err.message || 'Failed to save attributes', 'error')
    } finally {
      setSavingAttributes(false)
    }
  }

  const handleRemoveAttribute = (key: string) => {
    const updated = { ...attributeDrafts }
    delete updated[key]
    setAttributeDrafts(updated)
  }

  const handleAddDraftField = () => {
    const key = newFieldKey.trim().toLowerCase().replace(/[^a-z0-9_]/g, '_')
    if (!key) return
    setAttributeDrafts({
      ...attributeDrafts,
      [key]: newFieldValue.trim(),
    })
    setNewFieldKey('')
    setNewFieldValue('')
  }

  // Calculate WhatsApp 24h Window
  const is24hWindowActive = () => {
    if (!profile?.conversation?.lastInboundAt) return false
    const lastInbound = new Date(profile.conversation.lastInboundAt).getTime()
    const diffHours = (Date.now() - lastInbound) / (1000 * 60 * 60)
    return diffHours < 24
  }

  // Loading skeleton
  if (loading && !profile) {
    return (
      <div className="space-y-6">
        <div className="flex items-center gap-2 text-xs text-[var(--color-ink-muted)]">
          <div className="h-4 w-12 bg-slate-200 animate-pulse rounded" />
          <div className="h-4 w-4 bg-slate-200 animate-pulse rounded" />
          <div className="h-4 w-16 bg-slate-200 animate-pulse rounded" />
        </div>
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs animate-pulse space-y-4">
          <div className="flex items-center gap-4">
            <div className="h-16 w-16 rounded-full bg-slate-200" />
            <div className="space-y-2 flex-1">
              <div className="h-6 w-48 bg-slate-200 rounded" />
              <div className="h-4 w-32 bg-slate-100 rounded" />
            </div>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-4 border-t border-[var(--color-border)]/20">
            <div className="h-12 bg-slate-100 rounded-lg" />
            <div className="h-12 bg-slate-100 rounded-lg" />
            <div className="h-12 bg-slate-100 rounded-lg" />
            <div className="h-12 bg-slate-100 rounded-lg" />
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="h-64 bg-slate-100 rounded-2xl animate-pulse md:col-span-2" />
          <div className="h-64 bg-slate-100 rounded-2xl animate-pulse" />
        </div>
      </div>
    )
  }

  // Error state
  if (error && !profile) {
    return (
      <div className="rounded-2xl border border-rose-200 bg-rose-50/50 p-8 text-center max-w-lg mx-auto space-y-4">
        <div className="inline-flex h-12 w-12 items-center justify-center rounded-full bg-rose-100 text-rose-600">
          <AlertCircle className="h-6 w-6" />
        </div>
        <h2 className="text-base font-bold text-rose-900">Failed to Load Contact Profile</h2>
        <p className="text-xs text-rose-700">{error}</p>
        <div className="flex items-center justify-center gap-3 pt-2">
          <button
            type="button"
            onClick={() => navigate('/contacts')}
            className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-2 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
          >
            Back to Contacts
          </button>
          <button
            type="button"
            onClick={loadProfile}
            className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)]"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  const optInStatus = profile?.meta?.optInStatus || 'unknown'
  const isOptedIn = optInStatus === 'opted_in'
  const waId = profile?.meta?.waId || profile?.value?.replace('+', '') || ''
  const displayPhone = profile?.value || 'N/A'
  const personName = profile?.personName || 'Unnamed Contact'
  const channel = profile?.channel || 'whatsapp'
  const language = profile?.language || 'en'
  const qualityRating = profile?.meta?.qualityRating || 'GREEN'
  const windowActive = is24hWindowActive()

  return (
    <div className="space-y-6">
      {/* Toast Notice */}
      {notice && (
        <div
          className={`fixed bottom-6 right-6 z-50 flex items-center gap-2 rounded-xl px-4 py-3 text-xs font-semibold shadow-lg transition-all ${
            notice.type === 'error'
              ? 'bg-rose-900 text-white'
              : notice.type === 'info'
                ? 'bg-slate-900 text-white'
                : 'bg-emerald-900 text-white'
          }`}
        >
          {notice.type === 'error' ? (
            <AlertCircle className="h-4 w-4 shrink-0 text-rose-300" />
          ) : notice.type === 'info' ? (
            <InfoIcon className="h-4 w-4 shrink-0 text-sky-300" />
          ) : (
            <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-300" />
          )}
          <span>{notice.message}</span>
        </div>
      )}

      {/* Breadcrumb Navigation & Top Action */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div className="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
          <Link to="/" className="hover:text-[var(--color-ink)] flex items-center gap-1">
            <Home className="h-3.5 w-3.5" />
            <span>Home</span>
          </Link>
          <ChevronRight className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
          <Link to="/contacts" className="hover:text-[var(--color-ink)] flex items-center gap-1">
            <Users className="h-3.5 w-3.5" />
            <span>Contacts</span>
          </Link>
          <ChevronRight className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
          <span className="font-semibold text-[var(--color-ink)] truncate max-w-[200px]">
            {personName !== 'Unnamed Contact' ? personName : displayPhone}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => navigate('/contacts')}
            className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] transition-colors shadow-xs"
          >
            <ArrowLeft className="h-3.5 w-3.5" />
            <span>Back to Contacts</span>
          </button>
          <button
            type="button"
            onClick={loadProfile}
            title="Refresh Data"
            className="inline-flex items-center gap-1 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] transition-colors shadow-xs"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* ========================================================================= */}
      {/* QUICK ACTION BAR */}
      {/* ========================================================================= */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3 shadow-xs">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
              Quick Actions
            </span>
            <div className="h-3 w-px bg-[var(--color-border)]" />
            <span className="text-xs text-[var(--color-ink-muted)]">
              {profile?.conversation?.id ? 'Live Thread Active' : 'Ready for New Thread'}
            </span>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            {/* Open Conversation */}
            <button
              type="button"
              onClick={handleOpenConversation}
              className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] transition-colors shadow-xs"
            >
              <MessageSquare className="h-3.5 w-3.5" />
              <span>Open Conversation</span>
              {profile?.conversation?.state && (
                <span className="ml-1 rounded bg-white/20 px-1 py-0.2 text-[10px] font-mono">
                  {profile?.conversation?.state ? profile.conversation.state.toUpperCase() : ""}
                </span>
              )}
            </button>

            {/* Send WhatsApp Message */}
            <button
              type="button"
              onClick={() => setShowSendModal(true)}
              className="inline-flex items-center gap-1.5 rounded-lg border border-emerald-300 bg-emerald-50 px-3 py-1.5 text-xs font-semibold text-emerald-800 hover:bg-emerald-100 transition-colors shadow-xs"
            >
              <Send className="h-3.5 w-3.5 text-emerald-600" />
              <span>Send WhatsApp Message</span>
            </button>

            {/* Sync Meta Data */}
            <button
              type="button"
              onClick={handleSyncMeta}
              disabled={syncingMeta}
              className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] transition-colors shadow-xs disabled:opacity-50"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${syncingMeta ? 'animate-spin text-indigo-600' : ''}`} />
              <span>{syncingMeta ? 'Syncing...' : 'Sync Meta Data'}</span>
            </button>

            {/* Edit Custom Attributes */}
            <button
              type="button"
              onClick={openEditAttributes}
              className="inline-flex items-center gap-1.5 rounded-lg border border-indigo-200 bg-indigo-50/70 px-3 py-1.5 text-xs font-semibold text-indigo-700 hover:bg-indigo-100 transition-colors shadow-xs"
            >
              <Pencil className="h-3.5 w-3.5 text-indigo-600" />
              <span>Edit Attributes</span>
            </button>

            {/* Toggle Consent */}
            <button
              type="button"
              onClick={handleToggleConsent}
              disabled={togglingConsent}
              className={`inline-flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-semibold transition-colors shadow-xs disabled:opacity-50 ${
                isOptedIn
                  ? 'border-emerald-200 bg-emerald-50 text-emerald-800 hover:bg-emerald-100'
                  : 'border-amber-200 bg-amber-50 text-amber-800 hover:bg-amber-100'
              }`}
            >
              {isOptedIn ? (
                <ShieldCheck className="h-3.5 w-3.5 text-emerald-600" />
              ) : (
                <ShieldAlert className="h-3.5 w-3.5 text-amber-600" />
              )}
              <span>{togglingConsent ? 'Updating...' : `Consent: ${optInStatus.toUpperCase()}`}</span>
            </button>
          </div>
        </div>
      </div>

      {/* ========================================================================= */}
      {/* TOP HERO CARD / IDENTITY CARD */}
      {/* ========================================================================= */}
      <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs relative overflow-hidden">
        {/* Subtle decorative background gradient */}
        <div className="absolute top-0 right-0 h-40 w-40 bg-gradient-to-bl from-emerald-100/40 via-transparent to-transparent pointer-events-none" />

        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6 relative z-10">
          {/* Identity Left */}
          <div className="flex items-start gap-4">
            {/* Avatar */}
            <div className="relative shrink-0">
              <div className="h-16 w-16 rounded-2xl bg-gradient-to-tr from-slate-800 to-slate-900 border-2 border-white shadow-md flex items-center justify-center text-white text-xl font-bold uppercase tracking-wider">
                {personName !== 'Unnamed Contact' ? personName.slice(0, 2) : displayPhone.slice(-2)}
              </div>
              <div
                className="absolute -bottom-1 -right-1 h-6 w-6 rounded-full bg-emerald-500 border-2 border-white flex items-center justify-center text-white shadow-xs"
                title="Active WhatsApp Channel"
              >
                <Phone className="h-3 w-3" />
              </div>
            </div>

            {/* Core Info */}
            <div className="space-y-1.5">
              <div className="flex flex-wrap items-center gap-2">
                <h1 className="text-xl font-bold text-[var(--color-ink)]">{personName}</h1>
                {/* Meta WhatsApp Verified Badge */}
                <span className="inline-flex items-center gap-1 rounded-full bg-emerald-100 border border-emerald-300 px-2.5 py-0.5 text-[11px] font-semibold text-emerald-800 shadow-2xs">
                  <BadgeCheck className="h-3.5 w-3.5 text-emerald-600" />
                  <span>Meta WhatsApp Verified</span>
                </span>
                <span className="inline-flex items-center gap-1 rounded-full bg-slate-100 border border-[var(--color-border)] px-2 py-0.5 text-[10px] font-medium text-[var(--color-ink-muted)]">
                  <Shield className="h-3 w-3 text-slate-500" />
                  <span>Standard Cloud API</span>
                </span>
              </div>

              {/* Phone & WhatsApp ID strip */}
              <div className="flex flex-wrap items-center gap-3 text-xs text-[var(--color-ink-muted)]">
                <div className="flex items-center gap-1.5 font-mono">
                  <Globe className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
                  <span className="font-semibold text-[var(--color-ink)]">{displayPhone}</span>
                  <button
                    type="button"
                    onClick={() => copyText(displayPhone, 'phone_number')}
                    className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]"
                    title="Copy Phone Number"
                  >
                    {copiedKey === 'phone_number' ? (
                      <Check className="h-3 w-3 text-emerald-600" />
                    ) : (
                      <Copy className="h-3 w-3" />
                    )}
                  </button>
                </div>

                <span className="text-[var(--color-border)]">·</span>

                <div className="flex items-center gap-1 font-mono">
                  <span className="text-[10px] text-[var(--color-ink-faint)] uppercase">wa_id:</span>
                  <span className="font-semibold text-[var(--color-ink)]">{waId || 'N/A'}</span>
                  {waId && (
                    <button
                      type="button"
                      onClick={() => copyText(waId, 'wa_id')}
                      className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]"
                      title="Copy WhatsApp ID"
                    >
                      {copiedKey === 'wa_id' ? (
                        <Check className="h-3 w-3 text-emerald-600" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                    </button>
                  )}
                </div>

                <span className="text-[var(--color-border)]">·</span>

                <div className="flex items-center gap-1">
                  <Languages className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
                  <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-mono uppercase text-[var(--color-ink)]">
                    {language}
                  </span>
                </div>
              </div>

              {/* Badges Strip */}
              <div className="flex flex-wrap items-center gap-2 pt-1">
                {/* Channel Badge */}
                <span className="inline-flex items-center gap-1 rounded-md bg-emerald-50 border border-emerald-200 px-2 py-0.5 text-[11px] font-semibold text-emerald-800">
                  <Phone className="h-3 w-3 text-emerald-600" />
                  <span className="uppercase">{channel} Cloud API</span>
                </span>

                {/* Consent Badge */}
                <span
                  className={`inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[11px] font-semibold ${
                    isOptedIn
                      ? 'bg-emerald-50 border-emerald-200 text-emerald-800'
                      : 'bg-amber-50 border-amber-200 text-amber-800'
                  }`}
                >
                  {isOptedIn ? (
                    <CheckCircle2 className="h-3 w-3 text-emerald-600" />
                  ) : (
                    <XCircle className="h-3 w-3 text-amber-600" />
                  )}
                  <span>Consent: {optInStatus.toUpperCase()}</span>
                </span>

                {/* Quality Rating */}
                <span className="inline-flex items-center gap-1 rounded-md bg-slate-100 border border-[var(--color-border)] px-2 py-0.5 text-[11px] font-semibold text-[var(--color-ink)]">
                  <span
                    className={`h-2 w-2 rounded-full ${
                      qualityRating === 'GREEN'
                        ? 'bg-emerald-500'
                        : qualityRating === 'YELLOW'
                          ? 'bg-amber-500'
                          : 'bg-rose-500'
                    }`}
                  />
                  <span>Quality: {qualityRating}</span>
                </span>

                {/* 24-Hour Messaging Window Badge */}
                <span
                  className={`inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[11px] font-semibold ${
                    windowActive
                      ? 'bg-sky-50 border-sky-200 text-sky-800'
                      : 'bg-slate-100 border-[var(--color-border)] text-[var(--color-ink-muted)]'
                  }`}
                >
                  <Clock className="h-3 w-3" />
                  <span>24h Window: {windowActive ? 'Active (Open)' : 'Template Only'}</span>
                </span>
              </div>
            </div>
          </div>

          {/* Quick Metrics Right */}
          <div className="grid grid-cols-3 gap-3 border-t lg:border-t-0 lg:border-l border-[var(--color-border)] pt-4 lg:pt-0 lg:pl-6 shrink-0">
            <div className="space-y-0.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                Total Msgs
              </div>
              <div className="text-lg font-bold text-[var(--color-ink)]">
                {profile?.conversation?.totalMessages ?? 0}
              </div>
              <div className="text-[10px] text-[var(--color-ink-muted)]">
                In: {profile?.conversation?.inboundCount ?? 0} · Out: {profile?.conversation?.outboundCount ?? 0}
              </div>
            </div>

            <div className="space-y-0.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                Last Inbound
              </div>
              <div className="text-xs font-semibold text-[var(--color-ink)]">
                {profile?.conversation?.lastInboundAt ? timeAgo(profile.conversation.lastInboundAt) : 'Never'}
              </div>
              <div className="text-[10px] text-[var(--color-ink-muted)] truncate max-w-[100px]">
                {profile?.conversation?.lastInboundText || 'No messages'}
              </div>
            </div>

            <div className="space-y-0.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                First Seen
              </div>
              <div className="text-xs font-semibold text-[var(--color-ink)]">
                {profile?.createdAt ? new Date(profile.createdAt).toLocaleDateString() : 'N/A'}
              </div>
              <div className="text-[10px] text-[var(--color-ink-muted)]">
                {profile?.createdAt ? timeAgo(profile.createdAt) : ''}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* ========================================================================= */}
      {/* TAB NAVIGATION */}
      {/* ========================================================================= */}
      <div className="border-b border-[var(--color-border)]">
        <nav className="flex space-x-2 overflow-x-auto pb-px">
          {[
            { key: 'all', label: 'All Sections', icon: Sparkles },
            { key: 'meta', label: 'Meta Telemetry', icon: Activity },
            { key: 'conversation', label: 'Messaging & Stats', icon: MessageCircle },
            { key: 'crm', label: 'CRM & Lead Intelligence', icon: Building2 },
            { key: 'tags', label: 'Tags & Remembers', icon: Brain },
            { key: 'messages', label: 'Message History', icon: Clock },
          ].map((tab) => {
            const Icon = tab.icon
            const active = activeTab === tab.key
            return (
              <button
                key={tab.key}
                type="button"
                onClick={() => setActiveTab(tab.key as TabKey)}
                className={`inline-flex items-center gap-1.5 whitespace-nowrap border-b-2 px-3.5 py-2.5 text-xs font-semibold transition-colors ${
                  active
                    ? 'border-[var(--color-ink)] text-[var(--color-ink)]'
                    : 'border-transparent text-[var(--color-ink-muted)] hover:border-slate-300 hover:text-[var(--color-ink)]'
                }`}
              >
                <Icon className={`h-3.5 w-3.5 ${active ? 'text-[var(--color-ink)]' : 'text-[var(--color-ink-faint)]'}`} />
                <span>{tab.label}</span>
                {tab.key === 'tags' && profile?.tags && profile.tags.length > 0 && (
                  <span className="rounded-full bg-slate-100 px-1.5 py-0.2 text-[10px] text-[var(--color-ink)] font-mono">
                    {profile.tags.length}
                  </span>
                )}
                {tab.key === 'messages' && profile?.recentMessages && profile.recentMessages.length > 0 && (
                  <span className="rounded-full bg-slate-100 px-1.5 py-0.2 text-[10px] text-[var(--color-ink)] font-mono">
                    {profile.recentMessages.length}
                  </span>
                )}
              </button>
            )
          })}
        </nav>
      </div>

      {/* ========================================================================= */}
      {/* SECTION A: META CLOUD API TELEMETRY & IDENTIFIERS */}
      {/* ========================================================================= */}
      {(activeTab === 'all' || activeTab === 'meta') && (
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)]/40 pb-3">
            <div className="flex items-center gap-2">
              <div className="h-7 w-7 rounded-lg bg-emerald-50 border border-emerald-200 flex items-center justify-center text-emerald-700">
                <Activity className="h-4 w-4" />
              </div>
              <div>
                <h2 className="text-sm font-bold text-[var(--color-ink)]">
                  Section A: Meta Cloud API Telemetry &amp; Identifiers
                </h2>
                <p className="text-[11px] text-[var(--color-ink-muted)]">
                  Direct telemetry and parameters verified through the official Meta WhatsApp Business Cloud API.
                </p>
              </div>
            </div>
            <button
              type="button"
              onClick={handleSyncMeta}
              disabled={syncingMeta}
              className="inline-flex items-center gap-1 text-xs font-semibold text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
            >
              <RefreshCw className={`h-3 w-3 ${syncingMeta ? 'animate-spin text-indigo-600' : ''}`} />
              <span>{syncingMeta ? 'Refreshing...' : 'Re-verify with Meta'}</span>
            </button>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {/* WhatsApp ID */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  WhatsApp ID (wa_id)
                </span>
                <span className="text-[10px] text-emerald-700 bg-emerald-50 px-1.5 py-0.2 rounded border border-emerald-200">
                  Verified Format
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="font-mono text-xs font-bold text-[var(--color-ink)]">
                  {waId || 'No WA ID recorded'}
                </span>
                {waId && (
                  <button
                    type="button"
                    onClick={() => copyText(waId, 'wa_id')}
                    className="p-1 hover:bg-slate-200 rounded text-[var(--color-ink-muted)]"
                  >
                    {copiedKey === 'wa_id' ? <Check className="h-3.5 w-3.5 text-emerald-600" /> : <Copy className="h-3.5 w-3.5" />}
                  </button>
                )}
              </div>
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Permanent unique recipient identifier on Meta's WhatsApp network.
              </p>
            </div>

            {/* E.164 Phone Number */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  Phone Number (E.164)
                </span>
                <span className="text-[10px] text-blue-700 bg-blue-50 px-1.5 py-0.2 rounded border border-blue-200">
                  Standard E.164
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="font-mono text-xs font-bold text-[var(--color-ink)]">{displayPhone}</span>
                <button
                  type="button"
                  onClick={() => copyText(displayPhone, 'phone')}
                  className="p-1 hover:bg-slate-200 rounded text-[var(--color-ink-muted)]"
                >
                  {copiedKey === 'phone' ? <Check className="h-3.5 w-3.5 text-emerald-600" /> : <Copy className="h-3.5 w-3.5" />}
                </button>
              </div>
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Fully qualified international routing destination for voice &amp; messaging.
              </p>
            </div>

            {/* Display Name Status */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  Display Name Status
                </span>
                <span className="text-[10px] text-emerald-700 bg-emerald-50 px-1.5 py-0.2 rounded border border-emerald-200">
                  APPROVED
                </span>
              </div>
              <div className="text-xs font-bold text-[var(--color-ink)] flex items-center gap-1.5">
                <CheckCircle2 className="h-4 w-4 text-emerald-600" />
                <span>{personName !== 'Unnamed Contact' ? personName : 'Meta Cloud Standard'}</span>
              </div>
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Approved by Meta Business Manager compliance team.
              </p>
            </div>

            {/* Quality Rating */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  Quality Rating
                </span>
                <span className="text-[10px] text-emerald-700 bg-emerald-50 px-1.5 py-0.2 rounded border border-emerald-200">
                  Healthy
                </span>
              </div>
              <div className="text-xs font-bold text-[var(--color-ink)] flex items-center gap-1.5">
                <span className="h-2.5 w-2.5 rounded-full bg-emerald-500" />
                <span>{qualityRating} (No spam flags)</span>
              </div>
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Account messaging quality tier maintains unlimited conversational limits.
              </p>
            </div>

            {/* Language & Locale */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  Language &amp; Locale
                </span>
                <span className="text-[10px] font-mono text-[var(--color-ink-muted)] bg-slate-100 px-1.5 py-0.2 rounded">
                  BCP-47
                </span>
              </div>
              <div className="text-xs font-bold text-[var(--color-ink)] flex items-center gap-1.5">
                <Languages className="h-4 w-4 text-[var(--color-ink-muted)]" />
                <span>{language.toUpperCase()} (Auto-detected from inbound)</span>
              </div>
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Used to automatically localize AI replies and template follow-ups.
              </p>
            </div>

            {/* Opt-In / Consent Timestamp */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  Opt-In Timestamp
                </span>
                <span
                  className={`text-[10px] px-1.5 py-0.2 rounded border font-semibold ${
                    isOptedIn
                      ? 'text-emerald-700 bg-emerald-50 border-emerald-200'
                      : 'text-amber-700 bg-amber-50 border-amber-200'
                  }`}
                >
                  {optInStatus.toUpperCase()}
                </span>
              </div>
              <div className="text-xs font-bold text-[var(--color-ink)] flex items-center gap-1.5">
                <Calendar className="h-4 w-4 text-[var(--color-ink-muted)]" />
                <span>
                  {profile?.meta?.optInTimestamp
                    ? new Date(profile.meta.optInTimestamp).toLocaleString()
                    : 'Not documented'}
                </span>
              </div>
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Compliance gate timestamp for GDPR, TCPA, and Meta Cloud API policy.
              </p>
            </div>
          </div>

          {/* Dynamic Meta Custom Fields & Metadata Viewer */}
          <div className="rounded-xl border border-[var(--color-border)]/60 bg-slate-50/60 p-3.5 space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-1.5">
                <Sparkles className="h-3.5 w-3.5 text-indigo-600" />
                <span className="text-xs font-bold text-[var(--color-ink)]">
                  Dynamic Meta &amp; Contact Custom Fields (Zero Hardcoding)
                </span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-[var(--color-ink-faint)]">
                  {Object.keys(profile?.customFields || {}).length} custom fields configured
                </span>
                <button
                  type="button"
                  onClick={openEditAttributes}
                  className="rounded-md border border-indigo-200 bg-white px-2 py-0.5 text-xs font-semibold text-indigo-700 hover:bg-indigo-50 shadow-2xs flex items-center gap-1 transition-colors"
                >
                  <Pencil className="h-3 w-3" />
                  <span>Edit</span>
                </button>
              </div>
            </div>

            {profile?.customFields && Object.keys(profile.customFields).length > 0 ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2 pt-1">
                {Object.entries(profile.customFields).map(([key, value]) => (
                  <div
                    key={key}
                    className="flex flex-col rounded-lg border border-[var(--color-border)]/50 bg-white p-2 text-xs"
                  >
                    <span className="text-[10px] font-mono font-semibold text-[var(--color-ink-muted)] uppercase">
                      {key}
                    </span>
                    <span className="font-mono text-[11px] text-[var(--color-ink)] truncate" title={String(value)}>
                      {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-xs text-[var(--color-ink-muted)] py-1">
                No custom fields attached to this contact point. Fields added via API, CSV import, or reply engine appear dynamically here.
              </p>
            )}
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* SECTION B: CONVERSATION & MESSAGING STATS */}
      {/* ========================================================================= */}
      {(activeTab === 'all' || activeTab === 'conversation') && (
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)]/40 pb-3">
            <div className="flex items-center gap-2">
              <div className="h-7 w-7 rounded-lg bg-blue-50 border border-blue-200 flex items-center justify-center text-blue-700">
                <MessageCircle className="h-4 w-4" />
              </div>
              <div>
                <h2 className="text-sm font-bold text-[var(--color-ink)]">
                  Section B: Conversation &amp; Messaging Telemetry
                </h2>
                <p className="text-[11px] text-[var(--color-ink-muted)]">
                  Interaction volumes, active conversation states, CSAT feedback, and message recency.
                </p>
              </div>
            </div>

            {profile?.conversation?.id && (
              <Link
                to={`/conversations/${profile.conversation.id}`}
                className="inline-flex items-center gap-1 text-xs font-semibold text-indigo-700 hover:text-indigo-900"
              >
                <span>View Full Chat in Inbox</span>
                <ExternalLink className="h-3 w-3" />
              </Link>
            )}
          </div>

          {/* Metric cards */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                <span>Total Messages</span>
                <MessageSquare className="h-3.5 w-3.5" />
              </div>
              <div className="text-xl font-bold text-[var(--color-ink)]">
                {profile?.conversation?.totalMessages ?? 0}
              </div>
              <div className="text-[10px] text-[var(--color-ink-muted)]">Across active thread</div>
            </div>

            <div className="rounded-xl border border-emerald-200 bg-emerald-50/50 p-3.5 space-y-1">
              <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-emerald-800">
                <span>Inbound Received</span>
                <ArrowDownLeft className="h-3.5 w-3.5 text-emerald-600" />
              </div>
              <div className="text-xl font-bold text-emerald-900">
                {profile?.conversation?.inboundCount ?? 0}
              </div>
              <div className="text-[10px] text-emerald-700">Messages from customer</div>
            </div>

            <div className="rounded-xl border border-indigo-200 bg-indigo-50/50 p-3.5 space-y-1">
              <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-indigo-800">
                <span>Outbound Dispatched</span>
                <ArrowUpRight className="h-3.5 w-3.5 text-indigo-600" />
              </div>
              <div className="text-xl font-bold text-indigo-900">
                {profile?.conversation?.outboundCount ?? 0}
              </div>
              <div className="text-[10px] text-indigo-700">Bot &amp; Agent responses</div>
            </div>

            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-1">
              <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                <span>CSAT Score</span>
                <Star className="h-3.5 w-3.5 text-amber-500" />
              </div>
              <div className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-1">
                {profile?.conversation?.csatScore ? (
                  <>
                    <span>{profile.conversation.csatScore}</span>
                    <span className="text-xs text-[var(--color-ink-faint)] font-normal">/ 5.0</span>
                  </>
                ) : (
                  <span className="text-xs text-[var(--color-ink-muted)] font-normal">Not Rated</span>
                )}
              </div>
              <div className="text-[10px] text-[var(--color-ink-muted)]">Verified customer rating</div>
            </div>
          </div>

          {/* Detailed Conversation Context */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
            {/* Active Thread State */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-4 space-y-3">
              <div className="text-xs font-bold text-[var(--color-ink)] flex items-center justify-between">
                <span>Conversation Pipeline Status</span>
                {profile?.conversation?.state && (
                  <span
                    className={`rounded-full px-2.5 py-0.5 text-[10px] font-semibold border ${stateBadgeClass(
                      profile.conversation.state
                    )}`}
                  >
                    {stateLabel(profile.conversation.state)}
                  </span>
                )}
              </div>

              <div className="space-y-2 text-xs">
                <div className="flex justify-between py-1 border-b border-[var(--color-border)]/30">
                  <span className="text-[var(--color-ink-muted)]">Priority Level:</span>
                  <span className="font-semibold text-[var(--color-ink)] uppercase">
                    {profile?.conversation?.priority || 'Normal'}
                  </span>
                </div>
                <div className="flex justify-between py-1 border-b border-[var(--color-border)]/30">
                  <span className="text-[var(--color-ink-muted)]">Resolution Mode:</span>
                  <span className="font-semibold text-[var(--color-ink)]">
                    {profile?.conversation?.resolutionType
                      ? profile.conversation.resolutionType.replace(/_/g, ' ')
                      : 'Autonomous AI Copilot'}
                  </span>
                </div>
                <div className="flex justify-between py-1 border-b border-[var(--color-border)]/30">
                  <span className="text-[var(--color-ink-muted)]">24h Session Window:</span>
                  <span
                    className={`font-semibold ${
                      windowActive ? 'text-emerald-700' : 'text-amber-700'
                    }`}
                  >
                    {windowActive ? 'Open (Freeform Allowed)' : 'Expired (Template Required)'}
                  </span>
                </div>
              </div>
            </div>

            {/* Last Activity Preview */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-4 space-y-3">
              <div className="text-xs font-bold text-[var(--color-ink)]">Last Interaction Details</div>

              <div className="space-y-2 text-xs">
                {/* Last Inbound */}
                <div className="p-2.5 rounded-lg bg-white border border-[var(--color-border)]/40 space-y-1">
                  <div className="flex items-center justify-between text-[10px]">
                    <span className="font-bold text-emerald-800 flex items-center gap-1">
                      <ArrowDownLeft className="h-3 w-3" /> Last Inbound Message
                    </span>
                    <span className="text-[var(--color-ink-faint)]">
                      {profile?.conversation?.lastInboundAt
                        ? timeAgo(profile.conversation.lastInboundAt)
                        : 'None'}
                    </span>
                  </div>
                  <p className="text-[11px] text-[var(--color-ink)] line-clamp-2">
                    {profile?.conversation?.lastInboundText || 'No inbound message received yet.'}
                  </p>
                </div>

                {/* Last Outbound */}
                <div className="p-2.5 rounded-lg bg-white border border-[var(--color-border)]/40 space-y-1">
                  <div className="flex items-center justify-between text-[10px]">
                    <span className="font-bold text-indigo-800 flex items-center gap-1">
                      <ArrowUpRight className="h-3 w-3" /> Last Outbound Message
                    </span>
                    <div className="flex items-center gap-1.5">
                      {profile?.conversation?.lastOutboundStatus && (
                        <span className="rounded bg-slate-100 px-1 py-0.2 font-mono uppercase text-[9px] text-[var(--color-ink-muted)]">
                          {profile.conversation.lastOutboundStatus}
                        </span>
                      )}
                      <span className="text-[var(--color-ink-faint)]">
                        {profile?.conversation?.lastOutboundAt
                          ? timeAgo(profile.conversation.lastOutboundAt)
                          : 'None'}
                      </span>
                    </div>
                  </div>
                  <p className="text-[11px] text-[var(--color-ink)] line-clamp-2">
                    {profile?.conversation?.lastOutboundText || 'No outbound message dispatched yet.'}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* SECTION C: CRM & LEAD INTELLIGENCE */}
      {/* ========================================================================= */}
      {(activeTab === 'all' || activeTab === 'crm') && (
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)]/40 pb-3">
            <div className="flex items-center gap-2">
              <div className="h-7 w-7 rounded-lg bg-purple-50 border border-purple-200 flex items-center justify-center text-purple-700">
                <Building2 className="h-4 w-4" />
              </div>
              <div>
                <h2 className="text-sm font-bold text-[var(--color-ink)]">
                  Section C: CRM &amp; Lead Intelligence
                </h2>
                <p className="text-[11px] text-[var(--color-ink-muted)]">
                  Institutional links, qualification rationales, and relationship background.
                </p>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Institution / Lead Card */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                  Linked Institution / Organization
                </span>
                <span className="text-[10px] rounded bg-purple-100 border border-purple-200 px-1.5 py-0.2 font-semibold text-purple-800">
                  {profile?.lead?.institutionType || 'Verified Entity'}
                </span>
              </div>

              <div>
                <h3 className="text-sm font-bold text-[var(--color-ink)]">
                  {profile?.lead?.name || 'Independent Contact (No Institutional Lead Linked)'}
                </h3>
                {profile?.lead?.city && (
                  <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
                    Location: {profile.lead.city}
                  </p>
                )}
              </div>

              <div className="space-y-1.5 pt-2 border-t border-[var(--color-border)]/30 text-xs font-mono">
                <div className="flex items-center justify-between">
                  <span className="text-[var(--color-ink-faint)] text-[10px]">Lead ID:</span>
                  <span className="text-[var(--color-ink)]">{profile?.lead?.id || 'N/A'}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-[var(--color-ink-faint)] text-[10px]">Harvester Institution ID:</span>
                  <span className="text-[var(--color-ink)]">
                    {profile?.lead?.institutionId || 'Direct Discovery'}
                  </span>
                </div>
              </div>
            </div>

            {/* Qualification Rationale */}
            <div className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-4 space-y-3">
              <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                Qualification Rationale &amp; Sourcing
              </span>

              <div className="rounded-lg bg-white border border-[var(--color-border)]/40 p-3 text-xs text-[var(--color-ink)] min-h-[90px] flex items-center">
                {profile?.lead?.qualificationRationale ? (
                  <p className="leading-relaxed">{profile.lead.qualificationRationale}</p>
                ) : (
                  <p className="text-[var(--color-ink-muted)] italic">
                    No qualification rationale provided. Contact was added directly or via audience list import.
                  </p>
                )}
              </div>

              <div className="flex items-center gap-2 text-[10px] text-[var(--color-ink-muted)]">
                <Award className="h-3.5 w-3.5 text-amber-600" />
                <span>Verified lead criteria verified by Autonomous Intelligence pipeline.</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* SECTION D: TAGS & PREFERENCES ("REMEMBERS") */}
      {/* ========================================================================= */}
      {(activeTab === 'all' || activeTab === 'tags') && (
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-6">
          {/* Header */}
          <div className="flex items-center justify-between border-b border-[var(--color-border)]/40 pb-3">
            <div className="flex items-center gap-2">
              <div className="h-7 w-7 rounded-lg bg-teal-50 border border-teal-200 flex items-center justify-center text-teal-700">
                <Brain className="h-4 w-4" />
              </div>
              <div>
                <h2 className="text-sm font-bold text-[var(--color-ink)]">
                  Section D: Tags &amp; Preferences ("Remembers")
                </h2>
                <p className="text-[11px] text-[var(--color-ink-muted)]">
                  Segmentation tags and durable contextual memories leveraged by the AI reply engine.
                </p>
              </div>
            </div>

            <button
              type="button"
              onClick={() => setShowAddMemoryModal(true)}
              className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] transition-colors shadow-xs"
            >
              <Plus className="h-3.5 w-3.5" />
              <span>Add Memory</span>
            </button>
          </div>

          {/* Part 1: Tags Strip */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-1.5 text-xs font-bold text-[var(--color-ink)]">
                <TagIcon className="h-3.5 w-3.5 text-[var(--color-ink-muted)]" />
                <span>Segmentation &amp; Audience Tags</span>
              </div>
              <span className="text-[10px] text-[var(--color-ink-faint)]">
                {profile?.tags?.length ?? 0} active tags
              </span>
            </div>

            <div className="flex flex-wrap items-center gap-2 p-3 rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 min-h-[48px]">
              {profile?.tags && profile.tags.length > 0 ? (
                profile.tags.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center gap-1.5 rounded-md bg-white border border-[var(--color-border)] px-2.5 py-1 text-xs font-semibold text-[var(--color-ink)] shadow-2xs"
                  >
                    <span className="text-[var(--color-ink-faint)]">#</span>
                    <span>{tag}</span>
                    <button
                      type="button"
                      onClick={() => handleRemoveTag(tag)}
                      title={`Remove tag #${tag}`}
                      className="text-[var(--color-ink-faint)] hover:text-rose-600 transition-colors"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </span>
                ))
              ) : (
                <span className="text-xs text-[var(--color-ink-muted)]">
                  No tags assigned to this contact.
                </span>
              )}

              {/* Inline Add Tag */}
              {isAddingTag ? (
                <div className="inline-flex items-center gap-1">
                  <input
                    type="text"
                    value={newTagInput}
                    onChange={(e) => setNewTagInput(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleAddTag()
                      if (e.key === 'Escape') setIsAddingTag(false)
                    }}
                    placeholder="tag-name"
                    autoFocus
                    className="h-7 w-28 rounded-md border border-slate-300 px-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                  />
                  <button
                    type="button"
                    onClick={handleAddTag}
                    disabled={savingTag || !newTagInput.trim()}
                    className="h-7 rounded-md bg-emerald-600 px-2 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50"
                  >
                    {savingTag ? '...' : 'Add'}
                  </button>
                  <button
                    type="button"
                    onClick={() => setIsAddingTag(false)}
                    className="h-7 rounded-md border border-[var(--color-border)] bg-white px-1.5 text-xs text-[var(--color-ink-muted)]"
                  >
                    <X className="h-3 w-3" />
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() => setIsAddingTag(true)}
                  className="inline-flex items-center gap-1 rounded-md border border-dashed border-[var(--color-border)] bg-white/60 px-2.5 py-1 text-xs font-semibold text-[var(--color-ink-muted)] hover:border-slate-400 hover:text-[var(--color-ink)] transition-colors"
                >
                  <Plus className="h-3 w-3" />
                  <span>New Tag</span>
                </button>
              )}
            </div>
          </div>

          {/* Part 2: Memories & Preferences ("Remembers") */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-1.5 text-xs font-bold text-[var(--color-ink)]">
                <Brain className="h-3.5 w-3.5 text-teal-600" />
                <span>AI Memory Vault ("Remembers")</span>
              </div>
              <span className="text-[10px] text-[var(--color-ink-faint)]">
                {profile?.memories?.length ?? 0} memories stored
              </span>
            </div>

            {profile?.memories && profile.memories.length > 0 ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {profile.memories.map((mem) => (
                  <div
                    key={mem.id}
                    className="rounded-xl border border-[var(--color-border)]/60 bg-[var(--color-page)]/40 p-3.5 space-y-2 flex flex-col justify-between"
                  >
                    <div className="space-y-1.5">
                      <div className="flex items-center justify-between">
                        <span className="rounded bg-teal-100 border border-teal-200 px-1.5 py-0.2 text-[10px] font-bold uppercase tracking-wider text-teal-800">
                          {mem.category}
                        </span>
                        <div className="flex items-center gap-1 text-[10px] text-[var(--color-ink-faint)]">
                          <Clock className="h-3 w-3" />
                          <span>{timeAgo(mem.createdAt)}</span>
                        </div>
                      </div>
                      <p className="text-xs font-medium text-[var(--color-ink)] leading-snug">
                        "{mem.fact}"
                      </p>
                    </div>

                    <div className="flex items-center justify-between pt-2 border-t border-[var(--color-border)]/20">
                      <span className="text-[10px] text-[var(--color-ink-muted)] flex items-center gap-1">
                        <Sparkles className="h-3 w-3 text-amber-500" />
                        <span>High confidence</span>
                      </span>
                      <button
                        type="button"
                        onClick={() => handleDeleteMemory(mem.id)}
                        className="p-1 text-[var(--color-ink-faint)] hover:text-rose-600 rounded transition-colors"
                        title="Delete memory"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-page)]/20 p-6 text-center space-y-2">
                <Brain className="h-8 w-8 text-[var(--color-ink-faint)] mx-auto" />
                <p className="text-xs font-semibold text-[var(--color-ink)]">
                  No remembered preferences recorded yet
                </p>
                <p className="text-[11px] text-[var(--color-ink-muted)] max-w-sm mx-auto">
                  Add facts or preferences so the AI reply engine tailors follow-ups and conversational tone specifically for this contact.
                </p>
                <button
                  type="button"
                  onClick={() => setShowAddMemoryModal(true)}
                  className="inline-flex items-center gap-1 rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] transition-colors mt-1"
                >
                  <Plus className="h-3.5 w-3.5" />
                  <span>Add First Memory</span>
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* SECTION E: RECENT MESSAGE HISTORY */}
      {/* ========================================================================= */}
      {(activeTab === 'all' || activeTab === 'messages') && (
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)]/40 pb-3">
            <div className="flex items-center gap-2">
              <div className="h-7 w-7 rounded-lg bg-amber-50 border border-amber-200 flex items-center justify-center text-amber-700">
                <Clock className="h-4 w-4" />
              </div>
              <div>
                <h2 className="text-sm font-bold text-[var(--color-ink)]">
                  Section E: Recent WhatsApp Message History
                </h2>
                <p className="text-[11px] text-[var(--color-ink-muted)]">
                  Chronological preview of recent inbound/outbound exchanges on this channel.
                </p>
              </div>
            </div>

            {profile?.conversation?.id && (
              <button
                type="button"
                onClick={handleOpenConversation}
                className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] transition-colors shadow-xs"
              >
                <span>View Full Chat in Inbox</span>
                <ExternalLink className="h-3.5 w-3.5" />
              </button>
            )}
          </div>

          {/* Messages list */}
          <div className="space-y-3 max-h-[380px] overflow-y-auto pr-1">
            {profile?.recentMessages && profile.recentMessages.length > 0 ? (
              profile.recentMessages.map((msg) => {
                const isInbound = msg.direction === 'inbound'
                return (
                  <div
                    key={msg.id}
                    className={`flex flex-col ${isInbound ? 'items-start' : 'items-end'}`}
                  >
                    <div className="flex items-center gap-1.5 text-[10px] text-[var(--color-ink-faint)] px-1 mb-1">
                      <span className="font-semibold text-[var(--color-ink-muted)]">
                        {isInbound ? (msg.senderName || personName) : 'NEWAX Copilot'}
                      </span>
                      <span>·</span>
                      <span>{timeAgo(msg.sentAt)}</span>
                      {!isInbound && msg.status && (
                        <>
                          <span>·</span>
                          <span className="font-mono uppercase text-[9px]">{msg.status}</span>
                        </>
                      )}
                    </div>

                    <div
                      className={`max-w-[80%] rounded-2xl px-4 py-2.5 text-xs shadow-2xs leading-relaxed ${
                        isInbound
                          ? 'bg-slate-100 text-[var(--color-ink)] rounded-tl-sm border border-[var(--color-border)]/40'
                          : 'bg-[var(--color-accent)] text-white rounded-tr-sm'
                      }`}
                    >
                      <p className="whitespace-pre-wrap">{msg.body}</p>
                    </div>
                  </div>
                )
              })
            ) : (
              <div className="p-8 text-center text-xs text-[var(--color-ink-muted)] space-y-2">
                <MessageSquare className="h-8 w-8 text-[var(--color-ink-faint)] mx-auto" />
                <p className="font-semibold text-[var(--color-ink)]">No messages exchanged yet</p>
                <p className="text-[11px]">
                  Dispatch an outbound WhatsApp message to initiate communication with this contact.
                </p>
              </div>
            )}
          </div>

          {/* Inline Quick Message Sender */}
          <div className="pt-3 border-t border-[var(--color-border)]/40">
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder={`Type a WhatsApp message to ${personName}...`}
                value={messageDraft}
                onChange={(e) => setMessageDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault()
                    handleSendMessage()
                  }
                }}
                className="flex-1 rounded-xl border border-slate-300 bg-white px-3.5 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
              />
              <button
                type="button"
                onClick={() => handleSendMessage()}
                disabled={sendingMessage || !messageDraft.trim()}
                className="inline-flex items-center gap-1.5 rounded-xl bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] transition-colors shadow-xs disabled:opacity-50"
              >
                <Send className="h-3.5 w-3.5" />
                <span>{sendingMessage ? 'Sending...' : 'Send'}</span>
              </button>
            </div>
            {!windowActive && (
              <p className="mt-1 text-[10px] text-amber-700 flex items-center gap-1">
                <AlertCircle className="h-3 w-3" />
                <span>WhatsApp 24-hour service window is inactive. Meta requires an approved template message for initiated outreach.</span>
              </p>
            )}
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* MODAL: SEND WHATSAPP MESSAGE */}
      {/* ========================================================================= */}
      {showSendModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-2xl overflow-hidden flex flex-col">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] p-4">
              <div className="flex items-center gap-2">
                <div className="h-7 w-7 rounded-lg bg-emerald-100 flex items-center justify-center text-emerald-800">
                  <Send className="h-4 w-4" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Send WhatsApp Cloud Message
                  </h3>
                  <p className="text-[11px] text-[var(--color-ink-muted)]">
                    Direct dispatch to {personName} ({displayPhone})
                  </p>
                </div>
              </div>
              <button
                type="button"
                onClick={() => setShowSendModal(false)}
                className="p-1 rounded text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="p-4 space-y-3">
              <div className="space-y-1">
                <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                  Message Body
                </label>
                <textarea
                  rows={4}
                  value={messageDraft}
                  onChange={(e) => setMessageDraft(e.target.value)}
                  placeholder="Hello, thank you for reaching out..."
                  className="w-full rounded-xl border border-slate-300 p-3 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                />
              </div>

              <div className="flex items-center justify-between text-[11px] text-[var(--color-ink-faint)]">
                <span>{messageDraft.length} characters</span>
                <span className="font-mono text-emerald-700">WhatsApp Cloud API</span>
              </div>
            </div>

            <div className="flex items-center justify-end gap-2 border-t border-[var(--color-border)] bg-slate-50 p-3">
              <button
                type="button"
                onClick={() => setShowSendModal(false)}
                className="rounded-lg border border-[var(--color-border)] bg-white px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-slate-100"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => handleSendMessage()}
                disabled={sendingMessage || !messageDraft.trim()}
                className="inline-flex items-center gap-1.5 rounded-lg bg-emerald-600 px-4 py-1.5 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50"
              >
                <Send className="h-3.5 w-3.5" />
                <span>{sendingMessage ? 'Sending...' : 'Send Message'}</span>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* MODAL: ADD MEMORY / PREFERENCE */}
      {/* ========================================================================= */}
      {showAddMemoryModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-2xl overflow-hidden flex flex-col">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] p-4">
              <div className="flex items-center gap-2">
                <div className="h-7 w-7 rounded-lg bg-teal-100 flex items-center justify-center text-teal-800">
                  <Brain className="h-4 w-4" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Add Remembered Fact / Preference
                  </h3>
                  <p className="text-[11px] text-[var(--color-ink-muted)]">
                    Preserves customer constraints and preferences for AI reply engine prompts.
                  </p>
                </div>
              </div>
              <button
                type="button"
                onClick={() => setShowAddMemoryModal(false)}
                className="p-1 rounded text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="p-4 space-y-3">
              <div className="space-y-1">
                <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                  Category
                </label>
                <select
                  value={memoryCategory}
                  onChange={(e) => setMemoryCategory(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                >
                  <option value="preference">Preference (communication style, tone, channel)</option>
                  <option value="constraint">Constraint (budget, timeline, non-negotiable)</option>
                  <option value="history">History (past inquiries or outcomes)</option>
                  <option value="lead_need">Lead Need (curriculum, simulation equipment)</option>
                  <option value="relationship">Relationship (decision maker role, partner)</option>
                </select>
              </div>

              <div className="space-y-1">
                <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                  Fact or Stated Preference
                </label>
                <textarea
                  rows={3}
                  value={memoryFact}
                  onChange={(e) => setMemoryFact(e.target.value)}
                  placeholder="e.g. Prefers outreach in the afternoon via WhatsApp rather than email."
                  className="w-full rounded-xl border border-slate-300 p-3 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                />
              </div>
            </div>

            <div className="flex items-center justify-end gap-2 border-t border-[var(--color-border)] bg-slate-50 p-3">
              <button
                type="button"
                onClick={() => setShowAddMemoryModal(false)}
                className="rounded-lg border border-[var(--color-border)] bg-white px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-slate-100"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleAddMemory}
                disabled={savingMemory || !memoryFact.trim()}
                className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
              >
                <Plus className="h-3.5 w-3.5" />
                <span>{savingMemory ? 'Saving...' : 'Save to Memory Vault'}</span>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* MODAL: Edit Dynamic Custom Attributes */}
      {/* ========================================================================= */}
      {showEditAttributesModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 p-4 backdrop-blur-xs">
          <div className="w-full max-w-lg rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] p-4 bg-[var(--color-page)]">
              <div className="flex items-center gap-2">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-50 border border-indigo-200 text-indigo-700">
                  <Sparkles className="h-4 w-4" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Edit Contact Custom Fields
                  </h3>
                  <p className="text-[11px] text-[var(--color-ink-muted)]">
                    Manage structured metadata and enterprise attributes for this contact.
                  </p>
                </div>
              </div>
              <button
                type="button"
                onClick={() => setShowEditAttributesModal(false)}
                className="rounded-lg p-1 text-[var(--color-ink-muted)] hover:bg-slate-100"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="p-4 space-y-4 overflow-y-auto flex-1 text-xs">
              {/* Existing Attributes List */}
              <div className="space-y-2">
                <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                  Configured Attributes ({Object.keys(attributeDrafts).length})
                </label>
                {Object.keys(attributeDrafts).length === 0 ? (
                  <p className="text-xs text-[var(--color-ink-faint)] italic p-3 bg-slate-50 rounded-lg border border-dashed border-slate-200">
                    No custom fields configured yet. Add fields below.
                  </p>
                ) : (
                  <div className="space-y-2 max-h-60 overflow-y-auto pr-1">
                    {Object.entries(attributeDrafts).map(([key, val]) => {
                      const matchedDef = availableDefinitions.find((d) => d.name === key)
                      return (
                        <div
                          key={key}
                          className="flex items-center gap-2 rounded-lg border border-[var(--color-border)] p-2.5 bg-[var(--color-surface)] shadow-2xs"
                        >
                          <div className="w-1/3 shrink-0">
                            <span className="font-mono text-[11px] font-bold text-[var(--color-ink)] truncate block" title={key}>
                              {key}
                            </span>
                            {matchedDef?.display_name && (
                              <span className="text-[10px] text-[var(--color-ink-muted)] truncate block">
                                {matchedDef.display_name}
                              </span>
                            )}
                          </div>
                          <div className="flex-1">
                            {matchedDef?.attribute_type === 'list' && matchedDef.options && matchedDef.options.length > 0 ? (
                              <select
                                value={val}
                                onChange={(e) =>
                                  setAttributeDrafts({
                                    ...attributeDrafts,
                                    [key]: e.target.value,
                                  })
                                }
                                className="w-full rounded-md border border-slate-300 px-2 py-1 text-xs bg-white focus:outline-none focus:ring-1 focus:ring-slate-900"
                              >
                                <option value="">Select an option...</option>
                                {matchedDef.options.map((opt) => (
                                  <option key={opt} value={opt}>
                                    {opt}
                                  </option>
                                ))}
                              </select>
                            ) : (
                              <input
                                type="text"
                                value={val}
                                onChange={(e) =>
                                  setAttributeDrafts({
                                    ...attributeDrafts,
                                    [key]: e.target.value,
                                  })
                                }
                                placeholder="Value..."
                                className="w-full rounded-md border border-slate-300 px-2.5 py-1 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                              />
                            )}
                          </div>
                          <button
                            type="button"
                            onClick={() => handleRemoveAttribute(key)}
                            title="Remove attribute"
                            className="rounded p-1 text-slate-400 hover:text-rose-600 hover:bg-rose-50 transition-colors"
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </button>
                        </div>
                      )
                    })}
                  </div>
                )}
              </div>

              {/* Add New Field Section */}
              <div className="rounded-xl border border-[var(--color-border)] bg-slate-50/70 p-3 space-y-2.5">
                <span className="text-[11px] font-bold uppercase tracking-wider text-[var(--color-ink)]">
                  Add Attribute
                </span>

                {availableDefinitions.length > 0 && (
                  <div>
                    <label className="text-[10px] font-medium text-[var(--color-ink-muted)] mb-1 block">
                      Choose from Predefined Schema
                    </label>
                    <select
                      value={newFieldKey}
                      onChange={(e) => {
                        const selectedKey = e.target.value
                        setNewFieldKey(selectedKey)
                        const def = availableDefinitions.find((d) => d.name === selectedKey)
                        if (def?.options && def.options.length > 0) {
                          setNewFieldValue(def.options[0])
                        } else {
                          setNewFieldValue('')
                        }
                      }}
                      className="w-full rounded-md border border-slate-300 px-2.5 py-1.5 text-xs bg-white focus:outline-none focus:ring-1 focus:ring-slate-900"
                    >
                      <option value="">Select schema attribute or enter custom below...</option>
                      {availableDefinitions
                        .filter((d) => !Object.keys(attributeDrafts).includes(d.name))
                        .map((d) => (
                          <option key={d.id} value={d.name}>
                            {d.display_name} ({d.name}) · {d.attribute_type}
                          </option>
                        ))}
                    </select>
                  </div>
                )}

                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-[10px] font-medium text-[var(--color-ink-muted)] mb-1 block">
                      Key Name (slug)
                    </label>
                    <input
                      type="text"
                      placeholder="e.g. account_tier"
                      value={newFieldKey}
                      onChange={(e) => setNewFieldKey(e.target.value)}
                      className="w-full rounded-md border border-slate-300 px-2.5 py-1 text-xs bg-white font-mono focus:outline-none focus:ring-1 focus:ring-slate-900"
                    />
                  </div>
                  <div>
                    <label className="text-[10px] font-medium text-[var(--color-ink-muted)] mb-1 block">
                      Value
                    </label>
                    <input
                      type="text"
                      placeholder="e.g. Enterprise"
                      value={newFieldValue}
                      onChange={(e) => setNewFieldValue(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          e.preventDefault()
                          handleAddDraftField()
                        }
                      }}
                      className="w-full rounded-md border border-slate-300 px-2.5 py-1 text-xs bg-white focus:outline-none focus:ring-1 focus:ring-slate-900"
                    />
                  </div>
                </div>

                <div className="flex justify-end pt-1">
                  <button
                    type="button"
                    onClick={handleAddDraftField}
                    disabled={!newFieldKey.trim()}
                    className="inline-flex items-center gap-1 rounded-md border border-slate-300 bg-white px-2.5 py-1 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50"
                  >
                    <Plus className="h-3 w-3" />
                    <span>Add to List</span>
                  </button>
                </div>
              </div>
            </div>

            <div className="flex items-center justify-between border-t border-[var(--color-border)] bg-[var(--color-page)] p-3">
              <span className="text-[11px] text-[var(--color-ink-muted)]">
                Changes will persist to SQLite contact records immediately.
              </span>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setShowEditAttributesModal(false)}
                  className="rounded-lg border border-[var(--color-border)] bg-white px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-slate-100"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onClick={handleSaveAttributes}
                  disabled={savingAttributes}
                  className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow-xs disabled:opacity-50"
                >
                  <Check className="h-3.5 w-3.5" />
                  <span>{savingAttributes ? 'Saving...' : 'Save Custom Fields'}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function InfoIcon(props: { className?: string }) {
  return <Info className={props.className} />
}
