import { AlertCircle, CheckCircle2, Key, Activity, ArrowLeft, Check, RefreshCw, Send, Sparkles } from '../components/icons'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type SlackChannelData,
  type SocialEvent,
} from '../lib/api'
import { SlackIcon } from '../components/icons'

export function SlackIntegration() {
  const [data, setData] = useState<SlackChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<'credentials' | 'settings' | 'commands' | 'diagnostics'>('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const [inboxName, setInboxName] = useState('Slack Support')
  const [botToken, setBotToken] = useState('')
  const [appId, setAppId] = useState('')
  const [signingSecret, setSigningSecret] = useState('')

  const [autoRespond, setAutoRespond] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState('Hello! How can we help you today?')
  const [awayMessage, setAwayMessage] = useState('We are currently away. We will respond when we return.')

  const [testChannelId, setTestChannelId] = useState('')
  const [testMessage, setTestMessage] = useState('Hello! This is a test message from your Slack bot.')
  const [testResult, setTestResult] = useState<{ timestamp: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getSlackChannel(),
        api.listSlackEvents().catch(() => []),
      ])
      setData(channelData)
      setEvents(evList)
      setInboxName(channelData.inbox?.name || 'Slack Support')
      setAppId(channelData.botProfile?.appId || '')
      setSigningSecret(channelData.webhookConfig?.signingSecret || '')
      setAutoRespond(channelData.settings?.autoRespond ?? true)
      setGreetingMessage(channelData.settings?.greetingMessage || greetingMessage)
      setAwayMessage(channelData.settings?.awayMessage || awayMessage)
    } catch (err: any) {
      setError(err.message || 'Failed to load Slack channel configuration.')
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveSettings() {
    try {
      setSaving(true)
      setError(null)
      setSuccess(null)
      await api.updateSlackChannel({
        name: inboxName,
        app_id: appId,
        signing_secret: signingSecret,
        settings: { autoRespond, greetingMessage, awayMessage },
      })
      setSuccess('Slack settings saved successfully.')
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
      const result = await api.verifySlackConnection()
      if (result.verified) {
        setSuccess(result.message || 'Slack bot connection verified!')
      } else {
        setError(result.message || 'Verification failed.')
      }
      await loadChannel()
    } catch (err: any) {
      setError(err.message || 'Could not verify Slack connection.')
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
      const result = await api.sendSlackTestMessage({ channelId: testChannelId, message: testMessage })
      if (result.sent) {
        setTestResult({ timestamp: result.timestamp || '', message: 'Test message sent successfully!' })
      }
    } catch (err: any) {
      setError(err.message || 'Failed to send test message.')
    } finally {
      setSendingTest(false)
    }
  }

  if (loading) {
    return <div className="p-8 text-center text-muted-foreground">Loading Slack configuration...</div>
  }

  const health = data?.health
  const isConnected = health?.isConnected ?? false

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3">
        <Link to="/channels" className="text-muted-foreground hover:text-foreground">
          <ArrowLeft className="w-5 h-5" />
        </Link>
        <SlackIcon className="w-8 h-8" />
        <h1 className="text-2xl font-bold">Slack Integration</h1>
        {isConnected && <CheckCircle2 className="w-5 h-5 text-green-500 ml-auto" />}
      </div>

      {success && (
        <div className="p-4 rounded-lg bg-green-50 border border-green-200 text-green-800 flex items-center gap-2">
          <Check className="w-5 h-5" />
          {success}
        </div>
      )}
      {error && (
        <div className="p-4 rounded-lg bg-red-50 border border-red-200 text-red-800 flex items-center gap-2">
          <AlertCircle className="w-5 h-5" />
          {error}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 border-b">
        {(['credentials', 'settings', 'commands', 'diagnostics'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium capitalize rounded-t-lg ${
              activeTab === tab
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Credentials Tab */}
      {activeTab === 'credentials' && (
        <div className="space-y-6">
          <div className="p-6 rounded-lg border space-y-4">
            <h2 className="text-lg font-semibold flex items-center gap-2">
              <Key className="w-5 h-5" /> Bot Credentials
            </h2>
            <p className="text-sm text-muted-foreground">
              Create a Slack App at <a href="https://api.slack.com/apps" target="_blank" rel="noreferrer" className="underline">api.slack.com/apps</a>, then add Bot Token Scopes (chat:write, channels:read, groups:read) and install to your workspace.
            </p>
            <div>
              <label className="block text-sm font-medium mb-1">Inbox Name</label>
              <input
                className="w-full px-3 py-2 rounded-md border bg-background"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Bot Token (xoxb-...)</label>
              <input
                type="password"
                className="w-full px-3 py-2 rounded-md border bg-background"
                placeholder="xoxb-..."
                value={botToken}
                onChange={(e) => setBotToken(e.target.value)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">App ID</label>
              <input
                className="w-full px-3 py-2 rounded-md border bg-background"
                placeholder="A0..."
                value={appId}
                onChange={(e) => setAppId(e.target.value)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Signing Secret</label>
              <input
                type="password"
                className="w-full px-3 py-2 rounded-md border bg-background"
                placeholder="xoxb-..."
                value={signingSecret}
                onChange={(e) => setSigningSecret(e.target.value)}
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleSaveSettings}
                disabled={saving}
                className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
              >
                {saving ? 'Saving...' : 'Save Credentials'}
              </button>
              <button
                onClick={handleVerify}
                disabled={verifying}
                className="px-4 py-2 border rounded-md hover:bg-muted disabled:opacity-50 flex items-center gap-2"
              >
                <RefreshCw className={`w-4 h-4 ${verifying ? 'animate-spin' : ''}`} />
                {verifying ? 'Verifying...' : 'Verify Connection'}
              </button>
            </div>
          </div>

          {/* Connection Status */}
          {health && (
            <div className="p-6 rounded-lg border">
              <h2 className="text-lg font-semibold mb-4">Connection Status</h2>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Status:</span>
                  <span className={`ml-2 font-medium ${health.isConnected ? 'text-green-600' : 'text-yellow-600'}`}>
                    {health.status}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Bot User ID:</span>
                  <span className="ml-2 font-mono">{health.botUserId || '—'}</span>
                </div>
                <div className="col-span-2">
                  <span className="text-muted-foreground">Details:</span>
                  <span className="ml-2">{health.statusLabel}</span>
                </div>
                {health.missingCredentials.length > 0 && (
                  <div className="col-span-2 text-red-600">
                    Missing: {health.missingCredentials.join(', ')}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && (
        <div className="p-6 rounded-lg border space-y-4">
          <h2 className="text-lg font-semibold">Bot Settings</h2>
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Auto-respond</div>
              <div className="text-sm text-muted-foreground">Automatically respond to incoming messages</div>
            </div>
            <button
              onClick={() => setAutoRespond(!autoRespond)}
              className={`w-11 h-6 rounded-full transition-colors ${autoRespond ? 'bg-primary' : 'bg-muted'}`}
            >
              <div className={`w-5 h-5 bg-white rounded-full transition-transform ${autoRespond ? 'translate-x-5' : 'translate-x-0.5'}`} />
            </button>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Greeting Message</label>
            <input
              className="w-full px-3 py-2 rounded-md border bg-background"
              value={greetingMessage}
              onChange={(e) => setGreetingMessage(e.target.value)}
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Away Message</label>
            <input
              className="w-full px-3 py-2 rounded-md border bg-background"
              value={awayMessage}
              onChange={(e) => setAwayMessage(e.target.value)}
            />
          </div>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      )}

      {/* Commands Tab */}
      {activeTab === 'commands' && (
        <div className="p-6 rounded-lg border space-y-4">
          <h2 className="text-lg font-semibold">Slash Commands</h2>
          {data?.commands?.length ? (
            <div className="space-y-2">
              {data.commands.map((cmd) => (
                <div key={cmd.id} className="flex items-center justify-between p-3 rounded-md border">
                  <div>
                    <div className="font-mono text-sm">{cmd.name}</div>
                    <div className="text-sm text-muted-foreground">{cmd.description}</div>
                  </div>
                  <span className={`text-xs px-2 py-1 rounded-full ${cmd.isActive ? 'bg-green-100 text-green-700' : 'bg-muted text-muted-foreground'}`}>
                    {cmd.isActive ? 'Active' : 'Inactive'}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground">No commands configured.</p>
          )}
        </div>
      )}

      {/* Diagnostics Tab */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          <div className="p-6 rounded-lg border space-y-4">
            <h2 className="text-lg font-semibold flex items-center gap-2">
              <Sparkles className="w-5 h-5" /> Test Message
            </h2>
            <div>
              <label className="block text-sm font-medium mb-1">Channel ID</label>
              <input
                className="w-full px-3 py-2 rounded-md border bg-background"
                placeholder="C0123456789"
                value={testChannelId}
                onChange={(e) => setTestChannelId(e.target.value)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Message</label>
              <textarea
                className="w-full px-3 py-2 rounded-md border bg-background"
                rows={3}
                value={testMessage}
                onChange={(e) => setTestMessage(e.target.value)}
              />
            </div>
            <button
              onClick={handleSendTest}
              disabled={sendingTest}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50 flex items-center gap-2"
            >
              <Send className="w-4 h-4" />
              {sendingTest ? 'Sending...' : 'Send Test Message'}
            </button>
            {testResult && (
              <div className="p-3 rounded-md bg-green-50 border border-green-200 text-green-800 text-sm">
                {testResult.message}
              </div>
            )}
          </div>

          {/* Recent Events */}
          <div className="p-6 rounded-lg border space-y-4">
            <h2 className="text-lg font-semibold flex items-center gap-2">
              <Activity className="w-5 h-5" /> Recent Events
            </h2>
            {events.length > 0 ? (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {events.slice(0, 10).map((ev, i) => (
                  <div key={i} className="p-3 rounded-md border text-sm">
                    <div className="font-medium">{ev.type || ev.event_type || 'Event'}</div>
                    <div className="text-muted-foreground">{ev.message || ev.content || ''}</div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-muted-foreground text-sm">No recent events.</p>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
