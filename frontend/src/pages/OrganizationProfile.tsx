import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import {
  Building2, Globe, Mail, Phone, Users, MessageSquare,
  Bot, Shield, Loader2, CheckCircle2, XCircle, AlertCircle, Save
} from '../components/icons'

export function OrganizationProfile() {
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [data, setData] = useState<any>(null)
  const [stats, setStats] = useState<any>(null)
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    Promise.all([
      api.getOrganizationProfile().catch(() => null),
      api.getOrganizationStats().catch(() => null),
    ]).then(([profile, s]) => {
      setData(profile)
      setStats(s)
      setLoading(false)
    }).catch(() => {
      setError('Failed to load organization profile')
      setLoading(false)
    })
  }, [])

  async function handleSave(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setSaved(false)
    try {
      await api.updateOrganizationProfile(data)
      setSaved(true)
      setTimeout(() => setSaved(false), 3000)
    } catch {
      setError('Failed to save profile')
    } finally {
      setSaving(false)
    }
  }

  function update(field: string, value: string) {
    setData((prev: any) => ({ ...prev, [field]: value }))
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
        <span className="ml-2 text-sm text-gray-500">Loading organization profile...</span>
      </div>
    )
  }

  const connections = data?.connections || {}
  const connectedCount = Object.values(connections).filter((c: any) => c?.connected).length
  const totalChannels = Object.keys(connections).length

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-ink)]">Organization Profile</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-1">
            Manage your business information, connected channels, and AI agent configuration.
          </p>
        </div>
        {saved && (
          <div className="flex items-center gap-2 text-sm text-green-600 bg-green-50 px-3 py-1.5 rounded-lg">
            <CheckCircle2 className="h-4 w-4" />
            <span>Saved</span>
          </div>
        )}
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Stats Overview */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard icon={Users} label="Contacts" value={stats?.totalContacts ?? '—'} />
        <StatCard icon={MessageSquare} label="Conversations" value={stats?.totalConversations ?? '—'} />
        <StatCard icon={Bot} label="AI Replies" value={stats?.totalDrafts ?? '—'} />
        <StatCard icon={Shield} label="Channels Connected" value={`${connectedCount}/${totalChannels}`} />
      </div>

      <form onSubmit={handleSave} className="space-y-6">
        {/* Business Info */}
        <Section title="Business Information">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Field label="Business Name" value={data?.businessName || ''}
              onChange={(v) => update('businessName', v)} icon={Building2} />
            <Field label="Website" value={data?.organizationWebsite || ''}
              onChange={(v) => update('organizationWebsite', v)} icon={Globe} />
            <Field label="Industry" value={data?.organizationIndustry || ''}
              onChange={(v) => update('organizationIndustry', v)} />
            <Field label="Company Size" value={data?.organizationSize || ''}
              onChange={(v) => update('organizationSize', v)} />
            <Field label="Email" value={data?.organizationEmail || ''}
              onChange={(v) => update('organizationEmail', v)} icon={Mail} />
            <Field label="Phone" value={data?.organizationPhone || ''}
              onChange={(v) => update('organizationPhone', v)} icon={Phone} />
            <div className="md:col-span-2">
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Address</label>
              <input value={data?.organizationAddress || ''}
                onChange={(e) => update('organizationAddress', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none" />
            </div>
            <div className="md:col-span-2">
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Business Description</label>
              <textarea rows={3} value={data?.businessDescription || ''}
                onChange={(e) => update('businessDescription', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none resize-none" />
            </div>
          </div>
        </Section>

        {/* Channel Connections */}
        <Section title="Channel Connections">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {Object.entries(connections).map(([channel, status]: [string, any]) => (
              <div key={channel} className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                <div className="flex items-center gap-3">
                  {status?.connected
                    ? <CheckCircle2 className="h-5 w-5 text-green-500" />
                    : <XCircle className="h-5 w-5 text-gray-300" />
                  }
                  <div>
                    <p className="text-sm font-medium text-[var(--color-ink)] capitalize">{channel}</p>
                    {status?.lastConnectedAt && (
                      <p className="text-xs text-[var(--color-ink-muted)]">
                        Last: {new Date(status.lastConnectedAt).toLocaleDateString()}
                      </p>
                    )}
                  </div>
                </div>
                <span className={`text-xs px-2 py-1 rounded-full font-medium ${
                  status?.connected
                    ? 'bg-green-100 text-green-700'
                    : 'bg-gray-100 text-gray-500'
                }`}>
                  {status?.connected ? 'Connected' : 'Not configured'}
                </span>
              </div>
            ))}
          </div>
        </Section>

        {/* AI Agent Status */}
        <Section title="AI Agent">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="p-4 border border-gray-200 rounded-lg">
              <p className="text-xs text-[var(--color-ink-muted)] mb-1">Status</p>
              <p className={`text-sm font-semibold ${data?.aiAgentEnabled ? 'text-green-600' : 'text-gray-400'}`}>
                {data?.aiAgentEnabled ? 'Enabled' : 'Disabled'}
              </p>
            </div>
            <div className="p-4 border border-gray-200 rounded-lg">
              <p className="text-xs text-[var(--color-ink-muted)] mb-1">Tone</p>
              <p className="text-sm font-semibold text-[var(--color-ink)] capitalize">
                {data?.aiAgentTone || 'professional'}
              </p>
            </div>
            <div className="p-4 border border-gray-200 rounded-lg">
              <p className="text-xs text-[var(--color-ink-muted)] mb-1">Auto-reply Confidence</p>
              <p className="text-sm font-semibold text-[var(--color-ink)]">
                {data?.aiAgentAutoReplyConfidence ?? 0.85}
              </p>
            </div>
          </div>
        </Section>

        <div className="flex justify-end">
          <button type="submit" disabled={saving}
            className="px-4 py-2 bg-[var(--color-accent)] text-[var(--color-on-accent)] text-sm font-medium rounded-lg hover:opacity-90 transition disabled:opacity-50 flex items-center gap-2">
            {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
            <span>Save Changes</span>
          </button>
        </div>
      </form>
    </div>
  )
}

function StatCard({ icon: Icon, label, value }: { icon: any; label: string; value: string }) {
  return (
    <div className="p-4 border border-gray-200 rounded-lg bg-white">
      <div className="flex items-center gap-3">
        <div className="h-10 w-10 rounded-lg bg-[var(--color-accent-soft)] flex items-center justify-center">
          <Icon className="h-5 w-5 text-[var(--color-accent)]" />
        </div>
        <div>
          <p className="text-xs text-[var(--color-ink-muted)]">{label}</p>
          <p className="text-lg font-bold text-[var(--color-ink)]">{value}</p>
        </div>
      </div>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="border border-gray-200 rounded-lg bg-white">
      <div className="px-4 py-3 border-b border-gray-200">
        <h2 className="text-sm font-semibold text-[var(--color-ink)]">{title}</h2>
      </div>
      <div className="p-4">{children}</div>
    </div>
  )
}

function Field({ label, value, onChange, icon: Icon }: {
  label: string; value: string; onChange: (v: string) => void; icon?: any
}) {
  return (
    <div>
      <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">{label}</label>
      <div className="relative">
        {Icon && <Icon className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />}
        <input
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className={`w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none ${Icon ? 'pl-10' : ''}`}
        />
      </div>
    </div>
  )
}
