import { useEffect, useState } from 'react'
import {
  getCSATSummary,
  listCSATSurveys,
  listCSATTemplates,
  createCSATSurvey,
  respondToCSATSurvey,
  sendCSATSurvey,
  generateCSATSnapshot,
  type CSATSummary as CSATSummaryType,
  type CSATSurvey,
  type CSATSurveyTemplate,
} from '../lib/api'
import {
  Star,
  BarChart3,
  TrendingUp,
  MessageSquare,
  RefreshCw,
  Loader2,
  AlertTriangle,
  Plus,
  Eye,
  Check,
  X,
  Send,
} from '../components/icons'

export function CSATDashboard() {
  const [summary, setSummary] = useState<CSATSummaryType | null>(null)
  const [surveys, setSurveys] = useState<CSATSurvey[]>([])
  const [templates, setTemplates] = useState<CSATSurveyTemplate[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'surveys' | 'templates'>('overview')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [days, setDays] = useState(30)
  const [generating, setGenerating] = useState(false)
  const [actionLoading, setActionLoading] = useState<string | null>(null)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [createForm, setCreateForm] = useState({
    conversation_id: '',
    contact_id: '',
    channel: 'email',
  })

  useEffect(() => {
    loadData()
  }, [days])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [summaryRes, surveysRes, templatesRes] = await Promise.all([
        getCSATSummary(days),
        listCSATSurveys(statusFilter || undefined, undefined, 50),
        listCSATTemplates(),
      ])
      setSummary(summaryRes)
      setSurveys(surveysRes.surveys ?? [])
      setTemplates(templatesRes.templates ?? [])
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load CSAT data')
    } finally {
      setLoading(false)
    }
  }

  const handleGenerateSnapshot = async () => {
    setGenerating(true)
    try {
      await generateCSATSnapshot()
      alert('Snapshot generated successfully')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate snapshot')
    } finally {
      setGenerating(false)
    }
  }

  const handleCreateSurvey = async () => {
    if (!createForm.conversation_id || !createForm.contact_id) {
      alert('Conversation ID and Contact ID are required')
      return
    }
    setActionLoading('create')
    try {
      await createCSATSurvey(createForm)
      setShowCreateModal(false)
      setCreateForm({ conversation_id: '', contact_id: '', channel: 'email' })
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create survey')
    } finally {
      setActionLoading(null)
    }
  }

  const handleRespond = async (surveyId: string, score: number) => {
    setActionLoading(surveyId)
    try {
      await respondToCSATSurvey(surveyId, score)
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit response')
    } finally {
      setActionLoading(null)
    }
  }

  const handleSend = async (surveyId: string) => {
    setActionLoading(surveyId)
    try {
      await sendCSATSurvey(surveyId)
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send survey')
    } finally {
      setActionLoading(null)
    }
  }

  const renderStars = (score: number | null, interactive = false, onRate?: (s: number) => void) => {
    return (
      <div className="flex gap-1">
        {[1, 2, 3, 4, 5].map((s) => (
          <button
            key={s}
            type="button"
            disabled={!interactive}
            onClick={() => onRate?.(s)}
            className={`${interactive ? 'cursor-pointer hover:scale-110' : 'cursor-default'} transition-transform`}
          >
            <Star
              className={`h-5 w-5 ${
                score !== null && s <= score ? 'text-amber-400 fill-amber-400' : 'text-gray-300'
              }`}
            />
          </button>
        ))}
      </div>
    )
  }

  const statusBadge = (status: string) => {
    switch (status) {
      case 'responded':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
            <Check className="h-3 w-3" />
            Responded
          </span>
        )
      case 'sent':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
            <Send className="h-3 w-3" />
            Sent
          </span>
        )
      case 'pending':
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            Pending
          </span>
        )
      case 'expired':
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
            Expired
          </span>
        )
      default:
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            {status}
          </span>
        )
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
        <span className="ml-2 text-sm text-gray-500">Loading CSAT data...</span>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">CSAT Surveys</h1>
          <p className="text-sm text-gray-500 mt-1">
            Customer satisfaction surveys with direct feedback from your customers
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700"
          >
            <Plus className="h-4 w-4" />
            New Survey
          </button>
          <button
            onClick={handleGenerateSnapshot}
            disabled={generating}
            className="inline-flex items-center gap-2 px-4 py-2 border border-gray-200 rounded-lg hover:bg-gray-50 disabled:opacity-50"
          >
            {generating ? <Loader2 className="h-4 w-4 animate-spin" /> : <BarChart3 className="h-4 w-4" />}
            Snapshot
          </button>
        </div>
      </div>

      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {/* Time Range */}
      <div className="flex items-center gap-3">
        <span className="text-sm text-gray-500">Time range:</span>
        {([7, 14, 30, 90] as const).map((d) => (
          <button
            key={d}
            onClick={() => setDays(d)}
            className={`px-3 py-1 text-sm rounded-lg border ${
              days === d
                ? 'bg-purple-600 text-white border-purple-600'
                : 'border-gray-200 text-gray-600 hover:bg-gray-50'
            }`}
          >
            {d}d
          </button>
        ))}
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-gray-200">
        {(['overview', 'surveys', 'templates'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              activeTab === tab
                ? 'border-purple-600 text-purple-700'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            {tab === 'overview' && 'Overview'}
            {tab === 'surveys' && 'All Surveys'}
            {tab === 'templates' && 'Templates'}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">CSAT Score</div>
              <div className="mt-1 text-2xl font-bold text-purple-600">
                {summary?.overall_csat != null ? `${Math.round(summary.overall_csat)}%` : '—'}
              </div>
              {summary?.trend_7d != null && (
                <div className="flex items-center gap-1 mt-1">
                  <TrendingUp
                    className={`h-3 w-3 ${summary.trend_7d >= 0 ? 'text-green-500' : 'text-red-500'}`}
                  />
                  <span
                    className={`text-xs ${summary.trend_7d >= 0 ? 'text-green-600' : 'text-red-600'}`}
                  >
                    {summary.trend_7d >= 0 ? '+' : ''}
                    {summary.trend_7d.toFixed(1)}% vs last 7d
                  </span>
                </div>
              )}
            </div>
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Avg Score</div>
              <div className="mt-1 text-2xl font-bold">
                {summary?.overall_average_score != null
                  ? `${summary.overall_average_score.toFixed(1)}`
                  : '—'}
              </div>
              <div className="mt-1">{renderStars(summary?.overall_average_score ?? null)}</div>
            </div>
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Response Rate</div>
              <div className="mt-1 text-2xl font-bold text-blue-600">
                {summary?.response_rate != null ? `${Math.round(summary.response_rate)}%` : '—'}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {summary?.total_responses ?? 0} / {summary?.total_surveys_sent ?? 0} responses
              </div>
            </div>
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Total Sent</div>
              <div className="mt-1 text-2xl font-bold">{summary?.total_surveys_sent ?? 0}</div>
            </div>
          </div>

          {/* Score Distribution */}
          {summary && Object.keys(summary.score_distribution).length > 0 && (
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <h3 className="text-sm font-semibold mb-3">Score Distribution</h3>
              <div className="space-y-2">
                {[5, 4, 3, 2, 1].map((score) => {
                  const count = summary.score_distribution[score] ?? 0
                  const maxCount = Math.max(
                    ...Object.values(summary.score_distribution),
                    1
                  )
                  const pct = (count / maxCount) * 100
                  return (
                    <div key={score} className="flex items-center gap-3">
                      <div className="flex items-center w-8">
                        <Star
                          className={`h-4 w-4 ${
                            score >= 4 ? 'text-amber-400 fill-amber-400' : 'text-gray-300'
                          }`}
                        />
                        <span className="ml-1 text-sm">{score}</span>
                      </div>
                      <div className="flex-1 h-4 bg-gray-100 rounded-full overflow-hidden">
                        <div
                          className={`h-full rounded-full ${
                            score >= 4 ? 'bg-green-500' : score >= 3 ? 'bg-amber-500' : 'bg-red-500'
                          }`}
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                      <span className="text-sm font-medium w-8 text-right">{count}</span>
                    </div>
                  )
                })}
              </div>
            </div>
          )}

          {/* By Channel */}
          {summary && Object.keys(summary.by_channel).length > 0 && (
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <h3 className="text-sm font-semibold mb-3">By Channel</h3>
              <div className="space-y-2">
                {Object.entries(summary.by_channel).map(([channel, data]) => (
                  <div key={channel} className="flex items-center gap-3">
                    <MessageSquare className="h-4 w-4 text-gray-400" />
                    <span className="text-sm flex-1 capitalize">{channel}</span>
                    <span className="text-sm text-gray-500">
                      Avg: {(data as { avg: number }).avg.toFixed(1)}
                    </span>
                    <span className="text-sm font-medium">
                      {(data as { count: number }).count} responses
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Comments */}
          {summary && summary.recent_comments.length > 0 && (
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <h3 className="text-sm font-semibold mb-3">Recent Comments</h3>
              <div className="space-y-3">
                {summary.recent_comments.slice(0, 5).map((c) => (
                  <div key={c.survey_id} className="p-3 bg-gray-50 rounded-lg">
                    <div className="flex items-center justify-between mb-1">
                      {renderStars(c.score)}
                      <span className="text-xs text-gray-400 ml-2">{c.channel}</span>
                    </div>
                    <p className="text-sm text-gray-700">{c.comment}</p>
                    <p className="text-xs text-gray-400 mt-1">{c.contact_identifier}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* All Surveys Tab */}
      {activeTab === 'surveys' && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex items-center gap-3">
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm"
            >
              <option value="">All Statuses</option>
              <option value="pending">Pending</option>
              <option value="sent">Sent</option>
              <option value="responded">Responded</option>
              <option value="expired">Expired</option>
            </select>
            <button
              onClick={loadData}
              className="inline-flex items-center gap-1 px-3 py-2 border border-gray-200 rounded-lg text-sm hover:bg-gray-50"
            >
              <RefreshCw className="h-4 w-4" />
              Refresh
            </button>
          </div>

          {/* Surveys List */}
          {surveys.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <MessageSquare className="h-8 w-8 mx-auto mb-2 text-gray-300" />
              <p className="text-sm">No surveys found.</p>
              <p className="text-xs mt-1">Create a new survey to start collecting feedback.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {surveys.map((s) => (
                <div
                  key={s.id}
                  className="p-4 bg-white border border-gray-200 rounded-lg hover:border-purple-300 transition-colors"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="text-sm font-medium">{s.contact_identifier}</span>
                        <span className="text-xs text-gray-400 capitalize">{s.channel}</span>
                        {statusBadge(s.status)}
                      </div>
                      {s.score !== null && (
                        <div className="mt-2 flex items-center gap-2">
                          {renderStars(s.score)}
                          {s.comment && (
                            <p className="text-sm text-gray-600 mt-1 italic">"{s.comment}"</p>
                          )}
                        </div>
                      )}
                      <div className="mt-1 text-xs text-gray-400">
                        Conversation: {s.conversation_id.slice(0, 8)}...
                      </div>
                    </div>
                    <div className="flex gap-2 flex-shrink-0 ml-4">
                      {s.status === 'pending' && (
                        <button
                          onClick={() => handleSend(s.id)}
                          disabled={actionLoading === s.id}
                          className="inline-flex items-center gap-1 px-3 py-1.5 bg-blue-600 text-white text-xs rounded-lg hover:bg-blue-700 disabled:opacity-50"
                        >
                          {actionLoading === s.id ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <Send className="h-3 w-3" />
                          )}
                          Send
                        </button>
                      )}
                      {s.status === 'sent' && (
                        <div className="flex gap-1">
                          {[1, 2, 3, 4, 5].map((star) => (
                            <button
                              key={star}
                              onClick={() => handleRespond(s.id, star)}
                              disabled={actionLoading === s.id}
                              className="disabled:opacity-50"
                            >
                              <Star className="h-4 w-4 text-amber-400 fill-amber-400 hover:scale-110 transition-transform" />
                            </button>
                          ))}
                        </div>
                      )}
                    </div>
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
          <div className="space-y-3">
            {templates.map((t) => (
              <div
                key={t.id}
                className="p-4 bg-white border border-gray-200 rounded-lg"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <h4 className="text-sm font-semibold">{t.name}</h4>
                    <p className="text-sm text-gray-600 mt-1">{t.question}</p>
                    <div className="flex gap-2 mt-2">
                      <span className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded-full capitalize">
                        {t.scale_type}
                      </span>
                      <span className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded-full capitalize">
                        {t.trigger_type}
                      </span>
                      {t.include_comment && (
                        <span className="text-xs bg-blue-100 text-blue-600 px-2 py-0.5 rounded-full">
                          With comments
                        </span>
                      )}
                    </div>
                  </div>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full ${
                      t.enabled
                        ? 'bg-green-100 text-green-800'
                        : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    {t.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Create Survey Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">Create New Survey</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-gray-600 mb-1">Conversation ID</label>
                <input
                  type="text"
                  value={createForm.conversation_id}
                  onChange={(e) =>
                    setCreateForm({ ...createForm, conversation_id: e.target.value })
                  }
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm"
                  placeholder="e.g. conv-abc123"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Contact ID</label>
                <input
                  type="text"
                  value={createForm.contact_id}
                  onChange={(e) =>
                    setCreateForm({ ...createForm, contact_id: e.target.value })
                  }
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm"
                  placeholder="e.g. contact-xyz789"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Channel</label>
                <select
                  value={createForm.channel}
                  onChange={(e) =>
                    setCreateForm({ ...createForm, channel: e.target.value })
                  }
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm"
                >
                  <option value="email">Email</option>
                  <option value="sms">SMS</option>
                  <option value="whatsapp">WhatsApp</option>
                </select>
              </div>
            </div>
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 border border-gray-200 rounded-lg text-sm hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateSurvey}
                disabled={actionLoading === 'create'}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg text-sm hover:bg-purple-700 disabled:opacity-50"
              >
                {actionLoading === 'create' ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  'Create'
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
