import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type InstagramChannelData,
  type SocialEvent,
} from '../lib/api'

export function InstagramIntegration() {
  const [data, setData] = useState<InstagramChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'account' | 'webhooks' | 'story_mentions' | 'icebreakers' | 'diagnostics'
  >('account')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Account State
  const [inboxName, setInboxName] = useState('Instagram Direct')
  const [igUsername, setIgUsername] = useState('@bayworks')
  const [igId, setIgId] = useState('17841405822345678')
  const [connectedPageId, setConnectedPageId] = useState('100234567890123')
  const [accessToken, setAccessToken] = useState('')
  const [appSecret, setAppSecret] = useState('')
  const [webhookVerifyToken, setWebhookVerifyToken] = useState('meta_instagram_verify_token')

  // Story Mentions State
  const [storyMentionsEnabled, setStoryMentionsEnabled] = useState(true)
  const [storyReplyMessage, setStoryReplyMessage] = useState(
    'Thanks for mentioning us in your story! 🔥 How can we assist you today?'
  )

  // Icebreakers State
  const [icebreakers, setIcebreakers] = useState<Array<{ question: string; payload: string }>>([
    { question: 'See our product portfolio', payload: 'PORTFOLIO' },
    { question: 'Request collaboration or pricing', payload: 'COLLAB' },
    { question: 'Talk to support', payload: 'SUPPORT' },
  ])
  const [newQuestion, setNewQuestion] = useState('')
  const [newPayload, setNewPayload] = useState('')

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState(
    'Hey! Thanks for DMing Bay Works on Instagram. How can we help you today?'
  )
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState(
    "We're away from Instagram DMs right now, but will reply first thing tomorrow!"
  )
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('America/New_York')

  // Diagnostic Test Sender State
  const [testIgsid, setTestIgsid] = useState('igsid_test_549120')
  const [testUsername, setTestUsername] = useState('clara_design')
  const [testMessage, setTestMessage] = useState(
    'Hey Clara! This is an automated test DM from your Bay Works Instagram integration.'
  )
  const [testResult, setTestResult] = useState<{ mid: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getInstagramChannel(),
        api.listInstagramEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setIgUsername(channelData.accountProfile?.igUsername || channelData.inbox.credentials?.igUsername || '@bayworks')
      setIgId(channelData.accountProfile?.igId || channelData.inbox.credentials?.igId || '')
      setConnectedPageId(channelData.accountProfile?.connectedFacebookPage || channelData.inbox.credentials?.connectedPageId || '')
      setWebhookVerifyToken(channelData.webhookConfig?.verifyToken || 'meta_instagram_verify_token')

      if (channelData.storyMentions) {
        setStoryMentionsEnabled(channelData.storyMentions.enabled)
        setStoryReplyMessage(channelData.storyMentions.replyMessage)
      }

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
      setError(err.message || 'Failed to load Instagram channel configuration.')
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
        ig_username: igUsername,
        ig_id: igId,
        connected_page_id: connectedPageId,
        webhook_verify_token: webhookVerifyToken,
        story_mentions_enabled: storyMentionsEnabled,
        story_reply_message: storyReplyMessage,
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

      if (accessToken) payload.access_token = accessToken
      if (appSecret) payload.app_secret = appSecret

      const res = await api.updateInstagramChannel(payload)
      setData(res)
      setSuccess('Instagram Direct settings saved and synchronized successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Failed to save Instagram Direct settings.')
    } finally {
      setSaving(false)
    }
  }

  async function handleVerifyConnection() {
    try {
      setVerifying(true)
      setError(null)
      const res = await api.verifyInstagramConnection()
      setSuccess(res.message || 'Instagram Professional account verified successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Instagram account verification failed.')
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTestMessage() {
    try {
      setSendingTest(true)
      setError(null)
      setTestResult(null)

      const res = await api.sendInstagramTestMessage({
        recipient_igsid: testIgsid,
        recipient_username: testUsername,
        message: testMessage,
      })

      setTestResult({
        mid: res.mid,
        message: res.message,
      })
      setSuccess(`Diagnostic DM dispatched successfully! Message ID: ${res.mid}`)

      // Refresh event log
      const updatedEvents = await api.listInstagramEvents()
      setEvents(updatedEvents)
    } catch (err: any) {
      setError(err.message || 'Failed to dispatch test Instagram message.')
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
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-pink-600"></div>
        <span className="ml-3 text-sm text-gray-500 font-medium">Loading Instagram Direct settings...</span>
      </div>
    )
  }

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6">
      {/* Top Banner Navigation */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-gray-200">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Link to="/channels" className="text-sm font-medium text-pink-600 hover:text-pink-800">
              ← Back to Channels Hub
            </Link>
            <span className="text-gray-300">/</span>
            <span className="text-sm font-medium text-gray-500">Instagram Direct</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-yellow-500 via-pink-600 to-purple-600 flex items-center justify-center text-white shadow-sm font-bold text-xl">
              📸
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 tracking-tight">Instagram Direct Integration</h1>
              <p className="text-sm text-gray-500">
                Instagram Professional & Business Direct Messages, Story Mentions Auto-Reply, and Icebreaker FAQ chips.
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
            {verifying ? 'Checking Instagram...' : '⚡ Test Connection'}
          </button>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="px-4 py-2 text-sm font-medium text-white bg-gradient-to-r from-pink-600 to-purple-600 rounded-lg hover:from-pink-700 hover:to-purple-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
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
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Account Handle</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{igUsername || '@bayworks'}</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Verified
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">IG ID: {igId}</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Story Mentions</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">
              {storyMentionsEnabled ? 'Auto-Reply ON' : 'Disabled'}
            </span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-pink-50 text-pink-700 border border-pink-200">
              Story Sync
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Direct instant reply trigger</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Icebreakers</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{icebreakers.length} Active</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-purple-50 text-purple-700 border border-purple-200">
              FAQ Chips
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Direct messaging prompts</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">DM Quota</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">Unlimited</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Professional
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Meta Graph API v21.0</p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8">
          {[
            { id: 'account', label: 'Professional Account & IDs', icon: '📸' },
            { id: 'webhooks', label: 'Webhooks & Graph Sync', icon: '⚡' },
            { id: 'story_mentions', label: 'Story Mentions & Auto-Reply', icon: '✨' },
            { id: 'icebreakers', label: 'DM Icebreakers & Automations', icon: '💡' },
            { id: 'diagnostics', label: 'Diagnostic Sender & Events', icon: '📡' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`py-4 px-1 inline-flex items-center gap-2 border-b-2 font-medium text-sm transition ${
                activeTab === tab.id
                  ? 'border-pink-600 text-pink-600 font-semibold'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Tab 1: Account Credentials */}
      {activeTab === 'account' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Instagram Professional Credentials</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Connect your Instagram Professional or Business account linked to your Meta Facebook Page.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Inbox Display Name</label>
              <input
                type="text"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. Instagram Direct Support"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Instagram Handle</label>
              <input
                type="text"
                value={igUsername}
                onChange={(e) => setIgUsername(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. @bayworks"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Instagram Business Account ID</label>
              <input
                type="text"
                value={igId}
                onChange={(e) => setIgId(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. 17841405822345678"
              />
              <p className="text-xs text-gray-400 mt-1">Found via Graph API <code className="text-pink-600">GET /me?fields=instagram_business_account</code>.</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Connected Facebook Page ID</label>
              <input
                type="text"
                value={connectedPageId}
                onChange={(e) => setConnectedPageId(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. 100234567890123"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Graph Access Token</label>
              <input
                type="password"
                value={accessToken}
                onChange={(e) => setAccessToken(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-gray-400 mt-1">Requires <code className="text-pink-600">instagram_basic</code> and <code className="text-pink-600">instagram_manage_messages</code> permissions.</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Meta App Secret</label>
              <input
                type="password"
                value={appSecret}
                onChange={(e) => setAppSecret(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Webhooks & Graph Sync */}
      {activeTab === 'webhooks' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Instagram Webhook Settings</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Subscribe your Meta App to Instagram webhooks under <strong>Instagram → Webhooks</strong>.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Webhook Callback URL</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/instagram'}
                  className="w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-lg text-sm font-mono text-gray-600"
                />
                <button
                  onClick={() =>
                    copyToClipboard(
                      data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/instagram',
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
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
                <button
                  onClick={() => copyToClipboard(webhookVerifyToken, 'verifyToken')}
                  className="px-4 py-2 bg-gray-100 hover:bg-gray-200 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 transition"
                >
                  {copiedKey === 'verifyToken' ? '✓ Copied' : 'Copy'}
                </button>
              </div>
            </div>

            <div className="pt-4 border-t border-gray-100">
              <label className="block text-sm font-medium text-gray-700 mb-2">Subscribed Instagram Webhook Fields</label>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {['messages', 'messaging_seen', 'story_insights', 'mentions'].map((field) => (
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

      {/* Tab 3: Story Mentions & Auto-Reply */}
      {activeTab === 'story_mentions' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Story Mentions & Instant DM Responder</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Automatically send a personalized Direct Message to customers whenever they tag your brand in their Instagram Stories.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-gray-100">
            <div className="p-4 bg-gradient-to-r from-pink-50 to-purple-50 border border-pink-100 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Enable Story Mention Auto-Reply</h4>
                  <p className="text-xs text-gray-500">
                    Triggers within seconds when an Instagram user tags @bayworks in a public or mutual story.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={storyMentionsEnabled}
                  onChange={(e) => setStoryMentionsEnabled(e.target.checked)}
                  className="h-4 w-4 text-pink-600 rounded border-gray-300 focus:ring-pink-500"
                />
              </div>

              {storyMentionsEnabled && (
                <div>
                  <label className="block text-xs font-medium text-gray-700 mb-1">Story Mention Direct Message</label>
                  <textarea
                    rows={3}
                    value={storyReplyMessage}
                    onChange={(e) => setStoryReplyMessage(e.target.value)}
                    className="w-full p-2.5 bg-white border border-pink-200 rounded-lg text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                    placeholder="Thanks for mentioning us in your story! 🔥 How can we assist you today?"
                  />
                  <p className="text-xs text-gray-400 mt-1">Personalized, friendly greeting to convert brand mentions into customer relationships.</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Tab 4: Icebreakers & Automations */}
      {activeTab === 'icebreakers' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">DM Icebreakers & Automations</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Icebreakers provide fast one-tap options for users starting a new Instagram Direct Message conversation.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-gray-100">
            {/* Icebreaker List */}
            <div className="space-y-3">
              {icebreakers.map((ib, idx) => (
                <div key={idx} className="flex items-center justify-between p-3.5 bg-gray-50 border border-gray-200 rounded-xl">
                  <div className="flex items-center gap-3">
                    <span className="w-6 h-6 rounded-full bg-pink-100 text-pink-700 font-bold text-xs flex items-center justify-center">
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

            {/* Add Icebreaker */}
            <div className="p-4 bg-pink-50 border border-pink-100 rounded-xl space-y-3">
              <h4 className="text-xs font-semibold text-pink-900 uppercase tracking-wider">Add New DM Icebreaker</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <input
                  type="text"
                  placeholder="Question text (e.g. Can I request a quote?)"
                  value={newQuestion}
                  onChange={(e) => setNewQuestion(e.target.value)}
                  className="px-3 py-2 border border-pink-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-pink-500"
                />
                <input
                  type="text"
                  placeholder="Custom Payload (e.g. QUOTE_REQUEST)"
                  value={newPayload}
                  onChange={(e) => setNewPayload(e.target.value)}
                  className="px-3 py-2 border border-pink-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-pink-500 font-mono"
                />
              </div>
              <button
                onClick={handleAddIcebreaker}
                disabled={!newQuestion.trim()}
                className="px-4 py-1.5 text-xs font-semibold text-white bg-pink-600 rounded-lg hover:bg-pink-700 disabled:opacity-50 transition"
              >
                + Add DM Icebreaker
              </button>
            </div>

            {/* Welcome Greeting */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Welcome DM Greeting</h4>
                  <p className="text-xs text-gray-500">Sent immediately upon receiving a customer's first inbound DM.</p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-4 w-4 text-pink-600 rounded border-gray-300 focus:ring-pink-500"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">After-Hours Away Message</h4>
                  <p className="text-xs text-gray-500">Sent to DMs received outside scheduled operating hours.</p>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="h-4 w-4 text-pink-600 rounded border-gray-300 focus:ring-pink-500"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
              )}
            </div>

            {/* CSAT Rating */}
            <div className="flex items-center justify-between p-4 bg-gray-50 border border-gray-200 rounded-xl">
              <div>
                <h4 className="text-sm font-semibold text-gray-900">CSAT Customer Satisfaction Rating</h4>
                <p className="text-xs text-gray-500">Prompts for rating when conversation is marked resolved in Shared Inbox.</p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-4 w-4 text-pink-600 rounded border-gray-300 focus:ring-pink-500"
              />
            </div>
          </div>
        </div>
      )}

      {/* Tab 5: Diagnostic Sender & Events */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-4">
            <div>
              <h3 className="text-base font-semibold text-gray-900">Live Diagnostic Direct Message Dispatcher</h3>
              <p className="text-sm text-gray-500 mt-0.5">
                Dispatch a test outbound Direct Message to an Instagram-Scoped ID (IGSID) to verify Meta Graph API connectivity.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient IGSID</label>
                <input
                  type="text"
                  value={testIgsid}
                  onChange={(e) => setTestIgsid(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                  placeholder="e.g. igsid_test_549120"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient Username</label>
                <input
                  type="text"
                  value={testUsername}
                  onChange={(e) => setTestUsername(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                  placeholder="e.g. clara_design"
                />
              </div>

              <div className="md:col-span-2">
                <label className="block text-xs font-medium text-gray-700 mb-1">Test Message Body</label>
                <textarea
                  rows={2}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={handleSendTestMessage}
                disabled={sendingTest || !testIgsid.trim() || !testMessage.trim()}
                className="px-4 py-2 text-sm font-medium text-white bg-gradient-to-r from-pink-600 to-purple-600 rounded-lg hover:from-pink-700 hover:to-purple-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
              >
                {sendingTest ? 'Dispatching...' : '🚀 Send Diagnostic DM'}
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
                <h3 className="text-base font-semibold text-gray-900">Recent Instagram Direct Events</h3>
                <p className="text-sm text-gray-500">Live feed of inbound DMs, Story mention triggers, and agent responses.</p>
              </div>
              <button
                onClick={async () => setEvents(await api.listInstagramEvents())}
                className="px-3 py-1.5 text-xs font-medium text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
              >
                🔄 Refresh
              </button>
            </div>

            {events.length === 0 ? (
              <div className="py-12 text-center text-sm text-gray-500">
                No recent Instagram Direct events recorded yet. Dispatch a diagnostic DM above to verify.
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
                                ? 'bg-pink-50 text-pink-700 border border-pink-200'
                                : 'bg-purple-50 text-purple-700 border border-purple-200'
                            }`}
                          >
                            {ev.direction === 'inbound' ? 'Inbound DM' : 'Outbound DM'}
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
