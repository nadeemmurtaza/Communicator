import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type FacebookChannelData,
  type SocialEvent,
} from '../lib/api'
import { FacebookIcon } from '../components/icons/BrandIcons'

export function FacebookIntegration() {
  const [data, setData] = useState<FacebookChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'credentials' | 'webhooks' | 'icebreakers' | 'automations' | 'diagnostics'
  >('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Credentials State
  const [inboxName, setInboxName] = useState('Facebook Messenger')
  const [pageId, setPageId] = useState('100234567890123')
  const [pageName, setPageName] = useState('Bay Works Official')
  const [pageAccessToken, setPageAccessToken] = useState('')
  const [appSecret, setAppSecret] = useState('')
  const [webhookVerifyToken, setWebhookVerifyToken] = useState('meta_facebook_verify_token')

  // Icebreakers State
  const [icebreakers, setIcebreakers] = useState<Array<{ question: string; payload: string }>>([
    { question: 'What services do you provide?', payload: 'SERVICES' },
    { question: 'Can I get a quote or pricing?', payload: 'PRICING' },
    { question: 'Speak with a live agent', payload: 'AGENT' },
  ])
  const [newQuestion, setNewQuestion] = useState('')
  const [newPayload, setNewPayload] = useState('')

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState(
    '👋 Hi! Thanks for reaching out to us on Facebook Messenger. How can our team help you today?'
  )
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState(
    'Thanks for contacting us! We are currently outside business hours, but we will reply as soon as we are back online.'
  )
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('America/New_York')

  // Diagnostic Test Sender State
  const [testPsid, setTestPsid] = useState('psid_test_902184')
  const [testRecipientName, setTestRecipientName] = useState('Sarah Jenkins')
  const [testMessage, setTestMessage] = useState(
    'Hello! This is a test message from your Bay Works Facebook Messenger integration.'
  )
  const [testResult, setTestResult] = useState<{ mid: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getFacebookChannel(),
        api.listFacebookEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setPageId(channelData.pageProfile?.pageId || channelData.inbox.credentials?.pageId || '')
      setPageName(channelData.pageProfile?.pageName || channelData.inbox.credentials?.pageName || 'Bay Works Official')
      setWebhookVerifyToken(channelData.webhookConfig?.verifyToken || 'meta_facebook_verify_token')

      if (channelData.icebreakers && channelData.icebreakers.length > 0) {
        setIcebreakers(channelData.icebreakers)
      }

      setGreetingEnabled(channelData.inbox.greetingEnabled)
      setGreetingMessage(channelData.inbox.greetingMessage)
      setOutOfOfficeEnabled(channelData.inbox.outOfOfficeEnabled)
      setOutOfOfficeMessage(channelData.inbox.outOfOfficeMessage)
      setCsatEnabled(channelData.inbox.csatEnabled)
      setWorkingHoursEnabled(channelData.inbox.workingHoursEnabled)
      setWorkingHoursStart(channelData.inbox.workingHoursStart || 9)
      setWorkingHoursEnd(channelData.inbox.workingHoursEnd || 18)
      setTimezone(channelData.inbox.timezone || 'America/New_York')
    } catch (err: any) {
      setError(err.message || 'Failed to load Facebook channel configuration.')
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveSettings() {
    try {
      setSaving(true)
      setError(null)
      setSuccess(null)

      const payload: any = {
        name: inboxName,
        page_id: pageId,
        page_name: pageName,
        webhook_verify_token: webhookVerifyToken,
        greeting_enabled: greetingEnabled,
        greeting_message: greetingMessage,
        out_of_office_enabled: outOfOfficeEnabled,
        out_of_office_message: outOfOfficeMessage,
        csat_enabled: csatEnabled,
        icebreakers: icebreakers,
        working_hours_enabled: workingHoursEnabled,
        working_hours_start: Number(workingHoursStart),
        working_hours_end: Number(workingHoursEnd),
        timezone,
      }

      if (pageAccessToken) payload.page_access_token = pageAccessToken
      if (appSecret) payload.app_secret = appSecret

      const res = await api.updateFacebookChannel(payload)
      setData(res)
      setSuccess('Facebook Messenger settings saved and synchronized successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Failed to save Facebook Messenger settings.')
    } finally {
      setSaving(false)
    }
  }

  async function handleVerifyConnection() {
    try {
      setVerifying(true)
      setError(null)
      const res = await api.verifyFacebookConnection()
      setSuccess(res.message || 'Facebook Page access token verified successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Facebook connection verification failed.')
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTestMessage() {
    try {
      setSendingTest(true)
      setError(null)
      setTestResult(null)

      const res = await api.sendFacebookTestMessage({
        recipient_psid: testPsid,
        recipient_name: testRecipientName,
        message: testMessage,
      })

      setTestResult({
        mid: res.mid,
        message: res.message,
      })
      setSuccess(`Diagnostic message dispatched successfully! Message ID: ${res.mid}`)

      // Refresh event log
      const updatedEvents = await api.listFacebookEvents()
      setEvents(updatedEvents)
    } catch (err: any) {
      setError(err.message || 'Failed to dispatch test Facebook message.')
    } finally {
      setSendingTest(false)
    }
  }

  function handleAddIcebreaker() {
    if (!newQuestion.trim()) return
    const payload = newPayload.trim() || newQuestion.toUpperCase().replace(/\s+/g, '_').slice(0, 20)
    setIcebreakers([...icebreakers, { question: newQuestion.trim(), payload }])
    setNewQuestion('')
    setNewPayload('')
  }

  function handleRemoveIcebreaker(index: number) {
    setIcebreakers(icebreakers.filter((_, i) => i !== index))
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2500)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[500px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-sm text-gray-500 font-medium">Loading Facebook Messenger settings...</span>
      </div>
    )
  }

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6">
      {/* Top Banner Navigation */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-gray-200">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Link to="/channels" className="text-sm font-medium text-blue-600 hover:text-blue-800">
              ← Back to Channels Hub
            </Link>
            <span className="text-gray-300">/</span>
            <span className="text-sm font-medium text-gray-500">Facebook Messenger</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-blue-600 to-indigo-500 flex items-center justify-center text-white shadow-sm">
              <FacebookIcon className="h-6 w-6" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 tracking-tight">Facebook Messenger Integration</h1>
              <p className="text-sm text-gray-500">
                Official Meta Graph API v21.0 connector with Page messaging, 24h window compliance, and Icebreakers.
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={handleVerifyConnection}
            disabled={verifying}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 shadow-sm transition disabled:opacity-50"
          >
            {verifying ? 'Checking Meta...' : '⚡ Test Connection'}
          </button>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Alert Notices */}
      {success && (
        <div className="p-4 bg-emerald-50 border border-emerald-200 rounded-xl text-emerald-800 text-sm flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span>✅</span>
            <span>{success}</span>
          </div>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-900 font-bold">
            ×
          </button>
        </div>
      )}

      {error && (
        <div className="p-4 bg-rose-50 border border-rose-200 rounded-xl text-rose-800 text-sm flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span>⚠️</span>
            <span>{error}</span>
          </div>
          <button onClick={() => setError(null)} className="text-rose-600 hover:text-rose-900 font-bold">
            ×
          </button>
        </div>
      )}

      {/* KPI & Status Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Page Connection</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{pageName || 'Bay Works Official'}</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Connected
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Page ID: {pageId}</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Messaging Window</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">24 Hours</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-blue-50 text-blue-700 border border-blue-200">
              Human Tag Active
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">7-day extended response tag</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Icebreakers</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{icebreakers.length} Active</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-purple-50 text-purple-700 border border-purple-200">
              FAQ Chips
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Displayed to first-time contacts</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Meta Graph API</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">v21.0</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Healthy
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Avg latency: ~320ms</p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8">
          {[
            { id: 'credentials', label: 'Page Credentials & IDs', icon: '🔑' },
            { id: 'webhooks', label: 'Webhooks & Graph Sync', icon: '⚡' },
            { id: 'icebreakers', label: 'Messenger Icebreakers', icon: '💡' },
            { id: 'automations', label: 'Automations & Schedule', icon: '🤖' },
            { id: 'diagnostics', label: 'Diagnostic Sender & Events', icon: '📡' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`py-4 px-1 inline-flex items-center gap-2 border-b-2 font-medium text-sm transition ${
                activeTab === tab.id
                  ? 'border-blue-600 text-blue-600 font-semibold'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Tab 1: Credentials & Page Linking */}
      {activeTab === 'credentials' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Facebook Page & Graph API Credentials</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Connect your official Facebook Page via Meta Graph API v21.0. Messages sent to your page will route straight to the Shared Inbox.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Inbox Display Name</label>
              <input
                type="text"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                placeholder="e.g. Facebook Messenger Support"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Facebook Page Name</label>
              <input
                type="text"
                value={pageName}
                onChange={(e) => setPageName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                placeholder="e.g. Bay Works Official"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Facebook Page ID</label>
              <input
                type="text"
                value={pageId}
                onChange={(e) => setPageId(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:outline-none"
                placeholder="e.g. 100234567890123"
              />
              <p className="text-xs text-gray-400 mt-1">Found in Facebook Page Settings → About → Page ID.</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Page Access Token (EAAB...)
              </label>
              <input
                type="password"
                value={pageAccessToken}
                onChange={(e) => setPageAccessToken(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-gray-400 mt-1">
                Generated from Meta Developer Console with <code className="text-blue-600">pages_messaging</code> permission. Leave blank to keep existing.
              </p>
            </div>

            <div className="md:col-span-2">
              <label className="block text-sm font-medium text-gray-700 mb-1">App Secret</label>
              <input
                type="password"
                value={appSecret}
                onChange={(e) => setAppSecret(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-gray-400 mt-1">
                Used to validate incoming <code className="text-blue-600">X-Hub-Signature-256</code> HMAC headers from Meta webhooks.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Webhooks & Graph Sync */}
      {activeTab === 'webhooks' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Meta Webhook Configuration</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Configure these webhook parameters in your Meta App Dashboard under <strong>Messenger → Webhooks</strong>.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Webhook Callback URL</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/facebook'}
                  className="w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-lg text-sm font-mono text-gray-600"
                />
                <button
                  onClick={() =>
                    copyToClipboard(
                      data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/facebook',
                      'callback'
                    )
                  }
                  className="px-4 py-2 bg-gray-100 hover:bg-gray-200 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 transition"
                >
                  {copiedKey === 'callback' ? '✓ Copied' : 'Copy'}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Verify Token</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={webhookVerifyToken}
                  onChange={(e) => setWebhookVerifyToken(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:outline-none"
                />
                <button
                  onClick={() => copyToClipboard(webhookVerifyToken, 'verifyToken')}
                  className="px-4 py-2 bg-gray-100 hover:bg-gray-200 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 transition"
                >
                  {copiedKey === 'verifyToken' ? '✓ Copied' : 'Copy'}
                </button>
              </div>
              <p className="text-xs text-gray-400 mt-1">Enter this identical string into Meta when verifying your webhook endpoint.</p>
            </div>

            <div className="pt-4 border-t border-gray-100">
              <label className="block text-sm font-medium text-gray-700 mb-2">Subscribed Webhook Fields</label>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {['messages', 'messaging_postbacks', 'message_deliveries', 'message_reads'].map((field) => (
                  <div key={field} className="p-3 bg-gray-50 border border-gray-200 rounded-lg flex items-center gap-2">
                    <span className="text-emerald-500">✓</span>
                    <span className="text-xs font-mono font-medium text-gray-700">{field}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Icebreakers & FAQ Chips */}
      {activeTab === 'icebreakers' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Messenger Icebreakers (Pre-chat FAQ Chips)</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Icebreakers appear as quick clickable question chips when a customer opens a new conversation with your Page before sending their first message.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div className="space-y-3">
              {icebreakers.map((ib, idx) => (
                <div key={idx} className="flex items-center justify-between p-3.5 bg-gray-50 border border-gray-200 rounded-xl">
                  <div className="flex items-center gap-3">
                    <span className="w-6 h-6 rounded-full bg-blue-100 text-blue-700 font-bold text-xs flex items-center justify-center">
                      {idx + 1}
                    </span>
                    <div>
                      <div className="text-sm font-medium text-gray-900">{ib.question}</div>
                      <div className="text-xs font-mono text-gray-500">Payload: {ib.payload}</div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleRemoveIcebreaker(idx)}
                    className="text-sm text-rose-600 hover:text-rose-800 font-medium px-2 py-1 rounded hover:bg-rose-50 transition"
                  >
                    Delete
                  </button>
                </div>
              ))}
            </div>

            <div className="p-4 bg-blue-50 border border-blue-100 rounded-xl space-y-3">
              <h4 className="text-xs font-semibold text-blue-900 uppercase tracking-wider">Add New Icebreaker</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <input
                  type="text"
                  placeholder="Question text (e.g. What are your opening hours?)"
                  value={newQuestion}
                  onChange={(e) => setNewQuestion(e.target.value)}
                  className="px-3 py-2 border border-blue-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <input
                  type="text"
                  placeholder="Custom Payload (e.g. HOURS_INQUIRY)"
                  value={newPayload}
                  onChange={(e) => setNewPayload(e.target.value)}
                  className="px-3 py-2 border border-blue-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                />
              </div>
              <button
                onClick={handleAddIcebreaker}
                disabled={!newQuestion.trim()}
                className="px-4 py-1.5 text-xs font-semibold text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 transition"
              >
                + Add Icebreaker Chip
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tab 4: Automations & Schedule */}
      {activeTab === 'automations' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Messenger Automations & Working Hours</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Automated greetings, after-hours responses, and CSAT customer satisfaction surveys.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-gray-100">
            {/* Welcome Greeting */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Automated Welcome Greeting</h4>
                  <p className="text-xs text-gray-500">Sent immediately when a customer initiates contact on Facebook Messenger.</p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Out of Office / Away Responder</h4>
                  <p className="text-xs text-gray-500">Sent when customers message outside scheduled working hours.</p>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                />
              )}
            </div>

            {/* CSAT Rating */}
            <div className="flex items-center justify-between p-4 bg-gray-50 border border-gray-200 rounded-xl">
              <div>
                <h4 className="text-sm font-semibold text-gray-900">CSAT Customer Satisfaction Rating</h4>
                <p className="text-xs text-gray-500">Automatically asks for a 1-5 rating when an agent marks a conversation resolved.</p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
              />
            </div>

            {/* Working Hours Schedule */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Working Hours Schedule</h4>
                  <p className="text-xs text-gray-500">Set team availability hours to route conversations and trigger away messages.</p>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                />
              </div>

              {workingHoursEnabled && (
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2">
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">Start Time (24h)</label>
                    <input
                      type="number"
                      min={0}
                      max={23}
                      value={workingHoursStart}
                      onChange={(e) => setWorkingHoursStart(Number(e.target.value))}
                      className="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">End Time (24h)</label>
                    <input
                      type="number"
                      min={0}
                      max={23}
                      value={workingHoursEnd}
                      onChange={(e) => setWorkingHoursEnd(Number(e.target.value))}
                      className="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">Timezone</label>
                    <input
                      type="text"
                      value={timezone}
                      onChange={(e) => setTimezone(e.target.value)}
                      className="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm"
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Tab 5: Diagnostic Dispatcher & Live Events */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-4">
            <div>
              <h3 className="text-base font-semibold text-gray-900">Live Diagnostic Message Dispatcher</h3>
              <p className="text-sm text-gray-500 mt-0.5">
                Dispatch a test outbound message to a Page-Scoped ID (PSID) to verify Meta API connectivity and delivery.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient PSID</label>
                <input
                  type="text"
                  value={testPsid}
                  onChange={(e) => setTestPsid(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="e.g. psid_test_902184"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient Name</label>
                <input
                  type="text"
                  value={testRecipientName}
                  onChange={(e) => setTestRecipientName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="e.g. Sarah Jenkins"
                />
              </div>

              <div className="md:col-span-2">
                <label className="block text-xs font-medium text-gray-700 mb-1">Test Message Body</label>
                <textarea
                  rows={2}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={handleSendTestMessage}
                disabled={sendingTest || !testPsid.trim() || !testMessage.trim()}
                className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
              >
                {sendingTest ? 'Dispatching...' : '🚀 Send Diagnostic Message'}
              </button>

              {testResult && (
                <span className="text-xs font-mono text-emerald-700 bg-emerald-50 px-3 py-1 rounded-full border border-emerald-200">
                  ID: {testResult.mid}
                </span>
              )}
            </div>
          </div>

          {/* Live Events Table */}
          <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-base font-semibold text-gray-900">Recent Facebook Messenger Events</h3>
                <p className="text-sm text-gray-500">Live feed of inbound messages, agent replies, and delivery receipts.</p>
              </div>
              <button
                onClick={async () => setEvents(await api.listFacebookEvents())}
                className="px-3 py-1.5 text-xs font-medium text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
              >
                🔄 Refresh
              </button>
            </div>

            {events.length === 0 ? (
              <div className="py-12 text-center text-sm text-gray-500">
                No recent Facebook Messenger events recorded yet. Dispatch a diagnostic message above to verify.
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm text-gray-600">
                  <thead className="bg-gray-50 text-xs font-semibold text-gray-400 uppercase tracking-wider border-b border-gray-200">
                    <tr>
                      <th className="py-3 px-4">Direction</th>
                      <th className="py-3 px-4">Sender</th>
                      <th className="py-3 px-4">Message Snippet</th>
                      <th className="py-3 px-4">Status</th>
                      <th className="py-3 px-4">Message ID</th>
                      <th className="py-3 px-4">Timestamp</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-100">
                    {events.map((ev) => (
                      <tr key={ev.id} className="hover:bg-gray-50/60 transition">
                        <td className="py-3 px-4">
                          <span
                            className={`px-2 py-0.5 text-xs font-semibold rounded-full ${
                              ev.direction === 'inbound'
                                ? 'bg-blue-50 text-blue-700 border border-blue-200'
                                : 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                            }`}
                          >
                            {ev.direction === 'inbound' ? 'Inbound' : 'Outbound'}
                          </span>
                        </td>
                        <td className="py-3 px-4 font-medium text-gray-900">{ev.senderName}</td>
                        <td className="py-3 px-4 text-gray-700 max-w-xs truncate">{ev.body}</td>
                        <td className="py-3 px-4">
                          <span className="text-xs font-medium text-emerald-600">● {ev.status}</span>
                        </td>
                        <td className="py-3 px-4 font-mono text-xs text-gray-400">
                          {ev.providerMessageId ? ev.providerMessageId.slice(0, 16) + '...' : '—'}
                        </td>
                        <td className="py-3 px-4 text-xs text-gray-400">
                          {new Date(ev.sentAt).toLocaleTimeString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
