import { useEffect, useState } from 'react'
import { Check, ChevronDown, ChevronUp, GitBranch, Plus, X } from '../components/icons'
import { api, type Sequence, type VariantStats } from '../lib/api'

export function Sequences() {
  const [sequences, setSequences] = useState<Sequence[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [openId, setOpenId] = useState<string | null>(null)
  const [stats, setStats] = useState<VariantStats | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [isActive, setIsActive] = useState(true)
  const [saving, setSaving] = useState(false)

  function load() {
    api
      .listSequences()
      .then(setSequences)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  async function toggle(id: string) {
    if (openId === id) {
      setOpenId(null)
      return
    }
    setOpenId(id)
    setStats(null)
    try {
      const data = await api.sequenceVariantStats(id)
      setStats(data)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleCreateSequence(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    setSuccess(null)
    try {
      await api.createSequence({
        name,
        description: description.trim() || undefined,
        is_active: isActive,
      })
      setName('')
      setDescription('')
      setIsActive(true)
      setShowForm(false)
      setSuccess(`Sequence "${name}" created successfully.`)
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <GitBranch className="h-5 w-5 text-indigo-600" />
            <h1 className="text-xl font-semibold text-[var(--color-ink)]">Follow-up sequences</h1>
          </div>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            A/B variant performance and automated cadence per sequence.
          </p>
        </div>
        <button
          onClick={() => setShowForm((v) => !v)}
          className="shrink-0 flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] shadow-xs"
        >
          {showForm ? (
            <>
              <X className="h-4 w-4" /> Cancel
            </>
          ) : (
            <>
              <Plus className="h-4 w-4" /> Add sequence
            </>
          )}
        </button>
      </div>

      {error && (
        <div className="flex items-center justify-between rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="font-bold">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      {success && (
        <div className="flex items-center justify-between rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span className="flex items-center gap-1.5">
            <Check className="h-4 w-4 text-emerald-600" /> {success}
          </span>
          <button onClick={() => setSuccess(null)} className="font-bold">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      {showForm && (
        <form
          onSubmit={handleCreateSequence}
          className="space-y-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm"
        >
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Configure new follow-up sequence</h2>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Sequence Name
            </label>
            <input
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Enterprise Re-Engagement Sequence"
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              placeholder="Purpose and schedule of this follow-up sequence..."
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="is_active"
              checked={isActive}
              onChange={(e) => setIsActive(e.target.checked)}
              className="rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
            />
            <label htmlFor="is_active" className="text-xs font-medium text-[var(--color-ink)] cursor-pointer">
              Active sequence (enrollments permitted)
            </label>
          </div>
          <div className="flex items-center justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={() => setShowForm(false)}
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-4 py-1.5 text-xs font-bold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 shadow-xs"
            >
              <Check className="h-3.5 w-3.5" />
              {saving ? 'Saving...' : 'Save sequence'}
            </button>
          </div>
        </form>
      )}

      {sequences === null ? (
        <p className="text-sm text-[var(--color-ink-muted)]">Loading sequences...</p>
      ) : sequences.length === 0 ? (
        <div className="rounded-xl border border-dashed border-slate-300 bg-[var(--color-surface)] px-4 py-12 text-center text-sm text-[var(--color-ink-muted)]">
          <GitBranch className="h-8 w-8 text-slate-400 mx-auto mb-2" />
          No sequences configured. Add one to enable automated follow-up cadences.
        </div>
      ) : (
        <ul className="space-y-3">
          {sequences.map((s) => (
            <li
              key={s.id}
              className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs overflow-hidden transition-colors"
            >
              <button
                onClick={() => toggle(s.id)}
                className="flex w-full items-center justify-between p-4 text-left hover:bg-[var(--color-page)]/50"
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-50 text-indigo-600">
                    <GitBranch className="h-4 w-4" />
                  </div>
                  <div>
                    <span className="text-sm font-semibold text-[var(--color-ink)]">{s.name}</span>
                    <span className="ml-2 text-xs text-[var(--color-ink-faint)]">ID: {s.id}</span>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span
                    className={`inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold ${
                      s.isActive ? 'bg-emerald-100 text-emerald-800' : 'bg-slate-100 text-slate-600'
                    }`}
                  >
                    <span className={`h-1.5 w-1.5 rounded-full ${s.isActive ? 'bg-emerald-500' : 'bg-slate-400'}`} />
                    {s.isActive ? 'Active' : 'Inactive'}
                  </span>
                  {openId === s.id ? (
                    <ChevronUp className="h-4 w-4 text-[var(--color-ink-faint)]" />
                  ) : (
                    <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)]" />
                  )}
                </div>
              </button>
              {openId === s.id && (
                <div className="border-t border-[var(--color-border)] bg-[var(--color-page)]/40 p-4">
                  {stats === null ? (
                    <p className="text-xs text-[var(--color-ink-muted)]">Loading variant statistics...</p>
                  ) : stats.variants.length === 0 ? (
                    <p className="text-xs text-[var(--color-ink-muted)]">No enrollments yet recorded for this sequence.</p>
                  ) : (
                    <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden shadow-2xs">
                      <table className="w-full text-xs">
                        <thead className="bg-[var(--color-page)]/70 text-left uppercase tracking-wider text-[var(--color-ink-faint)] border-b border-[var(--color-border)]">
                          <tr>
                            <th className="py-2.5 px-3 font-semibold">Variant Name</th>
                            <th className="py-2.5 px-3 font-semibold text-right">Enrolled</th>
                            <th className="py-2.5 px-3 font-semibold text-right">Replied</th>
                            <th className="py-2.5 px-3 font-semibold text-right">Closed Won</th>
                            <th className="py-2.5 px-3 font-semibold text-right">Conversion Rate</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100">
                          {stats.variants.map((v) => {
                            const convRate = v.enrolled > 0 ? Math.round((v.closedWon / v.enrolled) * 100) : 0
                            return (
                              <tr key={v.variant} className="hover:bg-[var(--color-page)]/50">
                                <td className="py-2.5 px-3 font-medium text-[var(--color-ink)]">{v.variant}</td>
                                <td className="py-2.5 px-3 text-right text-[var(--color-ink)]">{v.enrolled}</td>
                                <td className="py-2.5 px-3 text-right text-indigo-600 font-medium">{v.replied}</td>
                                <td className="py-2.5 px-3 text-right text-emerald-700 font-semibold">{v.closedWon}</td>
                                <td className="py-2.5 px-3 text-right">
                                  <span className="rounded bg-emerald-50 px-1.5 py-0.5 font-bold text-emerald-700 border border-emerald-200">
                                    {convRate}%
                                  </span>
                                </td>
                              </tr>
                            )
                          })}
                        </tbody>
                      </table>
                    </div>
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
