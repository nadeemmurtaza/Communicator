import { useState, useEffect } from 'react'
import {
  getEmbedWidgetSummary, listEmbedWidgetConfigs, createEmbedWidgetConfig,
  updateEmbedWidgetConfig, deleteEmbedWidgetConfig, getEmbedWidgetSettings,
  updateEmbedWidgetSettings, getEmbedWidgetEvents, getEmbedScript
} from '../lib/api'
import type { EmbedWidgetConfig, EmbedWidgetEvent, EmbedWidgetSettings, EmbedWidgetSummary } from '../lib/api'
import {
  Code, Settings, Plus, Trash2, Play, Copy, CheckCircle,
  AlertCircle, RefreshCw, Eye, Search, Globe, MessageSquare,
  Layout, BarChart3
} from '../components/icons'

export default function EmbedWidgetDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'configs' | 'events' | 'settings'>('overview')
  const [summary, setSummary] = useState<EmbedWidgetSummary | null>(null)
  const [configs, setConfigs] = useState<EmbedWidgetConfig[]>([])
  const [settings, setSettings] = useState<EmbedWidgetSettings | null>(null)
  const [events, setEvents] = useState<EmbedWidgetEvent[]>([])
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  // Create widget modal
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newName, setNewName] = useState('')
  const [newWidgetType, setNewWidgetType] = useState('help_center')
  const [creating, setCreating] = useState(false)

  // Script copy
  const [copiedScript, setCopiedScript] = useState<string | null>(null)

  // Filter
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [searchQuery, setSearchQuery] = useState('')

  useEffect(() => {
    loadData()
  }, [])

  async function loadData() {
    setLoading(true)
    try {
      const [summaryData, configsData, settingsData, eventsData] = await Promise.all([
        getEmbedWidgetSummary(),
        listEmbedWidgetConfigs(),
        getEmbedWidgetSettings(),
        getEmbedWidgetEvents(undefined, undefined, 50)
      ])
      setSummary(summaryData)
      setConfigs(configsData)
      setSettings(settingsData)
      setEvents(eventsData)
    } catch (e) {
      console.error('Failed to load embed widget data:', e)
    }
    setLoading(false)
  }

  async function handleCreate() {
    if (!newName.trim()) return
    setCreating(true)
    try {
      await createEmbedWidgetConfig({
        name: newName.trim(),
        widgetType: newWidgetType
      })
      setNewName('')
      setShowCreateModal(false)
      await loadData()
    } catch (e) {
      console.error('Failed to create widget config:', e)
    }
    setCreating(false)
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this widget config?')) return
    try {
      await deleteEmbedWidgetConfig(id)
      await loadData()
    } catch (e) {
      console.error('Failed to delete widget config:', e)
    }
  }

  async function handleToggleStatus(id: string, currentStatus: string) {
    const newStatus = currentStatus === 'active' ? 'paused' : 'active'
    try {
      await updateEmbedWidgetConfig(id, { status: newStatus })
      await loadData()
    } catch (e) {
      console.error('Failed to update widget status:', e)
    }
  }

  async function handleCopyScript(configId: string) {
    try {
      const result = await getEmbedScript(configId)
      navigator.clipboard.writeText(result.script)
      setCopiedScript(configId)
      setTimeout(() => setCopiedScript(null), 2000)
    } catch (e) {
      console.error('Failed to get embed script:', e)
    }
  }

  async function handleSaveSettings() {
    if (!settings) return
    setSaving(true)
    try {
      await updateEmbedWidgetSettings(settings)
      await loadData()
    } catch (e) {
      console.error('Failed to save settings:', e)
    }
    setSaving(false)
  }

  const filteredConfigs = configs.filter(c => {
    if (statusFilter && c.status !== statusFilter) return false
    if (searchQuery && !c.name.toLowerCase().includes(searchQuery.toLowerCase())) return false
    return true
  })

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[var(--color-accent)] mx-auto mb-4" />
          <p className="text-sm text-[var(--color-text-muted)]">Loading embed widget data...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-text)]">Embed Widget</h1>
          <p className="text-sm text-[var(--color-text-muted)] mt-1">
            Manage embeddable help center and live chat widgets for external websites
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={loadData}
            className="inline-flex items-center gap-2 rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-bg-hover)]"
          >
            <RefreshCw className="h-4 w-4" />
            Refresh
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-blue-100 p-2 dark:bg-blue-900/30">
              <Layout className="h-5 w-5 text-blue-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{summary?.totalConfigs || 0}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Total Widgets</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-green-100 p-2 dark:bg-green-900/30">
              <CheckCircle className="h-5 w-5 text-green-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{summary?.activeConfigs || 0}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Active Widgets</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-purple-100 p-2 dark:bg-purple-900/30">
              <Globe className="h-5 w-5 text-purple-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{summary?.totalInstances || 0}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Total Instances</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-orange-100 p-2 dark:bg-orange-900/30">
              <BarChart3 className="h-5 w-5 text-orange-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{summary?.totalEvents || 0}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Total Events</p>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-[var(--color-border)]">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'overview' as const, label: 'Overview', icon: BarChart3 },
            { id: 'configs' as const, label: 'Widget Configs', icon: Layout },
            { id: 'events' as const, label: 'Events', icon: MessageSquare },
            { id: 'settings' as const, label: 'Settings', icon: Settings },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`inline-flex items-center gap-2 border-b-2 px-1 pb-4 text-sm font-medium ${
                activeTab === tab.id
                  ? 'border-[var(--color-accent)] text-[var(--color-accent)]'
                  : 'border-transparent text-[var(--color-text-muted)] hover:border-[var(--color-border)] hover:text-[var(--color-text)]'
              }`}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Widget Types</h2>
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              {[
                { type: 'help_center', label: 'Help Center', icon: Search, color: 'blue' },
                { type: 'live_chat', label: 'Live Chat', icon: MessageSquare, color: 'green' },
                { type: 'search', label: 'Search', icon: Search, color: 'purple' },
                { type: 'combined', label: 'Combined', icon: Layout, color: 'orange' },
              ].map((wt) => (
                <div key={wt.type} className={`rounded-lg bg-${wt.color}-50 border border-${wt.color}-200 p-4 dark:bg-${wt.color}-900/20 dark:border-${wt.color}-800`}>
                  <div className="flex items-center gap-2 mb-2">
                    <wt.icon className={`h-5 w-5 text-${wt.color}-600`} />
                    <h3 className="font-medium text-[var(--color-text)]">{wt.label}</h3>
                  </div>
                  <p className="text-sm text-[var(--color-text-muted)]">
                    {wt.type === 'help_center' && 'Embeddable help center with search and categories'}
                    {wt.type === 'live_chat' && 'Live chat widget for real-time conversations'}
                    {wt.type === 'search' && 'Search-only widget for quick article lookup'}
                    {wt.type === 'combined' && 'Combined help center and live chat'}
                  </p>
                </div>
              ))}
            </div>
          </div>

          {/* Events by Type */}
          {summary?.eventsByType && summary.eventsByType.length > 0 && (
            <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
              <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Events by Type</h2>
              <div className="space-y-3">
                {summary.eventsByType.map((et) => (
                  <div key={et.eventType} className="flex items-center gap-3">
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-sm font-medium text-[var(--color-text)]">{et.eventType}</span>
                        <span className="text-sm text-[var(--color-text-muted)]">{et.count}</span>
                      </div>
                      <div className="w-full bg-[var(--color-bg-hover)] rounded-full h-2">
                        <div
                          className="bg-[var(--color-accent)] h-2 rounded-full"
                          style={{ width: `${Math.min((et.count / (summary.totalEvents || 1)) * 100, 100)}%` }}
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Top Domains */}
          {summary?.topDomains && summary.topDomains.length > 0 && (
            <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
              <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Top Domains</h2>
              <div className="space-y-2">
                {summary.topDomains.map((d) => (
                  <div key={d.domain} className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-3 py-2">
                    <span className="text-sm font-medium text-[var(--color-text)]">{d.domain}</span>
                    <span className="text-sm text-[var(--color-text-muted)]">{d.instances} instances</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* How it works */}
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">How Embedding Works</h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Plus className="h-5 w-5 text-blue-600" />
                  <h3 className="font-medium text-[var(--color-text)]">1. Create Widget</h3>
                </div>
                <p className="text-sm text-[var(--color-text-muted)]">
                  Configure a widget with your preferred type, styles, and trigger settings.
                </p>
              </div>
              <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Code className="h-5 w-5 text-purple-600" />
                  <h3 className="font-medium text-[var(--color-text)]">2. Copy Script</h3>
                </div>
                <p className="text-sm text-[var(--color-text-muted)]">
                  Get the embed script and paste it into your website's HTML before the closing &lt;/body&gt; tag.
                </p>
              </div>
              <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Globe className="h-5 w-5 text-green-600" />
                  <h3 className="font-medium text-[var(--color-text)]">3. Go Live</h3>
                </div>
                <p className="text-sm text-[var(--color-text-muted)]">
                  Your widget appears on your site, providing self-service help or live chat to visitors.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Configs Tab */}
      {activeTab === 'configs' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder="Search widgets..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent)] focus:outline-none"
              />
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] focus:border-[var(--color-accent)] focus:outline-none"
              >
                <option value="">All Statuses</option>
                <option value="active">Active</option>
                <option value="draft">Draft</option>
                <option value="paused">Paused</option>
              </select>
            </div>
            <button
              onClick={() => setShowCreateModal(true)}
              className="inline-flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90"
            >
              <Plus className="h-4 w-4" />
              New Widget
            </button>
          </div>

          <div className="space-y-3">
            {filteredConfigs.map((config) => (
              <div key={config.id} className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className={`rounded-lg p-2 ${
                      config.widgetType === 'help_center' ? 'bg-blue-100 dark:bg-blue-900/30' :
                      config.widgetType === 'live_chat' ? 'bg-green-100 dark:bg-green-900/30' :
                      config.widgetType === 'search' ? 'bg-purple-100 dark:bg-purple-900/30' :
                      'bg-orange-100 dark:bg-orange-900/30'
                    }`}>
                      {config.widgetType === 'help_center' ? <Search className="h-5 w-5 text-blue-600" /> :
                       config.widgetType === 'live_chat' ? <MessageSquare className="h-5 w-5 text-green-600" /> :
                       config.widgetType === 'search' ? <Search className="h-5 w-5 text-purple-600" /> :
                       <Layout className="h-5 w-5 text-orange-600" />}
                    </div>
                    <div>
                      <h3 className="font-medium text-[var(--color-text)]">{config.name}</h3>
                      <p className="text-xs text-[var(--color-text-muted)]">{config.widgetType} · {config.installCount} installs</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      config.status === 'active' ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' :
                      config.status === 'paused' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400' :
                      'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400'
                    }`}>
                      {config.status}
                    </span>
                    <button
                      onClick={() => handleToggleStatus(config.id, config.status)}
                      className="rounded-lg p-1 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                      title={config.status === 'active' ? 'Pause' : 'Activate'}
                    >
                      <Play className="h-4 w-4" />
                    </button>
                    <button
                      onClick={() => handleCopyScript(config.id)}
                      className="rounded-lg p-1 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                      title="Copy embed script"
                    >
                      {copiedScript === config.id ? <CheckCircle className="h-4 w-4 text-green-600" /> : <Copy className="h-4 w-4" />}
                    </button>
                    <button
                      onClick={() => handleDelete(config.id)}
                      className="rounded-lg p-1 text-[var(--color-text-muted)] hover:text-red-600"
                      title="Delete"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
            {filteredConfigs.length === 0 && (
              <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-8 text-center">
                <Layout className="h-8 w-8 text-[var(--color-text-muted)] mx-auto mb-3" />
                <p className="text-sm text-[var(--color-text-muted)]">No widget configs found</p>
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="mt-3 inline-flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90"
                >
                  <Plus className="h-4 w-4" />
                  Create your first widget
                </button>
              </div>
            )}
          </div>

          {/* Create Modal */}
          {showCreateModal && (
            <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
              <div className="rounded-xl bg-[var(--color-bg)] border border-[var(--color-border)] p-6 w-full max-w-md">
                <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Create Widget</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-[var(--color-text)] mb-2">Name</label>
                    <input
                      type="text"
                      value={newName}
                      onChange={(e) => setNewName(e.target.value)}
                      className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] focus:border-[var(--color-accent)] focus:outline-none"
                      placeholder="My Help Center Widget"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-[var(--color-text)] mb-2">Widget Type</label>
                    <select
                      value={newWidgetType}
                      onChange={(e) => setNewWidgetType(e.target.value)}
                      className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] focus:border-[var(--color-accent)] focus:outline-none"
                    >
                      <option value="help_center">Help Center</option>
                      <option value="live_chat">Live Chat</option>
                      <option value="search">Search</option>
                      <option value="combined">Combined</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-end gap-2">
                    <button
                      onClick={() => setShowCreateModal(false)}
                      className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-bg-hover)]"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleCreate}
                      disabled={creating || !newName.trim()}
                      className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90 disabled:opacity-50"
                    >
                      {creating ? 'Creating...' : 'Create'}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Events Tab */}
      {activeTab === 'events' && (
        <div className="space-y-4">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-[var(--color-border)]">
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Type</th>
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Instance</th>
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Time</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-border)]">
                  {events.map((event) => (
                    <tr key={event.id} className="hover:bg-[var(--color-bg-hover)]">
                      <td className="px-4 py-3">
                        <span className="rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700 dark:bg-blue-900/30 dark:text-blue-400">
                          {event.eventType}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-sm font-mono text-[var(--color-text-muted)]">
                        {event.instanceId.slice(0, 8)}...
                      </td>
                      <td className="px-4 py-3 text-sm text-[var(--color-text-muted)]">
                        {new Date(event.createdAt).toLocaleString()}
                      </td>
                    </tr>
                  ))}
                  {events.length === 0 && (
                    <tr>
                      <td colSpan={3} className="px-4 py-8 text-center text-sm text-[var(--color-text-muted)]">
                        No events recorded yet
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Widget Settings</h2>
            {settings && (
              <div className="space-y-4">
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-4 py-3">
                  <div>
                    <p className="text-sm font-medium text-[var(--color-text)]">Enable Widgets</p>
                    <p className="text-xs text-[var(--color-text-muted)]">Allow embedding widgets on external sites</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, enabled: !settings.enabled })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.enabled ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-700'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.enabled ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-4 py-3">
                  <div>
                    <p className="text-sm font-medium text-[var(--color-text)]">Allow Custom Domains</p>
                    <p className="text-xs text-[var(--color-text-muted)]">Permit widgets on any domain</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, allowCustomDomains: !settings.allowCustomDomains })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.allowCustomDomains ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-700'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.allowCustomDomains ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-4 py-3">
                  <div>
                    <p className="text-sm font-medium text-[var(--color-text)]">Analytics</p>
                    <p className="text-xs text-[var(--color-text-muted)]">Track widget usage and events</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, analyticsEnabled: !settings.analyticsEnabled })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.analyticsEnabled ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-700'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.analyticsEnabled ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="rounded-lg bg-[var(--color-bg-hover)] px-4 py-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium text-[var(--color-text)]">Max Instances per Domain</p>
                      <p className="text-xs text-[var(--color-text-muted)]">Limit concurrent widget instances</p>
                    </div>
                    <input
                      type="number"
                      value={settings.maxInstancesPerDomain}
                      onChange={(e) => setSettings({ ...settings, maxInstancesPerDomain: parseInt(e.target.value) || 100 })}
                      className="w-20 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] text-right focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  </div>
                </div>
                <div className="rounded-lg bg-[var(--color-bg-hover)] px-4 py-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium text-[var(--color-text)]">Default Language</p>
                      <p className="text-xs text-[var(--color-text-muted)]">Language for widget content</p>
                    </div>
                    <input
                      type="text"
                      value={settings.defaultLanguage}
                      onChange={(e) => setSettings({ ...settings, defaultLanguage: e.target.value })}
                      className="w-20 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] text-right focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  </div>
                </div>
                <div className="flex justify-end">
                  <button
                    onClick={handleSaveSettings}
                    disabled={saving}
                    className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90 disabled:opacity-50"
                  >
                    {saving ? 'Saving...' : 'Save Settings'}
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
