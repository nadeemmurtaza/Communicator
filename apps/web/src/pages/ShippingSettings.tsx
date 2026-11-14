import { useEffect, useState } from 'react'
import { Truck, Plus, X, Check, Trash2 } from '../components/icons'
import { api, type ShippingMethod } from '../lib/api'

export function ShippingSettings() {
  const [methods, setMethods] = useState<ShippingMethod[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const brandId = api.getBrandId() || ''

  const [name, setName] = useState('')
  const [type, setType] = useState('flat_rate')
  const [rate, setRate] = useState('0')

  useEffect(() => { load() }, [brandId])

  async function load() {
    try { const data = await api.listShippingMethods(brandId).catch(() => []); setMethods(data) }
    catch (e) { setError((e as Error).message) }
    finally { setLoading(false) }
  }

  async function create(e: React.FormEvent) {
    e.preventDefault()
    try {
      await api.createShippingMethod({ brandId, name, type, rate: parseFloat(rate) })
      setShowForm(false); setName(''); setRate('0')
      setSuccess('Shipping method created.')
      load()
    } catch (e) { setError((e as Error).message) }
  }

  async function deleteMethod(id: string) {
    if (!confirm('Delete this shipping method?')) return
    try { await api.deleteShippingMethod(id); setSuccess('Deleted.'); load() }
    catch (e) { setError((e as Error).message) }
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Shipping Settings</h1>
          <p className="text-sm text-[var(--color-ink-muted)]">{methods.length} methods</p>
        </div>
        <button onClick={() => setShowForm(true)}
          className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
          <Plus className="h-4 w-4" /> Add method
        </button>
      </div>

      {error && <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
        <span>{error}</span><button onClick={() => setError(null)}><X className="h-4 w-4" /></button>
      </div>}
      {success && <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
        <span className="flex items-center gap-1.5"><Check className="h-4 w-4" /> {success}</span>
        <button onClick={() => setSuccess(null)}><X className="h-4 w-4" /></button>
      </div>}

      {showForm && (
        <form onSubmit={create} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-4">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">New Shipping Method</h2>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Name</label>
              <input required value={name} onChange={(e) => setName(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" placeholder="Standard Shipping" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Type</label>
              <select value={type} onChange={(e) => setType(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
                <option value="flat_rate">Flat Rate</option>
                <option value="free">Free Shipping</option>
                <option value="weight">Weight Based</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Rate ($)</label>
              <input type="number" step="0.01" value={rate} onChange={(e) => setRate(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
          </div>
          <div className="flex gap-2">
            <button type="submit" className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">Create</button>
            <button type="button" onClick={() => setShowForm(false)} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">Cancel</button>
          </div>
        </form>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {methods.map((m) => (
          <div key={m.id} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
            <div className="flex items-start justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <Truck className="h-4 w-4 text-[var(--color-accent)]" />
                  <h3 className="font-medium text-[var(--color-ink)]">{m.name}</h3>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)] mt-1 capitalize">{m.type}</p>
                <p className="text-lg font-bold text-[var(--color-ink)] mt-2">${m.rate.toFixed(2)}</p>
              </div>
              <button onClick={() => deleteMethod(m.id)} className="p-1 text-red-500 hover:bg-red-50 rounded">
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
