import { useEffect, useState } from 'react'
import { api, type Service } from '../lib/api'

export function Services() {
  const [services, setServices] = useState<Service[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [summary, setSummary] = useState('')
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
    try {
      await api.createService({ name, summary })
      setName('')
      setSummary('')
      setShowForm(false)
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
          <h1 className="text-xl font-semibold text-slate-900">Service catalog</h1>
          <p className="mt-1 text-sm text-slate-500">
            What the reply engine is allowed to tell leads about.
          </p>
        </div>
        <button
          onClick={() => setShowForm((v) => !v)}
          className="shrink-0 rounded-md bg-slate-900 px-3 py-2 text-sm font-medium text-white hover:bg-slate-800"
        >
          {showForm ? 'Cancel' : 'Add service'}
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
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-slate-500">Summary</label>
            <textarea
              value={summary}
              onChange={(e) => setSummary(e.target.value)}
              rows={2}
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <button
            type="submit"
            disabled={saving}
            className="rounded-md bg-slate-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
        </form>
      )}

      {services === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : services.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-white px-4 py-10 text-center text-sm text-slate-500">
          No services yet. Add one so the reply engine has something to cite.
        </div>
      ) : (
        <ul className="divide-y divide-slate-200 rounded-lg border border-slate-200 bg-white">
          {services.map((s) => (
            <li key={s.id} className="px-4 py-3">
              <p className="text-sm font-medium text-slate-900">{s.name}</p>
              {s.summary && <p className="mt-0.5 text-sm text-slate-500">{s.summary}</p>}
              {s.targetInstitutionTypes?.length > 0 && (
                <p className="mt-1 text-xs text-slate-400">
                  For: {s.targetInstitutionTypes.join(', ')}
                </p>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
