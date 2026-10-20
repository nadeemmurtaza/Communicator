import { useEffect, useRef, useState } from 'react'
import { api, type ContactRecord } from '../lib/api'

export function Contacts() {
  const [contacts, setContacts] = useState<ContactRecord[] | null>(null)
  const [tags, setTags] = useState<string[]>([])
  const [filter, setFilter] = useState<string>('all')
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [importing, setImporting] = useState(false)
  const fileRef = useRef<HTMLInputElement>(null)

  function load(tag?: string) {
    api
      .listContacts(tag)
      .then(setContacts)
      .catch((e) => setError(e.message))
  }

  useEffect(() => {
    load()
    api
      .listContactTags()
      .then((t) => setTags(t.map((x) => x.name)))
      .catch(() => {})
  }, [])

  useEffect(() => {
    load(filter === 'all' ? undefined : filter)
  }, [filter])

  async function handleImport(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    setImporting(true)
    setError(null)
    try {
      const result = await api.importContacts(file)
      setNotice(`Imported ${result.created} contacts (${result.skipped} skipped).`)
      load(filter === 'all' ? undefined : filter)
      api.listContactTags().then((t) => setTags(t.map((x) => x.name)))
    } catch (err) {
      setError((err as Error).message)
    } finally {
      setImporting(false)
      if (fileRef.current) fileRef.current.value = ''
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-slate-900">Contacts</h1>
          <p className="mt-1 text-sm text-slate-500">Your WhatsApp audience, tagged for broadcasts.</p>
        </div>
        <label className="shrink-0 cursor-pointer rounded-md bg-slate-900 px-3 py-2 text-sm font-medium text-white hover:bg-slate-800">
          {importing ? 'Importing…' : 'Import CSV'}
          <input
            ref={fileRef}
            type="file"
            accept=".csv"
            onChange={handleImport}
            disabled={importing}
            className="hidden"
          />
        </label>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}
      {notice && (
        <div className="rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
          {notice}
        </div>
      )}

      <p className="text-xs text-slate-500">
        CSV columns: <code className="rounded bg-slate-100 px-1">name</code>,{' '}
        <code className="rounded bg-slate-100 px-1">phone</code> (with country code),{' '}
        <code className="rounded bg-slate-100 px-1">tags</code> (comma-separated, optional).
      </p>

      <div className="flex flex-wrap gap-1.5">
        <button
          onClick={() => setFilter('all')}
          className={`rounded-full px-3 py-1 text-xs font-medium ${
            filter === 'all' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
          }`}
        >
          All
        </button>
        {tags.map((t) => (
          <button
            key={t}
            onClick={() => setFilter(t)}
            className={`rounded-full px-3 py-1 text-xs font-medium ${
              filter === t ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {contacts === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : contacts.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-white px-4 py-10 text-center text-sm text-slate-500">
          No contacts yet. Import a CSV, or they'll appear as leads sync in from Harvester.
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border border-slate-200 bg-white">
          <table className="w-full text-sm">
            <thead className="border-b border-slate-200 bg-slate-50 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
              <tr>
                <th className="px-4 py-2">Name</th>
                <th className="px-4 py-2">Number</th>
                <th className="px-4 py-2">Tags</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {contacts.map((c) => (
                <tr key={c.id}>
                  <td className="px-4 py-3 font-medium text-slate-900">{c.personName || '—'}</td>
                  <td className="px-4 py-3 text-slate-600">{c.value}</td>
                  <td className="px-4 py-3">
                    <div className="flex flex-wrap gap-1">
                      {c.tags.map((t) => (
                        <span key={t} className="badge badge-ghost badge-sm">
                          {t}
                        </span>
                      ))}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
