import { useEffect, useRef, useState } from 'react'
import { api, type KnowledgeBaseEntry, type KnowledgeGap } from '../lib/api'

export function KnowledgeBase() {
  const [activeTab, setActiveTab] = useState<'articles' | 'gaps'>('articles')
  const [entries, setEntries] = useState<KnowledgeBaseEntry[] | null>(null)
  const [gaps, setGaps] = useState<KnowledgeGap[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Forms & Modals
  const [showNoteForm, setShowNoteForm] = useState(false)
  const [showUrlModal, setShowUrlModal] = useState(false)
  const [urlInput, setUrlInput] = useState('')
  const [ingestingUrl, setIngestingUrl] = useState(false)

  // Note form fields
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [savingNote, setSavingNote] = useState(false)

  // File Upload
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploadingDoc, setUploadingDoc] = useState(false)

  // Gap scanning & processing
  const [scanningGaps, setScanningGaps] = useState(false)
  const [editingGapId, setEditingGapId] = useState<string | null>(null)
  const [gapCustomAnswer, setGapCustomAnswer] = useState<string>('')
  const [processingGapId, setProcessingGapId] = useState<string | null>(null)

  function loadAll() {
    api
      .listKnowledgeBase()
      .then(setEntries)
      .catch((e) => setError(e.message))

    api
      .listKnowledgeGaps('suggested')
      .then(setGaps)
      .catch((e) => setError(e.message))
  }

  useEffect(loadAll, [])

  async function submitNote(e: React.FormEvent) {
    e.preventDefault()
    setSavingNote(true)
    setError(null)
    try {
      await api.createKnowledgeBaseEntry({ title, content, source_type: 'note' })
      setTitle('')
      setContent('')
      setShowNoteForm(false)
      setSuccess('Knowledge note added successfully.')
      loadAll()
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
      loadAll()
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
      setSuccess(`Successfully parsed and added document "${entry.title}" (${file.name})`)
      if (fileInputRef.current) fileInputRef.current.value = ''
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setUploadingDoc(false)
    }
  }

  async function handleScanGaps() {
    setScanningGaps(true)
    setError(null)
    try {
      const res = await api.scanKnowledgeGaps()
      setSuccess(`AI gap detection completed. Found ${res.detectedCount} suggested FAQs.`)
      loadAll()
      setActiveTab('gaps')
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setScanningGaps(false)
    }
  }

  async function handleApproveGap(gap: KnowledgeGap) {
    setProcessingGapId(gap.id)
    setError(null)
    try {
      const ans = editingGapId === gap.id ? gapCustomAnswer : undefined
      await api.approveKnowledgeGap(gap.id, ans)
      setSuccess(`Approved "${gap.question}" and added to Knowledge Base!`)
      setEditingGapId(null)
      setGapCustomAnswer('')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setProcessingGapId(null)
    }
  }

  async function handleDismissGap(id: string) {
    setProcessingGapId(id)
    setError(null)
    try {
      await api.dismissKnowledgeGap(id)
      setGaps((prev) => prev?.filter((g) => g.id !== id) ?? null)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setProcessingGapId(null)
    }
  }

  async function removeEntry(id: string) {
    try {
      await api.deleteKnowledgeBaseEntry(id)
      setEntries((prev) => prev?.filter((e) => e.id !== id) ?? null)
      setSuccess('Knowledge article deleted.')
    } catch (e) {
      setError((e as Error).message)
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-slate-900 flex items-center gap-2">
            <span>Knowledge Base & Omni-Source Learning</span>
            <span className="rounded-full bg-blue-100 text-blue-700 px-2 py-0.5 text-xs font-semibold">
              AI Grounded
            </span>
          </h1>
          <p className="mt-1 text-sm text-slate-500">
            Omni-source knowledge ingestion from help center URLs, uploaded PDFs, policies, and
            auto-detected gap-filling FAQs.
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-wrap items-center gap-2">
          <button
            onClick={() => {
              setShowNoteForm(true)
              setShowUrlModal(false)
            }}
            className="inline-flex items-center gap-1.5 rounded-md bg-white border border-slate-300 px-3 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs"
          >
            <span>+ Add Note</span>
          </button>

          <button
            onClick={() => {
              setShowUrlModal(true)
              setShowNoteForm(false)
            }}
            className="inline-flex items-center gap-1.5 rounded-md bg-blue-50 border border-blue-200 px-3 py-1.5 text-xs font-semibold text-blue-700 hover:bg-blue-100 shadow-xs"
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
            className="inline-flex items-center gap-1.5 rounded-md bg-indigo-50 border border-indigo-200 px-3 py-1.5 text-xs font-semibold text-indigo-700 hover:bg-indigo-100 shadow-xs disabled:opacity-50"
          >
            <span>📄 {uploadingDoc ? 'Parsing...' : 'Upload PDF/Doc'}</span>
          </button>

          <button
            onClick={handleScanGaps}
            disabled={scanningGaps}
            className="inline-flex items-center gap-1.5 rounded-md bg-slate-900 px-3 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs disabled:opacity-50"
          >
            <span>✨ {scanningGaps ? 'Detecting Gaps...' : 'Detect Knowledge Gaps'}</span>
          </button>
        </div>
      </div>

      {/* Notifications */}
      {error && (
        <div className="flex items-center justify-between rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="font-bold hover:text-red-900">
            ×
          </button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="font-bold hover:text-emerald-900">
            ×
          </button>
        </div>
      )}

      {/* Tab Navigation */}
      <div className="flex border-b border-slate-200">
        <button
          onClick={() => setActiveTab('articles')}
          className={`pb-3 text-sm font-semibold transition-colors flex items-center gap-2 border-b-2 mr-6 ${
            activeTab === 'articles'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span>Articles & Policies</span>
          <span className="rounded-full bg-slate-100 text-slate-700 px-2 py-0.5 text-xs font-semibold">
            {entries?.length ?? 0}
          </span>
        </button>

        <button
          onClick={() => setActiveTab('gaps')}
          className={`pb-3 text-sm font-semibold transition-colors flex items-center gap-2 border-b-2 ${
            activeTab === 'gaps'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span>✨ Suggested FAQs (Fills Gaps)</span>
          {(gaps?.length ?? 0) > 0 && (
            <span className="rounded-full bg-amber-100 text-amber-800 px-2 py-0.5 text-xs font-bold animate-pulse">
              {gaps?.length}
            </span>
          )}
        </button>
      </div>

      {/* URL Ingestion Modal */}
      {showUrlModal && (
        <div className="rounded-xl border border-blue-200 bg-blue-50/60 p-5 shadow-xs">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <span className="text-xl">🌐</span>
              <h2 className="text-sm font-bold text-slate-900">Ingest Knowledge from URL ("Learns")</h2>
            </div>
            <button
              onClick={() => setShowUrlModal(false)}
              className="text-slate-400 hover:text-slate-700 text-lg font-bold"
            >
              ×
            </button>
          </div>
          <p className="text-xs text-slate-600 mb-3">
            Provide any external help center link (e.g.{' '}
            <code className="bg-white px-1.5 py-0.5 rounded border border-blue-200 text-blue-800">
              https://helpcenter.com/refunds
            </code>
            ) or documentation page. The crawler extracts the clean article content and grounds the
            AI agent.
          </p>
          <form onSubmit={submitUrl} className="flex gap-2">
            <input
              required
              type="url"
              placeholder="https://helpcenter.com/faq"
              value={urlInput}
              onChange={(e) => setUrlInput(e.target.value)}
              className="flex-1 rounded-md border border-slate-300 bg-white px-3 py-2 text-sm shadow-xs focus:border-blue-500 focus:outline-none"
            />
            <button
              type="submit"
              disabled={ingestingUrl}
              className="rounded-md bg-blue-600 px-4 py-2 text-xs font-semibold text-white hover:bg-blue-700 disabled:opacity-50 shadow-xs"
            >
              {ingestingUrl ? 'Crawling...' : 'Crawl & Ingest'}
            </button>
          </form>
        </div>
      )}

      {/* Manual Note Form */}
      {showNoteForm && (
        <form
          onSubmit={submitNote}
          className="space-y-3 rounded-xl border border-slate-200 bg-white p-5 shadow-xs"
        >
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-bold text-slate-900">Add Manual Knowledge Note</h2>
            <button
              type="button"
              onClick={() => setShowNoteForm(false)}
              className="text-slate-400 hover:text-slate-700 text-lg font-bold"
            >
              ×
            </button>
          </div>
          <div>
            <label className="block text-xs font-medium text-slate-600">Title</label>
            <input
              required
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g. Return and Cancellation Policy"
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-slate-600">Content / Guidelines</label>
            <textarea
              required
              value={content}
              onChange={(e) => setContent(e.target.value)}
              rows={4}
              placeholder="Detailed business policy that the AI agent should cite..."
              className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
            />
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={() => setShowNoteForm(false)}
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={savingNote}
              className="rounded-md bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white disabled:opacity-50"
            >
              {savingNote ? 'Saving...' : 'Save Knowledge'}
            </button>
          </div>
        </form>
      )}

      {/* Tab 1: Articles & Policies */}
      {activeTab === 'articles' && (
        <div className="space-y-4">
          {entries === null ? (
            <div className="py-12 text-center text-sm text-slate-500">Loading knowledge base...</div>
          ) : entries.length === 0 ? (
            <div className="rounded-xl border border-dashed border-slate-300 bg-white p-12 text-center">
              <span className="text-3xl">📚</span>
              <h3 className="mt-2 text-sm font-semibold text-slate-900">No knowledge entries yet</h3>
              <p className="mt-1 text-xs text-slate-500 max-w-md mx-auto">
                Ingest your external help center URLs, upload your policy PDFs (like refund-policy.pdf),
                or add manual notes so the AI agent answers questions accurately.
              </p>
              <div className="mt-4 flex justify-center gap-2">
                <button
                  onClick={() => setShowUrlModal(true)}
                  className="rounded-md bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-blue-700"
                >
                  🔗 Ingest help center URL
                </button>
                <button
                  onClick={() => fileInputRef.current?.click()}
                  className="rounded-md bg-slate-900 px-3 py-1.5 text-xs font-semibold text-white hover:bg-slate-800"
                >
                  📄 Upload PDF Policy
                </button>
              </div>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {entries.map((e) => (
                <div
                  key={e.id}
                  className="rounded-xl border border-slate-200 bg-white p-5 shadow-xs flex flex-col justify-between hover:border-slate-300 transition-colors"
                >
                  <div>
                    <div className="flex items-start justify-between gap-3">
                      <h3 className="font-semibold text-slate-900 text-sm">{e.title}</h3>
                      <button
                        onClick={() => removeEntry(e.id)}
                        className="text-slate-400 hover:text-rose-600 text-sm font-semibold"
                        title="Delete entry"
                      >
                        ×
                      </button>
                    </div>

                    {/* Source Badge */}
                    <div className="mt-2 flex items-center gap-2">
                      {e.sourceType === 'url' && (
                        <span className="inline-flex items-center gap-1 rounded bg-blue-50 px-2 py-0.5 text-[11px] font-medium text-blue-700 border border-blue-200">
                          <span>🌐 URL</span>
                          {e.sourceUrl && (
                            <span className="truncate max-w-[180px] text-blue-600">
                              · {new URL(e.sourceUrl).hostname}
                            </span>
                          )}
                        </span>
                      )}
                      {e.sourceType === 'document' && (
                        <span className="inline-flex items-center gap-1 rounded bg-indigo-50 px-2 py-0.5 text-[11px] font-medium text-indigo-700 border border-indigo-200">
                          <span>📄 Document</span>
                          {e.fileName && (
                            <span className="truncate max-w-[180px] text-indigo-600">
                              · {e.fileName}
                            </span>
                          )}
                        </span>
                      )}
                      {e.sourceType === 'suggested_faq' && (
                        <span className="inline-flex items-center gap-1 rounded bg-amber-50 px-2 py-0.5 text-[11px] font-semibold text-amber-800 border border-amber-200">
                          <span>✨ Suggested FAQ</span>
                        </span>
                      )}
                      {e.sourceType === 'note' && (
                        <span className="inline-flex items-center gap-1 rounded bg-slate-100 px-2 py-0.5 text-[11px] font-medium text-slate-600">
                          <span>📝 Note</span>
                        </span>
                      )}
                    </div>

                    <p className="mt-3 text-xs text-slate-600 whitespace-pre-wrap line-clamp-4 leading-relaxed">
                      {e.content}
                    </p>
                  </div>

                  <div className="mt-4 pt-3 border-t border-slate-100 flex items-center justify-between text-[11px] text-slate-400">
                    <span>Added {new Date(e.createdAt).toLocaleDateString()}</span>
                    {e.sourceUrl && (
                      <a
                        href={e.sourceUrl}
                        target="_blank"
                        rel="noreferrer"
                        className="text-blue-600 hover:underline"
                      >
                        Visit source ↗
                      </a>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Tab 2: ✨ Suggested FAQs ("Fills Gaps") */}
      {activeTab === 'gaps' && (
        <div className="space-y-4">
          <div className="rounded-xl border border-amber-200 bg-amber-50/70 p-4 text-xs text-amber-900 flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <span className="text-xl">✨</span>
              <div>
                <span className="font-bold">Autonomous Knowledge Gap Detection ("Fills gaps"): </span>
                <span>
                  The AI agent analyzes inbound customer questions and low-confidence drafts to
                  propose high-impact FAQs. Click "Approve & Add" to incorporate them into grounded
                  agent memory.
                </span>
              </div>
            </div>
            <button
              onClick={handleScanGaps}
              disabled={scanningGaps}
              className="shrink-0 rounded-md bg-amber-800 text-white px-3 py-1.5 font-semibold hover:bg-amber-900 disabled:opacity-50"
            >
              {scanningGaps ? 'Scanning...' : 'Scan Now'}
            </button>
          </div>

          {gaps === null ? (
            <div className="py-12 text-center text-sm text-slate-500">Checking for knowledge gaps...</div>
          ) : gaps.length === 0 ? (
            <div className="rounded-xl border border-dashed border-slate-300 bg-white p-12 text-center">
              <span className="text-3xl">🎉</span>
              <h3 className="mt-2 text-sm font-semibold text-slate-900">No unanswered gaps detected</h3>
              <p className="mt-1 text-xs text-slate-500 max-w-md mx-auto">
                Your knowledge base currently covers all frequent customer questions. Click "Scan Now"
                to re-evaluate recent conversations.
              </p>
              <button
                onClick={handleScanGaps}
                disabled={scanningGaps}
                className="mt-4 rounded-md bg-slate-900 px-3 py-1.5 text-xs font-semibold text-white hover:bg-slate-800"
              >
                Scan Recent Conversations
              </button>
            </div>
          ) : (
            <div className="space-y-3">
              {gaps.map((g) => (
                <div
                  key={g.id}
                  className="rounded-xl border border-amber-200/80 bg-white p-5 shadow-xs hover:border-amber-300 transition-colors"
                >
                  <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <span className="rounded bg-amber-100 px-2 py-0.5 text-xs font-bold text-amber-800">
                        ✨ Suggested FAQ
                      </span>
                      <span className="rounded bg-slate-100 px-2 py-0.5 text-xs font-semibold text-slate-700">
                        🔥 Asked {g.frequency} {g.frequency === 1 ? 'time' : 'times'}
                      </span>
                    </div>

                    <div className="flex items-center gap-2">
                      {editingGapId !== g.id ? (
                        <button
                          onClick={() => {
                            setEditingGapId(g.id)
                            setGapCustomAnswer(g.suggestedAnswer)
                          }}
                          className="rounded-md border border-slate-300 px-2.5 py-1 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                        >
                          Edit Answer
                        </button>
                      ) : (
                        <button
                          onClick={() => setEditingGapId(null)}
                          className="rounded-md border border-slate-300 px-2.5 py-1 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                        >
                          Cancel Edit
                        </button>
                      )}

                      <button
                        onClick={() => handleDismissGap(g.id)}
                        disabled={processingGapId === g.id}
                        className="rounded-md border border-slate-200 px-2.5 py-1 text-xs font-semibold text-slate-600 hover:bg-slate-50 hover:text-rose-600 disabled:opacity-50"
                      >
                        Dismiss
                      </button>

                      <button
                        onClick={() => handleApproveGap(g)}
                        disabled={processingGapId === g.id}
                        className="rounded-md bg-emerald-600 px-3.5 py-1 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50 shadow-xs"
                      >
                        {processingGapId === g.id ? 'Approving...' : '✓ Approve & Add to KB'}
                      </button>
                    </div>
                  </div>

                  <h3 className="mt-3 text-sm font-bold text-slate-900">{g.question}</h3>

                  {g.sourceQuery && (
                    <div className="mt-2 text-xs text-slate-500 italic bg-slate-50 rounded-md p-2.5 border border-slate-100">
                      Customer Inquiry: "{g.sourceQuery}"
                    </div>
                  )}

                  <div className="mt-3">
                    <label className="block text-[11px] font-semibold text-slate-500 uppercase tracking-wider mb-1">
                      Grounded Answer
                    </label>
                    {editingGapId === g.id ? (
                      <textarea
                        value={gapCustomAnswer}
                        onChange={(e) => setGapCustomAnswer(e.target.value)}
                        rows={3}
                        className="w-full rounded-md border border-amber-300 p-2.5 text-xs text-slate-800 focus:outline-none focus:ring-1 focus:ring-amber-500"
                      />
                    ) : (
                      <p className="text-xs text-slate-700 bg-amber-50/40 p-2.5 rounded-md border border-amber-100">
                        {g.suggestedAnswer}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
