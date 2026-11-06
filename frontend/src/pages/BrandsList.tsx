import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { api, type BrandItem } from '../lib/api'
import {
  Loader2, Plus, Palette, ExternalLink, Check,
  AlertTriangle, X, Search, Building2
} from '../components/icons'

export function BrandsList() {
  const navigate = useNavigate()
  const { organization, brand: selectedBrand, selectBrand } = useAuth()
  const [brands, setBrands] = useState<BrandItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [creating, setCreating] = useState(false)
  const [newName, setNewName] = useState('')
  const [newSlug, setNewSlug] = useState('')
  const [newDescription, setNewDescription] = useState('')

  useEffect(() => {
    loadBrands()
  }, [organization?.id])

  async function loadBrands() {
    try {
      setLoading(true)
      setError(null)
      const data = await api.listBrands()
      setBrands(data)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  async function handleCreate() {
    if (!newName.trim()) return
    if (!organization?.id) {
      setError('Please select an organization first')
      return
    }
    setCreating(true)
    try {
      const slug = newSlug.trim() || newName.trim().toLowerCase().replace(/\s+/g, '-')
      await api.createBrand({
        organizationId: organization.id,
        name: newName.trim(),
        slug,
        description: newDescription.trim(),
      })
      setShowCreateModal(false)
      setNewName('')
      setNewSlug('')
      setNewDescription('')
      await loadBrands()
    } catch (e: any) {
      setError(e.message)
    } finally {
      setCreating(false)
    }
  }

  async function handleSelect(brand: BrandItem) {
    try {
      selectBrand({
        id: brand.id,
        organizationId: brand.organizationId,
        name: brand.name,
        slug: brand.slug,
      })
      navigate('/workspace/brand/profile')
    } catch (e: any) {
      setError(e.message)
    }
  }

  const filteredBrands = brands.filter(b =>
    b.name.toLowerCase().includes(search.toLowerCase()) ||
    b.slug.toLowerCase().includes(search.toLowerCase())
  )

  if (showCreateModal) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
        <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">Create New Brand</h2>
            <button onClick={() => setShowCreateModal(false)}>
              <X className="w-5 h-5 text-muted-foreground" />
            </button>
          </div>

          {error && (
            <div className="flex items-center gap-2 mb-4 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
              <AlertTriangle className="w-4 h-4 shrink-0" />
              <span className="flex-1">{error}</span>
              <button onClick={() => setError(null)}><X className="w-4 h-4" /></button>
            </div>
          )}

          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Brand Name *</label>
              <input
                type="text"
                className="w-full mt-1 px-3 py-2 border rounded-lg text-sm"
                placeholder="My Brand"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                autoFocus
              />
            </div>
            <div>
              <label className="text-sm font-medium">Slug</label>
              <input
                type="text"
                className="w-full mt-1 px-3 py-2 border rounded-lg text-sm"
                placeholder="my-brand"
                value={newSlug}
                onChange={(e) => setNewSlug(e.target.value)}
              />
              <p className="text-xs text-muted-foreground mt-1">Auto-generated from name if left empty</p>
            </div>
            <div>
              <label className="text-sm font-medium">Description</label>
              <textarea
                className="w-full mt-1 px-3 py-2 border rounded-lg text-sm"
                rows={3}
                placeholder="Brief description of this brand..."
                value={newDescription}
                onChange={(e) => setNewDescription(e.target.value)}
              />
            </div>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-sm border rounded-lg hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={handleCreate}
                disabled={creating || !newName.trim()}
                className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-lg text-sm hover:opacity-90 disabled:opacity-50"
              >
                {creating ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
                Create Brand
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Brands</h1>
          <p className="text-muted-foreground text-sm mt-1">
            {organization ? `Manage brands under ${organization.name}` : 'Manage your brands'}
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-lg text-sm hover:opacity-90"
        >
          <Plus className="w-4 h-4" />
          New Brand
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span className="flex-1">{error}</span>
          <button onClick={() => setError(null)}><X className="w-4 h-4" /></button>
        </div>
      )}

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <input
          type="text"
          placeholder="Search brands..."
          className="w-full pl-10 pr-4 py-2 border rounded-lg text-sm"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {/* Loading */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
          <span className="ml-2 text-muted-foreground">Loading brands...</span>
        </div>
      )}

      {/* Brand List */}
      {!loading && filteredBrands.length === 0 && (
        <div className="text-center py-12">
          <Palette className="w-12 h-12 mx-auto text-muted-foreground/50" />
          <h3 className="mt-4 text-lg font-medium">No brands yet</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            Create your first brand to get started with BYOK and AI configuration.
          </p>
          <button
            onClick={() => setShowCreateModal(true)}
            className="mt-4 inline-flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-lg text-sm hover:opacity-90"
          >
            <Plus className="w-4 h-4" />
            Create Brand
          </button>
        </div>
      )}

      {/* Brand Cards */}
      {!loading && filteredBrands.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredBrands.map((b) => {
            const isSelected = selectedBrand?.id === b.id
            return (
              <div
                key={b.id}
                className={`group relative rounded-xl border p-5 transition-all cursor-pointer hover:shadow-md ${
                  isSelected
                    ? 'border-foreground bg-muted/50'
                    : 'border-border hover:border-foreground/30'
                }`}
                onClick={() => handleSelect(b)}
              >
                {isSelected && (
                  <div className="absolute top-3 right-3">
                    <Check className="w-5 h-5 text-foreground" />
                  </div>
                )}
                <div className="flex items-start gap-3">
                  <div className="w-10 h-10 rounded-lg bg-muted flex items-center justify-center shrink-0">
                    <Palette className="w-5 h-5" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <h3 className="font-semibold truncate">{b.name}</h3>
                    <p className="text-sm text-muted-foreground truncate">{b.slug}</p>
                    {b.description && (
                      <p className="text-xs text-muted-foreground mt-1 line-clamp-2">{b.description}</p>
                    )}
                  </div>
                </div>
                <div className="mt-4 flex items-center gap-2 text-xs text-muted-foreground">
                  <Building2 className="w-3 h-3" />
                  <span>Click to select & manage</span>
                </div>
                <div className="absolute inset-0 rounded-xl ring-2 ring-foreground/0 group-hover:ring-foreground/10 transition-all pointer-events-none" />
              </div>
            )
          })}
        </div>
      )}

      {/* Quick Actions */}
      {!loading && brands.length > 0 && (
        <div className="rounded-xl border p-4">
          <h3 className="text-sm font-medium mb-3">Quick Actions</h3>
          <div className="flex flex-wrap gap-2">
            <button
              onClick={() => navigate('/workspace/brand/profile')}
              className="flex items-center gap-2 px-3 py-1.5 text-sm border rounded-lg hover:bg-muted"
            >
              <Palette className="w-4 h-4" />
              Brand Profile
            </button>
            <button
              onClick={() => navigate('/workspace/brand/keys')}
              className="flex items-center gap-2 px-3 py-1.5 text-sm border rounded-lg hover:bg-muted"
            >
              <ExternalLink className="w-4 h-4" />
              Channel Keys
            </button>
            <button
              onClick={() => navigate('/workspace/brand/ai-config')}
              className="flex items-center gap-2 px-3 py-1.5 text-sm border rounded-lg hover:bg-muted"
            >
              <Building2 className="w-4 h-4" />
              AI Config
            </button>
            <button
              onClick={() => navigate('/workspace/brand/knowledge')}
              className="flex items-center gap-2 px-3 py-1.5 text-sm border rounded-lg hover:bg-muted"
            >
              <Search className="w-4 h-4" />
              Knowledge Base
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
