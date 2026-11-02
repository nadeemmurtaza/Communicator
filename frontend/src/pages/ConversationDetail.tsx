import {
  AlertCircle,
  ArrowLeft,
  Check,
  CheckCircle2,
  Clock,
  ExternalLink,
  Globe,
  Lightbulb,
  Sparkles,
  Tag,
  Ticket as TicketIcon,
  Users,
  User,
  Zap,
} from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import {
  api,
  type CannedResponse,
  type ConversationDetail as ConversationDetailType,
  type CustomAttributeDefinitionItem,
  type Team,
  type TeamMember,
  type Ticket,
} from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

export function ConversationDetail() {
  const { id } = useParams<{ id: string }>()
  const [conversation, setConversation] = useState<ConversationDetailType | null>(null)
  const [team, setTeam] = useState<TeamMember[]>([])
  const [teams, setTeams] = useState<Team[]>([])
  const [error, setError] = useState<string | null>(null)
  const [generating, setGenerating] = useState(false)
  const [togglingHandoff, setTogglingHandoff] = useState(false)
  const [replyText, setReplyText] = useState('')
  const [isSending, setIsSending] = useState(false)
  const [isResolving, setIsResolving] = useState(false)
  const [isSnoozing, setIsSnoozing] = useState(false)
  const [showSnoozeMenu, setShowSnoozeMenu] = useState(false)
  const [isAssigningTeam, setIsAssigningTeam] = useState(false)
  const [isAssigningAgent, setIsAssigningAgent] = useState(false)
  const [isUpdatingPriority, setIsUpdatingPriority] = useState(false)

  const [cannedResponses, setCannedResponses] = useState<CannedResponse[]>([])
  const [showCannedPicker, setShowCannedPicker] = useState(false)
  const [cannedCategory, setCannedCategory] = useState('all')
  const [cannedSelectedIndex, setCannedSelectedIndex] = useState(0)
  const [conversationTickets, setConversationTickets] = useState<Ticket[]>([])

  // Translation State
  const [translatingMsgId, setTranslatingMsgId] = useState<string | null>(null)
  const [showOriginalFor, setShowOriginalFor] = useState<Set<string>>(new Set())
  const [autoTranslateOutbound, setAutoTranslateOutbound] = useState(true)

  // CRM Custom Attributes State
  const [attrDefinitions, setAttrDefinitions] = useState<CustomAttributeDefinitionItem[]>([])
  const [convAttributes, setConvAttributes] = useState<Record<string, any>>({})
  const [showAttributesBar, setShowAttributesBar] = useState(false)
  const [savingAttributes, setSavingAttributes] = useState(false)
  const [attrSavedToast, setAttrSavedToast] = useState(false)

  function load() {
    if (!id) return
    api
      .getConversation(id)
      .then(setConversation)
      .catch((e) => setError(e.message))
    api
      .listTickets({ conversation_id: id })
      .then(setConversationTickets)
      .catch(() => {})
  }

  useEffect(load, [id])
  useEffect(() => {
    api.listTeam().then(setTeam).catch(() => {})
    api.listTeams().then(setTeams).catch(() => {})
    api.listCannedResponses().then((res) => {
      if (res && res.length > 0) setCannedResponses(res)
    }).catch(() => {})
    api.getCustomAttributes('conversation_attribute').then(setAttrDefinitions).catch(() => {})
  }, [])

  useEffect(() => {
    if (conversation?.customAttributes) {
      setConvAttributes(conversation.customAttributes)
    }
  }, [conversation?.customAttributes])

  // Slash command autocomplete detection
  const slashMatch = replyText.match(/(?:^|\s)\/([a-zA-Z0-9_-]*)$/)
  const isTypingSlash = Boolean(slashMatch)
  const slashQuery = slashMatch ? slashMatch[1].toLowerCase() : ''
  const cannedCategories = ['all', ...Array.from(new Set(cannedResponses.map((c) => (c.category || 'general').toLowerCase())))]

  const filteredCanned = cannedResponses.filter((c) => {
    const code = (c.shortCode || c.short_code || '').toLowerCase()
    const content = c.content.toLowerCase()
    const matchesQuery = !slashQuery || code.includes(slashQuery) || content.includes(slashQuery)
    const matchesCategory =
      cannedCategory === 'all' || (c.category || 'general').toLowerCase() === cannedCategory.toLowerCase()
    return matchesQuery && matchesCategory
  })

  useEffect(() => {
    if (cannedSelectedIndex >= filteredCanned.length) {
      setCannedSelectedIndex(0)
    }
  }, [filteredCanned.length, cannedSelectedIndex])

  function resolveCannedVariables(template: string): string {
    const custName = conversation?.contactName || conversation?.customerName || conversation?.leadName || 'Customer'
    const custFirstName = custName.split(' ')[0] || 'there'
    const agent = team.find((m) => m.id === conversation?.assignedToId)
    const agentName = agent?.name || conversation?.assignedToName || 'Support Agent'
    const channelName = conversation?.channel ? conversation.channel.charAt(0).toUpperCase() + conversation.channel.slice(1) : 'Support'
    const ticketId = conversation?.id ? conversation.id.slice(0, 8) : 'TICKET'

    return template
      .replace(/\{\{\s*customer\.name\s*\}\}/gi, custName)
      .replace(/\{\{\s*customer_name\s*\}\}/gi, custName)
      .replace(/\{\{\s*customer\.first_name\s*\}\}/gi, custFirstName)
      .replace(/\{\{\s*customer\.firstName\s*\}\}/gi, custFirstName)
      .replace(/\{\{\s*first_name\s*\}\}/gi, custFirstName)
      .replace(/\{\{\s*agent\.name\s*\}\}/gi, agentName)
      .replace(/\{\{\s*agent_name\s*\}\}/gi, agentName)
      .replace(/\{\{\s*channel\s*\}\}/gi, channelName)
      .replace(/\{\{\s*ticket\.id\s*\}\}/gi, ticketId)
      .replace(/\{\{\s*ticket_id\s*\}\}/gi, ticketId)
      .replace(/\{\{\s*conversation_id\s*\}\}/gi, conversation?.id || ticketId)
  }

  function insertCanned(canned: CannedResponse) {
    const resolved = resolveCannedVariables(canned.content)
    if (slashMatch) {
      const idx = replyText.lastIndexOf('/')
      const prefix = replyText.slice(0, idx)
      setReplyText(prefix + resolved + ' ')
    } else {
      setReplyText((prev) => (prev ? prev + ' ' : '') + resolved + ' ')
    }
    setShowCannedPicker(false)
    setCannedSelectedIndex(0)
  }

  async function handleSaveAttributes() {
    if (!id) return
    setSavingAttributes(true)
    setError(null)
    try {
      await api.updateCustomAttributes(id, convAttributes)
      setAttrSavedToast(true)
      setTimeout(() => setAttrSavedToast(false), 2500)
    } catch (e: any) {
      setError(e.message || 'Failed to save attributes')
    } finally {
      setSavingAttributes(false)
    }
  }

  function toggleOriginal(msgId: string) {
    setShowOriginalFor((prev) => {
      const next = new Set(prev)
      if (next.has(msgId)) {
        next.delete(msgId)
      } else {
        next.add(msgId)
      }
      return next
    })
  }

  async function handleTranslateMessage(msgId: string) {
    setTranslatingMsgId(msgId)
    setError(null)
    try {
      const res = await api.translateMessage(msgId, 'en')
      if (res && res.translation) {
        setConversation((prev) => {
          if (!prev) return null
          return {
            ...prev,
            messages: prev.messages.map((m) =>
              m.id === msgId ? { ...m, translation: res.translation } : m
            ),
          }
        })
      }
    } catch (e: any) {
      setError(e.message || 'Failed to translate message')
    } finally {
      setTranslatingMsgId(null)
    }
  }

  async function handleSendReply() {
    if (!id || !replyText.trim() || isSending) return
    setIsSending(true)
    setError(null)
    try {
      let textToSend = replyText.trim()
      if (autoTranslateOutbound && conversation?.language && conversation.language !== 'en') {
        try {
          const trans = await api.prepareOutboundTranslation(id, textToSend, conversation.language)
          if (trans && trans.translated_text) {
            textToSend = trans.translated_text
          }
        } catch {
          // fallback to original text if translation fails
        }
      }
      await api.sendConversationMessage(id, textToSend)
      setReplyText('')
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to send reply')
    } finally {
      setIsSending(false)
    }
  }

  async function handleResolve() {
    if (!id || isResolving) return
    setIsResolving(true)
    setError(null)
    try {
      await api.resolveConversation(id)
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to resolve conversation')
    } finally {
      setIsResolving(false)
    }
  }

  async function handleSnooze(hours: number | null) {
    if (!id || isSnoozing) return
    setIsSnoozing(true)
    setError(null)
    setShowSnoozeMenu(false)
    try {
      const until = hours ? new Date(Date.now() + hours * 3600 * 1000).toISOString() : null
      await api.snoozeConversation(id, until)
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to update snooze')
    } finally {
      setIsSnoozing(false)
    }
  }

  async function handleAssignTeam(teamId: string) {
    if (!id || isAssigningTeam) return
    setIsAssigningTeam(true)
    setError(null)
    try {
      await api.assignTeam(id, teamId || null)
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to assign team')
    } finally {
      setIsAssigningTeam(false)
    }
  }

  async function handleAssignAgent(agentId: string) {
    if (!id || isAssigningAgent) return
    setIsAssigningAgent(true)
    setError(null)
    try {
      await api.assignAgent(id, agentId || null)
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to assign agent')
    } finally {
      setIsAssigningAgent(false)
    }
  }

  async function handlePriorityChange(priority: string) {
    if (!id || isUpdatingPriority) return
    setIsUpdatingPriority(true)
    setError(null)
    try {
      await api.updatePriority(id, priority)
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to update priority')
    } finally {
      setIsUpdatingPriority(false)
    }
  }

  async function generateDraft() {
    if (!id) return
    setGenerating(true)
    setError(null)
    try {
      await api.generateDraft(id)
    } catch (e: any) {
      setError(e.message || 'Failed to generate AI draft')
    } finally {
      setGenerating(false)
    }
  }

  async function toggleHandoff() {
    if (!id || !conversation) return
    setTogglingHandoff(true)
    try {
      await api.setHumanTakeover(id, !conversation.humanTakeover)
      load()
    } catch (e: any) {
      setError(e.message || 'Failed to update human takeover state')
    } finally {
      setTogglingHandoff(false)
    }
  }

  const isResolved = (conversation?.state || '').toLowerCase() === 'resolved'
  const isSnoozed = Boolean(conversation?.snoozedUntil && new Date(conversation.snoozedUntil).getTime() > Date.now())

  return (
    <div className="space-y-6">
      <div>
        <Link
          to="/conversations"
          className="text-sm text-[var(--color-ink-muted)] hover:underline inline-flex items-center gap-1.5"
        >
          <ArrowLeft className="h-4 w-4" />
          <span>Conversations</span>
        </Link>
        <div className="mt-2 flex flex-wrap items-center gap-3">
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">
            {(conversation?.customerName || conversation?.leadName || conversation?.contactName) ? (
              <>
                {conversation.customerName || conversation.leadName || conversation.contactName}
                {(conversation.contactPhone || conversation.phone) && (
                  <span className="text-sm font-normal text-[var(--color-ink-muted)] ml-2 font-mono">
                    ({conversation.contactPhone || conversation.phone})
                  </span>
                )}
              </>
            ) : (
              `Conversation ${id?.slice(0, 8)}`
            )}
          </h1>
          {conversation && (
            <span className={`badge ${stateBadgeClass(conversation.state)}`}>
              {stateLabel(conversation.state)}
            </span>
          )}
          {conversation?.humanTakeover && (
            <span className="badge badge-warning">Human handling</span>
          )}
          {isSnoozed && (
            <span className="inline-flex items-center gap-1 rounded-full bg-amber-50 px-2.5 py-0.5 text-xs font-medium text-amber-800 border border-amber-200">
              <Clock className="h-3 w-3 text-amber-600" />
              Snoozed until {new Date(conversation!.snoozedUntil!).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </span>
          )}
          {conversation?.slaStatus && (
            <span
              className={`inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold border ${
                conversation.slaStatus === 'breached'
                  ? 'bg-rose-50 text-rose-800 border-rose-200'
                  : conversation.slaStatus === 'warning'
                  ? 'bg-amber-50 text-amber-800 border-amber-200'
                  : 'bg-emerald-50 text-emerald-800 border-emerald-200'
              }`}
            >
              <Clock className="h-3 w-3" />
              SLA: {conversation.slaStatus.toUpperCase()}
            </span>
          )}
          {conversation?.language && conversation.language !== 'en' && (
            <span className="inline-flex items-center gap-1 rounded-full bg-[var(--color-accent-soft)] px-2.5 py-0.5 text-xs font-semibold text-[var(--color-accent)] border border-[var(--color-border)]">
              <Globe className="h-3.5 w-3.5" /> Language: {conversation.language.toUpperCase()} • Auto-Translation Active
            </span>
          )}
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {conversation === null ? (
        <p className="text-sm text-[var(--color-ink-muted)]">Loading conversation...</p>
      ) : (
        <>
          {/* Action Toolbar */}
          <div className="flex flex-wrap items-center gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 shadow-xs">
            {/* Human Handoff */}
            <button
              onClick={toggleHandoff}
              disabled={togglingHandoff}
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50 cursor-pointer transition-colors"
            >
              {conversation.humanTakeover ? 'Resume Bot' : 'Take Over from Bot'}
            </button>

            {/* Priority Selector */}
            <div className="flex items-center gap-1.5">
              <AlertCircle className="h-3.5 w-3.5 text-slate-500" />
              <label className="text-xs font-medium text-[var(--color-ink-muted)]">Priority</label>
              <select
                value={conversation.priority ?? 'medium'}
                onChange={(e) => handlePriorityChange(e.target.value)}
                disabled={isUpdatingPriority}
                className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-2 py-1 text-xs capitalize cursor-pointer focus:border-blue-500 focus:outline-none"
              >
                <option value="urgent">Urgent</option>
                <option value="high">High</option>
                <option value="medium">Medium</option>
                <option value="low">Low</option>
              </select>
            </div>

            {/* Team Assignment */}
            <div className="flex items-center gap-1.5">
              <Users className="h-3.5 w-3.5 text-slate-500" />
              <label className="text-xs font-medium text-[var(--color-ink-muted)]">Team</label>
              <select
                value={conversation.teamId ?? ''}
                onChange={(e) => handleAssignTeam(e.target.value)}
                disabled={isAssigningTeam}
                className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-2 py-1 text-xs cursor-pointer focus:border-blue-500 focus:outline-none"
              >
                <option value="">No Team</option>
                {teams.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Agent Assignment */}
            <div className="flex items-center gap-1.5">
              <User className="h-3.5 w-3.5 text-slate-500" />
              <label className="text-xs font-medium text-[var(--color-ink-muted)]">Agent</label>
              <select
                value={conversation.assignedToId ?? ''}
                onChange={(e) => handleAssignAgent(e.target.value)}
                disabled={isAssigningAgent}
                className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-2 py-1 text-xs cursor-pointer focus:border-blue-500 focus:outline-none"
              >
                <option value="">Unassigned</option>
                {team.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Snooze Control */}
            <div className="relative">
              <button
                type="button"
                onClick={() => setShowSnoozeMenu((v) => !v)}
                disabled={isSnoozing}
                className={`rounded-md border px-3 py-1.5 text-xs font-semibold flex items-center gap-1.5 cursor-pointer transition-colors ${
                  isSnoozed
                    ? 'border-amber-300 bg-amber-50 text-amber-800 hover:bg-amber-100'
                    : 'border-slate-300 bg-[var(--color-surface)] text-slate-700 hover:bg-[var(--color-page)]'
                }`}
              >
                <Clock className="h-3.5 w-3.5" />
                <span>{isSnoozed ? 'Snoozed' : 'Snooze'}</span>
              </button>

              {showSnoozeMenu && (
                <div className="absolute left-0 mt-1.5 w-44 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] shadow-lg z-20 py-1 text-xs">
                  <button
                    type="button"
                    onClick={() => handleSnooze(1)}
                    className="w-full text-left px-3 py-1.5 hover:bg-[var(--color-page)] cursor-pointer text-slate-700"
                  >
                    Snooze 1 hour
                  </button>
                  <button
                    type="button"
                    onClick={() => handleSnooze(24)}
                    className="w-full text-left px-3 py-1.5 hover:bg-[var(--color-page)] cursor-pointer text-slate-700"
                  >
                    Snooze 24 hours (Tomorrow)
                  </button>
                  <button
                    type="button"
                    onClick={() => handleSnooze(72)}
                    className="w-full text-left px-3 py-1.5 hover:bg-[var(--color-page)] cursor-pointer text-slate-700"
                  >
                    Snooze 3 days
                  </button>
                  {isSnoozed && (
                    <button
                      type="button"
                      onClick={() => handleSnooze(null)}
                      className="w-full text-left px-3 py-1.5 hover:bg-rose-50 text-rose-600 font-semibold border-t border-slate-100 cursor-pointer"
                    >
                      Clear Snooze (Wake Up)
                    </button>
                  )}
                </div>
              )}
            </div>

            {/* Resolve Conversation Button */}
            <button
              type="button"
              onClick={handleResolve}
              disabled={isResolving || isResolved}
              className={`rounded-md px-3.5 py-1.5 text-xs font-semibold flex items-center gap-1.5 shadow-xs transition-colors cursor-pointer ${
                isResolved
                  ? 'border border-emerald-300 bg-emerald-50 text-emerald-800 opacity-90 cursor-default'
                  : 'bg-emerald-600 text-white hover:bg-emerald-700 disabled:opacity-50'
              }`}
            >
              <CheckCircle2 className="h-3.5 w-3.5" />
              <span>{isResolved ? 'Resolved' : isResolving ? 'Resolving...' : 'Resolve Thread'}</span>
            </button>

            {/* Right side: Tickets */}
            <div className="ml-auto flex items-center gap-2">
              {conversationTickets.length > 0 ? (
                <div className="flex items-center gap-2">
                  {conversationTickets.map((tk) => (
                    <Link
                      key={tk.id}
                      to="/tickets"
                      className="inline-flex items-center gap-1.5 rounded-md bg-[var(--color-accent-soft)] border border-[var(--color-border)] px-2.5 py-1 text-xs font-semibold text-[var(--color-accent)] hover:bg-indigo-100 transition-colors"
                      title={tk.title}
                    >
                      <TicketIcon className="h-3.5 w-3.5" />
                      <span>{tk.ticket_number}</span>
                      <span className="text-[10px] uppercase font-normal text-indigo-500">
                        ({tk.status})
                      </span>
                    </Link>
                  ))}
                </div>
              ) : (
                <Link
                  to="/tickets"
                  className="rounded-md border border-slate-300 px-2.5 py-1 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] transition-colors"
                >
                  + Open Ticket
                </Link>
              )}
            </div>
          </div>

          {/* DYNAMIC CRM CUSTOM ATTRIBUTES PANEL */}
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs overflow-hidden">
            <div className="flex items-center justify-between p-3.5 bg-[var(--color-page)]/80 border-b border-[var(--color-border)]">
              <div className="flex items-center gap-2">
                <Tag className="h-4 w-4 text-[var(--color-ink)]" />
                <span className="text-xs font-bold text-[var(--color-ink)]">CRM Custom Attributes</span>
                <span className="rounded-full bg-slate-200 px-2 py-0.2 text-[10px] font-semibold text-[var(--color-ink)]">
                  {Object.keys(convAttributes).length} set
                </span>
                {attrSavedToast && (
                  <span className="text-[11px] font-semibold text-emerald-600 animate-fade-in inline-flex items-center gap-1">
                    <Check className="h-3.5 w-3.5 text-emerald-600" />
                    <span>Attributes Saved!</span>
                  </span>
                )}
              </div>

              <div className="flex items-center gap-2">
                <Link
                  to="/custom-attributes"
                  className="text-[11px] font-semibold text-blue-600 hover:text-blue-800 inline-flex items-center gap-1"
                >
                  <span>Manage CRM Schema</span>
                  <ExternalLink className="h-3 w-3" />
                </Link>
                <button
                  type="button"
                  onClick={() => setShowAttributesBar((v) => !v)}
                  className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] cursor-pointer transition-colors"
                >
                  {showAttributesBar ? 'Collapse' : 'Expand Attributes'}
                </button>
              </div>
            </div>

            {showAttributesBar && (
              <div className="p-4 space-y-3 bg-[var(--color-surface)]">
                {attrDefinitions.length === 0 ? (
                  <p className="text-xs text-[var(--color-ink-muted)] py-2">
                    No custom conversation attributes defined yet.{' '}
                    <Link to="/custom-attributes" className="text-blue-600 underline">
                      Add attributes in CRM Schema Builder
                    </Link>
                    .
                  </p>
                ) : (
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                    {attrDefinitions.map((attr) => {
                      const val = convAttributes[attr.name] ?? attr.default_value ?? ''
                      return (
                        <div key={attr.id} className="space-y-1">
                          <div className="flex items-center justify-between">
                            <label className="text-[11px] font-semibold text-[var(--color-ink)] truncate" title={attr.description}>
                              {attr.display_name}
                            </label>
                            <span className="text-[9px] font-mono text-[var(--color-ink-faint)] uppercase">{attr.attribute_type}</span>
                          </div>

                          {attr.attribute_type === 'list' ? (
                            <select
                              value={val}
                              onChange={(e) =>
                                setConvAttributes((prev) => ({ ...prev, [attr.name]: e.target.value }))
                              }
                              className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                            >
                              <option value="">-- Select option --</option>
                              {attr.options?.map((opt) => (
                                <option key={opt} value={opt}>
                                  {opt}
                                </option>
                              ))}
                            </select>
                          ) : attr.attribute_type === 'boolean' ? (
                            <div className="flex items-center gap-2 pt-1">
                              <input
                                type="checkbox"
                                checked={Boolean(val === true || val === 'true')}
                                onChange={(e) =>
                                  setConvAttributes((prev) => ({ ...prev, [attr.name]: e.target.checked }))
                                }
                                className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500"
                              />
                              <span className="text-xs text-[var(--color-ink-muted)]">
                                {Boolean(val === true || val === 'true') ? 'Enabled / Yes' : 'Disabled / No'}
                              </span>
                            </div>
                          ) : attr.attribute_type === 'date' ? (
                            <input
                              type="date"
                              value={val}
                              onChange={(e) =>
                                setConvAttributes((prev) => ({ ...prev, [attr.name]: e.target.value }))
                              }
                              className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                            />
                          ) : (
                            <input
                              type={attr.attribute_type === 'number' || attr.attribute_type === 'currency' ? 'number' : 'text'}
                              value={val}
                              onChange={(e) =>
                                setConvAttributes((prev) => ({ ...prev, [attr.name]: e.target.value }))
                              }
                              placeholder={attr.description || `Enter ${attr.display_name}`}
                              className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                            />
                          )}
                        </div>
                      )
                    })}
                  </div>
                )}

                {attrDefinitions.length > 0 && (
                  <div className="pt-3 border-t border-[var(--color-border)] flex justify-end">
                    <button
                      type="button"
                      onClick={handleSaveAttributes}
                      disabled={savingAttributes}
                      className="rounded-md bg-blue-600 px-4 py-1.5 text-xs font-semibold text-white shadow-xs hover:bg-blue-700 disabled:opacity-50 cursor-pointer transition-colors"
                    >
                      {savingAttributes ? 'Saving...' : 'Save CRM Attributes'}
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Messages Stream */}
          <ol className="space-y-3">
            {conversation.messages.length === 0 ? (
              <div className="rounded-lg border border-dashed border-slate-300 bg-[var(--color-surface)] px-4 py-8 text-center text-xs text-[var(--color-ink-muted)]">
                No messages recorded in this conversation yet.
              </div>
            ) : (
              conversation.messages.map((m) => (
                <li
                  key={m.id}
                  className={`max-w-xl rounded-lg border px-4 py-3 text-sm shadow-2xs ${
                    m.direction === 'inbound'
                      ? 'border-[var(--color-border)] bg-[var(--color-surface)]'
                      : 'ml-auto border-slate-900 bg-[var(--color-accent)] text-white'
                  }`}
                >
                  {m.translation && (
                    <div className="mb-2 flex items-center justify-between gap-2 text-[11px] font-semibold text-blue-700 bg-blue-50/90 rounded-md px-2.5 py-1 border border-blue-200">
                      <span className="flex items-center gap-1">
                        <Globe className="h-3.5 w-3.5 text-blue-600" />
                        <span>
                          Translated from {m.translation.source_language.toUpperCase()} ({Math.round(m.translation.confidence * 100)}% match)
                        </span>
                      </span>
                      <button
                        type="button"
                        onClick={() => toggleOriginal(m.id)}
                        className="text-[10px] text-blue-800 underline hover:text-blue-900 cursor-pointer"
                      >
                        {showOriginalFor.has(m.id) ? 'Show English' : 'View Original'}
                      </button>
                    </div>
                  )}
                  <p className="whitespace-pre-wrap">
                    {m.translation && !showOriginalFor.has(m.id)
                      ? m.translation.translated_text
                      : m.body}
                  </p>
                  {!m.translation && m.direction === 'inbound' && (
                    <button
                      type="button"
                      onClick={() => handleTranslateMessage(m.id)}
                      disabled={translatingMsgId === m.id}
                      className="mt-1.5 inline-flex items-center gap-1 text-[11px] font-medium text-blue-600 hover:text-blue-800 disabled:opacity-50 cursor-pointer"
                    >
                      {translatingMsgId === m.id ? (
                        'Translating...'
                      ) : (
                        <span className="flex items-center gap-1">
                          <Globe className="h-3.5 w-3.5" /> Translate to English
                        </span>
                      )}
                    </button>
                  )}
                  <div
                    className={`mt-1 flex items-center justify-between text-xs ${
                      m.direction === 'inbound' ? 'text-[var(--color-ink-faint)]' : 'text-slate-300'
                    }`}
                  >
                    <span>
                      {m.senderName ? `${m.senderName} • ` : ''}
                      {m.channel} · {timeAgo(m.sentAt)}
                    </span>
                    {m.providerMessageId && (
                      <span className="text-[10px] font-mono opacity-80" title={`Meta ID: ${m.providerMessageId}`}>
                        ✓ Meta
                      </span>
                    )}
                  </div>
                </li>
              ))
            )}
          </ol>

          <div className="flex justify-end gap-2">
            <button
              onClick={generateDraft}
              disabled={generating}
              className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3.5 py-2 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50 transition-colors shadow-xs cursor-pointer"
            >
              {generating ? (
                'Generating…'
              ) : (
                <span className="flex items-center gap-1.5">
                  <Sparkles className="h-3.5 w-3.5 text-purple-500" />
                  <span>Generate AI Draft</span>
                </span>
              )}
            </button>
          </div>

          {/* Reply Composer with Live Slash Command Picker */}
          <div className="relative mt-4">
            {/* Canned Autocomplete Dropdown */}
            {(isTypingSlash || showCannedPicker) && (
              <div className="absolute bottom-full mb-2 left-0 right-0 z-30 max-h-72 overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl flex flex-col">
                <div className="bg-[var(--color-page)] border-b border-[var(--color-border)] px-3 py-2 flex items-center justify-between">
                  <div className="flex items-center gap-1.5 text-xs font-bold text-slate-800">
                    <span className="w-4 h-4 rounded-full bg-blue-100 text-blue-700 flex items-center justify-center text-[10px]">
                      /
                    </span>
                    <span>Canned Replies ({filteredCanned.length})</span>
                  </div>
                  <div className="flex items-center gap-1 overflow-x-auto">
                    {cannedCategories.map((cat) => (
                      <button
                        key={cat}
                        type="button"
                        onClick={() => {
                          setCannedCategory(cat)
                          setCannedSelectedIndex(0)
                        }}
                        className={`px-2 py-0.5 rounded-full text-[10px] font-semibold capitalize cursor-pointer transition-colors ${
                          cannedCategory === cat
                            ? 'bg-blue-600 text-white'
                            : 'bg-[var(--color-surface)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-slate-100'
                        }`}
                      >
                        {cat}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="max-h-48 overflow-y-auto divide-y divide-slate-100 p-1">
                  {filteredCanned.length === 0 ? (
                    <div className="py-4 text-center text-xs text-[var(--color-ink-muted)]">
                      No canned replies matching "/{slashQuery}"
                    </div>
                  ) : (
                    filteredCanned.map((c, idx) => {
                      const isSelected = idx === cannedSelectedIndex
                      const preview = resolveCannedVariables(c.content)
                      return (
                        <button
                          key={c.id || c.shortCode || c.short_code}
                          type="button"
                          onClick={() => insertCanned(c)}
                          onMouseEnter={() => setCannedSelectedIndex(idx)}
                          className={`w-full text-left p-2 rounded-lg transition-colors cursor-pointer flex flex-col gap-1 ${
                            isSelected
                              ? 'bg-blue-50/90 border-l-4 border-blue-600 shadow-2xs'
                              : 'hover:bg-[var(--color-page)] border-l-4 border-transparent'
                          }`}
                        >
                          <div className="flex items-center justify-between">
                            <span className="font-mono text-xs font-bold text-blue-700 bg-blue-100/70 px-1.5 py-0.5 rounded">
                              /{c.shortCode || c.short_code}
                            </span>
                            <span className="text-[10px] uppercase font-semibold text-[var(--color-ink-muted)] bg-slate-100 px-1.5 py-0.2 rounded border border-[var(--color-border)]">
                              {c.category}
                            </span>
                          </div>
                          <p className="text-xs text-[var(--color-ink)] line-clamp-2">{preview}</p>
                        </button>
                      )
                    })
                  )}
                </div>

                <div className="bg-[var(--color-page)] px-3 py-1.5 text-[10.5px] text-[var(--color-ink-faint)] border-t border-[var(--color-border)] flex items-center justify-between">
                  <span className="flex items-center gap-1.5">
                    <Lightbulb className="h-3.5 w-3.5 text-amber-500 shrink-0" />
                    <span>
                      Type <code className="bg-slate-200 px-1 rounded text-[var(--color-ink)] font-mono">/</code> to trigger replies
                    </span>
                  </span>
                  <span>Navigate with Arrow keys • Enter to insert • Esc to dismiss</span>
                </div>
              </div>
            )}

            <div className="rounded-xl border border-slate-300 bg-[var(--color-surface)] shadow-xs focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500">
              <textarea
                rows={3}
                value={replyText}
                onChange={(e) => setReplyText(e.target.value)}
                onKeyDown={(e) => {
                  const isPickerOpen = (isTypingSlash || showCannedPicker) && filteredCanned.length > 0
                  if (isPickerOpen) {
                    if (e.key === 'ArrowDown') {
                      e.preventDefault()
                      setCannedSelectedIndex((prev) => (prev + 1) % filteredCanned.length)
                      return
                    }
                    if (e.key === 'ArrowUp') {
                      e.preventDefault()
                      setCannedSelectedIndex((prev) => (prev - 1 + filteredCanned.length) % filteredCanned.length)
                      return
                    }
                    if (e.key === 'Enter' || e.key === 'Tab') {
                      if (!e.shiftKey && !e.ctrlKey && !e.metaKey) {
                        e.preventDefault()
                        if (filteredCanned[cannedSelectedIndex]) {
                          insertCanned(filteredCanned[cannedSelectedIndex])
                        }
                        return
                      }
                    }
                    if (e.key === 'Escape') {
                      e.preventDefault()
                      setShowCannedPicker(false)
                      return
                    }
                  }
                  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                    e.preventDefault()
                    handleSendReply()
                  }
                }}
                placeholder="Type a message to the customer... Start with '/' for canned responses, or press Ctrl+Enter to send"
                className="w-full resize-none px-3.5 py-2.5 text-xs text-slate-800 placeholder-slate-400 focus:outline-hidden"
              />

              <div className="flex items-center justify-between border-t border-[var(--color-border)] px-3 py-2 bg-[var(--color-page)]/50 rounded-b-xl">
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={() => setShowCannedPicker((v) => !v)}
                    className={`text-xs px-2.5 py-1 rounded font-medium transition-colors cursor-pointer flex items-center gap-1.5 ${
                      showCannedPicker
                        ? 'bg-blue-100 text-blue-700'
                        : 'text-[var(--color-ink-muted)] hover:bg-slate-200/60'
                    }`}
                  >
                    <Zap className="h-3.5 w-3.5 text-amber-500" />
                    <span>Saved Replies (/)</span>
                  </button>
                  <span className="text-[11px] text-[var(--color-ink-faint)]">Ctrl + Enter to send</span>
                  {conversation?.language && conversation.language !== 'en' && (
                    <label className="flex items-center gap-1.5 cursor-pointer text-[11px] text-[var(--color-accent)] font-semibold bg-[var(--color-accent-soft)] px-2.5 py-1 rounded-md border border-[var(--color-border)]">
                      <input
                        type="checkbox"
                        checked={autoTranslateOutbound}
                        onChange={(e) => setAutoTranslateOutbound(e.target.checked)}
                        className="h-3.5 w-3.5 rounded text-[var(--color-accent)] border-slate-300"
                      />
                      <span>Auto-translate outbound to {conversation.language.toUpperCase()}</span>
                    </label>
                  )}
                </div>

                <button
                  type="button"
                  onClick={handleSendReply}
                  disabled={!replyText.trim() || isSending}
                  className="rounded-lg bg-blue-600 px-4 py-1.5 text-xs font-semibold text-white hover:bg-blue-700 disabled:opacity-50 shadow-xs transition-colors cursor-pointer"
                >
                  {isSending ? 'Sending…' : 'Send Message'}
                </button>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  )
}
