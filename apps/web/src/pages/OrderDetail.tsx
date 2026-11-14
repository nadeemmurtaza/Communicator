import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Check, X, Package, CreditCard, MapPin, Clock, User } from '../components/icons'
import { api, type CommerceOrder } from '../lib/api'

const STATUS_OPTIONS = ['draft', 'pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled']

const STATUS_COLORS: Record<string, string> = {
  draft: 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]',
  pending: 'bg-yellow-100 text-yellow-700',
  confirmed: 'bg-blue-100 text-blue-700',
  processing: 'bg-purple-100 text-purple-700',
  shipped: 'bg-indigo-100 text-indigo-700',
  delivered: 'bg-green-100 text-green-700',
  cancelled: 'bg-red-100 text-red-700',
}

const PAYMENT_COLORS: Record<string, string> = {
  pending: 'bg-yellow-100 text-yellow-700',
  paid: 'bg-green-100 text-green-700',
  partially_paid: 'bg-blue-100 text-blue-700',
  refunded: 'bg-red-100 text-red-700',
  failed: 'bg-red-100 text-red-700',
}

export function OrderDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [order, setOrder] = useState<CommerceOrder | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [newStatus, setNewStatus] = useState('')

  useEffect(() => {
    async function load() {
      try {
        if (id) {
          const ord = await api.getOrder(id)
          setOrder(ord)
          setNewStatus(ord.status)
        }
      } catch (e) {
        setError((e as Error).message)
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [id])

  async function updateStatus() {
    if (!id || !newStatus) return
    setSaving(true)
    setError(null)
    try {
      const updated = await api.updateOrderStatus(id, newStatus)
      setOrder(updated)
      setSuccess(`Order status updated to "${newStatus}".`)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading order...</div>
  if (!order) return <div className="p-8 text-center text-red-600">Order not found.</div>

  // Parse items snapshot
  let items: any[] = []
  try {
    if (order.itemsSnapshot) {
      items = JSON.parse(order.itemsSnapshot)
    }
  } catch { /* ignore */ }

  // Parse shipping address
  let address: any = null
  try {
    if (order.shippingAddress) {
      address = JSON.parse(order.shippingAddress)
    }
  } catch { /* ignore */ }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/commerce/orders')}
            className="p-2 rounded-[var(--radius-sm)] hover:bg-[var(--color-surface-hover)]">
            <ArrowLeft className="h-5 w-5 text-[var(--color-ink-muted)]" />
          </button>
          <div>
            <h1 className="text-xl font-semibold text-[var(--color-ink)]">
              Order {order.orderNumber}
            </h1>
            <p className="text-sm text-[var(--color-ink-muted)]">
              Created {new Date(order.createdAt).toLocaleDateString()} · {order.source || 'internal'}
              {order.channelType ? ` · ${order.channelType}` : ''}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className={`rounded-full px-3 py-1 text-xs font-medium ${STATUS_COLORS[order.status] || 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]'}`}>
            {order.status}
          </span>
          <span className={`rounded-full px-3 py-1 text-xs font-medium ${PAYMENT_COLORS[order.paymentStatus] || 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]'}`}>
            {order.paymentStatus}
          </span>
        </div>
      </div>

      {error && (
        <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)}><X className="h-4 w-4" /></button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span className="flex items-center gap-1.5"><Check className="h-4 w-4" /> {success}</span>
          <button onClick={() => setSuccess(null)}><X className="h-4 w-4" /></button>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Order Items */}
        <div className="lg:col-span-2 space-y-6">
          {/* Items */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
            <div className="border-b border-[var(--color-border)] px-5 py-3">
              <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
                <Package className="h-4 w-4" /> Order Items
              </h2>
            </div>
            {items.length === 0 ? (
              <p className="p-5 text-sm text-[var(--color-ink-muted)]">No items in this order.</p>
            ) : (
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                    <th className="px-5 py-2">Product</th>
                    <th className="px-5 py-2">SKU</th>
                    <th className="px-5 py-2">Qty</th>
                    <th className="px-5 py-2">Price</th>
                    <th className="px-5 py-2">Total</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-border)]">
                  {items.map((item: any, i: number) => (
                    <tr key={i} className="hover:bg-[var(--color-surface-hover)]">
                      <td className="px-5 py-3 font-medium">{item.productName || item.name || '-'}</td>
                      <td className="px-5 py-3 text-[var(--color-ink-muted)]">{item.sku || '-'}</td>
                      <td className="px-5 py-3">{item.quantity}</td>
                      <td className="px-5 py-3">${(item.unitPrice ?? 0).toFixed(2)}</td>
                      <td className="px-5 py-3 font-medium">${(item.totalPrice ?? item.unitPrice * item.quantity ?? 0).toFixed(2)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
            <div className="border-t border-[var(--color-border)] px-5 py-3 space-y-1">
              <div className="flex justify-between text-sm">
                <span className="text-[var(--color-ink-muted)]">Subtotal</span>
                <span>${(order.subtotal ?? 0).toFixed(2)}</span>
              </div>
              {order.tax && order.tax > 0 && (
                <div className="flex justify-between text-sm">
                  <span className="text-[var(--color-ink-muted)]">Tax</span>
                  <span>${order.tax.toFixed(2)}</span>
                </div>
              )}
              {order.shipping && order.shipping > 0 && (
                <div className="flex justify-between text-sm">
                  <span className="text-[var(--color-ink-muted)]">Shipping</span>
                  <span>${order.shipping.toFixed(2)}</span>
                </div>
              )}
              <div className="flex justify-between text-base font-bold border-t border-[var(--color-border)] pt-2 mt-2">
                <span>Total</span>
                <span className="text-[var(--color-accent)]">${(order.totalAmount ?? 0).toFixed(2)}</span>
              </div>
            </div>
          </div>

          {/* Shipping Address */}
          {address && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5">
              <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2 mb-3">
                <MapPin className="h-4 w-4" /> Shipping Address
              </h2>
              <div className="text-sm text-[var(--color-ink-muted)] space-y-1">
                {address.name && <p>{address.name}</p>}
                {address.line1 && <p>{address.line1}</p>}
                {address.line2 && <p>{address.line2}</p>}
                <p>{[address.city, address.state, address.postalCode].filter(Boolean).join(', ')} {address.country || ''}</p>
              </div>
            </div>
          )}
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Status Update */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5 space-y-3">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <Clock className="h-4 w-4" /> Update Status
            </h2>
            <select value={newStatus} onChange={(e) => setNewStatus(e.target.value)}
              className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
              {STATUS_OPTIONS.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
            <button onClick={updateStatus} disabled={saving || newStatus === order.status}
              className="w-full rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50">
              {saving ? 'Updating...' : 'Update status'}
            </button>
          </div>

          {/* Customer Info */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5 space-y-3">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <User className="h-4 w-4" /> Customer
            </h2>
            <p className="text-sm text-[var(--color-ink-muted)]">
              ID: {order.customerId}
            </p>
          </div>

          {/* Payment Info */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5 space-y-3">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <CreditCard className="h-4 w-4" /> Payment
            </h2>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-[var(--color-ink-muted)]">Status</span>
                <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${PAYMENT_COLORS[order.paymentStatus] || 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]'}`}>
                  {order.paymentStatus}
                </span>
              </div>
              {order.paymentMethod && (
                <div className="flex justify-between">
                  <span className="text-[var(--color-ink-muted)]">Method</span>
                  <span className="capitalize">{order.paymentMethod}</span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-[var(--color-ink-muted)]">Currency</span>
                <span>{order.currency}</span>
              </div>
            </div>
          </div>

          {/* Order Timeline */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5 space-y-3">
            <h2 className="text-sm font-bold text-[var(--color-ink)]">Timeline</h2>
            <div className="space-y-2 text-sm">
              <div className="flex items-start gap-2">
                <div className="w-2 h-2 rounded-full bg-[var(--color-accent)] mt-1.5 shrink-0" />
                <div>
                  <p className="text-[var(--color-ink)]">Order created</p>
                  <p className="text-xs text-[var(--color-ink-muted)]">{new Date(order.createdAt).toLocaleString()}</p>
                </div>
              </div>
              {order.updatedAt !== order.createdAt && (
                <div className="flex items-start gap-2">
                  <div className="w-2 h-2 rounded-full bg-[var(--color-ink-muted)] mt-1.5 shrink-0" />
                  <div>
                    <p className="text-[var(--color-ink)]">Last updated</p>
                    <p className="text-xs text-[var(--color-ink-muted)]">{new Date(order.updatedAt).toLocaleString()}</p>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* External Order */}
          {order.externalOrderId && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5 space-y-2">
              <h2 className="text-sm font-bold text-[var(--color-ink)]">External Reference</h2>
              <p className="text-sm text-[var(--color-ink-muted)]">
                External ID: {order.externalOrderId}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
