import { useEffect, useState } from 'react'
import { api, type Broadcast, type BroadcastRecipient } from '../lib/api'
import { timeAgo } from '../lib/format'

export function Broadcasts() {
  const [broadcasts, setBroadcasts] = useState<Broadcast[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [templateName, setTemplateName] = useState('')
  const [tagName, setTagName] = useState('')
  const [saving, setSaving] = useState(false)
  const [dispatchingId, setDispatchingId] = useState<string | null>(null)
  const [openId, setOpenId] = useState<string | null>(null)
  const [recipients, setRecipients] = useState<BroadcastRecipient[] | null>(null)

  function load() {
    api
      .listBroadcasts()
      .then(setBroadcasts)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      await api.createBroadcast({
        name,
        template_name: templateName,
        template_language: 'en_US',
        tag_name: tagName,
      })
      setName('')
      setTemplateName('')
      setTagName('')
      setShowForm(false)
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function dispatch(id: string) {
    setDispatchingId(id)
    try {
      await api.dispatchBroadcast(id)
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setDispatchingId(null)
    }
  }

  async function toggleRecipients(id: string) {
    if (openId === id) {
      setOpenId(null)
      return
    }
    setOpenId(id)
    setRecipients(null)
    try {
      setRecipients(await api.listBroadcastRecipients(id))
    } catch (e) {
      setError((e as Error).message)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-slate-900">Broadcasts</h1>
          <p className="mt-1 text-sm text-slate-500">
            Send an approved template message to every contact with a tag — the way to re-engage
            outside the 24h session window.
          </p>
        </div>
        <button
          onClick={() => setShowForm((v) => !v)}
          className="shrink-0 rounded-md bg-slate-900 px-3 py-2 text-sm font-medium text-white hover:bg-slate-800"
        >
          {showForm ? 'Cancel' : 'New broadcast'}
        </button>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {showForm && (
        <form onSubmit={submit} className="space-y-3 rounded-lg border border-slate-200 bg-white p-4">
          <div>
            <label className="block text-xs font-medium text-slate-500">Name</label>
            <input
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="October re-engagement"
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-slate-500">
              Approved template name
            </label>
            <input
              required
              value={templateName}
              onChange={(e) => setTemplateName(e.target.value)}
              placeholder="order_update"
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-slate-500">Target tag</label>
            <input
              required
              value={tagName}
              onChange={(e) => setTagName(e.target.value)}
              placeholder="vip"
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <button
            type="submit"
            disabled={saving}
            className="rounded-md bg-slate-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
          >
            {saving ? 'Saving…' : 'Create'}
          </button>
        </form>
      )}

      {broadcasts === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : broadcasts.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-white px-4 py-10 text-center text-sm text-slate-500">
          No broadcasts yet.
        </div>
      ) : (
        <ul className="space-y-2">
          {broadcasts.map((b) => (
            <li key={b.id} className="rounded-lg border border-slate-200 bg-white">
              <div className="flex items-center justify-between gap-4 px-4 py-3">
                <div className="min-w-0">
                  <p className="text-sm font-medium text-slate-900">{b.name}</p>
                  <p className="text-xs text-slate-500">
                    {b.templateName} → #{b.tagName} · {timeAgo(b.createdAt)}
                  </p>
                </div>
                <div className="flex shrink-0 items-center gap-3">
                  <span className="text-xs text-slate-500">
                    {b.sentCount} sent · {b.deliveredCount} delivered · {b.readCount} read ·{' '}
                    {b.failedCount} failed
                  </span>
                  <button
                    onClick={() => toggleRecipients(b.id)}
                    className="rounded-md border border-slate-300 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50"
                  >
                    Recipients
                  </button>
                  {b.status === 'draft' && (
                    <button
                      onClick={() => dispatch(b.id)}
                      disabled={dispatchingId === b.id}
                      className="rounded-md bg-slate-900 px-2.5 py-1 text-xs font-medium text-white disabled:opacity-50"
                    >
                      {dispatchingId === b.id ? 'Sending…' : 'Send now'}
                    </button>
                  )}
                  {b.status !== 'draft' && (
                    <span className="badge badge-ghost badge-sm">{b.status}</span>
                  )}
                </div>
              </div>
              {openId === b.id && (
                <div className="border-t border-slate-100 px-4 py-3">
                  {recipients === null ? (
                    <p className="text-sm text-slate-500">Loading…</p>
                  ) : recipients.length === 0 ? (
                    <p className="text-sm text-slate-500">No recipients yet.</p>
                  ) : (
                    <ul className="space-y-1 text-sm">
                      {recipients.map((r) => (
                        <li key={r.id} className="flex items-center justify-between">
                          <span className="text-slate-600">{r.contactId.slice(0, 8)}</span>
                          <span
                            className={
                              r.status === 'failed'
                                ? 'text-red-600'
                                : r.status === 'read'
                                  ? 'text-emerald-600'
                                  : 'text-slate-500'
                            }
                          >
                            {r.status}
                            {r.error ? ` — ${r.error}` : ''}
                          </span>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
