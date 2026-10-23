import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, type WhatsAppChannelData, type WhatsAppTemplate } from '../lib/api'
import { WhatsAppIcon } from '../components/icons/BrandIcons'

export function WhatsAppIntegration() {
  const [data, setData] = useState<WhatsAppChannelData | null>(null)
  const [templates, setTemplates] = useState<WhatsAppTemplate[]>([])
  const [events, setEvents] = useState<any[]>([])
  const [activeTab, setActiveTab] = useState<'credentials' | 'webhooks' | 'templates' | 'automations' | 'profile' | 'diagnostics'>('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [syncingTemplates, setSyncingTemplates] = useState(false)
  const [testingSend, setTestingSend] = useState(false)
  const [savingProfile, setSavingProfile] = useState(false)
  const [registeringPin, setRegisteringPin] = useState(false)
  const [registerPin, setRegisterPin] = useState('123456')
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Form state
  const [name, setName] = useState('')
  const [phoneNumberId, setPhoneNumberId] = useState('')
  const [wabaId, setWabaId] = useState('')
  const [accessToken, setAccessToken] = useState('')
  const [appSecret, setAppSecret] = useState('')
  const [verifyToken, setVerifyToken] = useState('')
  const [catalogId, setCatalogId] = useState('')

  // Business Profile state
  const [profileAbout, setProfileAbout] = useState('Official Customer Support for Bay Works')
  const [profileAddress, setProfileAddress] = useState('100 Market St, Suite 400, San Francisco, CA 94105')
  const [profileDescription, setProfileDescription] = useState('Leading customer engagement platform with omnichannel messaging and AI Copilot.')
  const [profileEmail, setProfileEmail] = useState('support@bayworks.com')
  const [profileVertical, setProfileVertical] = useState('PROF_SERVICES')
  const [profileWebsite1, setProfileWebsite1] = useState('https://bayworks.com')
  const [profileWebsite2, setProfileWebsite2] = useState('https://help.bayworks.com')
  const [profilePictureUrl, setProfilePictureUrl] = useState('https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=300')

  // Automations state
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState('')
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState('')
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [autoReadReceipts, setAutoReadReceipts] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('UTC')

  // Test send state
  const [testPhone, setTestPhone] = useState('')
  const [testBody, setTestBody] = useState('Hi there! This is a test message from your Chatwoot WhatsApp Business integration.')
  const [testResult, setTestResult] = useState<any | null>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)

  useEffect(() => {
    loadAll()
  }, [])

  async function loadAll() {
    try {
      setLoading(true)
      const [channelData, tplList, evList, profileData] = await Promise.all([
        api.getWhatsAppChannel(),
        api.listWhatsAppTemplates(),
        api.listWhatsAppEvents(),
        api.getWhatsAppProfile().catch(() => null),
      ])

      setData(channelData)
      setTemplates(tplList)
      setEvents(evList)

      setName(channelData.inbox.name)
      setPhoneNumberId(channelData.inbox.credentials?.phoneNumberId || '')
      setWabaId(channelData.inbox.credentials?.wabaId || '')
      setVerifyToken(channelData.webhookConfig.verifyToken || '')
      setCatalogId(channelData.inbox.credentials?.catalogId || '')

      setGreetingEnabled(channelData.inbox.greetingEnabled)
      setGreetingMessage(channelData.inbox.greetingMessage)
      setOutOfOfficeEnabled(channelData.inbox.outOfOfficeEnabled)
      setOutOfOfficeMessage(channelData.inbox.outOfOfficeMessage)
      setCsatEnabled(channelData.inbox.csatEnabled)
      setAutoReadReceipts(channelData.inbox.autoReadReceipts)
      setWorkingHoursEnabled(channelData.inbox.workingHoursEnabled)
      setWorkingHoursStart(channelData.inbox.workingHoursStart)
      setWorkingHoursEnd(channelData.inbox.workingHoursEnd)
      setTimezone(channelData.inbox.timezone)

      if (profileData) {
        setProfileAbout(profileData.about || '')
        setProfileAddress(profileData.address || '')
        setProfileDescription(profileData.description || '')
        setProfileEmail(profileData.email || '')
        setProfileVertical(profileData.vertical || 'PROF_SERVICES')
        setProfileWebsite1(profileData.websites?.[0] || 'https://bayworks.com')
        setProfileWebsite2(profileData.websites?.[1] || '')
        setProfilePictureUrl(profileData.profilePictureUrl || '')
      }
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveProfile() {
    try {
      setSavingProfile(true)
      setError(null)
      const websites = [profileWebsite1.trim(), profileWebsite2.trim()].filter(Boolean)
      const res = await api.updateWhatsAppProfile({
        about: profileAbout,
        address: profileAddress,
        description: profileDescription,
        email: profileEmail,
        vertical: profileVertical,
        websites,
        profilePictureUrl,
      })
      setSuccess(
        res.metaSynced
          ? 'WhatsApp Business Profile synced with Meta Graph API and saved!'
          : 'WhatsApp Business Profile settings saved successfully!'
      )
      setTimeout(() => setSuccess(null), 4000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSavingProfile(false)
    }
  }

  async function handleRegisterPhone() {
    try {
      setRegisteringPin(true)
      setError(null)
      const res = await api.registerWhatsAppPhone(registerPin)
      if (res.status === 'registered') {
        setSuccess(res.message)
      } else {
        setError(res.message)
      }
      setTimeout(() => setSuccess(null), 4000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setRegisteringPin(false)
    }
  }

  async function handleSave() {
    try {
      setSaving(true)
      setError(null)
      const res = await api.updateWhatsAppChannel({
        name,
        phone_number_id: phoneNumberId,
        waba_id: wabaId,
        access_token: accessToken,
        app_secret: appSecret,
        webhook_verify_token: verifyToken,
        catalog_id: catalogId,
        greeting_enabled: greetingEnabled,
        greeting_message: greetingMessage,
        out_of_office_enabled: outOfOfficeEnabled,
        out_of_office_message: outOfOfficeMessage,
        csat_enabled: csatEnabled,
        auto_read_receipts: autoReadReceipts,
        working_hours_enabled: workingHoursEnabled,
        working_hours_start: workingHoursStart,
        working_hours_end: workingHoursEnd,
        timezone,
      })
      setData(res)
      setSuccess('WhatsApp Business settings successfully saved!')
      setTimeout(() => setSuccess(null), 4000)
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
      const res = await api.verifyWhatsAppConnection()
      if (res.status === 'ok') {
        setSuccess('Meta WhatsApp API connection successfully verified!')
        loadAll()
      } else {
        setError('Verification failed')
      }
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setVerifying(false)
    }
  }

  async function handleSyncTemplates() {
    try {
      setSyncingTemplates(true)
      const res = await api.syncWhatsAppTemplates()
      setSuccess(res.message)
      const tpls = await api.listWhatsAppTemplates()
      setTemplates(tpls)
      setTimeout(() => setSuccess(null), 3000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncingTemplates(false)
    }
  }

  async function handleSendTest() {
    if (!testPhone.trim()) {
      setError('Please provide a test recipient phone number')
      return
    }
    try {
      setTestingSend(true)
      setError(null)
      const res = await api.sendWhatsAppTestMessage(testPhone, testBody)
      setTestResult(res)
      setSuccess(`Test WhatsApp message dispatched (${res.provider})!`)
      const evList = await api.listWhatsAppEvents()
      setEvents(evList)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setTestingSend(false)
    }
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  if (loading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <div className="text-center">
          <div className="text-4xl animate-bounce">📱</div>
          <p className="mt-2 text-sm text-slate-500">Loading WhatsApp Channel configuration...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Top Breadcrumb & Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 text-xs text-slate-500 mb-1">
            <Link to="/connections" className="hover:text-slate-800">
              Channels Hub
            </Link>
            <span>/</span>
            <span className="font-semibold text-slate-700">WhatsApp Business API</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-500 text-white shadow-xs">
              <WhatsAppIcon className="h-6 w-6" />
            </div>
            <div>
              <h1 className="text-xl font-bold text-slate-900 flex items-center gap-2">
                WhatsApp Business Integration
                <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800">
                  {data?.health.qualityRating === 'GREEN' ? '🟢 Verified & Healthy' : 'Active'}
                </span>
              </h1>
              <p className="text-xs text-slate-500">
                Official Meta Cloud API · Tier 100K messages/day · 24-hour customer service window
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleVerify}
            disabled={verifying}
            className="rounded-md border border-slate-300 bg-white px-3.5 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50 shadow-xs"
          >
            {verifying ? 'Checking Meta...' : '⚡ Verify Connection'}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-md bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 disabled:opacity-50 shadow-xs"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Health Metric Banner */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">Display Phone</span>
          <p className="mt-1 font-bold text-slate-900 text-sm truncate">{data?.health.displayPhoneNumber || 'Configured'}</p>
          <span className="mt-1 inline-block text-[11px] text-emerald-600 font-medium">✓ Verified Number</span>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">Verified Name</span>
          <p className="mt-1 font-bold text-slate-900 text-sm truncate">{data?.health.displayName || 'WhatsApp Support'}</p>
          <span className="mt-1 inline-block text-[11px] text-blue-600 font-medium">Official Business Account</span>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">Quality Rating</span>
          <p className="mt-1 font-bold text-emerald-700 text-sm flex items-center gap-1.5">
            <span className="h-2 w-2 rounded-full bg-emerald-500"></span>
            HIGH (Green)
          </p>
          <span className="mt-1 inline-block text-[11px] text-slate-400">Zero spam flags</span>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">Messaging Limit</span>
          <p className="mt-1 font-bold text-slate-900 text-sm">100,000 / day</p>
          <span className="mt-1 inline-block text-[11px] text-purple-600 font-medium">Tier 3 Scaled</span>
        </div>
      </div>

      {/* Notifications */}
      {error && (
        <div className="flex items-center justify-between rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="font-bold">×</button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="font-bold">×</button>
        </div>
      )}

      {/* Navigation Tabs */}
      <div className="border-b border-slate-200 flex gap-6 text-sm font-medium">
        <button
          onClick={() => setActiveTab('credentials')}
          className={`pb-3 border-b-2 transition-colors ${
            activeTab === 'credentials' ? 'border-slate-900 text-slate-900 font-bold' : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          🔑 Meta Credentials
        </button>
        <button
          onClick={() => setActiveTab('webhooks')}
          className={`pb-3 border-b-2 transition-colors ${
            activeTab === 'webhooks' ? 'border-slate-900 text-slate-900 font-bold' : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          ⚡ Webhook Configuration
        </button>
        <button
          onClick={() => setActiveTab('templates')}
          className={`pb-3 border-b-2 transition-colors ${
            activeTab === 'templates' ? 'border-slate-900 text-slate-900 font-bold' : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          📋 Message Templates ({templates.length})
        </button>
        <button
          onClick={() => setActiveTab('automations')}
          className={`pb-3 border-b-2 transition-colors ${
            activeTab === 'automations' ? 'border-slate-900 text-slate-900 font-bold' : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          🤖 Channel Automations
        </button>
        <button
          onClick={() => setActiveTab('profile')}
          className={`pb-3 border-b-2 transition-colors ${
            activeTab === 'profile' ? 'border-slate-900 text-slate-900 font-bold' : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          🏢 Business Profile
        </button>
        <button
          onClick={() => setActiveTab('diagnostics')}
          className={`pb-3 border-b-2 transition-colors ${
            activeTab === 'diagnostics' ? 'border-slate-900 text-slate-900 font-bold' : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          🩺 Live Test & Diagnostics
        </button>
      </div>

      {/* TAB 1: Meta Credentials */}
      {activeTab === 'credentials' && (
        <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-6">
          <div className="border-b border-slate-100 pb-4">
            <h2 className="text-base font-bold text-slate-900">Meta WhatsApp Cloud API Credentials</h2>
            <p className="text-xs text-slate-500">
              Obtained from Meta for Developers &gt; App Dashboard &gt; WhatsApp &gt; API Setup.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-slate-700">Channel / Inbox Name</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-slate-900 focus:outline-hidden"
                placeholder="WhatsApp Support"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-slate-700">Phone Number ID</label>
              <input
                type="text"
                value={phoneNumberId}
                onChange={(e) => setPhoneNumberId(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-slate-900 focus:outline-hidden font-mono"
                placeholder="e.g. 109823749283742"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-slate-700">WhatsApp Business Account ID (WABA ID)</label>
              <input
                type="text"
                value={wabaId}
                onChange={(e) => setWabaId(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-slate-900 focus:outline-hidden font-mono"
                placeholder="e.g. 982374982374928"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-slate-700">App Secret (for HMAC-SHA256 signature verification)</label>
              <input
                type="password"
                value={appSecret}
                onChange={(e) => setAppSecret(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-slate-900 focus:outline-hidden font-mono"
                placeholder="••••••••••••••••••••••••••••••••"
              />
            </div>
            <div className="md:col-span-2">
              <label className="block text-xs font-semibold text-slate-700">
                System User Permanent Access Token
              </label>
              <input
                type="password"
                value={accessToken}
                onChange={(e) => setAccessToken(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-slate-900 focus:outline-hidden font-mono"
                placeholder="EAA... (Never expires system user token with whatsapp_business_messaging permissions)"
              />
              <p className="mt-1 text-[11px] text-slate-400">
                Generate in Meta Business Manager &gt; System Users &gt; Generate Token with <code>whatsapp_business_messaging</code> and <code>whatsapp_business_management</code>.
              </p>
            </div>
            <div>
              <label className="block text-xs font-semibold text-slate-700">Meta Commerce Catalog ID (Optional)</label>
              <input
                type="text"
                value={catalogId}
                onChange={(e) => setCatalogId(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-slate-900 focus:outline-hidden font-mono"
                placeholder="e.g. 782349872394872"
              />
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: Webhooks Configuration */}
      {activeTab === 'webhooks' && (
        <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-6">
          <div className="border-b border-slate-100 pb-4">
            <h2 className="text-base font-bold text-slate-900">Meta Inbound Webhook Configuration</h2>
            <p className="text-xs text-slate-500">
              Configure these endpoints in your Meta Developer Console under WhatsApp &gt; Configuration.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-xs font-semibold text-slate-700">Callback URL</label>
              <div className="mt-1 flex items-center gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/whatsapp'}
                  className="flex-1 rounded-md border border-slate-300 bg-slate-50 px-3 py-2 text-xs font-mono text-slate-800"
                />
                <button
                  onClick={() => copyToClipboard(data?.webhookConfig.callbackUrl || '', 'url')}
                  className="rounded-md border border-slate-300 bg-white px-3 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                >
                  {copiedKey === 'url' ? '✓ Copied' : 'Copy URL'}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-700">Verify Token</label>
              <div className="mt-1 flex items-center gap-2">
                <input
                  type="text"
                  value={verifyToken}
                  onChange={(e) => setVerifyToken(e.target.value)}
                  className="flex-1 rounded-md border border-slate-300 px-3 py-2 text-xs font-mono text-slate-800"
                />
                <button
                  onClick={() => copyToClipboard(verifyToken, 'token')}
                  className="rounded-md border border-slate-300 bg-white px-3 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                >
                  {copiedKey === 'token' ? '✓ Copied' : 'Copy Token'}
                </button>
              </div>
            </div>

            <div className="rounded-lg border border-slate-100 bg-slate-50/70 p-4 space-y-2">
              <h3 className="text-xs font-bold text-slate-800 uppercase tracking-wider">Subscribed Webhook Fields</h3>
              <div className="flex flex-wrap gap-2 text-xs">
                <span className="rounded-md bg-white border border-slate-200 px-2.5 py-1 font-mono text-emerald-800 font-semibold">
                  ✓ messages (Inbound customer messages &amp; media)
                </span>
                <span className="rounded-md bg-white border border-slate-200 px-2.5 py-1 font-mono text-blue-800 font-semibold">
                  ✓ message_deliveries (Delivered receipts)
                </span>
                <span className="rounded-md bg-white border border-slate-200 px-2.5 py-1 font-mono text-purple-800 font-semibold">
                  ✓ message_reads (Blue checkmarks)
                </span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: Message Templates */}
      {activeTab === 'templates' && (
        <div className="rounded-xl border border-slate-200 bg-white shadow-xs overflow-hidden">
          <div className="border-b border-slate-200 bg-slate-50/60 px-6 py-4 flex items-center justify-between">
            <div>
              <h2 className="text-base font-bold text-slate-900">Meta Approved Message Templates</h2>
              <p className="text-xs text-slate-500">
                Pre-approved templates required to initiate conversations or reply outside the 24-hour service window.
              </p>
            </div>
            <button
              onClick={handleSyncTemplates}
              disabled={syncingTemplates}
              className="rounded-md border border-slate-300 bg-white px-3.5 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs"
            >
              {syncingTemplates ? 'Syncing...' : '🔄 Sync from Meta'}
            </button>
          </div>

          <div className="divide-y divide-slate-100">
            {templates.map((tpl) => (
              <div key={tpl.id} className="p-5 flex flex-col md:flex-row md:items-start md:justify-between gap-4">
                <div className="space-y-1.5 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-sm text-slate-900 font-mono">{tpl.name}</span>
                    <span className="rounded-full bg-emerald-100 px-2 py-0.5 text-[11px] font-semibold text-emerald-800">
                      ✓ {tpl.status}
                    </span>
                    <span className="rounded-md bg-slate-100 px-2 py-0.5 text-[11px] font-semibold text-slate-700 uppercase">
                      {tpl.category}
                    </span>
                    <span className="text-xs text-slate-400">· {tpl.language}</span>
                  </div>

                  <p className="text-xs text-slate-600 bg-slate-50 p-2.5 rounded-md border border-slate-100 font-sans leading-relaxed">
                    "{tpl.bodyText}"
                  </p>

                  <div className="flex flex-wrap gap-2 text-[11px] text-slate-500">
                    {tpl.components?.map((c: any, idx: number) => (
                      <span key={idx} className="rounded bg-slate-100 px-2 py-0.5 font-medium">
                        {c.type}: {c.buttons ? `${c.buttons.length} buttons` : c.format || 'text'}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="shrink-0 text-xs text-slate-400">
                  Updated {new Date(tpl.lastUpdated).toLocaleDateString()}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* TAB 4: Channel Automations */}
      {activeTab === 'automations' && (
        <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-6">
          <div className="border-b border-slate-100 pb-4">
            <h2 className="text-base font-bold text-slate-900">WhatsApp Automated Workflows</h2>
            <p className="text-xs text-slate-500">Configure greeting rules, after-hours responses, read receipts, and CSAT.</p>
          </div>

          <div className="space-y-6 divide-y divide-slate-100">
            {/* Welcome Greeting */}
            <div className="pt-2 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-slate-900">Welcome Greeting</h3>
                  <p className="text-xs text-slate-500">
                    Automatically sent on first-touch or after 24+ hours of inactivity with interactive buttons.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-5 w-5 rounded border-slate-300 text-slate-900"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full rounded-md border border-slate-300 p-2.5 text-xs shadow-xs"
                />
              )}
            </div>

            {/* Read Receipts */}
            <div className="pt-4 flex items-center justify-between">
              <div>
                <h3 className="text-sm font-bold text-slate-900">Automatic Read Receipts (Blue Checks)</h3>
                <p className="text-xs text-slate-500">
                  Immediately sends <code>mark_as_read</code> to Meta upon incoming message receipt so users see blue double checks.
                </p>
              </div>
              <input
                type="checkbox"
                checked={autoReadReceipts}
                onChange={(e) => setAutoReadReceipts(e.target.checked)}
                className="h-5 w-5 rounded border-slate-300 text-slate-900"
              />
            </div>

            {/* CSAT Survey */}
            <div className="pt-4 flex items-center justify-between">
              <div>
                <h3 className="text-sm font-bold text-slate-900">Post-Resolution CSAT Rating Survey</h3>
                <p className="text-xs text-slate-500">
                  Sends interactive 1-to-5 star rating quick reply buttons when a conversation is marked resolved.
                </p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-5 w-5 rounded border-slate-300 text-slate-900"
              />
            </div>

            {/* Business Hours */}
            <div className="pt-4 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-slate-900">Business Hours &amp; Out-of-Office Away Message</h3>
                  <p className="text-xs text-slate-500">
                    Sends an interactive away message when contacts message outside working hours.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="h-5 w-5 rounded border-slate-300 text-slate-900"
                />
              </div>

              {workingHoursEnabled && (
                <div className="space-y-3 bg-slate-50 p-4 rounded-xl border border-slate-200">
                  <div className="flex items-center gap-4 text-xs">
                    <div>
                      <label className="font-semibold text-slate-700">Open Time (Hour 0-23)</label>
                      <input
                        type="number"
                        min={0}
                        max={23}
                        value={workingHoursStart}
                        onChange={(e) => setWorkingHoursStart(Number(e.target.value))}
                        className="mt-1 w-24 rounded border border-slate-300 p-1.5 bg-white text-xs"
                      />
                    </div>
                    <div>
                      <label className="font-semibold text-slate-700">Close Time (Hour 0-23)</label>
                      <input
                        type="number"
                        min={0}
                        max={23}
                        value={workingHoursEnd}
                        onChange={(e) => setWorkingHoursEnd(Number(e.target.value))}
                        className="mt-1 w-24 rounded border border-slate-300 p-1.5 bg-white text-xs"
                      />
                    </div>
                    <div>
                      <label className="font-semibold text-slate-700">Timezone</label>
                      <input
                        type="text"
                        value={timezone}
                        onChange={(e) => setTimezone(e.target.value)}
                        className="mt-1 w-32 rounded border border-slate-300 p-1.5 bg-white text-xs"
                      />
                    </div>
                  </div>
                  <div>
                    <label className="block text-xs font-semibold text-slate-700">Away Message</label>
                    <textarea
                      rows={2}
                      value={outOfOfficeMessage}
                      onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                      className="mt-1 w-full rounded border border-slate-300 p-2 text-xs bg-white"
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* TAB 5: WhatsApp Business Profile */}
      {activeTab === 'profile' && (
        <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-6">
          <div className="flex items-center justify-between border-b border-slate-200 pb-4">
            <div>
              <h2 className="text-base font-bold text-slate-900">WhatsApp Business Profile</h2>
              <p className="text-xs text-slate-500">
                This public information appears in your WhatsApp business card when contacts view your profile.
              </p>
            </div>
            <button
              onClick={handleSaveProfile}
              disabled={savingProfile}
              className="rounded-md bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 disabled:opacity-50 shadow-xs"
            >
              {savingProfile ? 'Updating Meta...' : 'Save & Sync with Meta'}
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-700">About / Status Text (Max 139 chars)</label>
                <input
                  type="text"
                  maxLength={139}
                  value={profileAbout}
                  onChange={(e) => setProfileAbout(e.target.value)}
                  placeholder="Official Customer Support for Bay Works"
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700">Business Description (Max 512 chars)</label>
                <textarea
                  rows={3}
                  maxLength={512}
                  value={profileDescription}
                  onChange={(e) => setProfileDescription(e.target.value)}
                  placeholder="Describe your business and what customers can expect..."
                  className="mt-1 w-full rounded-md border border-slate-300 p-2 text-sm shadow-xs"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700">Industry Vertical / Category</label>
                <select
                  value={profileVertical}
                  onChange={(e) => setProfileVertical(e.target.value)}
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs bg-white font-medium"
                >
                  <option value="PROF_SERVICES">Professional Services</option>
                  <option value="RETAIL">Retail & eCommerce</option>
                  <option value="TECH">Technology & Software</option>
                  <option value="FINANCE">Financial Services & Banking</option>
                  <option value="HEALTH">Health & Medical</option>
                  <option value="EDUCATION">Education & Training</option>
                  <option value="TRAVEL">Travel, Hospitality & Tourism</option>
                  <option value="OTHER">Other Business</option>
                </select>
              </div>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-700">Official Support Email</label>
                <input
                  type="email"
                  value={profileEmail}
                  onChange={(e) => setProfileEmail(e.target.value)}
                  placeholder="support@yourbrand.com"
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700">Business Physical Address</label>
                <input
                  type="text"
                  value={profileAddress}
                  onChange={(e) => setProfileAddress(e.target.value)}
                  placeholder="100 Market St, Suite 400, San Francisco, CA"
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-slate-700">Primary Website</label>
                  <input
                    type="url"
                    value={profileWebsite1}
                    onChange={(e) => setProfileWebsite1(e.target.value)}
                    placeholder="https://yourbrand.com"
                    className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-700">Help Center / Portal</label>
                  <input
                    type="url"
                    value={profileWebsite2}
                    onChange={(e) => setProfileWebsite2(e.target.value)}
                    placeholder="https://help.yourbrand.com"
                    className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700">Profile Picture / Avatar URL</label>
                <input
                  type="url"
                  value={profilePictureUrl}
                  onChange={(e) => setProfilePictureUrl(e.target.value)}
                  placeholder="https://..."
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs font-mono"
                />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 6: Live Test & Diagnostics */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          {/* Phone Registration Card */}
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-base font-bold text-slate-900 flex items-center gap-2">
                  <span>🔒 Phone Number 2-Step Registration</span>
                  <span className="rounded bg-emerald-100 text-emerald-800 text-[10px] font-bold px-2 py-0.5">
                    Meta Cloud API Standard
                  </span>
                </h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Register your phone number with Meta Graph API using your 6-digit two-factor verification PIN.
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3 max-w-md">
              <input
                type="text"
                maxLength={6}
                value={registerPin}
                onChange={(e) => setRegisterPin(e.target.value.replace(/\D/g, ''))}
                placeholder="6-digit PIN (e.g. 123456)"
                className="w-48 rounded-md border border-slate-300 px-3 py-2 text-sm font-mono tracking-widest text-center shadow-xs"
              />
              <button
                onClick={handleRegisterPhone}
                disabled={registeringPin || registerPin.length < 6}
                className="rounded-md bg-slate-900 px-4 py-2 text-xs font-semibold text-white hover:bg-slate-800 disabled:opacity-40 shadow-xs"
              >
                {registeringPin ? 'Registering...' : 'Register Number with PIN'}
              </button>
            </div>
          </div>

          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
            <h2 className="text-base font-bold text-slate-900">Send Live Test Message</h2>
            <p className="text-xs text-slate-500">
              Verify deliverability directly to your own WhatsApp account before going live.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-semibold text-slate-700">Recipient WhatsApp Phone Number</label>
                <input
                  type="text"
                  value={testPhone}
                  onChange={(e) => setTestPhone(e.target.value)}
                  placeholder="+14155551234"
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm font-mono shadow-xs"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold text-slate-700">Message Content</label>
                <input
                  type="text"
                  value={testBody}
                  onChange={(e) => setTestBody(e.target.value)}
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs"
                />
              </div>
            </div>

            <div className="flex justify-end">
              <button
                onClick={handleSendTest}
                disabled={testingSend}
                className="rounded-md bg-emerald-600 px-4 py-2 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50 shadow-xs"
              >
                {testingSend ? 'Dispatching...' : '🚀 Send Test Message'}
              </button>
            </div>

            {testResult && (
              <div className="rounded-lg border border-slate-200 bg-slate-900 p-3 text-xs font-mono text-emerald-400 overflow-x-auto">
                <pre>{JSON.stringify(testResult, null, 2)}</pre>
              </div>
            )}
          </div>

          {/* Live Events Stream */}
          <div className="rounded-xl border border-slate-200 bg-white shadow-xs overflow-hidden">
            <div className="border-b border-slate-200 bg-slate-50/60 px-5 py-3">
              <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">
                Recent WhatsApp Message Events ({events.length})
              </span>
            </div>

            <div className="divide-y divide-slate-100 max-h-80 overflow-y-auto">
              {events.length === 0 ? (
                <div className="p-6 text-center text-xs text-slate-400">No events received yet.</div>
              ) : (
                events.map((ev) => (
                  <div key={ev.id} className="p-3.5 flex items-center justify-between text-xs hover:bg-slate-50/60">
                    <div className="flex items-center gap-2">
                      <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                        ev.direction === 'inbound' ? 'bg-blue-100 text-blue-800' : 'bg-purple-100 text-purple-800'
                      }`}>
                        {ev.direction}
                      </span>
                      <span className="font-mono text-slate-700">{ev.providerMessageId || 'msg'}</span>
                      <span className="text-slate-600 truncate max-w-sm">"{ev.body}"</span>
                    </div>
                    <div className="flex items-center gap-3 text-slate-400">
                      <span className="font-semibold text-slate-700 capitalize">{ev.status}</span>
                      <span>{new Date(ev.sentAt).toLocaleTimeString()}</span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
