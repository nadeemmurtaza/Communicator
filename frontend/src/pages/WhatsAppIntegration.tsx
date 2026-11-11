import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, type WhatsAppChannelData, type WhatsAppTemplate } from '../lib/api'
import { WhatsAppIcon } from '../components/icons'
import { WhatsAppTemplateEditor } from '../components/whatsapp/WhatsAppTemplateEditor'
import type { CreateTemplateData } from '../components/whatsapp/WhatsAppTemplateEditor'
import { WhatsAppQualityDashboard } from '../components/whatsapp/WhatsAppQualityDashboard'
import { WhatsAppQRCode } from '../components/whatsapp/WhatsAppQRCode'
import { WhatsAppTroubleshooting } from '../components/whatsapp/WhatsAppTroubleshooting'
import { WhatsAppMarketingConstraints } from '../components/whatsapp/WhatsAppMarketingConstraints'
import {
  Key,
  Zap,
  ClipboardList,
  Bot,
  Activity,
  Building2,
  CheckCircle2,
  ShieldCheck,
  Smartphone,
  BarChart3,
  Send,
  Lock,
  Save,
  ExternalLink,
  Check,
  Copy,
  RefreshCw,
  X,
  Plus,
  ChevronDown,
  ChevronUp,
  Shield,
  DollarSign,
  MessageCircle,
  ArrowRightLeft,
  Clock,
  TrendingUp,
  TrendingDown,
  Minus,
  ShieldAlert,
} from '../components/icons'

function getBrandId(): string | null {
  try {
    const brand = localStorage.getItem('communicator.brand')
    if (brand) {
      const { id } = JSON.parse(brand)
      return id || null
    }
  } catch { /* ignore */ }
  return null
}

export function WhatsAppIntegration() {
  const brandId = getBrandId()
  const [data, setData] = useState<WhatsAppChannelData | null>(null)
  const [templates, setTemplates] = useState<WhatsAppTemplate[]>([])
  const [events, setEvents] = useState<any[]>([])
  const [activeTab, setActiveTab] = useState<'credentials' | 'webhooks' | 'templates' | 'automations' | 'diagnostics' | 'quality' | 'costs' | 'qr' | 'marketing' | 'troubleshooting'>('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [syncingTemplates, setSyncingTemplates] = useState(false)
  const [testingSend, setTestingSend] = useState(false)
  const [registeringPin, setRegisteringPin] = useState(false)
  const [registerPin, setRegisterPin] = useState('123456')
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Brand-aware state
  const [whatsappHealth, setWhatsAppHealth] = useState<any>(null)
  const [whatsappRules, setWhatsAppRules] = useState<any[]>([])
  const [healthLoading, setHealthLoading] = useState(false)
  const [rulesLoading, setRulesLoading] = useState(false)

  // Test send state
  const [testPhone, setTestPhone] = useState('')
  const [testBody, setTestBody] = useState('Hi there! This is a test message from your Chatwoot WhatsApp Business integration.')
  const [testResult, setTestResult] = useState<any | null>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)

  // Template editor state
  const [showTemplateEditor, setShowTemplateEditor] = useState(false)
  const [duplicateTemplate, setDuplicateTemplate] = useState<WhatsAppTemplate | null>(null)
  const [expandedTemplate, setExpandedTemplate] = useState<string | null>(null)

  // Gap 11: Meta OAuth Flow state
  const [oauthConnecting, setOauthConnecting] = useState(false)

  // Gap 12: Auto-Sync Display Name state
  const [autoSyncEnabled, setAutoSyncEnabled] = useState(false)
  const [autoSyncInterval, setAutoSyncInterval] = useState(3600) // seconds

  // Gap 16: Messenger → WhatsApp Switch state
  const [messengerSwitchEnabled, setMessengerSwitchEnabled] = useState(false)

  // Gap 14: Cost Tracking state
  const [costData, setCostData] = useState<any>(null)
  const [costLoading, setCostLoading] = useState(false)

  // Handle creating a new template
  async function handleCreateTemplate(data: CreateTemplateData) {
    if (!brandId) return
    try {
      // Build components JSON for Meta API format
      const components: any[] = []

      // Header component
      if (data.headerText && data.headerType === 'text') {
        components.push({ type: 'HEADER', format: 'TEXT', text: data.headerText })
      }

      // Body component with variables
      components.push({ type: 'BODY', text: data.bodyText })

      // Button components
      if (data.buttons.length > 0) {
        components.push({
          type: 'BUTTONS',
          buttons: data.buttons.map(b => ({
            type: b.type === 'quick_reply' ? 'QUICK_REPLY' : b.type === 'url' ? 'URL' : 'PHONE_NUMBER',
            text: b.text,
            ...(b.type === 'url' && { url: b.url }),
            ...(b.type === 'phone_number' && { phone_number: b.phoneNumber }),
          })),
        })
      }

      // Footer component
      if (data.footerText) {
        components.push({ type: 'FOOTER', text: data.footerText })
      }

      // Call the Meta API via backend to create template
      const response = await fetch(`/api/brands/${brandId}/whatsapp/templates`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: data.name,
          category: data.category,
          language: data.language,
          components,
        }),
      })

      if (!response.ok) {
        const err = await response.json()
        throw new Error(err.error || 'Failed to create template')
      }

      setSuccess(`Template "${data.name}" submitted for Meta approval`)
      setShowTemplateEditor(false)
      setDuplicateTemplate(null)
      // Reload templates
      setTemplates(await api.listWhatsAppTemplates(brandId))
      setTimeout(() => setSuccess(null), 3000)
    } catch (e: any) {
      setError(e.message || 'Failed to create template')
    }
  }

  useEffect(() => {
    loadAll()
    if (brandId) {
      loadWhatsAppHealth()
      loadWhatsAppRules()
    }
  }, [])

  async function loadAll() {
    try {
      setLoading(true)
      const [channelData, tplList, evList] = await Promise.all([
        api.getWhatsAppChannel(),
        brandId ? api.listWhatsAppTemplates(brandId) : Promise.resolve([]),
        api.listWhatsAppEvents(),
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

    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  // Brand-aware loading functions
  async function loadWhatsAppHealth() {
    if (!brandId) return
    try {
      setHealthLoading(true)
      const health = await api.getWhatsAppHealth(brandId)
      setWhatsAppHealth(health)
    } catch { /* ignore */ } finally {
      setHealthLoading(false)
    }
  }

  async function loadWhatsAppRules() {
    if (!brandId) return
    try {
      setRulesLoading(true)
      const rules = await api.listWhatsAppRules(brandId)
      setWhatsAppRules(rules)
    } catch { /* ignore */ } finally {
      setRulesLoading(false)
    }
  }

  // Gap 11: Meta OAuth Flow handler
  async function handleMetaOAuth() {
    if (!brandId) return
    try {
      setOauthConnecting(true)
      setError(null)
      // Redirect to Meta OAuth flow
      const metaAppId = data?.inbox?.credentials?.metaAppId || ''
      const redirectUri = `${window.location.origin}/auth/meta/callback`
      
      // Open Meta OAuth URL — the backend will handle the callback
      const oauthUrl = `/api/brands/${brandId}/whatsapp/oauth-url?redirect_uri=${encodeURIComponent(redirectUri)}`
      window.location.href = oauthUrl
    } catch (e) {
      setError((e as Error).message || 'Failed to start OAuth flow')
    } finally {
      setOauthConnecting(false)
    }
  }

  // Gap 12: Auto-Sync Display Name handler
  async function handleToggleAutoSync() {
    if (!brandId) return
    try {
      const enabled = !autoSyncEnabled
      setAutoSyncEnabled(enabled)
      const res = await fetch(`/api/brands/${brandId}/whatsapp/auto-sync`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled, interval: autoSyncInterval }),
      })
      if (!res.ok) {
        const err = await res.json()
        setError(err.error || 'Failed to update auto-sync')
        setAutoSyncEnabled(!enabled)
      } else {
        setSuccess(`Auto-sync ${enabled ? 'enabled' : 'disabled'}`)
        setTimeout(() => setSuccess(null), 3000)
      }
    } catch (e) {
      setError((e as Error).message)
      setAutoSyncEnabled(!autoSyncEnabled)
    }
  }

  // Gap 16: Messenger → WhatsApp Switch handler
  async function handleToggleMessengerSwitch() {
    if (!brandId) return
    try {
      const enabled = !messengerSwitchEnabled
      setMessengerSwitchEnabled(enabled)
      const res = await fetch(`/api/brands/${brandId}/whatsapp/messenger-switch`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled }),
      })
      if (!res.ok) {
        const err = await res.json()
        setError(err.error || 'Failed to update messenger switch')
        setMessengerSwitchEnabled(!enabled)
      } else {
        setSuccess(`Messenger→WhatsApp switch ${enabled ? 'enabled' : 'disabled'}`)
        setTimeout(() => setSuccess(null), 3000)
      }
    } catch (e) {
      setError((e as Error).message)
      setMessengerSwitchEnabled(!messengerSwitchEnabled)
    }
  }

  // Gap 14: Load Cost Tracking data
  async function loadCostData() {
    if (!brandId) return
    try {
      setCostLoading(true)
      const res = await fetch(`/api/brands/${brandId}/whatsapp/costs`)
      if (res.ok) {
        const data = await res.json()
        setCostData(data)
      }
    } catch { /* ignore */ } finally {
      setCostLoading(false)
    }
  }

  async function handleSyncTemplates() {
    if (!brandId) return
    try {
      setSyncingTemplates(true)
      const result = await api.syncWhatsAppTemplates(brandId)
      setSuccess(`Synced ${result.templatesSynced} templates`)
      setTemplates(await api.listWhatsAppTemplates(brandId))
      setTimeout(() => setSuccess(null), 3000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncingTemplates(false)
    }
  }

  async function handleSyncProfile() {
    if (!brandId) return
    try {
      setSyncingTemplates(true)
      const result = await api.syncWhatsAppProfile(brandId)
      setSuccess(`Profile synced: ${result.verifiedName || 'N/A'}`)
      loadWhatsAppHealth()
      setTimeout(() => setSuccess(null), 3000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncingTemplates(false)
    }
  }

  async function handleSendTest() {
    if (!brandId || !testPhone) return
    try {
      setTestingSend(true)
      const result = await api.sendWhatsAppTest(brandId, { to: testPhone, body: testBody })
      setTestResult(result)
      setSuccess('Test message sent successfully')
      setTimeout(() => { setTestResult(null); setSuccess(null) }, 3000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setTestingSend(false)
    }
  }

  async function handleDeleteRule(ruleId: string) {
    if (!brandId) return
    try {
      await api.deleteWhatsAppRule(brandId, ruleId)
      loadWhatsAppRules()
      setSuccess('Rule deleted')
      setTimeout(() => setSuccess(null), 3000)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  // Form state
  const [name, setName] = useState('')
  const [phoneNumberId, setPhoneNumberId] = useState('')
  const [wabaId, setWabaId] = useState('')
  const [accessToken, setAccessToken] = useState('')
  const [appSecret, setAppSecret] = useState('')
  const [verifyToken, setVerifyToken] = useState('')
  const [catalogId, setCatalogId] = useState('')

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

  async function handleRegisterPhone() {
    try {
      setRegisteringPin(true)
      setError(null)
      const res = await api.registerWhatsAppPhone(registerPin)
      if (res.status === 'registered' || res.status === 'ok') {
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
      if (res.status === 'ok' || res.status === 'connected') {
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

  function copyToClipboard(text: string, key: string) {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  if (loading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <div className="text-center">
          <Smartphone className="h-10 w-10 text-emerald-500 animate-pulse mx-auto" />
          <p className="mt-2 text-sm text-[var(--color-ink-faint)]">Loading WhatsApp Channel configuration...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Top Breadcrumb & Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 text-xs text-[var(--color-ink-faint)] mb-1">
            <Link to="/connections" className="hover:text-[var(--color-ink)]">
              Channels Hub
            </Link>
            <span>/</span>
            <span className="font-semibold text-[var(--color-ink)]">WhatsApp Business API</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-lg)] bg-emerald-500 text-white shadow-none">
              <WhatsAppIcon className="h-6 w-6" />
            </div>
            <div>
              <h1 className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
                WhatsApp Business Integration
                <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800">
                  <span className="flex items-center gap-1">
                    <CheckCircle2 className="h-3.5 w-3.5 text-emerald-700" />
                    Verified & Healthy
                  </span>
                </span>
              </h1>
              <p className="text-xs text-[var(--color-ink-faint)]">
                Official Meta Cloud API · Tier 100K messages/day · 24-hour customer service window
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleVerify}
            disabled={verifying}
            className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3.5 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] disabled:opacity-50 shadow-none flex items-center gap-1.5"
          >
            <Zap className={`h-3.5 w-3.5 text-amber-500 ${verifying ? 'animate-spin' : ''}`} />
            {verifying ? 'Checking Meta...' : 'Verify Connection'}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 shadow-none flex items-center gap-1.5"
          >
            <Save className="h-3.5 w-3.5" />
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
        </div>
      </div>

      {/* Health Metric Banner */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4 shadow-none">
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">Display Phone</span>
          <p className="mt-1 font-bold text-[var(--color-ink)] text-sm font-mono truncate">{data?.health.displayPhoneNumber || 'Configured in Meta'}</p>
          <span className="mt-1 inline-flex items-center gap-1 text-[11px] text-emerald-600 font-medium">
            <CheckCircle2 className="h-3 w-3" />
            Verified Number
          </span>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4 shadow-none">
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">Verified Name</span>
          <p className="mt-1 font-bold text-[var(--color-ink)] text-sm truncate">{data?.health.displayName || 'Newax Technologies'}</p>
          <span className="mt-1 inline-flex items-center gap-1 text-[11px] text-blue-600 font-medium">
            <ShieldCheck className="h-3 w-3" />
            {data?.health.verificationStatus || 'Standard Business Account'}
          </span>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4 shadow-none">
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">Quality Rating</span>
          <p className="mt-1 font-bold text-emerald-700 text-sm flex items-center gap-1.5">
            <span className="h-2 w-2 rounded-full bg-emerald-500"></span>
            {data?.health.qualityRating || 'HIGH (Green)'}
          </p>
          <span className="mt-1 inline-block text-[11px] text-[var(--color-ink-faint)]">Zero spam flags</span>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4 shadow-none">
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">Messaging Limit</span>
          <p className="mt-1 font-bold text-[var(--color-ink)] text-sm">{data?.health.messagingLimitTier || '250 / day'}</p>
          <span className="mt-1 inline-block text-[11px] text-purple-600 font-medium">{data?.health.tier || 'Standard Account Limit'}</span>
        </div>
      </div>

      {/* Shared Business Profile Link Banner */}
      <div className="rounded-[var(--radius-lg)] border border-indigo-100 bg-indigo-50/40 p-3.5 flex items-center justify-between gap-3 text-xs">
        <div className="flex items-center gap-2.5">
          <Building2 className="h-4 w-4 text-indigo-600 shrink-0" />
          <span className="text-[var(--color-ink)]">
            <strong>Shared Organization Profile:</strong> Profile, working hours, and brand identity are centrally shared across all communication channels.
          </span>
        </div>
        <Link
          to="/business-profile"
          className="font-semibold text-indigo-600 hover:text-indigo-800 flex items-center gap-1 whitespace-nowrap"
        >
          Manage Business Profile
          <ExternalLink className="h-3 w-3" />
        </Link>
      </div>

      {/* Brand-Aware Health Status */}
      {brandId && whatsappHealth && (
        <div className={`rounded-[var(--radius-lg)] border p-4 ${whatsappHealth.isConnected ? 'border-emerald-200 bg-emerald-50/50' : 'border-red-200 bg-red-50/50'}`}>
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <WhatsAppIcon className={`h-5 w-5 ${whatsappHealth.isConnected ? 'text-emerald-600' : 'text-red-600'}`} />
              <h3 className="font-semibold text-sm">
                Brand: {whatsappHealth.brandName || 'Unknown'}
              </h3>
              <span className={`rounded-full px-2 py-0.5 text-xs font-semibold ${whatsappHealth.isConnected ? 'bg-emerald-100 text-emerald-800' : 'bg-red-100 text-red-800'}`}>
                {whatsappHealth.isConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <button onClick={handleSyncProfile} disabled={syncingTemplates} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-1 text-xs font-semibold hover:bg-[var(--color-accent-muted)] disabled:opacity-50 flex items-center gap-1">
                <RefreshCw className={`h-3 w-3 ${syncingTemplates ? 'animate-spin' : ''}`} />
                Sync Profile
              </button>
              {/* Gap 12: Auto-Sync Display Name */}
              <button onClick={handleToggleAutoSync} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-1 text-xs font-semibold hover:bg-[var(--color-accent-muted)] flex items-center gap-1">
                <Clock className={`h-3 w-3 ${autoSyncEnabled ? 'text-emerald-600' : 'text-[var(--color-ink-faint)]'}`} />
                {autoSyncEnabled ? 'Auto-Sync ON' : 'Auto-Sync OFF'}
              </button>
              <button onClick={loadWhatsAppHealth} disabled={healthLoading} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-1 text-xs font-semibold hover:bg-[var(--color-accent-muted)] disabled:opacity-50 flex items-center gap-1">
                <RefreshCw className={`h-3 w-3 ${healthLoading ? 'animate-spin' : ''}`} />
                Refresh
              </button>
            </div>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
            <div>
              <span className="text-[var(--color-ink-faint)]">Verified Name</span>
              <p className="font-semibold text-[var(--color-ink)]">{whatsappHealth.verifiedName || 'N/A'}</p>
            </div>
            <div>
              <span className="text-[var(--color-ink-faint)]">Quality Rating</span>
              <p className="font-semibold text-emerald-700">{whatsappHealth.qualityRating || 'N/A'}</p>
            </div>
            <div>
              <span className="text-[var(--color-ink-faint)]">Messaging Tier</span>
              <p className="font-semibold text-[var(--color-ink)]">{whatsappHealth.messagingLimitTier || 'N/A'}</p>
            </div>
            <div>
              <span className="text-[var(--color-ink-faint)]">Status</span>
              <p className={`font-semibold ${whatsappHealth.connectionError ? 'text-red-600' : 'text-emerald-600'}`}>
                {whatsappHealth.connectionError || 'All good'}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Automation Rules */}
      {brandId && whatsappRules.length > 0 && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4 shadow-none">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <Bot className="h-4 w-4 text-purple-600" />
              <h3 className="font-semibold text-sm">Automation Rules ({whatsappRules.length})</h3>
            </div>
            <button onClick={loadWhatsAppRules} disabled={rulesLoading} className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-1 text-xs font-semibold hover:bg-[var(--color-accent-muted)] disabled:opacity-50 flex items-center gap-1">
              <RefreshCw className={`h-3 w-3 ${rulesLoading ? 'animate-spin' : ''}`} />
              Refresh
            </button>
          </div>
          <div className="space-y-2">
            {whatsappRules.map((rule: any) => (
              <div key={rule.id} className="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-3 text-xs">
                <div>
                  <span className="font-semibold text-[var(--color-ink)]">{rule.name}</span>
                  <span className="ml-2 text-[var(--color-ink-faint)]">Trigger: {rule.triggerType} → {rule.triggerValue || 'any'}</span>
                  <span className="ml-2 text-purple-600">Action: {rule.actionType}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className={`rounded-full px-2 py-0.5 text-[10px] font-semibold ${rule.isActive ? 'bg-emerald-100 text-emerald-800' : 'bg-[var(--color-accent-muted)] text-[var(--color-ink-faint)]'}`}>
                    {rule.isActive ? 'Active' : 'Inactive'}
                  </span>
                  <button onClick={() => handleDeleteRule(rule.id)} className="text-red-500 hover:text-red-700">
                    <X className="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Notifications */}
      {error && (
        <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-red-500 hover:text-red-700"><X className="h-4 w-4" /></button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-800"><X className="h-4 w-4" /></button>
        </div>
      )}

      {/* Navigation Tabs */}
      <div className="border-b border-[var(--color-border)] flex gap-6 text-sm font-medium">
        <button
          onClick={() => setActiveTab('credentials')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'credentials' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <Key className="h-4 w-4" />
          Meta Credentials
        </button>
        <button
          onClick={() => setActiveTab('webhooks')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'webhooks' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <Zap className="h-4 w-4" />
          Webhook Configuration
        </button>
        <button
          onClick={() => setActiveTab('templates')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'templates' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <ClipboardList className="h-4 w-4" />
          Message Templates ({templates.length})
        </button>
        <button
          onClick={() => setActiveTab('automations')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'automations' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <Bot className="h-4 w-4" />
          Channel Automations
        </button>
        <button
          onClick={() => setActiveTab('diagnostics')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'diagnostics' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <Activity className="h-4 w-4" />
          Live Test & Diagnostics
        </button>
        <button
          onClick={() => setActiveTab('quality')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'quality' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <BarChart3 className="h-4 w-4" />
          Quality Dashboard
        </button>
        <button
          onClick={() => { setActiveTab('costs'); loadCostData() }}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'costs' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <DollarSign className="h-4 w-4" />
          Cost Tracking
        </button>
        <button
          onClick={() => setActiveTab('qr')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'qr' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <Smartphone className="h-4 w-4" />
          QR Code & Link
        </button>
        <button
          onClick={() => setActiveTab('marketing')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'marketing' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <ShieldAlert className="h-4 w-4" />
          Marketing
        </button>
        <button
          onClick={() => setActiveTab('troubleshooting')}
          className={`pb-3 border-b-2 transition-colors flex items-center gap-2 ${
            activeTab === 'troubleshooting' ? 'border-[var(--color-accent)] text-[var(--color-ink)] font-bold' : 'border-transparent text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]'
          }`}
        >
          <ShieldAlert className="h-4 w-4" />
          Troubleshooting
        </button>
      </div>

      {/* TAB 1: Meta Credentials */}
      {activeTab === 'credentials' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none space-y-6">
          <div className="border-b border-[var(--color-border)] pb-4">
            <h2 className="text-base font-bold text-[var(--color-ink)]">Meta WhatsApp Cloud API Credentials</h2>
            <p className="text-xs text-[var(--color-ink-faint)]">
              Obtained from Meta for Developers &gt; App Dashboard &gt; WhatsApp &gt; API Setup.
            </p>
          </div>

          {/* Gap 11: Meta OAuth Flow Button */}
          <div className="rounded-[var(--radius-lg)] border border-indigo-200 bg-indigo-50/50 p-5 space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-lg)] bg-indigo-500 text-white">
                  <Shield className="h-5 w-5" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">Connect via Meta Business Manager</h3>
                  <p className="text-xs text-[var(--color-ink-faint)]">
                    The recommended way — OAuth login with your Facebook Business Manager account. Credentials are filled automatically.
                  </p>
                </div>
              </div>
              <button
                onClick={handleMetaOAuth}
                disabled={oauthConnecting}
                className="rounded-[var(--radius-md)] bg-indigo-600 px-4 py-2 text-xs font-semibold text-white hover:bg-indigo-700 disabled:opacity-50 flex items-center gap-2 shrink-0"
              >
                {oauthConnecting ? (
                  <>
                    <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                    Connecting...
                  </>
                ) : (
                  <>
                    <ExternalLink className="h-3.5 w-3.5" />
                    Connect with Meta
                  </>
                )}
              </button>
            </div>
            <div className="flex items-center gap-4 text-xs text-[var(--color-ink-faint)]">
              <span className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3 text-emerald-500" /> Auto-fills credentials</span>
              <span className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3 text-emerald-500" /> Sets up webhooks</span>
              <span className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3 text-emerald-500" /> No manual tokens</span>
            </div>
          </div>

          <div className="flex items-center gap-3 text-xs text-[var(--color-ink-faint)]">
            <div className="flex-1 h-px bg-[var(--color-border)]" />
            <span>OR configure manually below</span>
            <div className="flex-1 h-px bg-[var(--color-border)]" />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">Channel / Inbox Name</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none focus:border-[var(--color-accent)] focus:outline-hidden"
                placeholder="WhatsApp Support"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">Phone Number ID</label>
              <input
                type="text"
                value={phoneNumberId}
                onChange={(e) => setPhoneNumberId(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none focus:border-[var(--color-accent)] focus:outline-hidden font-mono"
                placeholder="e.g. 109823749283742"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">WhatsApp Business Account ID (WABA ID)</label>
              <input
                type="text"
                value={wabaId}
                onChange={(e) => setWabaId(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none focus:border-[var(--color-accent)] focus:outline-hidden font-mono"
                placeholder="e.g. 982374982374928"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">App Secret (for HMAC-SHA256 signature verification)</label>
              <input
                type="password"
                value={appSecret}
                onChange={(e) => setAppSecret(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none focus:border-[var(--color-accent)] focus:outline-hidden font-mono"
                placeholder="••••••••••••••••••••••••••••••••"
              />
            </div>
            <div className="md:col-span-2">
              <label className="block text-xs font-semibold text-[var(--color-ink)]">
                System User Permanent Access Token
              </label>
              <input
                type="password"
                value={accessToken}
                onChange={(e) => setAccessToken(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none focus:border-[var(--color-accent)] focus:outline-hidden font-mono"
                placeholder="EAA... (Never expires system user token with whatsapp_business_messaging permissions)"
              />
              <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
                Generate in Meta Business Manager &gt; System Users &gt; Generate Token with <code>whatsapp_business_messaging</code> and <code>whatsapp_business_management</code>.
              </p>
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">Meta Commerce Catalog ID (Optional)</label>
              <input
                type="text"
                value={catalogId}
                onChange={(e) => setCatalogId(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none focus:border-[var(--color-accent)] focus:outline-hidden font-mono"
                placeholder="e.g. 782349872394872"
              />
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: Webhooks Configuration */}
      {activeTab === 'webhooks' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none space-y-6">
          <div className="border-b border-[var(--color-border)] pb-4">
            <h2 className="text-base font-bold text-[var(--color-ink)]">Meta Inbound Webhook Configuration</h2>
            <p className="text-xs text-[var(--color-ink-faint)]">
              Configure these endpoints in your Meta Developer Console under WhatsApp &gt; Configuration.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">Callback URL</label>
              <div className="mt-1 flex items-center gap-2">
                <input
                  type="text"
                  readOnly
                  value={data?.webhookConfig.callbackUrl || 'https://api.yourdomain.com/api/v1/webhooks/meta/whatsapp'}
                  className="flex-1 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] px-3 py-2 text-xs font-mono text-[var(--color-ink)]"
                />
                <button
                  onClick={() => copyToClipboard(data?.webhookConfig.callbackUrl || '', 'url')}
                  className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-2 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] flex items-center gap-1.5"
                >
                  {copiedKey === 'url' ? <Check className="h-3.5 w-3.5 text-emerald-600" /> : <Copy className="h-3.5 w-3.5" />}
                  {copiedKey === 'url' ? 'Copied' : 'Copy URL'}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink)]">Verify Token</label>
              <div className="mt-1 flex items-center gap-2">
                <input
                  type="text"
                  value={verifyToken}
                  onChange={(e) => setVerifyToken(e.target.value)}
                  className="flex-1 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-xs font-mono text-[var(--color-ink)]"
                />
                <button
                  onClick={() => copyToClipboard(verifyToken, 'token')}
                  className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-2 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] flex items-center gap-1.5"
                >
                  {copiedKey === 'token' ? <Check className="h-3.5 w-3.5 text-emerald-600" /> : <Copy className="h-3.5 w-3.5" />}
                  {copiedKey === 'token' ? 'Copied' : 'Copy Token'}
                </button>
              </div>
            </div>

            <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-muted)]/70 p-4 space-y-2">
              <h3 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">Subscribed Webhook Fields</h3>
              <div className="flex flex-wrap gap-2 text-xs">
                <span className="rounded-[var(--radius-sm)] bg-white border border-[var(--color-border)] px-2.5 py-1 font-mono text-emerald-800 font-semibold inline-flex items-center gap-1.5">
                  <CheckCircle2 className="h-3 w-3 text-emerald-600" />
                  messages (Inbound customer messages &amp; media)
                </span>
                <span className="rounded-[var(--radius-sm)] bg-white border border-[var(--color-border)] px-2.5 py-1 font-mono text-blue-800 font-semibold inline-flex items-center gap-1.5">
                  <CheckCircle2 className="h-3 w-3 text-blue-600" />
                  message_deliveries (Delivered receipts)
                </span>
                <span className="rounded-[var(--radius-sm)] bg-white border border-[var(--color-border)] px-2.5 py-1 font-mono text-purple-800 font-semibold inline-flex items-center gap-1.5">
                  <CheckCircle2 className="h-3 w-3 text-purple-600" />
                  message_reads (Blue checkmarks)
                </span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: Message Templates */}
      {activeTab === 'templates' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white shadow-none overflow-hidden">
          <div className="border-b border-[var(--color-border)] bg-[var(--color-accent-muted)]/60 px-6 py-4 flex items-center justify-between">
            <div>
              <h2 className="text-base font-bold text-[var(--color-ink)]">Meta Approved Message Templates</h2>
              <p className="text-xs text-[var(--color-ink-faint)]">
                Pre-approved templates required to initiate conversations or reply outside the 24-hour service window.
              </p>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => { setDuplicateTemplate(null); setShowTemplateEditor(true) }}
                className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3.5 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] shadow-none flex items-center gap-1.5 cursor-pointer"
              >
                <Plus className="h-3.5 w-3.5 text-[var(--color-accent)]" />
                New Template
              </button>
              <button
                onClick={handleSyncTemplates}
                disabled={syncingTemplates}
                className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3.5 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] shadow-none flex items-center gap-1.5 cursor-pointer"
              >
                <RefreshCw className={`h-3.5 w-3.5 text-emerald-600 ${syncingTemplates ? 'animate-spin' : ''}`} />
                {syncingTemplates ? 'Syncing...' : 'Sync from Meta'}
              </button>
            </div>
          </div>

          <div className="divide-y divide-[var(--color-border)]">
            {templates.length === 0 && (
              <div className="p-12 text-center">
                <p className="text-sm text-[var(--color-ink-faint)]">No templates synced yet. Click "Sync from Meta" to fetch your approved templates.</p>
              </div>
            )}
            {templates.map((tpl) => {
              const isExpanded = expandedTemplate === tpl.id
              const body = tpl.body || tpl.bodyText || ''
              const quality = tpl.qualityRating
              const qualityColor = quality === 'high' ? 'emerald' : quality === 'low' ? 'rose' : 'amber'

              return (
                <div key={tpl.id} className="p-5">
                  {/* Header row */}
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 space-y-1.5">
                      <div className="flex items-center flex-wrap gap-2">
                        <span className="font-bold text-sm text-[var(--color-ink)] font-mono">{tpl.name}</span>
                        <span className={`rounded-full px-2 py-0.5 text-[11px] font-semibold ${
                          tpl.status === 'APPROVED' ? 'bg-emerald-100 text-emerald-800' :
                          tpl.status === 'PENDING' ? 'bg-amber-100 text-amber-800' :
                          'bg-red-100 text-red-800'
                        }`}>
                          {tpl.status}
                        </span>
                        <span className="rounded-[var(--radius-sm)] bg-[var(--color-accent-muted)] px-2 py-0.5 text-[11px] font-semibold text-[var(--color-ink)] uppercase">
                          {tpl.category}
                        </span>
                        <span className="text-xs text-[var(--color-ink-faint)]">· {tpl.language}</span>
                        {/* Quality badge */}
                        {quality && (
                          <span className={`rounded-full px-2 py-0.5 text-[11px] font-semibold bg-${qualityColor}-100 text-${qualityColor}-800`}>
                            {quality === 'high' ? '●' : quality === 'low' ? '●' : '●'} {quality.charAt(0).toUpperCase() + quality.slice(1)} Quality
                          </span>
                        )}
                        {/* Usage count */}
                        {tpl.sentCount && tpl.sentCount > 0 && (
                          <span className="rounded-full bg-blue-100 text-blue-800 px-2 py-0.5 text-[11px] font-semibold">
                            {tpl.sentCount} sent
                          </span>
                        )}
                      </div>

                      {/* Body preview */}
                      <p className="text-xs text-[var(--color-ink-muted)] bg-[var(--color-accent-muted)] p-2.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] font-sans leading-relaxed line-clamp-2">
                        "{body}"
                      </p>
                    </div>

                    {/* Actions */}
                    <div className="flex items-center gap-1 shrink-0">
                      <button
                        onClick={() => setExpandedTemplate(isExpanded ? null : tpl.id)}
                        className="p-1.5 text-[var(--color-ink-faint)] hover:bg-[var(--color-accent-muted)] rounded transition-colors cursor-pointer"
                        title={isExpanded ? 'Collapse' : 'Expand details'}
                      >
                        {isExpanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                      </button>
                      <button
                        onClick={() => { setDuplicateTemplate(tpl); setShowTemplateEditor(true) }}
                        className="p-1.5 text-[var(--color-ink-faint)] hover:bg-[var(--color-accent-muted)] rounded transition-colors cursor-pointer"
                        title="Duplicate template"
                      >
                        <Copy className="h-4 w-4" />
                      </button>
                    </div>
                  </div>

                  {/* Expanded details */}
                  {isExpanded && (
                    <div className="mt-4 space-y-3 pl-4 border-l-2 border-[var(--color-border)]">
                      {/* Components */}
                      <div className="flex flex-wrap gap-2">
                        {tpl.headerText && (
                          <span className="rounded bg-blue-50 border border-blue-200 px-2.5 py-1 text-[11px] font-medium text-blue-800">
                            Header: "{tpl.headerText}"
                          </span>
                        )}
                        {tpl.footerText && (
                          <span className="rounded bg-purple-50 border border-purple-200 px-2.5 py-1 text-[11px] font-medium text-purple-800">
                            Footer: "{tpl.footerText}"
                          </span>
                        )}
                        {tpl.buttons && tpl.buttons.length > 0 && (
                          <span className="rounded bg-emerald-50 border border-emerald-200 px-2.5 py-1 text-[11px] font-medium text-emerald-800">
                            {tpl.buttons.length} button{tpl.buttons.length > 1 ? 's' : ''}
                          </span>
                        )}
                      </div>

                      {/* Variables */}
                      {tpl.variables && tpl.variables.length > 0 && (
                        <div>
                          <p className="text-[11px] font-semibold text-[var(--color-ink)] mb-1">Variables</p>
                          <div className="flex flex-wrap gap-1.5">
                            {tpl.variables.map((v: string, i: number) => (
                              <span key={i} className="rounded-[var(--radius-sm)] bg-[var(--color-accent)]/10 border border-[var(--color-accent)]/20 px-2 py-0.5 text-[11px] font-mono text-[var(--color-accent)]">
                                {v}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Buttons detail */}
                      {tpl.buttons && tpl.buttons.length > 0 && (
                        <div>
                          <p className="text-[11px] font-semibold text-[var(--color-ink)] mb-1">Buttons</p>
                          <div className="space-y-1">
                            {tpl.buttons.map((btn: any, i: number) => (
                              <div key={i} className="rounded-[var(--radius-sm)] bg-[var(--color-accent-muted)] px-3 py-1.5 text-xs text-[var(--color-ink)]">
                                {btn.text || btn.title}
                                {btn.type && <span className="text-[var(--color-ink-faint)] ml-1">({btn.type})</span>}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Quality score bar */}
                      {tpl.qualityScore != null && (
                        <div>
                          <p className="text-[11px] font-semibold text-[var(--color-ink)] mb-1">
                            Quality Score: {tpl.qualityScore}/100
                          </p>
                          <div className="h-2 rounded-full bg-[var(--color-page)] border border-[var(--color-border)] overflow-hidden">
                            <div
                              className={`h-full rounded-full transition-all ${
                                quality === 'high' ? 'bg-emerald-500' :
                                quality === 'low' ? 'bg-rose-500' : 'bg-amber-500'
                              }`}
                              style={{ width: `${Math.min(100, Math.max(0, tpl.qualityScore))}%` }}
                            />
                          </div>
                        </div>
                      )}

                      {/* Last sent */}
                      {tpl.lastSentAt && (
                        <p className="text-[11px] text-[var(--color-ink-faint)]">
                          Last sent: {new Date(tpl.lastSentAt).toLocaleString()}
                        </p>
                      )}

                      {/* Last synced */}
                      {tpl.lastSyncedAt && (
                        <p className="text-[11px] text-[var(--color-ink-faint)]">
                          Last synced: {new Date(tpl.lastSyncedAt).toLocaleString()}
                        </p>
                      )}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Template Editor Modal */}
      <WhatsAppTemplateEditor
        open={showTemplateEditor}
        onClose={() => { setShowTemplateEditor(false); setDuplicateTemplate(null) }}
        onCreate={handleCreateTemplate}
        duplicateSource={duplicateTemplate}
      />

      {/* TAB 4: Channel Automations */}
      {activeTab === 'automations' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none space-y-6">
          <div className="border-b border-[var(--color-border)] pb-4">
            <h2 className="text-base font-bold text-[var(--color-ink)]">WhatsApp Automated Workflows</h2>
            <p className="text-xs text-[var(--color-ink-faint)]">Configure greeting rules, after-hours responses, read receipts, and CSAT.</p>
          </div>

          <div className="space-y-6 divide-y divide-slate-100">
            {/* Welcome Greeting */}
            <div className="pt-2 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">Welcome Greeting</h3>
                  <p className="text-xs text-[var(--color-ink-faint)]">
                    Automatically sent on first-touch or after 24+ hours of inactivity with interactive buttons.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={greetingEnabled}
                  onChange={(e) => setGreetingEnabled(e.target.checked)}
                  className="h-5 w-5 rounded border-[var(--color-border)] text-[var(--color-ink)]"
                />
              </div>
              {greetingEnabled && (
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] p-2.5 text-xs shadow-none"
                />
              )}
            </div>

            {/* Read Receipts */}
            <div className="pt-4 flex items-center justify-between">
              <div>
                <h3 className="text-sm font-bold text-[var(--color-ink)]">Automatic Read Receipts (Blue Checks)</h3>
                <p className="text-xs text-[var(--color-ink-faint)]">
                  Immediately sends <code>mark_as_read</code> to Meta upon incoming message receipt so users see blue double checks.
                </p>
              </div>
              <input
                type="checkbox"
                checked={autoReadReceipts}
                onChange={(e) => setAutoReadReceipts(e.target.checked)}
                className="h-5 w-5 rounded border-[var(--color-border)] text-[var(--color-ink)]"
              />
            </div>

            {/* CSAT Survey */}
            <div className="pt-4 flex items-center justify-between">
              <div>
                <h3 className="text-sm font-bold text-[var(--color-ink)]">Post-Resolution CSAT Rating Survey</h3>
                <p className="text-xs text-[var(--color-ink-faint)]">
                  Sends interactive 1-to-5 star rating quick reply buttons when a conversation is marked resolved.
                </p>
              </div>
              <input
                type="checkbox"
                checked={csatEnabled}
                onChange={(e) => setCsatEnabled(e.target.checked)}
                className="h-5 w-5 rounded border-[var(--color-border)] text-[var(--color-ink)]"
              />
            </div>

            {/* Gap 16: Messenger → WhatsApp Switch */}
            <div className="pt-4 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
                    <ArrowRightLeft className="h-4 w-4 text-blue-500" />
                    Switch from Messenger to WhatsApp
                  </h3>
                  <p className="text-xs text-[var(--color-ink-faint)]">
                    Offer customers switching from Messenger to WhatsApp. When enabled, Messenger conversations will show a prompt to continue on WhatsApp.
                  </p>
                </div>
                <button
                  onClick={handleToggleMessengerSwitch}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${messengerSwitchEnabled ? 'bg-emerald-500' : 'bg-[var(--color-border)]'}`}
                >
                  <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${messengerSwitchEnabled ? 'translate-x-6' : 'translate-x-1'}`} />
                </button>
              </div>
              {messengerSwitchEnabled && (
                <div className="rounded-[var(--radius-md)] border border-blue-200 bg-blue-50/50 p-3 text-xs text-blue-800 space-y-2">
                  <p className="font-semibold">How it works:</p>
                  <ul className="list-disc list-inside space-y-1 text-[11px]">
                    <li>When a Messenger customer sends a message, they receive an offer to switch to WhatsApp</li>
                    <li>The conversation history is preserved and transferred to WhatsApp</li>
                    <li>Future messages continue on WhatsApp automatically</li>
                    <li>Both channels must be connected for this feature to work</li>
                  </ul>
                </div>
              )}
            </div>

            {/* Business Hours */}
            <div className="pt-4 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">Business Hours &amp; Out-of-Office Away Message</h3>
                  <p className="text-xs text-[var(--color-ink-faint)]">
                    Sends an interactive away message when contacts message outside working hours.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={workingHoursEnabled}
                  onChange={(e) => setWorkingHoursEnabled(e.target.checked)}
                  className="h-5 w-5 rounded border-[var(--color-border)] text-[var(--color-ink)]"
                />
              </div>

              {workingHoursEnabled && (
                <div className="space-y-3 bg-[var(--color-accent-muted)] p-4 rounded-[var(--radius-lg)] border border-[var(--color-border)]">
                  <div className="flex items-center gap-4 text-xs">
                    <div>
                      <label className="font-semibold text-[var(--color-ink)]">Open Time (Hour 0-23)</label>
                      <input
                        type="number"
                        min={0}
                        max={23}
                        value={workingHoursStart}
                        onChange={(e) => setWorkingHoursStart(Number(e.target.value))}
                        className="mt-1 w-24 rounded border border-[var(--color-border)] p-1.5 bg-white text-xs"
                      />
                    </div>
                    <div>
                      <label className="font-semibold text-[var(--color-ink)]">Close Time (Hour 0-23)</label>
                      <input
                        type="number"
                        min={0}
                        max={23}
                        value={workingHoursEnd}
                        onChange={(e) => setWorkingHoursEnd(Number(e.target.value))}
                        className="mt-1 w-24 rounded border border-[var(--color-border)] p-1.5 bg-white text-xs"
                      />
                    </div>
                    <div>
                      <label className="font-semibold text-[var(--color-ink)]">Timezone</label>
                      <input
                        type="text"
                        value={timezone}
                        onChange={(e) => setTimezone(e.target.value)}
                        className="mt-1 w-32 rounded border border-[var(--color-border)] p-1.5 bg-white text-xs"
                      />
                    </div>
                  </div>
                  <div>
                    <label className="block text-xs font-semibold text-[var(--color-ink)]">Away Message</label>
                    <textarea
                      rows={2}
                      value={outOfOfficeMessage}
                      onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                      className="mt-1 w-full rounded border border-[var(--color-border)] p-2 text-xs bg-white"
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* TAB 7: Quality Dashboard */}
      {activeTab === 'quality' && brandId && (
        <WhatsAppQualityDashboard brandId={brandId} />
      )}

      {/* Gap 14: Cost Tracking Dashboard */}
      {activeTab === 'costs' && (
        <div className="space-y-6">
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
                  <DollarSign className="h-5 w-5 text-emerald-600" />
                  WhatsApp Cost Tracking
                </h2>
                <p className="text-xs text-[var(--color-ink-faint)]">
                  Track Meta WhatsApp conversation costs by category and campaign.
                </p>
              </div>
              <button
                onClick={loadCostData}
                disabled={costLoading}
                className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-1.5 text-xs font-semibold hover:bg-[var(--color-accent-muted)] disabled:opacity-50 flex items-center gap-1"
              >
                <RefreshCw className={`h-3 w-3 ${costLoading ? 'animate-spin' : ''}`} />
                Refresh
              </button>
            </div>

            {/* Cost Summary Cards */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-4">
                <p className="text-xs font-semibold text-[var(--color-ink-faint)]">Total Cost (30d)</p>
                <p className="text-2xl font-bold text-[var(--color-ink)] mt-1">
                  ${costData?.totalCost30d?.toFixed(2) || '0.00'}
                </p>
                <p className="text-[11px] text-[var(--color-ink-faint)] mt-1">
                  {costData?.totalConversations30d || 0} conversations
                </p>
              </div>
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-4">
                <p className="text-xs font-semibold text-[var(--color-ink-faint)]">Today</p>
                <p className="text-2xl font-bold text-[var(--color-ink)] mt-1">
                  ${costData?.todayCost?.toFixed(2) || '0.00'}
                </p>
                <p className="text-[11px] text-[var(--color-ink-faint)] mt-1">
                  {costData?.todayConversations || 0} conversations
                </p>
              </div>
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-4">
                <p className="text-xs font-semibold text-[var(--color-ink-faint)]">Avg per Conversation</p>
                <p className="text-2xl font-bold text-[var(--color-ink)] mt-1">
                  ${costData?.avgPerConversation?.toFixed(4) || '0.0000'}
                </p>
                <p className="text-[11px] text-[var(--color-ink-faint)] mt-1">Varies by region</p>
              </div>
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-4">
                <p className="text-xs font-semibold text-[var(--color-ink-faint)]">Messaging Tier</p>
                <p className="text-lg font-bold text-[var(--color-ink)] mt-2">
                  {costData?.messagingTier || whatsappHealth?.messagingLimitTier || 'N/A'}
                </p>
                <p className="text-[11px] text-[var(--color-ink-faint)] mt-1">
                  {costData?.tierLimit || '100K'} conversations/day
                </p>
              </div>
            </div>

            {/* Cost by Category */}
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] overflow-hidden">
              <div className="border-b border-[var(--color-border)] bg-[var(--color-accent-muted)]/60 px-4 py-3">
                <p className="text-xs font-semibold text-[var(--color-ink-faint)]">Cost by Conversation Category</p>
              </div>
              <table className="w-full text-sm">
                <thead className="border-b border-[var(--color-border)] bg-[var(--color-page)] text-left text-xs font-medium text-[var(--color-ink-muted)]">
                  <tr>
                    <th className="px-4 py-2.5">Category</th>
                    <th className="px-4 py-2.5">Conversations</th>
                    <th className="px-4 py-2.5">Rate</th>
                    <th className="px-4 py-2.5">Total Cost</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-border)]">
                  {['marketing', 'utility', 'authentication', 'service'].map((cat) => {
                    const catData = costData?.byCategory?.[cat] || { conversations: 0, rate: 0, cost: 0 }
                    const color = cat === 'marketing' ? 'text-rose-600' : cat === 'utility' ? 'text-blue-600' : cat === 'authentication' ? 'text-purple-600' : 'text-emerald-600'
                    return (
                      <tr key={cat} className="hover:bg-[var(--color-page)]">
                        <td className="px-4 py-3 capitalize font-semibold text-[var(--color-ink)]">{cat}</td>
                        <td className="px-4 py-3 text-[var(--color-ink-muted)]">{catData.conversations || 0}</td>
                        <td className="px-4 py-3 text-[var(--color-ink-muted)]">${(catData.rate || 0).toFixed(4)}</td>
                        <td className={`px-4 py-3 font-semibold ${color}`}>${(catData.cost || 0).toFixed(2)}</td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>

            {/* Cost by Campaign */}
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] overflow-hidden">
              <div className="border-b border-[var(--color-border)] bg-[var(--color-accent-muted)]/60 px-4 py-3">
                <p className="text-xs font-semibold text-[var(--color-ink-faint)]">Cost by Campaign</p>
              </div>
              {costData?.byCampaign?.length > 0 ? (
                <table className="w-full text-sm">
                  <thead className="border-b border-[var(--color-border)] bg-[var(--color-page)] text-left text-xs font-medium text-[var(--color-ink-muted)]">
                    <tr>
                      <th className="px-4 py-2.5">Campaign</th>
                      <th className="px-4 py-2.5">Sent</th>
                      <th className="px-4 py-2.5">Category</th>
                      <th className="px-4 py-2.5">Cost</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--color-border)]">
                    {costData.byCampaign.map((c: any) => (
                      <tr key={c.id} className="hover:bg-[var(--color-page)]">
                        <td className="px-4 py-3 font-semibold text-[var(--color-ink)]">{c.name}</td>
                        <td className="px-4 py-3 text-[var(--color-ink-muted)]">{c.sent || 0}</td>
                        <td className="px-4 py-3 text-[var(--color-ink-muted)] capitalize">{c.category || 'utility'}</td>
                        <td className="px-4 py-3 font-semibold text-rose-600">${(c.cost || 0).toFixed(2)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              ) : (
                <div className="p-8 text-center text-xs text-[var(--color-ink-faint)]">
                  No campaign cost data yet. Costs will appear as campaigns are sent.
                </div>
              )}
            </div>

            {/* Pricing Info */}
            <div className="rounded-[var(--radius-lg)] border border-amber-200 bg-amber-50/50 p-4 text-xs text-amber-800 space-y-2">
              <p className="font-semibold flex items-center gap-2">
                <MessageCircle className="h-4 w-4" />
                How WhatsApp Pricing Works
              </p>
              <ul className="list-disc list-inside space-y-1 text-[11px]">
                <li>Meta charges per <strong>24-hour conversation window</strong>, not per message</li>
                <li>Marketing conversations are the most expensive (promotional content)</li>
                <li>Service conversations (user-initiated) are the cheapest</li>
                <li>Utility and Authentication have moderate rates</li>
                <li>Rates vary by country/region — check your Meta Business Manager for exact rates</li>
                <li>Free 1,000 service conversations per month included</li>
              </ul>
            </div>
          </div>
        </div>
      )}

      {/* TAB 6: Live Test & Diagnostics */}
      {activeTab === 'diagnostics' && (
        <div className="space-y-6">
          {/* Phone Registration Card */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
                  <span className="flex items-center gap-2"><Lock className="h-4 w-4 text-[var(--color-ink)]" /> Phone Number 2-Step Registration</span>
                  <span className="rounded bg-emerald-100 text-emerald-800 text-[10px] font-bold px-2 py-0.5">
                    Meta Cloud API Standard
                  </span>
                </h2>
                <p className="text-xs text-[var(--color-ink-faint)] mt-0.5">
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
                className="w-48 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm font-mono tracking-widest text-center shadow-none"
              />
              <button
                onClick={handleRegisterPhone}
                disabled={registeringPin || registerPin.length < 6}
                className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 shadow-none"
              >
                {registeringPin ? 'Registering...' : 'Register Number with PIN'}
              </button>
            </div>
          </div>

          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none space-y-4">
            <h2 className="text-base font-bold text-[var(--color-ink)]">Send Live Test Message</h2>
            <p className="text-xs text-[var(--color-ink-faint)]">
              Verify deliverability directly to your own WhatsApp account before going live.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-semibold text-[var(--color-ink)]">Recipient WhatsApp Phone Number</label>
                <input
                  type="text"
                  value={testPhone}
                  onChange={(e) => setTestPhone(e.target.value)}
                  placeholder="+14155551234"
                  className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm font-mono shadow-none"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold text-[var(--color-ink)]">Message Content</label>
                <input
                  type="text"
                  value={testBody}
                  onChange={(e) => setTestBody(e.target.value)}
                  className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm shadow-none"
                />
              </div>
            </div>

            <div className="flex justify-end">
              <button
                onClick={handleSendTest}
                disabled={testingSend}
                className="rounded-[var(--radius-sm)] bg-emerald-600 px-4 py-2 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50 shadow-none"
              >
                <span className="flex items-center gap-1">{testingSend ? 'Dispatching...' : <><Send className="h-3.5 w-3.5" /> Send Test Message</>}</span>
              </button>
            </div>

            {testResult && (
              <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent)] p-3 text-xs font-mono text-emerald-400 overflow-x-auto">
                <pre>{JSON.stringify(testResult, null, 2)}</pre>
              </div>
            )}
          </div>

          {/* Live Events Stream */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white shadow-none overflow-hidden">
            <div className="border-b border-[var(--color-border)] bg-[var(--color-accent-muted)]/60 px-5 py-3">
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">
                Recent WhatsApp Message Events ({events.length})
              </span>
            </div>

            <div className="divide-y divide-slate-100 max-h-80 overflow-y-auto">
              {events.length === 0 ? (
                <div className="p-6 text-center text-xs text-[var(--color-ink-faint)]">No events received yet.</div>
              ) : (
                events.map((ev) => (
                  <div key={ev.id} className="p-3.5 flex items-center justify-between text-xs hover:bg-[var(--color-accent-muted)]/60">
                    <div className="flex items-center gap-2">
                      <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                        ev.direction === 'inbound' ? 'bg-blue-100 text-blue-800' : 'bg-purple-100 text-purple-800'
                      }`}>
                        {ev.direction}
                      </span>
                      <span className="font-mono text-[var(--color-ink)]">{ev.providerMessageId || 'msg'}</span>
                      <span className="text-[var(--color-ink-muted)] truncate max-w-sm">"{ev.body}"</span>
                    </div>
                    <div className="flex items-center gap-3 text-[var(--color-ink-faint)]">
                      <span className="font-semibold text-[var(--color-ink)] capitalize">{ev.status}</span>
                      <span>{new Date(ev.sentAt).toLocaleTimeString()}</span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}

      {/* TAB 8: QR Code & Link Sharing */}
      {activeTab === 'qr' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none">
          <WhatsAppQRCode
            phoneNumber={whatsappHealth?.phoneNumber || data?.phoneNumber || ''}
            displayName={whatsappHealth?.displayName || data?.name || ''}
            connected={!!(whatsappHealth?.connected || data?.connected)}
          />
        </div>
      )}

      {/* TAB 10: Marketing Template Constraints */}
      {activeTab === 'marketing' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none">
          <WhatsAppMarketingConstraints />
        </div>
      )}

      {/* TAB 11: Troubleshooting */}
      {activeTab === 'troubleshooting' && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 shadow-none">
          <WhatsAppTroubleshooting
            connected={!!(whatsappHealth?.connected || data?.connected)}
          />
        </div>
      )}
    </div>
  )
}
