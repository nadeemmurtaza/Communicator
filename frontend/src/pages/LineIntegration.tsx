import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type LineChannelData,
  type SocialEvent,
} from '../lib/api'
import { LineIcon } from '../components/icons/BrandIcons'

export function LineIntegration() {
  const [data, setData] = useState<LineChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'credentials' | 'webhooks' | 'quick_replies' | 'automations' | 'diagnostics'
  >('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Credentials State
  const [inboxName, setInboxName] = useState('LINE Support')
  const [channelId, setChannelId] = useState('1657890123')
  const [channelSecret, setChannelSecret] = useState('')
  const [channelAccessToken, setChannelAccessToken] = useState('')
  const [lineAccountId, setLineAccountId] = useState('@bayworks')

  // Quick Replies State
  const [quickReplies, setQuickReplies] = useState<Array<{ label: string; text: string }>>([
    { label: 'Pricing Plans', text: 'What are your pricing plans?' },
    { label: 'Tech Support', text: 'I need technical support' },
    { label: 'Contact Sales', text: 'Can I speak with a sales representative?' },
  ])
  const [newLabel, setNewLabel] = useState('')
  const [newText, setNewText] = useState('')

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState(
    '👋 Welcome to Bay Works LINE Official Account! How can our support team assist you today?'
  )
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState(
    'Thank you for contacting us on LINE. We are currently outside business hours and will respond as soon as we reopen.'
  )
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('Asia/Tokyo')

  // Diagnostic Test Sender State
  const [testUserId, setTestUserId] = useState('U4af4980629fbcc4c3fffed0cfa255d6')
  const [testRecipientName, setTestRecipientName] = useState('Kenji Sato')
  const [testMessage, setTestMessage] = useState(
    'Hello! This is a test push notification from your Bay Works LINE Messaging API integration.'
  )
  const [testResult, setTestResult] = useState<{ messageId: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getLineChannel(),
        api.listLineEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setChannelId(channelData.accountProfile?.channelId || channelData.inbox.credentials?.channelId || '1657890123')
      setLineAccountId(channelData.accountProfile?.lineAccountId || channelData.inbox.credentials?.lineAccountId || '@bayworks')

      if (channelData.quickReplies && channelData.quickReplies.length > 0) {
        setQuickReplies(channelData.quickReplies)
      }

      setGreetingEnabled(channelData.inbox.greetingEnabled)
      setGreetingMessage(channelData.inbox.greetingMessage)
      setOutOfOfficeEnabled(channelData.inbox.outOfOfficeEnabled)
      setOutOfOfficeMessage(channelData.inbox.outOfOfficeMessage)
      setCsatEnabled(channelData.inbox.csatEnabled)
      setWorkingHoursEnabled(channelData.inbox.workingHoursEnabled)
      setWorkingHoursStart(channelData.inbox.workingHoursStart || 9)
      setWorkingHoursEnd(channelData.inbox.workingHoursEnd || 18)
      setTimezone(channelData.inbox.timezone || 'Asia/Tokyo')
    } catch (err: any) {
      setError(err.message || 'Failed to load LINE channel configuration.')
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
        channel_id: channelId,
        line_account_id: lineAccountId,
        quick_replies: quickReplies,
        greeting_enabled: greetingEnabled,
        greeting_message: greetingMessage,
        out_of_office_enabled: outOfOfficeEnabled,
        out_of_office_message: outOfOfficeMessage,
        csat_enabled: csatEnabled,
        working_hours_enabled: workingHoursEnabled,
        working_hours_start: Number(workingHoursStart),
        working_hours_end: Number(workingHoursEnd),
        timezone,
      }

      if (channelSecret) payload.channel_secret = channelSecret
      if (channelAccessToken) payload.channel_access_token = channelAccessToken

      const res = await api.updateLineChannel(payload)
      setData(res)
      setSuccess('LINE channel settings saved and synchronized successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Failed to save LINE channel settings.')
    } finally {
      setSaving(false)
    }
  }

  async function handleVerifyConnection() {
    try {
      setVerifying(true)
      setError(null)
      const res = await api.verifyLineConnection()
      setSuccess(res.message || 'LINE Official Account verified successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'LINE channel verification failed.')
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTestMessage() {
    try {
      setSendingTest(true)
      setError(null)
      setTestResult(null)

      const res = await api.sendLineTestMessage({
        recipient_user_id: testUserId,
        recipient_name: testRecipientName,
        message: testMessage,
      })

      setTestResult({
        messageId: res.messageId,
        message: res.message,
      })
      setSuccess(`Push message dispatched successfully! Message ID: ${res.messageId}`)

      // Refresh event log
      const updatedEvents = await api.listLineEvents()
      setEvents(updatedEvents)
    } catch (err: any) {
      setError(err.message || 'Failed to dispatch test LINE message.')
    } finally {
      setSendingTest(false)
    }
  }

  function handleAddQuickReply() {
    if (!newLabel.trim() || !newText.trim()) return
    setQuickReplies([...quickReplies, { label: newLabel.trim(), text: newText.trim() }])
    setNewLabel('')
    setNewText('')
  }

  function handleRemoveQuickReply(index: number) {
    setQuickReplies(quickReplies.filter((_, i) => i !== index))
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2500)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[500px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-emerald-500"></div>
        <span className="ml-3 text-sm text-gray-500 font-medium">Loading LINE channel configuration...</span>
      </div>
    )
  }

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6">
      {/* Top Banner Navigation */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-gray-200">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Link to="/channels" className="text-sm font-medium text-emerald-600 hover:text-emerald-800">
              ← Back to Channels Hub
            </Link>
            <span className="text-gray-300">/</span>
            <span className="text-sm font-medium text-gray-500">LINE Messaging</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-emerald-500 to-green-600 flex items-center justify-center text-white shadow-sm">
              <LineIcon className="h-6 w-6" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 tracking-tight">LINE Messaging API Integration</h1>
              <p className="text-sm text-gray-500">
                Official LINE Messaging API connector with push messaging, HMAC-SHA256 webhook signatures, and quick reply buttons.
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
            {verifying ? 'Checking LINE...' : '⚡ Test Connection'}
          </button>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="px-4 py-2 text-sm font-medium text-white bg-emerald-600 rounded-lg hover:bg-emerald-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
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
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Official Account</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{lineAccountId || '@bayworks'}</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Verified
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Channel ID: {channelId}</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Signature Validation</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">HMAC-SHA256</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Active
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">X-Line-Signature verified</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Quick Replies</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{quickReplies.length} Action Chips</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-green-50 text-green-700 border border-green-200">
              Interactive
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Appended to support responses</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Push Messages Quota</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">Unlimited</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Standard
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">api.line.me/v2/bot</p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8">
          {[
            { id: 'credentials', label: 'Channel Credentials & IDs', icon: '🔑' },
            { id: 'webhooks', label: 'Webhooks & HMAC Security', icon: '⚡' },
            { id: 'quick_replies', label: 'Quick Replies & Menus', icon: '💡' },
            { id: 'automations', label: 'Automations & Schedule', icon: '🤖' },
            { id: 'diagnostics', label: 'Diagnostic Sender & Events', icon: '📡' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`py-4 px-1 inline-flex items-center gap-2 border-b-2 font-medium text-sm transition ${
                activeTab === tab.id
                  ? 'border-emerald-600 text-emerald-600 font-semibold'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Tab 1: Credentials */}
      {activeTab === 'credentials' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">LINE Developers Console Credentials</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Obtain your credentials from the <strong>LINE Developers Console</strong> under your Messaging API channel.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Inbox Display Name</label>
              <input
                type="text"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                placeholder="e.g. LINE Support"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">LINE Official Account ID (@handle)</label>
              <input
                type="text"
                value={lineAccountId}
                onChange={(e) => setLineAccountId(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                placeholder="e.g. @bayworks"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">LINE Channel ID</label>
              <input
                type="text"
                value={channelId}
                onChange={(e) => setChannelId(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                placeholder="e.g. 1657890123"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">LINE Channel Secret</label>
              <input
                type="password"
                value={channelSecret}
                onChange={(e) => setChannelSecret(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-gray-400 mt-1">Used to compute and verify the X-Line-Signature HMAC header.</p>
            </div>

            <div className="md:col-span-2">
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Channel Access Token (Long-lived)
              </label>
              <input
                type="password"
                value={channelAccessToken}
                onChange={(e) => setChannelAccessToken(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-gray-400 mt-1">
                Issue a long-lived channel access token in the Messaging API tab of the LINE Developers Console.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Webhooks & Signature Validation */}
      {activeTab === 'webhooks' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">LINE Webhook Settings</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Paste this URL into your LINE Developers Console under <strong>Messaging API → Webhook settings</strong>.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Webhook URL</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/line'}
                  className="w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-lg text-sm font-mono text-gray-600"
                />
                <button
                  onClick={() =>
                    copyToClipboard(
                      data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/line',
                      'callback'
                    )
                  }
                  className="px-4 py-2 bg-gray-100 hover:bg-gray-200 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 transition"
                >
                  {copiedKey === 'callback' ? '✓ Copied' : 'Copy'}
                </button>
              </div>
            </div>

            <div className="p-4 bg-emerald-50 border border-emerald-100 rounded-xl space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-emerald-700 font-bold">🔒 Cryptographic Verification</span>
                <span className="px-2 py-0.5 text-xs font-semibold rounded bg-emerald-100 text-emerald-800">
                  X-Line-Signature
                </span>
              </div>
              <p className="text-xs text-emerald-900 leading-relaxed">
                All requests received on this webhook are rigorously verified against your Channel Secret using HMAC-SHA256 Base64 digest validation, rejecting unauthorized or spoofed payloads.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Quick Replies & Menus */}
      {activeTab === 'quick_replies' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">LINE Quick Replies & Action Chips</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Quick replies appear horizontally above the user's keyboard on LINE, letting customers respond with a single tap.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div className="space-y-3">
              {quickReplies.map((qr, idx) => (
                <div key={idx} className="flex items-center justify-between p-3.5 bg-gray-50 border border-gray-200 rounded-xl">
                  <div className="flex items-center gap-3">
                    <span className="w-6 h-6 rounded-full bg-emerald-100 text-emerald-700 font-bold text-xs flex items-center justify-center">
                      {idx + 1}
                    </span>
                    <div>
                      <div className="text-sm font-medium text-gray-900">{qr.label}</div>
                      <div className="text-xs text-gray-500">Action Text: "{qr.text}"</div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleRemoveQuickReply(idx)}
                    className="text-sm text-rose-600 hover:text-rose-800 font-medium px-2 py-1 rounded hover:bg-rose-50 transition"
                  >
                    Delete
                  </button>
                </div>
              ))}
            </div>

            <div className="p-4 bg-emerald-50 border border-emerald-100 rounded-xl space-y-3">
              <h4 className="text-xs font-semibold text-emerald-900 uppercase tracking-wider">Add Quick Reply Option</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <input
                  type="text"
                  placeholder="Button Label (e.g. Schedule Call)"
                  value={newLabel}
                  onChange={(e) => setNewLabel(e.target.value)}
                  className="px-3 py-2 border border-emerald-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-emerald-500"
                />
                <input
                  type="text"
                  placeholder="Sent Text (e.g. I would like to schedule a call)"
                  value={newText}
                  onChange={(e) => setNewText(e.target.value)}
                  className="px-3 py-2 border border-emerald-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-emerald-500"
                />
              </div>
              <button
                onClick={handleAddQuickReply}
                disabled={!newLabel.trim() || !newText.trim()}
                className="px-4 py-1.5 text-xs font-semibold text-white bg-emerald-600 rounded-lg hover:bg-emerald-700 disabled:opacity-50 transition"
              >
                + Add Quick Reply Chip
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tab 4: Automations & Schedule */}
      {activeTab === 'automations' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">LINE Automations & Operating Schedule</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Automated greetings, after-hours responses, and CSAT customer satisfaction surveys.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-gray-100">
            {/* Welcome Greeting */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Welcome Message</h4>
                  <p className="text-xs text-gray-500">Sent automatically when a user adds your Official Account as a friend.</p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-4 w-4 text-emerald-600 rounded border-gray-300 focus:ring-emerald-500"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">After-Hours Away Message</h4>
                  <p className="text-xs text-gray-500">Sent to messages received outside scheduled working hours.</p>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="h-4 w-4 text-emerald-600 rounded border-gray-300 focus:ring-emerald-500"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                />
              )}
            </div>

            {/* CSAT Survey */}
            <div className="flex items-center justify-between p-4 bg-gray-50 border border-gray-200 rounded-xl">
              <div>
                <h4 className="text-sm font-semibold text-gray-900">CSAT Customer Rating</h4>
                <p className="text-xs text-gray-500">Asks for a satisfaction rating when conversation is marked resolved.</p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-4 w-4 text-emerald-600 rounded border-gray-300 focus:ring-emerald-500"
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
                  className="h-4 w-4 text-emerald-600 rounded border-gray-300 focus:ring-emerald-500"
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

      {/* Tab 5: Diagnostic Sender & Events */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-4">
            <div>
              <h3 className="text-base font-semibold text-gray-900">Live Diagnostic Push Message Dispatcher</h3>
              <p className="text-sm text-gray-500 mt-0.5">
                Dispatch a test push message to a LINE User ID to verify API connectivity and delivery.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient LINE User ID</label>
                <input
                  type="text"
                  value={testUserId}
                  onChange={(e) => setTestUserId(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                  placeholder="e.g. U4af4980629fbcc4c3fffed0cfa255d6"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient Name</label>
                <input
                  type="text"
                  value={testRecipientName}
                  onChange={(e) => setTestRecipientName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                  placeholder="e.g. Kenji Sato"
                />
              </div>

              <div className="md:col-span-2">
                <label className="block text-xs font-medium text-gray-700 mb-1">Test Message Body</label>
                <textarea
                  rows={2}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={handleSendTestMessage}
                disabled={sendingTest || !testUserId.trim() || !testMessage.trim()}
                className="px-4 py-2 text-sm font-medium text-white bg-emerald-600 rounded-lg hover:bg-emerald-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
              >
                {sendingTest ? 'Dispatching...' : '🚀 Send Diagnostic Push Message'}
              </button>

              {testResult && (
                <span className="text-xs font-mono text-emerald-700 bg-emerald-50 px-3 py-1 rounded-full border border-emerald-200">
                  ID: {testResult.messageId}
                </span>
              )}
            </div>
          </div>

          {/* Live Events Table */}
          <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-base font-semibold text-gray-900">Recent LINE Message Events</h3>
                <p className="text-sm text-gray-500">Live feed of inbound messages, push responses, and user postbacks.</p>
              </div>
              <button
                onClick={async () => setEvents(await api.listLineEvents())}
                className="px-3 py-1.5 text-xs font-medium text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
              >
                🔄 Refresh
              </button>
            </div>

            {events.length === 0 ? (
              <div className="py-12 text-center text-sm text-gray-500">
                No recent LINE events recorded yet. Dispatch a diagnostic push message above to verify.
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
                                ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                                : 'bg-green-50 text-green-700 border border-green-200'
                            }`}
                          >
                            {ev.direction === 'inbound' ? 'Inbound' : 'Outbound Push'}
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
