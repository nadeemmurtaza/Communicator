import { useState, useEffect } from 'react'
import {
  getCRMSyncSummary,
  getCRMProviders,
  listCRMConnections,
  createCRMConnection,
  updateCRMConnection,
  deleteCRMConnection,
  triggerCRMSync,
  listCRMSyncLogs,
  listFieldMappings,
} from '../lib/api'
import type { CRMConnection, CRMSyncLog, CRMFieldMapping, CRMProviderInfo, CRMSyncSummary } from '../lib/api'
import {
  Database,
  RefreshCw,
  Plus,
  Globe,
  CheckCircle,
  XCircle,
  AlertTriangle,
  ArrowRightLeft,
  Settings,
  Trash2,
  Play,
  X,
  Link as LinkIcon,
} from '../components/icons'

export default function CRMSyncDashboard() {
  const [tab, setTab] = useState<'overview' | 'connections' | 'logs' | 'mappings'>('overview')
  const [summary, setSummary] = useState<CRMSyncSummary | null>(null)
  const [connections, setConnections] = useState<CRMConnection[]>([])
  const [logs, setLogs] = useState<CRMSyncLog[]>([])
  const [mappings, setMappings] = useState<CRMFieldMapping[]>([])
  const [providers, setProviders] = useState<CRMProviderInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [syncing, setSyncing] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Create form
  const [newProvider, setNewProvider] = useState('salesforce')
  const [newAuthType, setNewAuthType] = useState('oauth')
  const [newCredentials, setNewCredentials] = useState('')

  // Filter state
  const [filterProvider, setFilterProvider] = useState('')
  const [filterStatus, setFilterStatus] = useState('')

  async function loadAll() {
    setLoading(true)
    setError(null)
    try {
      const [sum, conns, logsData, maps, provs] = await Promise.all([
        getCRMSyncSummary(),
        listCRMConnections(),
        listCRMSyncLogs(undefined, 20),
        listFieldMappings(),
        getCRMProviders(),
      ])
      setSummary(sum)
      setConnections(conns)
      setLogs(logsData)
      setMappings(maps)
      setProviders(provs)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadAll()
  }, [])

  async function handleCreateConnection() {
    setError(null)
    setSuccess(null)
    try {
      const creds = newCredentials.trim() ? JSON.parse(newCredentials) : undefined
      await createCRMConnection({
        provider: newProvider,
        authType: newAuthType,
        credentials: creds,
      })
      setSuccess(`Connection to ${newProvider} created`)
      setShowCreateModal(false)
      setNewCredentials('')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleDeleteConnection(id: string) {
    if (!confirm('Delete this CRM connection?')) return
    try {
      await deleteCRMConnection(id)
      setSuccess('Connection deleted')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleSync(id: string, direction: string, objectType: string) {
    setSyncing(id)
    setError(null)
    try {
      await triggerCRMSync(id, direction, objectType)
      setSuccess(`Sync ${direction} ${objectType} completed`)
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncing(null)
    }
  }

  async function handleSetStatus(id: string, status: string) {
    try {
      await updateCRMConnection(id, { status })
      setSuccess(`Connection status updated to ${status}`)
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function statusBadge(status: string) {
    const colors: Record<string, string> = {
      connected: 'bg-emerald-100 text-emerald-700',
      connecting: 'bg-yellow-100 text-yellow-700',
      disconnected: 'bg-gray-100 text-gray-700',
      error: 'bg-red-100 text-red-700',
    }
    return (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[status] || colors.disconnected}`}>
        {status}
      </span>
    )
  }

  function providerIcon(provider: string) {
    switch (provider) {
      case 'salesforce':
        return '☁️'
      case 'hubspot':
        return '🟠'
      case 'pipedrive':
        return '🟢'
      default:
        return '🔗'
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="w-6 h-6 animate-spin text-gray-400" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">CRM Sync</h1>
          <p className="text-sm text-gray-500 mt-1">
            Connect and sync with Salesforce, HubSpot, and Pipedrive
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
        >
          <Plus className="w-4 h-4" />
          New Connection
        </button>
      </div>

      {/* Error / Success */}
      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 text-red-700 rounded-lg">
          <XCircle className="w-4 h-4 flex-shrink-0" />
          <span className="text-sm">{error}</span>
          <button onClick={() => setError(null)} className="ml-auto">
            <X className="w-4 h-4" />
          </button>
        </div>
      )}
      {success && (
        <div className="flex items-center gap-2 p-3 bg-emerald-50 text-emerald-700 rounded-lg">
          <CheckCircle className="w-4 h-4 flex-shrink-0" />
          <span className="text-sm">{success}</span>
          <button onClick={() => setSuccess(null)} className="ml-auto">
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 border-b border-gray-200">
        {(['overview', 'connections', 'logs', 'mappings'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 text-sm font-medium capitalize rounded-t-lg ${
              tab === t
                ? 'bg-indigo-50 text-indigo-700 border-b-2 border-indigo-600'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {tab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="p-4 bg-white rounded-lg border border-gray-200">
              <div className="flex items-center gap-3">
                <Database className="w-8 h-8 text-indigo-500" />
                <div>
                  <p className="text-2xl font-bold">{summary?.totalConnections || 0}</p>
                  <p className="text-sm text-gray-500">Total Connections</p>
                </div>
              </div>
            </div>
            <div className="p-4 bg-white rounded-lg border border-gray-200">
              <div className="flex items-center gap-3">
                <CheckCircle className="w-8 h-8 text-emerald-500" />
                <div>
                  <p className="text-2xl font-bold">{summary?.activeConnections || 0}</p>
                  <p className="text-sm text-gray-500">Active</p>
                </div>
              </div>
            </div>
            <div className="p-4 bg-white rounded-lg border border-gray-200">
              <div className="flex items-center gap-3">
                <RefreshCw className="w-8 h-8 text-blue-500" />
                <div>
                  <p className="text-2xl font-bold">{summary?.totalSyncs7d || 0}</p>
                  <p className="text-sm text-gray-500">Syncs (7d)</p>
                </div>
              </div>
            </div>
            <div className="p-4 bg-white rounded-lg border border-gray-200">
              <div className="flex items-center gap-3">
                <Globe className="w-8 h-8 text-purple-500" />
                <div>
                  <p className="text-2xl font-bold">{summary?.totalRecordsSynced7d || 0}</p>
                  <p className="text-sm text-gray-500">Records Synced (7d)</p>
                </div>
              </div>
            </div>
          </div>

          {/* Provider Breakdown */}
          {summary?.providers?.length > 0 && (
            <div className="p-4 bg-white rounded-lg border border-gray-200">
              <h3 className="text-sm font-semibold text-gray-700 mb-3">Provider Breakdown</h3>
              <div className="space-y-2">
                {summary.providers.map((p) => (
                  <div key={p.provider} className="flex items-center gap-3">
                    <span className="text-lg">{providerIcon(p.provider)}</span>
                    <span className="text-sm font-medium capitalize flex-1">{p.provider}</span>
                    <span className="text-sm text-gray-500">{p.count} connection(s)</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Pending / Conflicts */}
          {(summary?.pendingRefs || summary?.conflictRefs) ? (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {summary.pendingRefs ? (
                <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="w-5 h-5 text-yellow-600" />
                    <p className="text-sm font-medium text-yellow-700">
                      {summary.pendingRefs} pending sync reference(s)
                    </p>
                  </div>
                </div>
              ) : null}
              {summary.conflictRefs ? (
                <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
                  <div className="flex items-center gap-2">
                    <XCircle className="w-5 h-5 text-red-600" />
                    <p className="text-sm font-medium text-red-700">
                      {summary.conflictRefs} conflict(s) detected
                    </p>
                  </div>
                </div>
              ) : null}
            </div>
          ) : null}

          {/* Recent Syncs */}
          {summary?.recentSyncs?.length > 0 && (
            <div className="p-4 bg-white rounded-lg border border-gray-200">
              <h3 className="text-sm font-semibold text-gray-700 mb-3">Recent Syncs</h3>
              <div className="space-y-2">
                {summary.recentSyncs.slice(0, 5).map((log) => (
                  <div key={log.id} className="flex items-center gap-3 text-sm">
                    <ArrowRightLeft className="w-4 h-4 text-gray-400" />
                    <span className="font-medium">{log.objectType}</span>
                    <span className="text-gray-500">{log.direction}</span>
                    <span className="text-gray-500">
                      {log.successCount}/{log.recordCount} records
                    </span>
                    <span className={`ml-auto text-xs font-medium ${
                      log.status === 'completed' ? 'text-emerald-600' :
                      log.status === 'failed' ? 'text-red-600' : 'text-yellow-600'
                    }`}>
                      {log.status}
                    </span>
                    <span className="text-gray-400 text-xs">{new Date(log.startedAt).toLocaleString()}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Connections Tab */}
      {tab === 'connections' && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex gap-3">
            <select
              value={filterProvider}
              onChange={(e) => { setFilterProvider(e.target.value); loadAll() }}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm"
            >
              <option value="">All Providers</option>
              <option value="salesforce">Salesforce</option>
              <option value="hubspot">HubSpot</option>
              <option value="pipedrive">Pipedrive</option>
            </select>
            <select
              value={filterStatus}
              onChange={(e) => { setFilterStatus(e.target.value); loadAll() }}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm"
            >
              <option value="">All Statuses</option>
              <option value="connected">Connected</option>
              <option value="connecting">Connecting</option>
              <option value="disconnected">Disconnected</option>
              <option value="error">Error</option>
            </select>
          </div>

          {connections.length === 0 ? (
            <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
              <Database className="w-12 h-12 text-gray-300 mx-auto mb-3" />
              <p className="text-gray-500">No CRM connections yet</p>
              <button
                onClick={() => setShowCreateModal(true)}
                className="mt-3 text-sm text-indigo-600 hover:text-indigo-700"
              >
                Create your first connection
              </button>
            </div>
          ) : (
            <div className="space-y-3">
              {connections.map((conn) => (
                <div key={conn.id} className="p-4 bg-white rounded-lg border border-gray-200">
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <span className="text-2xl">{providerIcon(conn.provider)}</span>
                      <div>
                        <h3 className="font-semibold capitalize">{conn.provider}</h3>
                        <p className="text-sm text-gray-500">
                          Auth: {conn.authType} &middot; Created: {new Date(conn.createdAt).toLocaleDateString()}
                        </p>
                        {conn.lastSyncAt && (
                          <p className="text-xs text-gray-400">
                            Last sync: {new Date(conn.lastSyncAt).toLocaleString()}
                          </p>
                        )}
                        {conn.lastError && (
                          <p className="text-xs text-red-500 mt-1">{conn.lastError}</p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {statusBadge(conn.status)}
                      {conn.status === 'connected' && (
                        <>
                          <button
                            onClick={() => handleSync(conn.id, 'outbound', 'lead')}
                            disabled={syncing === conn.id}
                            className="p-1.5 text-emerald-600 hover:bg-emerald-50 rounded"
                            title="Sync leads outbound"
                          >
                            <Play className="w-4 h-4" />
                          </button>
                          <button
                            onClick={() => handleSync(conn.id, 'inbound', 'contact')}
                            disabled={syncing === conn.id}
                            className="p-1.5 text-blue-600 hover:bg-blue-50 rounded"
                            title="Sync contacts inbound"
                          >
                            <RefreshCw className="w-4 h-4" />
                          </button>
                        </>
                      )}
                      {conn.status !== 'connected' && (
                        <button
                          onClick={() => handleSetStatus(conn.id, 'connected')}
                          className="px-2 py-1 text-xs bg-emerald-600 text-white rounded hover:bg-emerald-700"
                        >
                          Mark Connected
                        </button>
                      )}
                      <button
                        onClick={() => handleDeleteConnection(conn.id)}
                        className="p-1.5 text-red-500 hover:bg-red-50 rounded"
                        title="Delete"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Logs Tab */}
      {tab === 'logs' && (
        <div className="space-y-3">
          {logs.length === 0 ? (
            <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
              <RefreshCw className="w-12 h-12 text-gray-300 mx-auto mb-3" />
              <p className="text-gray-500">No sync logs yet</p>
            </div>
          ) : (
            <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-gray-50 border-b border-gray-200">
                  <tr>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Time</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Direction</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Object</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Records</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map((log) => (
                    <tr key={log.id} className="border-b border-gray-100 last:border-0">
                      <td className="px-4 py-2 text-gray-500">
                        {new Date(log.startedAt).toLocaleString()}
                      </td>
                      <td className="px-4 py-2">
                        <span className="capitalize">{log.direction}</span>
                      </td>
                      <td className="px-4 py-2 capitalize">{log.objectType}</td>
                      <td className="px-4 py-2">
                        <span className="text-emerald-600">{log.successCount}</span>
                        <span className="text-gray-400">/{log.recordCount}</span>
                        {log.errorCount > 0 && (
                          <span className="text-red-500 ml-1">({log.errorCount} errors)</span>
                        )}
                      </td>
                      <td className="px-4 py-2">
                        <span className={`text-xs font-medium ${
                          log.status === 'completed' ? 'text-emerald-600' :
                          log.status === 'failed' ? 'text-red-600' :
                          log.status === 'partial' ? 'text-yellow-600' : 'text-blue-600'
                        }`}>
                          {log.status}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Mappings Tab */}
      {tab === 'mappings' && (
        <div className="space-y-3">
          {mappings.length === 0 ? (
            <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
              <Settings className="w-12 h-12 text-gray-300 mx-auto mb-3" />
              <p className="text-gray-500">No field mappings configured</p>
            </div>
          ) : (
            <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-gray-50 border-b border-gray-200">
                  <tr>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Object</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Local Field</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">CRM Field</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Direction</th>
                    <th className="text-left px-4 py-2 font-medium text-gray-600">Required</th>
                  </tr>
                </thead>
                <tbody>
                  {mappings.map((m) => (
                    <tr key={m.id} className="border-b border-gray-100 last:border-0">
                      <td className="px-4 py-2 capitalize font-medium">{m.objectType}</td>
                      <td className="px-4 py-2 text-gray-600">{m.localField}</td>
                      <td className="px-4 py-2 text-gray-600">{m.crmField}</td>
                      <td className="px-4 py-2">
                        <span className="capitalize text-xs bg-gray-100 px-2 py-0.5 rounded">
                          {m.direction}
                        </span>
                      </td>
                      <td className="px-4 py-2">
                        {m.isRequired ? (
                          <CheckCircle className="w-4 h-4 text-emerald-500" />
                        ) : (
                          <span className="text-gray-300">—</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Create Connection Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl w-full max-w-md p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-bold">New CRM Connection</h2>
              <button onClick={() => setShowCreateModal(false)}>
                <X className="w-5 h-5 text-gray-400" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Provider</label>
                <select
                  value={newProvider}
                  onChange={(e) => setNewProvider(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                >
                  {providers.map((p) => (
                    <option key={p.provider} value={p.provider}>
                      {p.displayName}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Auth Type</label>
                <select
                  value={newAuthType}
                  onChange={(e) => setNewAuthType(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                >
                  <option value="oauth">OAuth</option>
                  <option value="api_key">API Key</option>
                  <option value="oauth2">OAuth 2.0</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Credentials (JSON)
                </label>
                <textarea
                  value={newCredentials}
                  onChange={(e) => setNewCredentials(e.target.value)}
                  placeholder='{"access_token": "...", "instance_url": "..."}'
                  rows={4}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg font-mono text-sm"
                />
                <p className="text-xs text-gray-400 mt-1">
                  Optional: paste credentials JSON, or leave empty to configure later
                </p>
              </div>

              <div className="flex gap-3 justify-end">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded-lg"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateConnection}
                  className="px-4 py-2 text-sm bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
                >
                  Create Connection
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
