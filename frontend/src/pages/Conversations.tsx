import { useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, type Conversation } from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

const STATES = [
  'all',
  'new',
  'awaiting_reply',
  'engaged',
  'hot_lead',
  'handed_off',
  'opted_out',
  'closed_won',
  'closed_lost',
]

export function Conversations() {
  const [conversations, setConversations] = useState<Conversation[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [filter, setFilter] = useState('all')

  useEffect(() => {
    api
      .listConversations()
      .then(setConversations)
      .catch((e) => setError(e.message))
  }, [])

  const filtered = useMemo(
    () => (conversations ?? []).filter((c) => filter === 'all' || c.state === filter),
    [conversations, filter],
  )

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--color-ink)]">Conversations</h1>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">Every lead thread, across every channel.</p>
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
            className={`rounded-full px-3 py-1 text-xs font-medium ${
              filter === s ? 'bg-[var(--color-accent)] text-white' : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
            }`}
          >
            {s === 'all' ? 'All' : stateLabel(s)}
          </button>
        ))}
      </div>

      {conversations === null ? (
        <p className="text-sm text-[var(--color-ink-muted)]">Loading…</p>
      ) : filtered.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-[var(--color-surface)] px-4 py-10 text-center text-sm text-[var(--color-ink-muted)]">
          No conversations in this state.
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
          <table className="w-full text-sm">
            <thead className="border-b border-[var(--color-border)] bg-[var(--color-page)] text-left text-xs font-medium uppercase tracking-wide text-[var(--color-ink-muted)]">
              <tr>
                <th className="px-4 py-2">Conversation</th>
                <th className="px-4 py-2">State</th>
                <th className="hidden px-4 py-2 sm:table-cell">Updated</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {filtered.map((c) => (
                <tr key={c.id} className="hover:bg-[var(--color-page)]">
                  <td className="px-4 py-3">
                    <Link to={`/conversations/${c.id}`} className="font-medium text-[var(--color-ink)] hover:underline">
                      {c.id.slice(0, 8)}
                    </Link>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`badge ${stateBadgeClass(c.state)}`}>{stateLabel(c.state)}</span>
                  </td>
                  <td className="hidden px-4 py-3 text-[var(--color-ink-muted)] sm:table-cell">{timeAgo(c.updatedAt)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
