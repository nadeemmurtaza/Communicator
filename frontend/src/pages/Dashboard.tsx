import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, type Conversation, type Draft } from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

export function Dashboard() {
  const [drafts, setDrafts] = useState<Draft[] | null>(null)
  const [conversations, setConversations] = useState<Conversation[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([api.listDrafts(), api.listConversations()])
      .then(([d, c]) => {
        setDrafts(d)
        setConversations(c)
      })
      .catch((e) => setError(e.message))
  }, [])

  const hotLeads = conversations?.filter((c) => c.state === 'hot_lead') ?? []

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-xl font-semibold text-slate-900">Dashboard</h1>
        <p className="mt-1 text-sm text-slate-500">What needs your attention right now.</p>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          Couldn't reach the API: {error}
        </div>
      )}

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <StatCard
          label="Pending drafts"
          value={drafts?.length ?? '—'}
          to="/drafts"
          tone={drafts && drafts.length > 0 ? 'warn' : 'default'}
        />
        <StatCard
          label="Hot leads"
          value={hotLeads.length}
          to="/conversations"
          tone={hotLeads.length > 0 ? 'danger' : 'default'}
        />
        <StatCard label="Open conversations" value={conversations?.length ?? '—'} to="/conversations" />
      </div>

      <section>
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-sm font-semibold text-slate-700">Drafts awaiting review</h2>
          <Link to="/drafts" className="text-sm font-medium text-slate-900 hover:underline">
            Review all
          </Link>
        </div>
        {drafts === null ? (
          <SkeletonList />
        ) : drafts.length === 0 ? (
          <EmptyState text="Nothing waiting on you. Drafts will show up here as leads reply." />
        ) : (
          <ul className="divide-y divide-slate-200 rounded-lg border border-slate-200 bg-white">
            {drafts.slice(0, 5).map((d) => (
              <li key={d.id} className="px-4 py-3">
                <p className="line-clamp-2 text-sm text-slate-800">{d.body}</p>
                <p className="mt-1 text-xs text-slate-400">
                  {Math.round(d.confidence * 100)}% confidence · {timeAgo(d.createdAt)}
                </p>
              </li>
            ))}
          </ul>
        )}
      </section>

      {hotLeads.length > 0 && (
        <section>
          <h2 className="mb-3 text-sm font-semibold text-slate-700">Hot leads</h2>
          <ul className="divide-y divide-slate-200 rounded-lg border border-slate-200 bg-white">
            {hotLeads.map((c) => (
              <li key={c.id} className="flex items-center justify-between px-4 py-3">
                <Link to={`/conversations/${c.id}`} className="text-sm font-medium text-slate-900 hover:underline">
                  Conversation {c.id.slice(0, 8)}
                </Link>
                <span className={`badge ${stateBadgeClass(c.state)}`}>{stateLabel(c.state)}</span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  )
}

function StatCard({
  label,
  value,
  to,
  tone = 'default',
}: {
  label: string
  value: number | string
  to: string
  tone?: 'default' | 'warn' | 'danger'
}) {
  const toneClass =
    tone === 'danger'
      ? 'border-red-200 bg-red-50'
      : tone === 'warn'
        ? 'border-amber-200 bg-amber-50'
        : 'border-slate-200 bg-white'
  return (
    <Link to={to} className={`block rounded-lg border p-4 transition hover:shadow-sm ${toneClass}`}>
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</p>
      <p className="mt-2 text-2xl font-semibold text-slate-900">{value}</p>
    </Link>
  )
}

function EmptyState({ text }: { text: string }) {
  return (
    <div className="rounded-lg border border-dashed border-slate-300 bg-white px-4 py-8 text-center text-sm text-slate-500">
      {text}
    </div>
  )
}

function SkeletonList() {
  return (
    <div className="space-y-2">
      {[0, 1, 2].map((i) => (
        <div key={i} className="h-14 animate-pulse rounded-lg border border-slate-200 bg-slate-100" />
      ))}
    </div>
  )
}
