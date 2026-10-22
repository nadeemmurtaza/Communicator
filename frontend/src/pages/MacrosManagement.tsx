import { useEffect, useState } from 'react'
import { api } from '../lib/api'

export function MacrosManagement() {
  const [macros, setMacros] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreate, setShowCreate] = useState(false)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [saving, setSaving] = useState(false)

  function refresh() {
    api.listMacros().then(setMacros).finally(() => setLoading(false))
  }

  useEffect(refresh, [])

  async function handleCreate() {
    if (!name.trim()) return
    setSaving(true)
    try {
      await api.createMacro({ name: name.trim(), description: description.trim(), actions: [] })
      setName('')
      setDescription('')
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
          <div className="flex justify-end">
            <button
              type="button"
              onClick={handleCreate}
              disabled={saving || !name.trim()}
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
