import { useState, useEffect } from 'react'
import {
  getFulfilmentSummary,
  listShippingZones,
  createShippingZone,
  deleteShippingZone,
  listShippingMethods,
  createShippingMethod,
  deleteShippingMethod,
  listShippingRates,
  deleteShippingRate,
  listReturnReasons,
  listReturnRequests,
  getReturnRequest,
  updateReturnStatus,
  getReturnStats,
  listManifests,
  dispatchManifest,
  type FulfilmentSummary,
  type ErpShippingZone,
  type ErpShippingMethod,
  type ErpShippingRate,
  type ErpReturnRequest,
  type ErpManifest,
} from '../lib/api'
import {
  Package, Truck, ArrowLeftRight, FileText, Plus, Trash2, Play, Search,
  ChevronDown, ChevronUp, CheckCircle, XCircle, Clock, DollarSign
} from '../components/icons'

type Tab = 'overview' | 'shipping' | 'returns' | 'manifests'

const statusColors: Record<string, string> = {
  REQUESTED: 'bg-yellow-100 text-yellow-800',
  APPROVED: 'bg-blue-100 text-blue-800',
  REJECTED: 'bg-red-100 text-red-800',
  IN_TRANSIT: 'bg-purple-100 text-purple-800',
  RECEIVED: 'bg-indigo-100 text-indigo-800',
  INSPECTED: 'bg-cyan-100 text-cyan-800',
  REFUNDED: 'bg-green-100 text-green-800',
  CLOSED: 'bg-gray-100 text-gray-800',
  OPEN: 'bg-yellow-100 text-yellow-800',
  DISPATCHED: 'bg-green-100 text-green-800',
}

export default function FulfillmentDashboard() {
  const [tab, setTab] = useState<Tab>('overview')
  const [summary, setSummary] = useState<FulfilmentSummary | null>(null)
  const [loading, setLoading] = useState(true)

  // Shipping state
  const [zones, setZones] = useState<ErpShippingZone[]>([])
  const [methods, setMethods] = useState<ErpShippingMethod[]>([])
  const [rates, setRates] = useState<ErpShippingRate[]>([])
  const [newZoneKey, setNewZoneKey] = useState('')
  const [newMethodKey, setNewMethodKey] = useState('')

  // Returns state
  const [returns, setReturns] = useState<ErpReturnRequest[]>([])
  const [returnFilter, setReturnFilter] = useState('')
  const [returnStats, setReturnStats] = useState<Record<string, unknown>>({})
  const [selectedReturn, setSelectedReturn] = useState<ErpReturnRequest | null>(null)
  const [returnDetail, setReturnDetail] = useState<{ request: ErpReturnRequest; items: any[] } | null>(null)

  // Manifests state
  const [manifests, setManifests] = useState<ErpManifest[]>([])

  useEffect(() => {
    loadSummary()
  }, [])

  useEffect(() => {
    if (tab === 'shipping') loadShipping()
    else if (tab === 'returns') loadReturns()
    else if (tab === 'manifests') loadManifests()
  }, [tab])

  async function loadSummary() {
    try {
      const data = await getFulfilmentSummary()
      setSummary(data)
    } catch (e) {
      console.error('Failed to load fulfilment summary:', e)
    }
    setLoading(false)
  }

  async function loadShipping() {
    try {
      const [z, m, r] = await Promise.all([
        listShippingZones(false),
        listShippingMethods(false),
        listShippingRates(),
      ])
      setZones(z)
      setMethods(m)
      setRates(r)
    } catch (e) {
      console.error('Failed to load shipping data:', e)
    }
  }

  async function loadReturns() {
    try {
      const [r, s] = await Promise.all([
        listReturnRequests(returnFilter || undefined),
        getReturnStats(),
      ])
      setReturns(r)
      setReturnStats(s)
    } catch (e) {
      console.error('Failed to load returns:', e)
    }
  }

  async function loadManifests() {
    try {
      const m = await listManifests()
      setManifests(m)
    } catch (e) {
      console.error('Failed to load manifests:', e)
    }
  }

  async function handleCreateZone() {
    if (!newZoneKey.trim()) return
    try {
      await createShippingZone({ key: newZoneKey.trim() })
      setNewZoneKey('')
      loadShipping()
    } catch (e) {
      console.error('Failed to create zone:', e)
    }
  }

  async function handleDeleteZone(id: string) {
    try {
      await deleteShippingZone(id)
      loadShipping()
    } catch (e) {
      console.error('Failed to delete zone:', e)
    }
  }

  async function handleCreateMethod() {
    if (!newMethodKey.trim()) return
    try {
      await createShippingMethod({ key: newMethodKey.trim() })
      setNewMethodKey('')
      loadShipping()
    } catch (e) {
      console.error('Failed to create method:', e)
    }
  }

  async function handleDeleteMethod(id: string) {
    try {
      await deleteShippingMethod(id)
      loadShipping()
    } catch (e) {
      console.error('Failed to delete method:', e)
    }
  }

  async function handleDeleteRate(id: string) {
    try {
      await deleteShippingRate(id)
      loadShipping()
    } catch (e) {
      console.error('Failed to delete rate:', e)
    }
  }

  async function handleViewReturn(id: string) {
    try {
      const data = await getReturnRequest(id)
      setReturnDetail(data)
      setSelectedReturn(data.request)
    } catch (e) {
      console.error('Failed to load return detail:', e)
    }
  }

  async function handleUpdateReturnStatus(id: string, status: string) {
    try {
      await updateReturnStatus(id, { status })
      setSelectedReturn(null)
      setReturnDetail(null)
      loadReturns()
    } catch (e) {
      console.error('Failed to update return status:', e)
    }
  }

  async function handleDispatchManifest(id: string) {
    try {
      await dispatchManifest(id)
      loadManifests()
    } catch (e) {
      console.error('Failed to dispatch manifest:', e)
    }
  }

  if (loading) {
    return <div className="p-8 text-center text-gray-500">Loading...</div>
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">ERP Fulfillment</h1>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-100 rounded-lg p-1">
        {([
          { key: 'overview', label: 'Overview', icon: Package },
          { key: 'shipping', label: 'Shipping', icon: Truck },
          { key: 'returns', label: 'Returns / RMA', icon: ArrowLeftRight },
          { key: 'manifests', label: 'Manifests', icon: FileText },
        ] as { key: Tab; label: string; icon: any }[]).map((t) => (
          <button
            key={t.key}
            onClick={() => setTab(t.key)}
            className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              tab === t.key ? 'bg-white shadow text-gray-900' : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <t.icon className="w-4 h-4" />
            {t.label}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {tab === 'overview' && summary && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            <StatCard title="Shipping Zones" value={summary.total_zones} subtitle={`${summary.active_zones} active`} icon={Package} />
            <StatCard title="Shipping Methods" value={summary.total_methods} subtitle={`${summary.active_methods} active`} icon={Truck} />
            <StatCard title="Shipping Rates" value={summary.total_rates} icon={DollarSign} />
            <StatCard title="Return Requests" value={summary.total_returns} subtitle={`${summary.pending_returns} pending`} icon={ArrowLeftRight} color="text-orange-600" />
            <StatCard title="Manifests" value={summary.total_manifests} subtitle={`${summary.open_manifests} open`} icon={FileText} />
          </div>

          <div className="bg-white rounded-lg border p-6">
            <h3 className="font-semibold mb-4">Return Statistics</h3>
            <div className="grid grid-cols-3 gap-4">
              <div className="text-center p-4 bg-gray-50 rounded-lg">
                <div className="text-2xl font-bold">{returnStats.totalReturns || 0}</div>
                <div className="text-sm text-gray-500">Total Returns</div>
              </div>
              <div className="text-center p-4 bg-yellow-50 rounded-lg">
                <div className="text-2xl font-bold text-yellow-600">{returnStats.pendingReturns || 0}</div>
                <div className="text-sm text-gray-500">Pending</div>
              </div>
              <div className="text-center p-4 bg-green-50 rounded-lg">
                <div className="text-2xl font-bold text-green-600">{(Number(returnStats.refundedAmount || 0) / 100).toFixed(2)}</div>
                <div className="text-sm text-gray-500">Refunded Amount</div>
              </div>
            </div>
          </div>

          <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
            <h3 className="font-semibold text-blue-900 mb-2">How It Works</h3>
            <ul className="text-sm text-blue-800 space-y-1">
              <li><strong>Shipping Zones</strong> — Define geographic areas for delivery</li>
              <li><strong>Shipping Methods</strong> — Configure courier services and delivery times</li>
              <li><strong>Shipping Rates</strong> — Set pricing rules per zone and method</li>
              <li><strong>Returns/RMA</strong> — Manage customer return requests with full lifecycle tracking</li>
              <li><strong>Manifests</strong> — Group shipments by courier for dispatch</li>
            </ul>
          </div>
        </div>
      )}

      {/* Shipping Tab */}
      {tab === 'shipping' && (
        <div className="space-y-6">
          {/* Zones */}
          <div className="bg-white rounded-lg border p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold flex items-center gap-2">
                <Package className="w-4 h-4" /> Shipping Zones ({zones.length})
              </h3>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newZoneKey}
                  onChange={(e) => setNewZoneKey(e.target.value)}
                  placeholder="Zone key (e.g. metro)"
                  className="border rounded-md px-3 py-1 text-sm"
                  onKeyDown={(e) => e.key === 'Enter' && handleCreateZone()}
                />
                <button onClick={handleCreateZone} className="flex items-center gap-1 bg-blue-600 text-white px-3 py-1 rounded-md text-sm hover:bg-blue-700">
                  <Plus className="w-3 h-3" /> Add
                </button>
              </div>
            </div>
            {zones.length === 0 ? (
              <p className="text-sm text-gray-400 py-4">No shipping zones configured</p>
            ) : (
              <div className="space-y-2">
                {zones.map((z) => (
                  <div key={z.id} className="flex items-center justify-between p-3 bg-gray-50 rounded-md">
                    <div>
                      <span className="font-medium text-sm">{z.key}</span>
                      <span className={`ml-2 text-xs px-2 py-0.5 rounded-full ${z.is_active ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500'}`}>
                        {z.is_active ? 'Active' : 'Inactive'}
                      </span>
                      {z.is_catch_all && <span className="ml-1 text-xs px-2 py-0.5 rounded-full bg-purple-100 text-purple-700">Catch-all</span>}
                    </div>
                    <button onClick={() => handleDeleteZone(z.id)} className="text-red-400 hover:text-red-600">
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Methods */}
          <div className="bg-white rounded-lg border p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold flex items-center gap-2">
                <Truck className="w-4 h-4" /> Shipping Methods ({methods.length})
              </h3>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newMethodKey}
                  onChange={(e) => setNewMethodKey(e.target.value)}
                  placeholder="Method key (e.g. standard)"
                  className="border rounded-md px-3 py-1 text-sm"
                  onKeyDown={(e) => e.key === 'Enter' && handleCreateMethod()}
                />
                <button onClick={handleCreateMethod} className="flex items-center gap-1 bg-blue-600 text-white px-3 py-1 rounded-md text-sm hover:bg-blue-700">
                  <Plus className="w-3 h-3" /> Add
                </button>
              </div>
            </div>
            {methods.length === 0 ? (
              <p className="text-sm text-gray-400 py-4">No shipping methods configured</p>
            ) : (
              <div className="space-y-2">
                {methods.map((m) => (
                  <div key={m.id} className="flex items-center justify-between p-3 bg-gray-50 rounded-md">
                    <div>
                      <span className="font-medium text-sm">{m.key}</span>
                      {m.courier_key && <span className="ml-2 text-xs text-gray-500">via {m.courier_key}</span>}
                      <span className="ml-2 text-xs text-gray-400">{m.eta_min_days}-{m.eta_max_days} days</span>
                      <span className={`ml-2 text-xs px-2 py-0.5 rounded-full ${m.is_active ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500'}`}>
                        {m.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </div>
                    <button onClick={() => handleDeleteMethod(m.id)} className="text-red-400 hover:text-red-600">
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Rates */}
          <div className="bg-white rounded-lg border p-6">
            <h3 className="font-semibold flex items-center gap-2 mb-4">
              <DollarSign className="w-4 h-4" /> Shipping Rates ({rates.length})
            </h3>
            {rates.length === 0 ? (
              <p className="text-sm text-gray-400 py-4">No shipping rates configured</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left py-2 px-3">Zone</th>
                      <th className="text-left py-2 px-3">Method</th>
                      <th className="text-left py-2 px-3">Type</th>
                      <th className="text-right py-2 px-3">Fee</th>
                      <th className="text-center py-2 px-3">Status</th>
                      <th className="py-2 px-3"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {rates.map((r) => (
                      <tr key={r.id} className="border-b hover:bg-gray-50">
                        <td className="py-2 px-3">{r.zone_id.substring(0, 8)}</td>
                        <td className="py-2 px-3">{r.method_id.substring(0, 8)}</td>
                        <td className="py-2 px-3">
                          <span className="px-2 py-0.5 bg-gray-100 rounded text-xs">{r.rate_type}</span>
                        </td>
                        <td className="py-2 px-3 text-right">{(r.fee / 100).toFixed(2)}</td>
                        <td className="py-2 px-3 text-center">
                          <span className={`text-xs px-2 py-0.5 rounded-full ${r.is_active ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500'}`}>
                            {r.is_active ? 'Active' : 'Inactive'}
                          </span>
                        </td>
                        <td className="py-2 px-3">
                          <button onClick={() => handleDeleteRate(r.id)} className="text-red-400 hover:text-red-600">
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Returns Tab */}
      {tab === 'returns' && (
        <div className="space-y-6">
          {/* Stats */}
          <div className="grid grid-cols-3 gap-4">
            <div className="bg-white rounded-lg border p-4 text-center">
              <div className="text-2xl font-bold">{returnStats.totalReturns || 0}</div>
              <div className="text-sm text-gray-500">Total Returns</div>
            </div>
            <div className="bg-white rounded-lg border p-4 text-center">
              <div className="text-2xl font-bold text-yellow-600">{returnStats.pendingReturns || 0}</div>
              <div className="text-sm text-gray-500">Pending</div>
            </div>
            <div className="bg-white rounded-lg border p-4 text-center">
              <div className="text-2xl font-bold text-green-600">{(Number(returnStats.refundedAmount || 0) / 100).toFixed(2)}</div>
              <div className="text-sm text-gray-500">Refunded</div>
            </div>
          </div>

          {/* Filter */}
          <div className="flex gap-2 flex-wrap">
            <button
              onClick={() => setReturnFilter('')}
              className={`px-3 py-1 rounded-full text-sm ${!returnFilter ? 'bg-gray-800 text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}`}
            >
              All
            </button>
            {['REQUESTED', 'APPROVED', 'IN_TRANSIT', 'RECEIVED', 'REFUNDED', 'CLOSED'].map((s) => (
              <button
                key={s}
                onClick={() => setReturnFilter(s)}
                className={`px-3 py-1 rounded-full text-sm ${returnFilter === s ? 'bg-gray-800 text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}`}
              >
                {s}
              </button>
            ))}
          </div>

          {/* Return Detail Modal */}
          {returnDetail && (
            <div className="bg-white rounded-lg border p-6 mb-4">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-semibold">Return #{returnDetail.request.number}</h3>
                <button onClick={() => { setSelectedReturn(null); setReturnDetail(null) }} className="text-gray-400 hover:text-gray-600">
                  <XCircle className="w-5 h-5" />
                </button>
              </div>
              <div className="grid grid-cols-2 gap-4 text-sm mb-4">
                <div>
                  <span className="text-gray-500">Status:</span>{' '}
                  <span className={`px-2 py-0.5 rounded-full text-xs ${statusColors[returnDetail.request.status] || 'bg-gray-100 text-gray-600'}`}>
                    {returnDetail.request.status}
                  </span>
                </div>
                <div><span className="text-gray-500">Order:</span> {returnDetail.request.order_id.substring(0, 12)}</div>
                <div><span className="text-gray-500">Requested:</span> {new Date(returnDetail.request.requested_at).toLocaleDateString()}</div>
                {returnDetail.request.resolution && (
                  <div><span className="text-gray-500">Resolution:</span> {returnDetail.request.resolution}</div>
                )}
                {returnDetail.request.customer_note && (
                  <div><span className="text-gray-500">Customer Note:</span> {returnDetail.request.customer_note}</div>
                )}
              </div>
              {returnDetail.items.length > 0 && (
                <div>
                  <h4 className="font-medium text-sm mb-2">Items</h4>
                  <div className="space-y-2">
                    {returnDetail.items.map((item: any) => (
                      <div key={item.id} className="flex items-center gap-4 p-2 bg-gray-50 rounded text-sm">
                        <span>Qty: {item.quantity}</span>
                        <span>Reason: {item.reason_id}</span>
                        {item.restocked && <span className="text-green-600">Restocked</span>}
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {returnDetail.request.status === 'REQUESTED' && (
                <div className="flex gap-2 mt-4">
                  <button
                    onClick={() => handleUpdateReturnStatus(returnDetail.request.id, 'APPROVED')}
                    className="flex items-center gap-1 bg-green-600 text-white px-3 py-1 rounded-md text-sm hover:bg-green-700"
                  >
                    <CheckCircle className="w-3 h-3" /> Approve
                  </button>
                  <button
                    onClick={() => handleUpdateReturnStatus(returnDetail.request.id, 'REJECTED')}
                    className="flex items-center gap-1 bg-red-600 text-white px-3 py-1 rounded-md text-sm hover:bg-red-700"
                  >
                    <XCircle className="w-3 h-3" /> Reject
                  </button>
                </div>
              )}
            </div>
          )}

          {/* Returns List */}
          <div className="bg-white rounded-lg border">
            <h3 className="font-semibold p-4 border-b">Return Requests ({returns.length})</h3>
            {returns.length === 0 ? (
              <p className="text-sm text-gray-400 p-4">No return requests</p>
            ) : (
              <div className="divide-y">
                {returns.map((r) => (
                  <div key={r.id} className="p-4 hover:bg-gray-50 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                      <button onClick={() => handleViewReturn(r.id)} className="flex items-center gap-2">
                        <FileText className="w-4 h-4 text-gray-400" />
                        <div>
                          <div className="font-medium text-sm">#{r.number}</div>
                          <div className="text-xs text-gray-400">Order: {r.order_id.substring(0, 12)} · {new Date(r.requested_at).toLocaleDateString()}</div>
                        </div>
                      </button>
                    </div>
                    <div className="flex items-center gap-2">
                      {r.refund_amount && (
                        <span className="text-sm text-green-600">{(r.refund_amount / 100).toFixed(2)}</span>
                      )}
                      <span className={`px-2 py-0.5 rounded-full text-xs ${statusColors[r.status] || 'bg-gray-100 text-gray-600'}`}>
                        {r.status}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Manifests Tab */}
      {tab === 'manifests' && (
        <div className="space-y-6">
          <div className="bg-white rounded-lg border">
            <h3 className="font-semibold p-4 border-b flex items-center gap-2">
              <FileText className="w-4 h-4" /> Manifests ({manifests.length})
            </h3>
            {manifests.length === 0 ? (
              <p className="text-sm text-gray-400 p-4">No manifests</p>
            ) : (
              <div className="divide-y">
                {manifests.map((m) => (
                  <div key={m.id} className="p-4 hover:bg-gray-50 flex items-center justify-between">
                    <div>
                      <div className="font-medium text-sm">{m.number}</div>
                      <div className="text-xs text-gray-400">
                        Courier: {m.courier_key} · {new Date(m.created_at).toLocaleDateString()}
                        {m.dispatched_at && ` · Dispatched: ${new Date(m.dispatched_at).toLocaleDateString()}`}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`px-2 py-0.5 rounded-full text-xs ${statusColors[m.status] || 'bg-gray-100 text-gray-600'}`}>
                        {m.status}
                      </span>
                      {m.status === 'OPEN' && (
                        <button
                          onClick={() => handleDispatchManifest(m.id)}
                          className="flex items-center gap-1 bg-green-600 text-white px-3 py-1 rounded-md text-sm hover:bg-green-700"
                        >
                          <Play className="w-3 h-3" /> Dispatch
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

function StatCard({ title, value, subtitle, icon: Icon, color = 'text-blue-600' }: {
  title: string
  value: number
  subtitle?: string
  icon: any
  color?: string
}) {
  return (
    <div className="bg-white rounded-lg border p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm text-gray-500">{title}</span>
        <Icon className={`w-5 h-5 ${color}`} />
      </div>
      <div className="text-2xl font-bold">{value}</div>
      {subtitle && <div className="text-xs text-gray-400 mt-1">{subtitle}</div>}
    </div>
  )
}
