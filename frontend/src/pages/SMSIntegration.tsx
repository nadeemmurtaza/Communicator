import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type SMSChannelData,
  type SMSEvent,
} from '../lib/api'
import { SmsIcon } from '../components/icons/BrandIcons'

export function SMSIntegration() {
  const [data, setData] = useState<SMSChannelData | null>(null)
  const [events, setEvents] = useState<SMSEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'credentials' | 'webhooks' | 'compliance' | 'automations' | 'diagnostics'
  >('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [sendingTest, setSendingTest] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Credentials State
  const [inboxName, setInboxName] = useState('Twilio SMS Support')
  const [accountSid, setAccountSid] = useState('')
  const [authToken, setAuthToken] = useState('')
  const [phoneNumber, setPhoneNumber] = useState('+1 (415) 555-2480')
  const [messagingServiceSid, setMessagingServiceSid] = useState('')

  // 10DLC & Compliance State
  const [a2pStatus, setA2pStatus] = useState('VERIFIED')
  const [brandName, setBrandName] = useState('Bay Works LLC')
  const [brandTier, setBrandTier] = useState('Standard High Throughput (10DLC)')
  const [optOutMessage, setOptOutMessage] = useState(
    'You have successfully unsubscribed from Bay Works SMS. No further messages will be sent. Reply START to resubscribe.'
  )
  const [optInMessage, setOptInMessage] = useState(
    'You have successfully resubscribed to Bay Works SMS. Standard msg & data rates may apply. Reply STOP to cancel.'
  )

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState(
    '👋 Hello from Bay Works SMS Support! Reply to this message to chat with an agent.'
  )
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState(
    'Thanks for contacting Bay Works SMS. We are currently outside business hours, but we will reply as soon as we reopen.'
  )
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('America/New_York')

  // Diagnostic Test Sender State
  const [testPhone, setTestPhone] = useState('+1 (415) 555-0199')
  const [testMessage, setTestMessage] = useState(
    'Hello! This is a test diagnostic SMS from your Bay Works Twilio 10DLC integration.'
  )
  const [testResult, setTestResult] = useState<{ sid: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getSMSChannel(),
        api.listSMSEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setAccountSid(channelData.inbox.credentials?.accountSid || '')
      setPhoneNumber(channelData.inbox.phoneNumber || channelData.inbox.credentials?.fromNumber || '')
      setMessagingServiceSid(channelData.inbox.credentials?.messagingServiceSid || '')

      setA2pStatus(channelData.compliance10dlc.status || 'VERIFIED')
      setBrandName(channelData.compliance10dlc.brandName || 'Bay Works LLC')
      setBrandTier(channelData.compliance10dlc.brandTier || 'Standard High Throughput (10DLC)')
      setOptOutMessage(channelData.compliance10dlc.optOutMessage || '')
      setOptInMessage(channelData.compliance10dlc.optInMessage || '')

      setGreetingEnabled(channelData.inbox.greetingEnabled)
      setGreetingMessage(channelData.inbox.greetingMessage)
      setOutOfOfficeEnabled(channelData.inbox.outOfOfficeEnabled)
      setOutOfOfficeMessage(channelData.inbox.outOfOfficeMessage)
      setCsatEnabled(channelData.inbox.csatEnabled)
      setWorkingHoursEnabled(channelData.inbox.workingHoursEnabled)
      setWorkingHoursStart(channelData.inbox.workingHoursStart)
      setWorkingHoursEnd(channelData.inbox.workingHoursEnd)
      setTimezone(channelData.inbox.timezone)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function handleSave() {
    try {
      setSaving(true)
      setError(null)
      const res = await api.updateSMSChannel({
        name: inboxName,
        account_sid: accountSid,
        auth_token: authToken,
        phone_number: phoneNumber,
        messaging_service_sid: messagingServiceSid,
        a2p_10dlc_status: a2pStatus,
        brand_name: brandName,
        brand_tier: brandTier,
        opt_out_message: optOutMessage,
        opt_in_message: optInMessage,
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
      setData(res)
      setSuccess('Twilio SMS settings saved successfully!')
      setTimeout(() => setSuccess(null), 3500)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleVerify() {
    try {
      setVerifying(true)
      setError(null)
      const res = await api.verifySMSConnection()
      if (res.verified) {
        setSuccess(`Twilio connection verified (${res.accountSid})! 10DLC status: ${res.a2p10dlcStatus}`)
        loadChannel()
        setTimeout(() => setSuccess(null), 4000)
      } else {
        setError('Verification failed')
      }
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setVerifying(false)
    }
  }

  async function handleSendTest() {
    if (!testPhone.trim()) {
      setError('Please provide a recipient phone number')
      return
    }
    try {
      setSendingTest(true)
      setError(null)
      const res = await api.sendSMSTestMessage({
        recipient_phone: testPhone,
        message: testMessage,
      })
      setTestResult({ sid: res.sid, message: res.message })
      setSuccess(`Diagnostic SMS dispatched! Twilio SID: ${res.sid}`)
      const evList = await api.listSMSEvents()
      setEvents(evList)
      setTimeout(() => setSuccess(null), 4500)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSendingTest(false)
    }
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  // Segment calculator
  const charCount = testMessage.length
  const segmentCount = charCount <= 160 ? 1 : Math.ceil(charCount / 153)

  if (loading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <div className="text-center">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-blue-600 border-r-transparent"></div>
          <p className="mt-2 text-sm text-slate-500 font-medium">Loading Twilio SMS Integration...</p>
        </div>
      </div>
    )
  }

  const callbackUrl = data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/twilio/sms'

  return (
    <div className="space-y-6">
      {/* Top Breadcrumb & Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-slate-200 pb-5">
        <div>
          <div className="flex items-center gap-2 text-xs text-slate-500 mb-1">
            <Link to="/channels" className="hover:text-blue-600 flex items-center gap-1 font-medium">
              <span>← Channels Hub</span>
            </Link>
            <span>/</span>
            <span>Channel 3</span>
            <span>/</span>
            <span className="text-slate-800 font-semibold">SMS Messaging (Twilio)</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-red-600 text-white shadow-sm">
              <SmsIcon className="h-6 w-6" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-2xl font-bold tracking-tight text-slate-900">{inboxName}</h1>
                <span className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2.5 py-0.5 text-xs font-semibold text-emerald-700 border border-emerald-200">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
                  10DLC Verified
                </span>
                <span className="inline-flex items-center rounded-md bg-red-50 px-2 py-0.5 text-xs font-medium text-red-700 border border-red-200">
                  Channel 3
                </span>
              </div>
              <p className="text-xs text-slate-500 mt-0.5">
                Official Twilio REST API integration, 10DLC carrier compliance, automated STOP/START opt-out, and shared team queue routing.
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2.5">
          <Link
            to="/"
            className="rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-xs font-semibold text-slate-700 shadow-sm hover:bg-slate-50"
          >
            Open Shared Inbox ↗
          </Link>
          <button
            onClick={handleVerify}
            disabled={verifying}
            className="rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-xs font-semibold text-slate-700 shadow-sm hover:bg-slate-50 disabled:opacity-50"
          >
            {verifying ? 'Checking...' : 'Verify Connection'}
          </button>
          <button
            onClick={() => setActiveTab('diagnostics')}
            className="rounded-lg border border-blue-200 bg-blue-50 px-3.5 py-2 text-xs font-semibold text-blue-700 shadow-sm hover:bg-blue-100"
          >
            ⚡ Test Diagnostic SMS
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Metrics Bar */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Primary Twilio Number</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-base font-bold text-slate-900 font-mono">{phoneNumber || '+1 (415) 555-2480'}</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">Capabilities: SMS, MMS</p>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">A2P 10DLC Status</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-base font-bold text-emerald-600">● {a2pStatus}</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">{brandTier}</p>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Carrier Deliverability</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-slate-900">98.7%</span>
            <span className="text-[11px] font-semibold text-emerald-600">High Trust</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">Zero carrier spam drops</p>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Throughput Quota</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-slate-900">200K</span>
            <span className="text-[11px] text-slate-500">msgs/day</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">Standard High Throughput</p>
        </div>
      </div>

      {/* Notifications */}
      {success && (
        <div className="rounded-lg border border-emerald-200 bg-emerald-50 p-3.5 text-xs text-emerald-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span>✅</span>
            <span className="font-medium">{success}</span>
          </div>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-900">×</button>
        </div>
      )}
      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-3.5 text-xs text-red-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span>⚠️</span>
            <span className="font-medium">{error}</span>
          </div>
          <button onClick={() => setError(null)} className="text-red-600 hover:text-red-900">×</button>
        </div>
      )}

      {/* Tabs Navigation */}
      <div className="border-b border-slate-200">
        <nav className="flex space-x-6 overflow-x-auto">
          {[
            { id: 'credentials', label: 'Credentials & Number', icon: '🔑' },
            { id: 'webhooks', label: 'Webhooks & Carrier Sync', icon: '📡' },
            { id: 'compliance', label: 'A2P 10DLC & Opt-Out', icon: '🛡️' },
            { id: 'automations', label: 'Automations & Schedule', icon: '⚙️' },
            { id: 'diagnostics', label: 'Live Sender & Events', icon: '🧪' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex items-center gap-2 border-b-2 py-3 px-1 text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === tab.id
                  ? 'border-red-600 text-red-600'
                  : 'border-transparent text-slate-500 hover:border-slate-300 hover:text-slate-700'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
              {tab.id === 'diagnostics' && (
                <span className="rounded-full bg-red-100 text-red-700 px-1.5 py-0.2 text-[10px] font-bold">
                  Diagnostic
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>

      {/* TAB 1: CREDENTIALS & PHONE NUMBER */}
      {activeTab === 'credentials' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-5">
            <h2 className="text-base font-semibold text-slate-900 border-b border-slate-100 pb-3">
              Twilio API Credentials
            </h2>

            <div>
              <label className="block text-xs font-semibold text-slate-700 mb-1">Inbox Display Name</label>
              <input
                type="text"
                value={inboxName}
                onChange={(e) => setInboxName(e.target.value)}
                className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-red-500 focus:outline-none"
                placeholder="e.g. Twilio SMS Support"
              />
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Twilio Account SID</label>
                <input
                  type="text"
                  value={accountSid}
                  onChange={(e) => setAccountSid(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono focus:border-red-500 focus:outline-none"
                  placeholder="ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                />
                <p className="text-[11px] text-slate-400 mt-1">Found in your Twilio Console Account Dashboard</p>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Twilio Auth Token</label>
                <input
                  type="password"
                  value={authToken}
                  onChange={(e) => setAuthToken(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono focus:border-red-500 focus:outline-none"
                  placeholder="••••••••••••••••••••••••••••••••"
                />
                <p className="text-[11px] text-slate-400 mt-1">Encrypted and securely stored on the server</p>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-slate-100">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Twilio Phone Number</label>
                <input
                  type="text"
                  value={phoneNumber}
                  onChange={(e) => setPhoneNumber(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono focus:border-red-500 focus:outline-none"
                  placeholder="+14155552480"
                />
                <p className="text-[11px] text-slate-400 mt-1">E.164 formatted sender number assigned to your project</p>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Messaging Service SID (Optional)</label>
                <input
                  type="text"
                  value={messagingServiceSid}
                  onChange={(e) => setMessagingServiceSid(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono focus:border-red-500 focus:outline-none"
                  placeholder="MGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                />
                <p className="text-[11px] text-slate-400 mt-1">Use for 10DLC number pooling or automated sender distribution</p>
              </div>
            </div>

            <div className="flex items-center gap-3 pt-3">
              <button
                onClick={handleSave}
                disabled={saving}
                className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
              >
                {saving ? 'Saving...' : 'Save Credentials'}
              </button>
              <button
                onClick={handleVerify}
                disabled={verifying}
                className="rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-xs font-semibold text-slate-700 shadow-sm hover:bg-slate-50 disabled:opacity-50"
              >
                {verifying ? 'Checking API...' : 'Test Twilio Connection'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: WEBHOOKS & CARRIER SYNC */}
      {activeTab === 'webhooks' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-5">
            <h2 className="text-base font-semibold text-slate-900 border-b border-slate-100 pb-3">
              Twilio Inbound SMS Webhook Configuration
            </h2>

            <div className="rounded-lg bg-slate-50 p-4 border border-slate-200 space-y-2">
              <div className="text-xs font-semibold text-slate-800">Inbound SMS Callback URL</div>
              <div className="flex items-center justify-between gap-3">
                <code className="text-xs font-mono text-slate-700 truncate">{callbackUrl}</code>
                <button
                  onClick={() => copyToClipboard(callbackUrl, 'webhook')}
                  className="rounded-md bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 shrink-0"
                >
                  {copiedKey === 'webhook' ? '✓ Copied' : '📋 Copy URL'}
                </button>
              </div>
              <p className="text-[11px] text-slate-500">
                Configure this URL in your Twilio Console under <span className="font-semibold text-slate-700">Phone Numbers → Active Numbers → A Message Comes In</span>.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-3">
              <div className="rounded-lg border border-slate-200 p-3.5">
                <div className="text-xs font-bold text-slate-800">HTTP Method</div>
                <div className="mt-1 text-xs font-mono text-emerald-600 font-bold">HTTP POST</div>
                <p className="text-[11px] text-slate-400 mt-0.5">Twilio sends form-encoded payload</p>
              </div>

              <div className="rounded-lg border border-slate-200 p-3.5">
                <div className="text-xs font-bold text-slate-800">Signature Security</div>
                <div className="mt-1 text-xs font-bold text-blue-600">X-Twilio-Signature</div>
                <p className="text-[11px] text-slate-400 mt-0.5">Validated against your Auth Token</p>
              </div>

              <div className="rounded-lg border border-slate-200 p-3.5">
                <div className="text-xs font-bold text-slate-800">Status Callbacks</div>
                <div className="mt-1 text-xs font-bold text-slate-700">Delivered / Undelivered</div>
                <p className="text-[11px] text-slate-400 mt-0.5">Real-time carrier receipt sync</p>
              </div>
            </div>

            <div className="rounded-lg bg-blue-50 border border-blue-200 p-4 text-xs text-blue-900 space-y-2">
              <div className="font-bold">Step-by-Step Twilio Console Setup:</div>
              <ol className="list-decimal list-inside space-y-1 text-[11.5px] text-blue-800">
                <li>Log into your <span className="font-semibold">Twilio Console</span>.</li>
                <li>Navigate to <span className="font-semibold">Phone Numbers → Manage → Active Numbers</span> and click your phone number.</li>
                <li>Scroll down to the <span className="font-semibold">Messaging</span> configuration section.</li>
                <li>Under <span className="font-semibold">A MESSAGE COMES IN</span>, select <span className="font-semibold">Webhook</span>.</li>
                <li>Paste the Inbound Callback URL above and select <span className="font-semibold">HTTP POST</span>.</li>
                <li>Click <span className="font-semibold">Save Configuration</span> at the bottom of the page.</li>
              </ol>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: 10DLC & OPT-OUT COMPLIANCE */}
      {activeTab === 'compliance' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-5">
            <h2 className="text-base font-semibold text-slate-900 border-b border-slate-100 pb-3">
              A2P 10DLC Brand Registration & Carrier Compliance
            </h2>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">A2P 10DLC Status</label>
                <select
                  value={a2pStatus}
                  onChange={(e) => setA2pStatus(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs"
                >
                  <option value="VERIFIED">VERIFIED (Campaign Approved)</option>
                  <option value="REGISTERED">REGISTERED (Standard)</option>
                  <option value="IN_REVIEW">IN_REVIEW (Carrier Review)</option>
                  <option value="EXEMPT">EXEMPT (Toll-Free / Short Code)</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Registered Brand Entity Name</label>
                <input
                  type="text"
                  value={brandName}
                  onChange={(e) => setBrandName(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-red-500 focus:outline-none"
                  placeholder="Bay Works LLC"
                />
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-700 mb-1">Brand Trust Tier</label>
              <input
                type="text"
                value={brandTier}
                onChange={(e) => setBrandTier(e.target.value)}
                className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-red-500 focus:outline-none"
                placeholder="Standard High Throughput (10DLC)"
              />
            </div>

            <div className="pt-3 border-t border-slate-100 space-y-4">
              <h3 className="text-xs font-bold uppercase tracking-wider text-slate-500">
                Automated Carrier Opt-Out / Opt-In Enforcement
              </h3>

              <div className="flex flex-wrap gap-2 text-xs">
                <span className="font-semibold text-slate-700 mr-2">Monitored Stop Keywords:</span>
                {['STOP', 'UNSUBSCRIBE', 'CANCEL', 'END', 'QUIT'].map((kw) => (
                  <span key={kw} className="rounded bg-red-100 text-red-800 font-mono font-bold px-2 py-0.5 text-[11px]">
                    {kw}
                  </span>
                ))}
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">
                  Automated Opt-Out Confirmation Message
                </label>
                <textarea
                  rows={2}
                  value={optOutMessage}
                  onChange={(e) => setOptOutMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-red-500 focus:outline-none"
                />
                <p className="text-[11px] text-slate-400 mt-1">
                  Dispatched immediately when any contact texts STOP. Contact is marked OPTED_OUT across all channels.
                </p>
              </div>

              <div className="flex flex-wrap gap-2 text-xs pt-2">
                <span className="font-semibold text-slate-700 mr-2">Monitored Start Keywords:</span>
                {['START', 'UNSTOP', 'YES'].map((kw) => (
                  <span key={kw} className="rounded bg-emerald-100 text-emerald-800 font-mono font-bold px-2 py-0.5 text-[11px]">
                    {kw}
                  </span>
                ))}
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">
                  Automated Opt-In / Resubscribe Message
                </label>
                <textarea
                  rows={2}
                  value={optInMessage}
                  onChange={(e) => setOptInMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-red-500 focus:outline-none"
                />
              </div>

              <div className="pt-2">
                <button
                  onClick={handleSave}
                  disabled={saving}
                  className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                >
                  {saving ? 'Saving...' : 'Save Compliance Rules'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 4: AUTOMATIONS & WORKING HOURS */}
      {activeTab === 'automations' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-6">
            <h2 className="text-base font-semibold text-slate-900 border-b border-slate-100 pb-3">
              Automated Replies & Support Hours
            </h2>

            {/* Greeting */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Welcome Greeting SMS</div>
                  <div className="text-[11px] text-slate-500">Automatically reply to first-time inbound SMS inquiries</div>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-red-600"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-red-500 focus:outline-none"
                />
              )}
            </div>

            {/* Out of Office */}
            <div className="space-y-3 pt-3 border-t border-slate-100">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Out of Office / After-Hours SMS</div>
                  <div className="text-[11px] text-slate-500">Send an automated reply when customers text outside working hours</div>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-red-600"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-red-500 focus:outline-none"
                />
              )}
            </div>

            {/* CSAT */}
            <div className="pt-3 border-t border-slate-100 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-800">Customer Satisfaction (CSAT) Rating</div>
                <div className="text-[11px] text-slate-500">Send a satisfaction survey via SMS when a conversation is resolved</div>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="rounded border-slate-300 text-red-600"
              />
            </div>

            {/* Working Hours */}
            <div className="pt-3 border-t border-slate-100 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Working Hours Enforcement</div>
                  <div className="text-[11px] text-slate-500">Define active support hours for agent presence</div>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-red-600"
                />
              </div>

              {workingHoursEnabled && (
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-2">
                  <div>
                    <label className="block text-[11px] font-semibold text-slate-700 mb-1">Start Hour</label>
                    <select
                      value={workingHoursStart}
                      onChange={(e) => setWorkingHoursStart(Number(e.target.value))}
                      className="w-full rounded-lg border border-slate-300 px-3 py-1.5 text-xs"
                    >
                      {Array.from({ length: 24 }).map((_, i) => (
                        <option key={i} value={i}>
                          {i === 0 ? '12 AM (Midnight)' : i < 12 ? `${i} AM` : i === 12 ? '12 PM (Noon)' : `${i - 12} PM`}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-[11px] font-semibold text-slate-700 mb-1">End Hour</label>
                    <select
                      value={workingHoursEnd}
                      onChange={(e) => setWorkingHoursEnd(Number(e.target.value))}
                      className="w-full rounded-lg border border-slate-300 px-3 py-1.5 text-xs"
                    >
                      {Array.from({ length: 24 }).map((_, i) => (
                        <option key={i} value={i}>
                          {i === 0 ? '12 AM (Midnight)' : i < 12 ? `${i} AM` : i === 12 ? '12 PM (Noon)' : `${i - 12} PM`}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-[11px] font-semibold text-slate-700 mb-1">Timezone</label>
                    <input
                      type="text"
                      value={timezone}
                      onChange={(e) => setTimezone(e.target.value)}
                      className="w-full rounded-lg border border-slate-300 px-3 py-1.5 text-xs"
                      placeholder="America/New_York"
                    />
                  </div>
                </div>
              )}
            </div>

            <div className="pt-2">
              <button
                onClick={handleSave}
                disabled={saving}
                className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
              >
                {saving ? 'Saving...' : 'Save Automations'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* TAB 5: DIAGNOSTIC SENDER & LIVE EVENTS */}
      {activeTab === 'diagnostics' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-6 space-y-6">
            <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-4">
              <div>
                <h2 className="text-base font-semibold text-slate-900">Diagnostic SMS Dispatcher</h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Send a live test message to verify carrier routing, 10DLC throughput, and Shared Inbox sync.
                </p>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Recipient Mobile Number</label>
                <input
                  type="text"
                  value={testPhone}
                  onChange={(e) => setTestPhone(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono focus:border-red-500 focus:outline-none"
                  placeholder="+14155550199"
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="block text-xs font-semibold text-slate-700">SMS Body</label>
                  <span className={`text-[11px] font-mono ${charCount > 160 ? 'text-amber-600 font-bold' : 'text-slate-400'}`}>
                    {charCount} chars · {segmentCount} {segmentCount === 1 ? 'segment' : 'segments'}
                  </span>
                </div>
                <textarea
                  rows={3}
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-red-500 focus:outline-none"
                />
              </div>

              <div className="flex items-center gap-3 pt-2">
                <button
                  onClick={handleSendTest}
                  disabled={sendingTest}
                  className="rounded-lg bg-red-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
                >
                  <span>{sendingTest ? 'Dispatching...' : '🚀 Dispatch Test SMS'}</span>
                </button>
                {testResult && (
                  <Link
                    to="/"
                    className="rounded-lg border border-emerald-300 bg-emerald-50 px-3.5 py-2 text-xs font-semibold text-emerald-800 hover:bg-emerald-100"
                  >
                    View in Shared Inbox →
                  </Link>
                )}
              </div>

              {testResult && (
                <div className="rounded-lg bg-emerald-50 border border-emerald-200 p-3.5 text-xs text-emerald-900 space-y-1">
                  <div className="font-bold flex items-center gap-2">
                    <span>✓ Message Queued with Twilio:</span>
                    <span className="font-mono text-emerald-700">{testResult.sid}</span>
                  </div>
                  <p className="text-[11px] text-emerald-800">{testResult.message}</p>
                </div>
              )}
            </div>
          </div>

          {/* Live SMS Stream */}
          <div className="lg:col-span-6">
            <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-4">
              <div className="flex items-center justify-between border-b border-slate-100 pb-3">
                <div>
                  <h3 className="text-sm font-semibold text-slate-900">Recent Twilio SMS Events</h3>
                  <p className="text-xs text-slate-500">Real-time incoming customer texts & agent replies</p>
                </div>
                <button
                  onClick={async () => {
                    const ev = await api.listSMSEvents()
                    setEvents(ev)
                  }}
                  className="text-xs font-semibold text-red-600 hover:text-red-800"
                >
                  🔄 Refresh
                </button>
              </div>

              {events.length === 0 ? (
                <div className="py-8 text-center text-xs text-slate-400">
                  No SMS events recorded yet. Send a test message from the left to start!
                </div>
              ) : (
                <div className="divide-y divide-slate-100 max-h-[460px] overflow-y-auto">
                  {events.map((ev) => (
                    <div key={ev.id} className="py-2.5 flex items-start justify-between gap-3">
                      <div className="space-y-0.5">
                        <div className="flex items-center gap-2">
                          <span
                            className={`rounded px-1.5 py-0.5 text-[10px] font-bold ${
                              ev.direction === 'inbound'
                                ? 'bg-blue-100 text-blue-800'
                                : 'bg-red-100 text-red-800'
                            }`}
                          >
                            {ev.direction.toUpperCase()}
                          </span>
                          <span className="text-xs font-semibold text-slate-800">{ev.senderName}</span>
                          <span className="text-[10px] text-slate-400">
                            {new Date(ev.sentAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                          </span>
                        </div>
                        <p className="text-xs text-slate-600 line-clamp-2">{ev.body}</p>
                      </div>
                      <Link
                        to="/"
                        className="text-[11px] font-semibold text-blue-600 hover:text-blue-800 shrink-0 mt-1"
                      >
                        Open ↗
                      </Link>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
