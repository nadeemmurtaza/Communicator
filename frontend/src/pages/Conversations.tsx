import { useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { MessageCircle, Phone, Mail, MessageSquare, RefreshCw } from 'lucide-react'
import { api, type Conversation } from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

const STATES = [
  'all',
  'new',
  'awaiting_reply',
  'engaged',
  'hot_lead',
  'handed_off',
  'resolved',
  'closed_won',
  'closed_lost',
  'opted_out',
]

function renderChannelBadge(channel?: string) {
  const ch = (channel || 'whatsapp').toLowerCase()
  if (ch === 'whatsapp') {
    return (
      <span className="inline-flex items-center gap-1 rounded-full border border-emerald-200 bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-800">
        <MessageCircle className="h-3 w-3 text-emerald-600" />
        WhatsApp
      </span>
    )
  }
  if (ch === 'sms') {
    return (
      <span className="inline-flex items-center gap-1 rounded-full border border-blue-200 bg-blue-50 px-2 py-0.5 text-xs font-medium text-blue-800">
        <Phone className="h-3 w-3 text-blue-600" />
        SMS
      </span>
    )
  }
  if (ch === 'email') {
    return (
      <span className="inline-flex items-center gap-1 rounded-full border border-indigo-200 bg-indigo-50 px-2 py-0.5 text-xs font-medium text-indigo-800">
        <Mail className="h-3 w-3 text-indigo-600" />
        Email
      </span>
    )
  }
  return (
    <span className="inline-flex items-center gap-1 rounded-full border border-slate-200 bg-slate-50 px-2 py-0.5 text-xs font-medium text-slate-700 capitalize">
      <MessageSquare className="h-3 w-3 text-slate-500" />
      {channel || 'Chat'}
    </span>
  )
}

export function Conversations() {
  const [conversations, setConversations] = useState<Conversation[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [refreshing, setRefreshing] = useState(false)
  const [filter, setFilter] = useState('all')

  useEffect(() => {
    loadConversations()
  }, [])

  async function loadConversations() {
    try {
      setError(null)
      const data = await api.listConversations()
      setConversations(data)
    } catch (e: any) {
      setError(e.message || 'Failed to load conversations')
    }
  }

  async function handleRefresh() {
    try {
      setRefreshing(true)
      await loadConversations()
    } finally {
      setRefreshing(false)
    }
  }

  const filtered = useMemo(
    () =>
      (conversations ?? []).filter((c) => {
        if (filter === 'all') return true
        const cState = (c.state || '').toLowerCase()
        return cState === filter.toLowerCase()
      }),
    [conversations, filter],
  )

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Conversations</h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">Every lead thread, across every channel.</p>
        </div>
        <button
          onClick={handleRefresh}
          disabled={refreshing}
          className="self-start sm:self-auto rounded-md border border-slate-300 bg-white px-3.5 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50 shadow-xs flex items-center gap-1.5 transition-colors cursor-pointer"
        >
          <RefreshCw className={`h-3.5 w-3.5 text-emerald-600 ${refreshing ? 'animate-spin' : ''}`} />
          {refreshing ? 'Refreshing...' : 'Refresh & Sync'}
        </button>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      <div className="flex flex-wrap gap-1.5">
        {STATES.map((s) => (
          <button
            key={s}
            onClick={() => setFilter(s)}
            className={`rounded-full px-3 py-1 text-xs font-medium cursor-pointer transition-colors ${
              filter === s
                ? 'bg-[var(--color-accent)] text-white shadow-xs'
                : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
            }`}
          >
            {s === 'all' ? 'All' : stateLabel(s)}
          </button>
        ))}
      </div>

      {conversations === null ? (
        <p className="text-sm text-[var(--color-ink-muted)]">Loading conversations...</p>
      ) : filtered.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-[var(--color-surface)] px-4 py-10 text-center text-sm text-[var(--color-ink-muted)]">
          No conversations in this state.
        </div>
      ) : (
        <div className="overflow-x-auto rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs">
          <table className="w-full text-sm">
            <thead className="border-b border-[var(--color-border)] bg-[var(--color-page)] text-left text-xs font-medium uppercase tracking-wide text-[var(--color-ink-muted)]">
              <tr>
                <th className="px-4 py-2.5">Phone Number</th>
                <th className="px-4 py-2.5">Customer / Lead Name</th>
                <th className="px-4 py-2.5">Channel</th>
                <th className="px-4 py-2.5">State</th>
                <th className="px-4 py-2.5">Updated</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {filtered.map((c) => (
                <tr key={c.id} className="hover:bg-[var(--color-page)] transition-colors">
                  <td className="px-4 py-3 font-mono text-xs font-semibold text-slate-800">
                    {c.contactPhone || c.phone || '—'}
                  </td>
                  <td className="px-4 py-3">
                    <Link
                      to={`/conversations/${c.id}`}
                      className="font-semibold text-slate-900 hover:text-emerald-700 hover:underline"
                    >
                      {c.contactName || c.customerName || c.leadName || c.contactPhone || 'Customer'}
                    </Link>
                  </td>
                  <td className="px-4 py-3">
                    {renderChannelBadge(c.channel)}
                  </td>
                  <td className="px-4 py-3">
                    <span className={`badge ${stateBadgeClass(c.state)}`}>{stateLabel(c.state)}</span>
                  </td>
                  <td className="px-4 py-3 text-xs text-[var(--color-ink-muted)] whitespace-nowrap">
                    {timeAgo(c.updatedAt)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
