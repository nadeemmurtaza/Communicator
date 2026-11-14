import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ShoppingCart, Search, X, Check, Trash2 } from '../components/icons'
import { api, type CommerceOrder } from '../lib/api'

const STATUS_OPTIONS = ['pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled']

export function OrdersList() {
  const navigate = useNavigate()
  const [orders, setOrders] = useState<CommerceOrder[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('')
  const brandId = api.getBrandId() || ''

  useEffect(() => { load() }, [brandId])

  async function load() {
    try {
      const ords = await api.listOrders(brandId).catch(() => [])
      setOrders(ords)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function updateStatus(orderId: string, status: string) {
    try {
      await api.updateOrderStatus(orderId, status)
      setSuccess(`Order status updated to ${status}.`)
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  const filtered = orders.filter((o) => {
    const matchSearch = o.orderNumber.toLowerCase().includes(search.toLowerCase())
    const matchStatus = !statusFilter || o.status === statusFilter
    return matchSearch && matchStatus
  })

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading orders...</div>

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--color-ink)]">Orders</h1>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">{orders.length} total orders</p>
      </div>

      {error && <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
        <span>{error}</span><button onClick={() => setError(null)}><X className="h-4 w-4" /></button>
      </div>}
      {success && <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
        <span className="flex items-center gap-1.5"><Check className="h-4 w-4" /> {success}</span>
        <button onClick={() => setSuccess(null)}><X className="h-4 w-4" /></button>
      </div>}

      <div className="flex gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-[var(--color-ink-muted)]" />
          <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search orders..."
            className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] pl-10 pr-3 py-2 text-sm bg-[var(--color-surface)]" />
        </div>
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}
          className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
          <option value="">All statuses</option>
          {STATUS_OPTIONS.map((s) => <option key={s} value={s}>{s}</option>)}
        </select>
      </div>

      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <p className="p-5 text-sm text-[var(--color-ink-muted)]">No orders found.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                <th className="px-5 py-2">Order</th>
                <th className="px-5 py-2">Status</th>
                <th className="px-5 py-2">Payment</th>
                <th className="px-5 py-2">Total</th>
                <th className="px-5 py-2">Channel</th>
                <th className="px-5 py-2">Date</th>
                <th className="px-5 py-2"></th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((o) => (
                <tr key={o.id} className="border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-surface-hover)]">
                  <td className="px-5 py-3">
                    <button onClick={() => navigate(`/commerce/orders/${o.id}`)}
                      className="font-medium text-[var(--color-accent)] hover:underline">{o.orderNumber}</button>
                  </td>
                  <td className="px-5 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      o.status === 'delivered' ? 'bg-green-100 text-green-700' :
                      o.status === 'cancelled' ? 'bg-red-100 text-red-700' :
                      o.status === 'pending' ? 'bg-amber-100 text-amber-700' :
                      'bg-blue-100 text-blue-700'
                    }`}>{o.status}</span>
                  </td>
                  <td className="px-5 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      o.paymentStatus === 'paid' ? 'bg-green-100 text-green-700' :
                      o.paymentStatus === 'refunded' ? 'bg-red-100 text-red-700' :
                      'bg-[var(--color-accent-muted)] text-[var(--color-ink)]'
                    }`}>{o.paymentStatus || 'pending'}</span>
                  </td>
                  <td className="px-5 py-3">${(o.totalAmount ?? 0).toFixed(2)}</td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)]">{o.channelType || '-'}</td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)]">{o.createdAt ? new Date(o.createdAt).toLocaleDateString() : '-'}</td>
                  <td className="px-5 py-3">
                    <select
                      value={o.status}
                      onChange={(e) => updateStatus(o.id, e.target.value)}
                      className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-2 py-1 text-xs bg-[var(--color-surface)]"
                    >
                      {STATUS_OPTIONS.map((s) => <option key={s} value={s}>{s}</option>)}
                    </select>
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
