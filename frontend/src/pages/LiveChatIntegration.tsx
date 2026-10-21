import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type LiveChatChannelData,
  type LiveChatEvent,
} from '../lib/api'

export function LiveChatIntegration() {
  const [data, setData] = useState<LiveChatChannelData | null>(null)
  const [events, setEvents] = useState<LiveChatEvent[]>([])
  const [activeTab, setActiveTab] = useState<
    'embed' | 'branding' | 'prechat' | 'security' | 'automations' | 'simulator'
  >('embed')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [regenerating, setRegenerating] = useState(false)
  const [simulating, setSimulating] = useState(false)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Form State
  const [inboxName, setInboxName] = useState('Website Live Chat')
  const [websiteDomain, setWebsiteDomain] = useState('bayworks.com')
  const [widgetColor, setWidgetColor] = useState('#1570EF')
  const [greetingTitle, setGreetingTitle] = useState('Hi there! 👋')
  const [greetingTagline, setGreetingTagline] = useState('We typically reply in a few minutes.')
  const [widgetPosition, setWidgetPosition] = useState<'right' | 'left'>('right')
  const [avatarUrl, setAvatarUrl] = useState('https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=128&h=128&fit=crop')

  // Pre-Chat State
  const [preChatEnabled, setPreChatEnabled] = useState(true)
  const [preChatFields, setPreChatFields] = useState<Array<{ name: string; label: string; required: boolean }>>([
    { name: 'fullName', label: 'Full Name', required: true },
    { name: 'email', label: 'Email Address', required: true },
    { name: 'phone', label: 'Phone Number', required: false },
    { name: 'company', label: 'Company', required: false },
  ])

  // Security State
  const [identityValidation, setIdentityValidation] = useState(false)
  const [hmacSecret, setHmacSecret] = useState('')

  // Automations & Schedule State
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState('👋 Welcome to our live chat! How can our support team help you today?')
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState('Our team is currently offline. Please leave your email and message, and we will get back to you shortly!')
  const [csatEnabled, setCsatEnabled] = useState(true)
  const [workingHoursEnabled, setWorkingHoursEnabled] = useState(false)
  const [workingHoursStart, setWorkingHoursStart] = useState(9)
  const [workingHoursEnd, setWorkingHoursEnd] = useState(18)
  const [timezone, setTimezone] = useState('UTC')

  // Simulator State
  const [simName, setSimName] = useState('Alex Mercer')
  const [simEmail, setSimEmail] = useState('alex.mercer@example.com')
  const [simPhone, setSimPhone] = useState('+1 (555) 234-5678')
  const [simCompany, setSimCompany] = useState('Acme Corporation')
  const [simMessage, setSimMessage] = useState('Hi! I have a question about your enterprise live chat features and setup.')
  const [simResult, setSimResult] = useState<{ conversationId: string; message: string } | null>(null)

  useEffect(() => {
    loadChannel()
  }, [])

  async function loadChannel() {
    try {
      setLoading(true)
      const [channelData, evList] = await Promise.all([
        api.getLiveChatChannel(),
        api.listLiveChatEvents().catch(() => []),
      ])

      setData(channelData)
      setEvents(evList)

      setInboxName(channelData.inbox.name)
      setWebsiteDomain(channelData.websiteDomain || 'bayworks.com')
      setWidgetColor(channelData.branding.widgetColor || '#1570EF')
      setGreetingTitle(channelData.branding.greetingTitle || 'Hi there! 👋')
      setGreetingTagline(channelData.branding.greetingTagline || 'We typically reply in a few minutes.')
      setWidgetPosition(channelData.branding.widgetPosition || 'right')
      setAvatarUrl(channelData.branding.avatarUrl || channelData.inbox.avatarUrl || '')

      setPreChatEnabled(channelData.preChatForm.enabled)
      if (channelData.preChatForm.fields && channelData.preChatForm.fields.length > 0) {
        setPreChatFields(channelData.preChatForm.fields)
      }

      setIdentityValidation(channelData.security.identityValidation)
      setHmacSecret(channelData.security.hmacSecret)

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
      const res = await api.updateLiveChatChannel({
        name: inboxName,
        website_domain: websiteDomain,
        widget_color: widgetColor,
        greeting_title: greetingTitle,
        greeting_tagline: greetingTagline,
        widget_position: widgetPosition,
        avatar_url: avatarUrl,
        pre_chat_form_enabled: preChatEnabled,
        pre_chat_fields: preChatFields,
        identity_validation: identityValidation,
        hmac_secret: hmacSecret,
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
      setSuccess('Website Live Chat settings saved successfully!')
      setTimeout(() => setSuccess(null), 3500)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleRegenerateToken() {
    if (!window.confirm('Are you sure you want to regenerate the Website Token? Existing website embeds using the old token will need to be updated.')) {
      return
    }
    try {
      setRegenerating(true)
      const res = await api.regenerateLiveChatToken()
      if (data) {
        setData({
          ...data,
          websiteToken: res.websiteToken,
          embedScript: res.embedScript,
        })
      }
      setSuccess('Website Live Chat token regenerated successfully!')
      setTimeout(() => setSuccess(null), 3500)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setRegenerating(false)
    }
  }

  async function handleRunSimulator() {
    try {
      setSimulating(true)
      setError(null)
      const res = await api.sendLiveChatTestVisitor({
        name: simName,
        email: simEmail,
        phone: simPhone,
        company: simCompany,
        initial_message: simMessage,
      })
      setSimResult({
        conversationId: res.conversationId,
        message: res.message,
      })
      setSuccess(`Visitor conversation started! Viewable in Shared Inbox.`)
      const evList = await api.listLiveChatEvents()
      setEvents(evList)
      setTimeout(() => setSuccess(null), 4000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSimulating(false)
    }
  }

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  function toggleFieldRequired(index: number) {
    setPreChatFields((prev) =>
      prev.map((f, i) => (i === index ? { ...f, required: !f.required } : f))
    )
  }

  const PALETTE = ['#1570EF', '#7F56D9', '#10B981', '#E11D48', '#F59E0B', '#0F172A']

  if (loading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <div className="text-center">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-blue-600 border-r-transparent"></div>
          <p className="mt-2 text-sm text-slate-500 font-medium">Loading Website Live Chat Integration...</p>
        </div>
      </div>
    )
  }

  const websiteToken = data?.websiteToken || 'web_live_sample'
  const embedCode = data?.embedScript || `<script>
  (function(d,t) {
    var BASE_URL = window.location.origin;
    var g = d.createElement(t), s = d.getElementsByTagName(t)[0];
    g.src = BASE_URL + "/packs/js/sdk.js";
    g.async = true;
    s.parentNode.insertBefore(g, s);
    g.onload = function() {
      window.chatwootSDK.run({
        websiteToken: "${websiteToken}",
        baseUrl: BASE_URL
      });
    };
  })(document, "script");
</script>`

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
            <span>Channel 2</span>
            <span>/</span>
            <span className="text-slate-800 font-semibold">Website Live Chat</span>
          </div>
          <div className="flex items-center gap-3">
            <div
              className="flex h-10 w-10 items-center justify-center rounded-xl text-white shadow-sm"
              style={{ backgroundColor: widgetColor }}
            >
              <span className="text-xl">🌐</span>
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-2xl font-bold tracking-tight text-slate-900">{inboxName}</h1>
                <span className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2.5 py-0.5 text-xs font-semibold text-emerald-700 border border-emerald-200">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
                  Active & Online
                </span>
                <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-0.5 text-xs font-medium text-blue-700 border border-blue-200">
                  Channel 2
                </span>
              </div>
              <p className="text-xs text-slate-500 mt-0.5">
                Embeddable real-time customer chat widget, pre-chat lead qualification, HMAC identity verification, and multi-channel team routing.
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
            onClick={() => setActiveTab('simulator')}
            className="rounded-lg border border-blue-200 bg-blue-50 px-3.5 py-2 text-xs font-semibold text-blue-700 shadow-sm hover:bg-blue-100"
          >
            ⚡ Test Live Chat
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50 flex items-center gap-1.5"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Health & Metrics Bar */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Active Visitors</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-slate-900">{data?.health.activeVisitors ?? 4}</span>
            <span className="text-[11px] font-semibold text-emerald-600">● Real-time</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">Browsing site right now</p>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Avg Response Time</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-slate-900">{data?.health.averageResponseTime ?? '1m 30s'}</span>
            <span className="text-[11px] font-semibold text-blue-600">⚡ High speed</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">Agent first response speed</p>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Resolution Rate</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-slate-900">{data?.health.resolutionRate ?? '94.2%'}</span>
            <span className="text-[11px] font-semibold text-emerald-600">↑ +3.1%</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">Resolved in chat session</p>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-slate-500 uppercase tracking-wider">Widget Identity</div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-slate-900">{identityValidation ? 'HMAC Enforced' : 'Public'}</span>
            <span className="text-[11px] font-semibold text-slate-500">SHA-256</span>
          </div>
          <p className="text-[11px] text-slate-400 mt-0.5">{identityValidation ? 'Impersonation blocked' : 'Standard visitor mode'}</p>
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

      {/* Main Tabs Navigation */}
      <div className="border-b border-slate-200">
        <nav className="flex space-x-6 overflow-x-auto">
          {[
            { id: 'embed', label: 'Embed & SDK Code', icon: '💻' },
            { id: 'branding', label: 'Branding & Appearance', icon: '🎨' },
            { id: 'prechat', label: 'Pre-Chat Form', icon: '📋' },
            { id: 'security', label: 'Security & HMAC', icon: '🔒' },
            { id: 'automations', label: 'Automations & Hours', icon: '⚙️' },
            { id: 'simulator', label: 'Live Preview & Simulator', icon: '🧪' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex items-center gap-2 border-b-2 py-3 px-1 text-xs font-semibold whitespace-nowrap transition-colors ${
                activeTab === tab.id
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-slate-500 hover:border-slate-300 hover:text-slate-700'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
              {tab.id === 'simulator' && (
                <span className="rounded-full bg-blue-100 text-blue-700 px-1.5 py-0.2 text-[10px] font-bold">
                  Interactive
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>

      {/* TAB 1: EMBED & SDK CODE */}
      {activeTab === 'embed' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-slate-100">
              <div>
                <h2 className="text-base font-semibold text-slate-900">Website Embed Code</h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Paste this snippet right before the closing <code className="text-blue-600 font-mono text-[11px]">&lt;/body&gt;</code> tag of every page you want the live chat widget to appear on.
                </p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => copyToClipboard(embedCode, 'embed')}
                  className="rounded-lg bg-blue-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 flex items-center gap-1.5"
                >
                  <span>{copiedKey === 'embed' ? '✓ Copied' : '📋 Copy Code'}</span>
                </button>
              </div>
            </div>

            <div className="mt-4">
              <pre className="rounded-lg bg-slate-950 p-4 text-xs font-mono text-emerald-400 overflow-x-auto border border-slate-800 leading-relaxed">
                {embedCode}
              </pre>
            </div>

            <div className="mt-5 grid grid-cols-1 md:grid-cols-2 gap-4 pt-4 border-t border-slate-100">
              <div className="rounded-lg bg-slate-50 p-3.5 border border-slate-200">
                <div className="text-xs font-semibold text-slate-800 mb-1">Website Token</div>
                <div className="flex items-center justify-between gap-2">
                  <code className="text-xs font-mono text-slate-700 truncate">{websiteToken}</code>
                  <button
                    onClick={() => copyToClipboard(websiteToken, 'token')}
                    className="text-xs font-semibold text-blue-600 hover:text-blue-800 shrink-0"
                  >
                    {copiedKey === 'token' ? 'Copied!' : 'Copy'}
                  </button>
                </div>
              </div>
              <div className="rounded-lg bg-slate-50 p-3.5 border border-slate-200 flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Need a fresh token?</div>
                  <div className="text-[11px] text-slate-500">Invalidates existing widget sessions</div>
                </div>
                <button
                  onClick={handleRegenerateToken}
                  disabled={regenerating}
                  className="rounded-md border border-slate-300 bg-white px-3 py-1.5 text-xs font-semibold text-red-600 hover:bg-red-50 disabled:opacity-50"
                >
                  {regenerating ? 'Regenerating...' : 'Regenerate'}
                </button>
              </div>
            </div>
          </div>

          {/* Framework Guides */}
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
            <h3 className="text-sm font-semibold text-slate-900 mb-4">Framework Integration Guides</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="rounded-lg border border-slate-200 p-4 hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-lg">⚛️</span>
                  <span className="text-xs font-bold text-slate-800">React & Next.js</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Use the Next.js <code className="text-blue-600">&lt;Script /&gt;</code> strategy or mount the script in a custom <code className="text-blue-600">useEffect</code> hook.
                </p>
                <pre className="rounded bg-slate-900 p-2 text-[10px] font-mono text-slate-300 overflow-x-auto">
{`import Script from 'next/script'

<Script
  src="https://api.yourdomain.com/packs/js/sdk.js"
  onLoad={() => {
    window.chatwootSDK.run({
      websiteToken: "${websiteToken}",
      baseUrl: "https://api.yourdomain.com"
    })
  }}
/>`}
                </pre>
              </div>

              <div className="rounded-lg border border-slate-200 p-4 hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-lg">🌐</span>
                  <span className="text-xs font-bold text-slate-800">WordPress & Shopify</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Go to theme settings, Header/Footer scripts, and paste the code snippet inside the Footer section before <code className="text-blue-600">&lt;/body&gt;</code>.
                </p>
                <div className="rounded bg-slate-50 p-2.5 text-[11px] text-slate-600 border border-slate-200 space-y-1">
                  <div>1. Appearance → Theme Editor</div>
                  <div>2. Select <code className="text-blue-600 font-mono">footer.php</code></div>
                  <div>3. Paste script right above <code className="text-blue-600 font-mono">&lt;?php wp_footer(); ?&gt;</code></div>
                </div>
              </div>

              <div className="rounded-lg border border-slate-200 p-4 hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-lg">🏷️</span>
                  <span className="text-xs font-bold text-slate-800">Google Tag Manager</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Create a "Custom HTML" tag, paste the embed script, and set the firing trigger to "All Pages - Page View".
                </p>
                <div className="rounded bg-slate-50 p-2.5 text-[11px] text-slate-600 border border-slate-200 space-y-1">
                  <div>1. New Tag → Custom HTML</div>
                  <div>2. Paste the full script tag</div>
                  <div>3. Trigger: Page View (All Pages)</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: BRANDING & APPEARANCE */}
      {activeTab === 'branding' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-7 space-y-6">
            <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-5">
              <h2 className="text-base font-semibold text-slate-900 border-b border-slate-100 pb-3">
                Widget Customization
              </h2>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Inbox Display Name</label>
                <input
                  type="text"
                  value={inboxName}
                  onChange={(e) => setInboxName(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="e.g. Website Live Chat"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Widget Brand Color</label>
                <div className="flex items-center gap-3">
                  <div
                    className="h-9 w-9 rounded-lg border border-slate-300 shadow-sm shrink-0"
                    style={{ backgroundColor: widgetColor }}
                  />
                  <input
                    type="text"
                    value={widgetColor}
                    onChange={(e) => setWidgetColor(e.target.value)}
                    className="w-32 rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono uppercase focus:border-blue-500 focus:outline-none"
                    placeholder="#1570EF"
                  />
                  <div className="flex items-center gap-1.5">
                    {PALETTE.map((hex) => (
                      <button
                        key={hex}
                        onClick={() => setWidgetColor(hex)}
                        className={`h-7 w-7 rounded-full border-2 transition-transform hover:scale-110 ${
                          widgetColor.toLowerCase() === hex.toLowerCase() ? 'border-slate-900 scale-110' : 'border-white'
                        }`}
                        style={{ backgroundColor: hex }}
                      />
                    ))}
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Greeting Header Title</label>
                  <input
                    type="text"
                    value={greetingTitle}
                    onChange={(e) => setGreetingTitle(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="Hi there! 👋"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Greeting Tagline</label>
                  <input
                    type="text"
                    value={greetingTagline}
                    onChange={(e) => setGreetingTagline(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="We typically reply in a few minutes."
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-2">Widget Launcher Position</label>
                <div className="grid grid-cols-2 gap-3">
                  <button
                    type="button"
                    onClick={() => setWidgetPosition('right')}
                    className={`rounded-lg border p-3 text-left transition-all ${
                      widgetPosition === 'right'
                        ? 'border-blue-600 bg-blue-50/50 ring-2 ring-blue-500/20'
                        : 'border-slate-200 hover:bg-slate-50'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold text-slate-900">Bottom Right</span>
                      <span className="text-xs">👉</span>
                    </div>
                    <p className="text-[11px] text-slate-500 mt-1">Standard position for 95% of websites</p>
                  </button>

                  <button
                    type="button"
                    onClick={() => setWidgetPosition('left')}
                    className={`rounded-lg border p-3 text-left transition-all ${
                      widgetPosition === 'left'
                        ? 'border-blue-600 bg-blue-50/50 ring-2 ring-blue-500/20'
                        : 'border-slate-200 hover:bg-slate-50'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold text-slate-900">Bottom Left</span>
                      <span className="text-xs">👈</span>
                    </div>
                    <p className="text-[11px] text-slate-500 mt-1">Prevents overlap with floating buttons</p>
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Support Avatar / Logo URL</label>
                <input
                  type="text"
                  value={avatarUrl}
                  onChange={(e) => setAvatarUrl(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="https://example.com/logo.png"
                />
              </div>

              <div className="pt-2">
                <button
                  onClick={handleSave}
                  disabled={saving}
                  className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                >
                  {saving ? 'Saving...' : 'Save Appearance'}
                </button>
              </div>
            </div>
          </div>

          {/* Real-time Widget Preview Mockup */}
          <div className="lg:col-span-5">
            <div className="sticky top-6 rounded-xl border border-slate-200 bg-slate-100 p-5 shadow-sm">
              <div className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-3 flex items-center justify-between">
                <span>Live Widget Preview</span>
                <span className="text-[10px] text-slate-400">Position: {widgetPosition}</span>
              </div>

              <div className="relative mx-auto h-[480px] w-full max-w-[340px] rounded-2xl bg-white shadow-xl border border-slate-200 overflow-hidden flex flex-col">
                {/* Widget Header */}
                <div
                  className="p-4 text-white shrink-0 transition-colors"
                  style={{ backgroundColor: widgetColor }}
                >
                  <div className="flex items-center gap-3">
                    <img
                      src={avatarUrl || 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=128'}
                      alt="Avatar"
                      className="h-10 w-10 rounded-full border-2 border-white/60 object-cover shadow-sm"
                    />
                    <div>
                      <div className="font-bold text-sm tracking-tight">{greetingTitle}</div>
                      <div className="text-[11px] text-white/80">{greetingTagline}</div>
                    </div>
                  </div>
                </div>

                {/* Widget Body */}
                <div className="flex-1 p-3.5 space-y-3 overflow-y-auto bg-slate-50/60">
                  <div className="rounded-xl bg-white p-3 shadow-sm border border-slate-100 max-w-[85%] text-xs text-slate-700">
                    {greetingMessage || '👋 Welcome to our live chat! How can our support team help you today?'}
                  </div>
                  <div className="ml-auto rounded-xl bg-blue-600 text-white p-3 shadow-sm max-w-[80%] text-xs">
                    Hi! I wanted to check your pricing for enterprise teams.
                  </div>
                  <div className="rounded-xl bg-white p-3 shadow-sm border border-slate-100 max-w-[85%] text-xs text-slate-700">
                    Certainly! Our enterprise plan includes WhatsApp, Website Live Chat, SLA routing, and Captain AI copilot.
                  </div>
                </div>

                {/* Widget Footer Input */}
                <div className="border-t border-slate-200 p-2.5 bg-white flex items-center gap-2">
                  <input
                    type="text"
                    disabled
                    placeholder="Write a message..."
                    className="flex-1 rounded-md border border-slate-200 px-3 py-1.5 text-xs text-slate-400 bg-slate-50 cursor-not-allowed"
                  />
                  <div
                    className="h-7 w-7 rounded-md flex items-center justify-center text-white text-xs shrink-0"
                    style={{ backgroundColor: widgetColor }}
                  >
                    ➤
                  </div>
                </div>
              </div>

              {/* Launcher Bubble */}
              <div className="mt-4 flex items-center justify-center gap-2 text-xs text-slate-500">
                <span>Launcher button preview:</span>
                <div
                  className="h-8 w-8 rounded-full flex items-center justify-center text-white shadow-md text-sm"
                  style={{ backgroundColor: widgetColor }}
                >
                  💬
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: PRE-CHAT FORM */}
      {activeTab === 'prechat' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-7 space-y-6">
            <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-5">
              <div className="flex items-center justify-between border-b border-slate-100 pb-4">
                <div>
                  <h2 className="text-base font-semibold text-slate-900">Pre-Chat Qualification Form</h2>
                  <p className="text-xs text-slate-500 mt-0.5">
                    Collect visitor credentials before opening the chat window to generate verified leads.
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={preChatEnabled}
                    onChange={(e) => setPreChatEnabled(e.target.checked)}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                </label>
              </div>

              {preChatEnabled ? (
                <div className="space-y-4">
                  <div className="text-xs font-semibold text-slate-700">Configured Form Fields</div>
                  <div className="divide-y divide-slate-100 rounded-lg border border-slate-200 overflow-hidden">
                    {preChatFields.map((field, idx) => (
                      <div key={field.name} className="flex items-center justify-between p-3.5 bg-white">
                        <div className="flex items-center gap-3">
                          <span className="text-slate-400 font-mono text-xs">#{idx + 1}</span>
                          <div>
                            <div className="text-xs font-semibold text-slate-800">{field.label}</div>
                            <div className="text-[10px] font-mono text-slate-400">key: {field.name}</div>
                          </div>
                        </div>
                        <div className="flex items-center gap-3">
                          <label className="flex items-center gap-1.5 text-xs text-slate-600 cursor-pointer">
                            <input
                              type="checkbox"
                              checked={field.required}
                              onChange={() => toggleFieldRequired(idx)}
                              className="rounded border-slate-300 text-blue-600"
                            />
                            <span>Required</span>
                          </label>
                          <span
                            className={`rounded px-1.5 py-0.5 text-[10px] font-semibold ${
                              field.required
                                ? 'bg-amber-50 text-amber-700 border border-amber-200'
                                : 'bg-slate-100 text-slate-600'
                            }`}
                          >
                            {field.required ? 'Mandatory' : 'Optional'}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>

                  <div className="rounded-lg bg-blue-50 border border-blue-200 p-3.5 text-xs text-blue-800">
                    <span className="font-bold">Lead Automation Note:</span> Visitors completing this form are automatically enriched as high-intent <code className="font-semibold text-blue-900">Lead</code> entities, synced with CRM contacts, and queued in the Shared Inbox under the <code className="font-semibold text-blue-900">Website Live Chat</code> inbox.
                  </div>

                  <div className="pt-2">
                    <button
                      onClick={handleSave}
                      disabled={saving}
                      className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                    >
                      {saving ? 'Saving...' : 'Save Pre-Chat Settings'}
                    </button>
                  </div>
                </div>
              ) : (
                <div className="rounded-lg bg-slate-50 border border-slate-200 p-6 text-center">
                  <span className="text-2xl">⚡</span>
                  <div className="mt-2 text-xs font-semibold text-slate-700">Pre-chat form is currently disabled</div>
                  <p className="mt-1 text-[11px] text-slate-500 max-w-sm mx-auto">
                    Visitors will be able to send messages immediately without providing their email or name first. Anonymous sessions will be tracked by browser cookie.
                  </p>
                  <button
                    onClick={() => setPreChatEnabled(true)}
                    className="mt-3 rounded-md bg-blue-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow hover:bg-blue-700"
                  >
                    Enable Pre-Chat Form
                  </button>
                </div>
              )}
            </div>
          </div>

          {/* Form Visual Preview */}
          <div className="lg:col-span-5">
            <div className="sticky top-6 rounded-xl border border-slate-200 bg-slate-100 p-5 shadow-sm">
              <div className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-3">
                Pre-Chat Form Preview
              </div>

              <div className="mx-auto max-w-[340px] rounded-2xl bg-white shadow-xl border border-slate-200 overflow-hidden">
                <div className="p-4 text-white" style={{ backgroundColor: widgetColor }}>
                  <div className="font-bold text-sm tracking-tight">{greetingTitle}</div>
                  <div className="text-[11px] text-white/80">{greetingTagline}</div>
                </div>

                <div className="p-4 space-y-3.5">
                  <div className="text-xs font-medium text-slate-600">
                    Please fill out the form below to start chatting with our team:
                  </div>

                  {preChatFields.map((field) => (
                    <div key={field.name} className="space-y-1">
                      <label className="block text-[11px] font-semibold text-slate-700">
                        {field.label} {field.required && <span className="text-red-500">*</span>}
                      </label>
                      <input
                        type="text"
                        disabled
                        placeholder={`Enter your ${field.label.toLowerCase()}`}
                        className="w-full rounded-md border border-slate-300 px-2.5 py-1.5 text-xs text-slate-400 bg-slate-50 cursor-not-allowed"
                      />
                    </div>
                  ))}

                  <div className="space-y-1">
                    <label className="block text-[11px] font-semibold text-slate-700">
                      Message <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      disabled
                      rows={2}
                      placeholder="How can we help?"
                      className="w-full rounded-md border border-slate-300 px-2.5 py-1.5 text-xs text-slate-400 bg-slate-50 cursor-not-allowed resize-none"
                    />
                  </div>

                  <button
                    disabled
                    className="w-full rounded-lg text-white font-semibold py-2 text-xs shadow-sm cursor-not-allowed opacity-90"
                    style={{ backgroundColor: widgetColor }}
                  >
                    Start Chatting →
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 4: SECURITY & IDENTITY VALIDATION */}
      {activeTab === 'security' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-5">
            <div className="flex items-center justify-between border-b border-slate-100 pb-4">
              <div>
                <h2 className="text-base font-semibold text-slate-900">User Identity Verification (HMAC-SHA256)</h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Enforces cryptographic signature checks to prevent malicious actors from impersonating logged-in customers.
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={identityValidation}
                  onChange={(e) => setIdentityValidation(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">HMAC Secret Key</label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={hmacSecret}
                    onChange={(e) => setHmacSecret(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono focus:border-blue-500 focus:outline-none"
                    placeholder="whsec_..."
                  />
                  <button
                    onClick={() => copyToClipboard(hmacSecret, 'hmac')}
                    className="rounded-lg border border-slate-300 px-3.5 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 shrink-0"
                  >
                    {copiedKey === 'hmac' ? 'Copied!' : 'Copy Key'}
                  </button>
                </div>
                <p className="text-[11px] text-slate-400 mt-1">
                  Keep this secret safe on your server. Do not expose this secret in client-side HTML or JS.
                </p>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Whitelisted Domain</label>
                <input
                  type="text"
                  value={websiteDomain}
                  onChange={(e) => setWebsiteDomain(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="bayworks.com"
                />
                <p className="text-[11px] text-slate-400 mt-1">
                  Prevents unauthorized domains from embedding your live chat widget.
                </p>
              </div>

              <div className="pt-2">
                <button
                  onClick={handleSave}
                  disabled={saving}
                  className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                >
                  {saving ? 'Saving...' : 'Save Security Settings'}
                </button>
              </div>
            </div>
          </div>

          {/* Server-Side Implementation Examples */}
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-4">
            <h3 className="text-sm font-semibold text-slate-900">How to generate HMAC identifier hash</h3>
            <p className="text-xs text-slate-500">
              When a user signs into your app, calculate the HMAC-SHA256 signature of their user identifier (e.g. email or user ID) using your HMAC secret key, then provide it to the SDK:
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <div className="text-xs font-semibold text-slate-700">Node.js / Express / Next.js</div>
                <pre className="rounded-lg bg-slate-950 p-3 text-[11px] font-mono text-emerald-400 overflow-x-auto">
{`const crypto = require('crypto');

function getIdentifierHash(userId) {
  return crypto
    .createHmac('sha256', '${hmacSecret || 'whsec_secret'}')
    .update(userId.toString())
    .digest('hex');
}`}
                </pre>
              </div>

              <div className="space-y-2">
                <div className="text-xs font-semibold text-slate-700">Python / FastAPI / Django</div>
                <pre className="rounded-lg bg-slate-950 p-3 text-[11px] font-mono text-emerald-400 overflow-x-auto">
{`import hmac
import hashlib

def get_identifier_hash(user_id: str) -> str:
    secret = b"${hmacSecret || 'whsec_secret'}"
    return hmac.new(
        secret, user_id.encode('utf-8'), hashlib.sha256
    ).hexdigest()`}
                </pre>
              </div>
            </div>

            <div className="rounded-lg bg-slate-50 border border-slate-200 p-4 text-xs font-mono text-slate-700">
              <div className="font-bold text-slate-900 mb-1">// Client-side SDK invocation:</div>
              {`window.$chatwoot.setUser(user.id, {
  name: user.fullName,
  email: user.email,
  identifier_hash: "${hmacSecret ? 'hmac_sha256_hash_here' : 'computed_hash'}"
});`}
            </div>
          </div>
        </div>
      )}

      {/* TAB 5: AUTOMATIONS & WORKING HOURS */}
      {activeTab === 'automations' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-6">
            <h2 className="text-base font-semibold text-slate-900 border-b border-slate-100 pb-3">
              Automations & Response Rules
            </h2>

            {/* Greeting */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Auto Greeting Message</div>
                  <div className="text-[11px] text-slate-500">Automatically post a welcome message when visitor opens live chat</div>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-blue-600"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="👋 Welcome to our live chat! How can our support team help you today?"
                />
              )}
            </div>

            {/* Out of office */}
            <div className="space-y-3 pt-3 border-t border-slate-100">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Out of Office / Away Reply</div>
                  <div className="text-[11px] text-slate-500">Send an automated reply when visitors message outside business hours</div>
                </div>
                <input
                  type="checkbox"
                  checked={outOfOfficeEnabled}
                  onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-blue-600"
                />
              </div>
              {outOfOfficeEnabled && (
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="Our team is currently offline. Please leave your email and message, and we will get back to you shortly!"
                />
              )}
            </div>

            {/* CSAT */}
            <div className="pt-3 border-t border-slate-100 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-800">Customer Satisfaction Survey (CSAT)</div>
                <div className="text-[11px] text-slate-500">Prompt visitors to rate support satisfaction (⭐ 1-5) once conversation is marked resolved</div>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="rounded border-slate-300 text-blue-600"
              />
            </div>

            {/* Working Hours */}
            <div className="pt-3 border-t border-slate-100 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Working Hours Enforcement</div>
                  <div className="text-[11px] text-slate-500">Set active support hours to toggle agent availability status</div>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="rounded border-slate-300 text-blue-600"
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
                      placeholder="UTC or America/New_York"
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
                {saving ? 'Saving...' : 'Save Automation Rules'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* TAB 6: LIVE PREVIEW & SIMULATOR */}
      {activeTab === 'simulator' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-6 space-y-6">
            <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-4">
              <div>
                <h2 className="text-base font-semibold text-slate-900">Live Visitor Test Simulator</h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Send a live test message as a website visitor. This immediately registers a lead, triggers your live chat greeting, and routes into the Shared Inbox.
                </p>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Visitor Full Name</label>
                  <input
                    type="text"
                    value={simName}
                    onChange={(e) => setSimName(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="Alex Mercer"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Visitor Email</label>
                  <input
                    type="email"
                    value={simEmail}
                    onChange={(e) => setSimEmail(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="alex.mercer@example.com"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Phone Number</label>
                  <input
                    type="text"
                    value={simPhone}
                    onChange={(e) => setSimPhone(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="+1 (555) 234-5678"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Company</label>
                  <input
                    type="text"
                    value={simCompany}
                    onChange={(e) => setSimCompany(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="Acme Corporation"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Visitor Message</label>
                <textarea
                  rows={3}
                  value={simMessage}
                  onChange={(e) => setSimMessage(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 p-2.5 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="Hi! I have a question about your enterprise live chat features and setup."
                />
              </div>

              <div className="flex items-center gap-3 pt-2">
                <button
                  onClick={handleRunSimulator}
                  disabled={simulating}
                  className="rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50 flex items-center gap-2"
                >
                  <span>{simulating ? 'Sending...' : '⚡ Submit Visitor Message'}</span>
                </button>
                {simResult && (
                  <Link
                    to="/"
                    className="rounded-lg border border-emerald-300 bg-emerald-50 px-3.5 py-2 text-xs font-semibold text-emerald-800 hover:bg-emerald-100 flex items-center gap-1.5"
                  >
                    <span>View in Shared Inbox →</span>
                  </Link>
                )}
              </div>

              {simResult && (
                <div className="rounded-lg bg-emerald-50 border border-emerald-200 p-3.5 text-xs text-emerald-900 space-y-1">
                  <div className="font-bold flex items-center gap-1.5">
                    <span>✓ Conversation Created:</span>
                    <span className="font-mono text-emerald-700">{simResult.conversationId.slice(0, 12)}...</span>
                  </div>
                  <p className="text-[11px] text-emerald-800">{simResult.message}</p>
                </div>
              )}
            </div>
          </div>

          {/* Real-time Events Log */}
          <div className="lg:col-span-6">
            <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm space-y-4">
              <div className="flex items-center justify-between border-b border-slate-100 pb-3">
                <div>
                  <h3 className="text-sm font-semibold text-slate-900">Recent Live Chat Events</h3>
                  <p className="text-xs text-slate-500">Live incoming visitor messages and agent responses</p>
                </div>
                <button
                  onClick={async () => {
                    const ev = await api.listLiveChatEvents()
                    setEvents(ev)
                  }}
                  className="text-xs font-semibold text-blue-600 hover:text-blue-800"
                >
                  🔄 Refresh
                </button>
              </div>

              {events.length === 0 ? (
                <div className="py-8 text-center text-xs text-slate-400">
                  No live chat message events logged yet. Use the simulator on the left to start a conversation!
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
                                : 'bg-emerald-100 text-emerald-800'
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
