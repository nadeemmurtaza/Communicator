import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  getPersonalProfileOverview,
  type PersonalProfileOverview,
  type PersonalOrder,
  type NidInvoice,
  type NidPayment,
  type NidSubscription,
} from '../lib/api'
import {
  User,
  ShoppingCart,
  FileText,
  CreditCard,
  CheckCircle,
  AlertCircle,
  ArrowRight,
  Package,
  Globe,
} from '../components/icons'

function StatusBadge({ status }: { status: string }) {
  const s = status.toLowerCase()
  let colors = 'text-muted bg-accent-muted'
  if (['active', 'paid', 'succeeded'].includes(s)) colors = 'text-green-600 bg-green-50'
  else if (['pending', 'processing', 'incomplete'].includes(s)) colors = 'text-amber-600 bg-amber-50'
  else if (['failed', 'overdue', 'void', 'canceled', 'cancelled'].includes(s))
    colors = 'text-red-600 bg-red-50'

  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-4px ${colors}`}>
      {status}
    </span>
  )
}

function formatAmount(minor: number, currency: string) {
  const amount = minor / 100
  return `${currency} ${amount.toLocaleString('en-US', { minimumFractionDigits: 2 })}`
}

function formatDate(dateStr?: string) {
  if (!dateStr) return '—'
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

export function PersonalProfile() {
  const { nidAccountId } = useParams<{ nidAccountId: string }>()
  const navigate = useNavigate()
  const [overview, setOverview] = useState<PersonalProfileOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'orders' | 'invoices' | 'payments' | 'subscriptions'>('orders')

  useEffect(() => {
    async function load() {
      if (!nidAccountId) return
      try {
        const data = await getPersonalProfileOverview(nidAccountId)
        setOverview(data)
      } catch (err: any) {
        setError(err.message || 'Failed to load profile')
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [nidAccountId])

  if (loading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-xl font-semibold">Personal Profile</h1>
          <p className="text-sm text-muted mt-0.5">Loading your personal data...</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <div key={i} className="bg-white border border-border rounded-8px p-5 animate-pulse">
              <div className="h-4 bg-accent-muted rounded w-1/2 mb-3" />
              <div className="h-6 bg-accent-muted rounded w-1/3" />
            </div>
          ))}
        </div>
      </div>
    )
  }

  if (error || !overview) {
    return (
      <div className="space-y-6">
        <h1 className="text-xl font-semibold">Personal Profile</h1>
        <div className="bg-white border border-border rounded-8px p-8 text-center">
          <AlertCircle size={24} className="mx-auto text-red-600 mb-3" />
          <p className="text-sm text-muted">{error || 'Profile not found'}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-black text-white text-sm rounded-4px hover:scale-[1.05] transition-transform"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  const { profile, orders, invoices, payments, subscriptions, totalSpent } = overview

  const tabs = [
    { key: 'orders' as const, label: 'Orders', count: orders.length, icon: <ShoppingCart size={14} /> },
    { key: 'invoices' as const, label: 'Invoices', count: invoices.length, icon: <FileText size={14} /> },
    { key: 'payments' as const, label: 'Payments', count: payments.length, icon: <CreditCard size={14} /> },
    { key: 'subscriptions' as const, label: 'Subscriptions', count: subscriptions.length, icon: <Globe size={14} /> },
  ]

  return (
    <div className="space-y-6">
      {/* Profile Header */}
      <div className="bg-white border border-border rounded-8px p-6">
        <div className="flex items-start gap-4">
          <div className="w-14 h-14 rounded-8px bg-accent-muted text-accent flex items-center justify-center flex-shrink-0">
            <User size={24} />
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-semibold">{profile.displayName}</h1>
              <span className="text-[10px] font-medium uppercase tracking-wider px-2 py-0.5 rounded-4px bg-accent-muted text-accent">
                Personal
              </span>
            </div>
            {profile.headline && <p className="text-sm text-muted mt-0.5">{profile.headline}</p>}
            <div className="flex flex-wrap items-center gap-3 mt-2 text-xs text-faint">
              <span className="uppercase tracking-wider">{profile.category}</span>
              {profile.phone && <span>· {profile.phone}</span>}
              {profile.publicEmail && <span>· {profile.publicEmail}</span>}
              {profile.city && <span>· {profile.city}</span>}
            </div>
          </div>
          <div className="text-right">
            <p className="text-xs text-faint uppercase tracking-wider">Total Spent</p>
            <p className="text-2xl font-bold mt-1">
              {totalSpent.toLocaleString('en-US', { style: 'currency', currency: 'USD' })}
            </p>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-border">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
              activeTab === tab.key
                ? 'border-accent text-accent'
                : 'border-transparent text-muted hover:text-ink'
            }`}
          >
            {tab.icon}
            {tab.label}
            {tab.count > 0 && (
              <span className="text-xs bg-accent-muted text-accent px-1.5 py-0.5 rounded-4px">
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Orders Tab */}
      {activeTab === 'orders' && (
        <div className="space-y-3">
          {orders.length === 0 ? (
            <div className="bg-white border border-border rounded-8px p-8 text-center">
              <ShoppingCart size={24} className="mx-auto text-faint mb-3" />
              <p className="text-sm text-muted">No orders yet</p>
            </div>
          ) : (
            orders.map((order) => (
              <button
                key={order.id}
                onClick={() => navigate(`/order/${order.id}`)}
                className="w-full text-left bg-white border border-border rounded-8px p-5 hover:border-accent transition-colors group"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-4px bg-accent-muted text-accent flex items-center justify-center">
                      <Package size={16} />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <p className="font-medium text-sm">{order.orderNumber}</p>
                        <StatusBadge status={order.status} />
                      </div>
                      <div className="flex items-center gap-2 mt-0.5 text-xs text-faint">
                        <span className="flex items-center gap-1">
                          <Globe size={10} />
                          {order.platformName}
                        </span>
                        <span>·</span>
                        <span>{order.brandName}</span>
                        {order.shopName && (
                          <>
                            <span>·</span>
                            <span>{order.shopName}</span>
                          </>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="text-right">
                      <p className="font-semibold text-sm">
                        {order.totalAmount.toLocaleString('en-US', {
                          style: 'currency',
                          currency: order.currency,
                        })}
                      </p>
                      <p className="text-xs text-faint">{formatDate(order.createdAt)}</p>
                    </div>
                    <ArrowRight
                      size={14}
                      className="text-faint group-hover:text-accent transition-colors"
                    />
                  </div>
                </div>
              </button>
            ))
          )}
        </div>
      )}

      {/* Invoices Tab */}
      {activeTab === 'invoices' && (
        <div className="space-y-3">
          {invoices.length === 0 ? (
            <div className="bg-white border border-border rounded-8px p-8 text-center">
              <FileText size={24} className="mx-auto text-faint mb-3" />
              <p className="text-sm text-muted">No invoices yet</p>
            </div>
          ) : (
            invoices.map((inv) => (
              <div
                key={inv.id}
                className="bg-white border border-border rounded-8px p-5"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-4px bg-blue-50 text-blue-600 flex items-center justify-center">
                      <FileText size={14} />
                    </div>
                    <div>
                      <p className="font-medium text-sm">{inv.invoiceNumber}</p>
                      <p className="text-xs text-faint">
                        {formatDate(inv.issuedAt)} · Due {formatDate(inv.dueAt)}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <StatusBadge status={inv.status} />
                    <p className="font-semibold text-sm">
                      {formatAmount(inv.totalMinor, inv.currency)}
                    </p>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Payments Tab */}
      {activeTab === 'payments' && (
        <div className="space-y-3">
          {payments.length === 0 ? (
            <div className="bg-white border border-border rounded-8px p-8 text-center">
              <CreditCard size={24} className="mx-auto text-faint mb-3" />
              <p className="text-sm text-muted">No payments yet</p>
            </div>
          ) : (
            payments.map((pay) => (
              <div
                key={pay.id}
                className="bg-white border border-border rounded-8px p-5"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-4px bg-green-50 text-green-600 flex items-center justify-center">
                      <CreditCard size={14} />
                    </div>
                    <div>
                      <p className="font-medium text-sm capitalize">{pay.provider}</p>
                      <p className="text-xs text-faint">
                        {pay.methodType && `${pay.methodType} · `}
                        {formatDate(pay.paidAt)}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <StatusBadge status={pay.status} />
                    <p className="font-semibold text-sm">
                      {formatAmount(pay.amountMinor, pay.currency)}
                    </p>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Subscriptions Tab */}
      {activeTab === 'subscriptions' && (
        <div className="space-y-3">
          {subscriptions.length === 0 ? (
            <div className="bg-white border border-border rounded-8px p-8 text-center">
              <Globe size={24} className="mx-auto text-faint mb-3" />
              <p className="text-sm text-muted">No active subscriptions</p>
            </div>
          ) : (
            subscriptions.map((sub) => (
              <div
                key={sub.id}
                className="bg-white border border-border rounded-8px p-5"
              >
                <div className="flex items-center justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <p className="font-medium text-sm">{sub.planName}</p>
                      <StatusBadge status={sub.status} />
                    </div>
                    <p className="text-xs text-faint mt-0.5">
                      {sub.planMode} · {sub.planKey}
                    </p>
                    {sub.currentPeriodStart && (
                      <p className="text-xs text-faint">
                        Period: {formatDate(sub.currentPeriodStart)} —{' '}
                        {formatDate(sub.currentPeriodEnd)}
                      </p>
                    )}
                  </div>
                  {sub.cancelAtPeriodEnd && (
                    <span className="text-xs text-amber-600 bg-amber-50 px-2 py-0.5 rounded-4px">
                      Cancels at period end
                    </span>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}
