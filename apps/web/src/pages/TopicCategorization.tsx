import { useEffect, useState } from 'react'
import {
  BarChart3,
  TrendingUp,
  Search,
  RefreshCw,
  Tag,
  AlertCircle,
  CheckCircle2,
  X,
} from '../components/icons'
import {
  getTopicSummary,
  overrideConversationTopic,
  batchCategorize,
  type TopicSummary as TopicSummaryType,
  type TopicCount,
  type KeywordCount,
  type TrendingTopic,
  type BatchCategorizationResult,
} from '../lib/api'

export function TopicCategorization() {
  const [summary, setSummary] = useState<TopicSummaryType | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [batching, setBatching] = useState(false)
  const [batchResult, setBatchResult] = useState<BatchCategorizationResult | null>(null)

  // Override modal state
  const [showOverrideModal, setShowOverrideModal] = useState(false)
  const [overrideConversationId, setOverrideConversationId] = useState('')
  const [overrideCurrentTopic, setOverrideCurrentTopic] = useState('')
  const [overrideNewTopic, setOverrideNewTopic] = useState('')
  const [overrideNewSubtopic, setOverrideNewSubtopic] = useState('')
  const [overriding, setOverriding] = useState(false)

  const [activeView, setActiveView] = useState<'overview' | 'topics' | 'trending' | 'keywords'>('overview')

  const availableTopics = [
    'product_inquiry',
    'pricing',
    'support',
    'sales',
    'complaint',
    'refund',
    'scheduling',
    'account',
    'feedback',
    'general',
  ]

  useEffect(() => {
    loadSummary()
  }, [])

  const loadSummary = async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await getTopicSummary()
      setSummary(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load topic summary')
    } finally {
      setLoading(false)
    }
  }

  const handleBatchCategorize = async () => {
    setBatching(true)
    setBatchResult(null)
    try {
      const result = await batchCategorize()
      setBatchResult(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Batch categorization failed')
    } finally {
      setBatching(false)
    }
  }

  const handleOpenOverride = (conversationId: string, currentTopic: string) => {
    setOverrideConversationId(conversationId)
    setOverrideCurrentTopic(currentTopic)
    setOverrideNewTopic(currentTopic)
    setOverrideNewSubtopic('')
    setShowOverrideModal(true)
  }

  const handleOverride = async () => {
    setOverriding(true)
    try {
      await overrideConversationTopic(
        overrideConversationId,
        overrideNewTopic,
        overrideNewSubtopic || undefined,
        'manager'
      )
      setShowOverrideModal(false)
      await loadSummary()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Override failed')
    } finally {
      setOverriding(false)
    }
  }

  const formatTopic = (topic: string) => {
    return topic.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
  }

  const getTopicColor = (topic: string) => {
    const colors: Record<string, string> = {
      product_inquiry: '#3b82f6',
      pricing: '#10b981',
      support: '#f59e0b',
      sales: '#8b5cf6',
      complaint: '#ef4444',
      refund: '#f97316',
      scheduling: '#06b6d4',
      account: '#6366f1',
      feedback: '#ec4899',
      general: '#6b7280',
    }
    return colors[topic] || '#6b7280'
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading topic categorization data...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-red-500 flex items-center gap-2">
          <AlertCircle className="w-5 h-5" />
          {error}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Topic Categorization</h1>
          <p className="text-gray-500 mt-1">
            AI-powered conversation categorization for trend analysis and content gap identification
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleBatchCategorize}
            disabled={batching}
            className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-[var(--radius-md)] hover:bg-indigo-700 disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${batching ? 'animate-spin' : ''}`} />
            {batching ? 'Categorizing...' : 'Categorize All'}
          </button>
          <button
            onClick={loadSummary}
            className="p-2 text-gray-500 hover:text-gray-700 rounded-[var(--radius-md)] hover:bg-gray-100"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Batch result notification */}
      {batchResult && (
        <div className="bg-green-50 border border-green-200 rounded-[var(--radius-md)] p-4">
          <div className="flex items-center gap-2 text-green-800">
            <CheckCircle2 className="w-5 h-5" />
            <span className="font-medium">
              Batch complete: {batchResult.categorized} categorized, {batchResult.failed} failed out of {batchResult.total}
            </span>
          </div>
        </div>
      )}

      {/* Navigation tabs */}
      <div className="flex gap-1 border-b border-gray-200">
        {[
          { id: 'overview' as const, label: 'Overview', icon: BarChart3 },
          { id: 'topics' as const, label: 'Topic Distribution', icon: Tag },
          { id: 'trending' as const, label: 'Trending', icon: TrendingUp },
          { id: 'keywords' as const, label: 'Keywords', icon: Search },
        ].map((tab) => {
          const Icon = tab.icon
          const isActive = activeView === tab.id
          return (
            <button
              key={tab.id}
              onClick={() => setActiveView(tab.id)}
              className={`flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors ${
                isActive
                  ? 'border-indigo-600 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              <Icon className="w-4 h-4" />
              {tab.label}
            </button>
          )
        })}
      </div>

      {/* Overview View */}
      {activeView === 'overview' && summary && (
        <div className="space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-white border rounded-[var(--radius-md)] p-4">
              <div className="text-sm text-gray-500">Total Conversations</div>
              <div className="text-2xl font-bold mt-1">{summary.total_conversations}</div>
            </div>
            <div className="bg-white border rounded-[var(--radius-md)] p-4">
              <div className="text-sm text-gray-500">Categorized</div>
              <div className="text-2xl font-bold mt-1 text-green-600">{summary.categorized_count}</div>
            </div>
            <div className="bg-white border rounded-[var(--radius-md)] p-4">
              <div className="text-sm text-gray-500">Uncategorized</div>
              <div className="text-2xl font-bold mt-1 text-amber-600">{summary.uncategorized_count}</div>
            </div>
            <div className="bg-white border rounded-[var(--radius-md)] p-4">
              <div className="text-sm text-gray-500">Unique Topics</div>
              <div className="text-2xl font-bold mt-1 text-indigo-600">{summary.topic_counts.length}</div>
            </div>
          </div>

          {/* Top Topics */}
          <div className="bg-white border rounded-[var(--radius-md)] p-6">
            <h3 className="font-semibold text-gray-900 mb-4">Top Topics</h3>
            <div className="space-y-3">
              {summary.topic_counts.slice(0, 10).map((tc) => (
                <TopicBar key={tc.topic} topic={tc} getTopicColor={getTopicColor} formatTopic={formatTopic} />
              ))}
              {summary.topic_counts.length === 0 && (
                <div className="text-gray-400 text-center py-8">
                  No categorized conversations yet. Click "Categorize All" to start.
                </div>
              )}
            </div>
          </div>

          {/* Trending */}
          <div className="bg-white border rounded-[var(--radius-md)] p-6">
            <h3 className="font-semibold text-gray-900 mb-4">Trending Topics (7-day change)</h3>
            <div className="space-y-2">
              {summary.trending_topics.slice(0, 8).map((tt) => (
                <div key={tt.topic} className="flex items-center justify-between py-2 border-b border-gray-100 last:border-0">
                  <div className="flex items-center gap-3">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: getTopicColor(tt.topic) }}
                    />
                    <span className="text-sm font-medium text-gray-700">{formatTopic(tt.topic)}</span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-gray-500">
                      {tt.current_count} ({tt.previous_count})
                    </span>
                    <span
                      className={`font-medium ${
                        tt.change_percentage > 0 ? 'text-green-600' : tt.change_percentage < 0 ? 'text-red-600' : 'text-gray-400'
                      }`}
                    >
                      {tt.change_percentage > 0 ? '+' : ''}
                      {tt.change_percentage}%
                    </span>
                  </div>
                </div>
              ))}
              {summary.trending_topics.length === 0 && (
                <div className="text-gray-400 text-center py-4">No trend data available yet</div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Topics View */}
      {activeView === 'topics' && summary && (
        <div className="bg-white border rounded-[var(--radius-md)] p-6">
          <h3 className="font-semibold text-gray-900 mb-4">Topic Distribution</h3>
          <div className="space-y-4">
            {summary.topic_counts.map((tc) => (
              <div key={tc.topic}>
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-2">
                    <div className="w-3 h-3 rounded-full" style={{ backgroundColor: getTopicColor(tc.topic) }} />
                    <span className="text-sm font-medium text-gray-700">{formatTopic(tc.topic)}</span>
                  </div>
                  <div className="text-sm text-gray-500">
                    {tc.count} ({tc.percentage}%)
                  </div>
                </div>
                <div className="w-full bg-gray-100 rounded-full h-2">
                  <div
                    className="h-2 rounded-full transition-all"
                    style={{ width: `${Math.min(tc.percentage, 100)}%`, backgroundColor: getTopicColor(tc.topic) }}
                  />
                </div>
              </div>
            ))}
            {summary.topic_counts.length === 0 && (
              <div className="text-gray-400 text-center py-8">No data yet</div>
            )}
          </div>
        </div>
      )}

      {/* Trending View */}
      {activeView === 'trending' && summary && (
        <div className="bg-white border rounded-[var(--radius-md)] p-6">
          <h3 className="font-semibold text-gray-900 mb-4">Topic Trends</h3>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-gray-200">
                  <th className="text-left py-3 px-4 text-gray-500 font-medium">Topic</th>
                  <th className="text-right py-3 px-4 text-gray-500 font-medium">Current (7d)</th>
                  <th className="text-right py-3 px-4 text-gray-500 font-medium">Previous (7d)</th>
                  <th className="text-right py-3 px-4 text-gray-500 font-medium">Change</th>
                </tr>
              </thead>
              <tbody>
                {summary.trending_topics.map((tt) => (
                  <tr key={tt.topic} className="border-b border-gray-100 last:border-0">
                    <td className="py-3 px-4">
                      <div className="flex items-center gap-2">
                        <div className="w-3 h-3 rounded-full" style={{ backgroundColor: getTopicColor(tt.topic) }} />
                        <span className="font-medium text-gray-700">{formatTopic(tt.topic)}</span>
                      </div>
                    </td>
                    <td className="text-right py-3 px-4 text-gray-700">{tt.current_count}</td>
                    <td className="text-right py-3 px-4 text-gray-500">{tt.previous_count}</td>
                    <td className="text-right py-3 px-4">
                      <span
                        className={`font-medium ${
                          tt.change_percentage > 0
                            ? 'text-green-600'
                            : tt.change_percentage < 0
                            ? 'text-red-600'
                            : 'text-gray-400'
                        }`}
                      >
                        {tt.change_percentage > 0 ? '+' : ''}
                        {tt.change_percentage}%
                      </span>
                    </td>
                  </tr>
                ))}
                {summary.trending_topics.length === 0 && (
                  <tr>
                    <td colSpan={4} className="text-center py-8 text-gray-400">
                      No trend data available yet
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Keywords View */}
      {activeView === 'keywords' && summary && (
        <div className="bg-white border rounded-[var(--radius-md)] p-6">
          <h3 className="font-semibold text-gray-900 mb-4">Top Keywords</h3>
          <div className="flex flex-wrap gap-2">
            {summary.top_keywords.map((kw) => (
              <span
                key={kw.keyword}
                className="inline-flex items-center gap-1 px-3 py-1.5 bg-gray-100 rounded-full text-sm text-gray-700"
                style={{
                  fontSize: `${Math.max(12, Math.min(20, 12 + kw.count))}px`,
                  backgroundColor: `rgba(99, 102, 241, ${Math.min(0.3, 0.05 + kw.count * 0.02)})`,
                }}
              >
                {kw.keyword}
                <span className="text-xs text-gray-400">({kw.count})</span>
              </span>
            ))}
            {summary.top_keywords.length === 0 && (
              <div className="text-gray-400 text-center py-8 w-full">No keywords extracted yet</div>
            )}
          </div>
        </div>
      )}

      {/* Override Modal */}
      {showOverrideModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-[var(--radius-md)] shadow-xl w-full max-w-md p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-900">Override Topic</h3>
              <button
                onClick={() => setShowOverrideModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Current Topic</label>
                <div className="px-3 py-2 bg-gray-50 rounded-[var(--radius-md)] text-sm text-gray-600">
                  {formatTopic(overrideCurrentTopic)}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">New Topic</label>
                <select
                  value={overrideNewTopic}
                  onChange={(e) => setOverrideNewTopic(e.target.value)}
                  className="w-full px-3 py-2 border rounded-[var(--radius-md)] text-sm"
                >
                  {availableTopics.map((t) => (
                    <option key={t} value={t}>
                      {formatTopic(t)}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Subtopic (optional)</label>
                <input
                  type="text"
                  value={overrideNewSubtopic}
                  onChange={(e) => setOverrideNewSubtopic(e.target.value)}
                  placeholder="e.g., features, bug_report, quote_request"
                  className="w-full px-3 py-2 border rounded-[var(--radius-md)] text-sm"
                />
              </div>

              <div className="flex justify-end gap-2">
                <button
                  onClick={() => setShowOverrideModal(false)}
                  className="px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-[var(--radius-md)]"
                >
                  Cancel
                </button>
                <button
                  onClick={handleOverride}
                  disabled={overriding}
                  className="px-4 py-2 text-sm bg-indigo-600 text-white rounded-[var(--radius-md)] hover:bg-indigo-700 disabled:opacity-50"
                >
                  {overriding ? 'Saving...' : 'Apply Override'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

// Topic bar component
function TopicBar({
  topic,
  getTopicColor,
  formatTopic,
}: {
  topic: TopicCount
  getTopicColor: (t: string) => string
  formatTopic: (t: string) => string
}) {
  return (
    <div>
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full" style={{ backgroundColor: getTopicColor(topic.topic) }} />
          <span className="text-sm font-medium text-gray-700">{formatTopic(topic.topic)}</span>
        </div>
        <div className="text-sm text-gray-500">
          {topic.count} ({topic.percentage}%)
        </div>
      </div>
      <div className="w-full bg-gray-100 rounded-full h-2">
        <div
          className="h-2 rounded-full transition-all"
          style={{ width: `${Math.min(topic.percentage, 100)}%`, backgroundColor: getTopicColor(topic.topic) }}
        />
      </div>
    </div>
  )
}
