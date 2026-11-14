import { useEffect, useState } from 'react'
import { Tag, Plus, X, Check, Trash2, Edit2 } from '../components/icons'
import { api, type CommerceCoupon } from '../lib/api'

export function CouponsManagement() {
  const [coupons, setCoupons] = useState<CommerceCoupon[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const brandId = api.getBrandId() || ''

  const [code, setCode] = useState('')
  const [type, setType] = useState('percentage')
  const [value, setValue] = useState('10')
  const [minOrderAmount, setMinOrderAmount] = useState('')
  const [usageLimit, setUsageLimit] = useState('')

  useEffect(() => { load() }, [brandId])

  async function load() {
    try {
      const data = await api.listCoupons(brandId).catch(() => [])
      setCoupons(data)
    } catch (e) { setError((e as Error).message) }
    finally { setLoading(false) }
  }

  async function create(e: React.FormEvent) {
    e.preventDefault()
    try {
      await api.createCoupon({
        brandId, code, type, value: parseFloat(value),
        minOrderAmount: minOrderAmount ? parseFloat(minOrderAmount) : undefined,
        usageLimit: usageLimit ? parseInt(usageLimit) : undefined,
      })
      setShowForm(false)
      setCode(''); setValue('10'); setMinOrderAmount(''); setUsageLimit('')
      setSuccess('Coupon created.')
      load()
    } catch (e) { setError((e as Error).message) }
  }

  async function toggleActive(coupon: CommerceCoupon) {
    try {
      await api.updateCoupon(coupon.id, { isActive: !coupon.isActive })
      load()
    } catch (e) { setError((e as Error).message) }
  }

  async function deleteCoupon(id: string) {
    if (!confirm('Delete this coupon?')) return
    try {
      await api.deleteCoupon(id)
      setSuccess('Coupon deleted.')
      load()
    } catch (e) { setError((e as Error).message) }
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading coupons...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Coupons & Discounts</h1>
          <p className="text-sm text-[var(--color-ink-muted)]">{coupons.length} coupons</p>
        </div>
        <button onClick={() => setShowForm(true)}
          className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
          <Plus className="h-4 w-4" /> New coupon
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
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">New Coupon</h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Code</label>
              <input required value={code} onChange={(e) => setCode(e.target.value.toUpperCase())}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" placeholder="SAVE20" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Type</label>
              <select value={type} onChange={(e) => setType(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
                <option value="percentage">Percentage</option>
                <option value="fixed">Fixed Amount</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Value</label>
              <input required type="number" value={value} onChange={(e) => setValue(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Min Order $</label>
              <input type="number" value={minOrderAmount} onChange={(e) => setMinOrderAmount(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Usage Limit</label>
              <input type="number" value={usageLimit} onChange={(e) => setUsageLimit(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
          </div>
          <div className="flex gap-2">
            <button type="submit" className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">Create</button>
            <button type="button" onClick={() => setShowForm(false)} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">Cancel</button>
          </div>
        </form>
      )}

      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
        {coupons.length === 0 ? (
          <p className="p-5 text-sm text-[var(--color-ink-muted)]">No coupons yet. Create one to get started.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                <th className="px-4 py-2">Code</th>
                <th className="px-4 py-2">Type</th>
                <th className="px-4 py-2">Value</th>
                <th className="px-4 py-2">Used</th>
                <th className="px-4 py-2">Status</th>
                <th className="px-4 py-2"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)]">
              {coupons.map((c) => (
                <tr key={c.id} className="hover:bg-[var(--color-surface-hover)]">
                  <td className="px-4 py-3 font-mono font-medium">{c.code}</td>
                  <td className="px-4 py-3 capitalize">{c.type}</td>
                  <td className="px-4 py-3">{c.type === 'percentage' ? `${c.value}%` : `$${c.value.toFixed(2)}`}</td>
                  <td className="px-4 py-3">{c.usedCount}{c.usageLimit ? ` / ${c.usageLimit}` : ''}</td>
                  <td className="px-4 py-3">
                    <button onClick={() => toggleActive(c)}
                      className={`rounded-full px-2 py-0.5 text-xs font-medium ${c.isActive ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'}`}>
                      {c.isActive ? 'Active' : 'Inactive'}
                    </button>
                  </td>
                  <td className="px-4 py-3">
                    <button onClick={() => deleteCoupon(c.id)} className="p-1 text-red-500 hover:bg-red-50 rounded">
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}
