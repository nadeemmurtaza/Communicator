import { useState, useEffect } from 'react'
import {
  getMCPSummary, listMCPServers, createMCPServer, updateMCPServer,
  deleteMCPServer, discoverMCPTools, listMCPTools, callMCPTool,
  listMCPTemplates, getMCPSettings, updateMCPSettings, listMCPCallLogs
} from '../lib/api'
import type { MCPServer, MCPTool, MCPCallLog, MCPServerTemplate, MCPSettings, MCPSummary } from '../lib/api'

export default function MCPDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'servers' | 'tools' | 'templates' | 'calls' | 'settings'>('overview')
  const [summary, setSummary] = useState<MCPSummary | null>(null)
  const [servers, setServers] = useState<MCPServer[]>([])
  const [tools, setTools] = useState<MCPTool[]>([])
  const [templates, setTemplates] = useState<MCPServerTemplate[]>([])
  const [calls, setCalls] = useState<MCPCallLog[]>([])
  const [settings, setSettings] = useState<MCPSettings | null>(null)
  const [loading, setLoading] = useState(false)
  const [showCreateServer, setShowCreateServer] = useState(false)
  const [newServer, setNewServer] = useState({ name: '', serverUrl: '', transportType: 'http' })

  useEffect(() => {
    loadAll()
  }, [activeTab])

  async function loadAll() {
    setLoading(true)
    try {
      const [s, svrs, tls, tmps, cls, stg] = await Promise.all([
        getMCPSummary(),
        listMCPServers(),
        listMCPTools(),
        listMCPTemplates(),
        listMCPCallLogs(),
        getMCPSettings(),
      ])
      setSummary(s)
      setServers(svrs)
      setTools(tls)
      setTemplates(tmps)
      setCalls(cls)
      setSettings(stg)
    } finally {
      setLoading(false)
    }
  }

  async function handleCreateServer() {
    if (!newServer.name || !newServer.serverUrl) return
    await createMCPServer({
      organizationId: 'org-default',
      name: newServer.name,
      serverUrl: newServer.serverUrl,
      transportType: newServer.transportType,
    })
    setShowCreateServer(false)
    setNewServer({ name: '', serverUrl: '', transportType: 'http' })
    loadAll()
  }

  async function handleDeleteServer(id: string) {
    if (!confirm('Delete this MCP server?')) return
    await deleteMCPServer(id)
    loadAll()
  }

  async function handleDiscover(serverId: string) {
    await discoverMCPTools(serverId)
    loadAll()
  }

  async function handleToggleServer(serverId: string, isActive: boolean) {
    await updateMCPServer(serverId, { is_active: isActive })
    loadAll()
  }

  async function handleUpdateSettings() {
    if (!settings) return
    await updateMCPSettings(settings)
    loadAll()
  }

  const statusColor = (status: string) => {
    switch (status) {
      case 'connected': return '#22c55e'
      case 'active': return '#22c55e'
      case 'completed': return '#3b82f6'
      case 'pending': return '#f59e0b'
      case 'failed': return '#ef4444'
      default: return '#6b7280'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">MCP Protocol</h1>
          <p className="text-gray-400 mt-1">Manage MCP server connections and AI tool integrations</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-gray-700">
        {(['overview', 'servers', 'tools', 'templates', 'calls', 'settings'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium capitalize rounded-t-lg transition-colors ${
              activeTab === tab
                ? 'bg-blue-600 text-white'
                : 'text-gray-400 hover:text-white hover:bg-gray-800'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {loading && <div className="text-center py-8 text-gray-400">Loading...</div>}

      {/* Overview Tab */}
      {activeTab === 'overview' && summary && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard label="Total Servers" value={summary.totalServers} />
            <StatCard label="Connected" value={summary.connectedServers} color="#22c55e" />
            <StatCard label="Total Tools" value={summary.totalTools} />
            <StatCard label="Enabled Tools" value={summary.enabledTools} color="#22c55e" />
            <StatCard label="Active Sessions" value={summary.activeSessions} />
            <StatCard label="Calls (7d)" value={summary.totalCalls7d} />
            <StatCard label="Calls (30d)" value={summary.totalCalls30d} />
            <StatCard label="Templates" value={summary.availableTemplates} />
          </div>

          {summary.topCalledTools.length > 0 && (
            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Top Called Tools</h3>
              <div className="space-y-3">
                {summary.topCalledTools.map((tool, i) => (
                  <div key={i} className="flex items-center gap-4">
                    <span className="text-sm text-gray-400 w-32 truncate">{tool.toolName}</span>
                    <div className="flex-1 bg-gray-700 rounded-full h-2">
                      <div
                        className="bg-blue-500 h-2 rounded-full"
                        style={{
                          width: `${Math.max(5, (tool.callCount / summary.topCalledTools[0].callCount) * 100)}%`
                        }}
                      />
                    </div>
                    <span className="text-sm text-gray-300 w-12 text-right">{tool.callCount}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {summary.recentCalls.length > 0 && (
            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Recent Calls</h3>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-gray-400 border-b border-gray-700">
                      <th className="text-left py-2 pr-4">Tool</th>
                      <th className="text-left py-2 pr-4">Server</th>
                      <th className="text-left py-2 pr-4">Status</th>
                      <th className="text-left py-2 pr-4">Duration</th>
                      <th className="text-left py-2">Called At</th>
                    </tr>
                  </thead>
                  <tbody>
                    {summary.recentCalls.map((call) => (
                      <tr key={call.id} className="border-b border-gray-700/50">
                        <td className="py-2 pr-4 text-white">{call.toolName}</td>
                        <td className="py-2 pr-4 text-gray-300">{call.serverName || '-'}</td>
                        <td className="py-2 pr-4">
                          <span className="px-2 py-0.5 rounded text-xs" style={{ color: statusColor(call.status) }}>
                            {call.status}
                          </span>
                        </td>
                        <td className="py-2 pr-4 text-gray-300">{call.durationMs ? `${call.durationMs}ms` : '-'}</td>
                        <td className="py-2 text-gray-400">{new Date(call.calledAt).toLocaleString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Servers Tab */}
      {activeTab === 'servers' && (
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h2 className="text-lg font-semibold text-white">MCP Servers</h2>
            <button
              onClick={() => setShowCreateServer(true)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm"
            >
              + Add Server
            </button>
          </div>

          {showCreateServer && (
            <div className="bg-gray-800 rounded-lg p-6 space-y-4">
              <h3 className="text-lg font-semibold text-white">Add MCP Server</h3>
              <input
                placeholder="Server name"
                value={newServer.name}
                onChange={(e) => setNewServer({ ...newServer, name: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600"
              />
              <input
                placeholder="Server URL"
                value={newServer.serverUrl}
                onChange={(e) => setNewServer({ ...newServer, serverUrl: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600"
              />
              <select
                value={newServer.transportType}
                onChange={(e) => setNewServer({ ...newServer, transportType: e.target.value })}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600"
              >
                <option value="http">HTTP</option>
                <option value="stdio">stdio</option>
              </select>
              <div className="flex gap-2">
                <button onClick={handleCreateServer} className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm">
                  Create
                </button>
                <button onClick={() => setShowCreateServer(false)} className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 text-sm">
                  Cancel
                </button>
              </div>
            </div>
          )}

          {servers.length === 0 ? (
            <div className="text-center py-8 text-gray-400">No MCP servers configured</div>
          ) : (
            <div className="space-y-3">
              {servers.map((server) => (
                <div key={server.id} className="bg-gray-800 rounded-lg p-4 flex items-center justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <h3 className="text-white font-medium">{server.name}</h3>
                      <span
                        className="px-2 py-0.5 rounded text-xs"
                        style={{ color: statusColor(server.connectionStatus) }}
                      >
                        {server.connectionStatus}
                      </span>
                    </div>
                    <p className="text-sm text-gray-400 mt-1">{server.serverUrl}</p>
                    <div className="flex gap-4 mt-2 text-xs text-gray-500">
                      <span>{server.transportType}</span>
                      <span>{server.toolCount || 0} tools</span>
                      <span>{server.isActive ? 'Active' : 'Inactive'}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleDiscover(server.id)}
                      className="px-3 py-1 bg-purple-600 text-white rounded text-xs hover:bg-purple-700"
                    >
                      Discover
                    </button>
                    <button
                      onClick={() => handleToggleServer(server.id, !server.isActive)}
                      className="px-3 py-1 bg-gray-600 text-white rounded text-xs hover:bg-gray-700"
                    >
                      {server.isActive ? 'Disable' : 'Enable'}
                    </button>
                    <button
                      onClick={() => handleDeleteServer(server.id)}
                      className="px-3 py-1 bg-red-600/20 text-red-400 rounded text-xs hover:bg-red-600/30"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Tools Tab */}
      {activeTab === 'tools' && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-white">MCP Tools</h2>
          {tools.length === 0 ? (
            <div className="text-center py-8 text-gray-400">No tools discovered. Connect a server and click "Discover" first.</div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {tools.map((tool) => (
                <div key={tool.id} className="bg-gray-800 rounded-lg p-4">
                  <div className="flex items-center justify-between">
                    <h3 className="text-white font-medium">{tool.toolName}</h3>
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-gray-400">{tool.serverName}</span>
                      <span
                        className="w-2 h-2 rounded-full"
                        style={{ backgroundColor: tool.isEnabled ? '#22c55e' : '#ef4444' }}
                      />
                    </div>
                  </div>
                  {tool.description && <p className="text-sm text-gray-400 mt-1">{tool.description}</p>}
                  <div className="flex gap-4 mt-2 text-xs text-gray-500">
                    <span>{tool.callCount || 0} calls</span>
                    <span>{tool.aiAccessible ? 'AI accessible' : 'AI blocked'}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Templates Tab */}
      {activeTab === 'templates' && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-white">Server Templates</h2>
          <p className="text-sm text-gray-400">Pre-built configurations for common MCP-compatible services</p>
          {templates.length === 0 ? (
            <div className="text-center py-8 text-gray-400">No templates available</div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {templates.map((template) => (
                <div key={template.id} className="bg-gray-800 rounded-lg p-6">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-blue-600/20 rounded-lg flex items-center justify-center">
                      <span className="text-blue-400 font-bold text-lg">{template.name[0]}</span>
                    </div>
                    <div>
                      <h3 className="text-white font-medium">{template.name}</h3>
                      <p className="text-xs text-gray-400">{template.provider}</p>
                    </div>
                  </div>
                  {template.description && (
                    <p className="text-sm text-gray-400 mt-3">{template.description}</p>
                  )}
                  <div className="mt-3 text-xs text-gray-500">
                    <p>Transport: {template.transportType}</p>
                    {template.defaultTools && template.defaultTools.length > 0 && (
                      <div className="flex flex-wrap gap-1 mt-2">
                        {template.defaultTools.map((t, i) => (
                          <span key={i} className="px-2 py-0.5 bg-gray-700 rounded text-gray-300">
                            {t}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                  <button
                    onClick={() => {
                      setNewServer({
                        name: template.name,
                        serverUrl: template.defaultUrl,
                        transportType: template.transportType,
                      })
                      setShowCreateServer(true)
                      setActiveTab('servers')
                    }}
                    className="mt-4 w-full px-3 py-2 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700"
                  >
                    Use Template
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Calls Tab */}
      {activeTab === 'calls' && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-white">Call Logs</h2>
          {calls.length === 0 ? (
            <div className="text-center py-8 text-gray-400">No call logs yet</div>
          ) : (
            <div className="bg-gray-800 rounded-lg overflow-hidden">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-gray-400 border-b border-gray-700 bg-gray-800/50">
                    <th className="text-left py-3 px-4">Tool</th>
                    <th className="text-left py-3 px-4">Server</th>
                    <th className="text-left py-3 px-4">Status</th>
                    <th className="text-left py-3 px-4">Duration</th>
                    <th className="text-left py-3 px-4">Called By</th>
                    <th className="text-left py-3 px-4">Time</th>
                  </tr>
                </thead>
                <tbody>
                  {calls.map((call) => (
                    <tr key={call.id} className="border-b border-gray-700/50 hover:bg-gray-700/30">
                      <td className="py-3 px-4 text-white font-medium">{call.toolName}</td>
                      <td className="py-3 px-4 text-gray-300">{call.serverName || '-'}</td>
                      <td className="py-3 px-4">
                        <span className="px-2 py-0.5 rounded text-xs" style={{ color: statusColor(call.status) }}>
                          {call.status}
                        </span>
                      </td>
                      <td className="py-3 px-4 text-gray-300">{call.durationMs ? `${call.durationMs}ms` : '-'}</td>
                      <td className="py-3 px-4 text-gray-400">{call.calledBy}</td>
                      <td className="py-3 px-4 text-gray-400">{new Date(call.calledAt).toLocaleString()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && settings && (
        <div className="space-y-6">
          <h2 className="text-lg font-semibold text-white">MCP Settings</h2>
          <div className="bg-gray-800 rounded-lg p-6 space-y-4 max-w-lg">
            <SettingToggle
              label="Enabled"
              description="Enable MCP protocol support"
              checked={settings.enabled}
              onChange={(v) => setSettings({ ...settings, enabled: v })}
            />
            <SettingToggle
              label="Auto-discover Tools"
              description="Automatically discover tools when connecting"
              checked={settings.autoDiscoverTools}
              onChange={(v) => setSettings({ ...settings, autoDiscoverTools: v })}
            />
            <SettingToggle
              label="Log All Calls"
              description="Record every tool call for auditing"
              checked={settings.logAllCalls}
              onChange={(v) => setSettings({ ...settings, logAllCalls: v })}
            />
            <div>
              <label className="block text-sm text-gray-400 mb-1">Max Concurrent Calls</label>
              <input
                type="number"
                value={settings.maxConcurrentCalls}
                onChange={(e) => setSettings({ ...settings, maxConcurrentCalls: parseInt(e.target.value) || 10 })}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Timeout (seconds)</label>
              <input
                type="number"
                value={settings.timeoutSeconds}
                onChange={(e) => setSettings({ ...settings, timeoutSeconds: parseInt(e.target.value) || 30 })}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600"
              />
            </div>
            <button
              onClick={handleUpdateSettings}
              className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm"
            >
              Save Settings
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

function StatCard({ label, value, color = '#3b82f6' }: { label: string; value: number; color?: string }) {
  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <p className="text-sm text-gray-400">{label}</p>
      <p className="text-2xl font-bold mt-1" style={{ color }}>{value}</p>
    </div>
  )
}

function SettingToggle({ label, description, checked, onChange }: {
  label: string
  description: string
  checked: boolean
  onChange: (v: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm text-white">{label}</p>
        <p className="text-xs text-gray-400">{description}</p>
      </div>
      <button
        onClick={() => onChange(!checked)}
        className={`relative w-11 h-6 rounded-full transition-colors ${checked ? 'bg-blue-600' : 'bg-gray-600'}`}
      >
        <span
          className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
            checked ? 'translate-x-5' : 'translate-x-0'
          }`}
        />
      </button>
    </div>
  )
}
