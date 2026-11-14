import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Package, Plus, Search, X, Check, Trash2, Edit2, Settings } from '../components/icons'
import { api, type CommerceProduct, type CommerceCategory } from '../lib/api'

export function CatalogManagement() {
  const navigate = useNavigate()
  const [products, setProducts] = useState<CommerceProduct[]>([])
  const [categories, setCategories] = useState<CommerceCategory[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [search, setSearch] = useState('')
  const [name, setName] = useState('')
  const [price, setPrice] = useState('')
  const [sku, setSku] = useState('')
  const [categoryId, setCategoryId] = useState('')
  const [saving, setSaving] = useState(false)
  const brandId = api.getBrandId() || ''

  useEffect(() => {
    async function load() {
      try {
        const [prods, cats] = await Promise.all([
          api.listProducts(brandId).catch(() => []),
          api.listCategories(brandId).catch(() => []),
        ])
        setProducts(prods)
        setCategories(cats)
      } catch (e) {
        setError((e as Error).message)
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [brandId])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    try {
      await api.createProduct({
        brandId,
        name,
        price: parseFloat(price) || 0,
        sku: sku || undefined,
        categoryId: categoryId || undefined,
      })
      setName('')
      setPrice('')
      setSku('')
      setCategoryId('')
      setShowForm(false)
      setSuccess(`Product "${name}" added.`)
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function deleteProduct(id: string) {
    if (!confirm('Delete this product?')) return
    try {
      await api.deleteProduct(id)
      setSuccess('Product deleted.')
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function load() {
    try {
      const [prods, cats] = await Promise.all([
        api.listProducts(brandId).catch(() => []),
        api.listCategories(brandId).catch(() => []),
      ])
      setProducts(prods)
      setCategories(cats)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  const filtered = products.filter((p) =>
    p.name.toLowerCase().includes(search.toLowerCase()) ||
    (p.sku || '').toLowerCase().includes(search.toLowerCase())
  )

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading catalog...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Product Catalog</h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">{products.length} products, {categories.length} categories</p>
        </div>
        <button onClick={() => setShowForm((v) => !v)}
          className="shrink-0 flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] shadow-none">
          {showForm ? <><X className="h-4 w-4" /> Cancel</> : <><Plus className="h-4 w-4" /> Add product</>}
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
        <form onSubmit={submit} className="space-y-4 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Add new product</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Name</label>
              <input required value={name} onChange={(e) => setName(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Price</label>
              <input required type="number" step="0.01" value={price} onChange={(e) => setPrice(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">SKU</label>
              <input value={sku} onChange={(e) => setSku(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Category</label>
              <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
                <option value="">None</option>
                {categories.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
              </select>
            </div>
          </div>
          <button type="submit" disabled={saving}
            className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50">
            {saving ? 'Saving...' : 'Add product'}
          </button>
        </form>
      )}

      <div className="relative">
        <Search className="absolute left-3 top-2.5 h-4 w-4 text-[var(--color-ink-muted)]" />
        <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search products..."
          className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] pl-10 pr-3 py-2 text-sm bg-[var(--color-surface)]" />
      </div>

      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <p className="p-5 text-sm text-[var(--color-ink-muted)]">No products found.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                <th className="px-5 py-2">Product</th>
                <th className="px-5 py-2">SKU</th>
                <th className="px-5 py-2">Price</th>
                <th className="px-5 py-2">Stock</th>
                <th className="px-5 py-2">Status</th>
                <th className="px-5 py-2"></th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((p) => (
                <tr key={p.id} className="border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-surface-hover)]">
                  <td className="px-5 py-3">
                    <button onClick={() => navigate(`/commerce/catalog/${p.id}`)}
                      className="font-medium text-[var(--color-accent)] hover:underline">{p.name}</button>
                  </td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)]">{p.sku || '-'}</td>
                  <td className="px-5 py-3">${(p.price ?? 0).toFixed(2)}</td>
                  <td className="px-5 py-3">{p.inventoryCount ?? 0}</td>
                  <td className="px-5 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${p.isActive ? 'bg-green-100 text-green-700' : 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]'}`}>
                      {p.isActive ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="px-5 py-3 flex items-center gap-2">
                    <button onClick={() => navigate(`/commerce/catalog/${p.id}/options`)}
                      className="text-[var(--color-ink-muted)] hover:text-[var(--color-accent)]" title="Options & Add-ons">
                      <Settings className="h-4 w-4" />
                    </button>
                    <button onClick={() => navigate(`/commerce/catalog/${p.id}`)} className="text-[var(--color-accent)] hover:text-[var(--color-accent-hover)]">
                      <Edit2 className="h-4 w-4" />
                    </button>
                    <button onClick={() => deleteProduct(p.id)} className="text-red-600 hover:text-red-800">
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
