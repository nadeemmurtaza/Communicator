import { Activity, AlertCircle, AlertTriangle, ArrowLeft, Camera, CheckCircle2, Flame, Webhook, X, Zap } from '../components/icons'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type InstagramChannelData,
  type SocialEvent,
} from '../lib/api'
import { InstagramIcon } from '../components/icons'

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
  const [igUsername, setIgUsername] = useState('')
  const [igId, setIgId] = useState('')
  const [connectedPageId, setConnectedPageId] = useState('')
  const [accessToken, setAccessToken] = useState('')
  const [appSecret, setAppSecret] = useState('')
  const [webhookVerifyToken, setWebhookVerifyToken] = useState('meta_instagram_verify_token')

  // Story Mentions State
  const [storyMentionsEnabled, setStoryMentionsEnabled] = useState(true)
  const [storyReplyMessage, setStoryReplyMessage] = useState(
    'Thanks for mentioning us in your story! How can we assist you today?'
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
    'Hey! Thanks for DMing us on Instagram. How can we help you today?'
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
  const [testUsername, setTestUsername] = useState('test_user')
  const [testMessage, setTestMessage] = useState(
    'Hey there! This is an automated test DM from your Instagram integration.'
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
      const rawHandle = channelData.accountProfile?.igUsername || channelData.inbox.credentials?.igUsername || ''
      setIgUsername(rawHandle === '@bayworks' ? '' : rawHandle)
      const rawIgId = channelData.accountProfile?.igId || channelData.inbox.credentials?.igId || ''
      setIgId(rawIgId === '17841405822345678' ? '' : rawIgId)
      const rawPageId = channelData.accountProfile?.connectedFacebookPage || channelData.inbox.credentials?.connectedPageId || ''
      setConnectedPageId(rawPageId === '100234567890123' ? '' : rawPageId)
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

  async function handleConnectInstagram() {
    try {
      setError(null)
      // A plain <a href> can't send the operator's session header that
      // /instagram/start now requires, so fetch the authorize URL via the
      // authenticated API client and navigate to it ourselves.
      const session = localStorage.getItem('communicator.session')
      const token = session ? JSON.parse(session).token : null
      const res = await fetch('/api/v1/auth/instagram/start', {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      })
      if (!res.ok) throw new Error('Could not start Instagram connection.')
      const { authorizeUrl } = await res.json()
      window.location.href = authorizeUrl
    } catch (err: any) {
      setError(err.message || 'Failed to start Instagram connection.')
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
        <span className="ml-3 text-sm text-[var(--color-ink-faint)] font-medium">Loading Instagram Direct settings...</span>
      </div>
    )
  }

  const isConnected = Boolean(data?.health?.isConnected && igUsername && igUsername !== '@bayworks')

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6">
      {/* Top Banner Navigation */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-[var(--color-border)]">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Link to="/channels" className="text-sm font-medium text-pink-600 hover:text-pink-800">
              <span className="flex items-center gap-1"><ArrowLeft className="h-3.5 w-3.5" /> Back to Channels Hub</span>
            </Link>
            <span className="text-[var(--color-ink-faint)]">/</span>
            <span className="text-sm font-medium text-[var(--color-ink-faint)]">Instagram Direct</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-[var(--radius-lg)] bg-gradient-to-tr from-yellow-500 via-pink-600 to-purple-600 flex items-center justify-center text-white shadow-sm">
              <InstagramIcon className="h-6 w-6" />
            </div>
            <div>
              <div className="flex items-center gap-2">

                <h1 className="text-2xl font-bold text-[var(--color-ink)] tracking-tight">Instagram Direct Integration</h1>

                {isConnected ? (

                  <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800 border border-emerald-200 flex items-center gap-1">

                    <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>

                    Connected &amp; Healthy

                  </span>

                ) : (

                  <span className="rounded-full bg-amber-50 px-2.5 py-0.5 text-xs font-semibold text-amber-800 border border-amber-200 flex items-center gap-1">

                    <span className="h-1.5 w-1.5 rounded-full bg-amber-500"></span>

                    Not Configured

                  </span>

                )}

              </div>
              <p className="text-sm text-[var(--color-ink-faint)]">
                Instagram Professional & Business Direct Messages, Story Mentions Auto-Reply, and Icebreaker FAQ chips.
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={handleVerifyConnection}
            disabled={verifying}
            className="px-4 py-2 text-sm font-medium text-[var(--color-ink)] bg-white border border-[var(--color-border)] rounded-[var(--radius-md)] hover:bg-[var(--color-accent-muted)] shadow-sm transition disabled:opacity-50"
          >
            {verifying ? 'Checking Instagram...' : 'Test Connection'}
          </button>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="px-4 py-2 text-sm font-medium text-white bg-gradient-to-r from-pink-600 to-purple-600 rounded-[var(--radius-md)] hover:from-pink-700 hover:to-purple-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Real Connection Status Banner */}

      {!isConnected && (

        <div className="rounded-[var(--radius-lg)] border border-amber-200 bg-amber-50 p-4 text-amber-900 flex items-start gap-3">

          <AlertTriangle className="h-5 w-5 text-amber-600 shrink-0 mt-0.5" />

          <div>

            <h4 className="text-sm font-semibold text-amber-900">Channel Not Connected</h4>

            <p className="text-xs text-amber-700 mt-0.5">

              Please enter your provider API credentials below and click Save Configuration to activate.

            </p>

          </div>

        </div>

      )}



      {/* Alert Notices */}
      {success && (
        <div className="p-4 bg-emerald-50 border border-emerald-200 rounded-[var(--radius-lg)] text-emerald-800 text-sm flex items-center justify-between">
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-4 w-4 text-emerald-600" />
            <span>{success}</span>
          </div>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-900 font-bold"><X className="h-4 w-4" /></button>
        </div>
      )}

      {error && (
        <div className="p-4 bg-rose-50 border border-rose-200 rounded-[var(--radius-lg)] text-rose-800 text-sm flex items-center justify-between">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4 text-rose-600" />
            <span>{error}</span>
          </div>
          <button onClick={() => setError(null)} className="text-rose-600 hover:text-rose-900 font-bold"><X className="h-4 w-4" /></button>
        </div>
      )}

      {/* KPI & Status Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="p-4 bg-white border border-[var(--color-border)] rounded-[var(--radius-lg)] shadow-sm">
          <div className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase tracking-wider mb-1">Account Handle</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-[var(--color-ink)]">{igUsername || 'Not Configured'}</span>
            <span className={`px-2 py-0.5 text-xs font-semibold rounded-full border ${
              isConnected ? 'bg-emerald-50 text-emerald-700 border-emerald-200' : 'bg-amber-50 text-amber-700 border-amber-200'
            }`}>
              {isConnected ? 'Connected' : 'Not Configured'}
            </span>
          </div>
          <p className="text-xs text-[var(--color-ink-faint)] mt-1">IG ID: {igId}</p>
        </div>

        <div className="p-4 bg-white border border-[var(--color-border)] rounded-[var(--radius-lg)] shadow-sm">
          <div className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase tracking-wider mb-1">Story Mentions</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-[var(--color-ink)]">
              {storyMentionsEnabled ? 'Auto-Reply ON' : 'Disabled'}
            </span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-pink-50 text-pink-700 border border-pink-200">
              Story Sync
            </span>
          </div>
          <p className="text-xs text-[var(--color-ink-faint)] mt-1">Direct instant reply trigger</p>
        </div>

        <div className="p-4 bg-white border border-[var(--color-border)] rounded-[var(--radius-lg)] shadow-sm">
          <div className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase tracking-wider mb-1">Icebreakers</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-[var(--color-ink)]">{icebreakers.length} Active</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-purple-50 text-purple-700 border border-purple-200">
              FAQ Chips
            </span>
          </div>
          <p className="text-xs text-[var(--color-ink-faint)] mt-1">Direct messaging prompts</p>
        </div>

        <div className="p-4 bg-white border border-[var(--color-border)] rounded-[var(--radius-lg)] shadow-sm">
          <div className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase tracking-wider mb-1">DM Quota</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-[var(--color-ink)]">Unlimited</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Professional
            </span>
          </div>
          <p className="text-xs text-[var(--color-ink-faint)] mt-1">Meta Graph API v21.0</p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="border-b border-[var(--color-border)]">
        <nav className="flex space-x-8">
          {[
            { id: 'account', label: 'Professional Account & IDs', icon: Camera },
            { id: 'webhooks', label: 'Webhooks & Graph Sync', icon: Webhook },
            { id: 'story_mentions', label: 'Story Mentions & Auto-Reply', icon: Flame },
            { id: 'icebreakers', label: 'DM Icebreakers & Automations', icon: Zap },
            { id: 'diagnostics', label: 'Diagnostic Sender & Events', icon: Activity },
          ].map((tab) => {
            const IconComp = tab.icon
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`py-4 px-1 inline-flex items-center gap-2 border-b-2 font-medium text-sm transition ${
                  activeTab === tab.id
                    ? 'border-pink-600 text-pink-600 font-semibold'
                    : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)] hover:border-[var(--color-border)]'
                }`}
              >
                <IconComp className="w-4 h-4" />
                <span>{tab.label}</span>
              </button>
            )
          })}
        </nav>
      </div>

      {/* Tab 1: Account Credentials */}
      {activeTab === 'account' && (
        <div className="bg-white rounded-[var(--radius-lg)] border border-[var(--color-border)] p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-[var(--color-ink)]">Instagram Professional Credentials</h3>
            <p className="text-sm text-[var(--color-ink-faint)] mt-0.5">
              Connect your Instagram Professional or Business account linked to your Meta Facebook Page.
            </p>
          </div>

          <div className="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-4">
            {data?.health.accountConnected ? (
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-full bg-gradient-to-tr from-pink-500 to-amber-400 flex items-center justify-center text-white text-sm font-semibold">
                  {(data.accountProfile.igUsername || '?').replace('@', '').slice(0, 1).toUpperCase()}
                </div>
                <div>
                  <p className="text-sm font-medium text-[var(--color-ink)]">{data.accountProfile.igUsername}</p>
                  <p className="text-xs text-[var(--color-ink-faint)] font-mono">{data.accountProfile.igId}</p>
                </div>
              </div>
            ) : (
              <p className="text-sm text-[var(--color-ink-faint)]">No Instagram account connected yet.</p>
            )}
            <button
              onClick={handleConnectInstagram}
              className="px-4 py-2 bg-gradient-to-tr from-pink-500 to-amber-400 text-white text-sm font-medium rounded-[var(--radius-md)] hover:opacity-90 transition"
            >
              {data?.health.accountConnected ? 'Reconnect Instagram Account' : 'Connect Instagram Account'}
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-[var(--color-border)]">
            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Inbox Display Name</label>
              <input
                type="text"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
                className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. Instagram Direct Support"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Instagram Handle</label>
              <input
                type="text"
                value={igUsername}
                onChange={(e) => setIgUsername(e.target.value)}
                className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. @your_instagram_handle"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Instagram Business Account ID</label>
              <input
                type="text"
                value={igId}
                onChange={(e) => setIgId(e.target.value)}
                className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. 17841405822345678"
              />
              <p className="text-xs text-[var(--color-ink-faint)] mt-1">Found via Graph API <code className="text-pink-600">GET /me?fields=instagram_business_account</code>.</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Connected Facebook Page ID</label>
              <input
                type="text"
                value={connectedPageId}
                onChange={(e) => setConnectedPageId(e.target.value)}
                className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="e.g. 100234567890123"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Graph Access Token</label>
              <input
                type="password"
                value={accessToken}
                onChange={(e) => setAccessToken(e.target.value)}
                className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-[var(--color-ink-faint)] mt-1">Requires <code className="text-pink-600">instagram_basic</code> and <code className="text-pink-600">instagram_manage_messages</code> permissions.</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Meta App Secret</label>
              <input
                type="password"
                value={appSecret}
                onChange={(e) => setAppSecret(e.target.value)}
                className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Webhooks & Graph Sync */}
      {activeTab === 'webhooks' && (
        <div className="bg-white rounded-[var(--radius-lg)] border border-[var(--color-border)] p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-[var(--color-ink)]">Instagram Webhook Settings</h3>
            <p className="text-sm text-[var(--color-ink-faint)] mt-0.5">
              Subscribe your Meta App to Instagram webhooks under <strong>Instagram &gt; Webhooks</strong>.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-[var(--color-border)]">
            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Webhook Callback URL</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/instagram'}
                  className="w-full px-3 py-2 bg-[var(--color-accent-muted)] border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono text-[var(--color-ink-muted)]"
                />
                <button
                  onClick={() =>
                    copyToClipboard(
                      data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/instagram',
                      'callback'
                    )
                  }
                  className="px-4 py-2 bg-[var(--color-accent-muted)] hover:bg-[var(--color-border)] border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-medium text-[var(--color-ink)] transition"
                >
                  {copiedKey === 'callback' ? 'Copied' : 'Copy'}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Verify Token</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={webhookVerifyToken}
                  onChange={(e) => setWebhookVerifyToken(e.target.value)}
                  className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
                <button
                  onClick={() => copyToClipboard(webhookVerifyToken, 'verifyToken')}
                  className="px-4 py-2 bg-[var(--color-accent-muted)] hover:bg-[var(--color-border)] border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-medium text-[var(--color-ink)] transition"
                >
                  {copiedKey === 'verifyToken' ? 'Copied' : 'Copy'}
                </button>
              </div>
            </div>

            <div className="pt-4 border-t border-[var(--color-border)]">
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-2">Subscribed Instagram Webhook Fields</label>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {['messages', 'messaging_seen', 'story_insights', 'mentions'].map((field) => (
                  <div key={field} className="p-3 bg-[var(--color-accent-muted)] border border-[var(--color-border)] rounded-[var(--radius-md)] flex items-center gap-2">
                    <span className="text-emerald-500"></span>
                    <span className="text-xs font-mono font-medium text-[var(--color-ink)]">{field}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Story Mentions & Auto-Reply */}
      {activeTab === 'story_mentions' && (
        <div className="bg-white rounded-[var(--radius-lg)] border border-[var(--color-border)] p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-[var(--color-ink)]">Story Mentions & Instant DM Responder</h3>
            <p className="text-sm text-[var(--color-ink-faint)] mt-0.5">
              Automatically send a personalized Direct Message to customers whenever they tag your brand in their Instagram Stories.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-[var(--color-border)]">
            <div className="p-4 bg-gradient-to-r from-pink-50 to-purple-50 border border-pink-100 rounded-[var(--radius-lg)] space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-[var(--color-ink)]">Enable Story Mention Auto-Reply</h4>
                  <p className="text-xs text-[var(--color-ink-faint)]">
                    Triggers within seconds when an Instagram user tags your account in a public or mutual story.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={storyMentionsEnabled}
                  onChange={(e) => setStoryMentionsEnabled(e.target.checked)}
                  className="h-4 w-4 text-pink-600 rounded border-[var(--color-border)] focus:ring-pink-500"
                />
              </div>

              {storyMentionsEnabled && (
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">Story Mention Direct Message</label>
                  <textarea
                    rows={3}
                    value={storyReplyMessage}
                    onChange={(e) => setStoryReplyMessage(e.target.value)}
                    className="w-full p-2.5 bg-white border border-pink-200 rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                    placeholder="Thanks for mentioning us in your story! How can we assist you today?"
                  />
                  <p className="text-xs text-[var(--color-ink-faint)] mt-1">Personalized, friendly greeting to convert brand mentions into customer relationships.</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Tab 4: Icebreakers & Automations */}
      {activeTab === 'icebreakers' && (
        <div className="bg-white rounded-[var(--radius-lg)] border border-[var(--color-border)] p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-[var(--color-ink)]">DM Icebreakers & Automations</h3>
            <p className="text-sm text-[var(--color-ink-faint)] mt-0.5">
              Icebreakers provide fast one-tap options for users starting a new Instagram Direct Message conversation.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-[var(--color-border)]">
            {/* Icebreaker List */}
            <div className="space-y-3">
              {icebreakers.map((ib, idx) => (
                <div key={idx} className="flex items-center justify-between p-3.5 bg-[var(--color-accent-muted)] border border-[var(--color-border)] rounded-[var(--radius-lg)]">
                  <div className="flex items-center gap-3">
                    <span className="w-6 h-6 rounded-full bg-pink-100 text-pink-700 font-bold text-xs flex items-center justify-center">
                      {idx + 1}
                    </span>
                    <div>
                      <div className="text-sm font-medium text-[var(--color-ink)]">{ib.question}</div>
                      <div className="text-xs font-mono text-[var(--color-ink-faint)]">Payload: {ib.payload}</div>
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
            <div className="p-4 bg-pink-50 border border-pink-100 rounded-[var(--radius-lg)] space-y-3">
              <h4 className="text-xs font-semibold text-pink-900 uppercase tracking-wider">Add New DM Icebreaker</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <input
                  type="text"
                  placeholder="Question text (e.g. Can I request a quote?)"
                  value={newQuestion}
                  onChange={(e) => setNewQuestion(e.target.value)}
                  className="px-3 py-2 border border-pink-200 rounded-[var(--radius-md)] text-sm bg-white focus:outline-none focus:ring-2 focus:ring-pink-500"
                />
                <input
                  type="text"
                  placeholder="Custom Payload (e.g. QUOTE_REQUEST)"
                  value={newPayload}
                  onChange={(e) => setNewPayload(e.target.value)}
                  className="px-3 py-2 border border-pink-200 rounded-[var(--radius-md)] text-sm bg-white focus:outline-none focus:ring-2 focus:ring-pink-500 font-mono"
                />
              </div>
              <button
                onClick={handleAddIcebreaker}
                disabled={!newQuestion.trim()}
                className="px-4 py-1.5 text-xs font-semibold text-white bg-pink-600 rounded-[var(--radius-md)] hover:bg-pink-700 disabled:opacity-50 transition"
              >
                + Add DM Icebreaker
              </button>
            </div>

            {/* Welcome Greeting */}
            <div className="p-4 bg-[var(--color-accent-muted)] border border-[var(--color-border)] rounded-[var(--radius-lg)] space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-[var(--color-ink)]">Welcome DM Greeting</h4>
                  <p className="text-xs text-[var(--color-ink-faint)]">Sent immediately upon receiving a customer's first inbound DM.</p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-4 w-4 text-pink-600 rounded border-[var(--color-border)] focus:ring-pink-500"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="p-4 bg-[var(--color-accent-muted)] border border-[var(--color-border)] rounded-[var(--radius-lg)] space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-[var(--color-ink)]">After-Hours Away Message</h4>
                  <p className="text-xs text-[var(--color-ink-faint)]">Sent to DMs received outside scheduled operating hours.</p>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="h-4 w-4 text-pink-600 rounded border-[var(--color-border)] focus:ring-pink-500"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
              )}
            </div>

            {/* CSAT Rating */}
            <div className="flex items-center justify-between p-4 bg-[var(--color-accent-muted)] border border-[var(--color-border)] rounded-[var(--radius-lg)]">
              <div>
                <h4 className="text-sm font-semibold text-[var(--color-ink)]">CSAT Customer Satisfaction Rating</h4>
                <p className="text-xs text-[var(--color-ink-faint)]">Prompts for rating when conversation is marked resolved in Shared Inbox.</p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-4 w-4 text-pink-600 rounded border-[var(--color-border)] focus:ring-pink-500"
              />
            </div>
          </div>
        </div>
      )}

      {/* Tab 5: Diagnostic Sender & Events */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          <div className="bg-white rounded-[var(--radius-lg)] border border-[var(--color-border)] p-6 shadow-sm space-y-4">
            <div>
              <h3 className="text-base font-semibold text-[var(--color-ink)]">Live Diagnostic Direct Message Dispatcher</h3>
              <p className="text-sm text-[var(--color-ink-faint)] mt-0.5">
                Dispatch a test outbound Direct Message to an Instagram-Scoped ID (IGSID) to verify Meta Graph API connectivity.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">Recipient IGSID</label>
                <input
                  type="text"
                  value={testIgsid}
                  onChange={(e) => setTestIgsid(e.target.value)}
                  className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm font-mono focus:ring-2 focus:ring-pink-500 focus:outline-none"
                  placeholder="e.g. igsid_test_549120"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">Recipient Username</label>
                <input
                  type="text"
                  value={testUsername}
                  onChange={(e) => setTestUsername(e.target.value)}
                  className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                  placeholder="e.g. clara_design"
                />
              </div>

              <div className="md:col-span-2">
                <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">Test Message Body</label>
                <textarea
                  rows={2}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full px-3 py-2 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-pink-500 focus:outline-none"
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={handleSendTestMessage}
                disabled={sendingTest || !testIgsid.trim() || !testMessage.trim()}
                className="px-4 py-2 text-sm font-medium text-white bg-gradient-to-r from-pink-600 to-purple-600 rounded-[var(--radius-md)] hover:from-pink-700 hover:to-purple-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
              >
                {sendingTest ? 'Dispatching...' : 'Send Diagnostic DM'}
              </button>

              {testResult && (
                <span className="text-xs font-mono text-emerald-700 bg-emerald-50 px-3 py-1 rounded-full border border-emerald-200">
                  ID: {testResult.mid}
                </span>
              )}
            </div>
          </div>

          {/* Live Events Table */}
          <div className="bg-white rounded-[var(--radius-lg)] border border-[var(--color-border)] p-6 shadow-sm space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-base font-semibold text-[var(--color-ink)]">Recent Instagram Direct Events</h3>
                <p className="text-sm text-[var(--color-ink-faint)]">Live feed of inbound DMs, Story mention triggers, and agent responses.</p>
              </div>
              <button
                onClick={async () => setEvents(await api.listInstagramEvents())}
                className="px-3 py-1.5 text-xs font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] bg-[var(--color-accent-muted)] hover:bg-[var(--color-border)] rounded-[var(--radius-md)] transition"
              >
                Refresh
              </button>
            </div>

            {events.length === 0 ? (
              <div className="py-12 text-center text-sm text-[var(--color-ink-faint)]">
                No recent Instagram Direct events recorded yet. Dispatch a diagnostic DM above to verify.
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm text-[var(--color-ink-muted)]">
                  <thead className="bg-[var(--color-accent-muted)] text-xs font-semibold text-[var(--color-ink-faint)] uppercase tracking-wider border-b border-[var(--color-border)]">
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
                      <tr key={ev.id} className="hover:bg-[var(--color-accent-muted)]/60 transition">
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
                        <td className="py-3 px-4 font-medium text-[var(--color-ink)]">{ev.senderName}</td>
                        <td className="py-3 px-4 text-[var(--color-ink)] max-w-xs truncate">{ev.body}</td>
                        <td className="py-3 px-4">
                          <span className="inline-flex items-center gap-1 text-xs font-medium text-emerald-600"><span className="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>{ev.status}</span>
                        </td>
                        <td className="py-3 px-4 font-mono text-xs text-[var(--color-ink-faint)]">
                          {ev.providerMessageId ? ev.providerMessageId.slice(0, 16) + '...' : '—'}
                        </td>
                        <td className="py-3 px-4 text-xs text-[var(--color-ink-faint)]">
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
