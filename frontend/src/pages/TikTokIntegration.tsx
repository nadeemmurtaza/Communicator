import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type TikTokChannelData,
  type SocialEvent,
} from '../lib/api'

export function TikTokIntegration() {
  const [data, setData] = useState<TikTokChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'credentials' | 'webhooks' | 'comments' | 'automations' | 'diagnostics'
  >('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTestMsg, setSendingTestMsg] = useState(false)
  const [sendingTestComment, setSendingTestComment] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Credentials State
  const [inboxName, setInboxName] = useState('TikTok DMs & Comments')
  const [appId, setAppId] = useState('tt_app_mock_6789')
  const [appSecret, setAppSecret] = useState('')
  const [accessToken, setAccessToken] = useState('')
  const [businessAccountId, setBusinessAccountId] = useState('tt_biz_acc_1001')
  const [webhookVerifyToken, setWebhookVerifyToken] = useState('tiktok_verify_token_xyz')

  // Comments Auto-Reply State
  const [commentAutoReplyEnabled, setCommentAutoReplyEnabled] = useState(true)
  const [commentAutoReplyText, setCommentAutoReplyText] = useState(
    'Thanks for your comment! Send us a direct message for personal support. ✨'
  )

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState(
    '👋 Hi there! Thanks for reaching out on TikTok. How can our support team assist you today?'
  )
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState(
    "We're currently offline on TikTok. We will respond as soon as our team is back!"
  )
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('UTC')

  // Diagnostics & Simulator State
  const [testOpenId, setTestOpenId] = useState('tt_user_open_id_1001')
  const [testRecipientName, setTestRecipientName] = useState('TikTok User')
  const [testMessage, setTestMessage] = useState(
    'Hello! This is an automated diagnostic test message from your Bay Works TikTok integration.'
  )
  const [testMsgResult, setTestMsgResult] = useState<{ messageId: string; message: string } | null>(null)

  const [testVideoId, setTestVideoId] = useState('v_789456123')
  const [testCommentId, setTestCommentId] = useState('c_123456789')
  const [testCommentReply, setTestCommentReply] = useState(
    'Thanks for watching! Feel free to DM our support team with any questions.'
  )
  const [testCommentResult, setTestCommentResult] = useState<{ replyId: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getTikTokChannel(),
        api.listTikTokEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setAppId(channelData.businessProfile?.appId || 'tt_app_mock_6789')
      setBusinessAccountId(channelData.businessProfile?.businessAccountId || 'tt_biz_acc_1001')
      setWebhookVerifyToken(channelData.webhookConfig?.verifyToken || 'tiktok_verify_token_xyz')

      if (channelData.commentAutoReply) {
        setCommentAutoReplyEnabled(channelData.commentAutoReply.enabled)
        setCommentAutoReplyText(channelData.commentAutoReply.template)
      }

      setGreetingEnabled(channelData.inbox.greetingEnabled)
      setGreetingMessage(channelData.inbox.greetingMessage)
      setOutOfOfficeEnabled(channelData.inbox.outOfOfficeEnabled)
      setOutOfOfficeMessage(channelData.inbox.outOfOfficeMessage)
      setCsatEnabled(channelData.inbox.csatEnabled)
      setWorkingHoursEnabled(channelData.inbox.workingHoursEnabled)
      setWorkingHoursStart(channelData.inbox.workingHoursStart || 9)
      setWorkingHoursEnd(channelData.inbox.workingHoursEnd || 18)
      setTimezone(channelData.inbox.timezone || 'UTC')
    } catch (err: any) {
      setError(err.message || 'Failed to load TikTok channel configuration.')
    } finally {
      setLoading(false)
    }
  }

  async function handleSave() {
    try {
      setSaving(true)
      setError(null)
      setSuccess(null)

      const updated = await api.updateTikTokChannel({
        name: inboxName,
        app_id: appId,
        app_secret: appSecret,
        access_token: accessToken,
        business_account_id: businessAccountId,
        webhook_verify_token: webhookVerifyToken,
        comment_auto_reply_enabled: commentAutoReplyEnabled,
        comment_auto_reply_text: commentAutoReplyText,
        greeting_enabled: greetingEnabled,
        greeting_message: greetingMessage,
        out_of_office_enabled: outOfOfficeEnabled,
        out_of_office_message: outOfOfficeMessage,
        csat_enabled: csatEnabled,
        working_hours_enabled: workingHoursEnabled,
        working_hours_start: workingHoursStart,
        working_hours_end: workingHoursEnd,
        timezone,
      })

      setData(updated)
      setSuccess('TikTok integration settings saved successfully.')
      setAppSecret('')
      setAccessToken('')
    } catch (err: any) {
      setError(err.message || 'Failed to update TikTok channel settings.')
    } finally {
      setSaving(false)
    }
  }

  async function handleVerify() {
    try {
      setVerifying(true)
      setError(null)
      setSuccess(null)

      const result = await api.verifyTikTokConnection()
      setSuccess(`Verified: ${result.message}`)
      await loadChannel()
    } catch (err: any) {
      setError(err.message || 'Failed to verify TikTok Business connection.')
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTestMessage() {
    try {
      setSendingTestMsg(true)
      setError(null)
      setSuccess(null)
      setTestMsgResult(null)

      const res = await api.sendTikTokTestMessage({
        recipient_open_id: testOpenId,
        recipient_name: testRecipientName,
        message: testMessage,
      })

      setTestMsgResult(res)
      setSuccess('Diagnostic direct message dispatched successfully.')
      const evList = await api.listTikTokEvents().catch(() => [])
      setEvents(evList)
    } catch (err: any) {
      setError(err.message || 'Failed to dispatch test direct message.')
    } finally {
      setSendingTestMsg(false)
    }
  }

  async function handleSendTestCommentReply() {
    try {
      setSendingTestComment(true)
      setError(null)
      setSuccess(null)
      setTestCommentResult(null)

      const res = await api.sendTikTokTestCommentReply({
        comment_id: testCommentId,
        video_id: testVideoId,
        recipient_open_id: testOpenId,
        recipient_name: testRecipientName,
        reply_text: testCommentReply,
      })

      setTestCommentResult(res)
      setSuccess('Diagnostic comment reply posted successfully.')
      const evList = await api.listTikTokEvents().catch(() => [])
      setEvents(evList)
    } catch (err: any) {
      setError(err.message || 'Failed to post test comment reply.')
    } finally {
      setSendingTestComment(false)
    }
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="flex items-center gap-2 text-slate-500">
          <div className="h-5 w-5 animate-spin rounded-full border-2 border-slate-300 border-t-slate-600" />
          <span>Loading TikTok integration...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 text-xs text-slate-500 mb-1">
            <Link to="/channels" className="hover:text-slate-700">
              ← Back to Channels Hub
            </Link>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-3xl">🎵</span>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-xl font-bold text-slate-900">TikTok Business Integration</h1>
                <span className="rounded-full bg-emerald-100 border border-emerald-200 px-2.5 py-0.5 text-xs font-semibold text-emerald-800">
                  {data?.health?.status || 'Connected'}
                </span>
              </div>
              <p className="text-xs text-slate-500 mt-0.5">
                Official TikTok for Business API v2 · Direct Messages, Video Comment Moderation &amp; Auto-Replies
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleVerify}
            disabled={verifying}
            className="rounded-md border border-slate-200 bg-white px-3.5 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs flex items-center gap-1.5"
          >
            {verifying ? 'Verifying...' : '⚡ Test Connection'}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-md bg-slate-900 px-4 py-2 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs flex items-center gap-1.5"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>

      {/* Alert Messages */}
      {success && (
        <div className="rounded-lg bg-emerald-50 border border-emerald-200 p-3.5 text-xs text-emerald-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span>✅</span>
            <span>{success}</span>
          </div>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-900">
            ✕
          </button>
        </div>
      )}

      {error && (
        <div className="rounded-lg bg-rose-50 border border-rose-200 p-3.5 text-xs text-rose-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span>⚠️</span>
            <span>{error}</span>
          </div>
          <button onClick={() => setError(null)} className="text-rose-600 hover:text-rose-900">
            ✕
          </button>
        </div>
      )}

      {/* Tabs */}
      <div className="border-b border-slate-200">
        <nav className="flex space-x-6">
          <button
            onClick={() => setActiveTab('credentials')}
            className={`pb-3 text-xs font-semibold border-b-2 transition-colors ${
              activeTab === 'credentials'
                ? 'border-slate-900 text-slate-900'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            🔑 Credentials &amp; Account
          </button>
          <button
            onClick={() => setActiveTab('webhooks')}
            className={`pb-3 text-xs font-semibold border-b-2 transition-colors ${
              activeTab === 'webhooks'
                ? 'border-slate-900 text-slate-900'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            📡 Webhooks &amp; Security
          </button>
          <button
            onClick={() => setActiveTab('comments')}
            className={`pb-3 text-xs font-semibold border-b-2 transition-colors ${
              activeTab === 'comments'
                ? 'border-slate-900 text-slate-900'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            💬 Video Comment Auto-Replies
          </button>
          <button
            onClick={() => setActiveTab('automations')}
            className={`pb-3 text-xs font-semibold border-b-2 transition-colors ${
              activeTab === 'automations'
                ? 'border-slate-900 text-slate-900'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            ⚙️ Automations &amp; Hours
          </button>
          <button
            onClick={() => setActiveTab('diagnostics')}
            className={`pb-3 text-xs font-semibold border-b-2 transition-colors ${
              activeTab === 'diagnostics'
                ? 'border-slate-900 text-slate-900'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            🧪 Diagnostics &amp; Live Events
          </button>
        </nav>
      </div>

      {/* Tab 1: Credentials */}
      {activeTab === 'credentials' && (
        <div className="space-y-5">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-slate-900">TikTok for Business API Configuration</h2>
            <p className="text-xs text-slate-500">
              Obtain your App ID and App Secret from the TikTok Developer Portal to synchronize direct messages and video comments with the omnichannel inbox.
            </p>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">Inbox Name</label>
                <input
                  type="text"
                  value={inboxName}
                  onChange={(e) => setInboxName(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">Business Account ID</label>
                <input
                  type="text"
                  value={businessAccountId}
                  onChange={(e) => setBusinessAccountId(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">TikTok App ID</label>
                <input
                  type="text"
                  value={appId}
                  onChange={(e) => setAppId(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900 font-mono"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">
                  App Secret {data?.inbox?.credentials?.appSecret && <span className="text-emerald-600 font-normal">(Configured)</span>}
                </label>
                <input
                  type="password"
                  value={appSecret}
                  onChange={(e) => setAppSecret(e.target.value)}
                  placeholder="••••••••••••••••"
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                />
              </div>

              <div className="sm:col-span-2">
                <label className="block text-xs font-medium text-slate-700 mb-1">
                  Access Token {data?.inbox?.credentials?.accessToken && <span className="text-emerald-600 font-normal">(Configured)</span>}
                </label>
                <input
                  type="password"
                  value={accessToken}
                  onChange={(e) => setAccessToken(e.target.value)}
                  placeholder="Paste permanent or long-lived TikTok Access Token"
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900 font-mono"
                />
              </div>
            </div>
          </div>

          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-3">
            <h3 className="text-xs font-bold uppercase tracking-wider text-slate-700">Connected Account Profile</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-1">
              <div className="rounded-lg bg-slate-50 p-3 border border-slate-100">
                <span className="text-[11px] text-slate-500 block">Account Handle</span>
                <span className="text-sm font-semibold text-slate-900">{data?.businessProfile?.accountHandle || '@bayworks_official'}</span>
              </div>
              <div className="rounded-lg bg-slate-50 p-3 border border-slate-100">
                <span className="text-[11px] text-slate-500 block">Verification Status</span>
                <span className="text-sm font-semibold text-emerald-700">{data?.businessProfile?.accountType || 'TikTok for Business Verified'}</span>
              </div>
              <div className="rounded-lg bg-slate-50 p-3 border border-slate-100">
                <span className="text-[11px] text-slate-500 block">API Version</span>
                <span className="text-sm font-semibold text-slate-900">{data?.health?.tiktokApi || 'open.tiktokapis.com/v2'}</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Webhooks & Security */}
      {activeTab === 'webhooks' && (
        <div className="space-y-5">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-slate-900">TikTok Webhook Subscription</h2>
            <p className="text-xs text-slate-500">
              Configure this webhook URL inside your TikTok Developer App settings. TikTok will deliver incoming direct messages and comment notifications in real-time.
            </p>

            <div className="space-y-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">Callback URL</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    readOnly
                    value={data?.webhookConfig?.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/tiktok'}
                    className="w-full rounded-md border border-slate-300 bg-slate-50 px-3 py-2 text-xs text-slate-600 font-mono"
                  />
                  <button
                    onClick={() =>
                      copyToClipboard(
                        data?.webhookConfig?.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/tiktok',
                        'webhook'
                      )
                    }
                    className="rounded-md border border-slate-200 bg-white px-3 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs"
                  >
                    {copiedKey === 'webhook' ? 'Copied! ✓' : 'Copy'}
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">Webhook Verify Token</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={webhookVerifyToken}
                    onChange={(e) => setWebhookVerifyToken(e.target.value)}
                    className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs font-mono"
                  />
                  <button
                    onClick={() => copyToClipboard(webhookVerifyToken, 'token')}
                    className="rounded-md border border-slate-200 bg-white px-3 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs"
                  >
                    {copiedKey === 'token' ? 'Copied! ✓' : 'Copy'}
                  </button>
                </div>
              </div>

              <div className="rounded-lg bg-slate-50 border border-slate-200 p-4 space-y-2">
                <span className="text-xs font-bold text-slate-800">Supported Real-Time Events</span>
                <ul className="text-xs text-slate-600 space-y-1 list-disc list-inside">
                  <li><code className="font-mono bg-white px-1.5 py-0.5 rounded border text-slate-700">im.direct_message.receive</code> – Incoming customer DMs</li>
                  <li><code className="font-mono bg-white px-1.5 py-0.5 rounded border text-slate-700">video.comment.receive</code> – Customer comments on brand videos</li>
                  <li><code className="font-mono bg-white px-1.5 py-0.5 rounded border text-slate-700">video.comment.reply</code> – Agent comment responses</li>
                </ul>
                <div className="pt-2 text-[11px] text-slate-500">
                  🛡️ <strong>HMAC-SHA256 Security:</strong> Incoming webhook payloads are automatically verified against the <code className="font-mono bg-white px-1 py-0.5 rounded border">X-TikTok-Signature</code> header using your configured App Secret.
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Comments Auto-Reply */}
      {activeTab === 'comments' && (
        <div className="space-y-5">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
            <div className="flex items-start justify-between">
              <div>
                <h2 className="text-sm font-bold text-slate-900">TikTok Video Comment Moderation &amp; Auto-Replies</h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Engage commenters instantly on viral videos and convert viewers into support conversations.
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={commentAutoReplyEnabled}
                  onChange={(e) => setCommentAutoReplyEnabled(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-slate-900"></div>
              </label>
            </div>

            <div className="space-y-3 pt-2">
              <div>
                <label className="block text-xs font-medium text-slate-700 mb-1">
                  Automated Comment Response Template
                </label>
                <textarea
                  rows={3}
                  value={commentAutoReplyText}
                  onChange={(e) => setCommentAutoReplyText(e.target.value)}
                  disabled={!commentAutoReplyEnabled}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900 disabled:bg-slate-50 disabled:text-slate-400"
                />
                <p className="text-[11px] text-slate-500 mt-1">
                  Sent automatically when customers ask questions on your public TikTok videos.
                </p>
              </div>

              <div className="rounded-lg bg-amber-50 border border-amber-200 p-3.5 text-xs text-amber-900">
                💡 <strong>Inbox Integration:</strong> In addition to auto-replies, all comments appear in your team inbox as threaded conversations, allowing agents to respond directly from the message composer.
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 4: Automations & Hours */}
      {activeTab === 'automations' && (
        <div className="space-y-5">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-slate-900">Automated Direct Messaging &amp; Greetings</h2>

            {/* Welcome Greeting */}
            <div className="border-b border-slate-100 pb-4 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-xs font-bold text-slate-800">Welcome Greeting</h3>
                  <p className="text-[11px] text-slate-500">Send an instant greeting when a customer initiates a new TikTok DM.</p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-slate-900 focus:ring-slate-900"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="border-b border-slate-100 pb-4 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-xs font-bold text-slate-800">Out-of-Office Away Message</h3>
                  <p className="text-[11px] text-slate-500">Respond automatically when messages arrive outside working hours.</p>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-slate-900 focus:ring-slate-900"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs"
                />
              )}
            </div>

            {/* CSAT Survey */}
            <div className="border-b border-slate-100 pb-4 flex items-center justify-between">
              <div>
                <h3 className="text-xs font-bold text-slate-800">CSAT Customer Satisfaction Rating</h3>
                <p className="text-[11px] text-slate-500">Send a 1–5 star rating request after a ticket is marked resolved.</p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="rounded border-slate-300 text-slate-900 focus:ring-slate-900"
              />
            </div>

            {/* Working Hours */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-xs font-bold text-slate-800">Operating Schedule &amp; Working Hours</h3>
                  <p className="text-[11px] text-slate-500">Enforce business hours for agent routing and away messages.</p>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-slate-900 focus:ring-slate-900"
                />
              </div>

              {workingHoursEnabled && (
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-2">
                  <div>
                    <label className="block text-[11px] text-slate-600 mb-1">Start Hour</label>
                    <input
                      type="number"
                      min={0}
                      max={23}
                      value={workingHoursStart}
                      onChange={(e) => setWorkingHoursStart(parseInt(e.target.value) || 0)}
                      className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
                    />
                  </div>
                  <div>
                    <label className="block text-[11px] text-slate-600 mb-1">End Hour</label>
                    <input
                      type="number"
                      min={0}
                      max={23}
                      value={workingHoursEnd}
                      onChange={(e) => setWorkingHoursEnd(parseInt(e.target.value) || 0)}
                      className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
                    />
                  </div>
                  <div>
                    <label className="block text-[11px] text-slate-600 mb-1">Timezone</label>
                    <input
                      type="text"
                      value={timezone}
                      onChange={(e) => setTimezone(e.target.value)}
                      className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono"
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Tab 5: Diagnostics & Live Events */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Direct Message Simulator */}
            <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-xs space-y-3">
              <h3 className="text-xs font-bold uppercase tracking-wider text-slate-800">
                1. Direct Message Simulator
              </h3>
              <p className="text-xs text-slate-500">
                Dispatch an outbound test direct message to verify API token authentication.
              </p>

              <div>
                <label className="block text-[11px] text-slate-600 mb-1">Recipient Open ID</label>
                <input
                  type="text"
                  value={testOpenId}
                  onChange={(e) => setTestOpenId(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono"
                />
              </div>

              <div>
                <label className="block text-[11px] text-slate-600 mb-1">Recipient Name</label>
                <input
                  type="text"
                  value={testRecipientName}
                  onChange={(e) => setTestRecipientName(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
                />
              </div>

              <div>
                <label className="block text-[11px] text-slate-600 mb-1">Message Body</label>
                <textarea
                  rows={2}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
                />
              </div>

              <button
                onClick={handleSendTestMessage}
                disabled={sendingTestMsg}
                className="w-full rounded-md bg-slate-900 py-2 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs"
              >
                {sendingTestMsg ? 'Dispatching...' : 'Send Test Direct Message'}
              </button>

              {testMsgResult && (
                <div className="rounded-md bg-slate-50 border border-slate-200 p-2.5 text-[11px] font-mono text-slate-700">
                  <div><strong>Status:</strong> {testMsgResult.messageId}</div>
                  <div className="text-emerald-700">{testMsgResult.message}</div>
                </div>
              )}
            </div>

            {/* Video Comment Reply Simulator */}
            <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-xs space-y-3">
              <h3 className="text-xs font-bold uppercase tracking-wider text-slate-800">
                2. Video Comment Reply Simulator
              </h3>
              <p className="text-xs text-slate-500">
                Post an outbound reply to a customer comment on a brand video.
              </p>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-[11px] text-slate-600 mb-1">Video ID</label>
                  <input
                    type="text"
                    value={testVideoId}
                    onChange={(e) => setTestVideoId(e.target.value)}
                    className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono"
                  />
                </div>
                <div>
                  <label className="block text-[11px] text-slate-600 mb-1">Comment ID</label>
                  <input
                    type="text"
                    value={testCommentId}
                    onChange={(e) => setTestCommentId(e.target.value)}
                    className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono"
                  />
                </div>
              </div>

              <div>
                <label className="block text-[11px] text-slate-600 mb-1">Reply Text</label>
                <textarea
                  rows={2}
                  value={testCommentReply}
                  onChange={(e) => setTestCommentReply(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
                />
              </div>

              <button
                onClick={handleSendTestCommentReply}
                disabled={sendingTestComment}
                className="w-full rounded-md bg-slate-900 py-2 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs"
              >
                {sendingTestComment ? 'Posting Reply...' : 'Post Test Comment Reply'}
              </button>

              {testCommentResult && (
                <div className="rounded-md bg-slate-50 border border-slate-200 p-2.5 text-[11px] font-mono text-slate-700">
                  <div><strong>Reply ID:</strong> {testCommentResult.replyId}</div>
                  <div className="text-emerald-700">{testCommentResult.message}</div>
                </div>
              )}
            </div>
          </div>

          {/* Live Events Stream */}
          <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-xs space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-bold uppercase tracking-wider text-slate-800">
                Recent TikTok Transmission Events
              </h3>
              <button
                onClick={loadChannel}
                className="text-xs text-slate-500 hover:text-slate-800 font-medium"
              >
                ↻ Refresh Stream
              </button>
            </div>

            {events.length === 0 ? (
              <p className="text-xs text-slate-400 py-4 text-center">
                No recent TikTok messages or comment events recorded. Use the simulators above or send a message via webhook.
              </p>
            ) : (
              <div className="divide-y divide-slate-100">
                {events.map((ev) => (
                  <div key={ev.id} className="py-2.5 flex items-center justify-between text-xs">
                    <div className="flex items-center gap-2">
                      <span
                        className={`rounded px-1.5 py-0.5 text-[10px] font-semibold ${
                          ev.direction === 'inbound'
                            ? 'bg-sky-100 text-sky-800'
                            : 'bg-emerald-100 text-emerald-800'
                        }`}
                      >
                        {ev.direction.toUpperCase()}
                      </span>
                      <span className="font-medium text-slate-800">{ev.senderName}</span>
                      <span className="text-slate-500 truncate max-w-md">{ev.body}</span>
                    </div>
                    <span className="text-[11px] text-slate-400 font-mono">
                      {new Date(ev.sentAt).toLocaleTimeString()}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
