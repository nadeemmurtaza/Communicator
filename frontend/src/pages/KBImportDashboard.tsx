import { useState, useEffect } from 'react'
import {
  getKBImportSummary, uploadDocument, listDocuments, deleteDocument,
  addURLSource, crawlURL, listURLSources, deleteURLSource,
  getKBImportSettings, updateKBImportSettings,
  type KBDocument, type KBURLSource, type KBImportSettings, type KBImportSummary
} from '../lib/api'
import { FileText, Globe, Settings, Upload, Trash2, Play, Plus, Search, AlertCircle, CheckCircle, Clock } from '../components/icons'

type Tab = 'overview' | 'documents' | 'urls' | 'settings'

export default function KBImportDashboard() {
  const [tab, setTab] = useState<Tab>('overview')
  const [summary, setSummary] = useState<KBImportSummary | null>(null)
  const [documents, setDocuments] = useState<KBDocument[]>([])
  const [urlSources, setUrlSources] = useState<KBURLSource[]>([])
  const [settings, setSettings] = useState<KBImportSettings | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [docStatusFilter, setDocStatusFilter] = useState('')
  const [urlStatusFilter, setUrlStatusFilter] = useState('')

  // Upload state
  const [uploading, setUploading] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)

  // URL state
  const [newUrl, setNewUrl] = useState('')
  const [crawlInterval, setCrawlInterval] = useState(168)

  // Settings state
  const [settingsForm, setSettingsForm] = useState({
    chunkSize: 2000,
    chunkOverlap: 200,
    maxFileSizeMb: 10,
    autoCreateKbEntries: true,
    crawlIntervalHours: 168,
  })

  async function loadSummary() {
    try {
      const data = await getKBImportSummary()
      setSummary(data)
    } catch (e: any) {
      setError(e.message || 'Failed to load summary')
    }
  }

  async function loadDocuments() {
    try {
      const data = await listDocuments(undefined, docStatusFilter || undefined)
      setDocuments(data)
    } catch (e: any) {
      setError(e.message || 'Failed to load documents')
    }
  }

  async function loadURLSources() {
    try {
      const data = await listURLSources(undefined, urlStatusFilter || undefined)
      setUrlSources(data)
    } catch (e: any) {
      setError(e.message || 'Failed to load URL sources')
    }
  }

  async function loadSettings() {
    try {
      const data = await getKBImportSettings()
      setSettings(data)
      setSettingsForm({
        chunkSize: data.chunkSize,
        chunkOverlap: data.chunkOverlap,
        maxFileSizeMb: data.maxFileSizeMb,
        autoCreateKbEntries: data.autoCreateKbEntries,
        crawlIntervalHours: data.crawlIntervalHours,
      })
    } catch (e: any) {
      setError(e.message || 'Failed to load settings')
    }
  }

  useEffect(() => {
    loadSummary()
    loadDocuments()
    loadURLSources()
    loadSettings()
  }, [])

  useEffect(() => { loadDocuments() }, [docStatusFilter])
  useEffect(() => { loadURLSources() }, [urlStatusFilter])

  async function handleFileUpload() {
    if (!selectedFile) return
    setUploading(true)
    setError(null)

    try {
      const reader = new FileReader()
      reader.onload = async () => {
        const base64 = reader.result?.toString().split(',')[1] || ''
        const ext = selectedFile.name.split('.').pop()?.toLowerCase() || 'txt'
        const result = await uploadDocument({
          fileName: selectedFile.name,
          fileType: ext,
          fileData: base64,
        })

        if (result.status === 'duplicate') {
          setError(`Duplicate file: ${result.errorMessage}`)
        } else {
          setError(null)
          loadSummary()
          loadDocuments()
          setSelectedFile(null)
        }
        setUploading(false)
      }
      reader.readAsDataURL(selectedFile)
    } catch (e: any) {
      setError(e.message || 'Upload failed')
      setUploading(false)
    }
  }

  async function handleAddURL() {
    if (!newUrl.trim()) return
    setLoading(true)
    setError(null)

    try {
      await addURLSource({ url: newUrl.trim(), crawlIntervalHours: crawlInterval })
      setNewUrl('')
      loadSummary()
      loadURLSources()
    } catch (e: any) {
      setError(e.message || 'Failed to add URL')
    }
    setLoading(false)
  }

  async function handleCrawl(id: string) {
    setLoading(true)
    setError(null)
    try {
      await crawlURL(id)
      loadSummary()
      loadURLSources()
    } catch (e: any) {
      setError(e.message || 'Crawl failed')
    }
    setLoading(false)
  }

  async function handleDeleteDocument(id: string) {
    try {
      await deleteDocument(id)
      loadSummary()
      loadDocuments()
    } catch (e: any) {
      setError(e.message || 'Delete failed')
    }
  }

  async function handleDeleteURL(id: string) {
    try {
      await deleteURLSource(id)
      loadSummary()
      loadURLSources()
    } catch (e: any) {
      setError(e.message || 'Delete failed')
    }
  }

  async function handleSaveSettings() {
    setLoading(true)
    setError(null)
    try {
      await updateKBImportSettings({ settings: settingsForm })
      loadSettings()
    } catch (e: any) {
      setError(e.message || 'Failed to save settings')
    }
    setLoading(false)
  }

  function formatFileSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  function statusColor(status: string) {
    switch (status) {
      case 'ready': case 'completed': return '#22c55e'
      case 'processing': case 'crawling': case 'running': case 'pending': return '#f59e0b'
      case 'error': case 'failed': return '#ef4444'
      default: return '#6b7280'
    }
  }

  function statusIcon(status: string) {
    switch (status) {
      case 'ready': case 'completed': return <CheckCircle size={16} />
      case 'processing': case 'crawling': case 'running': case 'pending': return <Clock size={16} />
      case 'error': case 'failed': return <AlertCircle size={16} />
      default: return <Clock size={16} />
    }
  }

  return (
    <div style={{ padding: 24, maxWidth: 1200, margin: '0 auto' }}>
      <h2 style={{ marginBottom: 24, fontSize: 24, fontWeight: 700 }}>
        KB Import — Documents & URL Crawling
      </h2>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 24, borderBottom: '1px solid #e5e7eb', paddingBottom: 12 }}>
        {([
          { key: 'overview', label: 'Overview', icon: <FileText size={16} /> },
          { key: 'documents', label: 'Documents', icon: <Upload size={16} /> },
          { key: 'urls', label: 'URL Sources', icon: <Globe size={16} /> },
          { key: 'settings', label: 'Settings', icon: <Settings size={16} /> },
        ] as { key: Tab; label: string; icon: React.ReactNode }[]).map((t) => (
          <button
            key={t.key}
            onClick={() => setTab(t.key)}
            style={{
              display: 'flex', alignItems: 'center', gap: 6, padding: '8px 16px',
              border: 'none', borderRadius: 8, cursor: 'pointer', fontSize: 14, fontWeight: 500,
              background: tab === t.key ? '#ede9fe' : 'transparent',
              color: tab === t.key ? '#7c3aed' : '#6b7280',
            }}
          >
            {t.icon} {t.label}
          </button>
        ))}
      </div>

      {error && (
        <div style={{ padding: '12px 16px', background: '#fef2f2', border: '1px solid #fecaca', borderRadius: 8, marginBottom: 16, color: '#dc2626', display: 'flex', alignItems: 'center', gap: 8 }}>
          <AlertCircle size={16} /> {error}
          <button onClick={() => setError(null)} style={{ marginLeft: 'auto', background: 'none', border: 'none', cursor: 'pointer', color: '#dc2626' }}>✕</button>
        </div>
      )}

      {/* Overview Tab */}
      {tab === 'overview' && summary && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 16, marginBottom: 24 }}>
            {[
              { label: 'Total Documents', value: summary.totalDocuments, color: '#7c3aed' },
              { label: 'Ready Documents', value: summary.readyDocuments, color: '#22c55e' },
              { label: 'Processing', value: summary.processingDocuments, color: '#f59e0b' },
              { label: 'Errors', value: summary.errorDocuments, color: '#ef4444' },
              { label: 'URL Sources', value: summary.totalUrlSources, color: '#3b82f6' },
              { label: 'Ready URLs', value: summary.readyUrlSources, color: '#22c55e' },
              { label: 'Total Chunks', value: summary.totalChunks, color: '#8b5cf6' },
              { label: 'KB Entries Created', value: summary.totalKbEntriesCreated, color: '#06b6d4' },
            ].map((card) => (
              <div key={card.label} style={{ padding: 20, background: '#f9fafb', borderRadius: 12, border: '1px solid #e5e7eb' }}>
                <div style={{ fontSize: 12, color: '#6b7280', marginBottom: 4 }}>{card.label}</div>
                <div style={{ fontSize: 28, fontWeight: 700, color: card.color }}>{card.value}</div>
              </div>
            ))}
          </div>

          {/* Recent Uploads */}
          {summary.recentUploads.length > 0 && (
            <div style={{ marginBottom: 24 }}>
              <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Recent Uploads</h3>
              <div style={{ border: '1px solid #e5e7eb', borderRadius: 12, overflow: 'hidden' }}>
                {summary.recentUploads.map((doc) => (
                  <div key={doc.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid #f3f4f6' }}>
                    <FileText size={20} style={{ color: '#7c3aed' }} />
                    <div style={{ flex: 1 }}>
                      <div style={{ fontWeight: 500 }}>{doc.fileName}</div>
                      <div style={{ fontSize: 12, color: '#6b7280' }}>{doc.fileType.toUpperCase()} · {formatFileSize(doc.fileSize)}</div>
                    </div>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 4, color: statusColor(doc.status), fontSize: 13 }}>
                      {statusIcon(doc.status)} {doc.status}
                    </span>
                    {doc.chunksCount && <span style={{ fontSize: 12, color: '#6b7280' }}>{doc.chunksCount} chunks</span>}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Crawls */}
          {summary.recentCrawls.length > 0 && (
            <div>
              <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 12 }}>Recent Crawls</h3>
              <div style={{ border: '1px solid #e5e7eb', borderRadius: 12, overflow: 'hidden' }}>
                {summary.recentCrawls.map((log) => (
                  <div key={log.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid #f3f4f6' }}>
                    <Globe size={20} style={{ color: '#3b82f6' }} />
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: 13, color: '#6b7280' }}>Crawl #{log.id.slice(0, 8)}</div>
                      <div style={{ fontSize: 12, color: '#6b7280' }}>{log.startedAt}</div>
                    </div>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 4, color: statusColor(log.status), fontSize: 13 }}>
                      {statusIcon(log.status)} {log.status}
                    </span>
                    <span style={{ fontSize: 12, color: '#6b7280' }}>{log.chunksCreated} chunks</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Documents Tab */}
      {tab === 'documents' && (
        <div>
          {/* Upload Area */}
          <div style={{ padding: 24, border: '2px dashed #d1d5db', borderRadius: 12, marginBottom: 24, textAlign: 'center' }}>
            <Upload size={32} style={{ color: '#9ca3af', marginBottom: 8 }} />
            <div style={{ fontSize: 14, color: '#6b7280', marginBottom: 12 }}>
              Upload PDF, DOCX, TXT, HTML, or MD files
            </div>
            <input
              type="file"
              accept=".pdf,.docx,.txt,.html,.md"
              onChange={(e) => setSelectedFile(e.target.files?.[0] || null)}
              style={{ marginBottom: 12 }}
            />
            {selectedFile && (
              <div style={{ fontSize: 13, color: '#7c3aed', marginBottom: 8 }}>
                Selected: {selectedFile.name} ({formatFileSize(selectedFile.size)})
              </div>
            )}
            <button
              onClick={handleFileUpload}
              disabled={!selectedFile || uploading}
              style={{
                padding: '8px 20px', background: '#7c3aed', color: '#fff', border: 'none',
                borderRadius: 8, cursor: selectedFile ? 'pointer' : 'not-allowed', opacity: selectedFile ? 1 : 0.5,
                fontSize: 14, fontWeight: 500,
              }}
            >
              {uploading ? 'Processing...' : 'Upload & Process'}
            </button>
          </div>

          {/* Filter */}
          <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
            <select
              value={docStatusFilter}
              onChange={(e) => setDocStatusFilter(e.target.value)}
              style={{ padding: '6px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 13 }}
            >
              <option value="">All Statuses</option>
              <option value="ready">Ready</option>
              <option value="processing">Processing</option>
              <option value="error">Error</option>
            </select>
          </div>

          {/* Document List */}
          <div style={{ border: '1px solid #e5e7eb', borderRadius: 12, overflow: 'hidden' }}>
            {documents.length === 0 && (
              <div style={{ padding: 24, textAlign: 'center', color: '#6b7280' }}>No documents found</div>
            )}
            {documents.map((doc) => (
              <div key={doc.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid #f3f4f6' }}>
                <FileText size={20} style={{ color: '#7c3aed' }} />
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 500 }}>{doc.fileName}</div>
                  <div style={{ fontSize: 12, color: '#6b7280' }}>
                    {doc.fileType.toUpperCase()} · {formatFileSize(doc.fileSize)}
                    {doc.chunksCount && ` · ${doc.chunksCount} chunks`}
                  </div>
                  {doc.errorMessage && <div style={{ fontSize: 12, color: '#ef4444' }}>{doc.errorMessage}</div>}
                </div>
                <span style={{ display: 'flex', alignItems: 'center', gap: 4, color: statusColor(doc.status), fontSize: 13 }}>
                  {statusIcon(doc.status)} {doc.status}
                </span>
                <button
                  onClick={() => handleDeleteDocument(doc.id)}
                  style={{ background: 'none', border: 'none', cursor: 'pointer', color: '#ef4444', padding: 4 }}
                  title="Delete"
                >
                  <Trash2 size={16} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* URL Sources Tab */}
      {tab === 'urls' && (
        <div>
          {/* Add URL */}
          <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
            <input
              type="url"
              placeholder="https://example.com/page"
              value={newUrl}
              onChange={(e) => setNewUrl(e.target.value)}
              style={{ flex: 1, padding: '8px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 14 }}
            />
            <input
              type="number"
              value={crawlInterval}
              onChange={(e) => setCrawlInterval(Number(e.target.value))}
              style={{ width: 80, padding: '8px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 14 }}
              title="Crawl interval (hours)"
            />
            <span style={{ display: 'flex', alignItems: 'center', fontSize: 12, color: '#6b7280' }}>hours</span>
            <button
              onClick={handleAddURL}
              disabled={!newUrl.trim() || loading}
              style={{
                padding: '8px 16px', background: '#3b82f6', color: '#fff', border: 'none',
                borderRadius: 8, cursor: newUrl.trim() ? 'pointer' : 'not-allowed', opacity: newUrl.trim() ? 1 : 0.5,
                fontSize: 14, fontWeight: 500, display: 'flex', alignItems: 'center', gap: 4,
              }}
            >
              <Plus size={16} /> Add URL
            </button>
          </div>

          {/* Filter */}
          <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
            <select
              value={urlStatusFilter}
              onChange={(e) => setUrlStatusFilter(e.target.value)}
              style={{ padding: '6px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 13 }}
            >
              <option value="">All Statuses</option>
              <option value="ready">Ready</option>
              <option value="pending">Pending</option>
              <option value="crawling">Crawling</option>
              <option value="error">Error</option>
            </select>
          </div>

          {/* URL List */}
          <div style={{ border: '1px solid #e5e7eb', borderRadius: 12, overflow: 'hidden' }}>
            {urlSources.length === 0 && (
              <div style={{ padding: 24, textAlign: 'center', color: '#6b7280' }}>No URL sources found</div>
            )}
            {urlSources.map((source) => (
              <div key={source.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '12px 16px', borderBottom: '1px solid #f3f4f6' }}>
                <Globe size={20} style={{ color: '#3b82f6' }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {source.title || source.url}
                  </div>
                  <div style={{ fontSize: 12, color: '#6b7280', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {source.url}
                  </div>
                  {source.errorMessage && <div style={{ fontSize: 12, color: '#ef4444' }}>{source.errorMessage}</div>}
                  <div style={{ fontSize: 11, color: '#9ca3af' }}>
                    {source.chunksCount && `${source.chunksCount} chunks`}
                    {source.lastCrawledAt && ` · Last crawled: ${source.lastCrawledAt}`}
                    {source.nextCrawlAt && ` · Next: ${source.nextCrawlAt}`}
                  </div>
                </div>
                <span style={{ display: 'flex', alignItems: 'center', gap: 4, color: statusColor(source.status), fontSize: 13 }}>
                  {statusIcon(source.status)} {source.status}
                </span>
                <button
                  onClick={() => handleCrawl(source.id)}
                  disabled={loading}
                  style={{ background: 'none', border: 'none', cursor: 'pointer', color: '#22c55e', padding: 4 }}
                  title="Crawl now"
                >
                  <Play size={16} />
                </button>
                <button
                  onClick={() => handleDeleteURL(source.id)}
                  style={{ background: 'none', border: 'none', cursor: 'pointer', color: '#ef4444', padding: 4 }}
                  title="Delete"
                >
                  <Trash2 size={16} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {tab === 'settings' && settings && (
        <div style={{ maxWidth: 600 }}>
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 16 }}>Import Settings</h3>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
              <label style={{ fontSize: 13, fontWeight: 500, color: '#374151', display: 'block', marginBottom: 4 }}>
                Chunk Size (characters)
              </label>
              <input
                type="number"
                value={settingsForm.chunkSize}
                onChange={(e) => setSettingsForm({ ...settingsForm, chunkSize: Number(e.target.value) })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 14 }}
              />
            </div>

            <div>
              <label style={{ fontSize: 13, fontWeight: 500, color: '#374151', display: 'block', marginBottom: 4 }}>
                Chunk Overlap (characters)
              </label>
              <input
                type="number"
                value={settingsForm.chunkOverlap}
                onChange={(e) => setSettingsForm({ ...settingsForm, chunkOverlap: Number(e.target.value) })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 14 }}
              />
            </div>

            <div>
              <label style={{ fontSize: 13, fontWeight: 500, color: '#374151', display: 'block', marginBottom: 4 }}>
                Max File Size (MB)
              </label>
              <input
                type="number"
                value={settingsForm.maxFileSizeMb}
                onChange={(e) => setSettingsForm({ ...settingsForm, maxFileSizeMb: Number(e.target.value) })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 14 }}
              />
            </div>

            <div>
              <label style={{ fontSize: 13, fontWeight: 500, color: '#374151', display: 'block', marginBottom: 4 }}>
                Default Crawl Interval (hours)
              </label>
              <input
                type="number"
                value={settingsForm.crawlIntervalHours}
                onChange={(e) => setSettingsForm({ ...settingsForm, crawlIntervalHours: Number(e.target.value) })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #d1d5db', borderRadius: 8, fontSize: 14 }}
              />
            </div>

            <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 14 }}>
              <input
                type="checkbox"
                checked={settingsForm.autoCreateKbEntries}
                onChange={(e) => setSettingsForm({ ...settingsForm, autoCreateKbEntries: e.target.checked })}
              />
              Auto-create KB entries from chunks
            </label>

            <div>
              <label style={{ fontSize: 13, fontWeight: 500, color: '#374151', display: 'block', marginBottom: 4 }}>
                Allowed File Types
              </label>
              <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                {settings.allowedFileTypes.map((type) => (
                  <span key={type} style={{ padding: '4px 12px', background: '#ede9fe', color: '#7c3aed', borderRadius: 16, fontSize: 12, fontWeight: 500 }}>
                    .{type}
                  </span>
                ))}
              </div>
            </div>

            <button
              onClick={handleSaveSettings}
              disabled={loading}
              style={{
                padding: '10px 20px', background: '#7c3aed', color: '#fff', border: 'none',
                borderRadius: 8, cursor: 'pointer', fontSize: 14, fontWeight: 500, width: 'fit-content',
              }}
            >
              {loading ? 'Saving...' : 'Save Settings'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
