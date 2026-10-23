import { useEffect, useRef, useState } from 'react'
import {
  api,
  type KnowledgeBaseEntry,
  type KnowledgeGap,
  type KnowledgeHubOverview,
  type KnowledgeRecommendation,
} from '../lib/api'

export function KnowledgeBase() {
  const [activeTab, setActiveTab] = useState<'library' | 'performance' | 'recommendations' | 'gaps'>('library')
  const [hubOverview, setHubOverview] = useState<KnowledgeHubOverview | null>(null)
  const [recommendations, setRecommendations] = useState<KnowledgeRecommendation[]>([])
  const [gaps, setGaps] = useState<KnowledgeGap[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Filters
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedTopic, setSelectedTopic] = useState('all')
  const [selectedTouchpoint, setSelectedTouchpoint] = useState('all')
  const [recommendationFilter, setRecommendationFilter] = useState<'all' | 'content_gap' | 'customer_data_gap' | 'action_gap'>('all')

  // Modals & Forms
  const [showNoteForm, setShowNoteForm] = useState(false)
  const [showUrlModal, setShowUrlModal] = useState(false)
  const [urlInput, setUrlInput] = useState('')
  const [ingestingUrl, setIngestingUrl] = useState(false)

  // Note form fields
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [noteTopic, setNoteTopic] = useState('Billing & Refunds')
  const [savingNote, setSavingNote] = useState(false)

  // Document upload
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploadingDoc, setUploadingDoc] = useState(false)

  // Recommendation & Gap actions
  const [processingRecId, setProcessingRecId] = useState<string | null>(null)
  const [scanningGaps, setScanningGaps] = useState(false)
  const [selectedEntryPreview, setSelectedEntryPreview] = useState<KnowledgeBaseEntry | null>(null)

  function loadHub() {
    setLoading(true)
    Promise.all([
      api.getKnowledgeHubOverview({
        q: searchQuery,
        topic: selectedTopic !== 'all' ? selectedTopic : undefined,
        touchpoint: selectedTouchpoint !== 'all' ? selectedTouchpoint : undefined,
      }),
      api.listKnowledgeRecommendations('open'),
      api.listKnowledgeGaps('suggested'),
    ])
      .then(([hub, recs, g]) => {
        setHubOverview(hub)
        setRecommendations(recs || [])
        setGaps(g || [])
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false))
  }

  useEffect(() => {
    loadHub()
  }, [selectedTopic, selectedTouchpoint])

  function handleSearchSubmit(e: React.FormEvent) {
    e.preventDefault()
    loadHub()
  }

  async function submitNote(e: React.FormEvent) {
    e.preventDefault()
    if (!title.trim() || !content.trim()) return
    setSavingNote(true)
    setError(null)
    try {
      await api.createKnowledgeBaseEntry({
        title: title.trim(),
        content: content.trim(),
        source_type: 'note',
        topic: noteTopic,
      })
      setTitle('')
      setContent('')
      setShowNoteForm(false)
      setSuccess('Knowledge article created and deployed to selected touchpoints.')
      loadHub()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSavingNote(false)
    }
  }

  async function submitUrl(e: React.FormEvent) {
    e.preventDefault()
    if (!urlInput.trim()) return
    setIngestingUrl(true)
    setError(null)
    try {
      const entry = await api.ingestUrl(urlInput.trim())
      setUrlInput('')
      setShowUrlModal(false)
      setSuccess(`Successfully ingested knowledge from ${entry.title || urlInput}`)
      loadHub()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setIngestingUrl(false)
    }
  }

  async function handleFileUpload(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    setUploadingDoc(true)
    setError(null)
    try {
      const entry = await api.uploadKnowledgeDocument(file)
      setSuccess(`Successfully parsed document "${entry.title}" (${file.name}) into Knowledge Hub`)
      if (fileInputRef.current) fileInputRef.current.value = ''
      loadHub()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setUploadingDoc(false)
    }
  }

  async function handleApplyRecommendation(rec: KnowledgeRecommendation) {
    setProcessingRecId(rec.id)
    setError(null)
    try {
      await api.applyKnowledgeRecommendation(rec.id)
      setSuccess(`Applied recommendation: "${rec.title}"! Article created in ${rec.topic} (+${rec.impactOnResolution}% Resolution).`)
      loadHub()
    } catch (e: any) {
      setError(e.message || 'Failed to apply recommendation')
    } finally {
      setProcessingRecId(null)
    }
  }

  async function handleDismissRecommendation(id: string) {
    try {
      await api.dismissKnowledgeRecommendation(id)
      setRecommendations((prev) => prev.filter((r) => r.id !== id))
    } catch (e: any) {
      setError(e.message || 'Failed to dismiss recommendation')
    }
  }

  async function handleApproveGap(gap: KnowledgeGap) {
    try {
      await api.approveKnowledgeGap(gap.id, gap.suggestedAnswer)
      setSuccess(`Approved "${gap.question}" and added to Knowledge Base!`)
      loadHub()
    } catch (e: any) {
      setError(e.message || 'Failed to approve gap')
    }
  }

  async function handleScanGaps() {
    setScanningGaps(true)
    try {
      const res = await api.scanKnowledgeGaps()
      setSuccess(`Scan completed. Detected ${res.detectedCount} new potential knowledge gaps from conversations.`)
      loadHub()
    } catch (e: any) {
      setError(e.message || 'Failed to scan gaps')
    } finally {
      setScanningGaps(false)
    }
  }

  async function handleDeleteSource(id: string) {
    if (!confirm('Are you sure you want to remove this knowledge source?')) return
    try {
      await api.deleteKnowledgeBaseEntry(id)
      setSuccess('Knowledge article removed.')
      loadHub()
    } catch (e: any) {
      setError(e.message || 'Failed to delete article')
    }
  }

  const allTopics = [
    'all',
    'Billing & Refunds',
    'Shipping & Delivery',
    'Orders & Returns',
    'Product & Catalog',
    'Account & VIP Access',
  ]

  const totalResolutionRate = hubOverview?.overallResolutionRate || 89.2
  const totalSourcesCount = hubOverview?.totalSources || 0
  const openRecsCount = recommendations.filter((r) => r.status === 'open').length

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Top Banner & Header */}
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4 border-b border-[var(--color-border)] pb-5">
        <div>
          <div className="flex items-center gap-2.5">
            <div className="w-10 h-10 rounded-xl bg-purple-600 text-white flex items-center justify-center font-bold text-lg shadow-sm">
              📚
            </div>
            <div>
              <h1 className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
                <span>Centralized Knowledge Hub</span>
                <span className="rounded-full bg-purple-100 text-purple-700 px-2.5 py-0.5 text-xs font-semibold">
                  Omni-Touchpoint
                </span>
              </h1>
              <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
                Centralize, manage, and optimize all information powering Fin AI Agent, Copilot in the inbox, and customer self-serve support.
              </p>
            </div>
          </div>
        </div>

        {/* Global Hub Action Buttons */}
        <div className="flex flex-wrap items-center gap-2">
          <button
            onClick={() => {
              setShowNoteForm(true)
              setShowUrlModal(false)
            }}
            className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-surface)] border border-slate-300 px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-2xs transition-colors cursor-pointer"
          >
            <span>➕ Add Source Note</span>
          </button>

          <button
            onClick={() => {
              setShowUrlModal(true)
              setShowNoteForm(false)
            }}
            className="inline-flex items-center gap-1.5 rounded-lg bg-blue-50 border border-blue-200 px-3 py-1.5 text-xs font-semibold text-blue-700 hover:bg-blue-100 shadow-2xs transition-colors cursor-pointer"
          >
            <span>🔗 Ingest URL</span>
          </button>

          <input
            ref={fileInputRef}
            type="file"
            accept=".pdf,.txt,.md,.markdown"
            onChange={handleFileUpload}
            className="hidden"
          />
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={uploadingDoc}
            className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent-soft)] border border-[var(--color-border)] px-3 py-1.5 text-xs font-semibold text-[var(--color-accent)] hover:bg-indigo-100 shadow-2xs transition-colors disabled:opacity-50 cursor-pointer"
          >
            <span>📄 {uploadingDoc ? 'Parsing...' : 'Upload PDF/Doc'}</span>
          </button>

          <button
            onClick={handleScanGaps}
            disabled={scanningGaps}
            className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow-xs transition-colors disabled:opacity-50 cursor-pointer"
          >
            <span>✨ {scanningGaps ? 'Scanning...' : 'Scan Gaps'}</span>
          </button>
        </div>
      </div>

      {/* Notifications */}
      {error && (
        <div className="flex items-center justify-between rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-xs text-rose-800 shadow-xs">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="font-bold text-sm cursor-pointer">✕</button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-xs text-emerald-800 shadow-xs">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="font-bold text-sm cursor-pointer">✕</button>
        </div>
      )}

      {/* Performance Summary Banner Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5">
        <div className="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-4 shadow-2xs">
          <div className="flex items-center justify-between text-xs text-[var(--color-ink-muted)] mb-1">
            <span>Total Knowledge Sources</span>
            <span>📚</span>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-black text-[var(--color-ink)]">{totalSourcesCount}</span>
            <span className="text-xs text-[var(--color-ink-faint)] font-normal">sources</span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-muted)] mt-1">Across 5 structured business topics</p>
        </div>

        <div className="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-4 shadow-2xs">
          <div className="flex items-center justify-between text-xs text-[var(--color-ink-muted)] mb-1">
            <span>Fin Agent Resolution Rate</span>
            <span>🤖</span>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-black text-emerald-600">{totalResolutionRate}%</span>
            <span className="text-xs font-bold text-emerald-600 bg-emerald-50 px-1.5 py-0.2 rounded">+2.4%</span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-muted)] mt-1">Autonomous answers without escalation</p>
        </div>

        <div className="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-4 shadow-2xs">
          <div className="flex items-center justify-between text-xs text-[var(--color-ink-muted)] mb-1">
            <span>Copilot Inbox Assistance</span>
            <span>💬</span>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-black text-purple-600">93.7%</span>
            <span className="text-xs text-[var(--color-ink-faint)]">adoption</span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-muted)] mt-1">Teammate reply insertions from Hub</p>
        </div>

        <div className="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-4 shadow-2xs">
          <div className="flex items-center justify-between text-xs text-[var(--color-ink-muted)] mb-1">
            <span>Content Gap Recommendations</span>
            <span>🎯</span>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-black text-amber-600">{openRecsCount}</span>
            <span className="text-xs font-bold text-amber-700 bg-amber-50 px-1.5 py-0.2 rounded">+13.3% impact</span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-muted)] mt-1">Gaps to close for higher resolution</p>
        </div>
      </div>

      {/* Main Tabs Navigation */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] text-xs">
        <div className="flex items-center gap-6">
          <button
            onClick={() => setActiveTab('library')}
            className={`pb-3 font-bold transition-all border-b-2 cursor-pointer flex items-center gap-2 ${
              activeTab === 'library'
                ? 'border-purple-600 text-purple-700'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
            }`}
          >
            <span>📁 Content Library by Topic</span>
            <span className="rounded-full bg-slate-100 text-[var(--color-ink)] px-2 py-0.5 text-[10px] font-semibold">
              {totalSourcesCount}
            </span>
          </button>

          <button
            onClick={() => setActiveTab('performance')}
            className={`pb-3 font-bold transition-all border-b-2 cursor-pointer flex items-center gap-2 ${
              activeTab === 'performance'
                ? 'border-purple-600 text-purple-700'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
            }`}
          >
            <span>📊 Cross-Touchpoint Reporting</span>
          </button>

          <button
            onClick={() => setActiveTab('recommendations')}
            className={`pb-3 font-bold transition-all border-b-2 cursor-pointer flex items-center gap-2 ${
              activeTab === 'recommendations'
                ? 'border-purple-600 text-purple-700'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
            }`}
          >
            <span>🎯 Content &amp; Action Gaps</span>
            {openRecsCount > 0 && (
              <span className="rounded-full bg-amber-500 text-white font-bold px-1.5 py-0.2 text-[10px] animate-pulse">
                {openRecsCount}
              </span>
            )}
          </button>

          <button
            onClick={() => setActiveTab('gaps')}
            className={`pb-3 font-bold transition-all border-b-2 cursor-pointer flex items-center gap-2 ${
              activeTab === 'gaps'
                ? 'border-purple-600 text-purple-700'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
            }`}
          >
            <span>✨ FAQ Gaps ({gaps.length})</span>
          </button>
        </div>
      </div>

      {/* ========================================================================= */}
      {/* TAB 1: CONTENT LIBRARY GROUPED BY TOPIC                                    */}
      {/* ========================================================================= */}
      {activeTab === 'library' && (
        <div className="space-y-6">
          {/* Filter Bar */}
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 bg-[var(--color-surface)] p-3 rounded-2xl border border-[var(--color-border)]">
            {/* Search */}
            <form onSubmit={handleSearchSubmit} className="flex items-center gap-2 flex-1 max-w-md">
              <input
                type="text"
                placeholder="Search articles, policies, or topics..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full text-xs border border-slate-300 rounded-lg px-3 py-1.5 focus:outline-hidden focus:border-purple-500"
              />
              <button
                type="submit"
                className="bg-[var(--color-accent)] text-white text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-[var(--color-accent-hover)] transition-colors cursor-pointer"
              >
                Search
              </button>
            </form>

            <div className="flex flex-wrap items-center gap-3">
              {/* Touchpoint Filter */}
              <div className="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
                <span className="text-[11px] font-semibold text-[var(--color-ink-faint)]">Touchpoint:</span>
                <select
                  value={selectedTouchpoint}
                  onChange={(e) => setSelectedTouchpoint(e.target.value)}
                  className="text-xs border border-slate-300 rounded-lg px-2 py-1 bg-[var(--color-surface)] focus:outline-hidden cursor-pointer"
                >
                  <option value="all">All Touchpoints</option>
                  <option value="agent">🤖 Fin Agent</option>
                  <option value="copilot">💬 Inbox Copilot</option>
                  <option value="help_center">🌐 Help Center</option>
                </select>
              </div>

              {/* Topic Filter Pills */}
              <div className="flex items-center gap-1 overflow-x-auto">
                {allTopics.map((top) => (
                  <button
                    key={top}
                    type="button"
                    onClick={() => setSelectedTopic(top)}
                    className={`px-2.5 py-1 rounded-full text-[11px] font-semibold capitalize transition-all cursor-pointer whitespace-nowrap ${
                      selectedTopic === top
                        ? 'bg-purple-600 text-white shadow-2xs'
                        : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
                    }`}
                  >
                    {top === 'all' ? 'All Topics' : top}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Grouped Topics Content */}
          {loading ? (
            <div className="py-16 text-center text-xs text-[var(--color-ink-muted)]">Loading Centralized Knowledge Hub...</div>
          ) : hubOverview?.topics.length === 0 ? (
            <div className="py-16 text-center bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] p-6">
              <p className="text-xs text-[var(--color-ink-muted)]">No knowledge sources found matching your filters.</p>
              <button
                type="button"
                onClick={() => {
                  setSelectedTopic('all')
                  setSelectedTouchpoint('all')
                  setSearchQuery('')
                }}
                className="mt-2 text-xs font-bold text-purple-600 hover:underline cursor-pointer"
              >
                Reset filters
              </button>
            </div>
          ) : (
            <div className="space-y-6">
              {hubOverview?.topics.map((group) => (
                <div key={group.name} className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] overflow-hidden shadow-2xs">
                  {/* Topic Group Header */}
                  <div className="bg-[var(--color-page)]/80 px-4 py-3 border-b border-[var(--color-border)] flex items-center justify-between">
                    <div className="flex items-center gap-2.5">
                      <span className="text-xl">{group.icon}</span>
                      <div>
                        <h3 className="font-bold text-[var(--color-ink)] text-sm">{group.name}</h3>
                        <p className="text-[11px] text-[var(--color-ink-muted)]">
                          {group.sourcesCount} knowledge {group.sourcesCount === 1 ? 'source' : 'sources'}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center gap-3">
                      <div className="text-right">
                        <span className="text-[10px] text-[var(--color-ink-faint)] uppercase font-semibold block">Avg Resolution</span>
                        <span className="text-xs font-bold text-emerald-700 bg-emerald-50 px-2 py-0.5 rounded-full border border-emerald-200">
                          {group.avgResolutionRate}%
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* Sources List in Topic */}
                  <div className="divide-y divide-slate-100 p-2">
                    {group.sources.map((source) => (
                      <div
                        key={source.id}
                        className="p-3 hover:bg-[var(--color-page)]/70 rounded-xl transition-colors flex flex-col md:flex-row md:items-start justify-between gap-3"
                      >
                        <div className="space-y-1.5 flex-1">
                          <div className="flex flex-wrap items-center gap-2">
                            <h4
                              onClick={() => setSelectedEntryPreview(source)}
                              className="font-bold text-[var(--color-ink)] text-xs hover:text-purple-700 cursor-pointer"
                            >
                              {source.title}
                            </h4>
                            <span className="text-[10px] uppercase font-semibold text-[var(--color-ink-muted)] bg-slate-100 px-1.5 py-0.2 rounded border border-[var(--color-border)]">
                              {source.sourceType}
                            </span>
                          </div>

                          <p className="text-xs text-[var(--color-ink-muted)] line-clamp-2 leading-relaxed">
                            {source.content}
                          </p>

                          {/* Deployment Touchpoints & Stats */}
                          <div className="flex flex-wrap items-center gap-3 text-[11px] text-[var(--color-ink-muted)] pt-1">
                            <div className="flex items-center gap-1">
                              <span className="text-[var(--color-ink-faint)] font-medium">Touchpoints:</span>
                              {(source.targetTouchpoints || []).map((tp) => (
                                <span
                                  key={tp}
                                  className="text-[10px] font-semibold px-1.5 py-0.2 rounded bg-purple-50 text-purple-700 border border-purple-200"
                                >
                                  {tp === 'agent' ? '🤖 Fin Agent' : tp === 'copilot' ? '💬 Copilot' : '🌐 Help Center'}
                                </span>
                              ))}
                            </div>

                            <span>•</span>
                            <span className="text-emerald-700 font-semibold">
                              ⚡ {source.resolutionRate}% Resolution
                            </span>
                            <span>•</span>
                            <span>📊 {source.usageCount} Invocations</span>
                            <span>•</span>
                            <span>👍 {source.helpfulCount} helpful</span>
                          </div>
                        </div>

                        {/* Actions */}
                        <div className="flex items-center gap-2 shrink-0 pt-1">
                          <button
                            type="button"
                            onClick={() => setSelectedEntryPreview(source)}
                            className="text-xs text-purple-600 hover:text-purple-800 font-semibold px-2.5 py-1 rounded hover:bg-purple-50 cursor-pointer transition-colors"
                          >
                            View
                          </button>
                          <button
                            type="button"
                            onClick={() => handleDeleteSource(source.id)}
                            className="text-xs text-rose-500 hover:text-rose-700 p-1 rounded hover:bg-rose-50 cursor-pointer transition-colors"
                            title="Delete article"
                          >
                            🗑️
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* ========================================================================= */}
      {/* TAB 2: CROSS-TOUCHPOINT REPORTING & ANALYTICS                             */}
      {/* ========================================================================= */}
      {activeTab === 'performance' && (
        <div className="space-y-6">
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] p-5 shadow-2xs space-y-4">
            <h3 className="font-bold text-[var(--color-ink)] text-sm">Touchpoint Deployment &amp; Performance Breakdown</h3>
            <p className="text-xs text-[var(--color-ink-muted)]">
              See how your centralized knowledge sources are performing across Fin AI, teammate inbox Copilot, and self-serve customer Help Center.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2">
              {/* Fin Agent */}
              <div className="border border-purple-200 bg-purple-50/40 rounded-xl p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 font-bold text-xs text-purple-900">
                    <span className="text-base">🤖</span>
                    <span>Fin Autonomous Agent</span>
                  </div>
                  <span className="text-[10px] bg-purple-200 text-purple-800 font-bold px-2 py-0.5 rounded-full">
                    {hubOverview?.touchpointBreakdown.agent.sourcesDeployed} Sources
                  </span>
                </div>
                <div className="space-y-1">
                  <div className="flex justify-between text-xs text-[var(--color-ink-muted)]">
                    <span>Autonomous Resolution Rate:</span>
                    <strong className="text-emerald-700">{hubOverview?.touchpointBreakdown.agent.resolutionRate}%</strong>
                  </div>
                  <div className="flex justify-between text-xs text-[var(--color-ink-muted)]">
                    <span>Total Deflections:</span>
                    <strong className="text-slate-800">{hubOverview?.touchpointBreakdown.agent.deflectionsCount} inquiries</strong>
                  </div>
                </div>
                <p className="text-[11px] text-purple-700 italic">
                  Answers customer tickets automatically across WhatsApp, Email, Live Chat without escalation.
                </p>
              </div>

              {/* Copilot */}
              <div className="border border-blue-200 bg-blue-50/40 rounded-xl p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 font-bold text-xs text-blue-900">
                    <span className="text-base">💬</span>
                    <span>Inbox Copilot Assistant</span>
                  </div>
                  <span className="text-[10px] bg-blue-200 text-blue-800 font-bold px-2 py-0.5 rounded-full">
                    {hubOverview?.touchpointBreakdown.copilot.sourcesDeployed} Sources
                  </span>
                </div>
                <div className="space-y-1">
                  <div className="flex justify-between text-xs text-[var(--color-ink-muted)]">
                    <span>Teammate Adoption Rate:</span>
                    <strong className="text-blue-700">{hubOverview?.touchpointBreakdown.copilot.resolutionRate}%</strong>
                  </div>
                  <div className="flex justify-between text-xs text-[var(--color-ink-muted)]">
                    <span>Replies Inserted:</span>
                    <strong className="text-slate-800">{hubOverview?.touchpointBreakdown.copilot.insertionsCount} insertions</strong>
                  </div>
                </div>
                <p className="text-[11px] text-blue-700 italic">
                  Proactively suggests grounded answers to human reps directly in the conversation composer.
                </p>
              </div>

              {/* Help Center */}
              <div className="border border-emerald-200 bg-emerald-50/40 rounded-xl p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 font-bold text-xs text-emerald-900">
                    <span className="text-base">🌐</span>
                    <span>Customer Help Center</span>
                  </div>
                  <span className="text-[10px] bg-emerald-200 text-emerald-800 font-bold px-2 py-0.5 rounded-full">
                    {hubOverview?.touchpointBreakdown.help_center.sourcesDeployed} Sources
                  </span>
                </div>
                <div className="space-y-1">
                  <div className="flex justify-between text-xs text-[var(--color-ink-muted)]">
                    <span>Helpful Feedback Rate:</span>
                    <strong className="text-emerald-700">{hubOverview?.touchpointBreakdown.help_center.satisfactionPct}%</strong>
                  </div>
                  <div className="flex justify-between text-xs text-[var(--color-ink-muted)]">
                    <span>Self-Serve Deflection:</span>
                    <strong className="text-slate-800">3,420 views</strong>
                  </div>
                </div>
                <p className="text-[11px] text-emerald-700 italic">
                  Public searchable documentation enabling customers to resolve questions before reaching out.
                </p>
              </div>
            </div>
          </div>

          {/* Topic Performance Matrix */}
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] overflow-hidden shadow-2xs">
            <div className="px-5 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
              <h4 className="font-bold text-[var(--color-ink)] text-xs uppercase tracking-wider">
                Topic Performance &amp; Deflection Efficiency
              </h4>
              <span className="text-xs text-[var(--color-ink-muted)]">{hubOverview?.topics.length} active topics</span>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs">
                <thead className="bg-[var(--color-page)] text-[var(--color-ink-muted)] uppercase text-[10px] font-bold">
                  <tr>
                    <th className="px-4 py-3">Topic Name</th>
                    <th className="px-4 py-3">Sources</th>
                    <th className="px-4 py-3">Resolution Rate</th>
                    <th className="px-4 py-3">Usage Activity</th>
                    <th className="px-4 py-3">Health Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {hubOverview?.topics.map((t) => (
                    <tr key={t.name} className="hover:bg-[var(--color-page)]/70">
                      <td className="px-4 py-3 font-semibold text-[var(--color-ink)] flex items-center gap-2">
                        <span>{t.icon}</span>
                        <span>{t.name}</span>
                      </td>
                      <td className="px-4 py-3 text-[var(--color-ink-muted)]">{t.sourcesCount} articles</td>
                      <td className="px-4 py-3">
                        <span className="font-bold text-emerald-700 bg-emerald-50 px-2 py-0.5 rounded-full border border-emerald-200">
                          {t.avgResolutionRate}%
                        </span>
                      </td>
                      <td className="px-4 py-3 text-[var(--color-ink-muted)]">
                        {t.sources.reduce((acc, curr) => acc + (curr.usageCount || 0), 0)} total inquiries
                      </td>
                      <td className="px-4 py-3">
                        <span className="inline-flex items-center gap-1 text-[11px] font-semibold text-emerald-700">
                          <span className="w-2 h-2 rounded-full bg-emerald-500"></span>
                          Healthy
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* TAB 3: RECOMMENDATIONS (CONTENT, CUSTOMER DATA & ACTION GAPS)              */}
      {/* ========================================================================= */}
      {activeTab === 'recommendations' && (
        <div className="space-y-6">
          {/* Header Banner */}
          <div className="bg-linear-to-r from-amber-500 via-orange-500 to-amber-600 rounded-2xl p-5 text-white shadow-md space-y-2">
            <div className="flex items-center gap-2">
              <span className="text-xl">🎯</span>
              <h3 className="font-bold text-base">Recommendations to Improve Agent Resolution Rate</h3>
            </div>
            <p className="text-xs text-amber-100 max-w-3xl leading-relaxed">
              Agent Fin continuously analyzes failed customer queries, escalations, and human rep responses.
              Closing these recommendations will boost your automated resolution rate by up to <strong>+13.3%</strong> with 1-click actions.
            </p>
          </div>

          {/* Filter Pills */}
          <div className="flex items-center gap-2">
            {[
              { id: 'all', label: 'All Gaps' },
              { id: 'content_gap', label: 'Missing Content Gaps' },
              { id: 'customer_data_gap', label: 'Customer Data Integrations' },
              { id: 'action_gap', label: 'Action & Workflow Gaps' },
            ].map((f) => (
              <button
                key={f.id}
                type="button"
                onClick={() => setRecommendationFilter(f.id as any)}
                className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors cursor-pointer ${
                  recommendationFilter === f.id
                    ? 'bg-amber-600 text-white shadow-2xs'
                    : 'bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]'
                }`}
              >
                {f.label}
              </button>
            ))}
          </div>

          {/* Recommendations List */}
          <div className="space-y-4">
            {recommendations
              .filter((r) => recommendationFilter === 'all' || r.gapType === recommendationFilter)
              .map((rec) => (
                <div
                  key={rec.id}
                  className="bg-[var(--color-surface)] rounded-2xl border-2 border-amber-200/80 p-5 shadow-2xs space-y-3.5 hover:border-amber-400 transition-all"
                >
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-[var(--color-border)] pb-3">
                    <div className="flex items-center gap-2">
                      <span className="text-lg">
                        {rec.gapType === 'content_gap' ? '📝' : rec.gapType === 'customer_data_gap' ? '🔌' : '⚡'}
                      </span>
                      <div>
                        <div className="flex items-center gap-2">
                          <h4 className="font-bold text-[var(--color-ink)] text-sm">{rec.title}</h4>
                          <span className="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-amber-100 text-amber-900 border border-amber-300">
                            {rec.gapType.replace('_', ' ')}
                          </span>
                        </div>
                        <span className="text-[11px] text-[var(--color-ink-muted)]">Topic: {rec.topic}</span>
                      </div>
                    </div>

                    <div className="flex items-center gap-2">
                      <span className="bg-emerald-100 text-emerald-800 font-bold px-2.5 py-1 rounded-lg text-xs border border-emerald-300 shadow-2xs">
                        +{rec.impactOnResolution}% Resolution Impact
                      </span>
                    </div>
                  </div>

                  <p className="text-xs text-[var(--color-ink)] leading-relaxed font-normal">
                    {rec.description}
                  </p>

                  {/* Suggested Solution & Content Box */}
                  <div className="bg-amber-50/60 border border-amber-200 rounded-xl p-3 space-y-1.5 text-xs">
                    <span className="font-bold text-amber-900 text-[11px] block">
                      💡 Recommended Action: {rec.suggestedAction}
                    </span>
                    <p className="text-slate-800 italic bg-[var(--color-surface)]/80 p-2 rounded-lg border border-amber-200/50">
                      "{rec.suggestedContent}"
                    </p>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex items-center justify-end gap-2 pt-1">
                    <button
                      type="button"
                      onClick={() => handleDismissRecommendation(rec.id)}
                      className="text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] px-3 py-1.5 rounded-lg hover:bg-slate-100 cursor-pointer font-semibold transition-colors"
                    >
                      Dismiss
                    </button>
                    <button
                      type="button"
                      disabled={processingRecId === rec.id}
                      onClick={() => handleApplyRecommendation(rec)}
                      className="bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs px-4 py-2 rounded-xl shadow-xs transition-all cursor-pointer flex items-center gap-1.5 disabled:opacity-50"
                    >
                      <span>🚀</span>
                      <span>{processingRecId === rec.id ? 'Publishing…' : 'Apply with 1-Click'}</span>
                    </button>
                  </div>
                </div>
              ))}
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* TAB 4: SUGGESTED FAQS / DETECTED CONVERSATION GAPS                        */}
      {/* ========================================================================= */}
      {activeTab === 'gaps' && (
        <div className="space-y-4">
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] p-4 shadow-2xs flex items-center justify-between">
            <div>
              <h3 className="font-bold text-[var(--color-ink)] text-sm">Autonomous Knowledge Gap Detection ("Fills gaps")</h3>
              <p className="text-xs text-[var(--color-ink-muted)]">
                Identifies recurring unanswered questions from live customer conversations and turns them into grounded FAQs with 1 click.
              </p>
            </div>
            <button
              onClick={handleScanGaps}
              disabled={scanningGaps}
              className="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white text-xs font-semibold px-3.5 py-1.5 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
            >
              {scanningGaps ? 'Scanning...' : 'Scan Now'}
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
            {gaps.map((g) => (
              <div key={g.id} className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] p-4 shadow-2xs space-y-2.5">
                <div className="flex items-start justify-between gap-2">
                  <h4 className="font-bold text-[var(--color-ink)] text-xs">"{g.question}"</h4>
                  <span className="text-[10px] font-bold text-purple-700 bg-purple-100 px-2 py-0.5 rounded-full shrink-0">
                    Asked {g.frequency}x
                  </span>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)] bg-[var(--color-page)] p-2.5 rounded-xl border border-[var(--color-border)]">
                  {g.suggestedAnswer}
                </p>
                <div className="flex justify-end pt-1">
                  <button
                    type="button"
                    onClick={() => handleApproveGap(g)}
                    className="bg-purple-600 hover:bg-purple-700 text-white font-semibold text-xs px-3 py-1.5 rounded-lg shadow-2xs cursor-pointer transition-colors"
                  >
                    + Add to Knowledge Hub
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* SOURCE PREVIEW MODAL                                                       */}
      {/* ========================================================================= */}
      {selectedEntryPreview && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] shadow-2xl max-w-2xl w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-start justify-between border-b border-[var(--color-border)] pb-3">
              <div>
                <span className="text-[10px] uppercase font-bold text-purple-600 bg-purple-50 px-2 py-0.5 rounded-full">
                  {selectedEntryPreview.topic}
                </span>
                <h3 className="font-bold text-[var(--color-ink)] text-base mt-1">{selectedEntryPreview.title}</h3>
              </div>
              <button
                onClick={() => setSelectedEntryPreview(null)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] font-bold p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            <div className="text-xs text-[var(--color-ink)] leading-relaxed max-h-72 overflow-y-auto whitespace-pre-wrap bg-[var(--color-page)] p-4 rounded-xl border border-[var(--color-border)]/70 font-normal">
              {selectedEntryPreview.content}
            </div>

            <div className="flex items-center justify-between pt-2 border-t border-[var(--color-border)] text-xs text-[var(--color-ink-muted)]">
              <span className="text-emerald-700 font-bold">⚡ {selectedEntryPreview.resolutionRate}% Resolution Efficiency</span>
              <button
                onClick={() => setSelectedEntryPreview(null)}
                className="px-4 py-1.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white rounded-lg font-semibold cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* CREATE KNOWLEDGE NOTE MODAL                                               */}
      {/* ========================================================================= */}
      {showNoteForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] shadow-2xl max-w-lg w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-start justify-between border-b border-[var(--color-border)] pb-2">
              <h3 className="font-bold text-[var(--color-ink)] text-base">Add Centralized Knowledge Source</h3>
              <button onClick={() => setShowNoteForm(false)} className="text-[var(--color-ink-faint)] font-bold p-1 cursor-pointer">✕</button>
            </div>

            <form onSubmit={submitNote} className="space-y-3 text-xs">
              <div>
                <label className="block text-[11px] font-semibold text-[var(--color-ink)] mb-1">Article / Policy Title</label>
                <input
                  type="text"
                  placeholder="e.g. Expedited Shipping Rules"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  className="w-full border border-slate-300 rounded-lg px-2.5 py-1.5 focus:outline-hidden focus:border-purple-500"
                  required
                />
              </div>

              <div>
                <label className="block text-[11px] font-semibold text-[var(--color-ink)] mb-1">Topic / Category</label>
                <select
                  value={noteTopic}
                  onChange={(e) => setNoteTopic(e.target.value)}
                  className="w-full border border-slate-300 rounded-lg px-2.5 py-1.5 bg-[var(--color-surface)] focus:outline-hidden focus:border-purple-500"
                >
                  <option value="Billing & Refunds">Billing &amp; Refunds</option>
                  <option value="Shipping & Delivery">Shipping &amp; Delivery</option>
                  <option value="Orders & Returns">Orders &amp; Returns</option>
                  <option value="Product & Catalog">Product &amp; Catalog</option>
                  <option value="Account & VIP Access">Account &amp; VIP Access</option>
                  <option value="General">General</option>
                </select>
              </div>

              <div>
                <label className="block text-[11px] font-semibold text-[var(--color-ink)] mb-1">Knowledge Content</label>
                <textarea
                  rows={4}
                  placeholder="Write policy details, FAQ instructions, or process steps..."
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  className="w-full border border-slate-300 rounded-lg p-2.5 focus:outline-hidden focus:border-purple-500"
                  required
                />
              </div>

              <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
                <button
                  type="button"
                  onClick={() => setShowNoteForm(false)}
                  className="px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:text-slate-800"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={savingNote}
                  className="bg-purple-600 hover:bg-purple-700 text-white font-bold text-xs px-4 py-1.5 rounded-lg shadow-xs cursor-pointer transition-colors"
                >
                  {savingNote ? 'Publishing…' : 'Publish to Hub'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* INGEST URL MODAL                                                           */}
      {/* ========================================================================= */}
      {showUrlModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] shadow-2xl max-w-md w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-start justify-between border-b border-[var(--color-border)] pb-2">
              <h3 className="font-bold text-[var(--color-ink)] text-base">Ingest Knowledge from URL</h3>
              <button onClick={() => setShowUrlModal(false)} className="text-[var(--color-ink-faint)] font-bold p-1 cursor-pointer">✕</button>
            </div>

            <form onSubmit={submitUrl} className="space-y-3 text-xs">
              <div>
                <label className="block text-[11px] font-semibold text-[var(--color-ink)] mb-1">Help Center or Doc URL</label>
                <input
                  type="url"
                  placeholder="https://help.example.com/shipping-policy"
                  value={urlInput}
                  onChange={(e) => setUrlInput(e.target.value)}
                  className="w-full border border-slate-300 rounded-lg px-2.5 py-1.5 focus:outline-hidden focus:border-blue-500"
                  required
                />
              </div>

              <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
                <button
                  type="button"
                  onClick={() => setShowUrlModal(false)}
                  className="px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:text-slate-800"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={ingestingUrl}
                  className="bg-blue-600 hover:bg-blue-700 text-white font-bold text-xs px-4 py-1.5 rounded-lg shadow-xs cursor-pointer transition-colors"
                >
                  {ingestingUrl ? 'Crawling…' : 'Crawl & Ingest'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
