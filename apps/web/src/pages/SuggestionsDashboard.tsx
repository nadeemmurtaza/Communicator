import { useEffect, useState } from 'react'
import {
  listSuggestions,
  getSuggestionSummary,
  approveSuggestion,
  dismissSuggestion,
  triggerBatchAnalysis,
  type Suggestion,
  type SuggestionSummary,
} from '../lib/api'
import {
  Lightbulb,
  Check,
  X,
  RefreshCw,
  Loader2,
  AlertTriangle,
  BookOpen,
  TrendingUp,
  Search,
  Sparkles,
  User,
} from '../components/icons'

export function SuggestionsDashboard() {
  const [suggestions, setSuggestions] = useState<Suggestion[]>([])
  const [summary, setSummary] = useState<SuggestionSummary | null>(null)
  const [loading, setLoading] = useState(true)
  const [analyzing, setAnalyzing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'suggestions' | 'batch'>('overview')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [typeFilter, setTypeFilter] = useState<string>('')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedSuggestion, setSelectedSuggestion] = useState<Suggestion | null>(null)
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [suggestionsRes, summaryRes] = await Promise.all([
        listSuggestions(statusFilter || undefined, typeFilter || undefined, 100),
        getSuggestionSummary(),
      ])
      setSuggestions(suggestionsRes.suggestions ?? [])
      setSummary(summaryRes)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load suggestions')
    } finally {
      setLoading(false)
    }
  }

  const handleApprove = async (id: string) => {
    setActionLoading(id)
    try {
      await approveSuggestion(id)
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to approve suggestion')
    } finally {
      setActionLoading(null)
    }
  }

  const handleDismiss = async (id: string) => {
    setActionLoading(id)
    try {
      await dismissSuggestion(id)
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to dismiss suggestion')
    } finally {
      setActionLoading(null)
    }
  }

  const handleBatchAnalyze = async () => {
    setAnalyzing(true)
    try {
      const result = await triggerBatchAnalysis(24)
      alert(
        `Batch analysis complete:\n` +
        `- Feedback analyzed: ${result.feedbackAnalyzed}\n` +
        `- Suggestions created: ${result.suggestionsCreated}\n` +
        `- Errors: ${result.errors}`
      )
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Batch analysis failed')
    } finally {
      setAnalyzing(false)
    }
  }

  const filteredSuggestions = suggestions.filter((s) => {
    if (searchQuery) {
      const q = searchQuery.toLowerCase()
      return (
        s.title.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        (s.suggestedContent && s.suggestedContent.toLowerCase().includes(q))
      )
    }
    return true
  })

  const suggestionTypeIcon = (type: string) => {
    switch (type) {
      case 'knowledge_gap':
        return <BookOpen className="h-4 w-4 text-blue-600" />
      case 'faq_addition':
        return <Lightbulb className="h-4 w-4 text-amber-600" />
      case 'agent_insight':
        return <User className="h-4 w-4 text-green-600" />
      default:
        return <Sparkles className="h-4 w-4 text-purple-600" />
    }
  }

  const suggestionTypeLabel = (type: string) => {
    switch (type) {
      case 'knowledge_gap':
        return 'Knowledge Gap'
      case 'faq_addition':
        return 'FAQ Addition'
      case 'agent_insight':
        return 'Agent Insight'
      default:
        return type
    }
  }

  const statusBadge = (status: string) => {
    switch (status) {
      case 'suggested':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
            Suggested
          </span>
        )
      case 'approved':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
            <Check className="h-3 w-3" />
            Approved
          </span>
        )
      case 'dismissed':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            <X className="h-3 w-3" />
            Dismissed
          </span>
        )
      default:
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            {status}
          </span>
        )
    }
  }

  const confidenceBar = (score: number) => {
    const pct = Math.round((score || 0) * 100)
    const color = pct >= 70 ? 'bg-green-500' : pct >= 40 ? 'bg-amber-500' : 'bg-red-500'
    return (
      <div className="flex items-center gap-2">
        <div className="flex-1 h-2 bg-gray-200 rounded-full overflow-hidden">
          <div className={`h-full ${color} rounded-full`} style={{ width: `${pct}%` }} />
        </div>
        <span className="text-xs text-gray-500 w-10 text-right">{pct}%</span>
      </div>
    )
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
        <span className="ml-2 text-sm text-gray-500">Loading suggestions...</span>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">AI Suggestions</h1>
          <p className="text-sm text-gray-500 mt-1">
            Auto-generated knowledge base improvements from negative feedback analysis
          </p>
        </div>
        <button
          onClick={handleBatchAnalyze}
          disabled={analyzing}
          className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50"
        >
          {analyzing ? <Loader2 className="h-4 w-4 animate-spin" /> : <Sparkles className="h-4 w-4" />}
          Analyze Feedback (24h)
        </button>
      </div>

      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 border-b border-gray-200">
        {(['overview', 'suggestions', 'batch'] as const).map((tab) => (
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
            {tab === 'suggestions' && 'All Suggestions'}
            {tab === 'batch' && 'Batch Analysis'}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Total Suggestions</div>
              <div className="mt-1 text-2xl font-bold">{summary?.totalSuggestions ?? 0}</div>
            </div>
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Open</div>
              <div className="mt-1 text-2xl font-bold text-blue-600">{summary?.openSuggestions ?? 0}</div>
            </div>
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Approved</div>
              <div className="mt-1 text-2xl font-bold text-green-600">{summary?.approvedSuggestions ?? 0}</div>
            </div>
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <div className="text-xs text-gray-500 uppercase tracking-wider">Dismissed</div>
              <div className="mt-1 text-2xl font-bold text-gray-600">{summary?.dismissedSuggestions ?? 0}</div>
            </div>
          </div>

          {/* Breakdown by Type */}
          {summary?.byType && summary.byType.length > 0 && (
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <h3 className="text-sm font-semibold mb-3">By Suggestion Type</h3>
              <div className="space-y-2">
                {summary.byType.map((item) => (
                  <div key={item.suggestionType} className="flex items-center gap-3">
                    {suggestionTypeIcon(item.suggestionType)}
                    <span className="text-sm flex-1">{suggestionTypeLabel(item.suggestionType)}</span>
                    <span className="text-sm font-medium">{item.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Breakdown by Category */}
          {summary?.byCategory && summary.byCategory.length > 0 && (
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <h3 className="text-sm font-semibold mb-3">By Category</h3>
              <div className="space-y-2">
                {summary.byCategory.map((item) => (
                  <div key={item.category} className="flex items-center gap-3">
                    <TrendingUp className="h-4 w-4 text-gray-400" />
                    <span className="text-sm flex-1 capitalize">{item.category}</span>
                    <span className="text-sm font-medium">{item.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Suggestions */}
          {summary?.recentSuggestions && summary.recentSuggestions.length > 0 && (
            <div className="p-4 bg-white border border-gray-200 rounded-lg">
              <h3 className="text-sm font-semibold mb-3">Recent Suggestions</h3>
              <div className="space-y-3">
                {summary.recentSuggestions.slice(0, 5).map((s) => (
                  <div
                    key={s.id}
                    className="flex items-start gap-3 p-3 bg-gray-50 rounded-lg cursor-pointer hover:bg-gray-100"
                    onClick={() => setSelectedSuggestion(s)}
                  >
                    {suggestionTypeIcon(s.suggestionType)}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium truncate">{s.title}</span>
                        {statusBadge(s.status)}
                      </div>
                      <p className="text-xs text-gray-500 mt-0.5 line-clamp-1">{s.description}</p>
                    </div>
                    <div className="w-24 flex-shrink-0">
                      {confidenceBar(s.confidence ?? 0)}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* All Suggestions Tab */}
      {activeTab === 'suggestions' && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex flex-wrap items-center gap-3">
            <div className="relative flex-1 min-w-[200px]">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                placeholder="Search suggestions..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-9 pr-3 py-2 border border-gray-200 rounded-lg text-sm"
              />
            </div>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm"
            >
              <option value="">All Statuses</option>
              <option value="suggested">Suggested</option>
              <option value="approved">Approved</option>
              <option value="dismissed">Dismissed</option>
            </select>
            <select
              value={typeFilter}
              onChange={(e) => setTypeFilter(e.target.value)}
              className="px-3 py-2 border border-gray-200 rounded-lg text-sm"
            >
              <option value="">All Types</option>
              <option value="knowledge_gap">Knowledge Gap</option>
              <option value="faq_addition">FAQ Addition</option>
              <option value="agent_insight">Agent Insight</option>
            </select>
            <button
              onClick={loadData}
              className="inline-flex items-center gap-1 px-3 py-2 border border-gray-200 rounded-lg text-sm hover:bg-gray-50"
            >
              <RefreshCw className="h-4 w-4" />
              Refresh
            </button>
          </div>

          {/* Suggestions List */}
          {filteredSuggestions.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <Lightbulb className="h-8 w-8 mx-auto mb-2 text-gray-300" />
              <p className="text-sm">No suggestions found.</p>
              <p className="text-xs mt-1">Run batch analysis to generate suggestions from negative feedback.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {filteredSuggestions.map((s) => (
                <div
                  key={s.id}
                  className="p-4 bg-white border border-gray-200 rounded-lg hover:border-purple-300 transition-colors"
                >
                  <div className="flex items-start gap-3">
                    <div className="mt-0.5">{suggestionTypeIcon(s.suggestionType)}</div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <h4 className="text-sm font-semibold">{s.title}</h4>
                        {statusBadge(s.status)}
                        {s.category && (
                          <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs bg-gray-100 text-gray-600 capitalize">
                            {s.category}
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-600 mt-1">{s.description}</p>
                      {s.suggestedContent && (
                        <details className="mt-2">
                          <summary className="text-xs text-purple-600 cursor-pointer hover:text-purple-700">
                            View suggested content
                          </summary>
                          <pre className="mt-1 p-2 bg-gray-50 rounded text-xs whitespace-pre-wrap max-h-40 overflow-y-auto">
                            {s.suggestedContent}
                          </pre>
                        </details>
                      )}
                      <div className="mt-2 flex items-center gap-4">
                        <div className="flex items-center gap-2">
                          <span className="text-xs text-gray-500">Confidence:</span>
                          {confidenceBar(s.confidence ?? 0)}
                        </div>
                        {s.sourceFeedbackId && (
                          <span className="text-xs text-gray-400">
                            Source: feedback #{s.sourceFeedbackId.slice(0, 8)}
                          </span>
                        )}
                      </div>
                    </div>
                    {s.status === 'suggested' && (
                      <div className="flex gap-2 flex-shrink-0">
                        <button
                          onClick={() => handleApprove(s.id)}
                          disabled={actionLoading === s.id}
                          className="inline-flex items-center gap-1 px-3 py-1.5 bg-green-600 text-white text-xs rounded-lg hover:bg-green-700 disabled:opacity-50"
                        >
                          {actionLoading === s.id ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <Check className="h-3 w-3" />
                          )}
                          Approve
                        </button>
                        <button
                          onClick={() => handleDismiss(s.id)}
                          disabled={actionLoading === s.id}
                          className="inline-flex items-center gap-1 px-3 py-1.5 bg-gray-200 text-gray-700 text-xs rounded-lg hover:bg-gray-300 disabled:opacity-50"
                        >
                          {actionLoading === s.id ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <X className="h-3 w-3" />
                          )}
                          Dismiss
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Batch Analysis Tab */}
      {activeTab === 'batch' && (
        <div className="space-y-6">
          <div className="p-6 bg-white border border-gray-200 rounded-lg">
            <h3 className="text-lg font-semibold mb-2">Batch Feedback Analysis</h3>
            <p className="text-sm text-gray-600 mb-4">
              Analyze all negative feedback from the last 24 hours and generate structured suggestions for knowledge
              base improvements. Claude examines each thumbs-down conversation and recommends new FAQ entries,
              knowledge base content, or agent behavior improvements.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
              <div className="p-3 bg-blue-50 rounded-lg">
                <div className="text-xs text-blue-600 font-medium">Knowledge Gaps</div>
                <div className="text-lg font-bold text-blue-900">
                  {summary?.byType.find((t) => t.suggestionType === 'knowledge_gap')?.count ?? 0}
                </div>
              </div>
              <div className="p-3 bg-amber-50 rounded-lg">
                <div className="text-xs text-amber-600 font-medium">FAQ Additions</div>
                <div className="text-lg font-bold text-amber-900">
                  {summary?.byType.find((t) => t.suggestionType === 'faq_addition')?.count ?? 0}
                </div>
              </div>
              <div className="p-3 bg-green-50 rounded-lg">
                <div className="text-xs text-green-600 font-medium">Agent Insights</div>
                <div className="text-lg font-bold text-green-900">
                  {summary?.byType.find((t) => t.suggestionType === 'agent_insight')?.count ?? 0}
                </div>
              </div>
            </div>
            <button
              onClick={handleBatchAnalyze}
              disabled={analyzing}
              className="inline-flex items-center gap-2 px-6 py-3 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 font-medium"
            >
              {analyzing ? <Loader2 className="h-5 w-5 animate-spin" /> : <Sparkles className="h-5 w-5" />}
              {analyzing ? 'Analyzing Feedback...' : 'Run Batch Analysis (Last 24h)'}
            </button>
          </div>

          <div className="p-6 bg-white border border-gray-200 rounded-lg">
            <h3 className="text-lg font-semibold mb-2">How It Works</h3>
            <ol className="space-y-3 text-sm text-gray-600 list-decimal list-inside">
              <li>
                <strong>Collect:</strong> System gathers all negative feedback (thumbs-down) from the last 24 hours
              </li>
              <li>
                <strong>Analyze:</strong> Claude examines each conversation context and identifies why the response
                was rated poorly
              </li>
              <li>
                <strong>Suggest:</strong> Structured suggestions are generated for knowledge gaps, FAQ additions,
                or agent behavior improvements
              </li>
              <li>
                <strong>Deduplicate:</strong> Duplicate suggestions are detected and skipped
              </li>
              <li>
                <strong>Review:</strong> Human reviews suggestions and approves or dismisses them
              </li>
              <li>
                <strong>Apply:</strong> Approved suggestions are applied to the knowledge base, closing the
                self-improvement loop
              </li>
            </ol>
          </div>
        </div>
      )}

      {/* Detail Modal */}
      {selectedSuggestion && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg max-w-2xl w-full max-h-[80vh] overflow-y-auto p-6">
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-2">
                {suggestionTypeIcon(selectedSuggestion.suggestionType)}
                <h3 className="text-lg font-bold">{selectedSuggestion.title}</h3>
              </div>
              <button
                onClick={() => setSelectedSuggestion(null)}
                className="p-1 hover:bg-gray-100 rounded"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            {statusBadge(selectedSuggestion.status)}

            <div className="mt-4 space-y-4">
              <div>
                <h4 className="text-sm font-medium text-gray-500">Description</h4>
                <p className="text-sm mt-1">{selectedSuggestion.description}</p>
              </div>

              {selectedSuggestion.suggestedContent && (
                <div>
                  <h4 className="text-sm font-medium text-gray-500">Suggested Content</h4>
                  <pre className="mt-1 p-3 bg-gray-50 rounded text-sm whitespace-pre-wrap">
                    {selectedSuggestion.suggestedContent}
                  </pre>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <h4 className="text-sm font-medium text-gray-500">Confidence Score</h4>
                  <div className="mt-1">{confidenceBar(selectedSuggestion.confidence ?? 0)}</div>
                </div>
                <div>
                  <h4 className="text-sm font-medium text-gray-500">Category</h4>
                  <p className="text-sm mt-1 capitalize">{selectedSuggestion.category || 'N/A'}</p>
                </div>
              </div>

              {selectedSuggestion.sourceFeedbackId && (
                <div>
                  <h4 className="text-sm font-medium text-gray-500">Source Feedback</h4>
                  <p className="text-sm mt-1 font-mono">{selectedSuggestion.sourceFeedbackId}</p>
                </div>
              )}

              {selectedSuggestion.status === 'suggested' && (
                <div className="flex gap-3 pt-4 border-t">
                  <button
                    onClick={() => {
                      handleApprove(selectedSuggestion.id)
                      setSelectedSuggestion(null)
                    }}
                    disabled={actionLoading === selectedSuggestion.id}
                    className="inline-flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
                  >
                    {actionLoading === selectedSuggestion.id ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <Check className="h-4 w-4" />
                    )}
                    Approve
                  </button>
                  <button
                    onClick={() => {
                      handleDismiss(selectedSuggestion.id)
                      setSelectedSuggestion(null)
                    }}
                    disabled={actionLoading === selectedSuggestion.id}
                    className="inline-flex items-center gap-2 px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 disabled:opacity-50"
                  >
                    {actionLoading === selectedSuggestion.id ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <X className="h-4 w-4" />
                    )}
                    Dismiss
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
