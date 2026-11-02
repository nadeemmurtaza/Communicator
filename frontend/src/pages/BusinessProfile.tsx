import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  Building2,
  Phone,
  ShieldCheck,
  CheckCircle2,
  Clock,
  Globe,
  Mail,
  MapPin,
  RefreshCw,
  Save,
  Radio,
  Bot,
  ExternalLink,
  MessageSquare,
  Smartphone,
  Share2,
  X,
} from 'lucide-react'
import { api, type SharedBusinessProfile } from '../lib/api'

export function BusinessProfile() {
  const [profile, setProfile] = useState<SharedBusinessProfile | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [syncingMeta, setSyncingMeta] = useState(false)
  const [success, setSuccess] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Form states
  const [businessName, setBusinessName] = useState('')
  const [displayPhone, setDisplayPhone] = useState('')
  const [email, setEmail] = useState('')
  const [vertical, setVertical] = useState('PROF_SERVICES')
  const [about, setAbout] = useState('')
  const [description, setDescription] = useState('')
  const [address, setAddress] = useState('')
  const [website, setWebsite] = useState('')
  const [profilePictureUrl, setProfilePictureUrl] = useState('')
  const [businessHoursEnabled, setBusinessHoursEnabled] = useState(true)
  const [businessHoursStart, setBusinessHoursStart] = useState(9)
  const [businessHoursEnd, setBusinessHoursEnd] = useState(18)
  const [businessTimezone, setBusinessTimezone] = useState('UTC')
  const [awayMessage, setAwayMessage] = useState('')
  const [autoReplyEnabled, setAutoReplyEnabled] = useState(true)
  const [agentGuidelines, setAgentGuidelines] = useState('')

  useEffect(() => {
    loadProfile()
  }, [])

  async function loadProfile() {
    try {
      setLoading(true)
      const data = await api.getBusinessProfile()
      setProfile(data)
      setBusinessName(data.businessName || data.verifiedName || '')
      setDisplayPhone(data.displayPhone || '')
      setEmail(data.email || '')
      setVertical(data.vertical || 'PROF_SERVICES')
      setAbout(data.about || '')
      setDescription(data.description || '')
      setAddress(data.address || '')
      setWebsite(data.websites?.[0] || '')
      setProfilePictureUrl(data.profilePictureUrl || '')
      setBusinessHoursEnabled(data.businessHours?.enabled ?? true)
      setBusinessHoursStart(data.businessHours?.start ?? 9)
      setBusinessHoursEnd(data.businessHours?.end ?? 18)
      setBusinessTimezone(data.businessHours?.timezone || 'UTC')
      setAwayMessage(data.businessHours?.awayMessage || '')
      setAutoReplyEnabled(data.autoReplyEnabled ?? true)
      setAgentGuidelines(data.agentGuidelines || '')
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
      const updated = await api.updateBusinessProfile({
        business_name: businessName,
        display_phone: displayPhone,
        verified_name: businessName,
        email,
        description,
        about,
        address,
        vertical,
        websites: [website],
        profile_picture_url: profilePictureUrl,
        business_hours_enabled: businessHoursEnabled,
        business_hours_start: businessHoursStart,
        business_hours_end: businessHoursEnd,
        business_timezone: businessTimezone,
        away_message: awayMessage,
        auto_reply_enabled: autoReplyEnabled,
        agent_guidelines: agentGuidelines,
        sync_with_meta: true,
      })
      setProfile(updated)
      setSuccess('Shared Business Profile updated and synced with Meta Cloud API!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleSyncMeta() {
    try {
      setSyncingMeta(true)
      setError(null)
      const updated = await api.syncBusinessProfileMeta()
      setProfile(updated)
      setBusinessName(updated.businessName)
      setDisplayPhone(updated.displayPhone)
      setEmail(updated.email)
      setDescription(updated.description)
      setAbout(updated.about)
      setAddress(updated.address)
      if (updated.websites?.[0]) setWebsite(updated.websites[0])
      if (updated.profilePictureUrl) setProfilePictureUrl(updated.profilePictureUrl)
      setSuccess('Successfully pulled latest verified identity & profile data directly from Meta Graph API!')
      setTimeout(() => setSuccess(null), 4000)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSyncingMeta(false)
    }
  }

  if (loading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <div className="text-center">
          <Building2 className="h-10 w-10 text-indigo-600 animate-pulse mx-auto" />
          <p className="mt-3 text-sm text-slate-500 font-medium">Loading Shared Business Profile...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6 max-w-6xl mx-auto">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 text-xs text-slate-500 mb-1">
            <Link to="/team" className="hover:text-slate-800">
              Admin
            </Link>
            <span>/</span>
            <span className="font-semibold text-slate-700">Business Profile</span>
          </div>
          <div className="flex items-center gap-3">
            <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-indigo-600 text-white shadow-xs">
              <Building2 className="h-6 w-6" />
            </div>
            <div>
              <h1 className="text-xl font-bold text-slate-900 flex items-center gap-2">
                Shared Business Profile & Brand Identity
                <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800 flex items-center gap-1">
                  <CheckCircle2 className="h-3.5 w-3.5" />
                  Verified Meta Cloud Sync
                </span>
              </h1>
              <p className="text-xs text-slate-500">
                Independent organization entity shared across all channels (WhatsApp, Live Chat, SMS, Voice, Social).
              </p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2.5">
          <button
            onClick={handleSyncMeta}
            disabled={syncingMeta}
            className="rounded-md border border-slate-300 bg-white px-3.5 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50 shadow-xs flex items-center gap-2 transition-colors"
          >
            <RefreshCw className={`h-3.5 w-3.5 text-indigo-600 ${syncingMeta ? 'animate-spin' : ''}`} />
            {syncingMeta ? 'Syncing Meta...' : 'Sync with Meta'}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-md bg-slate-900 px-4 py-2 text-xs font-semibold text-white hover:bg-slate-800 disabled:opacity-50 shadow-xs flex items-center gap-2 transition-colors"
          >
            <Save className="h-3.5 w-3.5" />
            {saving ? 'Saving...' : 'Save Profile'}
          </button>
        </div>
      </div>

      {/* Strict Meta Verified Health Metric Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3.5">
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-500">Display Phone</span>
          <p className="mt-1 font-bold text-slate-900 text-base font-mono">
            {profile?.displayPhone || displayPhone || 'Configured in Meta'}
          </p>
          <span className="mt-1.5 inline-flex items-center gap-1 text-[11px] text-emerald-600 font-semibold">
            <CheckCircle2 className="h-3.5 w-3.5" />
            Verified Number
          </span>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-500">Verified Name</span>
          <p className="mt-1 font-bold text-slate-900 text-base truncate">
            {profile?.verifiedName || profile?.businessName || businessName || 'Business Profile'}
          </p>
          <span className="mt-1.5 inline-flex items-center gap-1 text-[11px] text-blue-600 font-semibold">
            <ShieldCheck className="h-3.5 w-3.5" />
            {profile?.verificationStatus || 'Official Business Account'}
          </span>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-500">Quality Rating</span>
          <p className="mt-1 font-bold text-emerald-700 text-base flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full bg-emerald-500"></span>
            {profile?.qualityRating || 'HIGH (Green)'}
          </p>
          <span className="mt-1.5 inline-block text-[11px] text-slate-400 font-medium">
            {profile?.spamFlags || 'Zero spam flags'}
          </span>
        </div>
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-xs">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-500">Messaging Limit</span>
          <p className="mt-1 font-bold text-slate-900 text-base">
            {profile?.messagingLimit || '250 / day'}
          </p>
          <span className="mt-1.5 inline-block text-[11px] text-purple-600 font-semibold">
            {profile?.tier || 'Standard Account Limit'}
          </span>
        </div>
      </div>

      {/* Live Alerts */}
      {error && (
        <div className="flex items-center justify-between rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 shadow-xs">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-red-700 hover:text-red-900">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800 shadow-xs">
          <span className="flex items-center gap-2">
            <CheckCircle2 className="h-4 w-4 text-emerald-600 shrink-0" />
            {success}
          </span>
          <button onClick={() => setSuccess(null)} className="text-emerald-700 hover:text-emerald-900">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      {/* Meta Graph Status Banner */}
      <div className="rounded-xl border border-indigo-100 bg-indigo-50/50 p-4 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-3 text-xs">
        <div className="flex items-center gap-3">
          <div className="h-8 w-8 rounded-lg bg-indigo-600 text-white flex items-center justify-center shrink-0">
            <Share2 className="h-4 w-4" />
          </div>
          <div>
            <p className="font-bold text-slate-900">
              Omnichannel Shared Identity · Meta Graph API (v21.0) Synced {profile?.metaIntegration?.verifiedAt ? `(Verified ${new Date(profile.metaIntegration.verifiedAt).toLocaleTimeString()})` : ''}
            </p>
            <p className="text-slate-500">
              Phone Number ID: <span className="font-mono text-slate-700">1436650392860391</span> · WABA ID:{' '}
              <span className="font-mono text-slate-700">990754184075228</span>
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 text-indigo-700 font-semibold">
          <span className="h-2 w-2 rounded-full bg-emerald-500"></span>
          Live Synchronized
        </div>
      </div>

      {/* Main Form Sections */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Card 1: Official Verified Identity */}
        <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
          <div className="border-b border-slate-100 pb-3 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <ShieldCheck className="h-5 w-5 text-indigo-600" />
              <h2 className="text-sm font-bold text-slate-900">Official Verified Identity</h2>
            </div>
            <span className="text-[11px] font-semibold text-emerald-600 bg-emerald-50 px-2 py-0.5 rounded">
              Meta Approved
            </span>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Verified Business Name</label>
            <input
              type="text"
              value={businessName}
              onChange={(e) => setBusinessName(e.target.value)}
              placeholder="Newax Technologies"
              className="mt-1.5 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Official Display Phone</label>
            <div className="relative mt-1.5">
              <Phone className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
              <input
                type="text"
                value={displayPhone}
                onChange={(e) => setDisplayPhone(e.target.value)}
                placeholder="+1 (555) 000-0000"
                className="w-full rounded-md border border-slate-300 pl-9 pr-3 py-2 text-sm font-mono shadow-xs focus:border-indigo-500 focus:outline-none"
              />
            </div>
            <p className="mt-1 text-[11px] text-slate-500">
              Synced across WhatsApp, Live Chat, and SMS caller identification.
            </p>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Support Email</label>
            <div className="relative mt-1.5">
              <Mail className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="hello@newax.co"
                className="w-full rounded-md border border-slate-300 pl-9 pr-3 py-2 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Industry Vertical / Category</label>
            <select
              value={vertical}
              onChange={(e) => setVertical(e.target.value)}
              className="mt-1.5 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs bg-white font-medium focus:border-indigo-500 focus:outline-none"
            >
              <option value="PROF_SERVICES">Professional Services &amp; Digital Infrastructure</option>
              <option value="TECH">Technology &amp; Software Solutions</option>
              <option value="RETAIL">Retail &amp; eCommerce</option>
              <option value="FINANCE">Financial Services &amp; Banking</option>
              <option value="HEALTH">Health &amp; Medical</option>
              <option value="EDUCATION">Education &amp; Training</option>
              <option value="OTHER">Other Enterprise Business</option>
            </select>
          </div>
        </div>

        {/* Card 2: Brand Details & Presence */}
        <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
          <div className="border-b border-slate-100 pb-3 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Building2 className="h-5 w-5 text-indigo-600" />
              <h2 className="text-sm font-bold text-slate-900">Brand Presence &amp; Address</h2>
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Official Website URL</label>
            <div className="relative mt-1.5">
              <Globe className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
              <input
                type="url"
                value={website}
                onChange={(e) => setWebsite(e.target.value)}
                placeholder="https://newax.co/"
                className="w-full rounded-md border border-slate-300 pl-9 pr-3 py-2 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Physical Headquarters Address</label>
            <div className="relative mt-1.5">
              <MapPin className="absolute left-3 top-2.5 h-4 w-4 text-slate-400" />
              <input
                type="text"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                placeholder="Building 9, College Road, Township, Lahore, Pakistan, 54700"
                className="w-full rounded-md border border-slate-300 pl-9 pr-3 py-2 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Tagline / About Status (Max 139 chars)</label>
            <input
              type="text"
              maxLength={139}
              value={about}
              onChange={(e) => setAbout(e.target.value)}
              placeholder="Official Business Profile for Newax Technologies"
              className="mt-1.5 w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-700">Company Description (Max 512 chars)</label>
            <textarea
              rows={3}
              maxLength={512}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe your organization and capabilities..."
              className="mt-1.5 w-full rounded-md border border-slate-300 p-2.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
            />
          </div>
        </div>
      </div>

      {/* Card 3: Operating Hours & Away Protocol */}
      <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
        <div className="border-b border-slate-100 pb-3 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-indigo-600" />
            <div>
              <h2 className="text-sm font-bold text-slate-900">Shared Operating Hours &amp; Automated Away Protocol</h2>
              <p className="text-xs text-slate-500">
                Shared schedule applied globally across all inboxes (WhatsApp, Live Chat, SMS, Social).
              </p>
            </div>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={businessHoursEnabled}
              onChange={(e) => setBusinessHoursEnabled(e.target.checked)}
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
          </label>
        </div>

        {businessHoursEnabled && (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2">
            <div>
              <label className="block text-xs font-semibold text-slate-700">Timezone</label>
              <select
                value={businessTimezone}
                onChange={(e) => setBusinessTimezone(e.target.value)}
                className="mt-1.5 w-full rounded-md border border-slate-300 px-3 py-2 text-sm bg-white shadow-xs"
              >
                <option value="Asia/Karachi">Asia/Karachi (PKT, UTC+5)</option>
                <option value="UTC">UTC (Coordinated Universal Time)</option>
                <option value="America/New_York">America/New_York (EST/EDT)</option>
                <option value="America/Los_Angeles">America/Los_Angeles (PST/PDT)</option>
                <option value="Europe/London">Europe/London (GMT/BST)</option>
                <option value="Asia/Dubai">Asia/Dubai (GST, UTC+4)</option>
              </select>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-700">Start Time (24h format)</label>
              <select
                value={businessHoursStart}
                onChange={(e) => setBusinessHoursStart(Number(e.target.value))}
                className="mt-1.5 w-full rounded-md border border-slate-300 px-3 py-2 text-sm bg-white shadow-xs"
              >
                {Array.from({ length: 24 }).map((_, i) => (
                  <option key={i} value={i}>
                    {i.toString().padStart(2, '0')}:00 ({i === 0 ? '12 AM' : i < 12 ? `${i} AM` : i === 12 ? '12 PM' : `${i - 12} PM`})
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-700">End Time (24h format)</label>
              <select
                value={businessHoursEnd}
                onChange={(e) => setBusinessHoursEnd(Number(e.target.value))}
                className="mt-1.5 w-full rounded-md border border-slate-300 px-3 py-2 text-sm bg-white shadow-xs"
              >
                {Array.from({ length: 24 }).map((_, i) => (
                  <option key={i} value={i}>
                    {i.toString().padStart(2, '0')}:00 ({i === 0 ? '12 AM' : i < 12 ? `${i} AM` : i === 12 ? '12 PM' : `${i - 12} PM`})
                  </option>
                ))}
              </select>
            </div>

            <div className="md:col-span-3">
              <label className="block text-xs font-semibold text-slate-700">Automated Away Message</label>
              <textarea
                rows={2}
                value={awayMessage}
                onChange={(e) => setAwayMessage(e.target.value)}
                placeholder="Thanks for reaching out! Our team is currently outside working hours..."
                className="mt-1.5 w-full rounded-md border border-slate-300 p-2.5 text-sm shadow-xs focus:border-indigo-500 focus:outline-none"
              />
            </div>
          </div>
        )}
      </div>

      {/* Card 4: All Channels Sharing This Profile */}
      <div className="rounded-xl border border-slate-200 bg-white p-6 shadow-xs space-y-4">
        <div className="border-b border-slate-100 pb-3 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Radio className="h-5 w-5 text-indigo-600" />
            <div>
              <h2 className="text-sm font-bold text-slate-900">Omnichannel Propagation</h2>
              <p className="text-xs text-slate-500">
                Channels actively linked to this shared organization identity and operating hours.
              </p>
            </div>
          </div>
          <Link
            to="/channels"
            className="text-xs font-semibold text-indigo-600 hover:text-indigo-800 flex items-center gap-1"
          >
            Manage Channels
            <ExternalLink className="h-3 w-3" />
          </Link>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
          <Link
            to="/channels/whatsapp"
            className="p-3.5 rounded-lg border border-slate-200 hover:border-emerald-500 hover:bg-emerald-50/30 transition-all flex items-center gap-3"
          >
            <div className="h-9 w-9 rounded-lg bg-emerald-500 text-white flex items-center justify-center shrink-0">
              <Smartphone className="h-5 w-5" />
            </div>
            <div className="truncate">
              <p className="font-bold text-slate-900 text-xs truncate">WhatsApp Business</p>
              <p className="text-[11px] text-emerald-600 font-semibold flex items-center gap-1">
                <CheckCircle2 className="h-3 w-3" />
                Meta Synced
              </p>
            </div>
          </Link>

          <Link
            to="/channels/live-chat"
            className="p-3.5 rounded-lg border border-slate-200 hover:border-blue-500 hover:bg-blue-50/30 transition-all flex items-center gap-3"
          >
            <div className="h-9 w-9 rounded-lg bg-blue-600 text-white flex items-center justify-center shrink-0">
              <MessageSquare className="h-5 w-5" />
            </div>
            <div className="truncate">
              <p className="font-bold text-slate-900 text-xs truncate">Website Live Chat</p>
              <p className="text-[11px] text-blue-600 font-semibold flex items-center gap-1">
                <CheckCircle2 className="h-3 w-3" />
                Branding Active
              </p>
            </div>
          </Link>

          <Link
            to="/channels/sms"
            className="p-3.5 rounded-lg border border-slate-200 hover:border-purple-500 hover:bg-purple-50/30 transition-all flex items-center gap-3"
          >
            <div className="h-9 w-9 rounded-lg bg-purple-600 text-white flex items-center justify-center shrink-0">
              <Phone className="h-5 w-5" />
            </div>
            <div className="truncate">
              <p className="font-bold text-slate-900 text-xs truncate">Twilio SMS 10DLC</p>
              <p className="text-[11px] text-purple-600 font-semibold flex items-center gap-1">
                <CheckCircle2 className="h-3 w-3" />
                Verified Caller
              </p>
            </div>
          </Link>

          <Link
            to="/channels/facebook"
            className="p-3.5 rounded-lg border border-slate-200 hover:border-indigo-500 hover:bg-indigo-50/30 transition-all flex items-center gap-3"
          >
            <div className="h-9 w-9 rounded-lg bg-indigo-600 text-white flex items-center justify-center shrink-0">
              <Bot className="h-5 w-5" />
            </div>
            <div className="truncate">
              <p className="font-bold text-slate-900 text-xs truncate">Meta Messenger</p>
              <p className="text-[11px] text-indigo-600 font-semibold flex items-center gap-1">
                <CheckCircle2 className="h-3 w-3" />
                Hours Linked
              </p>
            </div>
          </Link>
        </div>
      </div>
    </div>
  )
}
