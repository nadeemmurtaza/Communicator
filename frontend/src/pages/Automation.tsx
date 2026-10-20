import { useEffect, useState } from 'react'
import { api, type AutomationRule } from '../lib/api'

const ACTION_LABELS: Record<string, string> = {
  auto_reply: 'Reply with',
  handoff: 'Hand off to a human',
  add_tag: 'Add tag',
}

export function Automation() {
  const [rules, setRules] = useState<AutomationRule[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [keyword, setKeyword] = useState('')
  const [matchType, setMatchType] = useState('contains')
  const [actionType, setActionType] = useState('auto_reply')
  const [actionValue, setActionValue] = useState('')
  const [saving, setSaving] = useState(false)

  function load() {
    api
      .listAutomationRules()
      .then(setRules)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      await api.createAutomationRule({
        name,
        trigger_keyword: keyword,
        match_type: matchType,
        action_type: actionType,
        action_value: actionValue,
        priority: 0,
        is_active: true,
      })
      setName('')
      setKeyword('')
      setActionValue('')
      setShowForm(false)
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function toggleActive(rule: AutomationRule) {
    try {
      await api.updateAutomationRule(rule.id, {
        name: rule.name,
        trigger_keyword: rule.triggerKeyword,
        match_type: rule.matchType,
        action_type: rule.actionType,
        action_value: rule.actionValue,
        priority: rule.priority,
        is_active: !rule.isActive,
      })
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteAutomationRule(id)
      setRules((prev) => prev?.filter((r) => r.id !== id) ?? null)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-slate-900">Automation rules</h1>
          <p className="mt-1 text-sm text-slate-500">
            Keyword shortcuts checked before the AI, highest priority first — "agent" hands off,
            "menu" replies instantly, that kind of thing.
          </p>
        </div>
        <button
          onClick={() => setShowForm((v) => !v)}
          className="shrink-0 rounded-md bg-slate-900 px-3 py-2 text-sm font-medium text-white hover:bg-slate-800"
        >
          {showForm ? 'Cancel' : 'Add rule'}
        </button>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {showForm && (
        <form onSubmit={submit} className="space-y-3 rounded-lg border border-slate-200 bg-white p-4">
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div>
              <label className="block text-xs font-medium text-slate-500">Name</label>
              <input
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-500">Keyword</label>
              <input
                required
                value={keyword}
                onChange={(e) => setKeyword(e.target.value)}
                placeholder="agent"
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-500">Match type</label>
              <select
                value={matchType}
                onChange={(e) => setMatchType(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              >
                <option value="contains">Contains</option>
                <option value="exact">Exact match</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-500">Action</label>
              <select
                value={actionType}
                onChange={(e) => setActionType(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              >
                <option value="auto_reply">Reply with</option>
                <option value="handoff">Hand off to a human</option>
                <option value="add_tag">Add tag</option>
              </select>
            </div>
          </div>
          {actionType !== 'handoff' && (
            <div>
              <label className="block text-xs font-medium text-slate-500">
                {actionType === 'add_tag' ? 'Tag name' : 'Reply text'}
              </label>
              <textarea
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                rows={actionType === 'add_tag' ? 1 : 3}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
          )}
          <button
            type="submit"
            disabled={saving}
            className="rounded-md bg-slate-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
        </form>
      )}

      {rules === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : rules.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-white px-4 py-10 text-center text-sm text-slate-500">
          No rules yet. The AI handles every reply until you add one.
        </div>
      ) : (
        <ul className="divide-y divide-slate-200 rounded-lg border border-slate-200 bg-white">
          {rules.map((r) => (
            <li key={r.id} className="flex items-center justify-between gap-4 px-4 py-3">
              <div className="min-w-0">
                <p className="text-sm font-medium text-slate-900">{r.name}</p>
                <p className="text-xs text-slate-500">
                  {r.matchType === 'exact' ? 'Exactly matches' : 'Contains'} "{r.triggerKeyword}" →{' '}
                  {ACTION_LABELS[r.actionType]}
                  {r.actionValue ? `: ${r.actionValue}` : ''}
                </p>
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <button
                  onClick={() => toggleActive(r)}
                  className={`rounded-full px-2.5 py-1 text-xs font-medium ${
                    r.isActive ? 'bg-emerald-50 text-emerald-700' : 'bg-slate-100 text-slate-500'
                  }`}
                >
                  {r.isActive ? 'Active' : 'Paused'}
                </button>
                <button
                  onClick={() => remove(r.id)}
                  className="text-xs font-medium text-slate-400 hover:text-red-600"
                >
                  Delete
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
