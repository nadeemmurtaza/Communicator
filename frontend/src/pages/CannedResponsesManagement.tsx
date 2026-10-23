import { useEffect, useState } from 'react'
import { api } from '../lib/api'

export function CannedResponsesManagement() {
  const [responses, setResponses] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [code, setCode] = useState('')
  const [category, setCategory] = useState('support')
  const [content, setContent] = useState('')
  const [saving, setSaving] = useState(false)

  function refresh() {
    api.listCannedResponses().then(setResponses).finally(() => setLoading(false))
  }

  useEffect(refresh, [])

  async function handleCreate() {
    if (!code.trim() || !content.trim()) return
    setSaving(true)
    try {
      await api.createCannedResponse({
        short_code: code.trim().replace(/^\//, ''),
        content: content.trim(),
        category: category.trim().toLowerCase() || 'general',
      })
      setCode('')
      setContent('')
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to create canned response')
    } finally {
      setSaving(false)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this canned response?')) return
    await api.deleteCannedResponse(id)
    refresh()
  }

  if (loading) return <p className="text-sm text-[var(--color-ink-muted)]">Loading canned responses...</p>

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-[var(--color-ink)]">Canned Responses &amp; Slash Commands</h1>
        <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
          Reusable response templates, accessible in any chat composer via <span className="font-mono font-semibold text-[var(--color-accent)]">/shortcode</span>.
        </p>
      </div>

      <div className="glass-card p-5 space-y-3">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div className="flex items-center">
            <span className="bg-[var(--color-accent-soft)] border border-r-0 border-[var(--color-border)] rounded-l-[var(--radius-sm)] px-2.5 py-1.5 text-xs font-mono font-bold text-[var(--color-accent)]">
              /
            </span>
            <input
              type="text"
              placeholder="e.g. shipping_time"
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/[^a-zA-Z0-9_-]/g, ''))}
              className="w-full text-sm border border-[var(--color-border)] rounded-r-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
            />
          </div>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
          >
            <option value="greetings">Greetings</option>
            <option value="sales">Sales</option>
            <option value="support">Support</option>
            <option value="billing">Billing</option>
            <option value="general">General</option>
          </select>
        </div>
        <textarea
          rows={2}
          placeholder="Hi {{customer.first_name}}! Thanks for reaching out..."
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
        />
        <div className="flex justify-end">
          <button
            type="button"
            onClick={handleCreate}
            disabled={saving || !code.trim() || !content.trim()}
            className="px-4 py-1.5 text-sm bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded-[var(--radius-sm)] font-semibold disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Canned Snippet'}
          </button>
        </div>
      </div>

      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3">{responses.length} snippets</h2>
        <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)]">
          {responses.map((c) => (
            <div key={c.id || c.shortCode || c.short_code} className="p-3 hover:bg-black/5 transition-colors flex items-start justify-between gap-3">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs font-bold text-[var(--color-accent)] bg-[var(--color-accent-soft)] px-2 py-0.5 rounded">
                    /{c.shortCode || c.short_code}
                  </span>
                  <span className="text-xs uppercase tracking-wide px-1.5 py-0.5 rounded-full bg-black/5 text-[var(--color-ink-muted)]">
                    {c.category || 'general'}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-ink)]">{c.content}</p>
              </div>
              <button
                type="button"
                onClick={() => handleDelete(c.id)}
                className="text-xs text-[var(--color-ink-muted)] hover:text-red-600 px-2 py-1 rounded hover:bg-red-50 shrink-0"
              >
                Delete
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
