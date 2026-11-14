import { useEffect, useState } from 'react'
import { RefreshCw, Plus, X, Check, Trash2, Play, Database, ArrowRightLeft } from '../components/icons'
import { api, type SyncConnection, type SyncLog } from '../lib/api'

const PLATFORMS = ['shopify', 'woocommerce', 'wix', 'square']

export function SyncSettings() {
  const [connections, setConnections] = useState<SyncConnection[]>([])
  const [logs, setLogs] = useState<SyncLog[]>([])
  const [mappings, setMappings] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [syncingId, setSyncingId] = useState<string | null>(null)
  const [syncingAll, setSyncingAll] = useState(false)
  const [showMappings, setShowMappings] = useState<string | null>(null)
  const [platform, setPlatform] = useState('shopify')
  const [storeUrl, setStoreUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const brandId = api.getBrandId() || ''

  useEffect(() => { load() }, [brandId])

  async function load() {
    try {
      const [conns, logEntries] = await Promise.all([
        api.listSyncConnections(brandId).catch(() => []),
        api.listSyncLogs().catch(() => []),
      ])
      setConnections(conns)
      setLogs(logEntries)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    try {
      const creds: Record<string, any> = {}
      if (platform === 'shopify') {
        creds.shop = storeUrl.replace(/\.myshopify\.com$/, '').replace(/^https?:\/\//, '').split('/')[0]
        creds.api_password = apiKey
      } else if (platform === 'woocommerce') {
        creds.store_url = storeUrl
        creds.consumer_key = apiKey
        creds.consumer_secret = ''
      } else if (platform === 'wix') {
        creds.api_key = apiKey
      } else if (platform === 'square') {
        creds.access_token = apiKey
        creds.location_id = storeUrl
      }
      await api.createSyncConnection({ brandId, platform, credentials: creds })
      setStoreUrl('')
      setApiKey('')
      setShowForm(false)
      setSuccess(`Connection to ${platform} added.`)
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function triggerSync(id: string) {
    setSyncingId(id)
    try {
      await api.triggerSync(id)
      setSuccess('Sync completed.')
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncingId(null)
    }
  }

  async function syncAll() {
    setSyncingAll(true)
    try {
      await api.syncAllPlatforms(brandId)
      setSuccess('All platforms synced.')
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncingAll(false)
    }
  }

  async function processQueue() {
    try {
      const result = await api.processSyncQueue()
      setSuccess(`Processed ${result.processed || 0} queued sync tasks.`)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function loadMappings(connectionId: string) {
    try {
      const data = await api.listProductMappings(connectionId)
      setMappings(data)
      setShowMappings(connectionId)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function deleteConnection(id: string) {
    if (!confirm('Delete this connection?')) return
    try {
      await api.deleteSyncConnection(id)
      setSuccess('Connection removed.')
      setShowMappings(null)
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading sync settings...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Sync Settings</h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">Connect external e-commerce platforms for bidirectional sync</p>
        </div>
        <div className="flex gap-2 shrink-0">
          {connections.length > 0 && (
            <button onClick={syncAll} disabled={syncingAll}
              className="shrink-0 flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 shadow-none">
              <ArrowRightLeft className={`h-4 w-4 ${syncingAll ? 'animate-spin' : ''}`} /> Sync All
            </button>
          )}
          <button onClick={() => setShowForm((v) => !v)}
            className="shrink-0 flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] shadow-none">
            {showForm ? <><X className="h-4 w-4" /> Cancel</> : <><Plus className="h-4 w-4" /> Add connection</>}
          </button>
        </div>
      </div>

      {error && <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
        <span>{error}</span><button onClick={() => setError(null)}><X className="h-4 w-4" /></button>
      </div>}
      {success && <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
        <span className="flex items-center gap-1.5"><Check className="h-4 w-4" /> {success}</span>
        <button onClick={() => setSuccess(null)}><X className="h-4 w-4" /></button>
      </div>}

      {/* How it works */}
      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
        <h2 className="text-sm font-bold text-[var(--color-ink)] mb-3">How Sync Works</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
          <div className="rounded-lg border border-[var(--color-border)] p-4">
            <div className="flex items-center gap-2 mb-2">
              <Database className="h-4 w-4 text-[var(--color-accent)]" />
            </div>
            <p className="text-[var(--color-ink-muted)]">When you create or update a product, it automatically syncs to all connected platforms. External products can be imported by running a sync.</p>
          </div>
          <div className="rounded-lg border border-[var(--color-border)] p-4">
            <div className="flex items-center gap-2 mb-2">
              <ArrowRightLeft className="h-4 w-4 text-[var(--color-accent)]" />
              <span className="font-semibold text-[var(--color-ink)]">Orders</span>
            </div>
            <p className="text-[var(--color-ink-muted)]">Orders from WhatsApp, Instagram, etc. are pushed to connected platforms. Orders placed on external platforms are imported into our system.</p>
          </div>
          <div className="rounded-lg border border-[var(--color-border)] p-4">
            <div className="flex items-center gap-2 mb-2">
              <RefreshCw className="h-4 w-4 text-[var(--color-accent)]" />
              <span className="font-semibold text-[var(--color-ink)]">Inventory</span>
            </div>
            <p className="text-[var(--color-ink-muted)]">Inventory changes are synced in real-time. When stock changes on any platform, all others are updated to stay in sync.</p>
          </div>
        </div>
      </div>

      {showForm && (
        <form onSubmit={submit} className="space-y-4 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Connect platform</h2>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Platform</label>
              <select value={platform} onChange={(e) => setPlatform(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]">
                {PLATFORMS.map((p) => <option key={p} value={p}>{p}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
                {platform === 'shopify' ? 'Shop URL' : platform === 'square' ? 'Location ID' : 'Store URL'}
              </label>
              <input required value={storeUrl} onChange={(e) => setStoreUrl(e.target.value)}
                placeholder={platform === 'shopify' ? 'my-store.myshopify.com' : platform === 'square' ? 'LOC_abc123' : 'https://mystore.com'}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">
                {platform === 'shopify' ? 'API Password' : platform === 'woocommerce' ? 'Consumer Key' : 'API Key / Token'}
              </label>
              <input required value={apiKey} onChange={(e) => setApiKey(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
          </div>
          <button type="submit" className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
            Connect
          </button>
        </form>
      )}

      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
        {connections.length === 0 ? (
          <p className="p-5 text-sm text-[var(--color-ink-muted)]">No connections yet. Add one to start syncing.</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                <th className="px-5 py-2">Platform</th>
                <th className="px-5 py-2">Status</th>
                <th className="px-5 py-2">Last Sync</th>
                <th className="px-5 py-2">Actions</th>
              </tr>
            </thead>
            <tbody>
              {connections.map((c) => (
                <tr key={c.id} className="border-b border-[var(--color-border)] last:border-0">
                  <td className="px-5 py-3 font-medium capitalize">{c.platform}</td>
                  <td className="px-5 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      c.syncStatus === 'connected' ? 'bg-green-100 text-green-700' :
                      c.syncStatus === 'syncing' ? 'bg-blue-100 text-blue-700' :
                      c.syncStatus === 'error' ? 'bg-red-100 text-red-700' :
                      'bg-[var(--color-accent-muted)] text-[var(--color-ink)]'
                    }`}>{c.syncStatus || 'disconnected'}</span>
                  </td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)]">
                    {c.lastSyncAt ? new Date(c.lastSyncAt).toLocaleString() : 'Never'}
                  </td>
                  <td className="px-5 py-3 flex gap-2">
                    <button onClick={() => triggerSync(c.id)} disabled={syncingId === c.id}
                      title="Sync this platform"
                      className="text-blue-600 hover:text-blue-800 disabled:opacity-50">
                      <RefreshCw className={`h-4 w-4 ${syncingId === c.id ? 'animate-spin' : ''}`} />
                    </button>
                    <button onClick={() => loadMappings(c.id)}
                      title="View product mappings"
                      className="text-[var(--color-accent)] hover:text-[var(--color-accent-hover)]">
                      <Database className="h-4 w-4" />
                    </button>
                    <button onClick={() => deleteConnection(c.id)} title="Delete connection"
                      className="text-red-600 hover:text-red-800">
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Product Mappings */}
      {showMappings && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm">
          <div className="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-3">
            <h2 className="text-sm font-bold text-[var(--color-ink)]">Product Mappings ({mappings.length})</h2>
            <button onClick={() => setShowMappings(null)}><X className="h-4 w-4" /></button>
          </div>
          {mappings.length === 0 ? (
            <p className="p-5 text-sm text-[var(--color-ink-muted)]">No product mappings yet. Run a sync to create mappings.</p>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                  <th className="px-5 py-2">Product</th>
                  <th className="px-5 py-2">SKU</th>
                  <th className="px-5 py-2">External ID</th>
                  <th className="px-5 py-2">Price</th>
                  <th className="px-5 py-2">Inventory</th>
                  <th className="px-5 py-2">Last Synced</th>
                </tr>
              </thead>
              <tbody>
                {mappings.map((m) => (
                  <tr key={m.mappingId} className="border-b border-[var(--color-border)] last:border-0">
                    <td className="px-5 py-3 font-medium">{m.productName || '-'}</td>
                    <td className="px-5 py-3 text-[var(--color-ink-muted)]">{m.productSku || '-'}</td>
                    <td className="px-5 py-3 font-mono text-xs">{m.externalId}</td>
                    <td className="px-5 py-3">${m.price?.toFixed(2) || '0.00'}</td>
                    <td className="px-5 py-3">{m.inventory ?? '-'}</td>
                    <td className="px-5 py-3 text-[var(--color-ink-muted)]">
                      {m.lastSyncedAt ? new Date(m.lastSyncedAt).toLocaleString() : 'Never'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {/* Sync Queue */}
      {connections.length > 0 && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm p-5">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-bold text-[var(--color-ink)]">Sync Queue</h2>
              <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">Auto-queued sync tasks from product/order changes</p>
            </div>
            <button onClick={processQueue}
              className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
              <Play className="h-3 w-3" /> Process Queue
            </button>
          </div>
        </div>
      )}

      {logs.length > 0 && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm">
          <div className="border-b border-[var(--color-border)] px-5 py-3">
            <h2 className="text-sm font-bold text-[var(--color-ink)]">Sync Logs</h2>
          </div>
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] text-left text-xs uppercase tracking-wider text-[var(--color-ink-muted)]">
                <th className="px-5 py-2">Direction</th>
                <th className="px-5 py-2">Entity</th>
                <th className="px-5 py-2">Status</th>
                <th className="px-5 py-2">Error</th>
                <th className="px-5 py-2">Time</th>
              </tr>
            </thead>
            <tbody>
              {logs.slice(0, 20).map((l) => (
                <tr key={l.id} className="border-b border-[var(--color-border)] last:border-0">
                  <td className="px-5 py-3 capitalize">{l.direction}</td>
                  <td className="px-5 py-3">{l.entityType}</td>
                  <td className="px-5 py-3">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      l.status === 'success' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
                    }`}>{l.status}</span>
                  </td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)] max-w-[200px] truncate">{l.error || '-'}</td>
                  <td className="px-5 py-3 text-[var(--color-ink-muted)]">
                    {l.createdAt ? new Date(l.createdAt).toLocaleString() : '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
