import { useEffect, useState } from 'react'
import { api } from '../lib/api'

export function MacrosManagement() {
  const [macros, setMacros] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreate, setShowCreate] = useState(false)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [resolve, setResolve] = useState(false)
  const [sendCsat, setSendCsat] = useState(false)
  const [tags, setTags] = useState('')
  const [message, setMessage] = useState('')
  const [note, setNote] = useState('')
  const [saving, setSaving] = useState(false)

  function refresh() {
    api.listMacros().then(setMacros).finally(() => setLoading(false))
  }

  useEffect(refresh, [])

  // Same action contract as the in-inbox macro creator (Dashboard.tsx's
  // handleCreateMacro) -- the backend rejects a macro with no actions, so
  // this form must build at least one real action, not submit actions: [].
  function buildActions(): any[] {
    const actions: any[] = []
    if (resolve) actions.push({ type: 'resolve_conversation', resolution_type: 'human_assisted' })
    if (sendCsat) actions.push({ type: 'send_csat_survey' })
    if (tags.trim()) {
      actions.push({
        type: 'add_tags',
        tags: tags.split(',').map((t) => t.trim().toLowerCase()).filter(Boolean),
      })
    }
    if (message.trim()) actions.push({ type: 'send_message', body: message.trim() })
    if (note.trim()) actions.push({ type: 'add_private_note', note: note.trim() })
    return actions
  }

  const actionsPreview = buildActions()

  async function handleCreate() {
    if (!name.trim() || actionsPreview.length === 0) return
    setSaving(true)
    try {
      await api.createMacro({ name: name.trim(), description: description.trim(), actions: actionsPreview })
      setName('')
      setDescription('')
      setResolve(false)
      setSendCsat(false)
      setTags('')
      setMessage('')
      setNote('')
      setShowCreate(false)
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to create macro')
    } finally {
      setSaving(false)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this macro?')) return
    await api.deleteMacro(id)
    refresh()
  }

  if (loading) return <p className="text-sm text-[var(--color-ink-muted)]">Loading macros...</p>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-[var(--color-ink)]">Macros</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
            One-click bundles of actions (reply, tag, resolve) agents trigger from a conversation.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setShowCreate((v) => !v)}
          className="px-4 py-2 rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold text-sm hover:bg-[var(--color-accent-hover)]"
        >
          {showCreate ? 'Cancel' : 'New Macro'}
        </button>
      </div>

      {showCreate && (
        <div className="glass-card p-5 space-y-3">
          <input
            type="text"
            placeholder="Macro name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />
          <textarea
            rows={2}
            placeholder="Description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />

          <p className="text-xs font-semibold text-[var(--color-ink-muted)] pt-1">
            What this macro does (at least one):
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <label className="flex items-center gap-2 text-sm text-[var(--color-ink)]">
              <input type="checkbox" checked={resolve} onChange={(e) => setResolve(e.target.checked)} />
              Resolve the conversation
            </label>
            <label className="flex items-center gap-2 text-sm text-[var(--color-ink)]">
              <input type="checkbox" checked={sendCsat} onChange={(e) => setSendCsat(e.target.checked)} />
              Send CSAT survey
            </label>
          </div>
          <input
            type="text"
            placeholder="Add tags (comma-separated)"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />
          <textarea
            rows={2}
            placeholder="Send message to contact"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />
          <textarea
            rows={2}
            placeholder="Add private note"
            value={note}
            onChange={(e) => setNote(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />

          <div className="flex items-center justify-between">
            <p className="text-xs text-[var(--color-ink-muted)]">
              {actionsPreview.length === 0 ? 'Select at least one action.' : `${actionsPreview.length} action(s)`}
            </p>
            <button
              type="button"
              onClick={handleCreate}
              disabled={saving || !name.trim() || actionsPreview.length === 0}
              className="px-4 py-1.5 text-sm bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded-[var(--radius-sm)] font-semibold disabled:opacity-50"
            >
              {saving ? 'Saving...' : 'Save Macro'}
            </button>
          </div>
        </div>
      )}

      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3">{macros.length} macros</h2>
        <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)] text-sm">
          {macros.map((m) => (
            <div key={m.id} className="p-3 flex items-center justify-between hover:bg-black/5 transition-colors">
              <div>
                <p className="font-medium text-[var(--color-ink)]">{m.name}</p>
                <p className="text-xs text-[var(--color-ink-muted)]">{m.description}</p>
              </div>
              <button
                type="button"
                onClick={() => handleDelete(m.id)}
                className="text-xs text-[var(--color-ink-muted)] hover:text-red-600 px-2 py-1 rounded hover:bg-red-50"
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
