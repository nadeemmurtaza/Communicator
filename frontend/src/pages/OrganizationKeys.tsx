import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import {
  Loader2, CheckCircle2, AlertCircle, Save, Shield,
  MessageSquare, Phone, Mail, Bot, Globe
} from '../components/icons'

const channels = [
  { id: 'whatsapp', name: 'WhatsApp', icon: MessageSquare, fields: [
    { key: 'meta_access_token', label: 'Access Token' },
    { key: 'meta_phone_number_id', label: 'Phone Number ID' },
    { key: 'meta_app_secret', label: 'App Secret' },
  ]},
  { id: 'sms', name: 'SMS / Twilio', icon: Phone, fields: [
    { key: 'twilio_account_sid', label: 'Account SID' },
    { key: 'twilio_auth_token', label: 'Auth Token' },
    { key: 'twilio_sms_from', label: 'From Number' },
  ]},
  { id: 'telegram', name: 'Telegram', icon: MessageSquare, fields: [
    { key: 'telegram_bot_token', label: 'Bot Token' },
  ]},
  { id: 'line', name: 'LINE', icon: MessageSquare, fields: [
    { key: 'line_channel_access_token', label: 'Channel Access Token' },
    { key: 'line_channel_secret', label: 'Channel Secret' },
  ]},
  { id: 'tiktok', name: 'TikTok', icon: Globe, fields: [
    { key: 'tiktok_app_id', label: 'App ID' },
    { key: 'tiktok_app_secret', label: 'App Secret' },
    { key: 'tiktok_access_token', label: 'Access Token' },
  ]},
  { id: 'facebook', name: 'Facebook', icon: Globe, fields: [
    { key: 'facebook_page_access_token', label: 'Page Access Token' },
    { key: 'facebook_page_id', label: 'Page ID' },
    { key: 'facebook_app_secret', label: 'App Secret' },
  ]},
  { id: 'instagram', name: 'Instagram', icon: Globe, fields: [
    { key: 'instagram_access_token', label: 'Access Token' },
    { key: 'instagram_account_id', label: 'Account ID' },
  ]},
  { id: 'email', name: 'Email', icon: Mail, fields: [
    { key: 'email_smtp_host', label: 'SMTP Host' },
    { key: 'email_smtp_port', label: 'SMTP Port' },
    { key: 'email_smtp_username', label: 'SMTP Username' },
    { key: 'email_smtp_password', label: 'SMTP Password' },
    { key: 'email_from_address', label: 'From Address' },
    { key: 'postmark_api_token', label: 'Postmark API Token' },
  ]},
]

const aiProviders = [
  { id: 'anthropic', name: 'Anthropic Claude', icon: Bot, fields: [
    { key: 'anthropic_api_key', label: 'API Key' },
    { key: 'anthropic_model', label: 'Model' },
    { key: 'anthropic_base_url', label: 'Base URL (optional)' },
  ]},
  { id: 'openai', name: 'OpenAI', icon: Bot, fields: [
    { key: 'openai_api_key', label: 'API Key' },
    { key: 'openai_model', label: 'Model' },
    { key: 'openai_base_url', label: 'Base URL (optional)' },
  ]},
  { id: 'embedding', name: 'Embeddings (for RAG)', icon: Shield, fields: [
    { key: 'embedding_provider', label: 'Provider' },
    { key: 'embedding_api_key', label: 'API Key' },
    { key: 'embedding_model', label: 'Model' },
  ]},
]

export function OrganizationKeys() {
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [data, setData] = useState<any>({})
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState(false)
  const [visible, setVisible] = useState<Set<string>>(new Set())

  useEffect(() => {
    api.getOrganizationSettings()
      .then(setData)
      .catch(() => setError('Failed to load settings'))
      .finally(() => setLoading(false))
  }, [])

  async function handleSave() {
    setSaving(true)
    setSaved(false)
    try {
      await api.updateOrganizationSettings(data)
      setSaved(true)
      setTimeout(() => setSaved(false), 3000)
    } catch {
      setError('Failed to save settings')
    } finally {
      setSaving(false)
    }
  }

  function toggleVisible(key: string) {
    setVisible((prev: Set<string>) => {
      const next = new Set(prev)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  }

  function update(key: string, value: string) {
    setData((prev: any) => ({ ...prev, [key]: value }))
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-6 w-6 animate-spin text-[var(--color-ink-faint)]" />
        <span className="ml-2 text-sm text-[var(--color-ink-faint)]">Loading API keys...</span>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-ink)]">API Keys (BYOK)</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-1">
            Configure your own API keys for each channel and AI service. Keys are stored encrypted per-organization.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {saved && (
            <div className="flex items-center gap-2 text-sm text-green-600 bg-green-50 px-3 py-1.5 rounded-[var(--radius-md)]">
              <CheckCircle2 className="h-4 w-4" />
              <span>Saved</span>
            </div>
          )}
          <button onClick={handleSave} disabled={saving}
            className="px-4 py-2 bg-[var(--color-accent)] text-[var(--color-on-accent)] text-sm font-medium rounded-[var(--radius-md)] hover:opacity-90 transition disabled:opacity-50 flex items-center gap-2">
            {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
            <span>Save All</span>
          </button>
        </div>
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-[var(--radius-md)] border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      <div className="p-3 bg-blue-50 border border-blue-200 rounded-[var(--radius-md)] text-sm text-blue-700 flex items-start gap-2">
        <Shield className="h-4 w-4 mt-0.5 shrink-0" />
        <div>
          <p className="font-medium">Bring Your Own Keys (BYOK)</p>
          <p className="text-xs text-blue-600 mt-1">
            Each organization manages its own API keys. Keys are encrypted at rest. Leave empty to use the system default (if configured).
          </p>
        </div>
      </div>

      {/* Channel Keys */}
      <Section title="Channel API Keys">
        <div className="space-y-6">
          {channels.map(channel => (
            <ChannelGroup key={channel.id} channel={channel} data={data} visible={visible}
              toggleVisible={toggleVisible} update={update} />
          ))}
        </div>
      </Section>

      {/* AI Provider Keys */}
      <Section title="AI Provider Keys">
        <div className="space-y-6">
          {aiProviders.map(provider => (
            <ChannelGroup key={provider.id} channel={provider} data={data} visible={visible}
              toggleVisible={toggleVisible} update={update} />
          ))}
        </div>
      </Section>
    </div>
  )
}

function ChannelGroup({ channel, data, visible, toggleVisible, update }: {
  channel: any; data: any; visible: Set<string>;
  toggleVisible: (k: string) => void; update: (k: string, v: string) => void
}) {
  const Icon = channel.icon
  const hasAnyKey = channel.fields.some((f: any) => data[f.key]?.trim())

  return (
    <div className="border border-[var(--color-border)] rounded-[var(--radius-md)] overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 bg-[var(--color-accent-muted)] border-b border-[var(--color-border)]">
        <div className="flex items-center gap-3">
          <div className="h-8 w-8 rounded-[var(--radius-md)] bg-[var(--color-accent-soft)] flex items-center justify-center">
            <Icon className="h-4 w-4 text-[var(--color-accent)]" />
          </div>
          <div>
            <p className="text-sm font-semibold text-[var(--color-ink)]">{channel.name}</p>
            {hasAnyKey && <p className="text-xs text-green-600">Configured</p>}
          </div>
        </div>
      </div>
      <div className="p-4 space-y-3">
        {channel.fields.map((field: any) => (
          <div key={field.key}>
            <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">{field.label}</label>
            <div className="relative">
              <input
                type={visible.has(field.key) ? 'text' : 'password'}
                value={data[field.key] || ''}
                onChange={(e) => update(field.key, e.target.value)}
                placeholder={`Enter ${field.label.toLowerCase()}`}
                className="w-full px-3 py-2 pr-10 border border-[var(--color-border)] rounded-[var(--radius-md)] text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none font-mono"
              />
              <button type="button" onClick={() => toggleVisible(field.key)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)]">
                {visible.has(field.key)
                  ? '👁' : '🔒'
                }
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h2 className="text-lg font-semibold text-[var(--color-ink)] mb-4">{title}</h2>
      {children}
    </div>
  )
}
