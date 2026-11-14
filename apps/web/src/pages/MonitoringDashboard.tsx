import { useEffect, useState } from 'react'
import { api, type MonitoringDashboard, type MonitoringAlert, type MonitoringAlertRule, type MetricSummary } from '../lib/api'
import {
  Activity,
  AlertTriangle,
  Bell,
  CheckCircle,
  Clock,
  Database,
  Eye,
  EyeOff,
  Gauge,
  GitBranch,
  HelpCircle,
  Loader2,
  Plus,
  RefreshCw,
  Shield,
  Trash2,
  TrendingDown,
  TrendingUp,
  XCircle,
} from '../components/icons'

export default function MonitoringDashboard() {
  const [dashboard, setDashboard] = useState<MonitoringDashboard | null>(null)
  const [alerts, setAlerts] = useState<MonitoringAlert[]>([])
  const [rules, setRules] = useState<MonitoringAlertRule[]>([])
  const [summaries, setSummaries] = useState<MetricSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [collecting, setCollecting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'alerts' | 'rules' | 'metrics'>('overview')
  const [windowHours, setWindowHours] = useState(24)
  const [showNewRule, setShowNewRule] = useState(false)

  // New rule form state
  const [newRuleName, setNewRuleName] = useState('')
  const [newRuleMetric, setNewRuleMetric] = useState('')
  const [newRuleCondition, setNewRuleCondition] = useState('above')
  const [newRuleThreshold, setNewRuleThreshold] = useState(0)
  const [newRuleSeverity, setNewRuleSeverity] = useState('warning')
  const [newRuleDescription, setNewRuleDescription] = useState('')

  const loadAll = async () => {
    setLoading(true)
    setError(null)
    try {
      const [dash, alertsRes, rulesRes, summaryRes] = await Promise.all([
        api.getMonitoringDashboard().catch(() => null),
        api.getMonitoringAlerts().catch(() => ({ alerts: [], count: 0 })),
        api.getMonitoringAlertRules().catch(() => ({ rules: [], count: 0 })),
        api.getMonitoringMetricsSummary(windowHours).catch(() => ({ summaries: [], window_hours: windowHours, count: 0 })),
      ])

      if (dash) setDashboard(dash)
      setAlerts(alertsRes.alerts ?? [])
      setRules(rulesRes.rules ?? [])
      setSummaries(summaryRes.summaries ?? [])
    } catch (e: any) {
      setError(e.message || 'Failed to load monitoring data')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadAll()
  }, [windowHours])

  const handleCollect = async () => {
    setCollecting(true)
    try {
      await api.collectMonitoringMetrics()
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to collect metrics')
    } finally {
      setCollecting(false)
    }
  }

  const handleEvaluate = async () => {
    try {
      await api.evaluateMonitoringAlerts()
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to evaluate alerts')
    }
  }

  const handleAcknowledge = async (alertId: string) => {
    try {
      await api.acknowledgeMonitoringAlert(alertId)
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to acknowledge alert')
    }
  }

  const handleResolve = async (alertId: string) => {
    try {
      await api.resolveMonitoringAlert(alertId)
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to resolve alert')
    }
  }

  const handleToggleRule = async (ruleId: string, currentEnabled: number) => {
    try {
      await api.updateMonitoringAlertRule(ruleId, { enabled: currentEnabled === 1 ? 0 : 1 })
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to update rule')
    }
  }

  const handleDeleteRule = async (ruleId: string) => {
    if (!confirm('Delete this alert rule?')) return
    try {
      await api.deleteMonitoringAlertRule(ruleId)
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to delete rule')
    }
  }

  const handleCreateRule = async () => {
    if (!newRuleName.trim() || !newRuleMetric.trim()) {
      setError('Rule name and metric are required')
      return
    }
    try {
      await api.createMonitoringAlertRule({
        name: newRuleName.trim(),
        description: newRuleDescription.trim(),
        metric_name: newRuleMetric.trim(),
        condition: newRuleCondition,
        threshold: newRuleThreshold,
        severity: newRuleSeverity,
      })
      setShowNewRule(false)
      setNewRuleName('')
      setNewRuleMetric('')
      setNewRuleCondition('above')
      setNewRuleThreshold(0)
      setNewRuleSeverity('warning')
      setNewRuleDescription('')
      await loadAll()
    } catch (e: any) {
      setError(e.message || 'Failed to create rule')
    }
  }

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'text-red-600 bg-red-50 border-red-200'
      case 'warning': return 'text-amber-600 bg-amber-50 border-amber-200'
      default: return 'text-blue-600 bg-blue-50 border-blue-200'
    }
  }

  const severityDot = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500'
      case 'warning': return 'bg-amber-500'
      default: return 'bg-blue-500'
    }
  }

  const healthStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'text-emerald-600 bg-emerald-50 border-emerald-200'
      case 'degraded': return 'text-amber-600 bg-amber-50 border-amber-200'
      case 'unhealthy': return 'text-red-600 bg-red-50 border-red-200'
      default: return 'text-gray-600 bg-gray-50 border-gray-200'
    }
  }

  const healthIcon = (status: string) => {
    switch (status) {
      case 'healthy': return <CheckCircle className="h-5 w-5 text-emerald-600" />
      case 'degraded': return <AlertTriangle className="h-5 w-5 text-amber-600" />
      case 'unhealthy': return <XCircle className="h-5 w-5 text-red-600" />
      default: return <HelpCircle className="h-5 w-5 text-gray-400" />
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-8 w-8 animate-spin text-[var(--color-ink-muted)]" />
        <span className="ml-3 text-[var(--color-ink-muted)]">Loading monitoring data...</span>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-ink)]">Monitoring & Alerting</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-1">
            System health, AI performance metrics, and operational alerts
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={windowHours}
            onChange={(e) => setWindowHours(Number(e.target.value))}
            className="rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm bg-white"
          >
            <option value={1}>Last 1 hour</option>
            <option value={6}>Last 6 hours</option>
            <option value={24}>Last 24 hours</option>
            <option value={168}>Last 7 days</option>
          </select>
          <button
            onClick={handleCollect}
            disabled={collecting}
            className="inline-flex items-center gap-2 rounded-[var(--radius-md)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            {collecting ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            Collect Metrics
          </button>
          <button
            onClick={handleEvaluate}
            className="inline-flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--color-border)] px-4 py-2 text-sm font-medium hover:bg-gray-50"
          >
            <Bell className="h-4 w-4" />
            Evaluate Alerts
          </button>
        </div>
      </div>

      {error && (
        <div className="rounded-[var(--radius-md)] border border-red-200 bg-red-50 p-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 rounded-[var(--radius-md)] bg-gray-100 p-1">
        {(['overview', 'alerts', 'rules', 'metrics'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`flex-1 rounded-[var(--radius-sm)] px-4 py-2 text-sm font-medium capitalize transition-colors ${
              activeTab === tab
                ? 'bg-white text-[var(--color-ink)] shadow-sm'
                : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Health Status Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Overall Health */}
            <div className={`rounded-[var(--radius-lg)] border p-5 ${healthStatusColor(dashboard?.health.overallStatus || 'healthy')}`}>
              <div className="flex items-center gap-3">
                {healthIcon(dashboard?.health.overallStatus || 'healthy')}
                <div>
                  <p className="text-sm font-medium opacity-80">Overall Status</p>
                  <p className="text-2xl font-bold capitalize">{dashboard?.health.overallStatus || 'unknown'}</p>
                </div>
              </div>
            </div>

            {/* Active Alerts */}
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5">
              <div className="flex items-center gap-3">
                <Bell className="h-5 w-5 text-[var(--color-ink-muted)]" />
                <div>
                  <p className="text-sm font-medium text-[var(--color-ink-muted)]">Active Alerts</p>
                  <p className="text-2xl font-bold text-[var(--color-ink)]">{dashboard?.activeAlerts || 0}</p>
                </div>
              </div>
              <p className="text-xs text-[var(--color-ink-faint)] mt-1">
                {dashboard?.pendingAlerts || 0} pending acknowledgment
              </p>
            </div>

            {/* Satisfaction Rate */}
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5">
              <div className="flex items-center gap-3">
                <TrendingUp className="h-5 w-5 text-[var(--color-ink-muted)]" />
                <div>
                  <p className="text-sm font-medium text-[var(--color-ink-muted)]">Satisfaction Rate</p>
                  <p className="text-2xl font-bold text-[var(--color-ink)]">
                    {Math.round((dashboard?.feedbackSummary.satisfactionRate || 0) * 100)}%
                  </p>
                </div>
              </div>
              <p className="text-xs text-[var(--color-ink-faint)] mt-1">
                {dashboard?.feedbackSummary.totalRatings || 0} ratings in last 24h
              </p>
            </div>
          </div>

          {/* Component Health */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5">
            <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">Component Health</h3>
            <div className="space-y-3">
              {dashboard?.health.components.map((comp) => (
                <div key={comp.name} className="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--color-border)] p-3">
                  <div className="flex items-center gap-3">
                    {comp.name === 'database' ? (
                      <Database className="h-4 w-4 text-[var(--color-ink-muted)]" />
                    ) : comp.name === 'claude_api' ? (
                      <GitBranch className="h-4 w-4 text-[var(--color-ink-muted)]" />
                    ) : (
                      <Activity className="h-4 w-4 text-[var(--color-ink-muted)]" />
                    )}
                    <div>
                      <p className="text-sm font-medium text-[var(--color-ink)] capitalize">
                        {comp.name.replace('_', ' ')}
                      </p>
                      {comp.details && (
                        <p className="text-xs text-[var(--color-ink-faint)]">{comp.details}</p>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs text-[var(--color-ink-faint)]">{comp.latencyMs}ms</span>
                    <span className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${healthStatusColor(comp.status)}`}>
                      <span className={`h-1.5 w-1.5 rounded-full ${
                        comp.status === 'healthy' ? 'bg-emerald-500' : comp.status === 'degraded' ? 'bg-amber-500' : 'bg-red-500'
                      }`} />
                      {comp.status}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Key Metrics */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5">
            <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">Key Metrics Summary</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {summaries.map((s) => (
                <div key={s.metricName} className="rounded-[var(--radius-md)] border border-[var(--color-border)] p-4">
                  <p className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
                    {s.metricName.replace(/_/g, ' ')}
                  </p>
                  <p className="text-2xl font-bold text-[var(--color-ink)] mt-1">
                    {s.latestValue % 1 === 0 ? s.latestValue : s.latestValue.toFixed(2)}
                  </p>
                  <div className="flex items-center gap-3 mt-2 text-xs text-[var(--color-ink-faint)]">
                    <span>avg: {s.avgValue.toFixed(2)}</span>
                    <span>min: {s.minValue.toFixed(2)}</span>
                    <span>max: {s.maxValue.toFixed(2)}</span>
                  </div>
                  <p className="text-xs text-[var(--color-ink-faint)] mt-1">{s.sampleCount} samples</p>
                </div>
              ))}
              {summaries.length === 0 && (
                <p className="text-sm text-[var(--color-ink-muted)] col-span-full">
                  No metrics collected yet. Click "Collect Metrics" to gather data.
                </p>
              )}
            </div>
          </div>

          {/* Recent Negative Feedback */}
          {dashboard?.feedbackSummary.recentNegative.length > 0 && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5">
              <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">
                Recent Negative Feedback ({dashboard.feedbackSummary.recentNegative.length})
              </h3>
              <div className="space-y-2">
                {dashboard.feedbackSummary.recentNegative.map((item, i) => (
                  <div key={i} className="rounded-[var(--radius-md)] border border-red-100 bg-red-50 p-3">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-medium text-red-700">{item.authorType}</span>
                      <span className="text-xs text-red-500">{new Date(item.createdAt).toLocaleString()}</span>
                    </div>
                    {item.comment && (
                      <p className="text-sm text-red-700 mt-1">{item.comment}</p>
                    )}
                    <p className="text-xs text-red-500 mt-1">
                      Message: {item.messageId.slice(0, 12)}... | Conversation: {item.conversationId.slice(0, 12)}...
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Alerts Tab */}
      {activeTab === 'alerts' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-[var(--color-ink)]">Alert Events</h3>
            <span className="text-sm text-[var(--color-ink-muted)]">
              {alerts.length} total alerts
            </span>
          </div>

          {alerts.length === 0 ? (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-12 text-center">
              <Shield className="h-12 w-12 mx-auto text-[var(--color-ink-faint)]" />
              <p className="text-[var(--color-ink-muted)] mt-3">No alerts — everything looks good!</p>
            </div>
          ) : (
            <div className="space-y-3">
              {alerts.map((alert) => (
                <div
                  key={alert.id}
                  className={`rounded-[var(--radius-lg)] border p-4 ${severityColor(alert.severity)}`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-3">
                      <span className={`mt-1 h-2.5 w-2.5 rounded-full flex-shrink-0 ${severityDot(alert.severity)}`} />
                      <div>
                        <div className="flex items-center gap-2">
                          <h4 className="font-semibold">{alert.alertName}</h4>
                          <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-white/50 capitalize">
                            {alert.severity}
                          </span>
                          {alert.acknowledged === 1 && (
                            <span className="text-xs text-[var(--color-ink-muted)] flex items-center gap-1">
                              <CheckCircle className="h-3 w-3" /> Acknowledged
                            </span>
                          )}
                          {alert.resolved === 1 && (
                            <span className="text-xs text-emerald-600 flex items-center gap-1">
                              <CheckCircle className="h-3 w-3" /> Resolved
                            </span>
                          )}
                        </div>
                        <p className="text-sm mt-1 opacity-80">{alert.message}</p>
                        <div className="flex items-center gap-4 mt-2 text-xs opacity-60">
                          <span className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {new Date(alert.createdAt).toLocaleString()}
                          </span>
                          <span>Metric: {alert.metricName}</span>
                          <span>Value: {alert.metricValue.toFixed(4)} (threshold: {alert.threshold})</span>
                        </div>
                        {alert.acknowledgedBy && (
                          <p className="text-xs mt-1 opacity-60">
                            Acknowledged by {alert.acknowledgedBy} at {alert.acknowledgedAt ? new Date(alert.acknowledgedAt).toLocaleString() : 'unknown'}
                          </p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {alert.acknowledged === 0 && alert.resolved === 0 && (
                        <>
                          <button
                            onClick={() => handleAcknowledge(alert.id)}
                            className="rounded-[var(--radius-sm)] px-3 py-1 text-xs font-medium bg-white/50 hover:bg-white/80 transition-colors"
                          >
                            Acknowledge
                          </button>
                          <button
                            onClick={() => handleResolve(alert.id)}
                            className="rounded-[var(--radius-sm)] px-3 py-1 text-xs font-medium bg-emerald-100 text-emerald-700 hover:bg-emerald-200 transition-colors"
                          >
                            Resolve
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Rules Tab */}
      {activeTab === 'rules' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-[var(--color-ink)]">Alert Rules</h3>
            <button
              onClick={() => setShowNewRule(!showNewRule)}
              className="inline-flex items-center gap-2 rounded-[var(--radius-md)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
            >
              <Plus className="h-4 w-4" />
              New Rule
            </button>
          </div>

          {/* New Rule Form */}
          {showNewRule && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5 space-y-4">
              <h4 className="font-semibold text-[var(--color-ink)]">Create Alert Rule</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Name</label>
                  <input
                    type="text"
                    value={newRuleName}
                    onChange={(e) => setNewRuleName(e.target.value)}
                    placeholder="e.g. High Error Rate"
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Metric</label>
                  <input
                    type="text"
                    value={newRuleMetric}
                    onChange={(e) => setNewRuleMetric(e.target.value)}
                    placeholder="e.g. error_rate"
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Condition</label>
                  <select
                    value={newRuleCondition}
                    onChange={(e) => setNewRuleCondition(e.target.value)}
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm"
                  >
                    <option value="above">Above</option>
                    <option value="below">Below</option>
                    <option value="equal">Equal</option>
                    <option value="not_equal">Not Equal</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Threshold</label>
                  <input
                    type="number"
                    step="any"
                    value={newRuleThreshold}
                    onChange={(e) => setNewRuleThreshold(Number(e.target.value))}
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Severity</label>
                  <select
                    value={newRuleSeverity}
                    onChange={(e) => setNewRuleSeverity(e.target.value)}
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm"
                  >
                    <option value="info">Info</option>
                    <option value="warning">Warning</option>
                    <option value="critical">Critical</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Description</label>
                  <input
                    type="text"
                    value={newRuleDescription}
                    onChange={(e) => setNewRuleDescription(e.target.value)}
                    placeholder="What happens when this triggers"
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm"
                  />
                </div>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={handleCreateRule}
                  className="rounded-[var(--radius-md)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
                >
                  Create Rule
                </button>
                <button
                  onClick={() => setShowNewRule(false)}
                  className="rounded-[var(--radius-md)] border border-[var(--color-border)] px-4 py-2 text-sm hover:bg-gray-50"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {/* Rules List */}
          <div className="space-y-3">
            {rules.map((rule) => (
              <div
                key={rule.id}
                className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-3">
                    <span className={`mt-1.5 h-2.5 w-2.5 rounded-full ${severityDot(rule.severity)}`} />
                    <div>
                      <div className="flex items-center gap-2">
                        <h4 className="font-semibold text-[var(--color-ink)]">{rule.name}</h4>
                        <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${severityColor(rule.severity)}`}>
                          {rule.severity}
                        </span>
                        {rule.enabled === 0 && (
                          <span className="text-xs text-[var(--color-ink-faint)]">Disabled</span>
                        )}
                      </div>
                      {rule.description && (
                        <p className="text-sm text-[var(--color-ink-muted)] mt-1">{rule.description}</p>
                      )}
                      <div className="flex items-center gap-4 mt-2 text-xs text-[var(--color-ink-faint)]">
                        <span>Metric: <code className="bg-gray-100 px-1 rounded">{rule.metricName}</code></span>
                        <span>Condition: {rule.condition} {rule.threshold}</span>
                        <span>Window: {rule.windowMinutes}m</span>
                        <span>Cooldown: {rule.cooldownMinutes}m</span>
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleToggleRule(rule.id, rule.enabled)}
                      className="rounded-[var(--radius-sm)] p-1.5 hover:bg-gray-100"
                      title={rule.enabled === 1 ? 'Disable rule' : 'Enable rule'}
                    >
                      {rule.enabled === 1 ? (
                        <Eye className="h-4 w-4 text-[var(--color-ink-muted)]" />
                      ) : (
                        <EyeOff className="h-4 w-4 text-[var(--color-ink-faint)]" />
                      )}
                    </button>
                    <button
                      onClick={() => handleDeleteRule(rule.id)}
                      className="rounded-[var(--radius-sm)] p-1.5 hover:bg-red-50 text-red-400 hover:text-red-600"
                      title="Delete rule"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Metrics Tab */}
      {activeTab === 'metrics' && (
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-[var(--color-ink)]">Metric Details</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {summaries.map((s) => (
              <div key={s.metricName} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5">
                <div className="flex items-center justify-between mb-3">
                  <h4 className="font-semibold text-[var(--color-ink)] capitalize">
                    {s.metricName.replace(/_/g, ' ')}
                  </h4>
                  <Gauge className="h-4 w-4 text-[var(--color-ink-muted)]" />
                </div>

                <div className="grid grid-cols-2 gap-3 mb-4">
                  <div className="rounded-[var(--radius-md)] bg-gray-50 p-3">
                    <p className="text-xs text-[var(--color-ink-muted)]">Latest</p>
                    <p className="text-xl font-bold text-[var(--color-ink)]">
                      {s.latestValue % 1 === 0 ? s.latestValue : s.latestValue.toFixed(2)}
                    </p>
                  </div>
                  <div className="rounded-[var(--radius-md)] bg-gray-50 p-3">
                    <p className="text-xs text-[var(--color-ink-muted)]">Average</p>
                    <p className="text-xl font-bold text-[var(--color-ink)]">
                      {s.avgValue.toFixed(2)}
                    </p>
                  </div>
                  <div className="rounded-[var(--radius-md)] bg-gray-50 p-3">
                    <p className="text-xs text-[var(--color-ink-muted)]">Min</p>
                    <p className="text-lg font-medium text-[var(--color-ink)]">
                      {s.minValue.toFixed(2)}
                    </p>
                  </div>
                  <div className="rounded-[var(--radius-md)] bg-gray-50 p-3">
                    <p className="text-xs text-[var(--color-ink-muted)]">Max</p>
                    <p className="text-lg font-medium text-[var(--color-ink)]">
                      {s.maxValue.toFixed(2)}
                    </p>
                  </div>
                </div>

                {/* Range bar */}
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-xs text-[var(--color-ink-faint)]">
                    <span>Range</span>
                    <span>{s.sampleCount} samples</span>
                  </div>
                  <div className="h-2 rounded-full bg-gray-200 overflow-hidden">
                    <div
                      className="h-full rounded-full bg-[var(--color-accent)]"
                      style={{
                        width: s.maxValue !== s.minValue
                          ? `${((s.latestValue - s.minValue) / (s.maxValue - s.minValue)) * 100}%`
                          : '100%',
                      }}
                    />
                  </div>
                  <div className="flex items-center justify-between text-xs text-[var(--color-ink-faint)]">
                    <span>{s.minValue.toFixed(2)}</span>
                    <span>{s.maxValue.toFixed(2)}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
          {summaries.length === 0 && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-12 text-center">
              <Gauge className="h-12 w-12 mx-auto text-[var(--color-ink-faint)]" />
              <p className="text-[var(--color-ink-muted)] mt-3">No metrics collected yet.</p>
              <button
                onClick={handleCollect}
                disabled={collecting}
                className="mt-4 inline-flex items-center gap-2 rounded-[var(--radius-md)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
              >
                {collecting ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
                Collect Metrics
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
