import { useEffect, useState } from 'react'
import {
  Building2,
  Plus,
  RefreshCw,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Clock,
  Shield,
} from 'lucide-react'

interface NidOrganization {
  id: string
  nid_organization_id: string
  nid_tenant_id: string
  tenant_slug: string
  legal_name: string
  registration_number: string | null
  country_code: string | null
  subject_id: string
  is_verified: boolean
  verified_level: string | null
  webhook_url: string | null
  sync_status: string
  last_synced_at: string | null
  last_sync_error: string
  linked_brand_id: string | null
  created_at: string
  updated_at: string
}

interface OrganizationConnection {
  id: string
  nID_base_url: string
  webhook_secret: string
  auto_sync_enabled: boolean
  auto_link_new_organizations: boolean
  sync_interval_seconds: number
  connection_status: string
  last_error: string
}

interface ConnectionHealth {
  connection_status: string
  nID_base_url: string
  auto_sync_enabled: boolean
  total_organizations: number
  synced_organizations: number
  error_organizations: number
  last_error: string
}

const API_BASE = ''

export function OrganizationDashboard() {
  const navigate = (url: string) => window.location.href = url
  const [organizations, setOrganizations] = useState<NidOrganization[]>([])
  const [connection, setConnection] = useState<OrganizationConnection | null>(null)
  const [health, setHealth] = useState<ConnectionHealth | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [redirecting, setRedirecting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showSettings, setShowSettings] = useState(false)

  useEffect(() => {
    Promise.all([
      fetchOrganizations(),
      fetchConnection(),
      fetchHealth(),
    ]).finally(() => setLoading(false))
  }, [])

  const fetchOrganizations = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/nid/org/organizations`)
      if (!res.ok) throw new Error('Failed to fetch organizations')
      const data = await res.json()
      setOrganizations(data || [])
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load organizations')
    }
  }

  const fetchConnection = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/nid/org/connection`)
      if (!res.ok) throw new Error('Failed to fetch connection')
      const data = await res.json()
      setConnection(data)
    } catch (err) {
      console.error('Failed to load connection:', err)
    }
  }

  const fetchHealth = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/nid/org/health`)
      if (!res.ok) throw new Error('Failed to fetch health')
      const data = await res.json()
      setHealth(data)
    } catch (err) {
      console.error('Failed to load health:', err)
    }
  }

  const handleRefresh = async () => {
    setRefreshing(true)
    await Promise.all([fetchOrganizations(), fetchConnection(), fetchHealth()])
    setRefreshing(false)
  }

  const handleCreateOrganization = async () => {
    setRedirecting(true)
    try {
      const return_url = `${window.location.origin}/organizations`
      const res = await fetch(`${API_BASE}/api/v1/nid/org/redirect/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          redirect_type: 'create',
          return_url,
        }),
      })

      if (!res.ok) throw new Error('Failed to create redirect session')

      const session = await res.json()
      window.location.href = session.redirect_url
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start organization creation')
      setRedirecting(false)
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'synced':
        return <CheckCircle size={16} className="text-green-500" />
      case 'error':
        return <XCircle size={16} className="text-red-500" />
      default:
        return <Clock size={16} className="text-yellow-500" />
    }
  }

  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      synced: 'bg-green-100 text-green-700',
      error: 'bg-red-100 text-red-700',
      pending: 'bg-yellow-100 text-yellow-700',
    }
    return (
      <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${styles[status] || styles.pending}`}>
        {getStatusIcon(status)}
        {status}
      </span>
    )
  }

  const getVerificationBadge = (org: NidOrganization) => {
    if (!org.is_verified) {
      return <span className="text-gray-400 text-xs">Not verified</span>
    }
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-700">
        <Shield size={12} />
        {org.verified_level || 'Verified'}
      </span>
    )
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <RefreshCw size={32} className="animate-spin mx-auto text-gray-400 mb-4" />
          <p className="text-gray-500">Loading organization dashboard…</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">nID Organization Management</h1>
          <p className="text-sm text-gray-500 mt-1">
            Manage your nID organizations, monitor sync status, and create new organizations.
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
            onClick={handleCreateOrganization}
            disabled={redirecting}
            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            {redirecting ? <RefreshCw size={16} className="animate-spin" /> : <Plus size={16} />}
            Create Organization
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

      {/* Health Cards */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Connection</p>
          <div className="mt-2 flex items-center gap-2">
            {health?.connection_status === 'connected' ? (
              <CheckCircle size={18} className="text-green-500" />
            ) : (
              <XCircle size={18} className="text-red-500" />
            )}
            <p className="text-lg font-semibold capitalize">
              {health?.connection_status || 'disconnected'}
            </p>
          </div>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Total</p>
          <p className="mt-2 text-lg font-semibold">{health?.total_organizations || 0}</p>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Synced</p>
          <p className="mt-2 text-lg font-semibold text-green-600">{health?.synced_organizations || 0}</p>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Errors</p>
          <p className="mt-2 text-lg font-semibold text-red-600">{health?.error_organizations || 0}</p>
        </div>
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <p className="text-xs text-gray-500 uppercase tracking-wider">Auto Sync</p>
          <p className="mt-2 text-lg font-semibold">
            {health?.auto_sync_enabled ? 'On' : 'Off'}
          </p>
        </div>
      </div>

      {/* Connection Settings */}
      {connection && (
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-700">Connection Settings</h3>
            <button
              onClick={() => setShowSettings(!showSettings)}
              className="text-xs text-blue-600 hover:underline"
            >
              {showSettings ? 'Hide' : 'Edit'}
            </button>
          </div>
          {showSettings ? (
            <div className="space-y-3">
              <div>
                <label className="block text-xs text-gray-500 mb-1">nID Base URL</label>
                <input
                  type="url"
                  defaultValue={connection.nID_base_url}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                  placeholder="http://localhost:4000"
                />
              </div>
              <div className="flex items-center gap-4">
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    defaultChecked={connection.auto_sync_enabled}
                    className="rounded"
                  />
                  Auto sync
                </label>
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    defaultChecked={connection.auto_link_new_organizations}
                    className="rounded"
                  />
                  Auto link new organizations
                </label>
              </div>
              <button className="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700">
                Save Settings
              </button>
            </div>
          ) : (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <p className="text-gray-500">nID URL</p>
                <p className="font-medium">{connection.nID_base_url}</p>
              </div>
              <div>
                <p className="text-gray-500">Status</p>
                <p className="font-medium capitalize">{connection.connection_status}</p>
              </div>
              <div>
                <p className="text-gray-500">Sync Interval</p>
                <p className="font-medium">{connection.sync_interval_seconds}s</p>
              </div>
              <div>
                <p className="text-gray-500">Last Error</p>
                <p className="font-medium text-red-600">{connection.last_error || 'None'}</p>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Organizations List */}
      <div className="bg-white border border-gray-200 rounded-lg">
        <div className="p-4 border-b border-gray-200">
          <h3 className="text-sm font-semibold text-gray-700">
            Organizations ({organizations.length})
          </h3>
        </div>

        {organizations.length === 0 ? (
          <div className="p-12 text-center">
            <Building2 size={48} className="mx-auto text-gray-300 mb-4" />
            <p className="text-gray-500 mb-2">No organizations synced yet</p>
            <p className="text-sm text-gray-400 mb-4">
              Create an organization in nID and it will automatically sync here.
            </p>
            <button
              onClick={handleCreateOrganization}
              className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700"
            >
              <Plus size={16} />
              Create Organization
            </button>
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {organizations.map((org) => (
              <div key={org.id} className="p-4 hover:bg-gray-50">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h4 className="font-semibold text-gray-900">{org.legal_name}</h4>
                      {getStatusBadge(org.sync_status)}
                      {getVerificationBadge(org)}
                    </div>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-sm text-gray-500">
                      <div>
                        <span className="text-gray-400">Tenant:</span> {org.tenant_slug}
                      </div>
                      <div>
                        <span className="text-gray-400">Country:</span> {org.country_code || '—'}
                      </div>
                      <div>
                        <span className="text-gray-400">Registration:</span> {org.registration_number || '—'}
                      </div>
                      <div>
                        <span className="text-gray-400">Synced:</span>{' '}
                        {org.last_synced_at ? new Date(org.last_synced_at).toLocaleString() : 'Never'}
                      </div>
                    </div>
                    {org.last_sync_error && (
                      <p className="mt-2 text-xs text-red-600">Error: {org.last_sync_error}</p>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Guide */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-blue-900 mb-2">How it works</h3>
        <ol className="text-sm text-blue-800 space-y-1 list-decimal list-inside">
          <li>Click "Create Organization" to redirect to nID dashboard</li>
          <li>Fill in the organization details (legal name, registration, country)</li>
          <li>Organization is created in nID and automatically synced to Communicator via webhook</li>
          <li>Invite employees and start the KYB verification process</li>
          <li>Profile updates in nID sync to Communicator in real-time (&lt;1 second)</li>
        </ol>
      </div>
    </div>
  )
}
