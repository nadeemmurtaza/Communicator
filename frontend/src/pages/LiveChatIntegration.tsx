import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type LiveChatChannelData,
  type LiveChatEvent,
  type WidgetEmbedCodes,
  type PreChatField,
  type PreChatRoutingRule,
  type PreChatTestIntakeResult,
} from '../lib/api'
import { LiveChatIcon } from '../components/icons/BrandIcons'

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

  // Gap 4: Live Chat Widget Customizer & Launcher State
  const [themeMode, setThemeMode] = useState<'light' | 'dark' | 'system'>('light')
  const [launcherStyle, setLauncherStyle] = useState<'round_bubble' | 'pill_with_text' | 'minimal_tab'>('round_bubble')
  const [launcherIcon, setLauncherIcon] = useState<'chat_bubble' | 'sparkle_ai' | 'headset_support' | 'envelope'>('chat_bubble')
  const [launcherText, setLauncherText] = useState('Chat with us')
  const [borderRadius, setBorderRadius] = useState<'rounded_lg' | 'rounded_2xl' | 'rounded_full'>('rounded_2xl')
  const [playSound, setPlaySound] = useState(true)
  const [showBranding, setShowBranding] = useState(true)
  const [allowedDomains, setAllowedDomains] = useState('bayworks.com, localhost')
  const [previewMode, setPreviewMode] = useState<'open' | 'launcher' | 'mobile'>('open')
  const [embedFormat, setEmbedFormat] = useState<'html' | 'react' | 'gtm' | 'raw'>('html')
  const [embedCodes, setEmbedCodes] = useState<WidgetEmbedCodes | null>(null)
  const [previewUrl, setPreviewUrl] = useState('/api/v1/widget/preview-page')

  // Gap 5: Pre-Chat Intake Form & Conditional Routing State
  const [preChatEnabled, setPreChatEnabled] = useState(true)
  const [preChatTitle, setPreChatTitle] = useState('Welcome to Live Chat')
  const [preChatDesc, setPreChatDesc] = useState('Please share a few details so Captain AI or our team can assist you right away.')
  const [preChatSubmitText, setPreChatSubmitText] = useState('Start Conversation')
  const [requireEmail, setRequireEmail] = useState(true)
  const [customPreChatFields, setCustomPreChatFields] = useState<PreChatField[]>([
    { id: 'f_name', name: 'fullName', label: 'Full Name', type: 'text', placeholder: 'e.g. Jane Doe', required: true, options: [], order: 1 },
    { id: 'f_email', name: 'email', label: 'Email Address', type: 'email', placeholder: 'e.g. jane@company.com', required: true, options: [], order: 2 },
    { id: 'f_inquiry', name: 'inquiryType', label: 'What can we help you with?', type: 'select', placeholder: 'Select inquiry reason', required: true, options: ['Sales & Pricing', 'Technical Support', 'Billing & Invoices', 'Partnerships & Other'], order: 3 },
    { id: 'f_company', name: 'company', label: 'Company / Organization', type: 'text', placeholder: 'e.g. Acme Innovations', required: false, options: [], order: 4 },
    { id: 'f_message', name: 'message', label: 'Initial Question or Requirement', type: 'textarea', placeholder: 'Briefly describe your request...', required: false, options: [], order: 5 },
  ])
  const [routingRules, setRoutingRules] = useState<PreChatRoutingRule[]>([
    { id: 'rule_billing', field: 'inquiryType', operator: 'equals', value: 'Billing & Invoices', assign_priority: 'high', assign_tag: 'billing', team_target: 'Finance & Billing' },
    { id: 'rule_sales', field: 'inquiryType', operator: 'equals', value: 'Sales & Pricing', assign_priority: 'high', assign_tag: 'sales_lead', team_target: 'Sales Operations' },
    { id: 'rule_tech', field: 'inquiryType', operator: 'equals', value: 'Technical Support', assign_priority: 'medium', assign_tag: 'tier1_support', team_target: 'Engineering Support' },
  ])

  // New Field Creator State
  const [showAddField, setShowAddField] = useState(false)
  const [newFieldLabel, setNewFieldLabel] = useState('')
  const [newFieldName, setNewFieldName] = useState('')
  const [newFieldType, setNewFieldType] = useState<PreChatField['type']>('text')
  const [newFieldPlaceholder, setNewFieldPlaceholder] = useState('')
  const [newFieldRequired, setNewFieldRequired] = useState(false)
  const [newFieldOptionsText, setNewFieldOptionsText] = useState('')

  // New Rule Creator State
  const [showAddRule, setShowAddRule] = useState(false)
  const [newRuleField, setNewRuleField] = useState('inquiryType')
  const [newRuleOp, setNewRuleOp] = useState<'equals' | 'contains' | 'not_equals'>('equals')
  const [newRuleVal, setNewRuleVal] = useState('')
  const [newRulePriority, setNewRulePriority] = useState<'low' | 'medium' | 'high' | 'urgent'>('high')
  const [newRuleTag, setNewRuleTag] = useState('')
  const [newRuleTeam, setNewRuleTeam] = useState('')

  // Pre-Chat Test Simulation State
  const [simAnswers, setSimAnswers] = useState<Record<string, any>>({
    fullName: 'David Hasselhoff',
    email: 'david@knightindustries.com',
    inquiryType: 'Billing & Invoices',
    company: 'Knight Industries',
    message: 'Need help reconciling our monthly autonomous fleet subscription.',
  })
  const [simIntakeResult, setSimIntakeResult] = useState<PreChatTestIntakeResult | null>(null)
  const [testingIntake, setTestingIntake] = useState(false)

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
      const [channelData, evList, customizerData, prechatData] = await Promise.all([
        api.getLiveChatChannel(),
        api.listLiveChatEvents().catch(() => []),
        api.getWidgetCustomizer().catch(() => null),
        api.getPreChatConfig().catch(() => null),
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

      if (customizerData) {
        setThemeMode(customizerData.styling.theme_mode || 'light')
        setLauncherStyle(customizerData.styling.launcher_style || 'round_bubble')
        setLauncherIcon(customizerData.styling.launcher_icon || 'chat_bubble')
        setLauncherText(customizerData.styling.launcher_text || 'Chat with us')
        setBorderRadius(customizerData.styling.border_radius || 'rounded_2xl')
        setPlaySound(customizerData.styling.play_sound ?? true)
        setShowBranding(customizerData.styling.show_branding ?? true)
        setAllowedDomains(customizerData.styling.allowed_domains || '')
        setEmbedCodes(customizerData.embed_codes)
        setPreviewUrl(customizerData.preview_url || '/api/v1/widget/preview-page')
      }

      if (prechatData) {
        setPreChatEnabled(prechatData.enabled)
        setPreChatTitle(prechatData.title || 'Welcome to Live Chat')
        setPreChatDesc(prechatData.description || 'Please share a few details so Captain AI or our team can assist you right away.')
        setPreChatSubmitText(prechatData.submit_button_text || 'Start Conversation')
        setRequireEmail(prechatData.require_email ?? true)
        if (prechatData.fields && prechatData.fields.length > 0) {
          setCustomPreChatFields(prechatData.fields)
        }
        if (prechatData.routing_rules) {
          setRoutingRules(prechatData.routing_rules)
        }
      } else {
        setPreChatEnabled(channelData.preChatForm.enabled)
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
      const [res, customizerRes] = await Promise.all([
        api.updateLiveChatChannel({
          name: inboxName,
          website_domain: websiteDomain,
          widget_color: widgetColor,
          greeting_title: greetingTitle,
          greeting_tagline: greetingTagline,
          widget_position: widgetPosition,
          avatar_url: avatarUrl,
          pre_chat_form_enabled: preChatEnabled,
          pre_chat_fields: customPreChatFields.map((f) => ({ name: f.name, label: f.label, required: f.required })),
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
        }),
        api.updateWidgetCustomizer({
          inbox_name: inboxName,
          website_domain: websiteDomain,
          widget_color: widgetColor,
          theme_mode: themeMode,
          launcher_style: launcherStyle,
          launcher_icon: launcherIcon,
          launcher_text: launcherText,
          widget_position: widgetPosition,
          greeting_title: greetingTitle,
          greeting_tagline: greetingTagline,
          avatar_url: avatarUrl,
          border_radius: borderRadius,
          play_sound: playSound,
          show_branding: showBranding,
          allowed_domains: allowedDomains,
          pre_chat_form_enabled: preChatEnabled,
          identity_validation: identityValidation,
          hmac_secret: hmacSecret,
        }),
        api.updatePreChatConfig({
          enabled: preChatEnabled,
          title: preChatTitle,
          description: preChatDesc,
          submit_button_text: preChatSubmitText,
          require_email: requireEmail,
          fields: customPreChatFields,
          routing_rules: routingRules,
        }),
      ])
      setData(res)
      if (customizerRes?.embed_codes) {
        setEmbedCodes(customizerRes.embed_codes)
      }
      setSuccess('Website Live Chat & Customizer styling saved successfully!')
      setTimeout(() => setSuccess(null), 3500)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleResetDefaults() {
    if (!window.confirm('Reset all widget styling, launcher appearance, and theme preferences to factory defaults?')) {
      return
    }
    try {
      setSaving(true)
      const res = await api.resetWidgetCustomizer()
      setWidgetColor(res.styling.widget_color)
      setThemeMode(res.styling.theme_mode)
      setLauncherStyle(res.styling.launcher_style)
      setLauncherIcon(res.styling.launcher_icon)
      setLauncherText(res.styling.launcher_text)
      setWidgetPosition(res.styling.widget_position)
      setGreetingTitle(res.styling.greeting_title)
      setGreetingTagline(res.styling.greeting_tagline)
      setAvatarUrl(res.styling.avatar_url)
      setBorderRadius(res.styling.border_radius)
      setPlaySound(res.styling.play_sound)
      setShowBranding(res.styling.show_branding)
      setAllowedDomains(res.styling.allowed_domains)
      setEmbedCodes(res.embed_codes)
      setSuccess('Widget styling reset to default factory settings!')
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


  // Gap 5 Pre-Chat Helpers
  function handleAddNewField() {
    if (!newFieldLabel.trim()) return
    const nameSlug = (newFieldName.trim() || newFieldLabel.trim())
      .toLowerCase()
      .replace(/[^a-z0-9_]/g, '')
    const options = newFieldType === 'select'
      ? newFieldOptionsText.split(',').map((s) => s.trim()).filter(Boolean)
      : []

    const newField: PreChatField = {
      id: `f_${Date.now()}`,
      name: nameSlug || `field_${customPreChatFields.length + 1}`,
      label: newFieldLabel.trim(),
      type: newFieldType,
      placeholder: newFieldPlaceholder.trim(),
      required: newFieldRequired,
      options,
      order: customPreChatFields.length + 1,
    }

    setCustomPreChatFields([...customPreChatFields, newField])
    setNewFieldLabel('')
    setNewFieldName('')
    setNewFieldPlaceholder('')
    setNewFieldOptionsText('')
    setNewFieldRequired(false)
    setShowAddField(false)
  }

  function handleRemoveCustomField(id: string) {
    setCustomPreChatFields(customPreChatFields.filter((f) => f.id !== id))
  }

  function handleMoveField(index: number, direction: 'up' | 'down') {
    if (direction === 'up' && index === 0) return
    if (direction === 'down' && index === customPreChatFields.length - 1) return
    const targetIdx = direction === 'up' ? index - 1 : index + 1
    const copy = [...customPreChatFields]
    const temp = copy[index]
    copy[index] = copy[targetIdx]
    copy[targetIdx] = temp
    copy.forEach((f, idx) => (f.order = idx + 1))
    setCustomPreChatFields(copy)
  }

  function handleToggleFieldReq(id: string) {
    setCustomPreChatFields(
      customPreChatFields.map((f) => (f.id === id ? { ...f, required: !f.required } : f))
    )
  }

  function handleAddNewRule() {
    if (!newRuleField.trim() || !newRuleVal.trim()) return
    const newRule: PreChatRoutingRule = {
      id: `rule_${Date.now()}`,
      field: newRuleField.trim(),
      operator: newRuleOp,
      value: newRuleVal.trim(),
      assign_priority: newRulePriority,
      assign_tag: newRuleTag.trim().toLowerCase(),
      team_target: newRuleTeam.trim(),
    }
    setRoutingRules([...routingRules, newRule])
    setNewRuleVal('')
    setNewRuleTag('')
    setNewRuleTeam('')
    setShowAddRule(false)
  }

  function handleRemoveRule(id: string) {
    setRoutingRules(routingRules.filter((r) => r.id !== id))
  }

  async function handleResetPreChatDefaults() {
    if (!window.confirm('Reset all pre-chat intake fields and conditional routing rules to default settings?')) {
      return
    }
    try {
      setSaving(true)
      const res = await api.resetPreChatConfig()
      setPreChatEnabled(res.enabled)
      setPreChatTitle(res.title)
      setPreChatDesc(res.description)
      setPreChatSubmitText(res.submit_button_text)
      setRequireEmail(res.require_email)
      setCustomPreChatFields(res.fields)
      setRoutingRules(res.routing_rules)
      setSuccess('Pre-Chat intake fields and routing rules reset to default!')
      setTimeout(() => setSuccess(null), 3500)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleRunIntakeSimulation() {
    try {
      setTestingIntake(true)
      const res = await api.testPreChatIntake(simAnswers)
      setSimIntakeResult(res)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setTestingIntake(false)
    }
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
              <LiveChatIcon className="h-5 w-5 text-white" />
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
                <h2 className="text-base font-semibold text-slate-900">Website Embed & SDK Exporter</h2>
                <p className="text-xs text-slate-500 mt-0.5">
                  Select your target frontend framework or tag manager to embed the real-time live chat widget powered by Captain AI.
                </p>
              </div>
              <div className="flex items-center gap-2">
                <a
                  href={`${previewUrl}?token=${encodeURIComponent(websiteToken)}`}
                  target="_blank"
                  rel="noreferrer"
                  className="rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs font-semibold text-slate-700 shadow-sm hover:bg-slate-50 flex items-center gap-1.5"
                >
                  <span>🚀 Open Live Test Page ↗</span>
                </a>
                <button
                  onClick={() => {
                    let textToCopy = embedCodes?.html_script || embedCode
                    if (embedFormat === 'react') textToCopy = embedCodes?.react_nextjs || ''
                    else if (embedFormat === 'gtm') textToCopy = embedCodes?.google_tag_manager || ''
                    else if (embedFormat === 'raw') textToCopy = embedCodes?.raw_init || ''
                    copyToClipboard(textToCopy, 'embed')
                  }}
                  className="rounded-lg bg-blue-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 flex items-center gap-1.5"
                >
                  <span>{copiedKey === 'embed' ? '✓ Copied' : '📋 Copy Snippet'}</span>
                </button>
              </div>
            </div>

            {/* Framework Selector Tabs */}
            <div className="mt-4 flex items-center gap-2 border-b border-slate-200 pb-2">
              <button
                type="button"
                onClick={() => setEmbedFormat('html')}
                className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-all ${
                  embedFormat === 'html'
                    ? 'bg-blue-50 text-blue-700 border border-blue-200 shadow-xs'
                    : 'text-slate-600 hover:text-slate-900 hover:bg-slate-50'
                }`}
              >
                🌐 Standard HTML Script
              </button>
              <button
                type="button"
                onClick={() => setEmbedFormat('react')}
                className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-all ${
                  embedFormat === 'react'
                    ? 'bg-blue-50 text-blue-700 border border-blue-200 shadow-xs'
                    : 'text-slate-600 hover:text-slate-900 hover:bg-slate-50'
                }`}
              >
                ⚛️ React / Next.js Component
              </button>
              <button
                type="button"
                onClick={() => setEmbedFormat('gtm')}
                className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-all ${
                  embedFormat === 'gtm'
                    ? 'bg-blue-50 text-blue-700 border border-blue-200 shadow-xs'
                    : 'text-slate-600 hover:text-slate-900 hover:bg-slate-50'
                }`}
              >
                🏷️ Google Tag Manager (GTM)
              </button>
              <button
                type="button"
                onClick={() => setEmbedFormat('raw')}
                className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-all ${
                  embedFormat === 'raw'
                    ? 'bg-blue-50 text-blue-700 border border-blue-200 shadow-xs'
                    : 'text-slate-600 hover:text-slate-900 hover:bg-slate-50'
                }`}
              >
                ⚡ Raw JS SDK Init
              </button>
            </div>

            {/* Active Code Box */}
            <div className="mt-4">
              <pre className="rounded-lg bg-slate-950 p-4 text-xs font-mono text-emerald-400 overflow-x-auto border border-slate-800 leading-relaxed max-h-[380px]">
                {embedFormat === 'html' && (embedCodes?.html_script || embedCode)}
                {embedFormat === 'react' && (embedCodes?.react_nextjs || '// Loading React snippet...')}
                {embedFormat === 'gtm' && (embedCodes?.google_tag_manager || '// Loading GTM snippet...')}
                {embedFormat === 'raw' && (embedCodes?.raw_init || '// Loading Raw Init snippet...')}
              </pre>
            </div>

            <div className="mt-5 grid grid-cols-1 md:grid-cols-3 gap-4 pt-4 border-t border-slate-100">
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

              <div className="rounded-lg bg-slate-50 p-3.5 border border-slate-200">
                <div className="text-xs font-semibold text-slate-800 mb-1">Allowed Host Domains</div>
                <div className="text-xs text-slate-600 truncate font-mono">
                  {allowedDomains || 'All domains permitted (*)'}
                </div>
                <div className="text-[10px] text-slate-400 mt-1">Configure in Branding & Appearance</div>
              </div>

              <div className="rounded-lg bg-slate-50 p-3.5 border border-slate-200 flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-800">Regenerate Token</div>
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
            <h3 className="text-sm font-semibold text-slate-900 mb-4">Quick Integration Guides</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="rounded-lg border border-slate-200 p-4 hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-lg">⚛️</span>
                  <span className="text-xs font-bold text-slate-800">React & Next.js</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Mount the ready-to-use <code className="text-blue-600 font-mono">&lt;CaptainLiveChat /&gt;</code> component inside your root layout or main App component.
                </p>
                <div className="rounded bg-slate-50 p-2 text-[11px] text-slate-600 border border-slate-200">
                  Import the component from our React snippet and include it in <code className="font-mono text-blue-600">app/layout.tsx</code>.
                </div>
              </div>

              <div className="rounded-lg border border-slate-200 p-4 hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-lg">🌐</span>
                  <span className="text-xs font-bold text-slate-800">WordPress & Shopify</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Go to theme settings, Header/Footer scripts, and paste the HTML snippet inside the Footer section before <code className="text-blue-600">&lt;/body&gt;</code>.
                </p>
                <div className="rounded bg-slate-50 p-2 text-[11px] text-slate-600 border border-slate-200 space-y-1">
                  <div>1. Online Store → Themes → Edit code</div>
                  <div>2. Select <code className="text-blue-600 font-mono">theme.liquid</code> or footer</div>
                  <div>3. Paste snippet before <code className="text-blue-600 font-mono">&lt;/body&gt;</code></div>
                </div>
              </div>

              <div className="rounded-lg border border-slate-200 p-4 hover:border-blue-300 transition-colors">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-lg">🏷️</span>
                  <span className="text-xs font-bold text-slate-800">Google Tag Manager</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Create a "Custom HTML" tag, paste the GTM snippet, and set the firing trigger to "All Pages - Page View".
                </p>
                <div className="rounded bg-slate-50 p-2 text-[11px] text-slate-600 border border-slate-200 space-y-1">
                  <div>1. New Tag → Custom HTML</div>
                  <div>2. Paste the GTM script snippet</div>
                  <div>3. Trigger: Initialization or All Pages</div>
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
              <div className="flex items-center justify-between border-b border-slate-100 pb-3">
                <div>
                  <h2 className="text-base font-semibold text-slate-900">
                    Live Chat Widget Customizer
                  </h2>
                  <p className="text-xs text-slate-500 mt-0.5">
                    Tailor color palette, themes, launcher button aesthetics, and real-time behavior.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={handleResetDefaults}
                  disabled={saving}
                  className="rounded-lg border border-slate-200 bg-slate-50 px-2.5 py-1 text-[11px] font-semibold text-slate-600 hover:bg-slate-100 transition-colors"
                >
                  ↺ Reset Defaults
                </button>
              </div>

              {/* General Info */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
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
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Primary Website Domain</label>
                  <input
                    type="text"
                    value={websiteDomain}
                    onChange={(e) => setWebsiteDomain(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="bayworks.com"
                  />
                </div>
              </div>

              {/* Brand Color */}
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Brand Accent Color</label>
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

              {/* Theme Mode & Border Radius */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Widget Theme Mode</label>
                  <div className="grid grid-cols-3 gap-2">
                    {[
                      { id: 'light', label: '☀️ Light' },
                      { id: 'dark', label: '🌙 Dark' },
                      { id: 'system', label: '🌓 Auto' },
                    ].map((mode) => (
                      <button
                        key={mode.id}
                        type="button"
                        onClick={() => setThemeMode(mode.id as any)}
                        className={`rounded-lg border py-2 text-xs font-semibold text-center transition-all ${
                          themeMode === mode.id
                            ? 'border-blue-600 bg-blue-50/60 text-blue-700 ring-2 ring-blue-500/20'
                            : 'border-slate-200 text-slate-600 hover:bg-slate-50'
                        }`}
                      >
                        {mode.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Corner Border Radius</label>
                  <div className="grid grid-cols-3 gap-2">
                    {[
                      { id: 'rounded_lg', label: 'Subtle', desc: '12px' },
                      { id: 'rounded_2xl', label: 'Modern', desc: '16px' },
                      { id: 'rounded_full', label: 'Pill', desc: '24px' },
                    ].map((radius) => (
                      <button
                        key={radius.id}
                        type="button"
                        onClick={() => setBorderRadius(radius.id as any)}
                        className={`rounded-lg border py-2 text-xs font-semibold text-center transition-all ${
                          borderRadius === radius.id
                            ? 'border-blue-600 bg-blue-50/60 text-blue-700 ring-2 ring-blue-500/20'
                            : 'border-slate-200 text-slate-600 hover:bg-slate-50'
                        }`}
                      >
                        <div>{radius.label}</div>
                        <div className="text-[10px] text-slate-400 font-normal">{radius.desc}</div>
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              {/* Launcher Button Style */}
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Launcher Button Appearance</label>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                  {[
                    {
                      id: 'round_bubble',
                      title: 'Round Bubble',
                      desc: 'Compact circular floating button',
                      icon: '🔘',
                    },
                    {
                      id: 'pill_with_text',
                      title: 'Pill with Text',
                      desc: 'Expanded badge with call to action',
                      icon: '💊',
                    },
                    {
                      id: 'minimal_tab',
                      title: 'Minimal Tab',
                      desc: 'Pinned corner docked button',
                      icon: '📑',
                    },
                  ].map((style) => (
                    <button
                      key={style.id}
                      type="button"
                      onClick={() => setLauncherStyle(style.id as any)}
                      className={`rounded-lg border p-3 text-left transition-all ${
                        launcherStyle === style.id
                          ? 'border-blue-600 bg-blue-50/50 ring-2 ring-blue-500/20'
                          : 'border-slate-200 hover:bg-slate-50'
                      }`}
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <span>{style.icon}</span>
                        <span className="text-xs font-bold text-slate-900">{style.title}</span>
                      </div>
                      <p className="text-[11px] text-slate-500">{style.desc}</p>
                    </button>
                  ))}
                </div>
              </div>

              {/* Launcher Icon & Launcher Text */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Launcher Icon</label>
                  <div className="grid grid-cols-4 gap-2">
                    {[
                      { id: 'chat_bubble', icon: '💬', label: 'Chat' },
                      { id: 'sparkle_ai', icon: '✨', label: 'Captain' },
                      { id: 'headset_support', icon: '🎧', label: 'Support' },
                      { id: 'envelope', icon: '✉️', label: 'Mail' },
                    ].map((item) => (
                      <button
                        key={item.id}
                        type="button"
                        onClick={() => setLauncherIcon(item.id as any)}
                        className={`rounded-lg border py-2 text-center transition-all ${
                          launcherIcon === item.id
                            ? 'border-blue-600 bg-blue-50/60 ring-2 ring-blue-500/20'
                            : 'border-slate-200 hover:bg-slate-50'
                        }`}
                      >
                        <div className="text-base">{item.icon}</div>
                        <div className="text-[10px] text-slate-600 font-semibold mt-0.5">{item.label}</div>
                      </button>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Launcher Action Text</label>
                  <input
                    type="text"
                    value={launcherText}
                    onChange={(e) => setLauncherText(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="e.g. Chat with us / Ask Captain AI"
                  />
                  <p className="text-[10px] text-slate-400 mt-1">Displayed when using Pill or Minimal Tab style</p>
                </div>
              </div>

              {/* Greeting Header & Tagline */}
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

              {/* Widget Launcher Position */}
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-2">Widget Screen Position</label>
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
                    <p className="text-[11px] text-slate-500 mt-1">Prevents overlap with right-docked CTAs</p>
                  </button>
                </div>
              </div>

              {/* Avatar URL */}
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Support Avatar / Brand Logo URL</label>
                <input
                  type="text"
                  value={avatarUrl}
                  onChange={(e) => setAvatarUrl(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                  placeholder="https://example.com/logo.png"
                />
              </div>

              {/* Toggles: Audio sound, Branding, Allowed domains */}
              <div className="rounded-lg bg-slate-50 p-4 border border-slate-200 space-y-3">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-xs font-semibold text-slate-800">Sound Notifications</div>
                    <div className="text-[11px] text-slate-500">Play pleasant audio chime when incoming agent replies arrive</div>
                  </div>
                  <input
                    type="checkbox"
                    checked={playSound}
                    onChange={(e) => setPlaySound(e.target.checked)}
                    className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>

                <div className="flex items-center justify-between border-t border-slate-200 pt-3">
                  <div>
                    <div className="text-xs font-semibold text-slate-800">Show "Powered by Captain AI" Badge</div>
                    <div className="text-[11px] text-slate-500">Display subtle brand endorsement badge in widget footer</div>
                  </div>
                  <input
                    type="checkbox"
                    checked={showBranding}
                    onChange={(e) => setShowBranding(e.target.checked)}
                    className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500"
                  />
                </div>

                <div className="border-t border-slate-200 pt-3">
                  <label className="block text-xs font-semibold text-slate-800 mb-1">Allowed Host Domains (CORS Whitelist)</label>
                  <input
                    type="text"
                    value={allowedDomains}
                    onChange={(e) => setAllowedDomains(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                    placeholder="e.g. bayworks.com, localhost, *.mycompany.com"
                  />
                  <p className="text-[10px] text-slate-400 mt-1">Comma-separated domains authorized to load this widget</p>
                </div>
              </div>

              <div className="pt-2 flex items-center justify-between">
                <button
                  type="button"
                  onClick={handleResetDefaults}
                  disabled={saving}
                  className="rounded-lg border border-slate-300 bg-white px-4 py-2 text-xs font-semibold text-slate-600 hover:bg-slate-50 disabled:opacity-50"
                >
                  Reset Defaults
                </button>
                <button
                  onClick={handleSave}
                  disabled={saving}
                  className="rounded-lg bg-blue-600 px-5 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                >
                  {saving ? 'Saving...' : 'Save Appearance'}
                </button>
              </div>
            </div>
          </div>

          {/* Real-time Widget Preview Mockup Studio */}
          <div className="lg:col-span-5">
            <div className="sticky top-6 rounded-xl border border-slate-200 bg-slate-100 p-5 shadow-sm space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-slate-600 uppercase tracking-wider">Live Mockup Studio</span>
                <span className="text-[11px] font-semibold text-slate-500 capitalize">{widgetPosition} docked</span>
              </div>

              {/* Viewport Mode Switcher */}
              <div className="flex items-center gap-1 bg-slate-200/80 p-1 rounded-lg text-xs font-semibold">
                <button
                  type="button"
                  onClick={() => setPreviewMode('open')}
                  className={`flex-1 py-1.5 rounded-md transition-all text-center ${
                    previewMode === 'open' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-600 hover:text-slate-900'
                  }`}
                >
                  💬 Open Chat
                </button>
                <button
                  type="button"
                  onClick={() => setPreviewMode('launcher')}
                  className={`flex-1 py-1.5 rounded-md transition-all text-center ${
                    previewMode === 'launcher' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-600 hover:text-slate-900'
                  }`}
                >
                  🔘 Floating Launcher
                </button>
                <button
                  type="button"
                  onClick={() => setPreviewMode('mobile')}
                  className={`flex-1 py-1.5 rounded-md transition-all text-center ${
                    previewMode === 'mobile' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-600 hover:text-slate-900'
                  }`}
                >
                  📱 Mobile View
                </button>
              </div>

              {/* 1. OPEN CHAT VIEW */}
              {previewMode === 'open' && (
                <div
                  className={`relative mx-auto h-[510px] w-full max-w-[340px] shadow-2xl border border-slate-200 overflow-hidden flex flex-col transition-all ${
                    borderRadius === 'rounded_lg'
                      ? 'rounded-xl'
                      : borderRadius === 'rounded_full'
                      ? 'rounded-3xl'
                      : 'rounded-2xl'
                  } ${themeMode === 'dark' ? 'bg-slate-900 text-slate-100' : 'bg-white text-slate-800'}`}
                >
                  {/* Widget Header */}
                  <div
                    className="p-4 text-white shrink-0 transition-colors shadow-sm"
                    style={{ backgroundColor: widgetColor }}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <img
                          src={avatarUrl || 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=128'}
                          alt="Avatar"
                          className="h-9 w-9 rounded-full border-2 border-white/60 object-cover shadow-xs"
                        />
                        <div>
                          <div className="font-bold text-sm tracking-tight">{greetingTitle}</div>
                          <div className="text-[11px] text-white/80">{greetingTagline}</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-1.5 text-white/80 text-xs">
                        <span className="h-2 w-2 rounded-full bg-emerald-400"></span>
                        <span className="text-[10px]">Online</span>
                      </div>
                    </div>
                  </div>

                  {/* Widget Body */}
                  <div
                    className={`flex-1 p-3.5 space-y-3 overflow-y-auto ${
                      themeMode === 'dark' ? 'bg-slate-950/70' : 'bg-slate-50/70'
                    }`}
                  >
                    <div
                      className={`rounded-xl p-3 shadow-xs border max-w-[85%] text-xs ${
                        themeMode === 'dark'
                          ? 'bg-slate-800 border-slate-700 text-slate-200'
                          : 'bg-white border-slate-100 text-slate-700'
                      }`}
                    >
                      {greetingMessage || '👋 Welcome to our live chat! How can our support team help you today?'}
                    </div>

                    <div
                      className="ml-auto rounded-xl p-3 shadow-xs max-w-[80%] text-xs text-white"
                      style={{ backgroundColor: widgetColor }}
                    >
                      Hi! Can you tell me about the Captain AI self-improving assistant?
                    </div>

                    <div
                      className={`rounded-xl p-3 shadow-xs border max-w-[85%] text-xs ${
                        themeMode === 'dark'
                          ? 'bg-slate-800 border-slate-700 text-slate-200'
                          : 'bg-white border-slate-100 text-slate-700'
                      }`}
                    >
                      Certainly! Captain AI autonomously handles customer inquiries with multi-provider routing, live simulations, and proactive knowledge retrieval.
                    </div>
                  </div>

                  {/* Widget Footer Input */}
                  <div
                    className={`border-t p-2.5 flex items-center gap-2 ${
                      themeMode === 'dark' ? 'bg-slate-900 border-slate-800' : 'bg-white border-slate-200'
                    }`}
                  >
                    <input
                      type="text"
                      disabled
                      placeholder="Write a message..."
                      className={`flex-1 rounded-md px-3 py-1.5 text-xs cursor-not-allowed ${
                        themeMode === 'dark'
                          ? 'bg-slate-800 border-slate-700 text-slate-400'
                          : 'bg-slate-50 border-slate-200 text-slate-400'
                      }`}
                    />
                    <div
                      className="h-7 w-7 rounded-md flex items-center justify-center text-white text-xs shrink-0 shadow-xs"
                      style={{ backgroundColor: widgetColor }}
                    >
                      ➤
                    </div>
                  </div>

                  {/* Powered by Branding Footer */}
                  {showBranding && (
                    <div
                      className={`py-1.5 text-center text-[10px] font-medium border-t ${
                        themeMode === 'dark'
                          ? 'bg-slate-900 text-slate-400 border-slate-800'
                          : 'bg-slate-50 text-slate-500 border-slate-100'
                      }`}
                    >
                      ⚡ Powered by <span className="font-semibold text-blue-600">Captain AI</span>
                    </div>
                  )}
                </div>
              )}

              {/* 2. FLOATING LAUNCHER VIEW */}
              {previewMode === 'launcher' && (
                <div className="relative mx-auto h-[510px] w-full max-w-[340px] rounded-2xl bg-white shadow-xl border border-slate-200 overflow-hidden flex flex-col">
                  {/* Dummy Website Canvas */}
                  <div className="p-3 bg-slate-100 border-b border-slate-200 flex items-center justify-between text-xs text-slate-500">
                    <div className="flex items-center gap-1.5">
                      <span className="h-2 w-2 rounded-full bg-red-400"></span>
                      <span className="h-2 w-2 rounded-full bg-amber-400"></span>
                      <span className="h-2 w-2 rounded-full bg-emerald-400"></span>
                      <span className="ml-2 font-mono text-[10px]">{websiteDomain || 'bayworks.com'}</span>
                    </div>
                  </div>

                  <div className="flex-1 p-5 space-y-4 bg-linear-to-b from-white to-slate-50">
                    <div className="h-4 w-32 bg-slate-200 rounded animate-pulse"></div>
                    <div className="h-2.5 w-48 bg-slate-200 rounded animate-pulse"></div>
                    <div className="h-24 bg-blue-50 border border-blue-100 rounded-xl p-3 text-xs text-blue-900 space-y-1">
                      <div className="font-bold">Enterprise Customer Communications</div>
                      <p className="text-[11px] text-blue-700">Autonomous customer support with multi-channel routing & Captain AI.</p>
                    </div>
                    <div className="space-y-2 pt-2">
                      <div className="h-2.5 w-full bg-slate-200 rounded"></div>
                      <div className="h-2.5 w-5/6 bg-slate-200 rounded"></div>
                      <div className="h-2.5 w-3/4 bg-slate-200 rounded"></div>
                    </div>
                  </div>

                  {/* Positioned Launcher Button */}
                  <div className={`p-4 flex ${widgetPosition === 'left' ? 'justify-start' : 'justify-end'}`}>
                    {launcherStyle === 'round_bubble' && (
                      <div
                        className="h-14 w-14 rounded-full shadow-xl flex items-center justify-center text-white text-2xl transition-transform hover:scale-105 cursor-pointer relative"
                        style={{ backgroundColor: widgetColor }}
                      >
                        {launcherIcon === 'chat_bubble' && '💬'}
                        {launcherIcon === 'sparkle_ai' && '✨'}
                        {launcherIcon === 'headset_support' && '🎧'}
                        {launcherIcon === 'envelope' && '✉️'}
                        <span className="absolute -top-0.5 -right-0.5 h-3.5 w-3.5 rounded-full bg-red-500 border-2 border-white"></span>
                      </div>
                    )}

                    {launcherStyle === 'pill_with_text' && (
                      <div
                        className="h-12 px-4 rounded-full shadow-xl flex items-center gap-2 text-white text-xs font-semibold transition-transform hover:scale-105 cursor-pointer"
                        style={{ backgroundColor: widgetColor }}
                      >
                        <span className="text-base">
                          {launcherIcon === 'chat_bubble' && '💬'}
                          {launcherIcon === 'sparkle_ai' && '✨'}
                          {launcherIcon === 'headset_support' && '🎧'}
                          {launcherIcon === 'envelope' && '✉️'}
                        </span>
                        <span>{launcherText || 'Chat with us'}</span>
                      </div>
                    )}

                    {launcherStyle === 'minimal_tab' && (
                      <div
                        className="h-10 px-4 rounded-t-xl shadow-lg flex items-center gap-2 text-white text-xs font-semibold cursor-pointer"
                        style={{ backgroundColor: widgetColor }}
                      >
                        <span className="text-sm">
                          {launcherIcon === 'chat_bubble' && '💬'}
                          {launcherIcon === 'sparkle_ai' && '✨'}
                          {launcherIcon === 'headset_support' && '🎧'}
                          {launcherIcon === 'envelope' && '✉️'}
                        </span>
                        <span>{launcherText || 'Chat'}</span>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* 3. MOBILE DEVICE FRAME VIEW */}
              {previewMode === 'mobile' && (
                <div className="relative mx-auto h-[510px] w-full max-w-[280px] rounded-3xl bg-slate-900 p-2.5 shadow-2xl border-4 border-slate-800 flex flex-col">
                  {/* Smartphone Screen */}
                  <div className="flex-1 rounded-2xl bg-white overflow-hidden flex flex-col">
                    {/* Status bar */}
                    <div className="h-6 bg-slate-900 text-white flex items-center justify-between px-3 text-[9px]">
                      <span>9:41</span>
                      <div className="h-2.5 w-12 bg-black rounded-full mx-auto"></div>
                      <span>5G 100%</span>
                    </div>

                    {/* Mobile Header */}
                    <div className="p-3 text-white flex items-center gap-2" style={{ backgroundColor: widgetColor }}>
                      <span className="text-xs">←</span>
                      <img
                        src={avatarUrl || 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=128'}
                        alt="Avatar"
                        className="h-7 w-7 rounded-full object-cover border border-white"
                      />
                      <div className="truncate">
                        <div className="text-xs font-bold truncate">{greetingTitle}</div>
                        <div className="text-[9px] text-white/80 truncate">{greetingTagline}</div>
                      </div>
                    </div>

                    {/* Mobile Chat Body */}
                    <div className="flex-1 p-2.5 space-y-2 bg-slate-50 text-[11px] overflow-y-auto">
                      <div className="bg-white p-2 rounded-lg border border-slate-200 text-slate-800">
                        {greetingMessage || '👋 Welcome to live chat!'}
                      </div>
                      <div className="ml-auto bg-blue-600 text-white p-2 rounded-lg max-w-[85%]">
                        Hello from iPhone mobile browser!
                      </div>
                    </div>

                    {/* Mobile Input */}
                    <div className="p-2 border-t border-slate-200 flex items-center gap-1.5 bg-white">
                      <input
                        type="text"
                        disabled
                        placeholder="Reply..."
                        className="flex-1 text-[11px] bg-slate-100 rounded px-2 py-1"
                      />
                      <div
                        className="h-6 w-6 rounded flex items-center justify-center text-white text-[10px]"
                        style={{ backgroundColor: widgetColor }}
                      >
                        ➤
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Launcher preview helper text */}
              <div className="flex items-center justify-between text-[11px] text-slate-500 pt-1">
                <span>Theme: <strong className="text-slate-700 capitalize">{themeMode}</strong></span>
                <span>Radius: <strong className="text-slate-700">{borderRadius.replace('rounded_', '')}</strong></span>
                <span>Sound: <strong className="text-slate-700">{playSound ? 'On' : 'Off'}</strong></span>
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
                  <h2 className="text-base font-semibold text-slate-900">Pre-Chat Intake Form Builder</h2>
                  <p className="text-xs text-slate-500 mt-0.5">
                    Collect visitor credentials, qualify requirements, and automatically route conversations before chat opens.
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={handleResetPreChatDefaults}
                    disabled={saving}
                    className="rounded-lg border border-slate-200 bg-slate-50 px-2.5 py-1 text-[11px] font-semibold text-slate-600 hover:bg-slate-100 transition-colors"
                  >
                    ↺ Reset Defaults
                  </button>
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
              </div>

              {preChatEnabled ? (
                <div className="space-y-6">
                  {/* Form Header Customization */}
                  <div className="rounded-lg bg-slate-50 p-4 border border-slate-200 space-y-3">
                    <div className="text-xs font-bold text-slate-800 uppercase tracking-wider">Form Presentation & Instructions</div>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                      <div>
                        <label className="block text-[11px] font-semibold text-slate-700 mb-1">Form Heading Title</label>
                        <input
                          type="text"
                          value={preChatTitle}
                          onChange={(e) => setPreChatTitle(e.target.value)}
                          className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                          placeholder="Welcome to Live Chat"
                        />
                      </div>
                      <div>
                        <label className="block text-[11px] font-semibold text-slate-700 mb-1">Submit CTA Button Label</label>
                        <input
                          type="text"
                          value={preChatSubmitText}
                          onChange={(e) => setPreChatSubmitText(e.target.value)}
                          className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                          placeholder="Start Conversation"
                        />
                      </div>
                    </div>
                    <div>
                      <label className="block text-[11px] font-semibold text-slate-700 mb-1">Introductory Prompt</label>
                      <textarea
                        rows={2}
                        value={preChatDesc}
                        onChange={(e) => setPreChatDesc(e.target.value)}
                        className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs focus:border-blue-500 focus:outline-none"
                        placeholder="Please share a few details so we can assist you..."
                      />
                    </div>
                    <div className="flex items-center gap-2 pt-1">
                      <input
                        type="checkbox"
                        id="reqEmailToggle"
                        checked={requireEmail}
                        onChange={(e) => setRequireEmail(e.target.checked)}
                        className="h-3.5 w-3.5 rounded border-slate-300 text-blue-600"
                      />
                      <label htmlFor="reqEmailToggle" className="text-xs text-slate-700 font-medium">
                        Strict email validation: require valid RFC-compliant email to start conversation
                      </label>
                    </div>
                  </div>

                  {/* Field Builder */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold text-slate-800 uppercase tracking-wider">
                        Configured Form Fields ({customPreChatFields.length})
                      </span>
                      <button
                        type="button"
                        onClick={() => setShowAddField(!showAddField)}
                        className="rounded-lg bg-blue-50 border border-blue-200 px-2.5 py-1 text-xs font-semibold text-blue-700 hover:bg-blue-100 flex items-center gap-1"
                      >
                        <span>{showAddField ? '✕ Cancel' : '+ Add Custom Field'}</span>
                      </button>
                    </div>

                    {/* Inline Add Field Form */}
                    {showAddField && (
                      <div className="rounded-xl border border-blue-200 bg-blue-50/40 p-4 space-y-3 shadow-xs">
                        <div className="text-xs font-bold text-blue-900">New Field Definition</div>
                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Field Label</label>
                            <input
                              type="text"
                              value={newFieldLabel}
                              onChange={(e) => {
                                setNewFieldLabel(e.target.value)
                                if (!newFieldName) {
                                  setNewFieldName(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ''))
                                }
                              }}
                              placeholder="e.g. Budget Range"
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            />
                          </div>
                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Variable Key (Slug)</label>
                            <input
                              type="text"
                              value={newFieldName}
                              onChange={(e) => setNewFieldName(e.target.value)}
                              placeholder="e.g. budgetRange"
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-mono"
                            />
                          </div>
                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Field Type</label>
                            <select
                              value={newFieldType}
                              onChange={(e) => setNewFieldType(e.target.value as any)}
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            >
                              <option value="text">Text (single line)</option>
                              <option value="email">Email Address</option>
                              <option value="phone">Phone Number</option>
                              <option value="select">Dropdown Select</option>
                              <option value="textarea">Textarea (multiline)</option>
                              <option value="number">Number</option>
                              <option value="checkbox">Checkbox (boolean)</option>
                            </select>
                          </div>
                        </div>

                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Placeholder Text</label>
                            <input
                              type="text"
                              value={newFieldPlaceholder}
                              onChange={(e) => setNewFieldPlaceholder(e.target.value)}
                              placeholder="e.g. Select your budget tier..."
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            />
                          </div>
                          {newFieldType === 'select' && (
                            <div>
                              <label className="block text-[11px] font-semibold text-slate-700 mb-1">Choices (Comma-Separated)</label>
                              <input
                                type="text"
                                value={newFieldOptionsText}
                                onChange={(e) => setNewFieldOptionsText(e.target.value)}
                                placeholder="<$1k, $1k-$5k, $5k-$20k, $20k+"
                                className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                              />
                            </div>
                          )}
                        </div>

                        <div className="flex items-center justify-between pt-1">
                          <label className="flex items-center gap-1.5 text-xs font-semibold text-slate-700 cursor-pointer">
                            <input
                              type="checkbox"
                              checked={newFieldRequired}
                              onChange={(e) => setNewFieldRequired(e.target.checked)}
                              className="h-3.5 w-3.5 rounded border-slate-300 text-blue-600"
                            />
                            <span>Mark this field as mandatory (Required)</span>
                          </label>
                          <button
                            type="button"
                            onClick={handleAddNewField}
                            disabled={!newFieldLabel.trim()}
                            className="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-blue-700 disabled:opacity-50"
                          >
                            Add Field
                          </button>
                        </div>
                      </div>
                    )}

                    {/* Field List */}
                    <div className="divide-y divide-slate-100 rounded-lg border border-slate-200 overflow-hidden bg-white shadow-xs">
                      {customPreChatFields.map((field, idx) => (
                        <div key={field.id || field.name} className="flex flex-col sm:flex-row sm:items-center justify-between p-3 gap-2 hover:bg-slate-50/50 transition-colors">
                          <div className="flex items-center gap-3">
                            <span className="text-slate-400 font-mono text-xs w-5">#{idx + 1}</span>
                            <div>
                              <div className="flex items-center gap-2">
                                <span className="text-xs font-semibold text-slate-900">{field.label}</span>
                                <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-mono text-slate-600 uppercase">
                                  {field.type}
                                </span>
                              </div>
                              <div className="text-[11px] font-mono text-slate-400 mt-0.5">
                                key: <span className="text-slate-600">{field.name}</span>
                                {field.options && field.options.length > 0 && (
                                  <span className="ml-2 font-sans text-slate-500">
                                    • {field.options.length} choices ({field.options.slice(0, 3).join(', ')}{field.options.length > 3 ? '...' : ''})
                                  </span>
                                )}
                              </div>
                            </div>
                          </div>

                          <div className="flex items-center gap-2 shrink-0">
                            {/* Reorder buttons */}
                            <button
                              type="button"
                              onClick={() => handleMoveField(idx, 'up')}
                              disabled={idx === 0}
                              className="h-6 w-6 rounded border border-slate-200 flex items-center justify-center text-xs text-slate-600 hover:bg-slate-100 disabled:opacity-30"
                              title="Move Up"
                            >
                              ↑
                            </button>
                            <button
                              type="button"
                              onClick={() => handleMoveField(idx, 'down')}
                              disabled={idx === customPreChatFields.length - 1}
                              className="h-6 w-6 rounded border border-slate-200 flex items-center justify-center text-xs text-slate-600 hover:bg-slate-100 disabled:opacity-30"
                              title="Move Down"
                            >
                              ↓
                            </button>

                            {/* Required toggle */}
                            <button
                              type="button"
                              onClick={() => handleToggleFieldReq(field.id)}
                              className={`rounded px-2 py-0.5 text-[11px] font-semibold border transition-all ${
                                field.required
                                  ? 'bg-amber-50 text-amber-700 border-amber-200'
                                  : 'bg-slate-50 text-slate-500 border-slate-200 hover:bg-slate-100'
                              }`}
                            >
                              {field.required ? '★ Required' : 'Optional'}
                            </button>

                            {/* Delete field */}
                            <button
                              type="button"
                              onClick={() => handleRemoveCustomField(field.id)}
                              className="h-6 w-6 rounded border border-red-200 text-red-600 hover:bg-red-50 flex items-center justify-center text-xs"
                              title="Delete Field"
                            >
                              ✕
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Conditional Routing Rules */}
                  <div className="space-y-3 pt-3 border-t border-slate-100">
                    <div className="flex items-center justify-between">
                      <div>
                        <span className="text-xs font-bold text-slate-800 uppercase tracking-wider">
                          Conditional Lead Triage & Routing Rules ({routingRules.length})
                        </span>
                        <p className="text-[11px] text-slate-500 mt-0.5">
                          Automatically tag, prioritize, and route conversation to teams based on submitted answers.
                        </p>
                      </div>
                      <button
                        type="button"
                        onClick={() => setShowAddRule(!showAddRule)}
                        className="rounded-lg bg-emerald-50 border border-emerald-200 px-2.5 py-1 text-xs font-semibold text-emerald-700 hover:bg-emerald-100 flex items-center gap-1"
                      >
                        <span>{showAddRule ? '✕ Cancel' : '+ Add Routing Rule'}</span>
                      </button>
                    </div>

                    {/* Inline Add Rule Form */}
                    {showAddRule && (
                      <div className="rounded-xl border border-emerald-200 bg-emerald-50/40 p-4 space-y-3 shadow-xs">
                        <div className="text-xs font-bold text-emerald-900">Define Conditional Routing Logic</div>
                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Evaluate Field</label>
                            <select
                              value={newRuleField}
                              onChange={(e) => setNewRuleField(e.target.value)}
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-mono"
                            >
                              {customPreChatFields.map((f) => (
                                <option key={f.name} value={f.name}>
                                  {f.label} ({f.name})
                                </option>
                              ))}
                            </select>
                          </div>

                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Condition</label>
                            <select
                              value={newRuleOp}
                              onChange={(e) => setNewRuleOp(e.target.value as any)}
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            >
                              <option value="equals">Equals (exact match)</option>
                              <option value="contains">Contains substring</option>
                              <option value="not_equals">Does not equal</option>
                            </select>
                          </div>

                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Value to Match</label>
                            <input
                              type="text"
                              value={newRuleVal}
                              onChange={(e) => setNewRuleVal(e.target.value)}
                              placeholder="e.g. Enterprise or Billing"
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            />
                          </div>
                        </div>

                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Set Priority</label>
                            <select
                              value={newRulePriority}
                              onChange={(e) => setNewRulePriority(e.target.value as any)}
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            >
                              <option value="low">Low Priority</option>
                              <option value="medium">Medium Priority</option>
                              <option value="high">High Priority</option>
                              <option value="urgent">⚡ Urgent Priority</option>
                            </select>
                          </div>

                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Attach Tag</label>
                            <input
                              type="text"
                              value={newRuleTag}
                              onChange={(e) => setNewRuleTag(e.target.value)}
                              placeholder="e.g. enterprise_lead"
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            />
                          </div>

                          <div>
                            <label className="block text-[11px] font-semibold text-slate-700 mb-1">Target Team / Queue</label>
                            <input
                              type="text"
                              value={newRuleTeam}
                              onChange={(e) => setNewRuleTeam(e.target.value)}
                              placeholder="e.g. Strategic Sales"
                              className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs"
                            />
                          </div>
                        </div>

                        <div className="flex justify-end pt-1">
                          <button
                            type="button"
                            onClick={handleAddNewRule}
                            disabled={!newRuleVal.trim()}
                            className="rounded-lg bg-emerald-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50"
                          >
                            Save Routing Rule
                          </button>
                        </div>
                      </div>
                    )}

                    {/* Routing Rules List */}
                    <div className="space-y-2">
                      {routingRules.map((rule) => (
                        <div
                          key={rule.id}
                          className="flex items-center justify-between rounded-lg border border-slate-200 bg-slate-50/60 p-3 text-xs"
                        >
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="font-semibold text-slate-700">IF</span>
                            <code className="rounded bg-white px-1.5 py-0.5 border border-slate-200 font-mono text-blue-600">
                              {rule.field}
                            </code>
                            <span className="text-slate-500">{rule.operator}</span>
                            <span className="font-bold text-slate-900">"{rule.value}"</span>
                            <span className="text-slate-400">➔</span>
                            <span className="rounded bg-amber-50 px-2 py-0.5 font-bold uppercase text-[10px] text-amber-700 border border-amber-200">
                              {rule.assign_priority} Priority
                            </span>
                            {rule.assign_tag && (
                              <span className="rounded bg-blue-50 px-2 py-0.5 text-[10px] font-semibold text-blue-700 border border-blue-200">
                                #{rule.assign_tag}
                              </span>
                            )}
                            {rule.team_target && (
                              <span className="rounded bg-purple-50 px-2 py-0.5 text-[10px] font-semibold text-purple-700 border border-purple-200">
                                Team: {rule.team_target}
                              </span>
                            )}
                          </div>
                          <button
                            type="button"
                            onClick={() => handleRemoveRule(rule.id)}
                            className="text-red-500 hover:text-red-700 p-1 text-xs shrink-0"
                            title="Remove Rule"
                          >
                            ✕
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="pt-2">
                    <button
                      onClick={handleSave}
                      disabled={saving}
                      className="rounded-lg bg-blue-600 px-5 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                    >
                      {saving ? 'Saving...' : 'Save Pre-Chat Form Configuration'}
                    </button>
                  </div>
                </div>
              ) : (
                <div className="rounded-lg bg-slate-50 p-6 text-center border border-dashed border-slate-200">
                  <span className="text-2xl">⚡</span>
                  <div className="text-xs font-semibold text-slate-800 mt-2">Pre-Chat Qualification is Disabled</div>
                  <p className="text-[11px] text-slate-500 max-w-sm mx-auto mt-1">
                    Visitors will immediately see the conversation message thread without answering any required form fields.
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Right Column: Live Interactive Visitor Preview & Simulator */}
          <div className="lg:col-span-5">
            <div className="sticky top-6 rounded-xl border border-slate-200 bg-slate-100 p-5 shadow-sm space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-slate-600 uppercase tracking-wider">
                  Live Intake Preview & Simulator
                </span>
                <span className="text-[11px] font-semibold text-emerald-600">● Interactive</span>
              </div>

              {/* Mockup Container */}
              <div
                className={`relative mx-auto h-[510px] w-full max-w-[340px] shadow-2xl border border-slate-200 overflow-hidden flex flex-col ${
                  borderRadius === 'rounded_lg'
                    ? 'rounded-xl'
                    : borderRadius === 'rounded_full'
                    ? 'rounded-3xl'
                    : 'rounded-2xl'
                } ${themeMode === 'dark' ? 'bg-slate-900 text-slate-100' : 'bg-white text-slate-800'}`}
              >
                {/* Header */}
                <div className="p-4 text-white shrink-0 shadow-sm" style={{ backgroundColor: widgetColor }}>
                  <div className="flex items-center gap-3">
                    <img
                      src={avatarUrl || 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=128'}
                      alt="Avatar"
                      className="h-9 w-9 rounded-full border-2 border-white/60 object-cover shadow-xs"
                    />
                    <div>
                      <div className="font-bold text-sm tracking-tight">{greetingTitle}</div>
                      <div className="text-[11px] text-white/80">{greetingTagline}</div>
                    </div>
                  </div>
                </div>

                {/* Form Body */}
                <div
                  className={`flex-1 p-4 space-y-3 overflow-y-auto ${
                    themeMode === 'dark' ? 'bg-slate-950/70' : 'bg-slate-50/70'
                  }`}
                >
                  <div className="border-b border-slate-200 pb-2">
                    <div className="text-xs font-bold text-slate-900 dark:text-white">{preChatTitle}</div>
                    <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">{preChatDesc}</div>
                  </div>

                  {customPreChatFields.map((field) => (
                    <div key={field.name} className="space-y-1">
                      <label className="block text-[11px] font-semibold text-slate-700 dark:text-slate-300">
                        {field.label} {field.required && <span className="text-red-500">*</span>}
                      </label>
                      {field.type === 'select' ? (
                        <select
                          value={simAnswers[field.name] || ''}
                          onChange={(e) => setSimAnswers({ ...simAnswers, [field.name]: e.target.value })}
                          className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-900 dark:bg-slate-800 dark:border-slate-700 dark:text-white"
                        >
                          <option value="">{field.placeholder || 'Select an option'}</option>
                          {field.options?.map((opt) => (
                            <option key={opt} value={opt}>
                              {opt}
                            </option>
                          ))}
                        </select>
                      ) : field.type === 'textarea' ? (
                        <textarea
                          rows={2}
                          value={simAnswers[field.name] || ''}
                          onChange={(e) => setSimAnswers({ ...simAnswers, [field.name]: e.target.value })}
                          placeholder={field.placeholder || ''}
                          className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-900 dark:bg-slate-800 dark:border-slate-700 dark:text-white"
                        />
                      ) : (
                        <input
                          type={field.type === 'number' ? 'number' : field.type === 'email' ? 'email' : 'text'}
                          value={simAnswers[field.name] || ''}
                          onChange={(e) => setSimAnswers({ ...simAnswers, [field.name]: e.target.value })}
                          placeholder={field.placeholder || ''}
                          className="w-full rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-900 dark:bg-slate-800 dark:border-slate-700 dark:text-white"
                        />
                      )}
                    </div>
                  ))}

                  <div className="pt-2">
                    <button
                      type="button"
                      onClick={handleRunIntakeSimulation}
                      disabled={testingIntake}
                      className="w-full rounded-md py-2 text-xs font-bold text-white shadow-sm transition-opacity hover:opacity-95"
                      style={{ backgroundColor: widgetColor }}
                    >
                      {testingIntake ? 'Evaluating Triage...' : preChatSubmitText}
                    </button>
                  </div>
                </div>

                {/* Footer Branding */}
                {showBranding && (
                  <div
                    className={`py-1.5 text-center text-[10px] font-medium border-t ${
                      themeMode === 'dark'
                        ? 'bg-slate-900 text-slate-400 border-slate-800'
                        : 'bg-white text-slate-500 border-slate-200'
                    }`}
                  >
                    ⚡ Powered by <span className="font-semibold text-blue-600">Captain AI</span>
                  </div>
                )}
              </div>

              {/* Simulation Result Box */}
              {simIntakeResult && (
                <div className="rounded-xl border border-blue-200 bg-white p-3.5 shadow-sm space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-xs font-bold text-slate-800">Triage & Routing Evaluation</span>
                    <span
                      className={`rounded px-2 py-0.5 text-[10px] font-bold uppercase ${
                        simIntakeResult.valid ? 'bg-emerald-50 text-emerald-700 border border-emerald-200' : 'bg-red-50 text-red-700 border border-red-200'
                      }`}
                    >
                      {simIntakeResult.valid ? '✓ Valid Intake' : 'Validation Error'}
                    </span>
                  </div>

                  {simIntakeResult.errors.length > 0 && (
                    <div className="text-[11px] text-red-600 space-y-0.5">
                      {simIntakeResult.errors.map((e, idx) => (
                        <div key={idx}>• {e}</div>
                      ))}
                    </div>
                  )}

                  {simIntakeResult.valid && (
                    <div className="space-y-1.5 text-xs pt-1">
                      <div className="flex items-center justify-between">
                        <span className="text-slate-500">Calculated Priority:</span>
                        <span className="font-bold text-amber-700 uppercase">
                          {simIntakeResult.routing_result.priority}
                        </span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-500">Target Team / Queue:</span>
                        <span className="font-semibold text-purple-700">
                          {simIntakeResult.routing_result.team_target || 'General Support Queue'}
                        </span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-500">Attached Tags:</span>
                        <div className="flex gap-1">
                          {simIntakeResult.routing_result.tags.map((t) => (
                            <span key={t} className="rounded bg-blue-50 text-blue-700 px-1.5 py-0.2 text-[10px] font-mono">
                              #{t}
                            </span>
                          ))}
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              )}
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
