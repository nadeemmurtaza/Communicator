import { useState, useEffect } from 'react'
import {
  getOperatorSummary,
  runOperatorScan,
  listOperatorProposals,
  approveOperatorProposal,
  dismissOperatorProposal,
  listOperatorLogs,
  updateOperatorConfig,
  getOperatorConfig,
  type OperatorProposal,
  type OperatorPerformanceLog,
  type OperatorConfig,
} from '../lib/api'
import {
  Cpu,
  Lightbulb,
  FileText,
  Activity,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  Sparkles,
  Settings,
  Play,
  RefreshCw,
  ChevronDown,
  ChevronUp,
} from '../components/icons'

type Tab = 'overview' | 'proposals' | 'logs' | 'config'

function statusColor(status: string) {
  switch (status) {
    case 'proposed': return 'bg-yellow-100 text-yellow-800'
    case 'reviewing': return 'bg-blue-100 text-blue-800'
    case 'approved': return 'bg-green-100 text-green-800'
    case 'testing': return 'bg-purple-100 text-purple-800'
    case 'deployed': return 'bg-emerald-100 text-emerald-800'
    case 'dismissed': return 'bg-gray-100 text-gray-600'
    case 'failed': return 'bg-red-100 text-red-800'
    default: return 'bg-gray-100 text-gray-800'
  }
}

function priorityColor(priority: string) {
  switch (priority) {
    case 'critical': return 'bg-red-100 text-red-800'
    case 'high': return 'bg-orange-100 text-orange-800'
    case 'medium': return 'bg-yellow-100 text-yellow-800'
    case 'low': return 'bg-green-100 text-green-800'
    default: return 'bg-gray-100 text-gray-800'
  }
}

function resultColor(result: string) {
  switch (result) {
    case 'success': return 'text-green-600'
    case 'failure': return 'text-red-600'
    case 'partial': return 'text-yellow-600'
    case 'skipped': return 'text-gray-500'
    default: return 'text-gray-600'
  }
}

export function FinOperatorDashboard() {
  const [activeTab, setActiveTab] = useState<Tab>('overview')
  const [loading, setLoading] = useState(true)
  const [scanning, setScanning] = useState(false)
  const [summary, setSummary] = useState<any>(null)
  const [proposals, setProposals] = useState<OperatorProposal[]>([])
  const [proposalStatus, setProposalStatus] = useState<string>('')
  const [proposalType, setProposalType] = useState<string>('')
  const [logs, setLogs] = useState<OperatorPerformanceLog[]>([])
  const [config, setConfig] = useState<OperatorConfig | null>(null)
  const [expandedProposal, setExpandedProposal] = useState<string | null>(null)
  const [configForm, setConfigForm] = useState<Partial<OperatorConfig>>({})
  const [savingConfig, setSavingConfig] = useState(false)

  async function loadSummary() {
    try {
      const data = await getOperatorSummary()
      setSummary(data)
    } catch (e) {
      console.error('Failed to load operator summary:', e)
    }
  }

  async function loadProposals() {
    try {
      const data = await listOperatorProposals(proposalStatus || undefined, proposalType || undefined)
      setProposals(data.proposals || [])
    } catch (e) {
      console.error('Failed to load proposals:', e)
    }
  }

  async function loadLogs() {
    try {
      const data = await listOperatorLogs()
      setLogs(data.logs || [])
    } catch (e) {
      console.error('Failed to load logs:', e)
    }
  }

  async function loadConfig() {
    try {
      const data = await getOperatorConfig()
      setConfig(data)
      setConfigForm(data)
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  useEffect(() => {
    loadSummary()
    loadProposals()
    loadLogs()
    loadConfig()
    setLoading(false)
  }, [])

  useEffect(() => {
    loadProposals()
  }, [proposalStatus, proposalType])

  async function handleScan() {
    setScanning(true)
    try {
      await runOperatorScan()
      await Promise.all([loadSummary(), loadProposals(), loadLogs()])
    } catch (e) {
      console.error('Scan failed:', e)
    }
    setScanning(false)
  }

  async function handleApprove(id: string) {
    try {
      await approveOperatorProposal(id)
      await Promise.all([loadSummary(), loadProposals()])
    } catch (e) {
      console.error('Approve failed:', e)
    }
  }

  async function handleDismiss(id: string) {
    try {
      await dismissOperatorProposal(id)
      await Promise.all([loadSummary(), loadProposals()])
    } catch (e) {
      console.error('Dismiss failed:', e)
    }
  }

  async function handleSaveConfig() {
    setSavingConfig(true)
    try {
      await updateOperatorConfig(configForm)
      await loadConfig()
    } catch (e) {
      console.error('Save config failed:', e)
    }
    setSavingConfig(false)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading Fin Operator...</div>
      </div>
    )
  }

  const tabs: { key: Tab; label: string; icon: any }[] = [
    { key: 'overview', label: 'Overview', icon: Activity },
    { key: 'proposals', label: 'Proposals', icon: Lightbulb },
    { key: 'logs', label: 'Activity Log', icon: FileText },
    { key: 'config', label: 'Configuration', icon: Settings },
  ]

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Fin Operator</h1>
          <p className="text-sm text-gray-500 mt-1">
            Meta-agent for autonomous self-improvement — monitors metrics, detects anomalies, proposes improvements
          </p>
        </div>
        <button
          onClick={handleScan}
          disabled={scanning}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50"
        >
          {scanning ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
          {scanning ? 'Scanning...' : 'Run Scan'}
        </button>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {tabs.map((tab) => {
            const Icon = tab.icon
            const isActive = activeTab === tab.key
            return (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`flex items-center gap-2 py-3 px-1 border-b-2 text-sm font-medium ${
                  isActive
                    ? 'border-indigo-500 text-indigo-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            )
          })}
        </nav>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Status Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Total Proposals</p>
                  <p className="text-3xl font-bold text-gray-900 mt-1">{summary?.totalProposals || 0}</p>
                </div>
                <Lightbulb className="w-8 h-8 text-yellow-500" />
              </div>
            </div>
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Anomalies Detected</p>
                  <p className="text-3xl font-bold text-gray-900 mt-1">{summary?.anomalyCount || 0}</p>
                </div>
                <AlertTriangle className="w-8 h-8 text-orange-500" />
              </div>
            </div>
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Operator Status</p>
                  <p className="text-lg font-bold mt-1">
                    {summary?.config?.enabled ? (
                      <span className="text-green-600">Active</span>
                    ) : (
                      <span className="text-gray-400">Disabled</span>
                    )}
                  </p>
                </div>
                <Cpu className="w-8 h-8 text-indigo-500" />
              </div>
            </div>
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Last Scan</p>
                  <p className="text-sm font-medium mt-1 text-gray-900">
                    {summary?.config?.lastScanAt
                      ? new Date(summary.config.lastScanAt).toLocaleString()
                      : 'Never'}
                  </p>
                </div>
                <Clock className="w-8 h-8 text-blue-500" />
              </div>
            </div>
          </div>

          {/* Metrics Snapshot */}
          {summary?.latestSnapshot && (
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Current Metrics (7-day)</h3>
              <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                {Object.entries(summary.latestSnapshot.metricsData || {}).map(([key, value]) => (
                  <div key={key} className="p-4 bg-gray-50 rounded-lg">
                    <p className="text-xs text-gray-500 uppercase tracking-wide">{key.replace(/_/g, ' ')}</p>
                    <p className="text-xl font-bold text-gray-900 mt-1">
                      {typeof value === 'number' ? (value * 100).toFixed(1) + '%' : value ?? 'N/A'}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Anomalies */}
          {summary?.latestSnapshot?.anomalies && summary.latestSnapshot.anomalies.length > 0 && (
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-orange-500" />
                Detected Anomalies
              </h3>
              <div className="space-y-2">
                {summary.latestSnapshot.anomalies.map((anomaly: any, i: number) => (
                  <div
                    key={i}
                    className={`p-3 rounded-lg ${
                      anomaly.severity === 'high' ? 'bg-red-50 border border-red-200' : 'bg-yellow-50 border border-yellow-200'
                    }`}
                  >
                    <p className="text-sm font-medium text-gray-900">{anomaly.message}</p>
                    <p className="text-xs text-gray-500 mt-1">
                      Metric: {anomaly.metric} | Change: {(anomaly.change * 100).toFixed(1)}% | Severity: {anomaly.severity}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Proposals by Status */}
          <div className="bg-white p-6 rounded-lg border border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Proposals by Status</h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
              {Object.entries(summary?.proposalsByStatus || {}).map(([status, count]) => (
                <div key={status} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusColor(status)}`}>
                    {status}
                  </span>
                  <span className="text-lg font-bold text-gray-900">{count as number}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Recent Activity */}
          {summary?.recentLogs && summary.recentLogs.length > 0 && (
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Activity</h3>
              <div className="space-y-2">
                {summary.recentLogs.slice(0, 5).map((log: OperatorPerformanceLog) => (
                  <div key={log.id} className="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                    <span className={`text-xs font-medium px-2 py-1 rounded ${resultColor(log.result)}`}>
                      {log.result}
                    </span>
                    <span className="text-sm text-gray-700 flex-1">{log.actionDetail}</span>
                    <span className="text-xs text-gray-400">
                      {new Date(log.createdAt).toLocaleString()}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Proposals Tab */}
      {activeTab === 'proposals' && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex gap-4">
            <select
              value={proposalStatus}
              onChange={(e) => setProposalStatus(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm"
            >
              <option value="">All Statuses</option>
              <option value="proposed">Proposed</option>
              <option value="reviewing">Reviewing</option>
              <option value="approved">Approved</option>
              <option value="testing">Testing</option>
              <option value="deployed">Deployed</option>
              <option value="dismissed">Dismissed</option>
              <option value="failed">Failed</option>
            </select>
            <select
              value={proposalType}
              onChange={(e) => setProposalType(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-lg text-sm"
            >
              <option value="">All Types</option>
              <option value="knowledge_update">Knowledge Update</option>
              <option value="procedure_change">Procedure Change</option>
              <option value="policy_update">Policy Update</option>
              <option value="prompt_tuning">Prompt Tuning</option>
              <option value="threshold_adjustment">Threshold Adjustment</option>
              <option value="content_creation">Content Creation</option>
              <option value="escalation_rule">Escalation Rule</option>
              <option value="general">General</option>
            </select>
          </div>

          {/* Proposal List */}
          {proposals.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <Lightbulb className="w-12 h-12 mx-auto text-gray-300 mb-3" />
              <p>No proposals yet. Run a scan to generate improvement proposals.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {proposals.map((proposal) => (
                <div key={proposal.id} className="bg-white rounded-lg border border-gray-200 overflow-hidden">
                  <div className="p-4">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2">
                          <h4 className="font-semibold text-gray-900">{proposal.title}</h4>
                          <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${statusColor(proposal.status)}`}>
                            {proposal.status}
                          </span>
                          <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${priorityColor(proposal.priority)}`}>
                            {proposal.priority}
                          </span>
                        </div>
                        <p className="text-sm text-gray-600">{proposal.description}</p>
                        <div className="flex items-center gap-4 mt-2 text-xs text-gray-400">
                          <span>Confidence: {(proposal.confidence * 100).toFixed(0)}%</span>
                          <span>Type: {proposal.proposalType}</span>
                          <span>Source: {proposal.source}</span>
                          <span>{new Date(proposal.createdAt).toLocaleString()}</span>
                        </div>
                      </div>
                      {(proposal.status === 'proposed' || proposal.status === 'reviewing') && (
                        <div className="flex gap-2">
                          <button
                            onClick={() => handleApprove(proposal.id)}
                            className="p-2 text-green-600 hover:bg-green-50 rounded-lg"
                            title="Approve"
                          >
                            <CheckCircle className="w-5 h-5" />
                          </button>
                          <button
                            onClick={() => handleDismiss(proposal.id)}
                            className="p-2 text-red-600 hover:bg-red-50 rounded-lg"
                            title="Dismiss"
                          >
                            <XCircle className="w-5 h-5" />
                          </button>
                        </div>
                      )}
                    </div>

                    {/* Expanded details */}
                    {expandedProposal === proposal.id && (
                      <div className="mt-4 pt-4 border-t border-gray-100 space-y-3">
                        <div>
                          <h5 className="text-sm font-medium text-gray-700">Rationale</h5>
                          <p className="text-sm text-gray-600 mt-1">{proposal.rationale}</p>
                        </div>
                        {proposal.expectedImpact && (
                          <div>
                            <h5 className="text-sm font-medium text-gray-700">Expected Impact</h5>
                            <pre className="text-xs text-gray-600 mt-1 bg-gray-50 p-2 rounded">
                              {JSON.stringify(proposal.expectedImpact, null, 2)}
                            </pre>
                          </div>
                        )}
                        {proposal.implementationPlan && (
                          <div>
                            <h5 className="text-sm font-medium text-gray-700">Implementation Plan</h5>
                            <pre className="text-xs text-gray-600 mt-1 bg-gray-50 p-2 rounded">
                              {JSON.stringify(proposal.implementationPlan, null, 2)}
                            </pre>
                          </div>
                        )}
                        {proposal.reviewedBy && (
                          <div className="text-xs text-gray-400">
                            Reviewed by {proposal.reviewedBy} at {proposal.reviewedAt ? new Date(proposal.reviewedAt).toLocaleString() : 'N/A'}
                          </div>
                        )}
                      </div>
                    )}

                    <button
                      onClick={() => setExpandedProposal(expandedProposal === proposal.id ? null : proposal.id)}
                      className="flex items-center gap-1 text-xs text-indigo-600 hover:text-indigo-800 mt-2"
                    >
                      {expandedProposal === proposal.id ? (
                        <>
                          <ChevronUp className="w-3 h-3" /> Less details
                        </>
                      ) : (
                        <>
                          <ChevronDown className="w-3 h-3" /> More details
                        </>
                      )}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Logs Tab */}
      {activeTab === 'logs' && (
        <div className="space-y-3">
          {logs.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <Activity className="w-12 h-12 mx-auto text-gray-300 mb-3" />
              <p>No activity logs yet. Run a scan to start logging operator actions.</p>
            </div>
          ) : (
            <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-gray-50 border-b border-gray-200">
                  <tr>
                    <th className="text-left px-4 py-3 font-medium text-gray-500">Time</th>
                    <th className="text-left px-4 py-3 font-medium text-gray-500">Action</th>
                    <th className="text-left px-4 py-3 font-medium text-gray-500">Detail</th>
                    <th className="text-left px-4 py-3 font-medium text-gray-500">Result</th>
                    <th className="text-left px-4 py-3 font-medium text-gray-500">Duration</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  {logs.map((log) => (
                    <tr key={log.id} className="hover:bg-gray-50">
                      <td className="px-4 py-3 text-gray-500 whitespace-nowrap">
                        {new Date(log.createdAt).toLocaleString()}
                      </td>
                      <td className="px-4 py-3">
                        <span className="px-2 py-0.5 bg-indigo-50 text-indigo-700 rounded text-xs font-medium">
                          {log.actionType}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-gray-700 max-w-md truncate">{log.actionDetail}</td>
                      <td className={`px-4 py-3 font-medium ${resultColor(log.result)}`}>{log.result}</td>
                      <td className="px-4 py-3 text-gray-500">
                        {log.durationMs ? `${log.durationMs}ms` : '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Config Tab */}
      {activeTab === 'config' && config && (
        <div className="max-w-2xl space-y-6">
          <div className="bg-white p-6 rounded-lg border border-gray-200 space-y-4">
            <h3 className="text-lg font-semibold text-gray-900">Operator Configuration</h3>

            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Enabled</label>
                <p className="text-xs text-gray-500">Enable/disable the operator</p>
              </div>
              <button
                onClick={() => setConfigForm({ ...configForm, enabled: !configForm.enabled })}
                className={`relative w-12 h-6 rounded-full transition ${configForm.enabled ? 'bg-indigo-600' : 'bg-gray-300'}`}
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    configForm.enabled ? 'translate-x-6' : ''
                  }`}
                />
              </button>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-700">Scan Interval (minutes)</label>
              <input
                type="number"
                value={configForm.scanIntervalMinutes || 60}
                onChange={(e) => setConfigForm({ ...configForm, scanIntervalMinutes: parseInt(e.target.value) })}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>

            <div>
              <label className="text-sm font-medium text-gray-700">Max Proposals per Scan</label>
              <input
                type="number"
                value={configForm.maxProposalsPerScan || 5}
                onChange={(e) => setConfigForm({ ...configForm, maxProposalsPerScan: parseInt(e.target.value) })}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>

            <div>
              <label className="text-sm font-medium text-gray-700">Min Confidence Threshold</label>
              <input
                type="number"
                step="0.1"
                min="0"
                max="1"
                value={configForm.minConfidenceThreshold ?? 0.7}
                onChange={(e) => setConfigForm({ ...configForm, minConfidenceThreshold: parseFloat(e.target.value) })}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Auto-Deploy</label>
                <p className="text-xs text-gray-500">Automatically deploy approved proposals</p>
              </div>
              <button
                onClick={() => setConfigForm({ ...configForm, autoDeployEnabled: !configForm.autoDeployEnabled })}
                className={`relative w-12 h-6 rounded-full transition ${configForm.autoDeployEnabled ? 'bg-indigo-600' : 'bg-gray-300'}`}
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    configForm.autoDeployEnabled ? 'translate-x-6' : ''
                  }`}
                />
              </button>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-700">Auto-Deploy Max Risk</label>
              <select
                value={configForm.autoDeployMaxRisk || 'low'}
                onChange={(e) => setConfigForm({ ...configForm, autoDeployMaxRisk: e.target.value })}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
              </select>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium text-gray-700">Alert on Failure</label>
                <p className="text-xs text-gray-500">Send alerts when operator actions fail</p>
              </div>
              <button
                onClick={() => setConfigForm({ ...configForm, alertOnFailure: !configForm.alertOnFailure })}
                className={`relative w-12 h-6 rounded-full transition ${configForm.alertOnFailure ? 'bg-indigo-600' : 'bg-gray-300'}`}
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    configForm.alertOnFailure ? 'translate-x-6' : ''
                  }`}
                />
              </button>
            </div>

            <div className="pt-4 border-t border-gray-100">
              <button
                onClick={handleSaveConfig}
                disabled={savingConfig}
                className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 text-sm"
              >
                {savingConfig ? 'Saving...' : 'Save Configuration'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
