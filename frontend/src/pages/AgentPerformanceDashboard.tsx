import {
  UserCheck,
  TrendingUp,
  AlertTriangle,
  Award,
  Lightbulb,
  BarChart3,
  Users,
  Clock,
  Star,
  CheckCircle,
  XCircle,
  Plus,
  RefreshCw,
} from '../components/icons'
import { useEffect, useState } from 'react'
import {
  api,
  type AgentPerformanceSummary,
  type AgentPerformanceSnapshot,
  type CoachingInsight,
  type ComparisonMetrics,
  type AgentPerformanceSettings,
} from '../lib/api'

export function AgentPerformanceDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'snapshots' | 'insights' | 'settings'>('overview')
  const [summary, setSummary] = useState<AgentPerformanceSummary | null>(null)
  const [snapshots, setSnapshots] = useState<AgentPerformanceSnapshot[]>([])
  const [insights, setInsights] = useState<CoachingInsight[]>([])
  const [settings, setSettings] = useState<AgentPerformanceSettings | null>(null)
  const [loading, setLoading] = useState(false)
  const [notice, setNotice] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [showInsightModal, setShowInsightModal] = useState(false)
  const [comparison, setComparison] = useState<ComparisonMetrics[]>([])

  useEffect(() => {
    loadSummary()
    loadSettings()
  }, [])

  useEffect(() => {
    if (activeTab === 'snapshots') loadSnapshots()
    if (activeTab === 'insights') loadInsights()
  }, [activeTab])

  function loadSummary() {
    setLoading(true)
    api.getAgentPerformanceSummary()
      .then(setSummary)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false))
  }

  function loadSnapshots() {
    api.listAgentSnapshots(undefined, 7)
      .then(setSnapshots)
      .catch((e) => setError(e.message))
  }

  function loadInsights() {
    api.listCoachingInsights()
      .then(setInsights)
      .catch((e) => setError(e.message))
  }

  function loadSettings() {
    api.getAgentPerformanceSettings()
      .then(setSettings)
      .catch((e) => setError(e.message))
  }

  async function handleAddressInsight(insightId: string) {
    try {
      await api.addressCoachingInsight(insightId, 'admin')
      setNotice('Insight marked as addressed')
      loadInsights()
      loadSummary()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleGenerateComparison() {
    try {
      await api.generateComparisonSnapshot()
      setNotice('Comparison snapshot generated')
      loadComparison()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function loadComparison() {
    api.getComparisonMetrics(7)
      .then(setComparison)
      .catch((e) => setError(e.message))
  }

  async function handleUpdateSettings(field: string, value: any) {
    try {
      const body: any = { [field]: value }
      await api.updateAgentPerformanceSettings(body)
      setNotice('Settings updated')
      loadSettings()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleCreateInsight(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    try {
      await api.createCoachingInsight({
        agentId: formData.get('agentId') as string,
        agentName: formData.get('agentName') as string,
        title: formData.get('title') as string,
        description: formData.get('description') as string,
        category: formData.get('category') as string,
        severity: formData.get('severity') as string,
        recommendation: formData.get('recommendation') as string,
      })
      setNotice('Insight created')
      setShowInsightModal(false)
      loadInsights()
      loadSummary()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function severityColor(severity: string) {
    switch (severity) {
      case 'high': return '#ef4444'
      case 'medium': return '#f59e0b'
      case 'low': return '#22c55e'
      default: return '#6b7280'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Agent Performance</h1>
          <p className="text-gray-500 mt-1">Per-agent metrics, coaching insights, and team benchmarks</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={loadSummary}
            className="px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg hover:bg-gray-50 flex items-center gap-1"
          >
            <RefreshCw className="w-4 h-4" /> Refresh
          </button>
        </div>
      </div>

      {notice && (
        <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-800 text-sm">
          {notice}
          <button onClick={() => setNotice(null)} className="ml-2 text-green-600 hover:text-green-800">×</button>
        </div>
      )}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          {error}
          <button onClick={() => setError(null)} className="ml-2 text-red-600 hover:text-red-800">×</button>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 bg-gray-100 p-1 rounded-lg w-fit">
        {(['overview', 'snapshots', 'insights', 'settings'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm rounded-md capitalize ${
              activeTab === tab
                ? 'bg-white shadow text-gray-900'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-white p-4 rounded-xl border border-gray-200">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <Users className="w-4 h-4" /> Total Agents
              </div>
              <div className="text-2xl font-bold">{summary?.totalAgents ?? 0}</div>
            </div>
            <div className="bg-white p-4 rounded-xl border border-gray-200">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <TrendingUp className="w-4 h-4" /> Avg Resolution Rate
              </div>
              <div className="text-2xl font-bold">
                {((summary?.teamAvgResolutionRate ?? 0) * 100).toFixed(1)}%
              </div>
            </div>
            <div className="bg-white p-4 rounded-xl border border-gray-200">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <Clock className="w-4 h-4" /> Avg Response Time
              </div>
              <div className="text-2xl font-bold">
                {Math.round(summary?.teamAvgResponseTime ?? 0)}s
              </div>
            </div>
            <div className="bg-white p-4 rounded-xl border border-gray-200">
              <div className="flex items-center gap-2 text-gray-500 text-sm mb-1">
                <Lightbulb className="w-4 h-4" /> Active Insights
              </div>
              <div className="text-2xl font-bold">{summary?.activeInsights ?? 0}</div>
            </div>
          </div>

          {/* Benchmark Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-green-50 p-4 rounded-xl border border-green-200">
              <div className="flex items-center gap-2 text-green-700 text-sm mb-1">
                <CheckCircle className="w-4 h-4" /> Exceeds Benchmark
              </div>
              <div className="text-2xl font-bold text-green-800">{summary?.exceedsBenchmark ?? 0}</div>
            </div>
            <div className="bg-yellow-50 p-4 rounded-xl border border-yellow-200">
              <div className="flex items-center gap-2 text-yellow-700 text-sm mb-1">
                <AlertTriangle className="w-4 h-4" /> Needs Coaching
              </div>
              <div className="text-2xl font-bold text-yellow-800">{summary?.needsCoaching ?? 0}</div>
            </div>
            <div className="bg-blue-50 p-4 rounded-xl border border-blue-200">
              <div className="flex items-center gap-2 text-blue-700 text-sm mb-1">
                <Star className="w-4 h-4" /> Avg Satisfaction
              </div>
              <div className="text-2xl font-bold text-blue-800">
                {summary?.teamAvgSatisfaction?.toFixed(1) ?? 'N/A'}
              </div>
            </div>
          </div>

          {/* Recent Insights */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold mb-4">Recent Coaching Insights</h2>
            {summary?.recentInsights.length === 0 ? (
              <p className="text-gray-400 text-sm">No coaching insights yet</p>
            ) : (
              <div className="space-y-3">
                {summary?.recentInsights.slice(0, 5).map((insight) => (
                  <div key={insight.id} className="flex items-start gap-3 p-3 bg-gray-50 rounded-lg">
                    <div
                      className="w-2 h-2 rounded-full mt-2 flex-shrink-0"
                      style={{ backgroundColor: severityColor(insight.severity) }}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-sm">{insight.title}</span>
                        <span className="text-xs text-gray-400">{insight.agentName}</span>
                        {insight.isAddressed && (
                          <span className="text-xs bg-green-100 text-green-700 px-1.5 py-0.5 rounded">Addressed</span>
                        )}
                      </div>
                      <p className="text-sm text-gray-500 mt-1">{insight.description}</p>
                      {!insight.isAddressed && (
                        <button
                          onClick={() => handleAddressInsight(insight.id)}
                          className="mt-2 text-xs text-blue-600 hover:text-blue-800"
                        >
                          Mark as addressed
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Comparison */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Team Comparison</h2>
              <button
                onClick={handleGenerateComparison}
                className="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-1"
              >
                <BarChart3 className="w-4 h-4" /> Generate Snapshot
              </button>
            </div>
            {comparison.length === 0 ? (
              <p className="text-gray-400 text-sm">No comparison data yet. Generate a snapshot to see team benchmarks.</p>
            ) : (
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <div className="text-xs text-gray-500">Team Avg Response</div>
                  <div className="text-lg font-semibold">{Math.round(comparison[0].teamAvgResponseTime)}s</div>
                </div>
                <div>
                  <div className="text-xs text-gray-500">Team Avg Resolution</div>
                  <div className="text-lg font-semibold">{(comparison[0].teamAvgResolutionRate * 100).toFixed(1)}%</div>
                </div>
                <div>
                  <div className="text-xs text-gray-500">Needs Coaching</div>
                  <div className="text-lg font-semibold text-yellow-600">{comparison[0].needsCoachingCount}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-500">Exceeds Benchmark</div>
                  <div className="text-lg font-semibold text-green-600">{comparison[0].exceedsBenchmarkCount}</div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Snapshots Tab */}
      {activeTab === 'snapshots' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-lg font-semibold mb-4">Performance Snapshots</h2>
          {snapshots.length === 0 ? (
            <p className="text-gray-400 text-sm">No snapshots yet</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-gray-200">
                    <th className="text-left py-2 px-3 text-gray-500 font-medium">Agent</th>
                    <th className="text-left py-2 px-3 text-gray-500 font-medium">Conversations</th>
                    <th className="text-left py-2 px-3 text-gray-500 font-medium">Resolution Rate</th>
                    <th className="text-left py-2 px-3 text-gray-500 font-medium">Avg Response</th>
                    <th className="text-left py-2 px-3 text-gray-500 font-medium">Satisfaction</th>
                    <th className="text-left py-2 px-3 text-gray-500 font-medium">CX Score</th>
                  </tr>
                </thead>
                <tbody>
                  {snapshots.map((snap) => (
                    <tr key={snap.id} className="border-b border-gray-100 hover:bg-gray-50">
                      <td className="py-2 px-3 font-medium">{snap.agentName}</td>
                      <td className="py-2 px-3">{snap.conversationsHandled}</td>
                      <td className="py-2 px-3">
                        <div className="flex items-center gap-2">
                          <div className="w-16 bg-gray-200 rounded-full h-1.5">
                            <div
                              className="bg-blue-600 h-1.5 rounded-full"
                              style={{ width: `${snap.resolutionRate * 100}%` }}
                            />
                          </div>
                          <span className="text-xs">{(snap.resolutionRate * 100).toFixed(0)}%</span>
                        </div>
                      </td>
                      <td className="py-2 px-3">{Math.round(snap.avgResponseTimeSeconds)}s</td>
                      <td className="py-2 px-3">{snap.customerSatisfactionAvg?.toFixed(1) ?? 'N/A'}</td>
                      <td className="py-2 px-3">{snap.cxScoreAvg?.toFixed(2) ?? 'N/A'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Insights Tab */}
      {activeTab === 'insights' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Coaching Insights</h2>
            <button
              onClick={() => setShowInsightModal(true)}
              className="px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-1"
            >
              <Plus className="w-4 h-4" /> New Insight
            </button>
          </div>

          {insights.length === 0 ? (
            <div className="bg-white rounded-xl border border-gray-200 p-8 text-center">
              <Lightbulb className="w-8 h-8 text-gray-300 mx-auto mb-2" />
              <p className="text-gray-400">No coaching insights yet</p>
            </div>
          ) : (
            <div className="space-y-3">
              {insights.map((insight) => (
                <div key={insight.id} className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="flex items-start gap-3">
                    <div
                      className="w-3 h-3 rounded-full mt-1 flex-shrink-0"
                      style={{ backgroundColor: severityColor(insight.severity) }}
                    />
                    <div className="flex-1">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-medium">{insight.title}</span>
                        <span className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded">{insight.category}</span>
                        <span
                          className="text-xs px-2 py-0.5 rounded text-white"
                          style={{ backgroundColor: severityColor(insight.severity) }}
                        >
                          {insight.severity}
                        </span>
                        {insight.isAddressed && (
                          <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded flex items-center gap-1">
                            <CheckCircle className="w-3 h-3" /> Addressed
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-500 mt-1">{insight.description}</p>
                      <div className="mt-2 text-sm text-blue-600">
                        <span className="font-medium">Recommendation:</span> {insight.recommendation}
                      </div>
                      <div className="mt-1 text-xs text-gray-400">
                        Agent: {insight.agentName} · {new Date(insight.createdAt).toLocaleDateString()}
                      </div>
                      {!insight.isAddressed && (
                        <button
                          onClick={() => handleAddressInsight(insight.id)}
                          className="mt-2 text-xs text-blue-600 hover:text-blue-800"
                        >
                          Mark as addressed
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && settings && (
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-lg font-semibold mb-4">Settings</h2>
          <div className="space-y-4 max-w-lg">
            <div className="flex items-center justify-between py-2 border-b border-gray-100">
              <div>
                <div className="font-medium text-sm">Enabled</div>
                <div className="text-xs text-gray-500">Track agent performance metrics</div>
              </div>
              <button
                onClick={() => handleUpdateSettings('enabled', !settings.enabled)}
                className={`w-10 h-5 rounded-full transition-colors ${settings.enabled ? 'bg-blue-600' : 'bg-gray-300'}`}
              >
                <div
                  className={`w-4 h-4 bg-white rounded-full shadow transition-transform ${
                    settings.enabled ? 'translate-x-5' : 'translate-x-0.5'
                  }`}
                />
              </button>
            </div>
            <div className="flex items-center justify-between py-2 border-b border-gray-100">
              <div>
                <div className="font-medium text-sm">Auto Generate Insights</div>
                <div className="text-xs text-gray-500">Automatically create coaching insights</div>
              </div>
              <button
                onClick={() => handleUpdateSettings('autoGenerateInsights', !settings.autoGenerateInsights)}
                className={`w-10 h-5 rounded-full transition-colors ${settings.autoGenerateInsights ? 'bg-blue-600' : 'bg-gray-300'}`}
              >
                <div
                  className={`w-4 h-4 bg-white rounded-full shadow transition-transform ${
                    settings.autoGenerateInsights ? 'translate-x-5' : 'translate-x-0.5'
                  }`}
                />
              </button>
            </div>
            <div className="flex items-center justify-between py-2 border-b border-gray-100">
              <div>
                <div className="font-medium text-sm">Show Leaderboard</div>
                <div className="text-xs text-gray-500">Display agent rankings</div>
              </div>
              <button
                onClick={() => handleUpdateSettings('showLeaderboard', !settings.showLeaderboard)}
                className={`w-10 h-5 rounded-full transition-colors ${settings.showLeaderboard ? 'bg-blue-600' : 'bg-gray-300'}`}
              >
                <div
                  className={`w-4 h-4 bg-white rounded-full shadow transition-transform ${
                    settings.showLeaderboard ? 'translate-x-5' : 'translate-x-0.5'
                  }`}
                />
              </button>
            </div>
            <div className="py-2 border-b border-gray-100">
              <div className="font-medium text-sm mb-1">Benchmark Response Time (seconds)</div>
              <input
                type="number"
                value={settings.benchmarkResponseTimeSeconds}
                onChange={(e) => handleUpdateSettings('benchmarkResponseTimeSeconds', parseInt(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>
            <div className="py-2 border-b border-gray-100">
              <div className="font-medium text-sm mb-1">Benchmark Resolution Rate</div>
              <input
                type="number"
                step="0.01"
                min="0"
                max="1"
                value={settings.benchmarkResolutionRate}
                onChange={(e) => handleUpdateSettings('benchmarkResolutionRate', parseFloat(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>
            <div className="py-2 border-b border-gray-100">
              <div className="font-medium text-sm mb-1">Benchmark Satisfaction</div>
              <input
                type="number"
                step="0.1"
                min="1"
                max="5"
                value={settings.benchmarkSatisfaction}
                onChange={(e) => handleUpdateSettings('benchmarkSatisfaction', parseFloat(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>
            <div className="py-2">
              <div className="font-medium text-sm mb-1">Insight Frequency</div>
              <select
                value={settings.insightFrequency}
                onChange={(e) => handleUpdateSettings('insightFrequency', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              >
                <option value="daily">Daily</option>
                <option value="weekly">Weekly</option>
                <option value="monthly">Monthly</option>
              </select>
            </div>
          </div>
        </div>
      )}

      {/* New Insight Modal */}
      {showInsightModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">New Coaching Insight</h3>
            <form onSubmit={handleCreateInsight} className="space-y-3">
              <div>
                <label className="block text-sm text-gray-600 mb-1">Agent ID</label>
                <input name="agentId" required className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Agent Name</label>
                <input name="agentName" required className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Title</label>
                <input name="title" required className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Description</label>
                <textarea name="description" required className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" rows={2} />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Category</label>
                <select name="category" className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm">
                  <option value="response_quality">Response Quality</option>
                  <option value="response_time">Response Time</option>
                  <option value="resolution">Resolution</option>
                  <option value="empathy">Empathy</option>
                  <option value="compliance">Compliance</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Severity</label>
                <select name="severity" className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm">
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Recommendation</label>
                <textarea name="recommendation" required className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm" rows={2} />
              </div>
              <div className="flex gap-2 justify-end">
                <button type="button" onClick={() => setShowInsightModal(false)} className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">
                  Cancel
                </button>
                <button type="submit" className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
