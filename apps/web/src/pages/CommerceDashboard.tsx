import { useEffect, useState } from 'react'
import { ShoppingCart, Package, DollarSign, TrendingUp, Plus, Search, X, Check } from '../components/icons'
import { api, type CommerceProduct, type CommerceCategory, type CommerceOrder } from '../lib/api'

export function CommerceDashboard() {
  const [stats, setStats] = useState<any>(null)
  const [products, setProducts] = useState<CommerceProduct[]>([])
  const [orders, setOrders] = useState<CommerceOrder[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const brandId = api.getBrandId() || ''

  useEffect(() => {
    async function load() {
      try {
        const [prods, ords] = await Promise.all([
          api.listProducts(brandId).catch(() => [] as CommerceProduct[]),
          api.listOrders(brandId).catch(() => [] as CommerceOrder[]),
        ])
        setProducts(prods)
        setOrders(ords)
        const totalRevenue = ords.reduce((s, o) => s + (o.totalAmount || 0), 0)
        const pendingOrders = ords.filter((o) => o.status === 'pending').length
        setStats({ totalProducts: prods.length, totalOrders: ords.length, totalRevenue, pendingOrders })
      } catch (e) {
        setError((e as Error).message)
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [brandId])

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading commerce data...</div>
  if (error) return <div className="p-8 text-center text-red-600">{error}</div>

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--color-ink)]">Commerce Dashboard</h1>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">Overview of your store performance</p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {[
          { label: 'Products', value: stats?.totalProducts ?? 0, icon: Package, color: 'text-blue-600' },
          { label: 'Orders', value: stats?.totalOrders ?? 0, icon: ShoppingCart, color: 'text-green-600' },
          { label: 'Revenue', value: `$${(stats?.totalRevenue ?? 0).toFixed(2)}`, icon: DollarSign, color: 'text-purple-600' },
          { label: 'Pending', value: stats?.pendingOrders ?? 0, icon: TrendingUp, color: 'text-amber-600' },
        ].map((kpi) => (
          <div key={kpi.label} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
            <div className="flex items-center gap-3">
              <kpi.icon className={`h-5 w-5 ${kpi.color}`} />
              <div>
                <p className="text-2xl font-bold text-[var(--color-ink)]">{kpi.value}</p>
                <p className="text-xs text-[var(--color-ink-muted)]">{kpi.label}</p>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm">
        <div className="border-b border-[var(--color-border)] px-5 py-3">
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Recent Orders</h2>
        </div>
        {orders.length === 0 ? (
          <p className="p-5 text-sm text-[var(--color-ink-muted)]">No orders yet.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                <th className="px-5 py-2">Order</th>
                <th className="px-5 py-2">Status</th>
                <th className="px-5 py-2">Total</th>
                <th className="px-5 py-2">Channel</th>
              </tr>
            </thead>
            <tbody>
              {orders.slice(0, 10).map((o) => (
                <tr key={o.id} className="border-b border-[var(--color-border)] last:border-0">
                  <td className="px-5 py-3 font-medium">{o.orderNumber}</td>
                  <td className="px-5 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      o.status === 'delivered' ? 'bg-green-100 text-green-700' :
                      o.status === 'pending' ? 'bg-amber-100 text-amber-700' :
                      'bg-blue-100 text-blue-700'
                    }`}>{o.status}</span>
                  </td>
                  <td className="px-5 py-3">${(o.totalAmount ?? 0).toFixed(2)}</td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)]">{o.channelType || '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}
