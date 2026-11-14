import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Building2,
  Plus,
  RefreshCw,
  ExternalLink,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Clock,
  ArrowRight,
} from 'lucide-react'

interface NidTenant {
  id: string
  nid_tenant_id: string
  name: string
  slug: string
  data_residency_region: string
  webhook_url: string | null
  sync_status: string
  last_synced_at: string | null
  last_sync_error: string
  linked_brand_id: string | null
  created_at: string
  updated_at: string
}

interface NidConnection {
  id: string
  nID_base_url: string
  auto_sync_enabled: boolean
  auto_link_new_tenants: boolean
  sync_interval_seconds: number
  connection_status: string
  last_error: string
}

interface ConnectionHealth {
  connection_status: string
  nID_base_url: string
  auto_sync_enabled: boolean
  total_tenants: number
  synced_tenants: number
  error_tenants: number
  last_error: string
}

const API_BASE = ''

export function TenantDashboard() {
  const navigate = useNavigate()
  const [tenants, setTenants] = useState<NidTenant[]>([])
  const [connection, setConnection] = useState<NidConnection | null>(null)
  const [health, setHealth] = useState<ConnectionHealth | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [redirecting, setRedirecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load tenants and connection status
  useEffect(() => {
    Promise.all([
      fetchTenants(),
      fetchConnection(),
      fetchHealth(),
    ]).finally(() => setLoading(false))
  }, [])

  const fetchTenants = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/nid/tenants`)
      if (!res.ok) throw new Error('Failed to fetch tenants')
      const data = await res.json()
      setTenants(data || [])
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tenants')
    }
  }

  const fetchConnection = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/nid/connection`)
      if (!res.ok) throw new Error('Failed to fetch connection')
      const data = await res.json()
      setConnection(data)
    } catch (err) {
      console.error('Failed to load connection:', err)
    }
  }

  const fetchHealth = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/nid/tenants`)
      if (!res.ok) throw new Error('Failed to fetch health')
      const data = await res.json()
      setHealth({
        connection_status: connection?.connection_status || 'unknown',
        nID_base_url: connection?.nID_base_url || '',
        auto_sync_enabled: connection?.auto_sync_enabled ?? false,
        total_tenants: data?.length || 0,
        synced_tenants: data?.filter((t: NidTenant) => t.sync_status === 'synced').length || 0,
        error_tenants: data?.filter((t: NidTenant) => t.sync_status === 'error').length || 0,
        last_error: connection?.last_error || '',
      })
    } catch (err) {
      console.error('Failed to load health:', err)
    }
  }

  const handleRefresh = async () => {
    setRefreshing(true)
    await Promise.all([fetchTenants(), fetchConnection(), fetchHealth()])
    setRefreshing(false)
  }

  const handleCreateTenant = async () => {
    setRedirecting(true)
    try {
      const return_url = `${window.location.origin}/tenants`
      const res = await fetch(`${API_BASE}/api/v1/nid/redirect/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          redirect_type: 'create',
          return_url,
        }),
      })

      if (!res.ok) throw new Error('Failed to create redirect session')

      const session = await res.json()

      // Redirect to nID tenant creation page
      const nidBaseUrl = connection?.nID_base_url || 'http://localhost:4000'
      const nidUrl = `${nidBaseUrl}/dashboard?redirect_state=${session.state_token}&return_url=${encodeURIComponent(return_url)}`
      window.location.href = nidUrl
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start tenant creation')
      setRedirecting(false)
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'synced':
        return <CheckCircle size={16} className="text-green-500" />
      case 'error':
        return <XCircle size={16} className="text-red-500" />
      case 'deleted':
        return <XCircle size={16} className="text-gray-400" />
      default:
        return <Clock size={16} className="text-yellow-500" />
    }
  }

  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      synced: 'bg-green-100 text-green-700',
      error: 'bg-red-100 text-red-700',
      deleted: 'bg-gray-100 text-gray-600',
      pending: 'bg-yellow-100 text-yellow-700',
      out_of_sync: 'bg-orange-100 text-orange-700',
    }
    return (
      <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${styles[status] || styles.pending}`}>
        {getStatusIcon(status)}
        {status}
      </span>
    )
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-gray-400 mb-4" />
          <p className="text-gray-500">Loading tenant dashboard…</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">nID Tenant Management</h1>
          <p className="text-sm text-gray-500 mt-1">
            Manage your nID tenant connections, monitor sync status, and create new tenants.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={handleRefresh}
            disabled={refreshing}
            className="inline-flex items-center gap-2 px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            <RefreshCw size={16} className={refreshing ? 'animate-spin' : ''} />
            Refresh
          </button>
          <button
            onClick={handleCreateTenant}
            disabled={redirecting}
            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            {redirecting ? <RefreshCw size={16} className="animate-spin" /> : <Plus size={16} />}
            Create Tenant
          </button>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="flex items-center gap-3 p-4 bg-red-50 border border-red-200 rounded-lg">
          <AlertTriangle size={20} className="text-red-500 flex-shrink-0" />
          <p className="text-sm text-red-700">{error}</p>
        </div>
      )}

      {/* Connection Status */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Connection</p>
          <div className="mt-2 flex items-center gap-2">
            {connection?.connection_status === 'connected' ? (
              <CheckCircle size={18} className="text-green-500" />
            ) : (
              <XCircle size={18} className="text-red-500" />
            )}
            <p className="text-lg font-semibold capitalize">
              {connection?.connection_status || 'disconnected'}
            </p>
          </div>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Total Tenants</p>
          <p className="mt-2 text-lg font-semibold">{health?.total_tenants || 0}</p>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Synced</p>
          <p className="mt-2 text-lg font-semibold text-green-600">{health?.synced_tenants || 0}</p>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Errors</p>
          <p className="mt-2 text-lg font-semibold text-red-600">{health?.error_tenants || 0}</p>
        </div>
      </div>

      {/* Connection Settings */}
      {connection && (
        <div className="bg-white border border-gray-200 rounded-lg p-6">
          <h2 className="text-sm font-semibold text-gray-900 mb-4">Connection Settings</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
            <div>
              <p className="text-gray-500">nID Base URL</p>
              <p className="font-mono text-gray-900 break-all">{connection.nID_base_url}</p>
            </div>
            <div>
              <p className="text-gray-500">Auto Sync</p>
              <p className="text-gray-900">
                {connection.auto_sync_enabled ? 'Enabled' : 'Disabled'}
              </p>
            </div>
            <div>
              <p className="text-gray-500">Sync Interval</p>
              <p className="text-gray-900">{connection.sync_interval_seconds}s</p>
            </div>
          </div>
          {connection.last_error && (
            <div className="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
              <p className="text-xs text-yellow-700">Last Error: {connection.last_error}</p>
            </div>
          )}
        </div>
      )}

      {/* Tenants List */}
      <div className="bg-white border border-gray-200 rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-sm font-semibold text-gray-900">Synced Tenants</h2>
        </div>

        {tenants.length === 0 ? (
          <div className="p-12 text-center">
            <Building2 size={48} className="mx-auto text-gray-300 mb-4" />
            <p className="text-gray-500 mb-4">No tenants synced from nID yet.</p>
            <button
              onClick={handleCreateTenant}
              className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700"
            >
              <Plus size={16} />
              Create Your First Tenant
            </button>
          </div>
        ) : (
          <div className="divide-y divide-gray-100">
            {tenants.map((tenant) => (
              <div key={tenant.id} className="px-6 py-4 hover:bg-gray-50 transition-colors">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center">
                      <Building2 size={20} className="text-blue-600" />
                    </div>
                    <div>
                      <p className="font-medium text-gray-900">{tenant.name}</p>
                      <p className="text-xs text-gray-500 font-mono">{tenant.slug}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-4">
                    {tenant.linked_brand_id && (
                      <span className="text-xs text-gray-400">
                        Linked to brand
                      </span>
                    )}
                    {getStatusBadge(tenant.sync_status)}
                    {tenant.webhook_url && (
                      <a
                        href={tenant.webhook_url}
                        target="_blank"
                        rel="noreferrer"
                        className="text-gray-400 hover:text-gray-600"
                      >
                        <ExternalLink size={16} />
                      </a>
                    )}
                  </div>
                </div>
                {tenant.last_sync_error && (
                  <div className="mt-2 ml-14">
                    <p className="text-xs text-red-600">{tenant.last_sync_error}</p>
                  </div>
                )}
                {tenant.last_synced_at && (
                  <p className="mt-1 ml-14 text-xs text-gray-400">
                    Last synced: {new Date(tenant.last_synced_at).toLocaleString()}
                  </p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* How it works */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
        <h3 className="text-sm font-semibold text-blue-900 mb-3">How Tenant Creation Works</h3>
        <ol className="space-y-2 text-sm text-blue-800">
          <li className="flex items-start gap-2">
            <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-200 text-blue-800 text-xs flex items-center justify-center font-medium">1</span>
            <span>Click "Create Tenant" — you'll be redirected to the nID dashboard.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-200 text-blue-800 text-xs flex items-center justify-center font-medium">2</span>
            <span>Fill in the tenant details (name, slug, webhook URL) in nID.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-200 text-blue-800 text-xs flex items-center justify-center font-medium">3</span>
            <span>After creation, you'll be redirected back here automatically.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-200 text-blue-800 text-xs flex items-center justify-center font-medium">4</span>
            <span>The new tenant appears in the list above via real-time webhook sync.</span>
          </li>
        </ol>
      </div>
    </div>
  )
}
