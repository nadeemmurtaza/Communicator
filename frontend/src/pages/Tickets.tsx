import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type Ticket,
  type TicketStats,
} from '../lib/api'

export function Tickets() {
  const [tickets, setTickets] = useState<Ticket[]>([])
  const [stats, setStats] = useState<TicketStats | null>(null)
  const [trackers, setTrackers] = useState<Ticket[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filters
  const [activeTab, setActiveTab] = useState<'all' | 'tracker' | 'customer_issue' | 'back_office' | 'resolved'>('all')
  const [selectedDept, setSelectedDept] = useState<string>('all')
  const [searchQuery, setSearchQuery] = useState('')

  // Selected Ticket for Detail Drawer/Modal
  const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null)

  // Modals
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showBroadcastModal, setShowBroadcastModal] = useState(false)
  const [broadcastTargetTracker, setBroadcastTargetTracker] = useState<Ticket | null>(null)
  const [broadcastMessage, setBroadcastMessage] = useState('')
  const [broadcastResolveAll, setBroadcastResolveAll] = useState(true)
  const [broadcastSendCustomer, setBroadcastSendCustomer] = useState(true)
  const [submittingBroadcast, setSubmittingBroadcast] = useState(false)

  // New Note
  const [newNoteMessage, setNewNoteMessage] = useState('')
  const [newNoteAuthor, setNewNoteAuthor] = useState('Agent')
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

  const openTicketDetail = async (ticketId: string) => {
    try {
      const detail = await api.getTicket(ticketId)
      setSelectedTicket(detail)
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to load ticket details')
    }
  }

  const handleUpdateStatus = async (ticketId: string, newStatus: string) => {
    try {
      const updated = await api.updateTicket(ticketId, { status: newStatus, author_name: 'Agent' })
      setSelectedTicket(updated)
      // Refresh list
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to update ticket status')
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
        author_name: newNoteAuthor || 'Agent',
      })
      setNewNoteMessage('')
      // Refresh detail
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
      const updated = await api.linkTicketToTracker(selectedTicket.id, linkTrackerId, 'Agent')
      setSelectedTicket(updated)
      setLinkTrackerId('')
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to link ticket to tracker')
    } finally {
      setLinkingTracker(false)
    }
  }

  const handleExecuteBroadcast = async () => {
    if (!broadcastTargetTracker || !broadcastMessage.trim()) return
    setSubmittingBroadcast(true)
    try {
      const res = await api.broadcastTrackerUpdate(broadcastTargetTracker.id, {
        broadcast_message: broadcastMessage.trim(),
        resolve_all_children: broadcastResolveAll,
        send_customer_message: broadcastSendCustomer,
        author_name: 'Incident Response Manager',
      })
      alert(
        `Broadcast Dispatched!\n• Impacted Tickets: ${res.impacted_tickets}\n• Customer Messages Sent: ${res.messages_sent}\n• Auto-Resolved: ${res.resolved_tickets}`
      )
      setShowBroadcastModal(false)
      setBroadcastMessage('')
      if (selectedTicket?.id === broadcastTargetTracker.id) {
        openTicketDetail(broadcastTargetTracker.id)
      }
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to broadcast update')
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
        author_name: 'Agent',
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

  // Filtered List
  const filteredTickets = tickets.filter((t) => {
    // Tab filter
    if (activeTab === 'tracker' && t.ticket_type !== 'tracker') return false
    if (activeTab === 'customer_issue' && t.ticket_type !== 'customer_issue') return false
    if (activeTab === 'back_office' && t.ticket_type !== 'back_office') return false
    if (activeTab === 'resolved' && t.status !== 'resolved' && t.status !== 'closed') return false

    // Department filter
    if (selectedDept !== 'all' && t.department !== selectedDept) return false

    // Search query
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase()
      const matchNumber = t.ticket_number.toLowerCase().includes(q)
      const matchTitle = t.title.toLowerCase().includes(q)
      const matchDesc = t.description.toLowerCase().includes(q)
      const matchContact = t.contact_name?.toLowerCase().includes(q) || false
      if (!matchNumber && !matchTitle && !matchDesc && !matchContact) return false
    }

    return true
  })

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
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center justify-center rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-[var(--color-accent-hover)]"
          >
            + Create Ticket
          </button>
        </div>
      </div>

      {/* KPI Stats Cards */}
      {stats && (
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
            <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
              Active Tickets
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-[var(--color-ink)]">{stats.open_tickets}</span>
              <span className="text-xs text-[var(--color-ink-faint)]">of {stats.total_tickets} total</span>
            </div>
            <div className="mt-2 text-xs text-[var(--color-ink-muted)]">Across Support & Back-Office</div>
          </div>

          <div className="rounded-xl border border-rose-200 bg-rose-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-rose-700 uppercase tracking-wider">
              Tracker Incidents
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-rose-900">{stats.tracker_tickets}</span>
              <span className="rounded-full bg-rose-200 px-2 py-0.5 text-xs font-semibold text-rose-800">
                Widespread
              </span>
            </div>
            <div className="mt-2 text-xs text-rose-600">
              {stats.impacted_customers_count} customers linked
            </div>
          </div>

          <div className="rounded-xl border border-violet-200 bg-violet-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-violet-700 uppercase tracking-wider">
              Back-Office Load
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-violet-900">{stats.back_office_tickets}</span>
              <span className="text-xs text-violet-600">tasks active</span>
            </div>
            <div className="mt-2 text-xs text-violet-600">
              Finance, Logistics, Engineering, Legal
            </div>
          </div>

          <div className="rounded-xl border border-emerald-200 bg-emerald-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-emerald-700 uppercase tracking-wider">
              Customer Issues
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-emerald-900">{stats.customer_issues}</span>
              <span className="text-xs text-emerald-600">reported</span>
            </div>
            <div className="mt-2 text-xs text-emerald-600">
              {stats.resolved_tickets} resolved to date
            </div>
          </div>
        </div>
      )}

      {/* Primary Filters & Tabs */}
      <div className="flex flex-col gap-4 border-b border-[var(--color-border)] pb-4 md:flex-row md:items-center md:justify-between">
        <div className="flex flex-wrap items-center gap-1">
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
            All Tickets ({tickets.length})
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
            <span className="h-2 w-2 rounded-full bg-rose-500 animate-pulse"></span>
            Tracker Tickets ({stats?.tracker_tickets ?? 0})
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
            Customer Issues ({stats?.customer_issues ?? 0})
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
            Back-Office Collaboration ({stats?.back_office_tickets ?? 0})
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
            Resolved ({stats?.resolved_tickets ?? 0})
          </button>
        </div>

        <div className="flex items-center gap-2">
          {/* Department Dropdown */}
          <select
            value={selectedDept}
            onChange={(e) => setSelectedDept(e.target.value)}
            className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] shadow-sm focus:border-indigo-500 focus:outline-none"
          >
            <option value="all">All Departments</option>
            <option value="support">Support</option>
            <option value="finance">Finance</option>
            <option value="logistics">Logistics</option>
            <option value="engineering">Engineering</option>
            <option value="warehouse">Warehouse</option>
            <option value="legal">Legal</option>
          </select>

          {/* Search Input */}
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search tickets, IDs, contacts..."
            className="w-48 sm:w-64 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs placeholder:text-[var(--color-ink-faint)] focus:border-indigo-500 focus:outline-none"
          />
        </div>
      </div>

      {/* Ticket List */}
      {loading ? (
        <div className="py-12 text-center text-sm text-[var(--color-ink-muted)]">Loading tickets...</div>
      ) : error ? (
        <div className="rounded-lg bg-rose-50 p-4 text-sm text-rose-700">{error}</div>
      ) : filteredTickets.length === 0 ? (
        <div className="rounded-xl border border-dashed border-slate-300 p-12 text-center">
          <p className="text-sm text-[var(--color-ink-muted)]">No tickets found matching current filters.</p>
          <button
            onClick={() => setShowCreateModal(true)}
            className="mt-3 inline-flex items-center text-xs font-semibold text-[var(--color-accent)] hover:text-indigo-800"
          >
            + Create a ticket now
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
                    {t.ticket_number}
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
                  ) : (
                    <span className="rounded-md bg-emerald-100 px-2 py-0.5 text-xs font-semibold text-emerald-800">
                      Customer Issue
                    </span>
                  )}

                  {/* Department Badge */}
                  <span className="rounded-md bg-slate-100 px-2 py-0.5 text-xs font-medium text-[var(--color-ink-muted)] uppercase">
                    {t.department}
                  </span>

                  {/* Priority Badge */}
                  <span
                    className={`rounded-md px-2 py-0.5 text-xs font-medium ${
                      t.priority === 'urgent'
                        ? 'bg-red-100 text-red-800 font-bold'
                        : t.priority === 'high'
                        ? 'bg-amber-100 text-amber-800'
                        : 'bg-slate-100 text-[var(--color-ink)]'
                    }`}
                  >
                    {t.priority.toUpperCase()}
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
                        : 'bg-amber-100 text-amber-800'
                    }`}
                  >
                    {t.status.replace(/_/g, ' ')}
                  </span>
                  <span className="text-xs text-[var(--color-ink-faint)]">
                    {new Date(t.created_at).toLocaleDateString([], {
                      month: 'short',
                      day: 'numeric',
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </span>
                </div>
              </div>

              {/* Title & Description */}
              <div className="mt-2">
                <h3 className="text-base font-semibold text-[var(--color-ink)]">{t.title}</h3>
                <p className="mt-1 line-clamp-2 text-xs text-[var(--color-ink-muted)]">{t.description}</p>
              </div>

              {/* Tracker / Linked Metadata Footer */}
              <div className="mt-3 flex flex-wrap items-center gap-3 border-t border-[var(--color-border)] pt-3 text-xs text-[var(--color-ink-muted)]">
                {t.ticket_type === 'tracker' && (
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-rose-700">
                      {t.child_count ?? 0} linked issues
                    </span>
                    <span className="text-slate-300">•</span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        setBroadcastTargetTracker(t)
                        setShowBroadcastModal(true)
                      }}
                      className="rounded bg-rose-600 px-2 py-0.5 text-xs font-semibold text-white hover:bg-rose-700"
                    >
                      Broadcast & Resolve
                    </button>
                  </div>
                )}

                {t.tracker_id && (
                  <div className="flex items-center gap-1 text-rose-700">
                    <span className="font-medium">Linked to Tracker:</span>
                    <span className="font-mono font-semibold">{t.tracker_number}</span>
                    <span className="truncate max-w-[200px]">({t.tracker_title})</span>
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
                    className="text-[var(--color-accent)] hover:text-indigo-800 font-medium"
                  >
                    View Conversation →
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
                  {selectedTicket.ticket_number}
                </span>
                <span
                  className={`rounded px-2 py-0.5 text-xs font-semibold ${
                    selectedTicket.ticket_type === 'tracker'
                      ? 'bg-rose-100 text-rose-800'
                      : selectedTicket.ticket_type === 'back_office'
                      ? 'bg-indigo-100 text-indigo-800'
                      : 'bg-emerald-100 text-emerald-800'
                  }`}
                >
                  {selectedTicket.ticket_type.replace(/_/g, ' ').toUpperCase()}
                </span>
                <span className="rounded bg-slate-200 px-2 py-0.5 text-xs font-semibold text-[var(--color-ink)] uppercase">
                  {selectedTicket.department}
                </span>
              </div>
              <button
                onClick={() => setSelectedTicket(null)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] text-lg font-bold"
              >
                ✕
              </button>
            </div>

            {/* Modal Body */}
            <div className="flex-1 overflow-y-auto p-6 space-y-6">
              {/* Title & Status Controls */}
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <h2 className="text-xl font-bold text-[var(--color-ink)]">{selectedTicket.title}</h2>
                  <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
                    Created on {new Date(selectedTicket.created_at).toLocaleString()}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)]">Status:</label>
                  <select
                    value={selectedTicket.status}
                    onChange={(e) => handleUpdateStatus(selectedTicket.id, e.target.value)}
                    className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium text-slate-800 shadow-sm focus:border-indigo-500 focus:outline-none"
                  >
                    <option value="open">Open</option>
                    <option value="in_progress">In Progress</option>
                    <option value="waiting_on_back_office">Waiting on Back-Office</option>
                    <option value="waiting_on_customer">Waiting on Customer</option>
                    <option value="resolved">Resolved</option>
                    <option value="closed">Closed</option>
                  </select>
                </div>
              </div>

              {/* Description */}
              <div className="rounded-xl bg-[var(--color-page)] p-4 text-xs text-[var(--color-ink)] leading-relaxed border border-[var(--color-border)]">
                <span className="font-semibold block mb-1 text-[var(--color-ink)]">Description:</span>
                {selectedTicket.description}
              </div>

              {/* Resolution Summary Banner if Resolved */}
              {selectedTicket.resolution_summary && (
                <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-xs text-emerald-800">
                  <span className="font-semibold block mb-1">Resolution Summary:</span>
                  {selectedTicket.resolution_summary}
                </div>
              )}

              {/* Tracker Widespread Management Banner */}
              {selectedTicket.ticket_type === 'tracker' && (
                <div className="rounded-xl border border-rose-200 bg-rose-50/70 p-4 space-y-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="text-sm font-bold text-rose-900 flex items-center gap-2">
                        <span>Widespread Incident Tracker</span>
                        <span className="rounded-full bg-rose-200 px-2 py-0.5 text-xs font-semibold text-rose-800">
                          {selectedTicket.child_tickets?.length ?? 0} Linked Issues
                        </span>
                      </h4>
                      <p className="text-xs text-rose-700 mt-0.5">
                        Broadcast real-time updates and resolutions to all linked customer conversations in one click.
                      </p>
                    </div>
                    <button
                      onClick={() => {
                        setBroadcastTargetTracker(selectedTicket)
                        setShowBroadcastModal(true)
                      }}
                      className="rounded-lg bg-rose-600 px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-rose-700"
                    >
                      Broadcast & Resolve All
                    </button>
                  </div>

                  {/* List of child tickets */}
                  {selectedTicket.child_tickets && selectedTicket.child_tickets.length > 0 && (
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
                                {child.ticket_number}
                              </span>
                              <span className="text-slate-800">{child.title}</span>
                              {child.contact_name && (
                                <span className="text-[var(--color-ink-muted)]">({child.contact_name})</span>
                              )}
                            </div>
                            <div className="flex items-center gap-2">
                              <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] text-[var(--color-ink-muted)]">
                                {child.status}
                              </span>
                              {child.conversation_id && (
                                <Link
                                  to={`/conversations/${child.conversation_id}`}
                                  className="text-[var(--color-accent)] hover:underline text-[11px]"
                                >
                                  Chat →
                                </Link>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
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
                        <span className="font-mono">{selectedTicket.tracker_number}</span> - {selectedTicket.tracker_title}
                      </div>
                      <button
                        onClick={() => openTicketDetail(selectedTicket.tracker_id!)}
                        className="text-rose-700 font-semibold underline hover:text-rose-900"
                      >
                        Open Tracker
                      </button>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2">
                      <select
                        value={linkTrackerId}
                        onChange={(e) => setLinkTrackerId(e.target.value)}
                        className="flex-1 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-none"
                      >
                        <option value="">Select an active tracker incident to link...</option>
                        {trackers.map((tr) => (
                          <option key={tr.id} value={tr.id}>
                            {tr.ticket_number}: {tr.title} ({tr.department})
                          </option>
                        ))}
                      </select>
                      <button
                        onClick={handleLinkTracker}
                        disabled={!linkTrackerId || linkingTracker}
                        className="rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
                      >
                        {linkingTracker ? 'Linking...' : 'Link to Tracker'}
                      </button>
                    </div>
                  )}
                </div>
              )}

              {/* Back-Office Collaboration Feed */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <h4 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">
                    Collaboration & Internal Audit Trail ({selectedTicket.activities?.length ?? 0})
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
                            : 'bg-[var(--color-surface)] border border-[var(--color-border)] text-slate-800'
                        }`}
                      >
                        <div className="flex items-center justify-between text-[11px] text-[var(--color-ink-muted)] mb-1">
                          <span className="font-semibold text-slate-800">{act.author_name}</span>
                          <span>
                            {new Date(act.created_at).toLocaleTimeString([], {
                              hour: '2-digit',
                              minute: '2-digit',
                            })}
                          </span>
                        </div>
                        <p className="whitespace-pre-wrap">{act.message}</p>
                      </div>
                    ))
                  ) : (
                    <div className="py-4 text-center text-xs text-[var(--color-ink-faint)]">
                      No internal collaboration notes yet.
                    </div>
                  )}
                </div>

                {/* Post New Note Composer */}
                <form onSubmit={handleAddNote} className="space-y-2 pt-2">
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={newNoteAuthor}
                      onChange={(e) => setNewNoteAuthor(e.target.value)}
                      placeholder="Your name or role (e.g. Finance Lead)"
                      className="w-48 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs focus:border-indigo-500 focus:outline-none"
                    />
                    <span className="text-xs text-[var(--color-ink-faint)]">
                      Adding to {selectedTicket.department.toUpperCase()} collaboration thread
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={newNoteMessage}
                      onChange={(e) => setNewNoteMessage(e.target.value)}
                      placeholder="Write an internal note or update for teammates..."
                      className="flex-1 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs focus:border-indigo-500 focus:outline-none"
                    />
                    <button
                      type="submit"
                      disabled={!newNoteMessage.trim() || submittingNote}
                      className="rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
                    >
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
                Dispatches an incident update to all customer conversations linked to{' '}
                <span className="font-mono font-semibold text-[var(--color-ink)]">
                  {broadcastTargetTracker.ticket_number}
                </span>{' '}
                ({broadcastTargetTracker.title}).
              </p>
            </div>

            <div className="space-y-2">
              <label className="text-xs font-semibold text-[var(--color-ink)]">
                Customer Broadcast Message:
              </label>
              <textarea
                rows={3}
                value={broadcastMessage}
                onChange={(e) => setBroadcastMessage(e.target.value)}
                placeholder="e.g. The payment gateway outage has been resolved. All pending transactions are now processing normally."
                className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-indigo-500 focus:outline-none"
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
                  Auto-resolve all {broadcastTargetTracker.child_count ?? 0} linked customer tickets
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
                  Send message directly to impacted customers in their conversation thread
                </span>
              </label>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2">
              <button
                onClick={() => setShowBroadcastModal(false)}
                className="rounded-lg px-3 py-1.5 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-slate-100"
              >
                Cancel
              </button>
              <button
                onClick={handleExecuteBroadcast}
                disabled={!broadcastMessage.trim() || submittingBroadcast}
                className="rounded-lg bg-rose-600 px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-rose-700 disabled:opacity-50"
              >
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
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] font-bold"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleCreateTicket} className="space-y-4">
              {/* Type Selection */}
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Ticket Type:
                </label>
                <div className="grid grid-cols-3 gap-2">
                  <button
                    type="button"
                    onClick={() => setNewType('customer_issue')}
                    className={`rounded-lg p-2 text-center text-xs font-medium border ${
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
                    className={`rounded-lg p-2 text-center text-xs font-medium border ${
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
                    className={`rounded-lg p-2 text-center text-xs font-medium border ${
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
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
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
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
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
                    <option value="support">Support</option>
                    <option value="finance">Finance</option>
                    <option value="logistics">Logistics</option>
                    <option value="engineering">Engineering</option>
                    <option value="warehouse">Warehouse</option>
                    <option value="legal">Legal</option>
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
                  className="rounded-lg px-3 py-1.5 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-slate-100"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creatingTicket}
                  className="rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
                >
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
