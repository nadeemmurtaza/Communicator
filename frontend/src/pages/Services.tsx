import { useEffect, useState } from 'react'
import { Briefcase, Check, Plus, Tag, X } from '../components/icons'
import { api, type Service } from '../lib/api'

export function Services() {
  const [services, setServices] = useState<Service[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [summary, setSummary] = useState('')
  const [targetTypes, setTargetTypes] = useState('')
  const [saving, setSaving] = useState(false)

  function load() {
    api
      .listServices()
      .then(setServices)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    setSuccess(null)
    try {
      const targets = targetTypes
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean)

      await api.createService({
        name,
        summary,
        target_institution_types: targets.length > 0 ? targets : ['Enterprise'],
      })
      setName('')
      setSummary('')
      setTargetTypes('')
      setShowForm(false)
      setSuccess(`Service "${name}" added to catalog successfully.`)
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
            <Briefcase className="h-5 w-5 text-indigo-600" />
            <h1 className="text-xl font-semibold text-[var(--color-ink)]">Service catalog</h1>
          </div>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            What the reply engine and AI copilot are allowed to cite when responding to leads.
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
              <Plus className="h-4 w-4" /> Add service
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
          onSubmit={submit}
          className="space-y-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm"
        >
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Add new service to catalog</h2>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Service Name
            </label>
            <input
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Enterprise Omnichannel Support"
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Summary
            </label>
            <textarea
              required
              value={summary}
              onChange={(e) => setSummary(e.target.value)}
              rows={3}
              placeholder="Brief description of the service used by the AI reply engine..."
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Target Institution Types (comma-separated)
            </label>
            <input
              value={targetTypes}
              onChange={(e) => setTargetTypes(e.target.value)}
              placeholder="e.g. Technology, Enterprise, Retail, Healthcare"
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
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
              {saving ? 'Saving...' : 'Save service'}
            </button>
          </div>
        </form>
      )}

      {services === null ? (
        <p className="text-sm text-[var(--color-ink-muted)]">Loading services...</p>
      ) : services.length === 0 ? (
        <div className="rounded-xl border border-dashed border-slate-300 bg-[var(--color-surface)] px-4 py-12 text-center text-sm text-[var(--color-ink-muted)]">
          <Briefcase className="h-8 w-8 text-slate-400 mx-auto mb-2" />
          No services yet. Add one so the reply engine has something to cite.
        </div>
      ) : (
        <ul className="divide-y divide-slate-100 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs overflow-hidden">
          {services.map((s) => (
            <li key={s.id} className="p-4 hover:bg-[var(--color-page)]/50 transition-colors">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">{s.name}</h3>
                  {s.summary && (
                    <p className="mt-1 text-sm text-[var(--color-ink-muted)] leading-relaxed">{s.summary}</p>
                  )}
                  {s.targetInstitutionTypes && s.targetInstitutionTypes.length > 0 && (
                    <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
                      <span className="text-[11px] font-medium text-[var(--color-ink-faint)]">Target:</span>
                      {s.targetInstitutionTypes.map((t) => (
                        <span
                          key={t}
                          className="inline-flex items-center gap-1 rounded bg-slate-100 px-2 py-0.5 text-[11px] font-medium text-slate-700"
                        >
                          <Tag className="h-2.5 w-2.5 text-slate-500" />
                          {t}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
