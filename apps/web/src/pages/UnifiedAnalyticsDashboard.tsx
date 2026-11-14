import { useEffect, useState } from 'react'
import { BarChart3, FileText, Clock, Settings, Plus, Play, Trash2, RefreshCw, Download } from '../components/icons'
import { api } from '../lib/api'
import type {
  UnifiedAnalyticsSummary,
  SavedReport,
  ScheduledReport,
  ReportTemplate,
  UnifiedAnalyticsSettings,
} from '../lib/api'

type TabType = 'overview' | 'reports' | 'scheduled' | 'settings'

export function UnifiedAnalyticsDashboard() {
  const [activeTab, setActiveTab] = useState<TabType>('overview')
  const [loading, setLoading] = useState(true)
  const [summary, setSummary] = useState<UnifiedAnalyticsSummary | null>(null)
  const [savedReports, setSavedReports] = useState<SavedReport[]>([])
  const [scheduledReports, setScheduledReports] = useState<ScheduledReport[]>([])
  const [templates, setTemplates] = useState<ReportTemplate[]>([])
  const [settings, setSettings] = useState<UnifiedAnalyticsSettings | null>(null)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showScheduleModal, setShowScheduleModal] = useState(false)

  const loadData = async () => {
    setLoading(true)
    try {
      const [summaryRes, reportsRes, scheduledRes, templatesRes, settingsRes] = await Promise.all([
        api.getUnifiedAnalyticsSummary(),
        api.listSavedReports(),
        api.listScheduledReports(),
        api.listReportTemplates(),
        api.getUnifiedAnalyticsSettings(),
      ])
      setSummary(summaryRes)
      setSavedReports(reportsRes)
      setScheduledReports(scheduledRes)
      setTemplates(templatesRes)
      setSettings(settingsRes)
    } catch (err) {
      console.error('Failed to load analytics data:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [])

  const handleCreateReport = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    try {
      await api.createSavedReport({
        org_id: 'default',
        name: formData.get('name') as string,
        report_type: formData.get('report_type') as string,
        config: {},
        time_range: formData.get('time_range') as string,
        filters: {},
        created_by: 'user',
      })
      setShowCreateModal(false)
      loadData()
    } catch (err) {
      console.error('Failed to create report:', err)
    }
  }

  const handleScheduleReport = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    try {
      await api.createScheduledReport({
        report_id: formData.get('report_id') as string,
        schedule: formData.get('schedule') as string,
        recipients: (formData.get('recipients') as string).split(',').map(s => s.trim()),
        format: formData.get('format') as string,
      })
      setShowScheduleModal(false)
      loadData()
    } catch (err) {
      console.error('Failed to schedule report:', err)
    }
  }

  const handleRunReport = async (id: string) => {
    try {
      await api.runReport(id)
      loadData()
    } catch (err) {
      console.error('Failed to run report:', err)
    }
  }

  const handleDeleteReport = async (id: string) => {
    if (confirm('Delete this report?')) {
      await api.deleteSavedReport(id)
      loadData()
    }
  }

  const handleToggleSchedule = async (id: string, status: string) => {
    try {
      await api.updateScheduledReport(id, status)
      loadData()
    } catch (err) {
      console.error('Failed to update schedule:', err)
    }
  }

  const handleDeleteSchedule = async (id: string) => {
    if (confirm('Delete this schedule?')) {
      await api.deleteScheduledReport(id)
      loadData()
    }
  }

  const handleUpdateSettings = async (updates: Record<string, unknown>) => {
    try {
      const updated = await api.updateUnifiedAnalyticsSettings(updates)
      setSettings(updated)
    } catch (err) {
      console.error('Failed to update settings:', err)
    }
  }

  if (loading) {
    return (
      <div className="p-6 max-w-7xl mx-auto">
        <div className="animate-pulse space-y-4">
          <div className="h-8 bg-gray-200 rounded w-1/3" />
          <div className="grid grid-cols-3 gap-4">
            {[...Array(6)].map((_, i) => (
              <div key={i} className="h-32 bg-gray-200 rounded" />
            ))}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-b border-[var(--color-border)] pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
            Unified Analytics Dashboard
          </h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-1">
            Comprehensive view of all platform metrics, saved reports, and scheduled analytics.
          </p>
        </div>
        <button
          onClick={loadData}
          disabled={loading}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] text-xs font-medium text-[var(--color-ink)] shadow-sm hover:bg-[var(--color-page)] disabled:opacity-50"
        >
          <RefreshCw className={`w-3.5 h-3.5 text-[var(--color-ink-muted)] ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-[var(--color-border)] gap-6 text-sm font-medium">
        {([
          { key: 'overview', label: 'Overview', icon: BarChart3 },
          { key: 'reports', label: 'Saved Reports', icon: FileText },
          { key: 'scheduled', label: 'Scheduled', icon: Clock },
          { key: 'settings', label: 'Settings', icon: Settings },
        ] as const).map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`pb-3 -mb-px flex items-center gap-2 border-b-2 transition-colors ${
              activeTab === tab.key
                ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            <tab.icon className="w-4 h-4" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && summary && (
        <div className="space-y-6">
          {/* Key Metrics */}
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            {[
              { label: 'Total Conversations', value: summary.total_conversations, color: 'bg-blue-50 text-blue-700' },
              { label: 'Resolution Rate', value: `${summary.resolution_rate.toFixed(1)}%`, color: 'bg-green-50 text-green-700' },
              { label: 'CX Score', value: summary.cx_score.toFixed(1), color: 'bg-purple-50 text-purple-700' },
              { label: 'CSAT Score', value: summary.csat_score.toFixed(1), color: 'bg-yellow-50 text-yellow-700' },
              { label: 'Satisfaction Rate', value: `${summary.feedback_satisfaction_rate.toFixed(1)}%`, color: 'bg-emerald-50 text-emerald-700' },
              { label: 'Validation Pass', value: `${summary.validation_pass_rate.toFixed(1)}%`, color: 'bg-indigo-50 text-indigo-700' },
            ].map((metric) => (
              <div key={metric.label} className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
                <div className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${metric.color}`}>
                  {metric.label}
                </div>
                <div className="mt-2 text-2xl font-bold text-[var(--color-ink)]">{metric.value}</div>
              </div>
            ))}
          </div>

          {/* Feedback & Suggestions */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)]">Feedback</h3>
              <div className="mt-2 space-y-1">
                <div className="flex justify-between text-sm">
                  <span>Total</span>
                  <span className="font-semibold">{summary.total_feedback}</span>
                </div>
                <div className="flex justify-between text-sm text-green-600">
                  <span>Positive</span>
                  <span className="font-semibold">{summary.positive_feedback}</span>
                </div>
                <div className="flex justify-between text-sm text-red-600">
                  <span>Negative</span>
                  <span className="font-semibold">{summary.negative_feedback}</span>
                </div>
              </div>
            </div>
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)]">Suggestions</h3>
              <div className="mt-2 space-y-1">
                <div className="flex justify-between text-sm">
                  <span>Total</span>
                  <span className="font-semibold">{summary.total_suggestions}</span>
                </div>
                <div className="flex justify-between text-sm text-green-600">
                  <span>Approved</span>
                  <span className="font-semibold">{summary.approved_suggestions}</span>
                </div>
              </div>
            </div>
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)]">Active Alerts</h3>
              <div className="mt-2 text-2xl font-bold text-[var(--color-ink)]">{summary.active_alerts}</div>
            </div>
          </div>

          {/* Channel & Status Breakdown */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)] mb-3">Conversations by Channel</h3>
              <div className="space-y-2">
                {summary.conversations_by_channel.slice(0, 8).map((item) => (
                  <div key={item.channel} className="flex items-center gap-2">
                    <span className="text-xs w-24 truncate">{item.channel}</span>
                    <div className="flex-1 bg-gray-100 rounded-full h-2">
                      <div
                        className="bg-indigo-500 h-2 rounded-full"
                        style={{ width: `${summary.total_conversations > 0 ? (item.count / summary.total_conversations) * 100 : 0}%` }}
                      />
                    </div>
                    <span className="text-xs font-medium w-8 text-right">{item.count}</span>
                  </div>
                ))}
              </div>
            </div>
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)] mb-3">Conversations by Status</h3>
              <div className="space-y-2">
                {summary.conversations_by_status.map((item) => (
                  <div key={item.status} className="flex items-center gap-2">
                    <span className="text-xs w-24 truncate capitalize">{item.status}</span>
                    <div className="flex-1 bg-gray-100 rounded-full h-2">
                      <div
                        className="bg-emerald-500 h-2 rounded-full"
                        style={{ width: `${summary.total_conversations > 0 ? (item.count / summary.total_conversations) * 100 : 0}%` }}
                      />
                    </div>
                    <span className="text-xs font-medium w-8 text-right">{item.count}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Topic Distribution */}
          {summary.topic_distribution.length > 0 && (
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)] mb-3">Topic Distribution</h3>
              <div className="flex flex-wrap gap-2">
                {summary.topic_distribution.map((item) => (
                  <span
                    key={item.topic}
                    className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-indigo-50 text-indigo-700"
                  >
                    {item.topic} ({item.count})
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Autonomous, Procedures, Workflows */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)]">Autonomous Resolution</h3>
              <div className="mt-2 space-y-1 text-sm">
                <div className="flex justify-between">
                  <span>Total</span>
                  <span className="font-semibold">{summary.autonomous_stats.total_resolutions}</span>
                </div>
                <div className="flex justify-between text-green-600">
                  <span>Auto-sent</span>
                  <span className="font-semibold">{summary.autonomous_stats.auto_sent}</span>
                </div>
                <div className="flex justify-between text-orange-600">
                  <span>Escalated</span>
                  <span className="font-semibold">{summary.autonomous_stats.escalated}</span>
                </div>
                <div className="flex justify-between">
                  <span>Rate</span>
                  <span className="font-semibold">{summary.autonomous_stats.resolution_rate.toFixed(1)}%</span>
                </div>
              </div>
            </div>
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)]">Procedures</h3>
              <div className="mt-2 space-y-1 text-sm">
                <div className="flex justify-between">
                  <span>Total</span>
                  <span className="font-semibold">{summary.procedure_stats.total_procedures}</span>
                </div>
                <div className="flex justify-between">
                  <span>Active</span>
                  <span className="font-semibold">{summary.procedure_stats.active_executions}</span>
                </div>
                <div className="flex justify-between">
                  <span>Completed</span>
                  <span className="font-semibold">{summary.procedure_stats.completed_executions}</span>
                </div>
                <div className="flex justify-between">
                  <span>Success</span>
                  <span className="font-semibold">{summary.procedure_stats.success_rate.toFixed(1)}%</span>
                </div>
              </div>
            </div>
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)]">Workflows</h3>
              <div className="mt-2 space-y-1 text-sm">
                <div className="flex justify-between">
                  <span>Total</span>
                  <span className="font-semibold">{summary.workflow_stats.total_workflows}</span>
                </div>
                <div className="flex justify-between">
                  <span>Active</span>
                  <span className="font-semibold">{summary.workflow_stats.active_workflows}</span>
                </div>
                <div className="flex justify-between">
                  <span>Executions</span>
                  <span className="font-semibold">{summary.workflow_stats.total_executions}</span>
                </div>
                <div className="flex justify-between">
                  <span>Success</span>
                  <span className="font-semibold">{summary.workflow_stats.success_rate.toFixed(1)}%</span>
                </div>
              </div>
            </div>
          </div>

          {/* Agent Leaderboard */}
          {summary.agent_leaderboard.length > 0 && (
            <div className="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4">
              <h3 className="text-sm font-medium text-[var(--color-ink-muted)] mb-3">Agent Leaderboard</h3>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-[var(--color-border)]">
                      <th className="text-left py-2 px-3 font-medium text-[var(--color-ink-muted)]">Agent</th>
                      <th className="text-right py-2 px-3 font-medium text-[var(--color-ink-muted)]">Handled</th>
                      <th className="text-right py-2 px-3 font-medium text-[var(--color-ink-muted)]">Resolution Rate</th>
                    </tr>
                  </thead>
                  <tbody>
                    {summary.agent_leaderboard.slice(0, 10).map((agent) => (
                      <tr key={agent.agent_id} className="border-b border-[var(--color-border)] last:border-0">
                        <td className="py-2 px-3">{agent.agent_name}</td>
                        <td className="py-2 px-3 text-right">{agent.conversations_handled}</td>
                        <td className="py-2 px-3 text-right font-medium">{agent.resolution_rate.toFixed(1)}%</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Saved Reports Tab */}
      {activeTab === 'reports' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-[var(--color-ink)]">Saved Reports</h2>
            <button
              onClick={() => setShowCreateModal(true)}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-[var(--radius-md)] bg-[var(--color-accent)] text-white text-xs font-medium hover:opacity-90"
            >
              <Plus className="w-3.5 h-3.5" />
              New Report
            </button>
          </div>

          {savedReports.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-ink-muted)]">
              <FileText className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p className="text-sm">No saved reports yet. Create one to get started.</p>
            </div>
          ) : (
            <div className="space-y-2">
              {savedReports.map((report) => (
                <div
                  key={report.id}
                  className="flex items-center justify-between p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]"
                >
                  <div>
                    <div className="text-sm font-medium text-[var(--color-ink)]">{report.name}</div>
                    <div className="text-xs text-[var(--color-ink-muted)] mt-1">
                      {report.report_type} · {report.time_range} · Last run: {report.last_run_at ? new Date(report.last_run_at).toLocaleDateString() : 'Never'}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleRunReport(report.id)}
                      className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium text-[var(--color-accent)] hover:bg-indigo-50"
                    >
                      <Play className="w-3 h-3" />
                      Run
                    </button>
                    <button
                      onClick={() => handleDeleteReport(report.id)}
                      className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium text-red-600 hover:bg-red-50"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Templates */}
          <div className="mt-6">
            <h3 className="text-sm font-medium text-[var(--color-ink-muted)] mb-3">Report Templates</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {templates.map((template) => (
                <div
                  key={template.id}
                  className="p-3 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-[var(--color-ink)]">{template.name}</span>
                    {template.is_default && (
                      <span className="text-xs px-1.5 py-0.5 rounded bg-gray-100 text-gray-600">Default</span>
                    )}
                  </div>
                  <p className="text-xs text-[var(--color-ink-muted)] mt-1">{template.description}</p>
                  <div className="text-xs text-[var(--color-ink-muted)] mt-1">
                    Type: {template.report_type} · Used {template.usage_count} times
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Scheduled Reports Tab */}
      {activeTab === 'scheduled' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-[var(--color-ink)]">Scheduled Reports</h2>
            <button
              onClick={() => setShowScheduleModal(true)}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-[var(--radius-md)] bg-[var(--color-accent)] text-white text-xs font-medium hover:opacity-90"
            >
              <Plus className="w-3.5 h-3.5" />
              New Schedule
            </button>
          </div>

          {scheduledReports.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-ink-muted)]">
              <Clock className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p className="text-sm">No scheduled reports. Create a schedule to automate report delivery.</p>
            </div>
          ) : (
            <div className="space-y-2">
              {scheduledReports.map((schedule) => (
                <div
                  key={schedule.id}
                  className="flex items-center justify-between p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]"
                >
                  <div>
                    <div className="text-sm font-medium text-[var(--color-ink)]">
                      Report: {schedule.report_id}
                    </div>
                    <div className="text-xs text-[var(--color-ink-muted)] mt-1">
                      {schedule.schedule} · {schedule.format} · Recipients: {schedule.recipients.join(', ')}
                    </div>
                    <div className="text-xs text-[var(--color-ink-muted)]">
                      Last run: {schedule.last_run_at ? new Date(schedule.last_run_at).toLocaleDateString() : 'Never'}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span
                      className={`text-xs px-2 py-0.5 rounded-full font-medium ${
                        schedule.status === 'active'
                          ? 'bg-green-100 text-green-700'
                          : 'bg-gray-100 text-gray-600'
                      }`}
                    >
                      {schedule.status}
                    </span>
                    <button
                      onClick={() => handleToggleSchedule(schedule.id, schedule.status === 'active' ? 'paused' : 'active')}
                      className="text-xs px-2 py-1 rounded text-[var(--color-accent)] hover:bg-indigo-50"
                    >
                      {schedule.status === 'active' ? 'Pause' : 'Enable'}
                    </button>
                    <button
                      onClick={() => handleDeleteSchedule(schedule.id)}
                      className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium text-red-600 hover:bg-red-50"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && settings && (
        <div className="space-y-4 max-w-2xl">
          <h2 className="text-lg font-semibold text-[var(--color-ink)]">Analytics Settings</h2>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
              <div>
                <div className="text-sm font-medium text-[var(--color-ink)]">Enable Analytics</div>
                <div className="text-xs text-[var(--color-ink-muted)]">Enable unified analytics dashboard</div>
              </div>
              <button
                onClick={() => handleUpdateSettings({ enabled: !settings.enabled })}
                className={`relative w-11 h-6 rounded-full transition-colors ${
                  settings.enabled ? 'bg-indigo-600' : 'bg-gray-300'
                }`}
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    settings.enabled ? 'translate-x-5' : ''
                  }`}
                />
              </button>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
              <div>
                <div className="text-sm font-medium text-[var(--color-ink)]">Auto Snapshot</div>
                <div className="text-xs text-[var(--color-ink-muted)]">Automatically collect metric snapshots</div>
              </div>
              <button
                onClick={() => handleUpdateSettings({ auto_snapshot: !settings.auto_snapshot })}
                className={`relative w-11 h-6 rounded-full transition-colors ${
                  settings.auto_snapshot ? 'bg-indigo-600' : 'bg-gray-300'
                }`}
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    settings.auto_snapshot ? 'translate-x-5' : ''
                  }`}
                />
              </button>
            </div>

            <div className="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
              <div className="text-sm font-medium text-[var(--color-ink)] mb-2">Snapshot Frequency</div>
              <select
                value={settings.snapshot_frequency}
                onChange={(e) => handleUpdateSettings({ snapshot_frequency: e.target.value })}
                className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
              >
                <option value="daily">Daily</option>
                <option value="weekly">Weekly</option>
                <option value="monthly">Monthly</option>
              </select>
            </div>

            <div className="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
              <div className="text-sm font-medium text-[var(--color-ink)] mb-2">Retention Days</div>
              <input
                type="number"
                value={settings.retention_days}
                onChange={(e) => handleUpdateSettings({ retention_days: parseInt(e.target.value) })}
                className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
              />
            </div>

            <div className="p-4 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
              <div className="text-sm font-medium text-[var(--color-ink)] mb-2">Default Dashboard</div>
              <select
                value={settings.default_dashboard}
                onChange={(e) => handleUpdateSettings({ default_dashboard: e.target.value })}
                className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
              >
                <option value="overview">Overview</option>
                <option value="reports">Reports</option>
              </select>
            </div>
          </div>
        </div>
      )}

      {/* Create Report Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-[var(--color-surface)] rounded-lg shadow-xl max-w-md w-full p-6">
            <h3 className="text-lg font-semibold text-[var(--color-ink)] mb-4">Create Report</h3>
            <form onSubmit={handleCreateReport} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Name</label>
                <input
                  name="name"
                  required
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                  placeholder="Monthly Analytics Report"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Type</label>
                <select
                  name="report_type"
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                >
                  <option value="unified">Unified Dashboard</option>
                  <option value="conversation">Conversation Overview</option>
                  <option value="agent">Agent Performance</option>
                  <option value="cx_qa">CX Score Summary</option>
                  <option value="csat">CSAT Report</option>
                  <option value="feedback">Feedback Report</option>
                  <option value="topic">Topic Distribution</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Time Range</label>
                <select
                  name="time_range"
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                >
                  <option value="7d">Last 7 Days</option>
                  <option value="30d">Last 30 Days</option>
                  <option value="90d">Last 90 Days</option>
                </select>
              </div>
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="px-3 py-1.5 rounded-md text-sm text-[var(--color-ink-muted)] hover:bg-gray-100"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3 py-1.5 rounded-md text-sm bg-[var(--color-accent)] text-white hover:opacity-90"
                >
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Schedule Report Modal */}
      {showScheduleModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-[var(--color-surface)] rounded-lg shadow-xl max-w-md w-full p-6">
            <h3 className="text-lg font-semibold text-[var(--color-ink)] mb-4">Schedule Report</h3>
            <form onSubmit={handleScheduleReport} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Report</label>
                <select
                  name="report_id"
                  required
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                >
                  <option value="">Select a report</option>
                  {savedReports.map((r) => (
                    <option key={r.id} value={r.id}>{r.name}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Schedule</label>
                <select
                  name="schedule"
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                >
                  <option value="daily">Daily</option>
                  <option value="weekly">Weekly</option>
                  <option value="monthly">Monthly</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Format</label>
                <select
                  name="format"
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                >
                  <option value="csv">CSV</option>
                  <option value="json">JSON</option>
                  <option value="pdf">PDF</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">
                  Recipients (comma-separated emails)
                </label>
                <input
                  name="recipients"
                  className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-sm bg-[var(--color-page)]"
                  placeholder="user@example.com,team@example.com"
                />
              </div>
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowScheduleModal(false)}
                  className="px-3 py-1.5 rounded-md text-sm text-[var(--color-ink-muted)] hover:bg-gray-100"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-3 py-1.5 rounded-md text-sm bg-[var(--color-accent)] text-white hover:opacity-90"
                >
                  Schedule
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
