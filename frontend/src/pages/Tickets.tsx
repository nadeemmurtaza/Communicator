import { useEffect, useState } from 'react'
import {
  X,
  Plus,
  ArrowRight,
  ExternalLink,
  RefreshCw,
  Trash2,
  Link2,
  Radio,
  Send,
  MessageSquare,
  Tag,
  AlertTriangle,
  Search,
  Building2,
  User,
  Ticket as TicketIcon,
  Check,
  XCircle,
} from '../components/icons'
import { Link } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import {
  api,
  type Ticket,
  type TicketStats,
  type LinkedIssue,
  type BroadcastDelivery,
  type BroadcastTrackerUpdateResult,
} from '../lib/api'

/** Routing choices offered when creating a ticket. Anything already stored on a
 * ticket is added to this list (see departmentOptions) so no real value becomes
 * unreachable, but the backend validates departments against nothing. */
const DEPARTMENT_CHOICES = ['support', 'finance', 'logistics', 'engineering', 'warehouse', 'legal']

/** The values the detail selects offer. A stored value outside these lists is
 * shown as its own option rather than being displayed as a blank selector. */
const STATUS_CHOICES = ['open', 'in_progress', 'waiting_on_back_office', 'waiting_on_customer', 'resolved', 'closed']
const PRIORITY_CHOICES = ['low', 'medium', 'high', 'urgent']

/** A value the API returned, or an explicit statement that it returned none. */
function textOrMissing(value: string | null | undefined, missing: string): string {
  return value && value.trim() ? value : missing
}

/** A count from the API, or an explicit statement that none came back -- never a
 * 0 that hides an absence. */
function countLabel(count: number | null | undefined): string {
  return typeof count === 'number' ? String(count) : 'not reported'
}

function dateOrMissing(iso: string | null | undefined, opts: Intl.DateTimeFormatOptions): string {
  if (!iso || !iso.trim()) return 'not recorded'
  const parsed = new Date(iso)
  return Number.isNaN(parsed.getTime()) ? `unreadable ("${iso}")` : parsed.toLocaleDateString(undefined, opts)
}

function dateTimeOrMissing(iso: string | null | undefined): string {
  if (!iso || !iso.trim()) return 'not recorded'
  const parsed = new Date(iso)
  return Number.isNaN(parsed.getTime()) ? `unreadable ("${iso}")` : parsed.toLocaleString()
}

function timeOrMissing(iso: string | null | undefined): string {
  if (!iso || !iso.trim()) return 'no timestamp stored'
  const parsed = new Date(iso)
  return Number.isNaN(parsed.getTime())
    ? `unreadable ("${iso}")`
    : parsed.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

/**
 * One recipient's row from a broadcast, labelled with what the backend actually
 * reported: 'sent' means a provider answered 2xx, 'failed' means the provider
 * refused, 'not_sent' means nothing left the server, 'error' means the send
 * could not be attempted or completed.
 */
function DeliveryRow({ delivery }: { delivery: BroadcastDelivery }) {
  const ticketRef = delivery.ticketId ?? delivery.ticket_id ?? null
  const convRef = delivery.conversationId ?? delivery.conversation_id ?? null
  const msgRef = delivery.messageId ?? delivery.message_id ?? null
  const status = delivery.status ?? null

  const tone =
    status === 'sent'
      ? 'border-emerald-200 bg-emerald-50 text-emerald-900'
      : status === 'failed' || status === 'error'
      ? 'border-rose-200 bg-rose-50 text-rose-900'
      : 'border-amber-200 bg-amber-50 text-amber-900'

  const statusLabel =
    status === 'sent'
      ? 'accepted by provider'
      : status === 'failed'
      ? 'refused by provider'
      : status === 'not_sent'
      ? 'not sent'
      : status === 'error'
      ? 'send error'
      : status ?? 'no status reported'

  return (
    <li className={`rounded-lg border px-2.5 py-1.5 text-[11px] ${tone}`}>
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
        {status === 'sent' ? (
          <Check className="h-3 w-3 shrink-0" />
        ) : status === 'not_sent' ? (
          <AlertTriangle className="h-3 w-3 shrink-0" />
        ) : (
          <XCircle className="h-3 w-3 shrink-0" />
        )}
        <span className="font-semibold uppercase tracking-wide">{statusLabel}</span>
        <span className="font-mono">{textOrMissing(ticketRef, 'no ticket id reported')}</span>
        <span>conv: {textOrMissing(convRef, 'none linked')}</span>
        <span>channel: {textOrMissing(delivery.channel, 'not reported')}</span>
        <span>provider: {textOrMissing(delivery.provider, 'never contacted')}</span>
        <span>
          HTTP:{' '}
          {typeof delivery.httpStatus === 'number'
            ? delivery.httpStatus
            : 'no provider response'}
        </span>
        <span>
          msg: {textOrMissing(msgRef, 'no message row written')} /{' '}
          {textOrMissing(delivery.providerMessageId, 'no provider id')}
        </span>
        {typeof delivery.simulated === 'boolean' && (
          <span>{delivery.simulated ? 'simulated (did not leave the server)' : 'real send'}</span>
        )}
      </div>
      {(delivery.detail || delivery.note) && (
        <p className="mt-1 whitespace-pre-wrap">
          {textOrMissing(delivery.detail, '')}
          {delivery.detail && delivery.note ? ' — ' : ''}
          {textOrMissing(delivery.note, '')}
        </p>
      )}
    </li>
  )
}

export function Tickets() {
  const { session } = useAuth()
  // The only identity this page may attribute work to: the signed-in session's
  // email. Empty means unknown, and writes then omit author_name so the backend
  // stores NULL rather than inventing a person.
  const actorName = session?.email?.trim() || ''

  const [tickets, setTickets] = useState<Ticket[]>([])
  const [stats, setStats] = useState<TicketStats | null>(null)
  const [trackers, setTrackers] = useState<Ticket[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filters
  const [activeTab, setActiveTab] = useState<'all' | 'tracker' | 'customer_issue' | 'back_office' | 'resolved'>('all')
  const [selectedDept, setSelectedDept] = useState<string>('all')
  const [selectedPriority, setSelectedPriority] = useState<string>('all')
  const [searchQuery, setSearchQuery] = useState('')

  // Selected Ticket for Detail Drawer/Modal
  const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null)

  // External Linked Issues
  const [linkedIssues, setLinkedIssues] = useState<LinkedIssue[]>([])
  const [linkedIssuesError, setLinkedIssuesError] = useState<string | null>(null)
  const [loadingLinkedIssues, setLoadingLinkedIssues] = useState(false)
  const [showCreateLinkedIssue, setShowCreateLinkedIssue] = useState(false)
  const [newLinkedProvider, setNewLinkedProvider] = useState<'linear' | 'github' | 'jira'>('linear')
  const [newLinkedTitle, setNewLinkedTitle] = useState('')
  const [newLinkedDescription, setNewLinkedDescription] = useState('')
  const [newLinkedPriority, setNewLinkedPriority] = useState('medium')
  const [creatingLinkedIssue, setCreatingLinkedIssue] = useState(false)
  const [syncingIssueId, setSyncingIssueId] = useState<string | null>(null)
  const [unlinkingIssueId, setUnlinkingIssueId] = useState<string | null>(null)

  // Modals
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showBroadcastModal, setShowBroadcastModal] = useState(false)
  const [broadcastTargetTracker, setBroadcastTargetTracker] = useState<Ticket | null>(null)
  const [broadcastMessage, setBroadcastMessage] = useState('')
  const [broadcastResolveAll, setBroadcastResolveAll] = useState(true)
  const [broadcastSendCustomer, setBroadcastSendCustomer] = useState(true)
  const [submittingBroadcast, setSubmittingBroadcast] = useState(false)
  // What the broadcast really did, reported per recipient instead of an alert.
  const [broadcastOutcome, setBroadcastOutcome] = useState<BroadcastTrackerUpdateResult | null>(null)
  const [broadcastFailure, setBroadcastFailure] = useState<string | null>(null)

  // New Note
  const [newNoteMessage, setNewNoteMessage] = useState('')
  const [newNoteAuthor, setNewNoteAuthor] = useState('')
  const [submittingNote, setSubmittingNote] = useState(false)

  // Link to Tracker
  const [linkTrackerId, setLinkTrackerId] = useState('')
  const [linkingTracker, setLinkingTracker] = useState(false)

  // Create Ticket Form
  const [newTitle, setNewTitle] = useState('')
  const [newDescription, setNewDescription] = useState('')
  const [newType, setNewType] = useState<'customer_issue' | 'back_office' | 'tracker'>('customer_issue')
  const [newDept, setNewDept] = useState('support')
  const [newPriority, setNewPriority] = useState('medium')
  const [newTrackerId, setNewTrackerId] = useState('')
  const [creatingTicket, setCreatingTicket] = useState(false)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [ticketList, statsData, trackerList] = await Promise.all([
        api.listTickets(),
        api.getTicketStats(),
        api.getTrackers(),
      ])
      setTickets(ticketList)
      setStats(statsData)
      setTrackers(trackerList)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tickets')
    } finally {
      setLoading(false)
    }
  }

  const loadLinkedIssues = async (ticketId: string) => {
    setLoadingLinkedIssues(true)
    setLinkedIssuesError(null)
    try {
      const issues = await api.listLinkedIssues(undefined, ticketId)
      setLinkedIssues(issues)
    } catch (err) {
      // A failed read is not an empty list: keep the reason visible to the operator.
      const message = err instanceof Error ? err.message : 'Failed to load linked issues'
      console.error('Failed to load linked issues', err)
      setLinkedIssues([])
      setLinkedIssuesError(message)
    } finally {
      setLoadingLinkedIssues(false)
    }
  }

  const openTicketDetail = async (ticketId: string) => {
    try {
      const detail = await api.getTicket(ticketId)
      setSelectedTicket(detail)
      setShowCreateLinkedIssue(false)
      loadLinkedIssues(detail.id)
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to load ticket details')
    }
  }

  const handleUpdateStatus = async (ticketId: string, newStatus: string) => {
    try {
      const updated = await api.updateTicket(ticketId, { status: newStatus, author_name: actorName || undefined })
      setSelectedTicket(updated)
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to update ticket status')
    }
  }

  const handleUpdatePriority = async (ticketId: string, newPriority: string) => {
    try {
      const updated = await api.updateTicket(ticketId, { priority: newPriority, author_name: actorName || undefined })
      setSelectedTicket(updated)
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to update ticket priority')
    }
  }

  const handleAddNote = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!selectedTicket || !newNoteMessage.trim()) return
    setSubmittingNote(true)
    try {
      await api.addTicketActivity(selectedTicket.id, {
        message: newNoteMessage.trim(),
        activity_type: selectedTicket.ticket_type === 'back_office' ? 'back_office_note' : 'comment',
        // Typed override first, then the signed-in email; blank means no author
        // is sent and the backend stores NULL instead of inventing one.
        author_name: newNoteAuthor.trim() || actorName || undefined,
      })
      setNewNoteMessage('')
      const refreshed = await api.getTicket(selectedTicket.id)
      setSelectedTicket(refreshed)
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to add activity note')
    } finally {
      setSubmittingNote(false)
    }
  }

  const handleLinkTracker = async () => {
    if (!selectedTicket || !linkTrackerId) return
    setLinkingTracker(true)
    try {
      const updated = await api.linkTicketToTracker(selectedTicket.id, linkTrackerId, actorName || undefined)
      setSelectedTicket(updated)
      setLinkTrackerId('')
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to link ticket to tracker')
    } finally {
      setLinkingTracker(false)
    }
  }

  const openBroadcastModal = (tracker: Ticket) => {
    setBroadcastTargetTracker(tracker)
    setBroadcastFailure(null)
    setShowBroadcastModal(true)
  }

  const closeBroadcastModal = () => {
    setShowBroadcastModal(false)
    setBroadcastFailure(null)
  }

  const handleExecuteBroadcast = async () => {
    if (!broadcastTargetTracker || !broadcastMessage.trim()) return
    setSubmittingBroadcast(true)
    setBroadcastFailure(null)
    try {
      const res = await api.broadcastTrackerUpdate(broadcastTargetTracker.id, {
        broadcast_message: broadcastMessage.trim(),
        resolve_all_children: broadcastResolveAll,
        send_customer_message: broadcastSendCustomer,
        author_name: actorName || undefined,
      })
      setBroadcastOutcome(res)
      closeBroadcastModal()
      setBroadcastMessage('')
      if (selectedTicket?.id === broadcastTargetTracker.id) {
        openTicketDetail(broadcastTargetTracker.id)
      }
      loadData()
    } catch (err) {
      // The dispatch itself failed: keep the modal open and show the API's own
      // reason rather than claiming anything was sent.
      setBroadcastOutcome(null)
      setBroadcastFailure(err instanceof Error ? err.message : 'Broadcast request failed')
      loadData()
    } finally {
      setSubmittingBroadcast(false)
    }
  }

  const handleCreateTicket = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newTitle.trim() || !newDescription.trim()) return
    setCreatingTicket(true)
    try {
      const created = await api.createTicket({
        title: newTitle.trim(),
        description: newDescription.trim(),
        ticket_type: newType,
        department: newDept,
        priority: newPriority,
        tracker_id: newTrackerId || null,
        author_name: actorName || undefined,
      })
      setShowCreateModal(false)
      setNewTitle('')
      setNewDescription('')
      setNewTrackerId('')
      await loadData()
      openTicketDetail(created.id)
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to create ticket')
    } finally {
      setCreatingTicket(false)
    }
  }

  const handleCreateLinkedIssue = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!selectedTicket || !newLinkedTitle.trim()) return
    setCreatingLinkedIssue(true)
    try {
      await api.createLinkedIssue({
        provider: newLinkedProvider,
        title: newLinkedTitle.trim(),
        description: newLinkedDescription.trim() || undefined,
        priority: newLinkedPriority.trim() || undefined,
        ticket_id: selectedTicket.id,
        conversation_id: selectedTicket.conversation_id || undefined,
        // Signed-in email or nothing at all; the backend stores NULL rather than
        // a default account address.
        actor_email: actorName || undefined,
      })
      setShowCreateLinkedIssue(false)
      setNewLinkedTitle('')
      setNewLinkedDescription('')
      await loadLinkedIssues(selectedTicket.id)
      const refreshed = await api.getTicket(selectedTicket.id)
      setSelectedTicket(refreshed)
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to create linked issue')
    } finally {
      setCreatingLinkedIssue(false)
    }
  }

  const handleSyncIssue = async (issueId: string) => {
    setSyncingIssueId(issueId)
    try {
      await api.syncIssueStatus(issueId)
      if (selectedTicket) {
        await loadLinkedIssues(selectedTicket.id)
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to sync issue')
    } finally {
      setSyncingIssueId(null)
    }
  }

  const handleUnlinkIssue = async (issueId: string) => {
    if (!confirm('Are you sure you want to unlink this external issue from this ticket?')) return
    setUnlinkingIssueId(issueId)
    try {
      await api.unlinkIssue(issueId)
      if (selectedTicket) {
        await loadLinkedIssues(selectedTicket.id)
        const refreshed = await api.getTicket(selectedTicket.id)
        setSelectedTicket(refreshed)
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to unlink issue')
    } finally {
      setUnlinkingIssueId(null)
    }
  }

  // Filtered List
  const filteredTickets = tickets.filter((t) => {
    // Tab filter
    if (activeTab === 'tracker' && t.ticket_type !== 'tracker') return false
    if (activeTab === 'customer_issue' && t.ticket_type !== 'customer_issue') return false
    if (activeTab === 'back_office' && t.ticket_type !== 'back_office') return false
    if (activeTab === 'resolved' && t.status !== 'resolved' && t.status !== 'closed') return false

    // Department filter
    if (selectedDept !== 'all' && t.department !== selectedDept) return false

    // Priority filter
    if (selectedPriority !== 'all' && t.priority !== selectedPriority) return false

    // Search query
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase()
      const matchNumber = t.ticket_number?.toLowerCase().includes(q) || false
      const matchTitle = t.title?.toLowerCase().includes(q) || false
      const matchDesc = t.description?.toLowerCase().includes(q) || false
      const matchContact = t.contact_name?.toLowerCase().includes(q) || false
      if (!matchNumber && !matchTitle && !matchDesc && !matchContact) return false
    }

    return true
  })

  // Tab counts derived from the ticket rows actually loaded, so a stats read that
  // never returned can't be shown as "0 tracker tickets".
  const trackerCount = tickets.filter((t) => t.ticket_type === 'tracker').length
  const openTrackerCount = tickets.filter(
    (t) => t.ticket_type === 'tracker' && t.status !== 'resolved' && t.status !== 'closed',
  ).length
  const customerIssueCount = tickets.filter((t) => t.ticket_type === 'customer_issue').length
  const backOfficeCount = tickets.filter((t) => t.ticket_type === 'back_office').length
  const resolvedCount = tickets.filter((t) => t.status === 'resolved' || t.status === 'closed').length
  const tabCount = (n: number) => (loading || error ? 'count unavailable' : String(n))

  // Departments the data really contains -- no invented filter vocabulary.
  const departmentsInData = Array.from(
    new Set(tickets.map((t) => t.department).filter((d) => !!d && !!d.trim())),
  ).sort()
  // Routing choices for the create form: the fixed vocabulary plus every value
  // already stored, so a ticket filed under an unusual department stays reachable.
  const departmentOptions = Array.from(new Set([...DEPARTMENT_CHOICES, ...departmentsInData])).sort()
  const departmentBreakdown = stats
    ? Object.entries(stats.department_breakdown ?? {}).map(([dept, count]) => `${dept} (${count})`)
    : []

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
              Ticketing & Back-Office Collaboration
            </h1>
            <span className="rounded-full bg-indigo-100 px-2.5 py-0.5 text-xs font-semibold text-indigo-800">
              Unified Hub
            </span>
          </div>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            One ticketing system for customer issues, back-office collaboration, and tracker tickets that resolve widespread problems across conversations.
          </p>
          <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
            Writes from this page are attributed to:{' '}
            <span className="font-medium text-[var(--color-ink)]">
              {actorName || 'no signed-in email available — author fields are left empty and stored as none'}
            </span>
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center justify-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-[var(--color-accent-hover)] transition-colors"
          >
            <Plus className="h-4 w-4" />
            Create Ticket
          </button>
        </div>
      </div>

      {/* KPI Stats Cards */}
      {stats ? (
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
                Active Tickets
              </div>
              <TicketIcon className="h-4 w-4 text-[var(--color-accent)]" />
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-[var(--color-ink)]">{stats.open_tickets}</span>
              <span className="text-xs text-[var(--color-ink-faint)]">of {stats.total_tickets} total</span>
            </div>
            <div className="mt-2 text-xs text-[var(--color-ink-muted)]">
              {typeof stats.urgent_tickets === 'number'
                ? `${stats.urgent_tickets} urgent not yet resolved`
                : 'urgent count not returned'}
            </div>
          </div>

          <div className="rounded-xl border border-rose-200 bg-rose-50/50 p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="text-xs font-medium text-rose-700 uppercase tracking-wider">
                Tracker Incidents
              </div>
              <Radio className="h-4 w-4 text-rose-600" />
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-rose-900">{stats.tracker_tickets}</span>
              <span className="rounded-full bg-rose-200 px-2 py-0.5 text-xs font-semibold text-rose-800">
                {loading || error ? 'open count unavailable' : `${openTrackerCount} open now`}
              </span>
            </div>
            <div className="mt-2 text-xs text-rose-600">
              {stats.impacted_customers_count} distinct contacts stored on tickets
            </div>
          </div>

          <div className="rounded-xl border border-violet-200 bg-violet-50/50 p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="text-xs font-medium text-violet-700 uppercase tracking-wider">
                Back-Office Load
              </div>
              <Building2 className="h-4 w-4 text-violet-600" />
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-violet-900">{stats.back_office_tickets}</span>
              <span className="text-xs text-violet-600">tasks, all statuses</span>
            </div>
            <div className="mt-2 text-xs text-violet-600">
              {departmentBreakdown.length > 0 ? departmentBreakdown.join(', ') : 'no departments returned by stats'}
            </div>
          </div>

          <div className="rounded-xl border border-emerald-200 bg-emerald-50/50 p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="text-xs font-medium text-emerald-700 uppercase tracking-wider">
                Customer Issues
              </div>
              <User className="h-4 w-4 text-emerald-600" />
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-emerald-900">{stats.customer_issues}</span>
              <span className="text-xs text-emerald-600">reported</span>
            </div>
            <div className="mt-2 text-xs text-emerald-600">
              {stats.resolved_tickets} resolved across all ticket types
            </div>
            <div className="mt-1 text-xs text-emerald-700">
              {typeof stats.avg_resolution_time_hours === 'number'
                ? `${stats.avg_resolution_time_hours}h average time to resolve`
                : 'no ticket with a resolution timestamp yet — no average available'}
            </div>
          </div>
        </div>
      ) : (
        !loading && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 p-3 text-xs text-amber-800">
            Ticket totals are not shown because the stats read did not return
            {error ? ` (${error})` : ' — they are not reported as zero.'}
          </div>
        )
      )}

      {/* Broadcast outcome: what the provider actually accepted, refused, or never sent */}
      {broadcastOutcome && (
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm space-y-3">
          <div className="flex items-start justify-between gap-3">
            <div>
              <h2 className="flex items-center gap-2 text-sm font-bold text-[var(--color-ink)]">
                <Radio className="h-4 w-4 text-rose-600" />
                Broadcast outcome for{' '}
                <span className="font-mono">
                  {textOrMissing(broadcastOutcome.tracker_number, 'tracker with no ticket number stored')}
                </span>
              </h2>
              <p className="mt-1 text-xs text-[var(--color-ink-muted)]">{broadcastOutcome.note}</p>
            </div>
            <button
              onClick={() => setBroadcastOutcome(null)}
              aria-label="Dismiss broadcast outcome"
              className="rounded-lg p-1 text-[var(--color-ink-faint)] hover:bg-slate-100 hover:text-[var(--color-ink-muted)]"
            >
              <X className="h-4 w-4" />
            </button>
          </div>

          <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-xs sm:grid-cols-4">
            <div>
              <dt className="text-[var(--color-ink-faint)] uppercase tracking-wide">Linked tickets run over</dt>
              <dd className="font-semibold text-[var(--color-ink)]">{broadcastOutcome.impacted_tickets}</dd>
            </div>
            <div>
              <dt className="text-[var(--color-ink-faint)] uppercase tracking-wide">Accepted by provider</dt>
              <dd className="font-semibold text-emerald-800">{broadcastOutcome.messages_sent}</dd>
            </div>
            <div>
              <dt className="text-[var(--color-ink-faint)] uppercase tracking-wide">Refused by provider</dt>
              <dd className="font-semibold text-rose-800">{broadcastOutcome.messages_failed}</dd>
            </div>
            <div>
              <dt className="text-[var(--color-ink-faint)] uppercase tracking-wide">Never left the server</dt>
              <dd className="font-semibold text-amber-800">{broadcastOutcome.messages_not_sent}</dd>
            </div>
          </dl>

          <div className="space-y-1 text-xs text-[var(--color-ink)]">
            <p>
              {!broadcastOutcome.customer_message_requested ? (
                <>No customer message was requested, so no customer was contacted.</>
              ) : broadcastOutcome.messages_sent === 0 ? (
                <>
                  No customer message was accepted by a provider.{' '}
                  {broadcastOutcome.deliveries.length > 0
                    ? 'Each recipient has a reason listed below.'
                    : 'No per-recipient delivery row came back.'}
                </>
              ) : (
                <>
                  {broadcastOutcome.messages_sent} customer message(s) accepted by a provider,{' '}
                  {broadcastOutcome.messages_failed} refused, {broadcastOutcome.messages_not_sent} not sent. Only
                  accepted sends reached a customer.
                </>
              )}
            </p>
            <p>
              {broadcastOutcome.resolve_all_requested ? (
                <>
                  {broadcastOutcome.resolved_tickets} linked ticket(s) resolved by this request,{' '}
                  {broadcastOutcome.already_resolved} already resolved, {broadcastOutcome.resolve_failed} not
                  resolved.
                </>
              ) : (
                <>Auto-resolve was not requested, so no linked ticket was closed.</>
              )}
            </p>
            <p>
              {broadcastOutcome.tracker_resolved
                ? `Tracker marked resolved; customer-notified flag ${
                    broadcastOutcome.customers_notified ? 'recorded as sent' : 'recorded as not sent'
                  }.`
                : `Tracker was NOT marked resolved${
                    broadcastOutcome.tracker_resolve_error ? `: ${broadcastOutcome.tracker_resolve_error}` : '.'
                  }`}
            </p>
          </div>

          {broadcastOutcome.activity_errors.length > 0 && (
            <div className="rounded-lg border border-amber-200 bg-amber-50 p-2.5 text-xs text-amber-900">
              <span className="font-semibold">Audit rows that were not written:</span>
              <ul className="mt-1 list-disc space-y-0.5 pl-4">
                {broadcastOutcome.activity_errors.map((err, i) => (
                  <li key={`activity-error-${i}`}>{err}</li>
                ))}
              </ul>
            </div>
          )}

          <div>
            <h3 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">
              Per-recipient deliveries ({broadcastOutcome.deliveries.length})
            </h3>
            {broadcastOutcome.deliveries.length === 0 ? (
              <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
                {broadcastOutcome.customer_message_requested
                  ? 'The request reported no delivery rows, so no send can be confirmed for any linked ticket.'
                  : 'No send was attempted because customer messaging was not requested.'}
              </p>
            ) : (
              <ul className="mt-1.5 max-h-48 space-y-1 overflow-y-auto">
                {broadcastOutcome.deliveries.map((delivery, i) => (
                  <DeliveryRow key={`${delivery.ticketId ?? delivery.ticket_id ?? 'delivery'}-${i}`} delivery={delivery} />
                ))}
              </ul>
            )}
          </div>
        </div>
      )}

      {/* Primary Filters & Tabs */}
      <div className="flex flex-col gap-4 border-b border-[var(--color-border)] pb-4 md:flex-row md:items-center md:justify-between">
        <div className="flex flex-wrap items-center gap-1.5">
          <button
            onClick={() => {
              setActiveTab('all')
              setSelectedDept('all')
            }}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
              activeTab === 'all'
                ? 'bg-[var(--color-accent)] text-white'
                : 'bg-[var(--color-surface)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-slate-100'
            }`}
          >
            All Tickets ({tabCount(tickets.length)})
          </button>
          <button
            onClick={() => {
              setActiveTab('tracker')
              setSelectedDept('all')
            }}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors flex items-center gap-1.5 ${
              activeTab === 'tracker'
                ? 'bg-rose-600 text-white'
                : 'bg-[var(--color-surface)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-slate-100'
            }`}
          >
            <span className={`h-2 w-2 rounded-full bg-rose-500 ${!loading && !error && openTrackerCount > 0 ? 'animate-pulse' : ''}`}></span>
            Tracker Tickets ({tabCount(trackerCount)})
          </button>
          <button
            onClick={() => {
              setActiveTab('customer_issue')
              setSelectedDept('all')
            }}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
              activeTab === 'customer_issue'
                ? 'bg-emerald-700 text-white'
                : 'bg-[var(--color-surface)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-slate-100'
            }`}
          >
            Customer Issues ({tabCount(customerIssueCount)})
          </button>
          <button
            onClick={() => {
              setActiveTab('back_office')
              setSelectedDept('all')
            }}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
              activeTab === 'back_office'
                ? 'bg-indigo-700 text-white'
                : 'bg-[var(--color-surface)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-slate-100'
            }`}
          >
            Back-Office Collaboration ({tabCount(backOfficeCount)})
          </button>
          <button
            onClick={() => {
              setActiveTab('resolved')
              setSelectedDept('all')
            }}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
              activeTab === 'resolved'
                ? 'bg-slate-700 text-white'
                : 'bg-[var(--color-surface)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-slate-100'
            }`}
          >
            Resolved ({tabCount(resolvedCount)})
          </button>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {/* Department Dropdown -- only the departments present in the loaded data */}
          <select
            value={selectedDept}
            onChange={(e) => setSelectedDept(e.target.value)}
            className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1.5 text-xs font-medium text-[var(--color-ink)] shadow-sm focus:border-indigo-500 focus:outline-none"
          >
            <option value="all">All Departments ({tabCount(tickets.length)})</option>
            {departmentsInData.map((dept) => (
              <option key={dept} value={dept}>
                {dept}
              </option>
            ))}
          </select>

          {/* Priority Dropdown */}
          <select
            value={selectedPriority}
            onChange={(e) => setSelectedPriority(e.target.value)}
            className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1.5 text-xs font-medium text-[var(--color-ink)] shadow-sm focus:border-indigo-500 focus:outline-none"
          >
            <option value="all">All Priorities</option>
            <option value="urgent">Urgent</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>

          {/* Search Input */}
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search tickets, IDs, contacts..."
              className="w-48 sm:w-60 rounded-lg border border-slate-300 bg-[var(--color-surface)] pl-8 pr-3 py-1.5 text-xs placeholder:text-[var(--color-ink-faint)] focus:border-indigo-500 focus:outline-none"
            />
          </div>
        </div>
      </div>

      {/* Ticket List */}
      {loading ? (
        <div className="py-12 text-center text-sm text-[var(--color-ink-muted)]">Loading tickets...</div>
      ) : error ? (
        <div className="rounded-lg bg-rose-50 p-4 text-sm text-rose-700">{error}</div>
      ) : filteredTickets.length === 0 ? (
        <div className="rounded-xl border border-dashed border-slate-300 p-12 text-center">
          <p className="text-sm text-[var(--color-ink-muted)]">
            {tickets.length === 0
              ? 'The ticket list came back with no rows -- there are no tickets stored yet.'
              : `No ticket matches the current filters (${filteredTickets.length} of ${tickets.length} loaded tickets shown).`}
          </p>
          <button
            onClick={() => setShowCreateModal(true)}
            className="mt-3 inline-flex items-center gap-1 text-xs font-semibold text-[var(--color-accent)] hover:text-indigo-800"
          >
            <Plus className="h-3.5 w-3.5" />
            Create a ticket now
          </button>
        </div>
      ) : (
        <div className="space-y-3">
          {filteredTickets.map((t) => (
            <div
              key={t.id}
              onClick={() => openTicketDetail(t.id)}
              className={`cursor-pointer rounded-xl border p-4 transition-all hover:shadow-md ${
                t.ticket_type === 'tracker'
                  ? 'border-rose-300 bg-rose-50/30'
                  : t.ticket_type === 'back_office'
                  ? 'border-[var(--color-border)] bg-[var(--color-accent-soft)]/20'
                  : 'border-[var(--color-border)] bg-[var(--color-surface)]'
              }`}
            >
              <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs font-bold text-[var(--color-ink)]">
                    {textOrMissing(t.ticket_number, 'no ticket number stored')}
                  </span>

                  {/* Type Badge */}
                  {t.ticket_type === 'tracker' ? (
                    <span className="rounded-md bg-rose-100 px-2 py-0.5 text-xs font-semibold text-rose-800">
                      Tracker Ticket
                    </span>
                  ) : t.ticket_type === 'back_office' ? (
                    <span className="rounded-md bg-indigo-100 px-2 py-0.5 text-xs font-semibold text-indigo-800">
                      Back-Office
                    </span>
                  ) : t.ticket_type === 'customer_issue' ? (
                    <span className="rounded-md bg-emerald-100 px-2 py-0.5 text-xs font-semibold text-emerald-800">
                      Customer Issue
                    </span>
                  ) : (
                    <span className="rounded-md bg-slate-100 px-2 py-0.5 text-xs font-semibold text-[var(--color-ink-muted)]">
                      type not stored
                    </span>
                  )}

                  {/* Department Badge */}
                  <span className="rounded-md bg-slate-100 px-2 py-0.5 text-xs font-medium text-[var(--color-ink-muted)] uppercase">
                    {textOrMissing(t.department, 'no department')}
                  </span>

                  {/* Priority Badge */}
                  <span
                    className={`rounded-md px-2 py-0.5 text-xs font-medium ${
                      t.priority === 'urgent'
                        ? 'bg-red-100 text-red-800 font-bold'
                        : t.priority === 'high'
                        ? 'bg-amber-100 text-amber-800'
                        : t.priority
                        ? 'bg-slate-100 text-[var(--color-ink)]'
                        : 'bg-slate-100 text-[var(--color-ink-faint)] italic'
                    }`}
                  >
                    {t.priority ? t.priority.toUpperCase() : 'priority not stored'}
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {/* Status Indicator */}
                  <span
                    className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                      t.status === 'resolved'
                        ? 'bg-emerald-100 text-emerald-800'
                        : t.status === 'in_progress'
                        ? 'bg-blue-100 text-blue-800'
                        : t.status === 'waiting_on_back_office'
                        ? 'bg-purple-100 text-purple-800'
                        : t.status
                        ? 'bg-amber-100 text-amber-800'
                        : 'bg-slate-100 text-[var(--color-ink-faint)]'
                    }`}
                  >
                    {t.status ? t.status.replace(/_/g, ' ') : 'status not stored'}
                  </span>
                  <span className="text-xs text-[var(--color-ink-faint)]">
                    {dateOrMissing(t.created_at, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
                  </span>
                </div>
              </div>

              {/* Title & Description */}
              <div className="mt-2">
                <h3 className="text-base font-semibold text-[var(--color-ink)]">
                  {textOrMissing(t.title, 'No title stored')}
                </h3>
                <p className="mt-1 line-clamp-2 text-xs text-[var(--color-ink-muted)]">
                  {textOrMissing(t.description, 'No description stored')}
                </p>
              </div>

              {/* Tracker / Linked Metadata Footer */}
              <div className="mt-3 flex flex-wrap items-center gap-3 border-t border-[var(--color-border)] pt-3 text-xs text-[var(--color-ink-muted)]">
                {t.ticket_type === 'tracker' && (
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-rose-700">
                      {typeof t.child_count === 'number'
                        ? `${t.child_count} linked issues`
                        : 'linked-issue count not returned'}
                    </span>
                    <span className="text-slate-300">•</span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        openBroadcastModal(t)
                      }}
                      className="inline-flex items-center gap-1 rounded bg-rose-600 px-2 py-0.5 text-xs font-semibold text-white hover:bg-rose-700 transition-colors"
                    >
                      <Radio className="h-3 w-3" />
                      Broadcast & Resolve
                    </button>
                  </div>
                )}

                {t.tracker_id && (
                  <div className="flex items-center gap-1 text-rose-700">
                    <span className="font-medium">Linked to Tracker:</span>
                    <span className="font-mono font-semibold">
                      {textOrMissing(t.tracker_number, 'no tracker number stored')}
                    </span>
                    {t.tracker_title && <span className="truncate max-w-[200px]">({t.tracker_title})</span>}
                  </div>
                )}

                {t.contact_name && (
                  <div className="flex items-center gap-1 text-[var(--color-ink)]">
                    <span className="text-[var(--color-ink-faint)]">Customer:</span>
                    <span className="font-medium">{t.contact_name}</span>
                    {t.contact_identifier && (
                      <span className="text-[var(--color-ink-faint)] font-mono">({t.contact_identifier})</span>
                    )}
                  </div>
                )}

                {t.conversation_id && (
                  <Link
                    to={`/conversations/${t.conversation_id}`}
                    onClick={(e) => e.stopPropagation()}
                    className="inline-flex items-center gap-1 text-[var(--color-accent)] hover:text-indigo-800 font-medium"
                  >
                    <span>View Conversation</span>
                    <ArrowRight className="h-3 w-3" />
                  </Link>
                )}

                {t.assigned_to_name && (
                  <div className="ml-auto text-[var(--color-ink-muted)]">
                    Assignee: <span className="font-medium">{t.assigned_to_name}</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Ticket Detail Slide-out / Modal */}
      {selectedTicket && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="relative flex max-h-[90vh] w-full max-w-3xl flex-col rounded-2xl bg-[var(--color-surface)] shadow-2xl overflow-hidden">
            {/* Modal Header */}
            <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4 bg-[var(--color-page)]">
              <div className="flex items-center gap-2">
                <span className="font-mono text-sm font-bold text-[var(--color-ink)]">
                  {textOrMissing(selectedTicket.ticket_number, 'no ticket number stored')}
                </span>
                <span
                  className={`rounded px-2 py-0.5 text-xs font-semibold ${
                    selectedTicket.ticket_type === 'tracker'
                      ? 'bg-rose-100 text-rose-800'
                      : selectedTicket.ticket_type === 'back_office'
                      ? 'bg-indigo-100 text-indigo-800'
                      : selectedTicket.ticket_type === 'customer_issue'
                      ? 'bg-emerald-100 text-emerald-800'
                      : 'bg-slate-100 text-[var(--color-ink-muted)]'
                  }`}
                >
                  {selectedTicket.ticket_type
                    ? selectedTicket.ticket_type.replace(/_/g, ' ').toUpperCase()
                    : 'TYPE NOT STORED'}
                </span>
                <span className="rounded bg-slate-200 px-2 py-0.5 text-xs font-semibold text-[var(--color-ink)] uppercase">
                  {textOrMissing(selectedTicket.department, 'no department stored')}
                </span>
              </div>
              <button
                onClick={() => setSelectedTicket(null)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] p-1 rounded-lg hover:bg-slate-100"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="flex-1 overflow-y-auto p-6 space-y-6">
              {/* Title & Status/Priority Controls */}
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <h2 className="text-xl font-bold text-[var(--color-ink)]">
                    {textOrMissing(selectedTicket.title, 'No title stored')}
                  </h2>
                  <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
                    Created on {dateTimeOrMissing(selectedTicket.created_at)}
                  </p>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <div className="flex items-center gap-1.5">
                    <label className="text-xs font-semibold text-[var(--color-ink-muted)]">Status:</label>
                    <select
                      value={selectedTicket.status ?? ''}
                      onChange={(e) => handleUpdateStatus(selectedTicket.id, e.target.value)}
                      className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs font-medium text-slate-800 shadow-sm focus:border-indigo-500 focus:outline-none"
                    >
                      {!STATUS_CHOICES.includes(selectedTicket.status) && (
                        <option value={selectedTicket.status}>
                          {selectedTicket.status ? `${selectedTicket.status} (as stored)` : 'no status stored'}
                        </option>
                      )}
                      <option value="open">Open</option>
                      <option value="in_progress">In Progress</option>
                      <option value="waiting_on_back_office">Waiting on Back-Office</option>
                      <option value="waiting_on_customer">Waiting on Customer</option>
                      <option value="resolved">Resolved</option>
                      <option value="closed">Closed</option>
                    </select>
                  </div>

                  <div className="flex items-center gap-1.5">
                    <label className="text-xs font-semibold text-[var(--color-ink-muted)]">Priority:</label>
                    <select
                      value={selectedTicket.priority ?? ''}
                      onChange={(e) => handleUpdatePriority(selectedTicket.id, e.target.value)}
                      className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs font-medium text-slate-800 shadow-sm focus:border-indigo-500 focus:outline-none"
                    >
                      {!PRIORITY_CHOICES.includes(selectedTicket.priority) && (
                        <option value={selectedTicket.priority}>
                          {selectedTicket.priority ? `${selectedTicket.priority} (as stored)` : 'no priority stored'}
                        </option>
                      )}
                      <option value="low">Low</option>
                      <option value="medium">Medium</option>
                      <option value="high">High</option>
                      <option value="urgent">Urgent</option>
                    </select>
                  </div>
                </div>
              </div>

              {/* Description */}
              <div className="rounded-xl bg-[var(--color-page)] p-4 text-xs text-[var(--color-ink)] leading-relaxed border border-[var(--color-border)]">
                <span className="font-semibold block mb-1 text-[var(--color-ink)]">Description:</span>
                {textOrMissing(selectedTicket.description, 'No description stored on this ticket.')}
              </div>

              {/* Resolution Summary Banner if Resolved */}
              {selectedTicket.resolution_summary ? (
                <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-xs text-emerald-800">
                  <span className="font-semibold block mb-1">Resolution Summary:</span>
                  {selectedTicket.resolution_summary}
                  {selectedTicket.resolved_at ? ` — resolved ${dateTimeOrMissing(selectedTicket.resolved_at)}` : ' — no resolution timestamp stored'}
                </div>
              ) : selectedTicket.status === 'resolved' || selectedTicket.status === 'closed' ? (
                <div className="rounded-xl border border-amber-200 bg-amber-50 p-4 text-xs text-amber-900">
                  This ticket is marked {selectedTicket.status} but no resolution summary is stored.
                </div>
              ) : null}

              {/* Tracker Widespread Management Banner */}
              {selectedTicket.ticket_type === 'tracker' && (
                <div className="rounded-xl border border-rose-200 bg-rose-50/70 p-4 space-y-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="text-sm font-bold text-rose-900 flex items-center gap-2">
                        <span>Widespread Incident Tracker</span>
                        <span className="rounded-full bg-rose-200 px-2 py-0.5 text-xs font-semibold text-rose-800">
                          {selectedTicket.child_tickets
                            ? `${selectedTicket.child_tickets.length} Linked Issues`
                            : `${countLabel(selectedTicket.child_count)} linked issues from the list count`}
                        </span>
                      </h4>
                      <p className="text-xs text-rose-700 mt-0.5">
                        A broadcast asks each linked conversation's provider to deliver the message and can close the
                        linked tickets. Provider refusals and tickets with no conversation or no address are reported
                        per recipient instead of counted as sent.
                      </p>
                    </div>
                    <button
                      onClick={() => openBroadcastModal(selectedTicket)}
                      className="inline-flex items-center gap-1.5 rounded-lg bg-rose-600 px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-rose-700 transition-colors"
                    >
                      <Radio className="h-3.5 w-3.5" />
                      Broadcast & Resolve All
                    </button>
                  </div>

                  {/* List of child tickets */}
                  {selectedTicket.child_tickets && selectedTicket.child_tickets.length > 0 ? (
                    <div className="space-y-1.5 mt-2">
                      <span className="text-xs font-semibold text-rose-900">Linked Customer Issues:</span>
                      <div className="max-h-40 overflow-y-auto space-y-1">
                        {selectedTicket.child_tickets.map((child) => (
                          <div
                            key={child.id}
                            className="flex items-center justify-between rounded bg-[var(--color-surface)] p-2 text-xs border border-rose-200"
                          >
                            <div className="flex items-center gap-2">
                              <span className="font-mono font-bold text-[var(--color-ink)]">
                                {textOrMissing(child.ticket_number, 'no number stored')}
                              </span>
                              <span className="text-slate-800">
                                {textOrMissing(child.title, 'no title stored')}
                              </span>
                              {child.contact_name && (
                                <span className="text-[var(--color-ink-muted)]">({child.contact_name})</span>
                              )}
                            </div>
                            <div className="flex items-center gap-2">
                              <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] text-[var(--color-ink-muted)]">
                                {textOrMissing(child.status, 'status not stored')}
                              </span>
                              {child.conversation_id ? (
                                <Link
                                  to={`/conversations/${child.conversation_id}`}
                                  className="inline-flex items-center gap-0.5 text-[var(--color-accent)] hover:underline text-[11px]"
                                >
                                  <span>Chat</span>
                                  <ArrowRight className="h-2.5 w-2.5" />
                                </Link>
                              ) : (
                                <span className="text-[11px] text-amber-700">no conversation — cannot be messaged</span>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  ) : (
                    <p className="mt-2 rounded-lg border border-dashed border-rose-300 bg-[var(--color-surface)] p-2 text-xs text-rose-800">
                      {selectedTicket.child_tickets
                        ? 'No customer ticket is linked to this tracker, so a broadcast has no recipient to message and no ticket to resolve.'
                        : 'The linked tickets were not returned by the detail read, so this broadcast may reach fewer tickets than the tracker reports.'}
                    </p>
                  )}
                </div>
              )}

              {/* Link to Tracker for Child/Customer Tickets */}
              {selectedTicket.ticket_type !== 'tracker' && (
                <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
                  <h4 className="text-xs font-semibold text-[var(--color-ink)] mb-2">
                    Tracker Incident Link
                  </h4>
                  {selectedTicket.tracker_id ? (
                    <div className="flex items-center justify-between text-xs bg-rose-50 p-2.5 rounded-lg border border-rose-200 text-rose-800">
                      <div>
                        <span className="font-semibold">Linked to: </span>
                        <span className="font-mono">
                          {textOrMissing(selectedTicket.tracker_number, 'no tracker number stored')}
                        </span>{' '}
                        - {textOrMissing(selectedTicket.tracker_title, 'no tracker title stored')}
                      </div>
                      <button
                        onClick={() => openTicketDetail(selectedTicket.tracker_id!)}
                        className="text-rose-700 font-semibold underline hover:text-rose-900"
                      >
                        Open Tracker
                      </button>
                    </div>
                  ) : (
                    <>
                      <div className="flex items-center gap-2">
                        <select
                          value={linkTrackerId}
                          onChange={(e) => setLinkTrackerId(e.target.value)}
                          className="flex-1 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                        >
                          <option value="">
                            {trackers.length > 0
                              ? 'Select an active tracker incident to link...'
                              : 'No tracker ticket is available to link'}
                          </option>
                          {trackers.map((tr) => (
                            <option key={tr.id} value={tr.id}>
                              {tr.ticket_number}: {tr.title} ({tr.department})
                            </option>
                          ))}
                        </select>
                        <button
                          onClick={handleLinkTracker}
                          disabled={!linkTrackerId || linkingTracker}
                          className="rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 transition-colors"
                        >
                          {linkingTracker ? 'Linking...' : 'Link to Tracker'}
                        </button>
                      </div>
                      {trackers.length === 0 && (
                        <p className="mt-1.5 text-[11px] text-[var(--color-ink-faint)]">
                          The tracker read returned no tracker tickets, so there is nothing to link this ticket to yet.
                        </p>
                      )}
                    </>
                  )}
                </div>
              )}

              {/* External Issue Trackers (Linear / GitHub / Jira) */}
              <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Link2 className="h-4 w-4 text-[var(--color-accent)]" />
                    <h4 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">
                      External Issue Trackers (Linear / GitHub / Jira)
                    </h4>
                    <span className="rounded-full bg-slate-100 px-2 py-0.5 text-[10px] font-semibold text-[var(--color-ink-muted)]">
                      {linkedIssuesError ? 'read failed' : `${linkedIssues.length} linked`}
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => {
                      setNewLinkedTitle(selectedTicket.title || '')
                      setNewLinkedDescription(selectedTicket.description || '')
                      setNewLinkedPriority(selectedTicket.priority || '')
                      setShowCreateLinkedIssue(!showCreateLinkedIssue)
                    }}
                    className="inline-flex items-center gap-1 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1 text-xs font-medium text-[var(--color-ink)] hover:bg-slate-50 transition-colors"
                  >
                    <Plus className="h-3 w-3" />
                    {showCreateLinkedIssue ? 'Cancel' : 'Link / Create Issue'}
                  </button>
                </div>

                {/* Create linked issue inline form */}
                {showCreateLinkedIssue && (
                  <form onSubmit={handleCreateLinkedIssue} className="rounded-xl border border-indigo-200 bg-[var(--color-accent-soft)]/30 p-3 space-y-3">
                    <div className="text-xs font-semibold text-indigo-950">
                      Create & Link Issue to External Tracker
                    </div>
                    <p className="text-[11px] text-[var(--color-ink-muted)]">
                      Recorded as created by:{' '}
                      <span className="font-medium text-[var(--color-ink)]">
                        {actorName || 'no signed-in email in this session — the issue row will store no creator'}
                      </span>
                    </p>
                    <div className="grid grid-cols-2 gap-2">
                      <div>
                        <label className="text-[11px] font-medium text-[var(--color-ink-muted)] block mb-1">
                          Provider:
                        </label>
                        <select
                          value={newLinkedProvider}
                          onChange={(e) => setNewLinkedProvider(e.target.value as any)}
                          className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                        >
                          <option value="linear">Linear (Engineering)</option>
                          <option value="github">GitHub Issues (Repo)</option>
                          <option value="jira">Jira Cloud (Support & Escalations)</option>
                        </select>
                      </div>
                      <div>
                        <label className="text-[11px] font-medium text-[var(--color-ink-muted)] block mb-1">
                          Priority:
                        </label>
                        <select
                          value={newLinkedPriority}
                          onChange={(e) => setNewLinkedPriority(e.target.value)}
                          className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                        >
                          <option value="">Not specified on the ticket</option>
                          <option value="low">Low</option>
                          <option value="medium">Medium</option>
                          <option value="high">High</option>
                          <option value="urgent">Urgent</option>
                        </select>
                      </div>
                    </div>
                    <div>
                      <label className="text-[11px] font-medium text-[var(--color-ink-muted)] block mb-1">
                        Issue Title:
                      </label>
                      <input
                        type="text"
                        required
                        value={newLinkedTitle}
                        onChange={(e) => setNewLinkedTitle(e.target.value)}
                        placeholder="Issue title in external tracker..."
                        className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                      />
                    </div>
                    <div>
                      <label className="text-[11px] font-medium text-[var(--color-ink-muted)] block mb-1">
                        Description (Optional):
                      </label>
                      <textarea
                        rows={2}
                        value={newLinkedDescription}
                        onChange={(e) => setNewLinkedDescription(e.target.value)}
                        placeholder="Additional context or repro steps..."
                        className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                      />
                    </div>
                    <div className="flex justify-end gap-2">
                      <button
                        type="button"
                        onClick={() => setShowCreateLinkedIssue(false)}
                        className="rounded-md px-2.5 py-1 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-slate-200"
                      >
                        Cancel
                      </button>
                      <button
                        type="submit"
                        disabled={creatingLinkedIssue || !newLinkedTitle.trim()}
                        className="inline-flex items-center gap-1 rounded-md bg-[var(--color-accent)] px-3 py-1 text-xs font-semibold text-white shadow-sm hover:bg-[var(--color-accent-hover)] disabled:opacity-50 transition-colors"
                      >
                        {creatingLinkedIssue ? 'Creating & Linking...' : 'Create in Tracker'}
                      </button>
                    </div>
                  </form>
                )}

                {/* Linked Issues List */}
                {loadingLinkedIssues ? (
                  <div className="py-2 text-center text-xs text-[var(--color-ink-muted)]">Loading linked issues...</div>
                ) : linkedIssues.length === 0 ? (
                  <div className="rounded-lg border border-dashed border-slate-200 py-3 text-center text-xs text-[var(--color-ink-faint)]">
                    {linkedIssuesError
                      ? 'The linked-issue read failed, so this list is unknown rather than empty.'
                      : 'No external issues are linked to this ticket.'}
                  </div>
                ) : (
                  <div className="space-y-2">
                    {linkedIssues.map((issue) => (
                      <div
                        key={issue.id}
                        className="flex items-center justify-between rounded-lg border border-[var(--color-border)] bg-[var(--color-page)] p-2.5 text-xs"
                      >
                        <div className="flex items-center gap-2">
                          <span
                            className={`rounded px-1.5 py-0.5 text-[10px] font-bold uppercase ${
                              issue.provider === 'linear'
                                ? 'bg-purple-100 text-purple-800'
                                : issue.provider === 'github'
                                ? 'bg-slate-200 text-slate-800'
                                : issue.provider === 'jira'
                                ? 'bg-blue-100 text-blue-800'
                                : 'bg-slate-100 text-[var(--color-ink-faint)]'
                            }`}
                          >
                            {textOrMissing(issue.provider, 'provider not stored')}
                          </span>
                          {issue.issue_url ? (
                            <a
                              href={issue.issue_url}
                              target="_blank"
                              rel="noreferrer"
                              className="font-mono font-semibold text-[var(--color-accent)] hover:underline inline-flex items-center gap-1"
                            >
                              {textOrMissing(issue.issue_key, 'no issue key')}
                              <ExternalLink className="h-3 w-3" />
                            </a>
                          ) : (
                            <span
                              className="font-mono font-semibold text-[var(--color-ink-muted)] inline-flex items-center gap-1"
                              title="The linked row stores no URL, so there is nothing to open"
                            >
                              {textOrMissing(issue.issue_key, 'no issue key')} (no url stored)
                            </span>
                          )}
                          <span className="font-medium text-[var(--color-ink)] truncate max-w-[240px]">
                            {textOrMissing(issue.title, 'no title stored')}
                          </span>
                        </div>

                        <div className="flex items-center gap-2">
                          <span
                            className={`rounded-full border px-2 py-0.5 text-[10px] font-medium ${
                              issue.status
                                ? /open|new|todo|in_progress|progress|in progress/i.test(issue.status)
                                  ? 'bg-amber-50 border-amber-200 text-amber-800'
                                  : /closed|done|complete|resolved|merged/i.test(issue.status)
                                  ? 'bg-emerald-50 border-emerald-200 text-emerald-800'
                                  : 'bg-slate-50 border-slate-200 text-slate-700'
                                : 'bg-slate-50 border-dashed border-slate-300 text-[var(--color-ink-faint)]'
                            }`}
                          >
                            {textOrMissing(issue.status, 'status not returned')}
                          </span>
                          <span
                            className="text-[10px] text-[var(--color-ink-faint)]"
                            title="Creator is the signed-in email the create request carried; NULL when it carried none"
                          >
                            {textOrMissing(issue.created_by_email, 'no creator recorded')}
                            {' · '}
                            {issue.sync_status ? `sync: ${issue.sync_status}` : 'sync status not reported'}
                          </span>
                          <button
                            type="button"
                            onClick={() => handleSyncIssue(issue.id)}
                            disabled={syncingIssueId === issue.id}
                            title="Sync status with external tracker"
                            className="rounded p-1 text-[var(--color-ink-muted)] hover:bg-slate-200 hover:text-[var(--color-ink)] transition-colors"
                          >
                            <RefreshCw className={`h-3.5 w-3.5 ${syncingIssueId === issue.id ? 'animate-spin' : ''}`} />
                          </button>
                          <button
                            type="button"
                            onClick={() => handleUnlinkIssue(issue.id)}
                            disabled={unlinkingIssueId === issue.id}
                            title="Unlink issue"
                            className="rounded p-1 text-rose-500 hover:bg-rose-100 hover:text-rose-700 transition-colors"
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {linkedIssuesError && (
                  <div role="alert" className="rounded-lg bg-rose-50 p-3 text-xs text-rose-700">
                    External issues could not be read: {linkedIssuesError}
                  </div>
                )}
              </div>

              {/* Back-Office Collaboration Feed */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <h4 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">
                    Collaboration & Internal Audit Trail (
                    {selectedTicket.activities
                      ? selectedTicket.activities.length
                      : 'not returned by the detail read'}
                    )
                  </h4>
                  <span className="text-xs text-[var(--color-ink-faint)]">Teammate & Back-Office Notes</span>
                </div>

                <div className="space-y-2 max-h-60 overflow-y-auto border border-[var(--color-border)] rounded-xl p-3 bg-[var(--color-page)]/50">
                  {selectedTicket.activities && selectedTicket.activities.length > 0 ? (
                    selectedTicket.activities.map((act) => (
                      <div
                        key={act.id}
                        className={`rounded-lg p-2.5 text-xs ${
                          act.activity_type === 'back_office_note'
                            ? 'bg-[var(--color-accent-soft)] border border-[var(--color-border)] text-indigo-950'
                            : act.activity_type === 'broadcast'
                            ? 'bg-rose-50 border border-rose-200 text-rose-950 font-medium'
                            : act.activity_type === 'external_issue'
                            ? 'bg-purple-50 border border-purple-200 text-purple-950 font-medium'
                            : act.activity_type === 'tracker_link'
                            ? 'bg-amber-50 border border-amber-200 text-amber-950 font-medium'
                            : 'bg-[var(--color-surface)] border border-[var(--color-border)] text-slate-800'
                        }`}
                      >
                        <div className="flex items-center justify-between text-[11px] text-[var(--color-ink-muted)] mb-1">
                          <div className="flex items-center gap-1.5">
                            {act.activity_type === 'broadcast' ? (
                              <Radio className="h-3 w-3 text-rose-600" />
                            ) : act.activity_type === 'external_issue' ? (
                              <Link2 className="h-3 w-3 text-purple-600" />
                            ) : act.activity_type === 'tracker_link' ? (
                              <Tag className="h-3 w-3 text-amber-600" />
                            ) : act.activity_type === 'status_change' ? (
                              <RefreshCw className="h-3 w-3 text-blue-600" />
                            ) : act.activity_type === 'priority_change' ? (
                              <AlertTriangle className="h-3 w-3 text-amber-600" />
                            ) : act.activity_type === 'creation' ? (
                              <Plus className="h-3 w-3 text-emerald-600" />
                            ) : (
                              <MessageSquare className="h-3 w-3 text-slate-500" />
                            )}
                            <span
                              className={`font-semibold ${
                                act.author_name ? 'text-slate-800' : 'text-[var(--color-ink-faint)] italic'
                              }`}
                            >
                              {textOrMissing(act.author_name, 'no stored author')}
                            </span>
                            <span className="text-[var(--color-ink-faint)]">
                              {textOrMissing(act.activity_type, 'type not stored')}
                            </span>
                          </div>
                          <span>{timeOrMissing(act.created_at)}</span>
                        </div>
                        <p className="whitespace-pre-wrap">
                          {textOrMissing(act.message, 'No message stored on this activity row.')}
                        </p>
                      </div>
                    ))
                  ) : (
                    <div className="py-4 text-center text-xs text-[var(--color-ink-faint)]">
                      {selectedTicket.activities
                        ? 'This ticket has no activity rows yet -- nothing has been recorded against it.'
                        : 'The detail read returned no activity list, so the audit trail is unknown rather than empty.'}
                    </div>
                  )}
                </div>

                {/* Post New Note Composer */}
                <form onSubmit={handleAddNote} className="space-y-2 pt-2">
                  <div className="flex flex-wrap items-center gap-2">
                    <input
                      type="text"
                      value={newNoteAuthor}
                      onChange={(e) => setNewNoteAuthor(e.target.value)}
                      placeholder="Optional override; blank records the signed-in email"
                      className="w-64 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                    />
                    <span className="text-xs text-[var(--color-ink-faint)]">
                      Adding to{' '}
                      {selectedTicket.department ? selectedTicket.department.toUpperCase() : 'the unassigned'}
                      {' '}collaboration thread as:{' '}
                      <span className="font-medium text-[var(--color-ink)]">
                        {newNoteAuthor.trim() || actorName || 'no author (stored as none)'}
                      </span>
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={newNoteMessage}
                      onChange={(e) => setNewNoteMessage(e.target.value)}
                      placeholder="Write an internal note or update for teammates..."
                      className="flex-1 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                    />
                    <button
                      type="submit"
                      disabled={!newNoteMessage.trim() || submittingNote}
                      className="inline-flex items-center gap-1 rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-[var(--color-accent-hover)] disabled:opacity-50 transition-colors"
                    >
                      <Send className="h-3 w-3" />
                      {submittingNote ? 'Posting...' : 'Post Note'}
                    </button>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Broadcast & Resolve All Modal */}
      {showBroadcastModal && broadcastTargetTracker && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl bg-[var(--color-surface)] p-6 shadow-2xl space-y-4">
            <div>
              <h3 className="text-lg font-bold text-[var(--color-ink)]">
                Widespread Broadcast & Resolution
              </h3>
              <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
                Runs over every ticket linked to{' '}
                <span className="font-mono font-semibold text-[var(--color-ink)]">
                  {textOrMissing(broadcastTargetTracker.ticket_number, 'this tracker (no number stored)')}
                </span>{' '}
                ({textOrMissing(broadcastTargetTracker.title, 'no title stored')}). Each customer message is handed to
                that conversation's provider: only a provider 2xx counts as sent, refusals and recipients with no
                conversation or no addressable contact are reported after dispatch, per recipient.
              </p>
              <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
                Recorded as authored by:{' '}
                <span className="font-medium text-[var(--color-ink)]">
                  {actorName || 'no signed-in email in this session — the activity rows will store no author'}
                </span>
              </p>
            </div>

            {typeof broadcastTargetTracker.child_count === 'number' && broadcastTargetTracker.child_count === 0 && (
              <div className="rounded-lg border border-amber-200 bg-amber-50 p-2.5 text-xs text-amber-900">
                This tracker reports 0 linked tickets right now, so a broadcast would message nobody and resolve
                nothing. Link the impacted customer tickets first.
              </div>
            )}

            <div className="space-y-2">
              <label className="text-xs font-semibold text-[var(--color-ink)]">
                Customer Broadcast Message:
              </label>
              <textarea
                rows={3}
                value={broadcastMessage}
                onChange={(e) => setBroadcastMessage(e.target.value)}
                placeholder="e.g. The payment gateway outage has been resolved. All pending transactions are now processing normally."
                className="w-full rounded-lg border border-slate-300 p-2.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
              />
            </div>

            <div className="space-y-2 rounded-xl bg-[var(--color-page)] p-3 border border-[var(--color-border)]">
              <label className="flex items-center gap-2 text-xs text-[var(--color-ink)] cursor-pointer">
                <input
                  type="checkbox"
                  checked={broadcastResolveAll}
                  onChange={(e) => setBroadcastResolveAll(e.target.checked)}
                  className="rounded text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <span className="font-medium">
                  {typeof broadcastTargetTracker.child_count === 'number'
                    ? `Auto-resolve all ${broadcastTargetTracker.child_count} linked customer tickets`
                    : 'Auto-resolve every linked customer ticket (this tracker returned no linked count)'}
                </span>
              </label>
              <label className="flex items-center gap-2 text-xs text-[var(--color-ink)] cursor-pointer">
                <input
                  type="checkbox"
                  checked={broadcastSendCustomer}
                  onChange={(e) => setBroadcastSendCustomer(e.target.checked)}
                  className="rounded text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <span className="font-medium">
                  Ask each linked conversation's provider to deliver this message to the customer
                </span>
              </label>
            </div>

            {broadcastFailure && (
              <div role="alert" className="rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-800">
                <span className="font-semibold block">The broadcast did not complete:</span>
                {broadcastFailure}
              </div>
            )}

            <div className="flex items-center justify-end gap-2 pt-2">
              <button
                onClick={closeBroadcastModal}
                className="rounded-lg px-3 py-1.5 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-slate-100 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleExecuteBroadcast}
                disabled={!broadcastMessage.trim() || submittingBroadcast}
                className="inline-flex items-center gap-1.5 rounded-lg bg-rose-600 px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-rose-700 disabled:opacity-50 transition-colors"
              >
                <Radio className="h-3.5 w-3.5" />
                {submittingBroadcast ? 'Dispatching...' : 'Dispatch Broadcast'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Create Ticket Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl bg-[var(--color-surface)] p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
              <h3 className="text-lg font-bold text-[var(--color-ink)]">Create New Ticket</h3>
              <button
                onClick={() => setShowCreateModal(false)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] p-1 rounded-lg hover:bg-slate-100"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            <form onSubmit={handleCreateTicket} className="space-y-4">
              <p className="text-xs text-[var(--color-ink-faint)]">
                The creation activity row will be authored by:{' '}
                <span className="font-medium text-[var(--color-ink)]">
                  {actorName || 'no signed-in email in this session — stored with no author'}
                </span>
              </p>
              {/* Type Selection */}
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Ticket Type:
                </label>
                <div className="grid grid-cols-3 gap-2">
                  <button
                    type="button"
                    onClick={() => setNewType('customer_issue')}
                    className={`rounded-lg p-2 text-center text-xs font-medium border transition-colors ${
                      newType === 'customer_issue'
                        ? 'border-emerald-600 bg-emerald-50 text-emerald-800 font-bold'
                        : 'border-[var(--color-border)] text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]'
                    }`}
                  >
                    Customer Issue
                  </button>
                  <button
                    type="button"
                    onClick={() => setNewType('back_office')}
                    className={`rounded-lg p-2 text-center text-xs font-medium border transition-colors ${
                      newType === 'back_office'
                        ? 'border-indigo-600 bg-[var(--color-accent-soft)] text-indigo-800 font-bold'
                        : 'border-[var(--color-border)] text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]'
                    }`}
                  >
                    Back-Office Task
                  </button>
                  <button
                    type="button"
                    onClick={() => setNewType('tracker')}
                    className={`rounded-lg p-2 text-center text-xs font-medium border transition-colors ${
                      newType === 'tracker'
                        ? 'border-rose-600 bg-rose-50 text-rose-800 font-bold'
                        : 'border-[var(--color-border)] text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]'
                    }`}
                  >
                    Tracker (Widespread)
                  </button>
                </div>
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Ticket Title:
                </label>
                <input
                  type="text"
                  required
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  placeholder="Summary of the issue or task..."
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Description:
                </label>
                <textarea
                  required
                  rows={3}
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                  placeholder="Detailed breakdown of the issue or instructions for teammates..."
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                    Department:
                  </label>
                  <select
                    value={newDept}
                    onChange={(e) => setNewDept(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 bg-[var(--color-surface)] p-2 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                  >
                    {departmentOptions.map((dept) => (
                      <option key={dept} value={dept}>
                        {dept.charAt(0).toUpperCase() + dept.slice(1)}
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                    Priority:
                  </label>
                  <select
                    value={newPriority}
                    onChange={(e) => setNewPriority(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 bg-[var(--color-surface)] p-2 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                  >
                    <option value="low">Low</option>
                    <option value="medium">Medium</option>
                    <option value="high">High</option>
                    <option value="urgent">Urgent</option>
                  </select>
                </div>
              </div>

              {newType !== 'tracker' && trackers.length > 0 && (
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                    Link to Tracker Incident (Optional):
                  </label>
                  <select
                    value={newTrackerId}
                    onChange={(e) => setNewTrackerId(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 bg-[var(--color-surface)] p-2 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                  >
                    <option value="">None (Standalone ticket)</option>
                    {trackers.map((tr) => (
                      <option key={tr.id} value={tr.id}>
                        {tr.ticket_number}: {tr.title}
                      </option>
                    ))}
                  </select>
                </div>
              )}

              <div className="flex items-center justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="rounded-lg px-3 py-1.5 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-slate-100 transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creatingTicket}
                  className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-[var(--color-accent-hover)] disabled:opacity-50 transition-colors"
                >
                  <Plus className="h-3.5 w-3.5" />
                  {creatingTicket ? 'Creating...' : 'Create Ticket'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
