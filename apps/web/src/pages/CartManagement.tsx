import { useEffect, useState } from 'react'
import { ShoppingCart, Plus, X, Check, Trash2, CreditCard, Search } from '../components/icons'
import { api, type CommerceCart, type CommerceCartItem, type CommerceProduct } from '../lib/api'

export function CartManagement() {
  const [carts, setCarts] = useState<CommerceCart[]>([])
  const [selectedCart, setSelectedCart] = useState<string | null>(null)
  const [items, setItems] = useState<CommerceCartItem[]>([])
  const [products, setProducts] = useState<CommerceProduct[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showAddProduct, setShowAddProduct] = useState(false)
  const [selectedProductId, setSelectedProductId] = useState('')
  const [quantity, setQuantity] = useState('1')
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('active')
  const brandId = api.getBrandId() || ''

  useEffect(() => {
    load()
  }, [brandId, statusFilter])

  useEffect(() => {
    if (selectedCart) loadItems(selectedCart)
  }, [selectedCart])

  async function load() {
    try {
      const [cartData, prodData] = await Promise.all([
        api.listCarts(brandId).catch(() => []),
        api.listProducts(brandId).catch(() => []),
      ])
      setCarts(cartData)
      setProducts(prodData)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function loadItems(cartId: string) {
    try {
      const itemData = await api.getCartItems(cartId).catch(() => [])
      setItems(itemData)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function addItem(e: React.FormEvent) {
    e.preventDefault()
    if (!selectedCart || !selectedProductId) return
    const prod = products.find((p) => p.id === selectedProductId)
    if (!prod) return
    try {
      await api.addCartItem(selectedCart, {
        productId: selectedProductId,
        quantity: parseInt(quantity) || 1,
        unitPrice: prod.price ?? 0,
      })
      setShowAddProduct(false)
      setSelectedProductId('')
      setQuantity('1')
      setSuccess('Item added to cart.')
      loadItems(selectedCart)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function removeItem(itemId: string) {
    try {
      await api.removeCartItem(itemId)
      setItems((prev) => prev.filter((i) => i.id !== itemId))
      setSuccess('Item removed.')
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function updateQuantity(itemId: string, newQty: number) {
    if (newQty < 1) return
    try {
      await api.updateCartItem(itemId, newQty)
      setItems((prev) => prev.map((i) => i.id === itemId ? { ...i, quantity: newQty } : i))
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function checkout() {
    if (!selectedCart) return
    if (!confirm('Convert this cart to an order?')) return
    try {
      await api.checkoutCart(selectedCart)
      setSuccess('Cart converted to order!')
      setSelectedCart(null)
      setItems([])
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  const filteredCarts = carts.filter((c) => c.status === statusFilter)
  const selectedItems = selectedCart ? items : []
  const total = selectedItems.reduce((sum, i) => sum + i.unitPrice * i.quantity, 0)

  const filteredProducts = products.filter((p) =>
    p.name.toLowerCase().includes(search.toLowerCase()) ||
    (p.sku || '').toLowerCase().includes(search.toLowerCase())
  )

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading carts...</div>

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--color-ink)]">Cart Management</h1>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">{carts.length} carts, {filteredCarts.length} {statusFilter}</p>
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
        {/* Cart List */}
        <div className="lg:col-span-1 space-y-4">
          <div className="flex items-center gap-2">
            <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}
              className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-2 py-1 text-sm bg-[var(--color-surface)]">
              <option value="active">Active</option>
              <option value="saved">Saved</option>
              <option value="converted">Converted</option>
            </select>
          </div>

          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden divide-y divide-[var(--color-border)]">
            {filteredCarts.length === 0 ? (
              <p className="p-5 text-sm text-[var(--color-ink-muted)]">No carts found.</p>
            ) : (
              filteredCarts.map((c) => (
                <button key={c.id} onClick={() => { setSelectedCart(c.id); setItems([]); }}
                  className={`w-full text-left px-4 py-3 hover:bg-[var(--color-surface-hover)] transition-colors
                    ${selectedCart === c.id ? 'bg-[var(--color-accent)]/10 border-l-4 border-[var(--color-accent)]' : ''}`}>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-[var(--color-ink)]">
                      Cart #{c.id.slice(0, 8)}
                    </span>
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium
                      ${c.status === 'active' ? 'bg-green-100 text-green-700' :
                        c.status === 'converted' ? 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]' :
                        'bg-blue-100 text-blue-700'}`}>
                      {c.status}
                    </span>
                  </div>
                  <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
                    {c.customerId.slice(0, 8)}{c.channelType ? ` · ${c.channelType}` : ''}
                  </p>
                </button>
              ))
            )}
          </div>
        </div>

        {/* Cart Items */}
        <div className="lg:col-span-2 space-y-4">
          {selectedCart ? (
            <>
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-bold text-[var(--color-ink)]">Cart Items</h2>
                <div className="flex items-center gap-2">
                  {selectedItems.length > 0 && (
                    <button onClick={checkout}
                      className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
                      <CreditCard className="h-4 w-4" /> Checkout
                    </button>
                  )}
                  <button onClick={() => setShowAddProduct((v) => !v)}
                    className="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">
                    {showAddProduct ? <><X className="h-4 w-4" /> Cancel</> : <><Plus className="h-4 w-4" /> Add item</>}
                  </button>
                </div>
              </div>

              {/* Add item form */}
              {showAddProduct && (
                <form onSubmit={addItem} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm space-y-3">
                  <div className="relative">
                    <Search className="absolute left-3 top-2.5 h-4 w-4 text-[var(--color-ink-muted)]" />
                    <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search products..."
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] pl-10 pr-3 py-2 text-sm bg-[var(--color-surface)]" />
                  </div>
                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                    <select required value={selectedProductId} onChange={(e) => setSelectedProductId(e.target.value)}
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
                      <option value="">Select product...</option>
                      {filteredProducts.slice(0, 50).map((p) => (
                        <option key={p.id} value={p.id}>{p.name} - ${(p.price ?? 0).toFixed(2)}</option>
                      ))}
                    </select>
                    <input type="number" min="1" value={quantity} onChange={(e) => setQuantity(e.target.value)}
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
                    <button type="submit"
                      className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
                      Add to cart
                    </button>
                  </div>
                </form>
              )}

              {/* Items list */}
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
                {selectedItems.length === 0 ? (
                  <p className="p-5 text-sm text-[var(--color-ink-muted)]">No items in this cart.</p>
                ) : (
                  <>
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                          <th className="px-4 py-2">Product</th>
                          <th className="px-4 py-2">Price</th>
                          <th className="px-4 py-2">Qty</th>
                          <th className="px-4 py-2">Total</th>
                          <th className="px-4 py-2"></th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-[var(--color-border)]">
                        {selectedItems.map((item) => {
                          const prod = products.find((p) => p.id === item.productId)
                          return (
                            <tr key={item.id} className="hover:bg-[var(--color-surface-hover)]">
                              <td className="px-4 py-3 font-medium">{prod?.name || item.productId.slice(0, 8)}</td>
                              <td className="px-4 py-3">${item.unitPrice.toFixed(2)}</td>
                              <td className="px-4 py-3">
                                <div className="flex items-center gap-1">
                                  <button onClick={() => updateQuantity(item.id, item.quantity - 1)}
                                    className="w-6 h-6 rounded border border-[var(--color-border)] flex items-center justify-center text-sm hover:bg-[var(--color-surface-hover)]">-</button>
                                  <span className="w-8 text-center text-sm">{item.quantity}</span>
                                  <button onClick={() => updateQuantity(item.id, item.quantity + 1)}
                                    className="w-6 h-6 rounded border border-[var(--color-border)] flex items-center justify-center text-sm hover:bg-[var(--color-surface-hover)]">+</button>
                                </div>
                              </td>
                              <td className="px-4 py-3 font-medium">${(item.unitPrice * item.quantity).toFixed(2)}</td>
                              <td className="px-4 py-3">
                                <button onClick={() => removeItem(item.id)} className="text-red-600 hover:text-red-800">
                                  <Trash2 className="h-4 w-4" />
                                </button>
                              </td>
                            </tr>
                          )
                        })}
                      </tbody>
                    </table>
                    <div className="border-t border-[var(--color-border)] px-4 py-3 flex items-center justify-between">
                      <span className="text-sm font-bold text-[var(--color-ink)]">Total</span>
                      <span className="text-lg font-bold text-[var(--color-accent)]">${total.toFixed(2)}</span>
                    </div>
                  </>
                )}
              </div>
            </>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm">
              <ShoppingCart className="h-12 w-12 text-[var(--color-ink-muted)] mb-3" />
              <p className="text-sm text-[var(--color-ink-muted)]">Select a cart to view items</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
