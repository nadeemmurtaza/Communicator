import { useEffect, useState } from 'react'
import {
  getCXSummary,
  batchScoreConversations,
  listQARules,
  updateQARule,
  type CXSummary,
  type QAScoringRule,
} from '../lib/api'
import {
  BarChart3,
  TrendingUp,
  TrendingDown,
  Minus,
  ShieldCheck,
  AlertTriangle,
  CheckCircle2,
  RefreshCw,
  Settings,
  BarChart,
  ListTodo,
} from '../components/icons'

export function CXQAScoring() {
  const [activeTab, setActiveTab] = useState<'overview' | 'rules' | 'batch'>('overview')
  const [summary, setSummary] = useState<CXSummary | null>(null)
  const [rules, setRules] = useState<QAScoringRule[]>([])
  const [loading, setLoading] = useState(true)
  const [batching, setBatching] = useState(false)
  const [batchResult, setBatchResult] = useState<string | null>(null)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    setLoading(true)
    try {
      const [summaryData, rulesData] = await Promise.all([
        getCXSummary().catch(() => null),
        listQARules().catch(() => []),
      ])
      setSummary(summaryData)
      setRules(rulesData)
    } finally {
      setLoading(false)
    }
  }

  const handleBatchScore = async () => {
    setBatching(true)
    setBatchResult(null)
    try {
      const result = await batchScoreConversations()
      setBatchResult(
        `Batch scoring complete: ${result.scored} scored, ${result.failed} failed, ${result.skipped} skipped out of ${result.total} conversations.`
      )
      loadData()
    } catch (err) {
      setBatchResult(`Error: ${err instanceof Error ? err.message : 'Batch scoring failed'}`)
    } finally {
      setBatching(false)
    }
  }

  const handleToggleRule = async (rule: QAScoringRule) => {
    try {
      await updateQARule(rule.id, !rule.is_enabled)
      setRules((prev) =>
        prev.map((r) => (r.id === rule.id ? { ...r, is_enabled: !r.is_enabled } : r))
      )
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to update rule')
    }
  }

  const scoreColor = (score: number) => {
    if (score >= 80) return 'text-emerald-600'
    if (score >= 60) return 'text-amber-600'
    return 'text-rose-600'
  }

  const scoreBg = (score: number) => {
    if (score >= 80) return 'bg-emerald-100 text-emerald-800'
    if (score >= 60) return 'bg-amber-100 text-amber-800'
    return 'bg-rose-100 text-rose-800'
  }

  const trendIcon = (direction: string) => {
    if (direction === 'improving') return <TrendingUp className="h-4 w-4 text-emerald-600" />
    if (direction === 'declining') return <TrendingDown className="h-4 w-4 text-rose-600" />
    return <Minus className="h-4 w-4 text-gray-400" />
  }

  const severityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-rose-100 text-rose-800'
      case 'high':
        return 'bg-orange-100 text-orange-800'
      case 'medium':
        return 'bg-amber-100 text-amber-800'
      default:
        return 'bg-blue-100 text-blue-800'
    }
  }

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="flex items-center gap-2 text-[var(--color-ink-faint)]">
          <span className="loading loading-spinner loading-md"></span>
          <span>Loading CX Score &amp; QA data...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
            CX Score &amp; Automated QA
          </h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            AI-powered conversation quality scoring across every conversation. Provides 5x more
            coverage than traditional CSAT surveys.
          </p>
        </div>
        <button
          onClick={handleBatchScore}
          disabled={batching}
          className="inline-flex items-center justify-center rounded-[var(--radius-md)] bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
        >
          {batching ? (
            'Scoring...'
          ) : (
            <>
              <RefreshCw className="mr-1.5 h-4 w-4" /> Score All Conversations
            </>
          )}
        </button>
      </div>

      {batchResult && (
        <div className="rounded-[var(--radius-md)] bg-blue-50 p-3 text-sm text-blue-800 border border-blue-200">
          {batchResult}
        </div>
      )}

      {/* Tabs */}
      <div className="flex border-b border-[var(--color-border)] gap-2">
        <button
          onClick={() => setActiveTab('overview')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeTab === 'overview'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          <BarChart3 className="h-4 w-4" /> Overview
        </button>
        <button
          onClick={() => setActiveTab('rules')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeTab === 'rules'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          <Settings className="h-4 w-4" /> QA Rules ({rules.length})
        </button>
        <button
          onClick={() => setActiveTab('batch')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeTab === 'batch'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          <ListTodo className="h-4 w-4" /> Batch Scoring
        </button>
      </div>

      {/* TAB 1: Overview */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Overall CX Score */}
          {summary && (
            <>
              <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
                  <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
                    Overall CX Score
                  </div>
                  <div className="mt-2 flex items-baseline gap-2">
                    <span className={`text-3xl font-bold ${scoreColor(summary.overall_cx_score)}`}>
                      {summary.overall_cx_score.toFixed(1)}
                    </span>
                    <span className={`rounded-full px-2 py-0.5 text-xs font-semibold ${scoreBg(summary.overall_cx_score)}`}>
                      {trendIcon(summary.trend_direction)} {summary.trend_direction}
                    </span>
                  </div>
                  <div className="mt-2 text-xs text-[var(--color-ink-muted)]">
                    {summary.total_conversations_scored} conversations scored
                  </div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
                  <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
                    Last 24 Hours
                  </div>
                  <div className="mt-2 text-2xl font-bold text-[var(--color-ink)]">
                    {summary.conversations_last_24h}
                  </div>
                  <div className="mt-1 text-xs text-[var(--color-ink-muted)]">conversations scored</div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
                  <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
                    Last 7 Days
                  </div>
                  <div className="mt-2 text-2xl font-bold text-[var(--color-ink)]">
                    {summary.conversations_last_7d}
                  </div>
                  <div className="mt-1 text-xs text-[var(--color-ink-muted)]">conversations scored</div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
                  <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
                    Top Issues
                  </div>
                  <div className="mt-2 text-2xl font-bold text-[var(--color-ink)]">
                    {summary.top_issues.length}
                  </div>
                  <div className="mt-1 text-xs text-[var(--color-ink-muted)]">unique issues found</div>
                </div>
              </div>

              {/* Dimension Scores */}
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
                <h3 className="text-sm font-bold text-[var(--color-ink)] mb-4 flex items-center gap-2">
                  <BarChart className="h-4 w-4" /> Dimension Scores
                </h3>
                <div className="grid grid-cols-1 sm:grid-cols-5 gap-4">
                  {[
                    { label: 'Accuracy', value: summary.avg_accuracy, icon: '🎯' },
                    { label: 'Empathy', value: summary.avg_empathy, icon: '💝' },
                    { label: 'Resolution', value: summary.avg_resolution, icon: '✅' },
                    { label: 'Compliance', value: summary.avg_compliance, icon: '🛡️' },
                    { label: 'Timeliness', value: summary.avg_timeliness, icon: '⏱️' },
                  ].map((dim) => (
                    <div key={dim.label} className="text-center">
                      <div className="text-2xl mb-1">{dim.icon}</div>
                      <div className={`text-xl font-bold ${scoreColor(dim.value)}`}>
                        {dim.value.toFixed(1)}
                      </div>
                      <div className="text-xs text-[var(--color-ink-muted)] mt-1">{dim.label}</div>
                      {/* Progress bar */}
                      <div className="mt-2 h-2 rounded-full bg-[var(--color-accent-muted)] overflow-hidden">
                        <div
                          className={`h-full rounded-full transition-all ${
                            dim.value >= 80
                              ? 'bg-emerald-500'
                              : dim.value >= 60
                              ? 'bg-amber-500'
                              : 'bg-rose-500'
                          }`}
                          style={{ width: `${Math.min(dim.value, 100)}%` }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Dimension Trends */}
              {summary.dimension_trend.length > 0 && (
                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
                  <h3 className="text-sm font-bold text-[var(--color-ink)] mb-4">
                    Dimension Trends (7-Day Change)
                  </h3>
                  <div className="overflow-x-auto">
                    <table className="min-w-full text-xs">
                      <thead>
                        <tr className="border-b border-[var(--color-border)]">
                          <th className="text-left py-2 px-3 font-semibold text-[var(--color-ink-muted)]">
                            Dimension
                          </th>
                          <th className="text-right py-2 px-3 font-semibold text-[var(--color-ink-muted)]">
                            Current
                          </th>
                          <th className="text-right py-2 px-3 font-semibold text-[var(--color-ink-muted)]">
                            Previous
                          </th>
                          <th className="text-right py-2 px-3 font-semibold text-[var(--color-ink-muted)]">
                            Change
                          </th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-[var(--color-border)]">
                        {summary.dimension_trend.map((trend) => (
                          <tr key={trend.dimension}>
                            <td className="py-2 px-3 capitalize text-[var(--color-ink)]">
                              {trend.dimension}
                            </td>
                            <td className={`py-2 px-3 text-right font-semibold ${scoreColor(trend.current_avg)}`}>
                              {trend.current_avg.toFixed(1)}
                            </td>
                            <td className="py-2 px-3 text-right text-[var(--color-ink-muted)]">
                              {trend.previous_avg.toFixed(1)}
                            </td>
                            <td
                              className={`py-2 px-3 text-right font-semibold ${
                                trend.change > 0
                                  ? 'text-emerald-600'
                                  : trend.change < 0
                                  ? 'text-rose-600'
                                  : 'text-[var(--color-ink-muted)]'
                              }`}
                            >
                              {trend.change > 0 ? '+' : ''}
                              {trend.change.toFixed(1)}%
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {/* Top Issues */}
              {summary.top_issues.length > 0 && (
                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
                  <h3 className="text-sm font-bold text-[var(--color-ink)] mb-4 flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4 text-amber-600" /> Top Issues Found
                  </h3>
                  <div className="space-y-2">
                    {summary.top_issues.map((issue, idx) => (
                      <div
                        key={idx}
                        className="flex items-center justify-between rounded-[var(--radius-md)] bg-[var(--color-page)] p-3 border border-[var(--color-border)]"
                      >
                        <span className="text-sm text-[var(--color-ink)]">{issue.issue}</span>
                        <span className="rounded-full bg-amber-100 px-2.5 py-0.5 text-xs font-semibold text-amber-800">
                          {issue.count} occurrences
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      )}

      {/* TAB 2: QA Rules */}
      {activeTab === 'rules' && (
        <div className="space-y-4">
          <div>
            <h3 className="text-sm font-bold text-[var(--color-ink)]">QA Scoring Rules</h3>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Configure which quality checks are applied to every conversation.
            </p>
          </div>

          {rules.length === 0 ? (
            <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center text-sm text-[var(--color-ink-muted)]">
              No QA rules configured. Default rules will be seeded on first run.
            </div>
          ) : (
            <div className="space-y-3">
              {rules.map((rule) => (
                <div
                  key={rule.id}
                  className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <h4 className="text-sm font-bold text-[var(--color-ink)]">{rule.name}</h4>
                        <span
                          className={`rounded-full px-2 py-0.5 text-xs font-semibold ${severityColor(
                            rule.severity
                          )}`}
                        >
                          {rule.severity}
                        </span>
                        <span className="rounded bg-[var(--color-accent-muted)] px-2 py-0.5 text-xs font-medium text-[var(--color-ink-muted)] capitalize">
                          {rule.dimension}
                        </span>
                      </div>
                      <p className="text-xs text-[var(--color-ink-muted)] mt-1">
                        {rule.description}
                      </p>
                      <div className="mt-2 text-xs text-[var(--color-ink-faint)]">
                        Score impact: {rule.score_impact > 0 ? '+' : ''}
                        {rule.score_impact}
                      </div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer ml-4">
                      <input
                        type="checkbox"
                        checked={rule.is_enabled}
                        onChange={() => handleToggleRule(rule)}
                        className="sr-only peer"
                      />
                      <div className="w-9 h-5 bg-[var(--color-border)] peer-focus:outline-hidden rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-[var(--color-border)] after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
                    </label>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* TAB 3: Batch Scoring */}
      {activeTab === 'batch' && (
        <div className="space-y-6">
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-sm">
            <h3 className="text-sm font-bold text-[var(--color-ink)] mb-2">
              Batch Score Conversations
            </h3>
            <p className="text-xs text-[var(--color-ink-muted)] mb-4">
              Score all resolved/closed conversations that haven't been QA-scored yet. This uses
              Claude to evaluate each conversation on 5 dimensions.
            </p>
            <button
              onClick={handleBatchScore}
              disabled={batching}
              className="rounded-[var(--radius-md)] bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
            >
              {batching ? 'Scoring in progress...' : 'Start Batch Scoring'}
            </button>
          </div>

          {summary && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
              <h3 className="text-sm font-bold text-[var(--color-ink)] mb-3">Scoring Progress</h3>
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-[var(--color-ink)]">
                    {summary.total_conversations_scored}
                  </div>
                  <div className="text-xs text-[var(--color-ink-muted)]">Total Scored</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-[var(--color-ink)]">
                    {summary.conversations_last_24h}
                  </div>
                  <div className="text-xs text-[var(--color-ink-muted)]">Last 24h</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-[var(--color-ink)]">
                    {summary.conversations_last_7d}
                  </div>
                  <div className="text-xs text-[var(--color-ink-muted)]">Last 7 Days</div>
                </div>
                <div className="text-center">
                  <div className={`text-2xl font-bold ${scoreColor(summary.overall_cx_score)}`}>
                    {summary.overall_cx_score.toFixed(1)}
                  </div>
                  <div className="text-xs text-[var(--color-ink-muted)]">Avg CX Score</div>
                </div>
              </div>
            </div>
          )}

          <div className="rounded-[var(--radius-lg)] border border-blue-200 bg-blue-50/50 p-5 shadow-sm">
            <h4 className="text-xs font-bold text-blue-900 flex items-center gap-1.5">
              <ShieldCheck className="h-4 w-4 text-blue-600" /> How QA Scoring Works
            </h4>
            <ul className="mt-2 text-xs text-blue-800 space-y-1 list-disc list-inside">
              <li>
                <strong>Accuracy</strong>: Are responses factually correct and relevant?
              </li>
              <li>
                <strong>Empathy</strong>: Does the agent show understanding and appropriate tone?
              </li>
              <li>
                <strong>Resolution</strong>: Did the conversation reach a clear resolution?
              </li>
              <li>
                <strong>Compliance</strong>: Were policies and guardrails followed?
              </li>
              <li>
                <strong>Timeliness</strong>: Were responses provided in reasonable time?
              </li>
              <li>
                If Claude is unavailable, keyword-based heuristics provide baseline scores.
              </li>
            </ul>
          </div>
        </div>
      )}
    </div>
  )
}
