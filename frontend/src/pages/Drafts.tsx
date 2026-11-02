import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, type Draft } from '../lib/api'
import { timeAgo } from '../lib/format'

export function Drafts() {
  const [drafts, setDrafts] = useState<Draft[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busyId, setBusyId] = useState<string | null>(null)

  function load() {
    api
      .listDrafts()
      .then(setDrafts)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  async function act(id: string, action: 'approve' | 'reject') {
    setBusyId(id)
    try {
      if (action === 'approve') await api.approveDraft(id)
      else await api.rejectDraft(id)
      setDrafts((prev) => prev?.filter((d) => d.id !== id) ?? null)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setBusyId(null)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--color-ink)]">Draft review</h1>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
          AI-drafted replies waiting for your approval before they go out.
        </p>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {drafts === null ? (
        <p className="text-sm text-[var(--color-ink-muted)]">Loading…</p>
      ) : drafts.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-[var(--color-surface)] px-4 py-10 text-center text-sm text-[var(--color-ink-muted)]">
          Nothing pending. New drafts land here as leads reply.
        </div>
      ) : (
        <ul className="space-y-3">
          {drafts.map((d) => (
            <li key={d.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <Link
                    to={`/conversations/${d.conversationId}`}
                    className="text-xs font-medium text-[var(--color-ink-muted)] hover:underline"
                  >
                    Conversation {d.conversationId.slice(0, 8)}
                  </Link>
                  <p className="mt-1 whitespace-pre-wrap text-sm text-slate-800">{d.body}</p>
                  <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-[var(--color-ink-faint)]">
                    <span>{Math.round(d.confidence * 100)}% confidence</span>
                    <span>·</span>
                    <span>{timeAgo(d.createdAt)}</span>
                    {d.citations.length > 0 && (
                      <>
                        <span>·</span>
                        <span>cites {d.citations.join(', ')}</span>
                      </>
                    )}
                  </div>
                </div>
                <div className="flex shrink-0 gap-2">
                  <button
                    disabled={busyId === d.id}
                    onClick={() => act(d.id, 'reject')}
                    className="rounded-md border border-slate-300 px-3 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50"
                  >
                    Reject
                  </button>
                  <button
                    disabled={busyId === d.id}
                    onClick={() => act(d.id, 'approve')}
                    className="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
                  >
                    Approve &amp; send
                  </button>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
