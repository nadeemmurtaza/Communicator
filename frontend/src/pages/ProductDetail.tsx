import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Save, X, Plus, Trash2, Check } from '../components/icons'
import { api, type CommerceProduct, type CommerceCategory } from '../lib/api'

const CHANNELS = ['whatsapp', 'email', 'sms', 'instagram', 'telegram', 'tiktok', 'facebook']

export function ProductDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [product, setProduct] = useState<CommerceProduct | null>(null)
  const [categories, setCategories] = useState<CommerceCategory[]>([])
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Form state
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [price, setPrice] = useState('')
  const [compareAtPrice, setCompareAtPrice] = useState('')
  const [sku, setSku] = useState('')
  const [categoryId, setCategoryId] = useState('')
  const [inventoryCount, setInventoryCount] = useState('0')
  const [trackInventory, setTrackInventory] = useState(false)
  const [isActive, setIsActive] = useState(true)
  const [variants, setVariants] = useState('')
  const [images, setImages] = useState('')
  const [visibleChannels, setVisibleChannels] = useState<string[]>([])

  useEffect(() => {
    async function load() {
      try {
        const [prod, cats] = await Promise.all([
          id ? api.getProduct(id) : Promise.resolve(null),
          api.listCategories(api.getBrandId() || '').catch(() => []),
        ])
        if (prod) {
          setProduct(prod)
          setName(prod.name)
          setDescription(prod.description || '')
          setPrice((prod.price ?? 0).toString())
          setCompareAtPrice((prod.compareAtPrice ?? 0).toString())
          setSku(prod.sku || '')
          setCategoryId(prod.categoryId || '')
          setInventoryCount((prod.inventoryCount ?? 0).toString())
          setTrackInventory(prod.trackInventory)
          setIsActive(prod.isActive)
          setVariants(prod.variants || '')
          setImages(prod.images || '')
          // Parse visibility JSON
          try {
            const vis = prod.visibility ? JSON.parse(prod.visibility) : {}
            const ch = Object.keys(vis).filter((k) => vis[k])
            setVisibleChannels(ch)
          } catch {
            setVisibleChannels([])
          }
        }
        setCategories(cats)
      } catch (e) {
        setError((e as Error).message)
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [id])

  async function save(e: React.FormEvent) {
    e.preventDefault()
    if (!id) return
    setSaving(true)
    setError(null)
    try {
      const visibility: Record<string, boolean> = {}
      visibleChannels.forEach((ch) => (visibility[ch] = true))
      await api.updateProduct(id, {
        name,
        description: description || undefined,
        price: parseFloat(price) || 0,
        compareAtPrice: compareAtPrice ? parseFloat(compareAtPrice) : undefined,
        sku: sku || undefined,
        categoryId: categoryId || undefined,
        inventoryCount: parseInt(inventoryCount) || 0,
        trackInventory,
        isActive,
      })
      setSuccess('Product updated successfully.')
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  function toggleChannel(ch: string) {
    setVisibleChannels((prev) =>
      prev.includes(ch) ? prev.filter((c) => c !== ch) : [...prev, ch]
    )
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading product...</div>
  if (!product) return <div className="p-8 text-center text-red-600">Product not found.</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <button onClick={() => navigate('/commerce/catalog')}
          className="p-2 rounded-[var(--radius-sm)] hover:bg-[var(--color-surface-hover)]">
          <ArrowLeft className="h-5 w-5 text-[var(--color-ink-muted)]" />
        </button>
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Edit Product</h1>
          <p className="text-sm text-[var(--color-ink-muted)]">ID: {product.id}</p>
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

      <form onSubmit={save} className="space-y-6">
        {/* Basic Info */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-4">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">Basic Information</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="sm:col-span-2">
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Name</label>
              <input required value={name} onChange={(e) => setName(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Description</label>
              <textarea rows={3} value={description} onChange={(e) => setDescription(e.target.value)}
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
        </div>

        {/* Pricing */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-4">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">Pricing & Inventory</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Price</label>
              <input required type="number" step="0.01" value={price} onChange={(e) => setPrice(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Compare at Price</label>
              <input type="number" step="0.01" value={compareAtPrice} onChange={(e) => setCompareAtPrice(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Inventory Count</label>
              <input type="number" value={inventoryCount} onChange={(e) => setInventoryCount(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div className="flex items-center gap-2 pt-6">
              <input type="checkbox" id="trackInventory" checked={trackInventory} onChange={(e) => setTrackInventory(e.target.checked)}
                className="rounded border-[var(--color-border)]" />
              <label htmlFor="trackInventory" className="text-sm text-[var(--color-ink)]">Track inventory</label>
            </div>
            <div className="flex items-center gap-2 pt-6">
              <input type="checkbox" id="isActive" checked={isActive} onChange={(e) => setIsActive(e.target.checked)}
                className="rounded border-[var(--color-border)]" />
              <label htmlFor="isActive" className="text-sm text-[var(--color-ink)]">Active</label>
            </div>
          </div>
        </div>

        {/* Variants & Images */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-4">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">Variants & Images</h2>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Variants (JSON)
            </label>
            <textarea rows={3} value={variants} onChange={(e) => setVariants(e.target.value)}
              placeholder='[{"id":"v1","name":"Red","options":{"color":"red"},"priceAdjustment":0}]'
              className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)] font-mono" />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
              Images (JSON array of URLs)
            </label>
            <textarea rows={2} value={images} onChange={(e) => setImages(e.target.value)}
              placeholder='["https://example.com/image1.jpg"]'
              className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)] font-mono" />
          </div>
        </div>

        {/* Channel Visibility */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-4">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">Channel Visibility</h2>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
            {CHANNELS.map((ch) => (
              <label key={ch} className="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" checked={visibleChannels.includes(ch)} onChange={() => toggleChannel(ch)}
                  className="rounded border-[var(--color-border)]" />
                <span className="text-sm text-[var(--color-ink)] capitalize">{ch}</span>
              </label>
            ))}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-3">
          <button type="submit" disabled={saving}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50">
            <Save className="h-4 w-4" /> {saving ? 'Saving...' : 'Save changes'}
          </button>
          <button type="button" onClick={() => navigate(`/commerce/catalog/${id}/options`)}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">
            <Plus className="h-4 w-4" /> Manage Options & Add-ons
          </button>
          <button type="button" onClick={() => navigate('/commerce/catalog')}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">
            <X className="h-4 w-4" /> Cancel
          </button>
        </div>
      </form>
    </div>
  )
}
