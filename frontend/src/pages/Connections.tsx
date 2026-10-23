import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type ChannelGateway,
  type IntegrationSettings,
  type IssueTrackerConfig,
  type StripeConfig,
  type SupportedProvidersCatalog,
  type Template,
} from '../lib/api'

export function Connections() {
  const [activeTab, setActiveTab] = useState<'gateways' | 'stripe' | 'trackers' | 'whatsapp' | 'business' | 'templates'>('gateways')
  const [data, setData] = useState<IntegrationSettings | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  function load() {
    api
      .getIntegrationSettings()
      .then(setData)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Connections & Universal Gateways</h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Connect any SMS provider, any Email provider, any VoIP/SIP trunk, and manage multi-channel credentials.
          </p>
        </div>

        <Link
          to="/self-improving"
          className="inline-flex items-center gap-1.5 self-start rounded-md bg-[var(--color-accent-soft)] border border-[var(--color-border)] px-3 py-1.5 text-xs font-medium text-[var(--color-accent)] hover:bg-indigo-100 transition-colors"
        >
          <span>🤖</span>
          <span>Configure AI Model Hub</span>
          <span className="text-indigo-400">→</span>
        </Link>
      </div>

      {error && <Banner tone="error">{error}</Banner>}
      {notice && <Banner tone="success">{notice}</Banner>}

      {/* Primary Tabs */}
      <div className="flex border-b border-[var(--color-border)] overflow-x-auto gap-2">
        <button
          onClick={() => setActiveTab('gateways')}
          className={`py-2 px-3 border-b-2 font-medium text-sm whitespace-nowrap transition-colors ${
            activeTab === 'gateways'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          🌐 Universal Gateways (Any SMS, Email, VoIP/SIP)
        </button>
        <button
          onClick={() => setActiveTab('stripe')}
          className={`py-2 px-3 border-b-2 font-medium text-sm whitespace-nowrap transition-colors flex items-center gap-1.5 ${
            activeTab === 'stripe'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          <span>💳</span>
          <span>Stripe Billing & Payments</span>
        </button>
        <button
          onClick={() => setActiveTab('trackers')}
          className={`py-2 px-3 border-b-2 font-medium text-sm whitespace-nowrap transition-colors flex items-center gap-1.5 ${
            activeTab === 'trackers'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          <span>🎯</span>
          <span>Issue Trackers & Slack (Linear / GitHub / Jira)</span>
        </button>
        <button
          onClick={() => setActiveTab('whatsapp')}
          className={`py-2 px-3 border-b-2 font-medium text-sm whitespace-nowrap transition-colors ${
            activeTab === 'whatsapp'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          💬 WhatsApp Cloud API
        </button>
        <button
          onClick={() => setActiveTab('business')}
          className={`py-2 px-3 border-b-2 font-medium text-sm whitespace-nowrap transition-colors ${
            activeTab === 'business'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          🏢 Business Profile & Hours
        </button>
        <button
          onClick={() => setActiveTab('templates')}
          className={`py-2 px-3 border-b-2 font-medium text-sm whitespace-nowrap transition-colors ${
            activeTab === 'templates'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          📋 Meta HSM Templates
        </button>
      </div>

      {activeTab === 'gateways' && (
        <UniversalGatewaysSection onNotice={setNotice} onError={setError} />
      )}

      {activeTab === 'stripe' && (
        <StripeBillingSection onNotice={setNotice} onError={setError} />
      )}

      {activeTab === 'trackers' && (
        <IssueTrackersSection onNotice={setNotice} onError={setError} />
      )}

      {activeTab === 'whatsapp' && (
        data ? (
          <WhatsAppSection
            data={data}
            onSaved={(msg) => {
              setNotice(msg)
              load()
            }}
            onError={setError}
          />
        ) : (
          <p className="text-sm text-[var(--color-ink-muted)]">Loading…</p>
        )
      )}

      {activeTab === 'business' && (
        data ? (
          <BusinessProfileSection
            data={data}
            onSaved={(msg) => {
              setNotice(msg)
              load()
            }}
            onError={setError}
          />
        ) : (
          <p className="text-sm text-[var(--color-ink-muted)]">Loading…</p>
        )
      )}

      {activeTab === 'templates' && (
        data ? (
          <TemplatesSection onError={setError} wabaConfigured={!!data.whatsapp.wabaId} />
        ) : (
          <p className="text-sm text-[var(--color-ink-muted)]">Loading…</p>
        )
      )}
    </div>
  )
}

function UniversalGatewaysSection({
  onNotice,
  onError,
}: {
  onNotice: (msg: string) => void
  onError: (msg: string) => void
}) {
  const [gateways, setGateways] = useState<ChannelGateway[]>([])
  const [catalog, setCatalog] = useState<SupportedProvidersCatalog | null>(null)
  const [filterType, setFilterType] = useState<string>('all')
  const [loading, setLoading] = useState(false)
  const [testingId, setTestingId] = useState<string | null>(null)
  const [showAddModal, setShowAddModal] = useState(false)
  const [pingModalGateway, setPingModalGateway] = useState<ChannelGateway | null>(null)

  function loadGateways() {
    setLoading(true)
    api
      .listGateways(filterType === 'all' ? undefined : filterType)
      .then(setGateways)
      .catch((e) => onError(e.message))
      .finally(() => setLoading(false))
  }

  useEffect(() => {
    loadGateways()
  }, [filterType])

  useEffect(() => {
    api.getSupportedGatewayProviders().then(setCatalog).catch(console.error)
  }, [])

  async function handleTestConnection(id: string) {
    setTestingId(id)
    try {
      const res = await api.testGatewayConnection(id)
      if (res.success) {
        onNotice(`Connection test successful! Ping latency: ${res.latency_ms}ms`)
      } else {
        onError(`Connection test failed: ${res.error || 'Unknown error'}`)
      }
      loadGateways()
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setTestingId(null)
    }
  }

  async function handleSetDefault(id: string) {
    try {
      await api.setDefaultGateway(id)
      onNotice('Default gateway updated successfully.')
      loadGateways()
    } catch (e) {
      onError((e as Error).message)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this gateway connection?')) return
    try {
      await api.deleteGateway(id)
      onNotice('Gateway removed.')
      loadGateways()
    } catch (e) {
      onError((e as Error).message)
    }
  }

  return (
    <div className="space-y-6">
      {/* Overview & Action Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 bg-[var(--color-surface)] p-4 rounded-lg border border-[var(--color-border)]">
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mr-2">Filter:</span>
          {[
            { id: 'all', label: 'All Gateways' },
            { id: 'sms', label: '📱 SMS Gateways' },
            { id: 'email', label: '📧 Email Transports' },
            { id: 'voice_sip', label: '📞 VoIP & SIP Trunks' },
          ].map((item) => (
            <button
              key={item.id}
              onClick={() => setFilterType(item.id)}
              className={`rounded-full px-3 py-1 text-xs font-medium transition-colors ${
                filterType === item.id
                  ? 'bg-[var(--color-accent)] text-white'
                  : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
              }`}
            >
              {item.label}
            </button>
          ))}
        </div>

        <button
          onClick={() => setShowAddModal(true)}
          className="inline-flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] px-3.5 py-1.5 text-xs font-medium text-white shadow-sm transition-colors"
        >
          <span>+</span>
          <span>Connect Any Provider</span>
        </button>
      </div>

      {/* Gateway Cards Grid */}
      {loading ? (
        <p className="text-sm text-[var(--color-ink-muted)] py-6 text-center">Loading gateways…</p>
      ) : gateways.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 p-8 text-center bg-[var(--color-surface)]">
          <p className="text-sm text-[var(--color-ink-muted)]">No gateways configured for this filter.</p>
          <button
            onClick={() => setShowAddModal(true)}
            className="mt-3 text-xs font-medium text-[var(--color-accent)] hover:underline"
          >
            + Connect your first provider
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {gateways.map((gw) => (
            <div
              key={gw.id}
              className={`flex flex-col justify-between rounded-lg border bg-[var(--color-surface)] p-4 shadow-xs transition-shadow hover:shadow-sm ${
                gw.is_default ? 'border-indigo-300 ring-1 ring-indigo-200' : 'border-[var(--color-border)]'
              }`}
            >
              <div>
                <div className="flex items-start justify-between gap-2 mb-2">
                  <div className="min-w-0">
                    <div className="flex items-center gap-1.5 flex-wrap">
                      <span className="text-base font-semibold text-[var(--color-ink)] truncate">{gw.name}</span>
                      {gw.is_default && (
                        <span className="inline-flex items-center rounded-full bg-[var(--color-accent-soft)] px-2 py-0.5 text-3xs font-semibold text-[var(--color-accent)] border border-[var(--color-border)]">
                          ★ Default
                        </span>
                      )}
                    </div>
                    <div className="mt-0.5 flex items-center gap-2 text-xs text-[var(--color-ink-muted)]">
                      <span className="font-mono text-2xs uppercase px-1.5 py-0.5 bg-slate-100 rounded text-[var(--color-ink-muted)]">
                        {gw.provider}
                      </span>
                      <span>·</span>
                      <span className="capitalize">{gw.gateway_type.replace('_', ' ')}</span>
                    </div>
                  </div>

                  {/* Status Indicator */}
                  <span
                    className={`shrink-0 inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-2xs font-medium ${
                      gw.test_status === 'connected'
                        ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                        : gw.test_status === 'error'
                          ? 'bg-red-50 text-red-700 border border-red-200'
                          : 'bg-slate-100 text-[var(--color-ink-muted)] border border-[var(--color-border)]'
                    }`}
                  >
                    <span
                      className={`h-1.5 w-1.5 rounded-full ${
                        gw.test_status === 'connected'
                          ? 'bg-emerald-500'
                          : gw.test_status === 'error'
                            ? 'bg-red-500'
                            : 'bg-slate-400'
                      }`}
                    />
                    {gw.test_status === 'connected'
                      ? `${gw.latency_ms}ms`
                      : gw.test_status === 'error'
                        ? 'Error'
                        : 'Untested'}
                  </span>
                </div>

                {/* Configuration Specs Preview */}
                <div className="my-3 rounded-md bg-[var(--color-page)] p-2.5 text-xs text-[var(--color-ink-muted)] space-y-1 font-mono">
                  {gw.gateway_type === 'sms' && (
                    <>
                      {gw.config?.from_number && <div>From: {gw.config.from_number}</div>}
                      {gw.config?.endpoint_url && <div className="truncate">URL: {gw.config.endpoint_url}</div>}
                    </>
                  )}
                  {gw.gateway_type === 'email' && (
                    <>
                      {gw.config?.smtp_host && <div>Host: {gw.config.smtp_host}:{gw.config.smtp_port || 587}</div>}
                      {gw.config?.from_address && <div>From: {gw.config.from_address}</div>}
                    </>
                  )}
                  {gw.gateway_type === 'voice_sip' && (
                    <>
                      {gw.config?.sip_server && <div>SIP: {gw.config.sip_server}:{gw.config.sip_port || 5060}</div>}
                      {gw.config?.caller_id && <div>Caller ID: {gw.config.caller_id}</div>}
                    </>
                  )}
                  {Object.entries(gw.credentials || {}).map(([k, v]) => (
                    <div key={k} className="text-2xs text-[var(--color-ink-muted)]">
                      {k}: <span className="text-[var(--color-ink)]">{v || 'configured'}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Action Toolbar */}
              <div className="mt-3 pt-3 border-t border-[var(--color-border)] flex items-center justify-between text-xs gap-2">
                <div className="flex items-center gap-1.5">
                  <button
                    onClick={() => handleTestConnection(gw.id)}
                    disabled={testingId === gw.id}
                    className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50"
                  >
                    {testingId === gw.id ? 'Pinging…' : '⚡ Test'}
                  </button>
                  <button
                    onClick={() => setPingModalGateway(gw)}
                    className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)]"
                  >
                    ✉ Ping
                  </button>
                  {!gw.is_default && (
                    <button
                      onClick={() => handleSetDefault(gw.id)}
                      className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-page)]"
                    >
                      ★ Default
                    </button>
                  )}
                </div>

                <button
                  onClick={() => handleDelete(gw.id)}
                  className="text-[var(--color-ink-faint)] hover:text-red-600 transition-colors p-1"
                  title="Delete Gateway"
                >
                  ✕
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Connect Any Provider Modal */}
      {showAddModal && (
        <AddGatewayModal
          catalog={catalog}
          onClose={() => setShowAddModal(false)}
          onCreated={() => {
            setShowAddModal(false)
            onNotice('New gateway connected successfully!')
            loadGateways()
          }}
          onError={onError}
        />
      )}

      {/* Send Test Ping Modal */}
      {pingModalGateway && (
        <SendPingModal
          gateway={pingModalGateway}
          onClose={() => setPingModalGateway(null)}
          onNotice={onNotice}
          onError={onError}
        />
      )}
    </div>
  )
}

function AddGatewayModal({
  catalog,
  onClose,
  onCreated,
  onError,
}: {
  catalog: SupportedProvidersCatalog | null
  onClose: () => void
  onCreated: () => void
  onError: (msg: string) => void
}) {
  const [gatewayType, setGatewayType] = useState<'sms' | 'email' | 'voice_sip'>('sms')
  const [provider, setProvider] = useState<string>('twilio')
  const [name, setName] = useState('')
  const [isDefault, setIsDefault] = useState(false)
  const [saving, setSaving] = useState(false)

  // Dynamic field states
  const [accountSid, setAccountSid] = useState('')
  const [authToken, setAuthToken] = useState('')
  const [fromNumber, setFromNumber] = useState('')
  const [endpointUrl, setEndpointUrl] = useState('')
  const [bearerToken, setBearerToken] = useState('')
  const [smtpHost, setSmtpHost] = useState('')
  const [smtpPort, setSmtpPort] = useState(587)
  const [smtpUsername, setSmtpUsername] = useState('')
  const [smtpPassword, setSmtpPassword] = useState('')
  const [fromAddress, setFromAddress] = useState('')
  const [fromName, setFromName] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [sipServer, setSipServer] = useState('')
  const [sipPort, setSipPort] = useState(5060)
  const [sipUser, setSipUser] = useState('')
  const [sipPass, setSipPass] = useState('')

  // Set default provider on type switch
  useEffect(() => {
    if (gatewayType === 'sms') {
      setProvider('vonage')
      setName('Vonage SMS Global')
    } else if (gatewayType === 'email') {
      setProvider('sendgrid')
      setName('SendGrid Production Mailer')
    } else {
      setProvider('telnyx')
      setName('Telnyx SIP & Voice')
    }
  }, [gatewayType])

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)

    const credentials: Record<string, any> = {}
    const config: Record<string, any> = {}

    if (gatewayType === 'sms') {
      if (provider === 'custom_http') {
        config.endpoint_url = endpointUrl
        credentials.bearer_token = bearerToken
        config.payload_template = '{"to": "{{to}}", "message": "{{body}}"}'
      } else if (provider === 'vonage') {
        credentials.api_key = apiKey
        credentials.api_secret = authToken
        config.from = fromNumber || 'BayWorks'
      } else if (provider === 'messagebird') {
        credentials.access_key = apiKey
        config.originator = fromNumber || 'BayWorks'
      } else {
        credentials.account_sid = accountSid
        credentials.auth_token = authToken
        config.from_number = fromNumber
      }
    } else if (gatewayType === 'email') {
      if (provider === 'smtp') {
        config.smtp_host = smtpHost
        config.smtp_port = Number(smtpPort)
        credentials.smtp_username = smtpUsername
        credentials.smtp_password = smtpPassword
        config.from_address = fromAddress
        config.from_name = fromName
      } else {
        credentials.api_key = apiKey
        config.from_address = fromAddress
        config.from_name = fromName
      }
    } else if (gatewayType === 'voice_sip') {
      if (provider === 'sip_trunk') {
        config.sip_server = sipServer
        config.sip_port = Number(sipPort)
        credentials.sip_username = sipUser
        credentials.sip_password = sipPass
        config.caller_id = fromNumber
      } else {
        credentials.account_sid = accountSid
        credentials.auth_token = authToken
        credentials.api_key = apiKey
        config.caller_id = fromNumber
      }
    }

    try {
      await api.createGateway({
        gateway_type: gatewayType,
        provider,
        name: name || `${provider.toUpperCase()} Gateway`,
        credentials,
        config,
        is_default: isDefault,
      })
      onCreated()
    } catch (err) {
      onError((err as Error).message)
    } finally {
      setSaving(false)
    }
  }

  const providerList = catalog ? catalog[gatewayType] || [] : []

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 p-4 backdrop-blur-xs">
      <div className="w-full max-w-lg rounded-xl bg-[var(--color-surface)] shadow-xl border border-[var(--color-border)] overflow-hidden max-h-[90vh] flex flex-col">
        <div className="border-b border-[var(--color-border)] px-5 py-3.5 flex items-center justify-between bg-[var(--color-page)]">
          <h2 className="text-sm font-semibold text-[var(--color-ink)]">+ Connect Any Provider</h2>
          <button onClick={onClose} className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] text-sm">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 space-y-4 overflow-y-auto flex-1 text-xs">
          {/* Gateway Type Selection */}
          <div>
            <label className="block font-medium text-[var(--color-ink)] mb-1">Gateway Type</label>
            <div className="grid grid-cols-3 gap-2">
              {[
                { id: 'sms', label: '📱 SMS' },
                { id: 'email', label: '📧 Email' },
                { id: 'voice_sip', label: '📞 VoIP / SIP' },
              ].map((t) => (
                <button
                  key={t.id}
                  type="button"
                  onClick={() => setGatewayType(t.id as any)}
                  className={`py-1.5 px-3 rounded-md border text-center font-medium ${
                    gatewayType === t.id
                      ? 'border-indigo-600 bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                      : 'border-[var(--color-border)] hover:bg-[var(--color-page)] text-[var(--color-ink)]'
                  }`}
                >
                  {t.label}
                </button>
              ))}
            </div>
          </div>

          {/* Provider Selection */}
          <div>
            <label className="block font-medium text-[var(--color-ink)] mb-1">Select Provider</label>
            <select
              value={provider}
              onChange={(e) => setProvider(e.target.value)}
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs text-slate-800"
            >
              {providerList.map((p) => (
                <option key={p.provider} value={p.provider}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block font-medium text-[var(--color-ink)] mb-1">Gateway Name</label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
              placeholder="e.g. Production Vonage Gateway"
            />
          </div>

          {/* Dynamic Provider Input Fields */}
          {provider === 'custom_http' && (
            <div className="space-y-3 p-3 bg-[var(--color-page)] rounded-md border border-[var(--color-border)]">
              <p className="font-semibold text-[var(--color-ink)]">Custom HTTP REST Gateway Settings</p>
              <div>
                <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Endpoint URL</label>
                <input
                  type="url"
                  required
                  value={endpointUrl}
                  onChange={(e) => setEndpointUrl(e.target.value)}
                  className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  placeholder="https://sms-gw.internal.company.com/v1/send"
                />
              </div>
              <div>
                <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Bearer Token / Secret Key</label>
                <input
                  type="password"
                  value={bearerToken}
                  onChange={(e) => setBearerToken(e.target.value)}
                  className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  placeholder="Bearer token"
                />
              </div>
            </div>
          )}

          {provider === 'smtp' && (
            <div className="space-y-3 p-3 bg-[var(--color-page)] rounded-md border border-[var(--color-border)]">
              <p className="font-semibold text-[var(--color-ink)]">SMTP Server Configuration</p>
              <div className="grid grid-cols-3 gap-2">
                <div className="col-span-2">
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">SMTP Host</label>
                  <input
                    type="text"
                    required
                    value={smtpHost}
                    onChange={(e) => setSmtpHost(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                    placeholder="smtp.example.com"
                  />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Port</label>
                  <input
                    type="number"
                    value={smtpPort}
                    onChange={(e) => setSmtpPort(Number(e.target.value))}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Username</label>
                  <input
                    type="text"
                    value={smtpUsername}
                    onChange={(e) => setSmtpUsername(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Password</label>
                  <input
                    type="password"
                    value={smtpPassword}
                    onChange={(e) => setSmtpPassword(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">From Address</label>
                  <input
                    type="email"
                    value={fromAddress}
                    onChange={(e) => setFromAddress(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                    placeholder="support@company.com"
                  />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">From Name</label>
                  <input
                    type="text"
                    value={fromName}
                    onChange={(e) => setFromName(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                    placeholder="Support Team"
                  />
                </div>
              </div>
            </div>
          )}

          {provider === 'sip_trunk' && (
            <div className="space-y-3 p-3 bg-[var(--color-page)] rounded-md border border-[var(--color-border)]">
              <p className="font-semibold text-[var(--color-ink)]">Generic SIP Trunk / PBX Settings</p>
              <div className="grid grid-cols-3 gap-2">
                <div className="col-span-2">
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">SIP Server IP / Host</label>
                  <input
                    type="text"
                    required
                    value={sipServer}
                    onChange={(e) => setSipServer(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                    placeholder="pbx.company.com"
                  />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">SIP Port</label>
                  <input
                    type="number"
                    value={sipPort}
                    onChange={(e) => setSipPort(Number(e.target.value))}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">SIP Auth User</label>
                  <input
                    type="text"
                    value={sipUser}
                    onChange={(e) => setSipUser(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                    placeholder="1001"
                  />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">SIP Auth Secret</label>
                  <input
                    type="password"
                    value={sipPass}
                    onChange={(e) => setSipPass(e.target.value)}
                    className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  />
                </div>
              </div>
              <div>
                <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Outbound Caller ID</label>
                <input
                  type="text"
                  value={fromNumber}
                  onChange={(e) => setFromNumber(e.target.value)}
                  className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  placeholder="+14155550199 or extension"
                />
              </div>
            </div>
          )}

          {/* Generic API Key / Secret Form for REST Providers */}
          {provider !== 'custom_http' && provider !== 'smtp' && provider !== 'sip_trunk' && (
            <div className="space-y-3 p-3 bg-[var(--color-page)] rounded-md border border-[var(--color-border)]">
              <p className="font-semibold text-[var(--color-ink)]">{provider.toUpperCase()} Credentials</p>
              <div>
                <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">API Key / Token / Account SID</label>
                <input
                  type="password"
                  required
                  value={apiKey || accountSid}
                  onChange={(e) => {
                    setApiKey(e.target.value)
                    setAccountSid(e.target.value)
                  }}
                  className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  placeholder="Paste credentials"
                />
              </div>
              <div>
                <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Secret / Auth Token (if required)</label>
                <input
                  type="password"
                  value={authToken}
                  onChange={(e) => setAuthToken(e.target.value)}
                  className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  placeholder="Auth Token / Secret"
                />
              </div>
              <div>
                <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Sender ID / Phone / From Email</label>
                <input
                  type="text"
                  value={fromNumber || fromAddress}
                  onChange={(e) => {
                    setFromNumber(e.target.value)
                    setFromAddress(e.target.value)
                  }}
                  className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
                  placeholder="+14155550100 or support@company.com"
                />
              </div>
            </div>
          )}

          {/* Set as Default Checkbox */}
          <label className="flex items-center gap-2 text-xs text-[var(--color-ink)] pt-2">
            <input
              type="checkbox"
              checked={isDefault}
              onChange={(e) => setIsDefault(e.target.checked)}
              className="h-4 w-4 rounded border-slate-300 text-[var(--color-accent)]"
            />
            <span>Set as active default gateway for {gatewayType.toUpperCase()}</span>
          </label>

          <div className="flex justify-end gap-2 pt-4 border-t border-[var(--color-border)]">
            <button
              type="button"
              onClick={onClose}
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="rounded-md bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] px-4 py-1.5 text-xs font-medium text-white disabled:opacity-50"
            >
              {saving ? 'Connecting…' : 'Save & Connect Provider'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

function SendPingModal({
  gateway,
  onClose,
  onNotice,
  onError,
}: {
  gateway: ChannelGateway
  onClose: () => void
  onNotice: (msg: string) => void
  onError: (msg: string) => void
}) {
  const [target, setTarget] = useState(
    gateway.gateway_type === 'email'
      ? 'test@example.com'
      : gateway.gateway_type === 'voice_sip'
        ? '+14155550199'
        : '+14155550100'
  )
  const [message, setMessage] = useState('Test transmission via Universal Gateway Hub')
  const [sending, setSending] = useState(false)

  async function handleSend(e: React.FormEvent) {
    e.preventDefault()
    setSending(true)
    try {
      const res = await api.sendGatewayPing(gateway.id, target, message)
      if (res.success) {
        onNotice(`Test transmission sent via ${res.provider}! Message ID: ${res.message_id} (${res.latency_ms}ms)`)
        onClose()
      } else {
        onError('Failed to send test ping.')
      }
    } catch (err) {
      onError((err as Error).message)
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 p-4 backdrop-blur-xs">
      <div className="w-full max-w-md rounded-xl bg-[var(--color-surface)] shadow-xl border border-[var(--color-border)] p-5 space-y-4 text-xs">
        <div className="flex items-center justify-between border-b pb-3 border-[var(--color-border)]">
          <h3 className="font-semibold text-[var(--color-ink)] text-sm">Send Test Ping · {gateway.name}</h3>
          <button onClick={onClose} className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)]">✕</button>
        </div>

        <form onSubmit={handleSend} className="space-y-3">
          <div>
            <label className="block text-2xs text-[var(--color-ink-muted)] mb-1">
              Destination ({gateway.gateway_type === 'email' ? 'Email Address' : 'Phone Number or SIP URI'})
            </label>
            <input
              type="text"
              required
              value={target}
              onChange={(e) => setTarget(e.target.value)}
              className="w-full rounded border border-slate-300 px-3 py-1.5 text-xs"
            />
          </div>

          <div>
            <label className="block text-2xs text-[var(--color-ink-muted)] mb-1">Test Payload</label>
            <textarea
              rows={2}
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              className="w-full rounded border border-slate-300 px-3 py-1.5 text-xs"
            />
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded border border-slate-300 px-3 py-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={sending}
              className="rounded bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] px-3 py-1.5 text-white font-medium disabled:opacity-50"
            >
              {sending ? 'Sending…' : 'Transmit Test Ping'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

function Banner({ tone, children }: { tone: 'error' | 'success'; children: React.ReactNode }) {
  const cls =
    tone === 'error'
      ? 'border-red-200 bg-red-50 text-red-700'
      : 'border-emerald-200 bg-emerald-50 text-emerald-700'
  return <div className={`rounded-md border px-4 py-3 text-sm ${cls}`}>{children}</div>
}

function SectionCard({
  title,
  description,
  children,
}: {
  title: string
  description: string
  children: React.ReactNode
}) {
  return (
    <section className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="border-b border-[var(--color-border)] px-5 py-4">
        <h2 className="text-sm font-semibold text-[var(--color-ink)]">{title}</h2>
        <p className="mt-0.5 text-xs text-[var(--color-ink-muted)]">{description}</p>
      </div>
      <div className="p-5">{children}</div>
    </section>
  )
}

function Field({
  label,
  value,
  onChange,
  placeholder,
  secret = false,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  secret?: boolean
}) {
  return (
    <div>
      <label className="block text-xs font-medium text-[var(--color-ink-muted)]">{label}</label>
      <input
        type={secret ? 'password' : 'text'}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
      />
    </div>
  )
}

function WhatsAppSection({
  data,
  onSaved,
  onError,
}: {
  data: IntegrationSettings
  onSaved: (msg: string) => void
  onError: (msg: string) => void
}) {
  const [accessToken, setAccessToken] = useState('')
  const [phoneNumberId, setPhoneNumberId] = useState(data.whatsapp.phoneNumberId)
  const [wabaId, setWabaId] = useState(data.whatsapp.wabaId)
  const [appSecret, setAppSecret] = useState('')
  const [verifyToken, setVerifyToken] = useState('')
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)

  const connected = !!data.whatsapp.verifiedAt

  async function save() {
    setSaving(true)
    try {
      await api.updateIntegrationSettings({
        whatsapp: {
          phone_number_id: phoneNumberId,
          waba_id: wabaId,
          ...(accessToken ? { access_token: accessToken } : {}),
          ...(appSecret ? { app_secret: appSecret } : {}),
          ...(verifyToken ? { webhook_verify_token: verifyToken } : {}),
        },
      })
      setAccessToken('')
      setAppSecret('')
      setVerifyToken('')
      onSaved('WhatsApp settings saved.')
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function verify() {
    setVerifying(true)
    try {
      const result = await api.verifyWhatsApp()
      onSaved(`Connected to ${result.verifiedName || result.displayNumber || 'WhatsApp'}.`)
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setVerifying(false)
    }
  }

  return (
    <SectionCard
      title="WhatsApp connection"
      description="Credentials from your Meta app's WhatsApp → API setup page."
    >
      <div className="mb-4 flex items-center gap-2">
        <span
          className={`inline-block h-2 w-2 rounded-full ${connected ? 'bg-emerald-500' : 'bg-slate-300'}`}
        />
        <span className="text-sm text-[var(--color-ink)]">
          {connected
            ? `Connected — ${data.whatsapp.verifiedName || data.whatsapp.displayNumber}`
            : 'Not verified yet'}
        </span>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Field
          label="Access token"
          value={accessToken}
          onChange={setAccessToken}
          placeholder={data.whatsapp.accessTokenMasked ?? 'Paste access token'}
          secret
        />
        <Field label="Phone number ID" value={phoneNumberId} onChange={setPhoneNumberId} />
        <Field
          label="WhatsApp Business Account ID"
          value={wabaId}
          onChange={setWabaId}
          placeholder="Needed for template messages"
        />
        <Field
          label="App secret"
          value={appSecret}
          onChange={setAppSecret}
          placeholder={data.whatsapp.appSecretMasked ?? 'From App settings → Basic'}
          secret
        />
      </div>

      <div className="mt-4 rounded-md border border-[var(--color-border)] bg-[var(--color-page)] px-4 py-3">
        <p className="text-xs font-medium text-[var(--color-ink-muted)]">Webhook setup (paste into Meta's console)</p>
        <dl className="mt-2 space-y-1 text-sm">
          <div className="flex gap-2">
            <dt className="shrink-0 text-[var(--color-ink-muted)]">Callback URL path:</dt>
            <dd className="font-mono text-xs text-slate-800">{data.whatsapp.webhookCallbackPath}</dd>
          </div>
        </dl>
        <div className="mt-3 max-w-xs">
          <Field
            label="Verify token"
            value={verifyToken}
            onChange={setVerifyToken}
            placeholder={data.whatsapp.webhookVerifyTokenMasked ?? 'Choose a shared secret'}
            secret
          />
        </div>
      </div>

      <div className="mt-4 flex gap-2">
        <button
          onClick={save}
          disabled={saving}
          className="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
        >
          {saving ? 'Saving…' : 'Save'}
        </button>
        <button
          onClick={verify}
          disabled={verifying}
          className="rounded-md border border-slate-300 px-3 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50"
        >
          {verifying ? 'Verifying…' : 'Verify connection'}
        </button>
      </div>
    </SectionCard>
  )
}

function BusinessProfileSection({
  data,
  onSaved,
  onError,
}: {
  data: IntegrationSettings
  onSaved: (msg: string) => void
  onError: (msg: string) => void
}) {
  const [name, setName] = useState(data.business.name)
  const [description, setDescription] = useState(data.business.description)
  const [guidelines, setGuidelines] = useState(data.business.agentGuidelines)
  const [autoReply, setAutoReply] = useState(data.business.autoReplyEnabled)
  const [hoursEnabled, setHoursEnabled] = useState(data.business.businessHoursEnabled)
  const [hoursStart, setHoursStart] = useState(data.business.businessHoursStart)
  const [hoursEnd, setHoursEnd] = useState(data.business.businessHoursEnd)
  const [timezone, setTimezone] = useState(data.business.businessTimezone)
  const [awayMessage, setAwayMessage] = useState(data.business.awayMessage)
  const [saving, setSaving] = useState(false)

  async function save() {
    setSaving(true)
    try {
      await api.updateIntegrationSettings({
        business: {
          name,
          description,
          agent_guidelines: guidelines,
          auto_reply_enabled: autoReply,
          business_hours_enabled: hoursEnabled,
          business_hours_start: hoursStart,
          business_hours_end: hoursEnd,
          business_timezone: timezone,
          away_message: awayMessage,
        },
      })
      onSaved('Business profile saved.')
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <SectionCard
      title="Business profile & agent guidelines"
      description="What the AI reply engine knows about your business, and how it should behave."
    >
      <div className="space-y-4">
        <Field label="Business name" value={name} onChange={setName} placeholder="Acme Inc." />
        <div>
          <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
            placeholder="What you do, who you serve."
            className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
          />
        </div>
        <div>
          <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Agent guidelines</label>
          <textarea
            value={guidelines}
            onChange={(e) => setGuidelines(e.target.value)}
            rows={4}
            placeholder="Tone, things to never promise, when to escalate to a human, etc."
            className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-[var(--color-ink)]">
          <input
            type="checkbox"
            checked={autoReply}
            onChange={(e) => setAutoReply(e.target.checked)}
            className="h-4 w-4 rounded border-slate-300"
          />
          Auto-reply enabled
        </label>

        <div className="rounded-md border border-[var(--color-border)] bg-[var(--color-page)] p-4">
          <label className="flex items-center gap-2 text-sm font-medium text-[var(--color-ink)]">
            <input
              type="checkbox"
              checked={hoursEnabled}
              onChange={(e) => setHoursEnabled(e.target.checked)}
              className="h-4 w-4 rounded border-slate-300"
            />
            Only auto-reply during business hours
          </label>
          {hoursEnabled && (
            <div className="mt-3 space-y-3">
              <div className="grid grid-cols-3 gap-3">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Start (24h)</label>
                  <input
                    type="number"
                    min={0}
                    max={23}
                    value={hoursStart}
                    onChange={(e) => setHoursStart(Number(e.target.value))}
                    className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink-muted)]">End (24h)</label>
                  <input
                    type="number"
                    min={0}
                    max={23}
                    value={hoursEnd}
                    onChange={(e) => setHoursEnd(Number(e.target.value))}
                    className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Timezone</label>
                  <input
                    value={timezone}
                    onChange={(e) => setTimezone(e.target.value)}
                    placeholder="Asia/Karachi"
                    className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                  />
                </div>
              </div>
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink-muted)]">
                  Away message (sent outside hours instead of an AI reply)
                </label>
                <textarea
                  value={awayMessage}
                  onChange={(e) => setAwayMessage(e.target.value)}
                  rows={2}
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                />
              </div>
            </div>
          )}
        </div>
      </div>
      <button
        onClick={save}
        disabled={saving}
        className="mt-4 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
      >
        {saving ? 'Saving…' : 'Save'}
      </button>
    </SectionCard>
  )
}

function TemplatesSection({
  onError,
  wabaConfigured,
}: {
  onError: (msg: string) => void
  wabaConfigured: boolean
}) {
  const [templates, setTemplates] = useState<Template[] | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [bodyText, setBodyText] = useState('')
  const [category, setCategory] = useState('UTILITY')
  const [saving, setSaving] = useState(false)

  function load() {
    api
      .listTemplates()
      .then(setTemplates)
      .catch((e) => onError((e as Error).message))
  }

  useEffect(() => {
    if (wabaConfigured) load()
  }, [wabaConfigured])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      await api.createTemplate({ name, category, language: 'en_US', body_text: bodyText })
      setName('')
      setBodyText('')
      setShowForm(false)
      load()
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function deleteTemplate(templateName: string) {
    try {
      await api.deleteTemplate(templateName)
      load()
    } catch (e) {
      onError((e as Error).message)
    }
  }

  return (
    <SectionCard
      title="Template messages"
      description="Pre-approved messages for re-engaging a lead outside the 24h session window."
    >
      {!wabaConfigured ? (
        <p className="text-sm text-[var(--color-ink-muted)]">
          Set a WhatsApp Business Account ID above to manage templates.
        </p>
      ) : (
        <>
          <div className="mb-3 flex justify-end">
            <button
              onClick={() => setShowForm((v) => !v)}
              className="rounded-md border border-slate-300 px-3 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              {showForm ? 'Cancel' : 'New template'}
            </button>
          </div>

          {showForm && (
            <form onSubmit={submit} className="mb-4 space-y-3 rounded-md border border-[var(--color-border)] p-4">
              <Field label="Name" value={name} onChange={setName} placeholder="order_update" />
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Category</label>
                <select
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                >
                  <option value="UTILITY">Utility</option>
                  <option value="MARKETING">Marketing</option>
                  <option value="AUTHENTICATION">Authentication</option>
                </select>
              </div>
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Body text</label>
                <textarea
                  value={bodyText}
                  onChange={(e) => setBodyText(e.target.value)}
                  rows={3}
                  required
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                />
              </div>
              <button
                type="submit"
                disabled={saving}
                className="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
              >
                {saving ? 'Submitting…' : 'Submit for approval'}
              </button>
            </form>
          )}

          {templates === null ? (
            <p className="text-sm text-[var(--color-ink-muted)]">Loading…</p>
          ) : templates.length === 0 ? (
            <p className="text-sm text-[var(--color-ink-muted)]">No templates yet.</p>
          ) : (
            <ul className="divide-y divide-slate-100 rounded-md border border-[var(--color-border)]">
              {templates.map((t) => (
                <li key={t.id} className="flex items-center justify-between gap-3 px-3 py-2 text-sm">
                  <div className="min-w-0">
                    <span className="font-medium text-slate-800">{t.name}</span>
                    <span className="ml-2 text-xs text-[var(--color-ink-muted)]">
                      {t.category} · {t.language} ·{' '}
                      <span
                        className={
                          t.status === 'APPROVED'
                            ? 'text-emerald-600'
                            : t.status === 'REJECTED'
                              ? 'text-red-600'
                              : 'text-amber-600'
                        }
                      >
                        {t.status}
                      </span>
                    </span>
                  </div>
                  <button
                    onClick={() => deleteTemplate(t.name)}
                    className="shrink-0 text-xs font-medium text-[var(--color-ink-faint)] hover:text-red-600"
                  >
                    Delete
                  </button>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </SectionCard>
  )
}

function StripeBillingSection({
  onNotice,
  onError,
}: {
  onNotice: (m: string) => void
  onError: (m: string) => void
}) {
  const [config, setConfig] = useState<StripeConfig | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<{ status: string; mode: string; message: string; tested_at: string } | null>(null)

  const [accountName, setAccountName] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [publishableKey, setPublishableKey] = useState('')
  const [webhookSecret, setWebhookSecret] = useState('')
  const [testMode, setTestMode] = useState(true)
  const [isEnabled, setIsEnabled] = useState(true)
  const [currency, setCurrency] = useState('usd')

  function loadConfig() {
    setLoading(true)
    api.getStripeConfig()
      .then((cfg) => {
        setConfig(cfg)
        setAccountName(cfg.account_name)
        setPublishableKey(cfg.publishable_key)
        setTestMode(cfg.test_mode)
        setIsEnabled(cfg.is_enabled)
        setCurrency(cfg.default_currency)
        setLoading(false)
      })
      .catch((err) => {
        onError(`Failed to load Stripe config: ${err.message}`)
        setLoading(false)
      })
  }

  useEffect(() => {
    loadConfig()
  }, [])

  async function handleSave() {
    setSaving(true)
    try {
      const updates: any = {
        account_name: accountName,
        publishable_key: publishableKey,
        test_mode: testMode,
        is_enabled: isEnabled,
        default_currency: currency,
      }
      if (apiKey.trim()) updates.api_key = apiKey.trim()
      if (webhookSecret.trim()) updates.webhook_secret = webhookSecret.trim()

      const res = await api.updateStripeConfig(updates)
      setConfig(res)
      setApiKey('')
      setWebhookSecret('')
      onNotice('Stripe billing integration configuration saved successfully.')
    } catch (e: any) {
      onError(`Failed to save Stripe config: ${e.message}`)
    } finally {
      setSaving(false)
    }
  }

  async function handleTest() {
    setTesting(true)
    setTestResult(null)
    try {
      const res = await api.testStripeConnection()
      setTestResult(res)
      if (res.status === 'connected' || res.status === 'connected_simulated') {
        onNotice(res.message)
      } else {
        onError(res.message)
      }
    } catch (e: any) {
      onError(`Connection test error: ${e.message}`)
    } finally {
      setTesting(false)
    }
  }

  if (loading) {
    return (
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center text-[var(--color-ink-muted)]">
        <div className="inline-block h-6 w-6 animate-spin rounded-full border-2 border-indigo-600 border-t-transparent" />
        <p className="mt-2 text-sm">Loading Stripe configuration...</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-[var(--color-border)] pb-5">
          <div className="flex items-center gap-3">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--color-accent-soft)] border border-indigo-100 text-2xl">
              💳
            </div>
            <div>
              <h2 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
                Stripe Billing & Sidebar Integration
                <span className={`rounded-full px-2 py-0.5 text-xs font-semibold ${
                  config?.last_test_status === 'connected' || config?.last_test_status === 'connected_simulated'
                    ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                    : 'bg-slate-100 text-[var(--color-ink-muted)]'
                }`}>
                  {config?.last_test_status === 'connected' ? 'Live Connected' : (config?.last_test_status === 'connected_simulated' ? 'Sandbox Active' : 'Untested')}
                </span>
              </h2>
              <p className="text-xs text-[var(--color-ink-muted)]">
                Powers real-time subscription badges, MRR & LTV metrics, customer invoices, and 1-click refunds in the conversation drawer.
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={handleTest}
              disabled={testing}
              className="inline-flex items-center gap-1.5 rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50 shadow-2xs cursor-pointer transition-colors"
            >
              <span>{testing ? 'Testing...' : '⚡ Test Connection'}</span>
            </button>
            <button
              onClick={handleSave}
              disabled={saving}
              className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-bold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 shadow-xs cursor-pointer transition-colors"
            >
              <span>{saving ? 'Saving...' : 'Save Configuration'}</span>
            </button>
          </div>
        </div>

        {testResult && (
          <div className={`rounded-lg p-3 text-xs flex items-center justify-between border ${
            testResult.status === 'connected' || testResult.status === 'connected_simulated'
              ? 'bg-emerald-50 text-emerald-800 border-emerald-200'
              : 'bg-rose-50 text-rose-800 border-rose-200'
          }`}>
            <div className="flex items-center gap-2">
              <span>{testResult.status === 'connected' || testResult.status === 'connected_simulated' ? '✅' : '❌'}</span>
              <span className="font-medium">{testResult.message}</span>
            </div>
            <span className="text-[10px] opacity-75 font-mono">Mode: {testResult.mode}</span>
          </div>
        )}

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          <div>
            <label className="block text-xs font-bold text-[var(--color-ink)] mb-1">Stripe Account Name</label>
            <input
              type="text"
              value={accountName}
              onChange={(e) => setAccountName(e.target.value)}
              placeholder="e.g. Primary Production Billing"
              className="w-full rounded-lg border border-slate-300 p-2.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-hidden"
            />
          </div>

          <div>
            <label className="block text-xs font-bold text-[var(--color-ink)] mb-1">Default Workspace Currency</label>
            <select
              value={currency}
              onChange={(e) => setCurrency(e.target.value)}
              className="w-full rounded-lg border border-slate-300 p-2.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-hidden"
            >
              <option value="usd">USD ($)</option>
              <option value="eur">EUR (€)</option>
              <option value="gbp">GBP (£)</option>
              <option value="cad">CAD ($)</option>
              <option value="aud">AUD ($)</option>
            </select>
          </div>

          <div>
            <div className="flex items-center justify-between mb-1">
              <label className="text-xs font-bold text-[var(--color-ink)]">Stripe Secret Key (sk_...)</label>
              {config?.has_api_key && (
                <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">Current: {config.api_key_masked}</span>
              )}
            </div>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={config?.has_api_key ? '•••••••••••••••• (Leave blank to keep)' : 'sk_test_... or sk_live_...'}
              className="w-full rounded-lg border border-slate-300 p-2.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-hidden font-mono"
            />
            <p className="text-[11px] text-[var(--color-ink-faint)] mt-1">
              Leave blank to use the built-in Sandbox Simulator with high-fidelity customer mock subscriptions.
            </p>
          </div>

          <div>
            <label className="block text-xs font-bold text-[var(--color-ink)] mb-1">Stripe Publishable Key (pk_...)</label>
            <input
              type="text"
              value={publishableKey}
              onChange={(e) => setPublishableKey(e.target.value)}
              placeholder="pk_test_... or pk_live_..."
              className="w-full rounded-lg border border-slate-300 p-2.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-hidden font-mono"
            />
          </div>

          <div>
            <div className="flex items-center justify-between mb-1">
              <label className="text-xs font-bold text-[var(--color-ink)]">Stripe Webhook Signing Secret (whsec_...)</label>
              {config?.has_webhook_secret && (
                <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">Current: {config.webhook_secret_masked}</span>
              )}
            </div>
            <input
              type="password"
              value={webhookSecret}
              onChange={(e) => setWebhookSecret(e.target.value)}
              placeholder={config?.has_webhook_secret ? '•••••••••••••••• (Leave blank to keep)' : 'whsec_...'}
              className="w-full rounded-lg border border-slate-300 p-2.5 text-xs text-[var(--color-ink)] focus:border-indigo-500 focus:outline-hidden font-mono"
            />
          </div>

          <div className="flex flex-col justify-center space-y-3 pt-2">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={testMode}
                onChange={(e) => setTestMode(e.target.checked)}
                className="h-4 w-4 rounded border-slate-300 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
              />
              <span className="text-xs font-semibold text-[var(--color-ink)]">
                Enable Sandbox / Test Mode Simulator
              </span>
            </label>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={isEnabled}
                onChange={(e) => setIsEnabled(e.target.checked)}
                className="h-4 w-4 rounded border-slate-300 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
              />
              <span className="text-xs font-semibold text-[var(--color-ink)]">
                Stripe Integration Enabled in Conversation Sidebar
              </span>
            </label>
          </div>
        </div>
      </div>
    </div>
  )
}

function IssueTrackersSection({
  onNotice,
  onError,
}: {
  onNotice: (m: string) => void
  onError: (m: string) => void
}) {
  const [configs, setConfigs] = useState<IssueTrackerConfig[]>([])
  const [loading, setLoading] = useState(true)
  const [testingProvider, setTestingProvider] = useState<string | null>(null)
  const [savingProvider, setSavingProvider] = useState<string | null>(null)

  // Local form inputs
  const [linearApiKey, setLinearApiKey] = useState('')
  const [linearTeamKey, setLinearTeamKey] = useState('ENG')

  const [ghToken, setGhToken] = useState('')
  const [ghOwner, setGhOwner] = useState('newax-org')
  const [ghRepo, setGhRepo] = useState('product-engine')

  const [jiraDomain, setJiraDomain] = useState('newax.atlassian.net')
  const [jiraEmail, setJiraEmail] = useState('')
  const [jiraToken, setJiraToken] = useState('')
  const [jiraProject, setJiraProject] = useState('SUP')

  const [slackToken, setSlackToken] = useState('')
  const [slackChannel, setSlackChannel] = useState('#customer-escalations')

  function load() {
    setLoading(true)
    api.listTrackerConfigs()
      .then((items) => {
        setConfigs(items)
        const lin = items.find((c) => c.provider === 'linear')
        if (lin) setLinearTeamKey(lin.config?.team_key || 'ENG')

        const gh = items.find((c) => c.provider === 'github')
        if (gh) {
          setGhOwner(gh.config?.owner || 'newax-org')
          setGhRepo(gh.config?.repo || 'product-engine')
        }

        const jira = items.find((c) => c.provider === 'jira')
        if (jira) {
          setJiraDomain(jira.credentials?.domain || 'newax.atlassian.net')
          setJiraEmail(jira.credentials?.email || '')
          setJiraProject(jira.config?.project_key || 'SUP')
        }

        const slack = items.find((c) => c.provider === 'slack')
        if (slack) setSlackChannel(slack.config?.channel_name || '#customer-escalations')

        setLoading(false)
      })
      .catch((e) => {
        onError(`Failed to load tracker configs: ${e.message}`)
        setLoading(false)
      })
  }

  useEffect(() => {
    load()
  }, [])

  async function handleTest(provider: string) {
    setTestingProvider(provider)
    try {
      const res = await api.testTrackerConnection(provider)
      if (res.status === 'connected' || res.status === 'connected_simulated') {
        onNotice(res.message)
      } else {
        onError(res.message)
      }
      load()
    } catch (e: any) {
      onError(`Test error: ${e.message}`)
    } finally {
      setTestingProvider(null)
    }
  }

  async function handleSaveLinear() {
    setSavingProvider('linear')
    try {
      const updates: any = { config: { team_key: linearTeamKey } }
      if (linearApiKey.trim()) updates.credentials = { api_key: linearApiKey.trim() }
      await api.updateTrackerConfig('linear', updates)
      setLinearApiKey('')
      onNotice('Linear configuration saved successfully.')
      load()
    } catch (e: any) {
      onError(`Failed to save Linear: ${e.message}`)
    } finally {
      setSavingProvider(null)
    }
  }

  async function handleSaveGitHub() {
    setSavingProvider('github')
    try {
      const updates: any = { config: { owner: ghOwner, repo: ghRepo } }
      if (ghToken.trim()) updates.credentials = { access_token: ghToken.trim() }
      await api.updateTrackerConfig('github', updates)
      setGhToken('')
      onNotice('GitHub Issues configuration saved successfully.')
      load()
    } catch (e: any) {
      onError(`Failed to save GitHub: ${e.message}`)
    } finally {
      setSavingProvider(null)
    }
  }

  async function handleSaveJira() {
    setSavingProvider('jira')
    try {
      const creds: any = { domain: jiraDomain, email: jiraEmail }
      if (jiraToken.trim()) creds.api_token = jiraToken.trim()
      await api.updateTrackerConfig('jira', { credentials: creds, config: { project_key: jiraProject } })
      setJiraToken('')
      onNotice('Jira Cloud configuration saved successfully.')
      load()
    } catch (e: any) {
      onError(`Failed to save Jira: ${e.message}`)
    } finally {
      setSavingProvider(null)
    }
  }

  async function handleSaveSlack() {
    setSavingProvider('slack')
    try {
      const updates: any = { config: { channel_name: slackChannel } }
      if (slackToken.trim()) updates.credentials = { bot_token: slackToken.trim() }
      await api.updateTrackerConfig('slack', updates)
      setSlackToken('')
      onNotice('Slack sync configuration saved successfully.')
      load()
    } catch (e: any) {
      onError(`Failed to save Slack: ${e.message}`)
    } finally {
      setSavingProvider(null)
    }
  }

  if (loading) {
    return (
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center text-[var(--color-ink-muted)]">
        <div className="inline-block h-6 w-6 animate-spin rounded-full border-2 border-indigo-600 border-t-transparent" />
        <p className="mt-2 text-sm">Loading issue tracker &amp; Slack configurations...</p>
      </div>
    )
  }

  const linCfg = configs.find((c) => c.provider === 'linear')
  const ghCfg = configs.find((c) => c.provider === 'github')
  const jiraCfg = configs.find((c) => c.provider === 'jira')
  const slackCfg = configs.find((c) => c.provider === 'slack')

  return (
    <div className="space-y-6">
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-2">
        <div className="flex items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-purple-50 border border-purple-100 text-2xl">
            🎯
          </div>
          <div>
            <h2 className="text-base font-bold text-[var(--color-ink)]">
              1-Click Issue Trackers &amp; Slack Workspace Sync
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Connect Linear, GitHub Issues, Jira, and Slack so support agents can escalate bugs and sync conversation threads directly with engineering.
            </p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* 1. Linear */}
        <div className="rounded-xl border border-purple-100 bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
            <div className="flex items-center gap-2">
              <span className="text-xl">🟣</span>
              <div>
                <h3 className="font-bold text-sm text-[var(--color-ink)]">Linear</h3>
                <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">
                  Status: {linCfg?.last_test_status || 'untested'}
                </span>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => handleTest('linear')}
                disabled={testingProvider === 'linear'}
                className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-[11px] font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] cursor-pointer"
              >
                {testingProvider === 'linear' ? 'Testing...' : '⚡ Test'}
              </button>
              <button
                onClick={handleSaveLinear}
                disabled={savingProvider === 'linear'}
                className="rounded bg-purple-600 px-3 py-1 text-[11px] font-bold text-white hover:bg-purple-700 cursor-pointer"
              >
                {savingProvider === 'linear' ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>

          <div className="space-y-3 text-xs">
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">
                API Key (lin_api_...)
              </label>
              <input
                type="password"
                placeholder={linCfg?.credentials?.has_api_key ? '•••••••••••••••• (Leave blank to keep)' : 'lin_api_...'}
                value={linearApiKey}
                onChange={(e) => setLinearApiKey(e.target.value)}
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
              />
            </div>
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">
                Default Team Key
              </label>
              <input
                type="text"
                value={linearTeamKey}
                onChange={(e) => setLinearTeamKey(e.target.value)}
                placeholder="ENG"
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
              />
            </div>
          </div>
        </div>

        {/* 2. GitHub Issues */}
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
            <div className="flex items-center gap-2">
              <span className="text-xl">🐙</span>
              <div>
                <h3 className="font-bold text-sm text-[var(--color-ink)]">GitHub Issues</h3>
                <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">
                  Status: {ghCfg?.last_test_status || 'untested'}
                </span>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => handleTest('github')}
                disabled={testingProvider === 'github'}
                className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-[11px] font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] cursor-pointer"
              >
                {testingProvider === 'github' ? 'Testing...' : '⚡ Test'}
              </button>
              <button
                onClick={handleSaveGitHub}
                disabled={savingProvider === 'github'}
                className="rounded bg-[var(--color-accent)] px-3 py-1 text-[11px] font-bold text-white hover:bg-[var(--color-accent-hover)] cursor-pointer"
              >
                {savingProvider === 'github' ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>

          <div className="space-y-3 text-xs">
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">
                Personal Access Token (ghp_...)
              </label>
              <input
                type="password"
                placeholder={ghCfg?.credentials?.has_access_token ? '•••••••••••••••• (Leave blank to keep)' : 'ghp_...'}
                value={ghToken}
                onChange={(e) => setGhToken(e.target.value)}
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
              />
            </div>
            <div className="grid grid-cols-2 gap-2">
              <div>
                <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">Org / Owner</label>
                <input
                  type="text"
                  value={ghOwner}
                  onChange={(e) => setGhOwner(e.target.value)}
                  placeholder="newax-org"
                  className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
                />
              </div>
              <div>
                <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">Repository</label>
                <input
                  type="text"
                  value={ghRepo}
                  onChange={(e) => setGhRepo(e.target.value)}
                  placeholder="product-engine"
                  className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
                />
              </div>
            </div>
          </div>
        </div>

        {/* 3. Jira Cloud */}
        <div className="rounded-xl border border-blue-100 bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
            <div className="flex items-center gap-2">
              <span className="text-xl">🔷</span>
              <div>
                <h3 className="font-bold text-sm text-[var(--color-ink)]">Jira Cloud</h3>
                <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">
                  Status: {jiraCfg?.last_test_status || 'untested'}
                </span>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => handleTest('jira')}
                disabled={testingProvider === 'jira'}
                className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-[11px] font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] cursor-pointer"
              >
                {testingProvider === 'jira' ? 'Testing...' : '⚡ Test'}
              </button>
              <button
                onClick={handleSaveJira}
                disabled={savingProvider === 'jira'}
                className="rounded bg-blue-600 px-3 py-1 text-[11px] font-bold text-white hover:bg-blue-700 cursor-pointer"
              >
                {savingProvider === 'jira' ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>

          <div className="space-y-3 text-xs">
            <div className="grid grid-cols-2 gap-2">
              <div>
                <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">Jira Domain</label>
                <input
                  type="text"
                  value={jiraDomain}
                  onChange={(e) => setJiraDomain(e.target.value)}
                  placeholder="newax.atlassian.net"
                  className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
                />
              </div>
              <div>
                <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">Project Key</label>
                <input
                  type="text"
                  value={jiraProject}
                  onChange={(e) => setJiraProject(e.target.value)}
                  placeholder="SUP"
                  className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
                />
              </div>
            </div>
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">User Email</label>
              <input
                type="text"
                value={jiraEmail}
                onChange={(e) => setJiraEmail(e.target.value)}
                placeholder="agent@newax.com"
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs"
              />
            </div>
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">API Token</label>
              <input
                type="password"
                placeholder={jiraCfg?.credentials?.has_api_token ? '•••••••••••••••• (Leave blank to keep)' : 'API token'}
                value={jiraToken}
                onChange={(e) => setJiraToken(e.target.value)}
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
              />
            </div>
          </div>
        </div>

        {/* 4. Slack Workspace Sync */}
        <div className="rounded-xl border border-emerald-100 bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
          <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
            <div className="flex items-center gap-2">
              <span className="text-xl">💬</span>
              <div>
                <h3 className="font-bold text-sm text-[var(--color-ink)]">Slack Workspace Sync</h3>
                <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">
                  Status: {slackCfg?.last_test_status || 'untested'}
                </span>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => handleTest('slack')}
                disabled={testingProvider === 'slack'}
                className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-[11px] font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] cursor-pointer"
              >
                {testingProvider === 'slack' ? 'Testing...' : '⚡ Test'}
              </button>
              <button
                onClick={handleSaveSlack}
                disabled={savingProvider === 'slack'}
                className="rounded bg-emerald-600 px-3 py-1 text-[11px] font-bold text-white hover:bg-emerald-700 cursor-pointer"
              >
                {savingProvider === 'slack' ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>

          <div className="space-y-3 text-xs">
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">
                Bot Token (xoxb-...)
              </label>
              <input
                type="password"
                placeholder={slackCfg?.credentials?.has_bot_token ? '•••••••••••••••• (Leave blank to keep)' : 'xoxb-...'}
                value={slackToken}
                onChange={(e) => setSlackToken(e.target.value)}
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
              />
            </div>
            <div>
              <label className="block text-[11px] font-bold text-[var(--color-ink)] mb-1">
                Default Notification Channel
              </label>
              <input
                type="text"
                value={slackChannel}
                onChange={(e) => setSlackChannel(e.target.value)}
                placeholder="#customer-escalations"
                className="w-full rounded-lg border border-[var(--color-border)] p-2 text-xs font-mono"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
