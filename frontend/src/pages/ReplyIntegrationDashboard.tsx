import { useEffect, useState } from 'react'
import {
  getReplyIntegrationSummary,
  getReplyIntegrationSettings,
  updateReplyIntegrationSettings,
  buildReplyContext,
  getKnowledgeSources,
  ReplyIntegrationSummary,
  ReplyIntegrationSettings,
  ReplyContextPackage,
} from '../lib/api'
import {
  Database,
  BookOpen,
  FileText,
  Users,
  Settings,
  Sparkles,
  Play,
  Eye,
  CheckCircle,
  XCircle,
  Layers,
} from '../components/icons'

export default function ReplyIntegrationDashboard() {
  const [tab, setTab] = useState<'overview' | 'settings' | 'test' | 'sources'>('overview')
  const [summary, setSummary] = useState<ReplyIntegrationSummary | null>(null)
  const [settings, setSettings] = useState<ReplyIntegrationSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [testResult, setTestResult] = useState<ReplyContextPackage | null>(null)
  const [testLoading, setTestLoading] = useState(false)
  const [sources, setSources] = useState<any[]>([])
  const [sourcesTotal, setSourcesTotal] = useState(0)
  const [sourcesLoading, setSourcesLoading] = useState(false)

  async function load() {
    setLoading(true)
    try {
      const [s, st] = await Promise.all([getReplyIntegrationSummary(), getReplyIntegrationSettings()])
      setSummary(s)
      setSettings(st)
    } catch (e) {
      console.error('Failed to load reply integration data:', e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  async function handleSettingsUpdate(e: React.FormEvent) {
    e.preventDefault()
    if (!settings) return
    try {
      const updated = await updateReplyIntegrationSettings(settings)
      setSettings(updated)
      load()
    } catch (e) {
      console.error('Failed to update settings:', e)
    }
  }

  async function handleTestBuild(e: React.FormEvent) {
    e.preventDefault()
    const form = e.target as HTMLFormElement
    const fd = new FormData(form)
    const lastInbound = (fd.get('last_inbound') as string) || ''
    if (!lastInbound.trim()) return
    setTestLoading(true)
    try {
      const result = await buildReplyContext({
        last_inbound: lastInbound,
        tenant_key: 'default',
      })
      setTestResult(result)
    } catch (e) {
      console.error('Failed to build context:', e)
    } finally {
      setTestLoading(false)
    }
  }

  async function loadSources() {
    setSourcesLoading(true)
    try {
      const result = await getKnowledgeSources()
      setSources(result.sources)
      setSourcesTotal(result.total)
    } catch (e) {
      console.error('Failed to load sources:', e)
    } finally {
      setSourcesLoading(false)
    }
  }

  if (loading) return <div className="p-6 text-gray-500">Loading...</div>
  if (!summary || !settings) return <div className="p-6 text-red-500">Failed to load data</div>

  const tabs = [
    { id: 'overview' as const, label: 'Overview', icon: Layers },
    { id: 'settings' as const, label: 'Settings', icon: Settings },
    { id: 'test' as const, label: 'Test Build', icon: Play },
    { id: 'sources' as const, label: 'Knowledge Sources', icon: Database },
  ]

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Reply Engine Integration</h1>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-100 rounded-lg p-1">
        {tabs.map(t => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition ${
              tab === t.id ? 'bg-white shadow text-purple-700' : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <t.icon className="w-4 h-4" />
            {t.label}
          </button>
        ))}
      </div>

      {/* Overview */}
      {tab === 'overview' && (
        <div className="space-y-6">
          {/* Status Cards */}
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            <StatusCard
              icon={Sparkles}
              label="Tone of Voice"
              value={summary.tone_of_voice_enabled ? 'Active' : 'Off'}
              good={summary.tone_of_voice_enabled}
            />
            <StatusCard
              icon={BookOpen}
              label="Guidance Rules"
              value={`${summary.guidance_rules_active}`}
              good={summary.guidance_rules_active > 0}
            />
            <StatusCard
              icon={Users}
              label="Audience Scoping"
              value={summary.audience_scoping_enabled ? 'On' : 'Off'}
              good={summary.audience_scoping_enabled}
            />
            <StatusCard
              icon={FileText}
              label="Help Center"
              value={summary.help_center_enabled ? 'Linked' : 'Empty'}
              good={summary.help_center_enabled}
            />
            <StatusCard
              icon={Database}
              label="Doc Chunks"
              value={summary.document_chunks_enabled ? 'Linked' : 'Empty'}
              good={summary.document_chunks_enabled}
            />
          </div>

          {/* Tracked Sources */}
          <div className="bg-white rounded-xl border p-6">
            <h2 className="text-lg font-semibold mb-4">Knowledge Sources Tracked</h2>
            <div className="text-3xl font-bold mb-2">{summary.total_knowledge_sources_tracked}</div>
            <p className="text-sm text-gray-500 mb-4">Total sources used across all AI-generated replies</p>
            {summary.sources_by_type.length > 0 && (
              <div className="space-y-2">
                {summary.sources_by_type.map(s => (
                  <div key={s.source_type} className="flex items-center gap-3">
                    <span className="w-32 text-sm font-medium text-gray-700 capitalize">{s.source_type.replace(/_/g, ' ')}</span>
                    <div className="flex-1 bg-gray-100 rounded-full h-3">
                      <div
                        className="bg-purple-500 h-3 rounded-full"
                        style={{ width: `${Math.min(100, (s.count / summary.total_knowledge_sources_tracked) * 100)}%` }}
                      />
                    </div>
                    <span className="text-sm text-gray-600 w-12 text-right">{s.count}</span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* How It Works */}
          <div className="bg-gradient-to-r from-purple-50 to-indigo-50 rounded-xl border border-purple-200 p-6">
            <h2 className="text-lg font-semibold mb-3 text-purple-900">How Reply Integration Works</h2>
            <div className="grid md:grid-cols-4 gap-4 text-sm">
              <div className="bg-white rounded-lg p-4">
                <div className="font-semibold mb-1">1. Knowledge Base</div>
                <p className="text-gray-600">KB entries are fetched, with audience scoping if the contact belongs to a segment with linked KB entries.</p>
              </div>
              <div className="bg-white rounded-lg p-4">
                <div className="font-semibold mb-1">2. Help Center</div>
                <p className="text-gray-600">Published help center articles are added as additional knowledge context for the AI.</p>
              </div>
              <div className="bg-white rounded-lg p-4">
                <div className="font-semibold mb-1">3. Document Chunks</div>
                <p className="text-gray-600">Uploaded PDFs, DOCX files and crawled webpages are chunked and included as knowledge.</p>
              </div>
              <div className="bg-white rounded-lg p-4">
                <div className="font-semibold mb-1">4. Tone & Guidance</div>
                <p className="text-gray-600">Configured tone of voice and guidance rules are injected into the AI system prompt.</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Settings */}
      {tab === 'settings' && (
        <div className="bg-white rounded-xl border p-6 max-w-2xl">
          <h2 className="text-lg font-semibold mb-4">Integration Settings</h2>
          <form onSubmit={handleSettingsUpdate} className="space-y-4">
            <ToggleSetting
              label="Use Tone of Voice"
              description="Inject tone and style instructions into AI prompt"
              checked={settings.use_tone_of_voice}
              onChange={v => setSettings({ ...settings, use_tone_of_voice: v })}
            />
            <ToggleSetting
              label="Use Guidance Rules"
              description="Apply custom guidance rules to AI responses"
              checked={settings.use_guidance_rules}
              onChange={v => setSettings({ ...settings, use_guidance_rules: v })}
            />
            <ToggleSetting
              label="Use Audience Scoping"
              description="Scope KB entries based on contact's segment membership"
              checked={settings.use_audience_scoping}
              onChange={v => setSettings({ ...settings, use_audience_scoping: v })}
            />
            <ToggleSetting
              label="Use Help Center Articles"
              description="Include published help center articles as knowledge"
              checked={settings.use_help_center_articles}
              onChange={v => setSettings({ ...settings, use_help_center_articles: v })}
            />
            <ToggleSetting
              label="Use Document Chunks"
              description="Include uploaded document chunks as knowledge"
              checked={settings.use_document_chunks}
              onChange={v => setSettings({ ...settings, use_document_chunks: v })}
            />
            <ToggleSetting
              label="Track Knowledge Sources"
              description="Log which sources were used in each AI reply"
              checked={settings.track_knowledge_sources}
              onChange={v => setSettings({ ...settings, track_knowledge_sources: v })}
            />
            <div className="grid grid-cols-3 gap-4">
              <NumberSetting
                label="Max KB Entries"
                value={settings.max_knowledge_entries}
                onChange={v => setSettings({ ...settings, max_knowledge_entries: v })}
              />
              <NumberSetting
                label="Max Doc Chunks"
                value={settings.max_document_chunks}
                onChange={v => setSettings({ ...settings, max_document_chunks: v })}
              />
              <NumberSetting
                label="Max HC Articles"
                value={settings.max_help_center_articles}
                onChange={v => setSettings({ ...settings, max_help_center_articles: v })}
              />
            </div>
            <button type="submit" className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition">
              Save Settings
            </button>
          </form>
        </div>
      )}

      {/* Test Build */}
      {tab === 'test' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border p-6 max-w-2xl">
            <h2 className="text-lg font-semibold mb-4">Test Context Build</h2>
            <p className="text-sm text-gray-500 mb-4">
              Enter a sample customer message to see what knowledge context the AI would receive.
            </p>
            <form onSubmit={handleTestBuild} className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Customer Message</label>
                <textarea
                  name="last_inbound"
                  className="w-full border rounded-lg p-3 text-sm"
                  rows={3}
                  placeholder="e.g. How do I process a refund for order #12345?"
                  required
                />
              </div>
              <button
                type="submit"
                disabled={testLoading}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition disabled:opacity-50"
              >
                {testLoading ? 'Building...' : 'Build Context'}
              </button>
            </form>
          </div>

          {testResult && (
            <div className="bg-white rounded-xl border p-6 space-y-4">
              <h3 className="text-lg font-semibold">Context Package Result</h3>
              <div className="flex gap-2">
                <span className={`px-2 py-1 rounded text-xs font-medium ${testResult.audience_scoped ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'}`}>
                  {testResult.audience_scoped ? 'Audience Scoped' : 'No Scoping'}
                </span>
                <span className="px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-700">
                  {testResult.sources_used.length} sources
                </span>
              </div>
              {testResult.tone_instructions && (
                <div>
                  <h4 className="text-sm font-semibold mb-1">Tone Instructions</h4>
                  <pre className="bg-gray-50 rounded p-3 text-xs whitespace-pre-wrap max-h-40 overflow-y-auto">{testResult.tone_instructions}</pre>
                </div>
              )}
              {testResult.guidance_instructions && (
                <div>
                  <h4 className="text-sm font-semibold mb-1">Guidance Rules</h4>
                  <pre className="bg-gray-50 rounded p-3 text-xs whitespace-pre-wrap max-h-40 overflow-y-auto">{testResult.guidance_instructions}</pre>
                </div>
              )}
              {testResult.sources_used.length > 0 && (
                <div>
                  <h4 className="text-sm font-semibold mb-1">Knowledge Sources</h4>
                  <div className="space-y-1">
                    {testResult.sources_used.map((s, i) => (
                      <div key={i} className="flex items-center gap-2 text-xs">
                        <span className="px-1.5 py-0.5 bg-purple-100 text-purple-700 rounded font-medium capitalize">
                          {s.source_type.replace(/_/g, ' ')}
                        </span>
                        <span className="text-gray-700">{s.source_title}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {testResult.knowledge_context && (
                <div>
                  <h4 className="text-sm font-semibold mb-1">Knowledge Context ({testResult.knowledge_context.length} chars)</h4>
                  <pre className="bg-gray-50 rounded p-3 text-xs whitespace-pre-wrap max-h-60 overflow-y-auto">{testResult.knowledge_context}</pre>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Sources */}
      {tab === 'sources' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Knowledge Source Usage Log</h2>
            <button
              onClick={loadSources}
              disabled={sourcesLoading}
              className="px-3 py-1.5 text-sm bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50"
            >
              {sourcesLoading ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          <p className="text-sm text-gray-500">Total tracked: {sourcesTotal}</p>
          {sources.length === 0 && !sourcesLoading && (
            <div className="bg-white rounded-xl border p-8 text-center text-gray-500">
              No knowledge sources tracked yet. Sources are logged when AI generates replies.
            </div>
          )}
          {sources.length > 0 && (
            <div className="bg-white rounded-xl border overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="text-left p-3 font-medium">Source Type</th>
                    <th className="text-left p-3 font-medium">Title</th>
                    <th className="text-left p-3 font-medium">Draft ID</th>
                    <th className="text-left p-3 font-medium">Used</th>
                    <th className="text-left p-3 font-medium">Time</th>
                  </tr>
                </thead>
                <tbody>
                  {sources.map((s: any) => (
                    <tr key={s.id} className="border-t hover:bg-gray-50">
                      <td className="p-3">
                        <span className="px-1.5 py-0.5 bg-purple-100 text-purple-700 rounded text-xs font-medium capitalize">
                          {s.source_type?.replace(/_/g, ' ')}
                        </span>
                      </td>
                      <td className="p-3 text-gray-700">{s.source_title}</td>
                      <td className="p-3 font-mono text-xs text-gray-500">{s.draft_id?.slice(0, 8)}...</td>
                      <td className="p-3">
                        {s.used_in_reply ? (
                          <CheckCircle className="w-4 h-4 text-green-500" />
                        ) : (
                          <XCircle className="w-4 h-4 text-red-500" />
                        )}
                      </td>
                      <td className="p-3 text-gray-500 text-xs">
                        {s.created_at ? new Date(s.created_at).toLocaleString() : '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function StatusCard({ icon: Icon, label, value, good }: { icon: any; label: string; value: string; good: boolean }) {
  return (
    <div className={`rounded-xl border p-4 ${good ? 'bg-green-50 border-green-200' : 'bg-gray-50 border-gray-200'}`}>
      <div className="flex items-center gap-2 mb-2">
        <Icon className={`w-4 h-4 ${good ? 'text-green-600' : 'text-gray-400'}`} />
        <span className="text-sm font-medium text-gray-700">{label}</span>
      </div>
      <div className={`text-xl font-bold ${good ? 'text-green-700' : 'text-gray-500'}`}>{value}</div>
    </div>
  )
}

function ToggleSetting({ label, description, checked, onChange }: {
  label: string; description: string; checked: boolean; onChange: (v: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between py-2">
      <div>
        <div className="text-sm font-medium">{label}</div>
        <div className="text-xs text-gray-500">{description}</div>
      </div>
      <label className="relative inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          className="sr-only peer"
          checked={checked}
          onChange={e => onChange(e.target.checked)}
        />
        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-600"></div>
      </label>
    </div>
  )
}

function NumberSetting({ label, value, onChange }: {
  label: string; value: number; onChange: (v: number) => void
}) {
  return (
    <div>
      <label className="block text-sm font-medium mb-1">{label}</label>
      <input
        type="number"
        value={value}
        onChange={e => onChange(parseInt(e.target.value) || 0)}
        className="w-full border rounded-lg p-2 text-sm"
        min={1}
        max={200}
      />
    </div>
  )
}
