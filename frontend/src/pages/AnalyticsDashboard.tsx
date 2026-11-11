import { useEffect, useState } from 'react'
import { TrendingUp, TrendingDown, DollarSign, ShoppingCart, Users, Package, BarChart3 } from '../components/icons'
import { api, type AnalyticsOverview, type RevenueTrendPoint, type TopProduct, type TopCustomer, type ChannelPerformance } from '../lib/api'

export function AnalyticsDashboard() {
  const [overview, setOverview] = useState<AnalyticsOverview | null>(null)
  const [trend, setTrend] = useState<RevenueTrendPoint[]>([])
  const [topProducts, setTopProducts] = useState<TopProduct[]>([])
  const [topCustomers, setTopCustomers] = useState<TopCustomer[]>([])
  const [channels, setChannels] = useState<ChannelPerformance[]>([])
  const [loading, setLoading] = useState(true)
  const brandId = api.getBrandId() || ''

  useEffect(() => {
    async function load() {
      try {
        const [ov, tr, tp, tc, ch] = await Promise.all([
          api.getAnalyticsOverview(brandId, 30).catch(() => null),
          api.getRevenueTrend(brandId, 30).catch(() => []),
          api.getTopProducts(brandId, 10).catch(() => []),
          api.getTopCustomers(brandId, 10).catch(() => []),
          api.getChannelPerformance(brandId).catch(() => []),
        ])
        setOverview(ov)
        setTrend(tr)
        setTopProducts(tp)
        setTopCustomers(tc)
        setChannels(ch)
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [brandId])

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading analytics...</div>

  const maxRevenue = Math.max(...trend.map((t) => t.revenue), 1)

  return (
    <div className="space-y-6">
      <h1 className="text-xl font-semibold text-[var(--color-ink)]">Analytics Dashboard</h1>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
        <KpiCard title="Revenue" value={`$${overview?.totalRevenue.toFixed(2) ?? '0.00'}`} change={overview?.revenueChangePercent} icon={<DollarSign className="h-5 w-5" />} />
        <KpiCard title="Orders" value={overview?.totalOrders ?? 0} change={overview?.ordersChangePercent} icon={<ShoppingCart className="h-5 w-5" />} />
        <KpiCard title="Avg Order" value={`$${(overview?.avgOrderValue ?? 0).toFixed(2)}`} icon={<BarChart3 className="h-5 w-5" />} />
        <KpiCard title="Customers" value={overview?.totalCustomers ?? 0} icon={<Users className="h-5 w-5" />} />
        <KpiCard title="Products" value={overview?.totalProducts ?? 0} icon={<Package className="h-5 w-5" />} />
      </div>

      {/* Revenue Trend */}
      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
        <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider mb-4">Revenue Trend (30 days)</h2>
        {trend.length === 0 ? (
          <p className="text-sm text-[var(--color-ink-muted)]">No data yet.</p>
        ) : (
          <div className="space-y-1">
            {trend.map((t) => (
              <div key={t.date} className="flex items-center gap-3">
                <span className="text-xs text-[var(--color-ink-muted)] w-20 shrink-0">{t.date}</span>
                <div className="flex-1 bg-[var(--color-surface-hover)] rounded-full h-4 overflow-hidden">
                  <div
                    className="h-full bg-[var(--color-accent)] rounded-full transition-all"
                    style={{ width: `${(t.revenue / maxRevenue) * 100}%` }}
                  />
                </div>
                <span className="text-xs font-medium text-[var(--color-ink)] w-24 text-right">${t.revenue.toFixed(2)}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Top Products */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider mb-4">Top Products</h2>
          {topProducts.length === 0 ? (
            <p className="text-sm text-[var(--color-ink-muted)]">No sales data yet.</p>
          ) : (
            <div className="space-y-3">
              {topProducts.map((p, i) => (
                <div key={p.productId} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-xs font-bold text-[var(--color-ink-muted)] w-5">{i + 1}</span>
                    <div>
                      <p className="text-sm font-medium text-[var(--color-ink)]">{p.productName}</p>
                      <p className="text-xs text-[var(--color-ink-muted)]">{p.quantitySold} sold</p>
                    </div>
                  </div>
                  <span className="text-sm font-semibold text-[var(--color-ink)]">${p.revenue.toFixed(2)}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Top Customers */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider mb-4">Top Customers</h2>
          {topCustomers.length === 0 ? (
            <p className="text-sm text-[var(--color-ink-muted)]">No customer data yet.</p>
          ) : (
            <div className="space-y-3">
              {topCustomers.map((c, i) => (
                <div key={c.customerId} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-xs font-bold text-[var(--color-ink-muted)] w-5">{i + 1}</span>
                    <div>
                      <p className="text-sm font-medium text-[var(--color-ink)]">{c.customerName || c.customerId.slice(0, 8)}</p>
                      <p className="text-xs text-[var(--color-ink-muted)]">{c.totalOrders} orders</p>
                    </div>
                  </div>
                  <span className="text-sm font-semibold text-[var(--color-ink)]">${c.totalSpent.toFixed(2)}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Channel Performance */}
      {channels.length > 0 && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider mb-4">Channel Performance</h2>
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4">
            {channels.map((ch) => (
              <div key={ch.channelType} className="text-center p-3 rounded-[var(--radius-md)] bg-[var(--color-surface-hover)]">
                <p className="text-xs text-[var(--color-ink-muted)] capitalize">{ch.channelType || 'direct'}</p>
                <p className="text-lg font-bold text-[var(--color-ink)]">${ch.revenue.toFixed(0)}</p>
                <p className="text-xs text-[var(--color-ink-muted)]">{ch.ordersCount} orders</p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

function KpiCard({ title, value, change, icon }: { title: string; value: string | number; change?: number; icon?: React.ReactNode }) {
  return (
    <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider">{title}</span>
        {icon && <span className="text-[var(--color-accent)]">{icon}</span>}
      </div>
      <p className="text-2xl font-bold text-[var(--color-ink)]">{value}</p>
      {change !== undefined && (
        <div className={`flex items-center gap-1 mt-1 text-xs ${change >= 0 ? 'text-emerald-600' : 'text-red-600'}`}>
          {change >= 0 ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
          {change >= 0 ? '+' : ''}{change.toFixed(1)}%
        </div>
      )}
    </div>
  )
}
