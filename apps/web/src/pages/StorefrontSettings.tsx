import { useEffect, useState } from 'react'
import { Globe, Plus, X, Check, Trash2, ExternalLink } from '../components/icons'
import { api, type Storefront } from '../lib/api'

export function StorefrontSettings() {
  const [storefronts, setStorefronts] = useState<Storefront[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const brandId = api.getBrandId() || ''

  const [slug, setSlug] = useState('')
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [seoTitle, setSeoTitle] = useState('')
  const [seoDescription, setSeoDescription] = useState('')

  useEffect(() => { load() }, [brandId])

  async function load() {
    try { const data = await api.listStorefronts(brandId).catch(() => []); setStorefronts(data) }
    catch (e) { setError((e as Error).message) }
    finally { setLoading(false) }
  }

  async function create(e: React.FormEvent) {
    e.preventDefault()
    try {
      await api.createStorefront({ brandId, slug, title, description, seoTitle, seoDescription })
      setShowForm(false); setSlug(''); setTitle(''); setDescription(''); setSeoTitle(''); setSeoDescription('')
      setSuccess('Storefront created.')
      load()
    } catch (e) { setError((e as Error).message) }
  }

  async function toggleActive(sf: Storefront) {
    try { await api.updateStorefront(sf.id, { isActive: !sf.isActive }); load() }
    catch (e) { setError((e as Error).message) }
  }

  async function deleteStorefront(id: string) {
    if (!confirm('Delete this storefront?')) return
    try { await api.deleteStorefront(id); setSuccess('Deleted.'); load() }
    catch (e) { setError((e as Error).message) }
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Storefront Builder</h1>
          <p className="text-sm text-[var(--color-ink-muted)]">{storefronts.length} storefronts</p>
        </div>
        <button onClick={() => setShowForm(true)}
          className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
          <Plus className="h-4 w-4" /> New storefront
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
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">New Storefront</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Slug (URL)</label>
              <input required value={slug} onChange={(e) => setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ''))}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" placeholder="my-store" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Title</label>
              <input required value={title} onChange={(e) => setTitle(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" placeholder="My Store" />
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Description</label>
              <textarea rows={2} value={description} onChange={(e) => setDescription(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">SEO Title</label>
              <input value={seoTitle} onChange={(e) => setSeoTitle(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">SEO Description</label>
              <input value={seoDescription} onChange={(e) => setSeoDescription(e.target.value)}
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
        {storefronts.map((sf) => (
          <div key={sf.id} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                <Globe className="h-5 w-5 text-[var(--color-accent)]" />
                <div>
                  <h3 className="font-medium text-[var(--color-ink)]">{sf.title}</h3>
                  <p className="text-xs text-[var(--color-ink-muted)]">/{sf.slug}</p>
                </div>
              </div>
              <div className="flex items-center gap-1">
                <a href={`/storefront/${sf.slug}`} target="_blank" rel="noopener noreferrer"
                  className="p-1 text-[var(--color-accent)] hover:bg-[var(--color-surface-hover)] rounded">
                  <ExternalLink className="h-4 w-4" />
                </a>
                <button onClick={() => deleteStorefront(sf.id)} className="p-1 text-red-500 hover:bg-red-50 rounded">
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            </div>
            <div className="mt-3 flex items-center gap-2">
              <button onClick={() => toggleActive(sf)}
                className={`rounded-full px-2 py-0.5 text-xs font-medium ${sf.isActive ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'}`}>
                {sf.isActive ? 'Active' : 'Inactive'}
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
