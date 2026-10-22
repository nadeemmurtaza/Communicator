import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type TelegramChannelData,
  type SocialEvent,
} from '../lib/api'

export function TelegramIntegration() {
  const [data, setData] = useState<TelegramChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'credentials' | 'webhooks' | 'commands' | 'automations' | 'diagnostics'
  >('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Credentials State
  const [inboxName, setInboxName] = useState('Telegram Support')
  const [botToken, setBotToken] = useState('')
  const [botUsername, setBotUsername] = useState('@bayworks_support_bot')
  const [secretToken, setSecretToken] = useState('tg_sec_token_xyz')

  // Bot Commands State
  const [commands, setCommands] = useState<Array<{ command: string; description: string }>>([
    { command: '/start', description: 'Start conversation and show options' },
    { command: '/quote', description: 'Request an instant project quote' },
    { command: '/agent', description: 'Speak with a live support agent' },
  ])
  const [newCommand, setNewCommand] = useState('')
  const [newDescription, setNewDescription] = useState('')

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState(
    '👋 Hello! Welcome to Bay Works on Telegram. How can our team assist you today?'
  )
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState(
    'Thanks for reaching out! Our team is currently outside working hours, but we will reply as soon as we reopen.'
  )
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('America/New_York')

  // Diagnostic Test Sender State
  const [testChatId, setTestChatId] = useState('987654321')
  const [testRecipientName, setTestRecipientName] = useState('Dmitri Rostov')
  const [testMessage, setTestMessage] = useState(
    'Hello! This is a test message dispatched via your Bay Works Telegram Bot API integration.'
  )
  const [testResult, setTestResult] = useState<{ messageId: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getTelegramChannel(),
        api.listTelegramEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setBotUsername(channelData.botProfile?.botUsername || channelData.inbox.credentials?.botUsername || '@bayworks_support_bot')
      setSecretToken(channelData.webhookConfig?.secretToken || 'tg_sec_token_xyz')

      if (channelData.commands && channelData.commands.length > 0) {
        setCommands(channelData.commands)
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
      setError(err.message || 'Failed to load Telegram channel configuration.')
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
        bot_username: botUsername,
        secret_token: secretToken,
        commands,
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

      if (botToken) payload.bot_token = botToken

      const res = await api.updateTelegramChannel(payload)
      setData(res)
      setSuccess('Telegram Bot configuration saved and synchronized successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Failed to save Telegram channel settings.')
    } finally {
      setSaving(false)
    }
  }

  async function handleVerifyConnection() {
    try {
      setVerifying(true)
      setError(null)
      const res = await api.verifyTelegramConnection()
      setSuccess(res.message || 'Telegram Bot connection verified successfully!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (err: any) {
      setError(err.message || 'Telegram Bot verification failed.')
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTestMessage() {
    try {
      setSendingTest(true)
      setError(null)
      setTestResult(null)

      const res = await api.sendTelegramTestMessage({
        recipient_chat_id: testChatId,
        recipient_name: testRecipientName,
        message: testMessage,
      })

      setTestResult({
        messageId: res.messageId,
        message: res.message,
      })
      setSuccess(`Diagnostic message dispatched successfully! Message ID: ${res.messageId}`)

      // Refresh event log
      const updatedEvents = await api.listTelegramEvents()
      setEvents(updatedEvents)
    } catch (err: any) {
      setError(err.message || 'Failed to dispatch test Telegram message.')
    } finally {
      setSendingTest(false)
    }
  }

  function handleAddCommand() {
    if (!newCommand.trim() || !newDescription.trim()) return
    const cmd = newCommand.trim().startsWith('/') ? newCommand.trim() : `/${newCommand.trim()}`
    setCommands([...commands, { command: cmd, description: newDescription.trim() }])
    setNewCommand('')
    setNewDescription('')
  }

  function handleRemoveCommand(index: number) {
    setCommands(commands.filter((_, i) => i !== index))
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2500)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[500px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-sky-500"></div>
        <span className="ml-3 text-sm text-gray-500 font-medium">Loading Telegram configuration...</span>
      </div>
    )
  }

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-6">
      {/* Top Banner Navigation */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-gray-200">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Link to="/channels" className="text-sm font-medium text-sky-600 hover:text-sky-800">
              ← Back to Channels Hub
            </Link>
            <span className="text-gray-300">/</span>
            <span className="text-sm font-medium text-gray-500">Telegram Bot</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-sky-400 to-blue-600 flex items-center justify-center text-white shadow-sm font-bold text-xl">
              ✈️
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 tracking-tight">Telegram Bot Integration</h1>
              <p className="text-sm text-gray-500">
                Official Telegram Bot API connector with webhook streaming, secret token authentication, and interactive bot commands.
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
            {verifying ? 'Checking Telegram...' : '⚡ Test Connection'}
          </button>
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="px-4 py-2 text-sm font-medium text-white bg-sky-600 rounded-lg hover:bg-sky-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
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
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Bot Username</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{botUsername || '@bayworks_support_bot'}</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              Active
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Direct bot handle</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Webhook Security</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">Secret Token</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-sky-50 text-sky-700 border border-sky-200">
              Header Verified
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">X-Telegram-Bot-Api-Secret-Token</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">Bot Commands</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">{commands.length} Commands</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-purple-50 text-purple-700 border border-purple-200">
              Menu Active
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Interactive Telegram menu</p>
        </div>

        <div className="p-4 bg-white border border-gray-200 rounded-xl shadow-sm">
          <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">API Throughput</div>
          <div className="flex items-center justify-between">
            <span className="text-lg font-bold text-gray-900">30 msgs/sec</span>
            <span className="px-2 py-0.5 text-xs font-semibold rounded-full bg-emerald-50 text-emerald-700 border border-emerald-200">
              High
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Official Telegram Bot API</p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8">
          {[
            { id: 'credentials', label: 'Bot Credentials & Setup', icon: '🔑' },
            { id: 'webhooks', label: 'Webhooks & Secret Token', icon: '⚡' },
            { id: 'commands', label: 'Bot Menu & Commands', icon: '💡' },
            { id: 'automations', label: 'Automations & Schedule', icon: '🤖' },
            { id: 'diagnostics', label: 'Diagnostic Sender & Events', icon: '📡' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`py-4 px-1 inline-flex items-center gap-2 border-b-2 font-medium text-sm transition ${
                activeTab === tab.id
                  ? 'border-sky-600 text-sky-600 font-semibold'
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
            <h3 className="text-base font-semibold text-gray-900">Telegram Bot Credentials</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Obtain your Bot Token from Telegram's official <strong>@BotFather</strong> by creating a new bot.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Inbox Display Name</label>
              <input
                type="text"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-sky-500 focus:outline-none"
                placeholder="e.g. Telegram Support"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Telegram Bot Username</label>
              <input
                type="text"
                value={botUsername}
                onChange={(e) => setBotUsername(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-sky-500 focus:outline-none"
                placeholder="e.g. @bayworks_support_bot"
              />
              <p className="text-xs text-gray-400 mt-1">Must end in 'bot' as required by Telegram.</p>
            </div>

            <div className="md:col-span-2">
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Telegram Bot Token
              </label>
              <input
                type="password"
                value={botToken}
                onChange={(e) => setBotToken(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-sky-500 focus:outline-none"
                placeholder="••••••••••••••••••••••••"
              />
              <p className="text-xs text-gray-400 mt-1">
                Format: <code className="text-sky-600">123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ</code>. Leave blank to keep existing.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: Webhooks & Secret Token */}
      {activeTab === 'webhooks' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Telegram Webhook Configuration</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Set this webhook URL via Telegram's <code className="text-sky-600">setWebhook</code> API method or our automatic connector.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Webhook Endpoint URL</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/telegram'}
                  className="w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-lg text-sm font-mono text-gray-600"
                />
                <button
                  onClick={() =>
                    copyToClipboard(
                      data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/telegram',
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
              <label className="block text-sm font-medium text-gray-700 mb-1">Webhook Secret Token</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={secretToken}
                  onChange={(e) => setSecretToken(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-sky-500 focus:outline-none"
                />
                <button
                  onClick={() => copyToClipboard(secretToken, 'secretToken')}
                  className="px-4 py-2 bg-gray-100 hover:bg-gray-200 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 transition"
                >
                  {copiedKey === 'secretToken' ? '✓ Copied' : 'Copy'}
                </button>
              </div>
              <p className="text-xs text-gray-400 mt-1">
                Passed in <code className="text-sky-600">X-Telegram-Bot-Api-Secret-Token</code> to prevent webhook spoofing.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Bot Commands & Menus */}
      {activeTab === 'commands' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Telegram Bot Menu Commands</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Commands registered here appear in the Telegram command auto-complete menu when users type '/' or tap the Menu button.
            </p>
          </div>

          <div className="space-y-4 pt-4 border-t border-gray-100">
            <div className="space-y-3">
              {commands.map((cmd, idx) => (
                <div key={idx} className="flex items-center justify-between p-3.5 bg-gray-50 border border-gray-200 rounded-xl">
                  <div className="flex items-center gap-3">
                    <span className="w-8 h-8 rounded-lg bg-sky-100 text-sky-700 font-mono font-bold text-xs flex items-center justify-center">
                      /
                    </span>
                    <div>
                      <div className="text-sm font-bold font-mono text-gray-900">{cmd.command}</div>
                      <div className="text-xs text-gray-500">{cmd.description}</div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleRemoveCommand(idx)}
                    className="text-sm text-rose-600 hover:text-rose-800 font-medium px-2 py-1 rounded hover:bg-rose-50 transition"
                  >
                    Delete
                  </button>
                </div>
              ))}
            </div>

            <div className="p-4 bg-sky-50 border border-sky-100 rounded-xl space-y-3">
              <h4 className="text-xs font-semibold text-sky-900 uppercase tracking-wider">Register New Bot Command</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <input
                  type="text"
                  placeholder="Command (e.g. /pricing)"
                  value={newCommand}
                  onChange={(e) => setNewCommand(e.target.value)}
                  className="px-3 py-2 border border-sky-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-sky-500 font-mono"
                />
                <input
                  type="text"
                  placeholder="Description (e.g. View enterprise pricing tiers)"
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                  className="px-3 py-2 border border-sky-200 rounded-lg text-sm bg-white focus:outline-none focus:ring-2 focus:ring-sky-500"
                />
              </div>
              <button
                onClick={handleAddCommand}
                disabled={!newCommand.trim() || !newDescription.trim()}
                className="px-4 py-1.5 text-xs font-semibold text-white bg-sky-600 rounded-lg hover:bg-sky-700 disabled:opacity-50 transition"
              >
                + Add Command to Menu
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tab 4: Automations & Schedule */}
      {activeTab === 'automations' && (
        <div className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm space-y-6">
          <div>
            <h3 className="text-base font-semibold text-gray-900">Telegram Automations & Working Hours</h3>
            <p className="text-sm text-gray-500 mt-0.5">
              Automated greetings, after-hours responses, and CSAT customer satisfaction surveys.
            </p>
          </div>

          <div className="space-y-6 pt-4 border-t border-gray-100">
            {/* Welcome Greeting */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Welcome Greeting</h4>
                  <p className="text-xs text-gray-500">Sent immediately when a user sends /start or their first message.</p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-4 w-4 text-sky-600 rounded border-gray-300 focus:ring-sky-500"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-sky-500 focus:outline-none"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">After-Hours Away Message</h4>
                  <p className="text-xs text-gray-500">Sent to messages received outside scheduled support hours.</p>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="h-4 w-4 text-sky-600 rounded border-gray-300 focus:ring-sky-500"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full p-2.5 bg-white border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-sky-500 focus:outline-none"
                />
              )}
            </div>

            {/* CSAT Survey */}
            <div className="flex items-center justify-between p-4 bg-gray-50 border border-gray-200 rounded-xl">
              <div>
                <h4 className="text-sm font-semibold text-gray-900">CSAT Customer Rating</h4>
                <p className="text-xs text-gray-500">Sends 1-5 inline rating buttons when conversation is marked resolved.</p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-4 w-4 text-sky-600 rounded border-gray-300 focus:ring-sky-500"
              />
            </div>

            {/* Working Hours Schedule */}
            <div className="p-4 bg-gray-50 border border-gray-200 rounded-xl space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-gray-900">Working Hours Schedule</h4>
                  <p className="text-xs text-gray-500">Enforce team availability hours and automate away responses.</p>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="h-4 w-4 text-sky-600 rounded border-gray-300 focus:ring-sky-500"
                />
              </div>

              {workingHoursEnabled && (
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2">
                  <div>
                    <label className="block text-xs font-medium text-gray-700 mb-1">Start Hour (24h)</label>
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
                    <label className="block text-xs font-medium text-gray-700 mb-1">End Hour (24h)</label>
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
              <h3 className="text-base font-semibold text-gray-900">Live Diagnostic Message Dispatcher</h3>
              <p className="text-sm text-gray-500 mt-0.5">
                Dispatch a test outbound message to a Telegram Chat ID to verify Bot API connectivity and delivery.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient Chat ID</label>
                <input
                  type="text"
                  value={testChatId}
                  onChange={(e) => setTestChatId(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-sky-500 focus:outline-none"
                  placeholder="e.g. 987654321"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-700 mb-1">Recipient Name</label>
                <input
                  type="text"
                  value={testRecipientName}
                  onChange={(e) => setTestRecipientName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-sky-500 focus:outline-none"
                  placeholder="e.g. Dmitri Rostov"
                />
              </div>

              <div className="md:col-span-2">
                <label className="block text-xs font-medium text-gray-700 mb-1">Test Message Body</label>
                <textarea
                  rows={2}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-sky-500 focus:outline-none"
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-2">
              <button
                onClick={handleSendTestMessage}
                disabled={sendingTest || !testChatId.trim() || !testMessage.trim()}
                className="px-4 py-2 text-sm font-medium text-white bg-sky-600 rounded-lg hover:bg-sky-700 shadow-sm transition disabled:opacity-50 flex items-center gap-2"
              >
                {sendingTest ? 'Dispatching...' : '🚀 Send Diagnostic Message'}
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
                <h3 className="text-base font-semibold text-gray-900">Recent Telegram Message Events</h3>
                <p className="text-sm text-gray-500">Live feed of inbound commands, agent responses, and button clicks.</p>
              </div>
              <button
                onClick={async () => setEvents(await api.listTelegramEvents())}
                className="px-3 py-1.5 text-xs font-medium text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
              >
                🔄 Refresh
              </button>
            </div>

            {events.length === 0 ? (
              <div className="py-12 text-center text-sm text-gray-500">
                No recent Telegram events recorded yet. Dispatch a diagnostic message above to verify.
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
                                ? 'bg-sky-50 text-sky-700 border border-sky-200'
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
