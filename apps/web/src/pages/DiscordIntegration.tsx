import { AlertCircle, CheckCircle2, Key, Bot, Activity, ArrowLeft, Check, RefreshCw, Rocket, Send } from '../components/icons'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type DiscordChannelData,
  type SocialEvent,
} from '../lib/api'
import { DiscordIcon } from '../components/icons'

export function DiscordIntegration() {
  const [data, setData] = useState<DiscordChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<'credentials' | 'settings' | 'commands' | 'diagnostics'>('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const [inboxName, setInboxName] = useState('Discord Support')
  const [botToken, setBotToken] = useState('')
  const [clientId, setClientId] = useState('')
  const [interactionSecret, setInteractionSecret] = useState('')

  const [autoRespond, setAutoRespond] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState('Hello! How can we help you today?')
  const [awayMessage, setAwayMessage] = useState('We are currently away. We will respond when we return.')

  const [testChannelId, setTestChannelId] = useState('')
  const [testMessage, setTestMessage] = useState('Hello! This is a test message from your Discord bot.')
  const [testResult, setTestResult] = useState<{ messageId: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getDiscordChannel(),
        api.listDiscordEvents().catch(() => []),
      ])
      setData(channelData)
      setEvents(evList)
      setInboxName(channelData.inbox?.name || 'Discord Support')
      setClientId(channelData.botProfile?.clientId || '')
      setInteractionSecret(channelData.webhookConfig?.interactionSecret || '')
      setAutoRespond(channelData.settings?.autoRespond ?? true)
      setGreetingMessage(channelData.settings?.greetingMessage || greetingMessage)
      setAwayMessage(channelData.settings?.awayMessage || awayMessage)
    } catch (err: any) {
      setError(err.message || 'Failed to load Discord channel configuration.')
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveSettings() {
    try {
      setSaving(true)
      setError(null)
      setSuccess(null)
      await api.updateDiscordChannel({
        name: inboxName,
        client_id: clientId,
        interaction_secret: interactionSecret,
        settings: { autoRespond, greetingMessage, awayMessage },
      })
      setSuccess('Discord settings saved successfully.')
      await loadChannel()
    } catch (err: any) {
      setError(err.message || 'Failed to save settings.')
    } finally {
      setSaving(false)
    }
  }

  async function handleVerify() {
    try {
      setVerifying(true)
      setError(null)
      const result = await api.verifyDiscordConnection()
      if (result.verified) {
        setSuccess(result.message || 'Discord bot connection verified!')
      } else {
        setError(result.message || 'Verification failed.')
      }
      await loadChannel()
    } catch (err: any) {
      setError(err.message || 'Could not verify Discord connection.')
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTest() {
    if (!testChannelId) {
      setError('Channel ID is required.')
      return
    }
    try {
      setSendingTest(true)
      setError(null)
      const result = await api.sendDiscordTestMessage({ channelId: testChannelId, message: testMessage })
      if (result.sent) {
        setTestResult({ messageId: result.messageId || '', message: 'Test message sent successfully!' })
      }
    } catch (err: any) {
      setError(err.message || 'Failed to send test message.')
    } finally {
      setSendingTest(false)
    }
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-slate-950 flex items-center justify-center">
        <div className="text-slate-400">Loading Discord configuration...</div>
      </div>
    )
  }

  const connected = data?.health?.isConnected ?? false
  const status = data?.health?.status ?? 'NOT_CONFIGURED'

  return (
    <div className="min-h-screen bg-slate-950 p-6">
      <div className="max-w-5xl mx-auto">
        <Link to="/channels" className="inline-flex items-center gap-2 text-slate-400 hover:text-white mb-6">
          <ArrowLeft className="w-4 h-4" /> Back to Channels
        </Link>

        <div className="flex items-center gap-4 mb-8">
          <div className="w-12 h-12 rounded-xl bg-indigo-600/20 flex items-center justify-center">
            <DiscordIcon className="w-7 h-7 text-indigo-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">Discord Integration</h1>
            <p className="text-slate-400">Connect your Discord bot for two-way messaging</p>
          </div>
          <div className={`ml-auto px-3 py-1 rounded-full text-sm font-medium ${connected ? 'bg-emerald-500/20 text-emerald-400' : 'bg-amber-500/20 text-amber-400'}`}>
            {connected ? 'Connected' : status === 'NOT_CONFIGURED' ? 'Not Configured' : 'Unverified'}
          </div>
        </div>

        {success && (
          <div className="mb-6 p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg flex items-center gap-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400" />
            <span className="text-emerald-300">{success}</span>
          </div>
        )}
        {error && (
          <div className="mb-6 p-4 bg-red-500/10 border border-red-500/30 rounded-lg flex items-center gap-3">
            <AlertCircle className="w-5 h-5 text-red-400" />
            <span className="text-red-300">{error}</span>
          </div>
        )}

        <div className="flex gap-1 mb-6 bg-slate-900 rounded-lg p-1">
          {(['credentials', 'settings', 'commands', 'diagnostics'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors capitalize ${
                activeTab === tab ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-white'
              }`}
            >
              {tab}
            </button>
          ))}
        </div>

        {activeTab === 'credentials' && (
          <div className="space-y-6">
            <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Key className="w-5 h-5 text-indigo-400" /> Bot Credentials
              </h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-slate-400 mb-1">Inbox Name</label>
                  <input
                    value={inboxName}
                    onChange={(e) => setInboxName(e.target.value)}
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
                  />
                </div>
                <div>
                  <label className="block text-sm text-slate-400 mb-1">Bot Token</label>
                  <input
                    type="password"
                    value={botToken}
                    onChange={(e) => setBotToken(e.target.value)}
                    placeholder="Enter your Discord bot token"
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
                  />
                </div>
                <div>
                  <label className="block text-sm text-slate-400 mb-1">Client ID</label>
                  <input
                    value={clientId}
                    onChange={(e) => setClientId(e.target.value)}
                    placeholder="Your Discord application client ID"
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
                  />
                </div>
                <div>
                  <label className="block text-sm text-slate-400 mb-1">Interaction Secret</label>
                  <input
                    value={interactionSecret}
                    onChange={(e) => setInteractionSecret(e.target.value)}
                    placeholder="Public key bytes (base64) for interaction verification"
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
                  />
                </div>
              </div>
            </div>

            {data?.botProfile && (
              <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
                <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                  <Bot className="w-5 h-5 text-indigo-400" /> Bot Profile
                </h2>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <span className="text-sm text-slate-400">Username</span>
                    <p className="text-white">{data.botProfile.botUsername || '—'}</p>
                  </div>
                  <div>
                    <span className="text-sm text-slate-400">Bot ID</span>
                    <p className="text-white font-mono text-sm">{data.botProfile.botId || '—'}</p>
                  </div>
                  <div>
                    <span className="text-sm text-slate-400">Client ID</span>
                    <p className="text-white font-mono text-sm">{data.botProfile.clientId || '—'}</p>
                  </div>
                  <div>
                    <span className="text-sm text-slate-400">Source</span>
                    <p className="text-white">{data.botProfile.source}</p>
                  </div>
                </div>
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={handleSaveSettings}
                disabled={saving}
                className="px-6 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium disabled:opacity-50"
              >
                {saving ? 'Saving...' : 'Save Settings'}
              </button>
              <button
                onClick={handleVerify}
                disabled={verifying}
                className="px-6 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-lg font-medium disabled:opacity-50 flex items-center gap-2"
              >
                <RefreshCw className={`w-4 h-4 ${verifying ? 'animate-spin' : ''}`} />
                {verifying ? 'Verifying...' : 'Verify Connection'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="bg-slate-900 rounded-xl p-6 border border-slate-800 space-y-4">
            <h2 className="text-lg font-semibold text-white mb-4">Bot Settings</h2>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-white font-medium">Auto Respond</p>
                <p className="text-sm text-slate-400">Automatically respond to incoming messages</p>
              </div>
              <button
                onClick={() => setAutoRespond(!autoRespond)}
                className={`w-12 h-6 rounded-full transition-colors ${autoRespond ? 'bg-indigo-600' : 'bg-slate-700'}`}
              >
                <div className={`w-5 h-5 bg-white rounded-full transition-transform ${autoRespond ? 'translate-x-6' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Greeting Message</label>
              <textarea
                value={greetingMessage}
                onChange={(e) => setGreetingMessage(e.target.value)}
                rows={2}
                className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
              />
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Away Message</label>
              <textarea
                value={awayMessage}
                onChange={(e) => setAwayMessage(e.target.value)}
                rows={2}
                className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
              />
            </div>
            <button
              onClick={handleSaveSettings}
              disabled={saving}
              className="px-6 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium disabled:opacity-50"
            >
              {saving ? 'Saving...' : 'Save Settings'}
            </button>
          </div>
        )}

        {activeTab === 'commands' && (
          <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
            <h2 className="text-lg font-semibold text-white mb-4">Bot Commands</h2>
            {data?.commands?.length ? (
              <div className="space-y-3">
                {data.commands.map((cmd) => (
                  <div key={cmd.id} className="flex items-center justify-between p-3 bg-slate-800 rounded-lg">
                    <div>
                      <p className="text-white font-mono">/{cmd.name}</p>
                      <p className="text-sm text-slate-400">{cmd.description}</p>
                    </div>
                    <span className={`px-2 py-1 rounded text-xs ${cmd.isActive ? 'bg-emerald-500/20 text-emerald-400' : 'bg-slate-700 text-slate-400'}`}>
                      {cmd.isActive ? 'Active' : 'Inactive'}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-slate-400">No commands configured.</p>
            )}
          </div>
        )}

        {activeTab === 'diagnostics' && (
          <div className="space-y-6">
            <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Activity className="w-5 h-5 text-indigo-400" /> Connection Health
              </h2>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <span className="text-sm text-slate-400">Status</span>
                  <p className={`font-medium ${connected ? 'text-emerald-400' : 'text-amber-400'}`}>{data?.health?.statusLabel || status}</p>
                </div>
                <div>
                  <span className="text-sm text-slate-400">Configured</span>
                  <p className="text-white">{data?.health?.configured ? 'Yes' : 'No'}</p>
                </div>
                <div>
                  <span className="text-sm text-slate-400">Inbox Active</span>
                  <p className="text-white">{data?.health?.inboxActive ? 'Yes' : 'No'}</p>
                </div>
                <div>
                  <span className="text-sm text-slate-400">Webhook URL</span>
                  <p className="text-white text-xs font-mono break-all">{data?.webhookConfig?.callbackUrl || 'Not configured'}</p>
                </div>
              </div>
            </div>

            <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Send className="w-5 h-5 text-indigo-400" /> Send Test Message
              </h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-slate-400 mb-1">Discord Channel ID</label>
                  <input
                    value={testChannelId}
                    onChange={(e) => setTestChannelId(e.target.value)}
                    placeholder="e.g. 123456789012345678"
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
                  />
                </div>
                <div>
                  <label className="block text-sm text-slate-400 mb-1">Message</label>
                  <textarea
                    value={testMessage}
                    onChange={(e) => setTestMessage(e.target.value)}
                    rows={2}
                    className="w-full bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-white"
                  />
                </div>
                <button
                  onClick={handleSendTest}
                  disabled={sendingTest || !testChannelId}
                  className="px-6 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium disabled:opacity-50 flex items-center gap-2"
                >
                  <Rocket className="w-4 h-4" />
                  {sendingTest ? 'Sending...' : 'Send Test Message'}
                </button>
                {testResult && (
                  <div className="p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-lg flex items-center gap-3">
                    <Check className="w-5 h-5 text-emerald-400" />
                    <span className="text-emerald-300">{testResult.message}</span>
                  </div>
                )}
              </div>
            </div>

            {events.length > 0 && (
              <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
                <h2 className="text-lg font-semibold text-white mb-4">Recent Events</h2>
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {events.slice(0, 10).map((ev, i) => (
                    <div key={i} className="p-3 bg-slate-800 rounded-lg">
                      <p className="text-white text-sm">{ev.type || ev.eventType || 'event'}</p>
                      <p className="text-xs text-slate-400">{ev.timestamp || ev.createdAt || ''}</p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
