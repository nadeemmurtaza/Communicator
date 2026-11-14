import { useEffect, useState } from 'react'
import { Warehouse, Plus, X, Check, Trash2, AlertTriangle } from '../components/icons'
import { api, type Warehouse as WarehouseType, type LowStockAlert } from '../lib/api'

export function InventoryManagement() {
  const [warehouses, setWarehouses] = useState<WarehouseType[]>([])
  const [alerts, setAlerts] = useState<LowStockAlert[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const brandId = api.getBrandId() || ''
  const [name, setName] = useState('')

  useEffect(() => { load() }, [brandId])

  async function load() {
    try {
      const [wh, al] = await Promise.all([
        api.listWarehouses(brandId).catch(() => []),
        api.listLowStockAlerts().catch(() => []),
      ])
      setWarehouses(wh)
      setAlerts(al)
    } catch (e) { setError((e as Error).message) }
    finally { setLoading(false) }
  }

  async function create(e: React.FormEvent) {
    e.preventDefault()
    try {
      await api.createWarehouse({ brandId, name })
      setShowForm(false); setName('')
      setSuccess('Warehouse created.')
      load()
    } catch (e) { setError((e as Error).message) }
  }

  async function deleteWarehouse(id: string) {
    if (!confirm('Delete this warehouse?')) return
    try { await api.deleteWarehouse(id); setSuccess('Deleted.'); load() }
    catch (e) { setError((e as Error).message) }
  }

  async function acknowledgeAlert(id: string) {
    try { await api.acknowledgeAlert(id); load() }
    catch (e) { setError((e as Error).message) }
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Inventory Management</h1>
          <p className="text-sm text-[var(--color-ink-muted)]">{warehouses.length} warehouses, {alerts.filter(a => !a.acknowledged).length} unacknowledged alerts</p>
        </div>
        <button onClick={() => setShowForm(true)}
          className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
          <Plus className="h-4 w-4" /> Add warehouse
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
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider">New Warehouse</h2>
          <div>
            <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">Name</label>
            <input required value={name} onChange={(e) => setName(e.target.value)}
              className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" placeholder="Main Warehouse" />
          </div>
          <div className="flex gap-2">
            <button type="submit" className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">Create</button>
            <button type="button" onClick={() => setShowForm(false)} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">Cancel</button>
          </div>
        </form>
      )}

      {/* Warehouses */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {warehouses.map((w) => (
          <div key={w.id} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                <Warehouse className="h-5 w-5 text-[var(--color-accent)]" />
                <div>
                  <h3 className="font-medium text-[var(--color-ink)]">{w.name}</h3>
                  <p className="text-xs text-[var(--color-ink-muted)]">{w.id.slice(0, 8)}</p>
                </div>
              </div>
              <button onClick={() => deleteWarehouse(w.id)} className="p-1 text-red-500 hover:bg-red-50 rounded">
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Low Stock Alerts */}
      {alerts.length > 0 && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)] uppercase tracking-wider mb-4 flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 text-amber-500" /> Low Stock Alerts
          </h2>
          <div className="space-y-2">
            {alerts.map((a) => (
              <div key={a.id} className={`flex items-center justify-between p-3 rounded-[var(--radius-md)] ${a.acknowledged ? 'bg-[var(--color-surface-hover)]' : 'bg-amber-50 border border-amber-200'}`}>
                <div>
                  <p className="text-sm font-medium text-[var(--color-ink)]">
                    Product {a.productId.slice(0, 8)} — Warehouse {a.warehouseId.slice(0, 8)}
                  </p>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    Stock: {a.currentQuantity} / Reorder at: {a.reorderPoint}
                  </p>
                </div>
                {!a.acknowledged && (
                  <button onClick={() => acknowledgeAlert(a.id)}
                    className="text-xs rounded-[var(--radius-sm)] border border-[var(--color-border)] px-2 py-1 text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">
                    Acknowledge
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
