import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type AudiencePreview,
  type Broadcast,
  type CampaignAnalytics,
  type PricingEstimate,
} from '../lib/api'

export function Broadcasts() {
  const [campaigns, setCampaigns] = useState<Broadcast[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Creation Wizard Modal
  const [showWizard, setShowWizard] = useState(false)
  const [name, setName] = useState('Spring sale early access')
  const [selectedTags, setSelectedTags] = useState<string[]>(['vip-customers'])
  const [tagInput, setTagInput] = useState('')
  const [audiencePreview, setAudiencePreview] = useState<AudiencePreview | null>(null)
  const [templateName, setTemplateName] = useState('spring_sale_offer')
  const [customerNameSample, setCustomerNameSample] = useState('Maya')
  const [discountParam, setDiscountParam] = useState('20%')
  const [scheduleTime, setScheduleTime] = useState('2026-03-12T10:00')
  const [sendImmediately, setSendImmediately] = useState(false)
  const [creating, setCreating] = useState(false)

  // Analytics View Modal
  const [selectedCampaignId, setSelectedCampaignId] = useState<string | null>(null)
  const [analytics, setAnalytics] = useState<CampaignAnalytics | null>(null)
  const [loadingAnalytics, setLoadingAnalytics] = useState(false)

  // Pricing Calculator Modal
  const [showPricingCalculator, setShowPricingCalculator] = useState(false)
  const [calcRegion, setCalcRegion] = useState('US')
  const [calcCategory, setCalcCategory] = useState('marketing')
  const [calcVolume, setCalcVolume] = useState(1000)
  const [pricingEstimate, setPricingEstimate] = useState<PricingEstimate | null>(null)

  // Dispatching state
  const [dispatchingId, setDispatchingId] = useState<string | null>(null)

  function loadCampaigns() {
    api
      .listBroadcasts()
      .then(setCampaigns)
      .catch((e) => setError(e.message))
  }

  useEffect(loadCampaigns, [])

  // Auto-fetch audience preview when tags change in wizard
  useEffect(() => {
    if (!showWizard || selectedTags.length === 0) return
    api
      .getAudiencePreview(selectedTags.join(','))
      .then(setAudiencePreview)
      .catch(() => {})
  }, [showWizard, selectedTags])

  // Recalculate WhatsApp pricing estimate
  useEffect(() => {
    if (!showPricingCalculator) return
    api
      .getWhatsAppPricing(calcRegion, calcCategory, calcVolume)
      .then(setPricingEstimate)
      .catch(() => {})
  }, [showPricingCalculator, calcRegion, calcCategory, calcVolume])

  async function handleCreateCampaign(e: React.FormEvent) {
    e.preventDefault()
    setCreating(true)
    setError(null)

    const previewText = `Hi ${customerNameSample}, our spring sale starts Friday. As a VIP customer you get early access and ${discountParam} off until Sunday.`

    try {
      await api.createBroadcast({
        name,
        template_name: templateName,
        template_language: 'en_US',
        body_preview: previewText,
        template_params: {
          customer_name: customerNameSample,
          discount: discountParam,
        },
        tag_name: selectedTags.join(','),
        scheduled_at: sendImmediately ? null : new Date(scheduleTime).toISOString(),
      })
      setShowWizard(false)
      setSuccess(`Campaign "${name}" scheduled successfully!`)
      loadCampaigns()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setCreating(false)
    }
  }

  async function openAnalytics(id: string) {
    setSelectedCampaignId(id)
    setLoadingAnalytics(true)
    try {
      const data = await api.getCampaignAnalytics(id)
      setAnalytics(data)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoadingAnalytics(false)
    }
  }

  async function handleDispatch(id: string) {
    setDispatchingId(id)
    setError(null)
    try {
      await api.dispatchBroadcast(id)
      setSuccess('Campaign dispatched successfully!')
      loadCampaigns()
      if (selectedCampaignId === id) {
        openAnalytics(id)
      }
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setDispatchingId(null)
    }
  }

  async function handleSeedDefaults() {
    setError(null)
    try {
      const res = await api.seedDefaultCampaigns()
      if (res.status === 'seeded') {
        setSuccess('Pre-built Chatwoot WhatsApp example campaigns installed!')
      } else {
        setSuccess('Example campaigns already active.')
      }
      loadCampaigns()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleProcessScheduled() {
    try {
      setError(null)
      const res = await api.processScheduledCampaigns()
      setSuccess(`Checked scheduled campaigns: ${res.processed} dispatched.`)
      loadCampaigns()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function addTag() {
    const val = tagInput.trim().toLowerCase()
    if (val && !selectedTags.includes(val)) {
      setSelectedTags([...selectedTags, val])
      setTagInput('')
    }
  }

  function removeTag(tag: string) {
    setSelectedTags(selectedTags.filter((t) => t !== tag))
  }

  return (
    <div className="space-y-6">
      {/* Top Banner & Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-[var(--color-ink)]">WhatsApp campaigns</h1>
            <span className="rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-semibold text-emerald-800">
              Meta Cloud API
            </span>
          </div>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Send WhatsApp campaigns from your shared inbox. Choose who gets the message, pick a time,
            and answer everyone who writes back with your team and Captain.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <button
            onClick={() => setShowPricingCalculator(true)}
            className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-xs"
          >
            💰 Pricing Calculator
          </button>

          <button
            onClick={handleSeedDefaults}
            className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs font-semibold text-emerald-800 hover:bg-emerald-100 shadow-xs"
          >
            ⚡ Load Example Data
          </button>

          <button
            onClick={handleProcessScheduled}
            className="rounded-md border border-blue-200 bg-blue-50 px-3 py-1.5 text-xs font-semibold text-blue-800 hover:bg-blue-100 shadow-xs"
            title="Checks and sends any campaigns whose scheduled time has arrived"
          >
            🕒 Process Due
          </button>

          <button
            onClick={() => setShowWizard(true)}
            className="rounded-md bg-[var(--color-accent)] px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow-xs"
          >
            + Create campaign
          </button>
        </div>
      </div>

      {/* Notifications */}
      {error && (
        <div className="flex items-center justify-between rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="font-bold">
            ×
          </button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="font-bold">
            ×
          </button>
        </div>
      )}

      {/* Campaigns List */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs overflow-hidden">
        <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/50 px-5 py-3 flex items-center justify-between">
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
            Active & Past Campaigns
          </span>
          <span className="text-xs text-[var(--color-ink-faint)]">
            Works with your Meta WhatsApp Business number
          </span>
        </div>

        {campaigns === null ? (
          <div className="py-12 text-center text-sm text-[var(--color-ink-muted)]">Loading campaigns...</div>
        ) : campaigns.length === 0 ? (
          <div className="py-12 text-center">
            <span className="text-3xl">📢</span>
            <h3 className="mt-2 text-sm font-semibold text-[var(--color-ink)]">No campaigns yet</h3>
            <p className="mt-1 text-xs text-[var(--color-ink-muted)] max-w-sm mx-auto">
              Create your first WhatsApp campaign in 3 easy steps: audience, message, and schedule.
            </p>
            <div className="mt-4 flex justify-center gap-2">
              <button
                onClick={handleSeedDefaults}
                className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
              >
                Load Chatwoot Demo Campaigns
              </button>
              <button
                onClick={() => setShowWizard(true)}
                className="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)]"
              >
                Create campaign
              </button>
            </div>
          </div>
        ) : (
          <div className="divide-y divide-slate-100">
            {campaigns.map((c) => (
              <div
                key={c.id}
                className="p-5 flex flex-col md:flex-row md:items-center md:justify-between gap-4 hover:bg-[var(--color-page)]/60 transition-colors"
              >
                <div>
                  <div className="flex items-center gap-2.5">
                    <h3 className="font-bold text-[var(--color-ink)] text-sm">{c.name}</h3>
                    {c.status === 'scheduled' && (
                      <span className="rounded-full bg-blue-100 px-2.5 py-0.5 text-[11px] font-semibold text-blue-800">
                        Scheduled
                      </span>
                    )}
                    {c.status === 'sending' && (
                      <span className="rounded-full bg-amber-100 px-2.5 py-0.5 text-[11px] font-semibold text-amber-800 animate-pulse">
                        Processing
                      </span>
                    )}
                    {c.status === 'completed' && (
                      <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-[11px] font-semibold text-emerald-800">
                        Completed
                      </span>
                    )}
                    {c.status === 'draft' && (
                      <span className="rounded-full bg-slate-100 px-2.5 py-0.5 text-[11px] font-semibold text-[var(--color-ink)]">
                        Draft
                      </span>
                    )}
                  </div>

                  <p className="mt-1 text-xs text-[var(--color-ink-muted)] flex items-center gap-2">
                    <span>WhatsApp Support</span>
                    <span>·</span>
                    <span>
                      {c.scheduledAt
                        ? new Date(c.scheduledAt).toLocaleDateString('en-US', {
                            month: 'short',
                            day: 'numeric',
                            hour: 'numeric',
                            minute: '2-digit',
                          })
                        : new Date(c.createdAt).toLocaleDateString('en-US', {
                            month: 'short',
                            day: 'numeric',
                          })}
                    </span>
                    <span>·</span>
                    <span className="font-medium text-[var(--color-ink-muted)]">Tags: {c.tagName}</span>
                  </p>

                  {/* Delivery Stats Bar */}
                  <div className="mt-2.5 flex items-center gap-4 text-xs text-[var(--color-ink-muted)]">
                    <span>
                      Audience: <strong>{c.audienceCount || c.sentCount}</strong>
                    </span>
                    <span>
                      Delivered: <strong>{c.deliveredCount}</strong>
                    </span>
                    <span>
                      Read: <strong>{c.readCount}</strong>
                    </span>
                    {c.failedCount > 0 && (
                      <span className="text-rose-600">
                        Failed: <strong>{c.failedCount}</strong>
                      </span>
                    )}
                  </div>
                </div>

                <div className="flex items-center gap-2 self-start md:self-center">
                  <button
                    onClick={() => openAnalytics(c.id)}
                    className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-xs"
                  >
                    View analytics
                  </button>

                  {c.status !== 'completed' && c.status !== 'sending' && (
                    <button
                      onClick={() => handleDispatch(c.id)}
                      disabled={dispatchingId === c.id}
                      className="rounded-md bg-emerald-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-emerald-700 disabled:opacity-50 shadow-xs"
                    >
                      {dispatchingId === c.id ? 'Sending...' : 'Send Now'}
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 3-Step Campaign Creation Wizard ("Choose who, what, and when on one page") */}
      {showWizard && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-xs overflow-y-auto">
          <div className="w-full max-w-3xl rounded-2xl bg-[var(--color-surface)] shadow-2xl border border-[var(--color-border)] overflow-hidden my-8">
            <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/70 px-6 py-4 flex items-center justify-between">
              <div>
                <h2 className="text-base font-bold text-[var(--color-ink)]">Send a campaign in three steps</h2>
                <p className="text-xs text-[var(--color-ink-muted)]">Choose who, what, and when on one page.</p>
              </div>
              <button
                onClick={() => setShowWizard(false)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink)] text-xl font-bold"
              >
                ×
              </button>
            </div>

            <form onSubmit={handleCreateCampaign} className="p-6 space-y-6">
              {/* Campaign Name */}
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] mb-1">
                  Campaign Name
                </label>
                <input
                  required
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. Spring sale early access"
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm shadow-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              {/* Step 1: Audience */}
              <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)]/40 p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-[11px] font-bold text-white">
                      1
                    </span>
                    <h3 className="text-sm font-bold text-[var(--color-ink)]">Audience: Choose who gets it</h3>
                  </div>
                  <span className="text-xs font-semibold text-[var(--color-ink-muted)]">
                    Daily limit: 1,000 customers
                  </span>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Pick one or more labels, such as VIP customers or newsletter subscribers. You see
                  how many people will get the message before you send it, and nobody gets it twice.
                </p>

                {/* Tags Picker */}
                <div className="flex flex-wrap items-center gap-2">
                  {selectedTags.map((t) => (
                    <span
                      key={t}
                      className="inline-flex items-center gap-1 rounded bg-blue-100 px-2.5 py-1 text-xs font-semibold text-blue-800"
                    >
                      <span>🏷️ {t}</span>
                      <button
                        type="button"
                        onClick={() => removeTag(t)}
                        className="ml-1 text-blue-600 hover:text-blue-900 font-bold"
                      >
                        ×
                      </button>
                    </span>
                  ))}

                  <div className="flex items-center gap-1">
                    <input
                      type="text"
                      placeholder="Add tag (e.g. newsletter)"
                      value={tagInput}
                      onChange={(e) => setTagInput(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          e.preventDefault()
                          addTag()
                        }
                      }}
                      className="rounded border border-slate-300 px-2 py-1 text-xs shadow-xs"
                    />
                    <button
                      type="button"
                      onClick={addTag}
                      className="rounded bg-slate-200 px-2 py-1 text-xs font-semibold text-[var(--color-ink)] hover:bg-slate-300"
                    >
                      + Add
                    </button>
                  </div>
                </div>

                {/* Audience Count Live Badge */}
                <div className="mt-2 flex items-center justify-between rounded-lg bg-[var(--color-surface)] p-3 border border-[var(--color-border)] text-xs">
                  <div className="flex items-center gap-2">
                    <span className="h-2 w-2 rounded-full bg-emerald-500" />
                    <span className="font-bold text-[var(--color-ink)]">
                      {audiencePreview ? `${audiencePreview.totalSelected} contacts selected` : 'Calculating audience...'}
                    </span>
                  </div>
                  <span className="text-[var(--color-ink-muted)]">Deduplicated across all tags</span>
                </div>
              </div>

              {/* Step 2: Message */}
              <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)]/40 p-4 space-y-3">
                <div className="flex items-center gap-2">
                  <span className="flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-[11px] font-bold text-white">
                    2
                  </span>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Message: Write it once, make it personal
                  </h3>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Start from a message WhatsApp has already approved for your business. Each
                  customer's name and other details are filled in for you, and you see the message the
                  way they will.
                </p>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-3">
                    <div>
                      <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">
                        Approved Template
                      </label>
                      <select
                        value={templateName}
                        onChange={(e) => setTemplateName(e.target.value)}
                        className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium"
                      >
                        <option value="spring_sale_offer">Spring sale offer (Approved)</option>
                        <option value="store_launch_announcement">New store opening (Approved)</option>
                        <option value="loyalty_rewards_update">Loyalty points reminder (Approved)</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">
                        Variable {'{{1}}'}: Customer Name
                      </label>
                      <input
                        value={customerNameSample}
                        onChange={(e) => setCustomerNameSample(e.target.value)}
                        placeholder="Maya"
                        className="w-full rounded-md border border-slate-300 px-3 py-1 text-xs"
                      />
                    </div>

                    <div>
                      <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">
                        Variable {'{{2}}'}: Discount Percentage
                      </label>
                      <input
                        value={discountParam}
                        onChange={(e) => setDiscountParam(e.target.value)}
                        placeholder="20%"
                        className="w-full rounded-md border border-slate-300 px-3 py-1 text-xs"
                      />
                    </div>
                  </div>

                  {/* WhatsApp Mobile Balloon Preview */}
                  <div className="rounded-lg bg-[#efeae2] p-4 flex flex-col justify-end border border-[#d1d7db]">
                    <span className="text-[10px] font-bold text-[var(--color-ink-muted)] uppercase tracking-wider mb-2">
                      WhatsApp Live Customer Preview
                    </span>
                    <div className="rounded-lg bg-[var(--color-surface)] p-3 shadow-xs max-w-[90%] text-xs text-[var(--color-ink)] relative">
                      <p className="leading-relaxed">
                        Hi <strong>{customerNameSample}</strong>, our spring sale starts Friday. As a
                        VIP customer you get early access and <strong>{discountParam}</strong> off
                        until Sunday.
                      </p>
                      <div className="mt-2 flex items-center justify-end gap-1 text-[10px] text-[var(--color-ink-faint)]">
                        <span>10:00 AM</span>
                        <span className="text-blue-500 font-bold">✓✓</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Step 3: Schedule */}
              <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)]/40 p-4 space-y-3">
                <div className="flex items-center gap-2">
                  <span className="flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-[11px] font-bold text-white">
                    3
                  </span>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">Schedule: Pick the time</h3>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Send at the moment that suits your customers. You can change the audience, the
                  message, or the time until sending starts.
                </p>

                <div className="flex flex-wrap items-center gap-4">
                  <label className="flex items-center gap-2 text-xs font-medium text-[var(--color-ink)] cursor-pointer">
                    <input
                      type="radio"
                      name="schedule_mode"
                      checked={!sendImmediately}
                      onChange={() => setSendImmediately(false)}
                      className="text-blue-600"
                    />
                    <span>Schedule for specific date & time</span>
                  </label>

                  <label className="flex items-center gap-2 text-xs font-medium text-[var(--color-ink)] cursor-pointer">
                    <input
                      type="radio"
                      name="schedule_mode"
                      checked={sendImmediately}
                      onChange={() => setSendImmediately(true)}
                      className="text-blue-600"
                    />
                    <span>Send immediately</span>
                  </label>
                </div>

                {!sendImmediately && (
                  <div className="pt-2">
                    <input
                      type="datetime-local"
                      value={scheduleTime}
                      onChange={(e) => setScheduleTime(e.target.value)}
                      className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium shadow-xs"
                    />
                  </div>
                )}
              </div>

              {/* Form Buttons */}
              <div className="flex items-center justify-end gap-3 pt-4 border-t border-[var(--color-border)]">
                <button
                  type="button"
                  onClick={() => setShowWizard(false)}
                  className="rounded-md border border-slate-300 px-4 py-2 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creating}
                  className="rounded-md bg-blue-600 px-5 py-2 text-xs font-bold text-white hover:bg-blue-700 disabled:opacity-50 shadow-xs"
                >
                  {creating ? 'Saving...' : sendImmediately ? 'Send Campaign Now' : 'Schedule campaign'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Campaign Analytics View Modal */}
      {selectedCampaignId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-xs overflow-y-auto">
          <div className="w-full max-w-4xl rounded-2xl bg-[var(--color-surface)] shadow-2xl border border-[var(--color-border)] overflow-hidden my-8">
            <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/70 px-6 py-4 flex items-center justify-between">
              <div>
                <h2 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
                  <span>Campaign Analytics: {analytics?.campaign.name || 'Campaign'}</span>
                  <span className="rounded-full bg-emerald-100 text-emerald-800 px-2 py-0.5 text-xs font-semibold">
                    Live Data
                  </span>
                </h2>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  See who got your message and who read it. Watch numbers live while the campaign is
                  sending.
                </p>
              </div>
              <button
                onClick={() => {
                  setSelectedCampaignId(null)
                  setAnalytics(null)
                }}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink)] text-xl font-bold"
              >
                ×
              </button>
            </div>

            {loadingAnalytics ? (
              <div className="py-20 text-center text-sm text-[var(--color-ink-muted)]">Loading live analytics...</div>
            ) : analytics ? (
              <div className="p-6 space-y-6">
                {/* 6 Top Metric Cards */}
                <div className="grid grid-cols-2 md:grid-cols-6 gap-3">
                  <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3 shadow-xs">
                    <span className="text-[11px] font-semibold text-[var(--color-ink-faint)] uppercase">Audience</span>
                    <div className="mt-1 text-xl font-bold text-[var(--color-ink)]">
                      {analytics.metrics.audience}
                    </div>
                    <span className="text-[10px] text-[var(--color-ink-muted)]">Target contacts</span>
                  </div>

                  <div className="rounded-xl border border-blue-200 bg-blue-50/40 p-3 shadow-xs">
                    <span className="text-[11px] font-semibold text-blue-700 uppercase">Submitted</span>
                    <div className="mt-1 text-xl font-bold text-blue-900">
                      {analytics.metrics.submitted}
                    </div>
                    <span className="text-[10px] text-blue-600 font-medium">
                      {analytics.metrics.submittedPct}% of audience
                    </span>
                  </div>

                  <div className="rounded-xl border border-emerald-200 bg-emerald-50/40 p-3 shadow-xs">
                    <span className="text-[11px] font-semibold text-emerald-700 uppercase">Delivered</span>
                    <div className="mt-1 text-xl font-bold text-emerald-900">
                      {analytics.metrics.delivered}
                    </div>
                    <span className="text-[10px] text-emerald-600 font-medium">
                      {analytics.metrics.deliveredPct}% of audience
                    </span>
                  </div>

                  <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-accent-soft)]/40 p-3 shadow-xs">
                    <span className="text-[11px] font-semibold text-[var(--color-accent)] uppercase">Read</span>
                    <div className="mt-1 text-xl font-bold text-indigo-900">
                      {analytics.metrics.read}
                    </div>
                    <span className="text-[10px] text-[var(--color-accent)] font-medium">
                      {analytics.metrics.readPct}% of audience
                    </span>
                  </div>

                  <div className="rounded-xl border border-rose-200 bg-rose-50/40 p-3 shadow-xs">
                    <span className="text-[11px] font-semibold text-rose-700 uppercase">Failed</span>
                    <div className="mt-1 text-xl font-bold text-rose-900">
                      {analytics.metrics.failed}
                    </div>
                    <span className="text-[10px] text-rose-600 font-medium">
                      {analytics.metrics.failedPct}% of audience
                    </span>
                  </div>

                  <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)] p-3 shadow-xs">
                    <span className="text-[11px] font-semibold text-[var(--color-ink-muted)] uppercase">Skipped</span>
                    <div className="mt-1 text-xl font-bold text-slate-800">
                      {analytics.metrics.skipped}
                    </div>
                    <span className="text-[10px] text-[var(--color-ink-muted)] font-medium">
                      {analytics.metrics.skippedPct}% of audience
                    </span>
                  </div>
                </div>

                {/* Delivery Breakdown Visual Progress Bar */}
                <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)]/60 p-4">
                  <div className="flex items-center justify-between text-xs font-semibold text-[var(--color-ink)] mb-2">
                    <span>Delivery breakdown</span>
                    <span className="text-emerald-700 font-bold">
                      {analytics.metrics.deliveredPct}% delivered
                    </span>
                  </div>
                  <div className="h-3 w-full rounded-full bg-slate-200 overflow-hidden flex">
                    <div
                      style={{ width: `${analytics.metrics.readPct}%` }}
                      className="bg-[var(--color-accent)] h-full"
                      title={`Read: ${analytics.metrics.readPct}%`}
                    />
                    <div
                      style={{
                        width: `${Math.max(0, analytics.metrics.deliveredPct - analytics.metrics.readPct)}%`,
                      }}
                      className="bg-emerald-500 h-full"
                      title={`Delivered (unread): ${analytics.metrics.deliveredPct - analytics.metrics.readPct}%`}
                    />
                    <div
                      style={{ width: `${analytics.metrics.failedPct}%` }}
                      className="bg-rose-500 h-full"
                      title={`Failed: ${analytics.metrics.failedPct}%`}
                    />
                  </div>
                  <div className="mt-2 flex items-center justify-between text-[11px] text-[var(--color-ink-muted)]">
                    <div className="flex items-center gap-3">
                      <span className="flex items-center gap-1">
                        <span className="h-2 w-2 rounded-full bg-[var(--color-accent)]" /> Read (
                        {analytics.metrics.readPct}%)
                      </span>
                      <span className="flex items-center gap-1">
                        <span className="h-2 w-2 rounded-full bg-emerald-500" /> Delivered (
                        {analytics.metrics.deliveredPct}%)
                      </span>
                      <span className="flex items-center gap-1">
                        <span className="h-2 w-2 rounded-full bg-rose-500" /> Failed (
                        {analytics.metrics.failedPct}%)
                      </span>
                    </div>
                    <span>Total audience: {analytics.metrics.audience} contacts</span>
                  </div>
                </div>

                {/* Per-Recipient Breakdown Table */}
                <div>
                  <h3 className="text-xs font-bold uppercase tracking-wider text-[var(--color-ink-muted)] mb-2">
                    Recipient Activity & Delivery Status
                  </h3>
                  <div className="rounded-xl border border-[var(--color-border)] overflow-hidden bg-[var(--color-surface)]">
                    <div className="max-h-64 overflow-y-auto divide-y divide-slate-100">
                      {analytics.recipients.length === 0 ? (
                        <div className="p-6 text-center text-xs text-[var(--color-ink-faint)]">
                          {analytics.campaign.status === 'scheduled'
                            ? 'Campaign scheduled to send. Recipient logs will populate when dispatch starts.'
                            : 'No recipient logs recorded.'}
                        </div>
                      ) : (
                        analytics.recipients.map((r) => (
                          <div
                            key={r.id}
                            className="p-3.5 flex items-center justify-between text-xs hover:bg-[var(--color-page)]"
                          >
                            <div className="flex items-center gap-3">
                              <span className="flex h-7 w-7 items-center justify-center rounded-full bg-slate-100 font-bold text-[var(--color-ink-muted)]">
                                {r.contactName?.charAt(0) || 'C'}
                              </span>
                              <div>
                                <span className="font-semibold text-[var(--color-ink)]">{r.contactName}</span>
                                <span className="ml-2 text-[var(--color-ink-faint)]">{r.contactPhone}</span>
                                {r.error && (
                                  <p className="text-[11px] text-rose-600 font-medium mt-0.5">
                                    ⚠️ {r.error}
                                  </p>
                                )}
                              </div>
                            </div>

                            <div className="flex items-center gap-3">
                              {r.status === 'read' && (
                                <span className="rounded bg-indigo-100 px-2 py-0.5 font-semibold text-indigo-800">
                                  Read ✓✓
                                </span>
                              )}
                              {r.status === 'delivered' && (
                                <span className="rounded bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-800">
                                  Delivered ✓✓
                                </span>
                              )}
                              {r.status === 'sent' && (
                                <span className="rounded bg-blue-100 px-2 py-0.5 font-semibold text-blue-800">
                                  Sent ✓
                                </span>
                              )}
                              {r.status === 'failed' && (
                                <span className="rounded bg-rose-100 px-2 py-0.5 font-semibold text-rose-800">
                                  Failed
                                </span>
                              )}
                              {r.status === 'skipped' && (
                                <span className="rounded bg-slate-100 px-2 py-0.5 font-semibold text-[var(--color-ink-muted)]">
                                  Skipped
                                </span>
                              )}

                              <Link
                                to={`/`}
                                className="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] font-semibold text-[var(--color-ink)] hover:bg-slate-100"
                              >
                                Follow up in chat ↗
                              </Link>
                            </div>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                </div>
              </div>
            ) : null}
          </div>
        </div>
      )}

      {/* Meta WhatsApp Pricing Calculator Modal */}
      {showPricingCalculator && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-xs">
          <div className="w-full max-w-xl rounded-2xl bg-[var(--color-surface)] shadow-2xl border border-[var(--color-border)] overflow-hidden">
            <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/70 px-6 py-4 flex items-center justify-between">
              <div>
                <h2 className="text-base font-bold text-[var(--color-ink)]">WhatsApp Pricing Calculator</h2>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Estimate Meta's per-conversation message costs across categories.
                </p>
              </div>
              <button
                onClick={() => setShowPricingCalculator(false)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink)] text-xl font-bold"
              >
                ×
              </button>
            </div>

            <div className="p-6 space-y-4">
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">
                    Country / Market
                  </label>
                  <select
                    value={calcRegion}
                    onChange={(e) => setCalcRegion(e.target.value)}
                    className="w-full rounded-md border border-slate-300 p-2 text-xs font-medium"
                  >
                    <option value="US">United States 🇺🇸</option>
                    <option value="UK">United Kingdom 🇬🇧</option>
                    <option value="EU">European Union 🇪🇺</option>
                    <option value="IN">India 🇮🇳</option>
                    <option value="BR">Brazil 🇧🇷</option>
                    <option value="MX">Mexico 🇲🇽</option>
                    <option value="UAE">United Arab Emirates 🇦🇪</option>
                    <option value="ROW">Rest of World 🌐</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">
                    Conversation Category
                  </label>
                  <select
                    value={calcCategory}
                    onChange={(e) => setCalcCategory(e.target.value)}
                    className="w-full rounded-md border border-slate-300 p-2 text-xs font-medium"
                  >
                    <option value="marketing">Marketing (Promotions, Offers)</option>
                    <option value="utility">Utility (Confirmations, Alerts)</option>
                    <option value="auth">Authentication (OTPs, Security)</option>
                    <option value="service">Service (User-initiated Support)</option>
                  </select>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)]">
                    Campaign Volume: {calcVolume.toLocaleString()} conversations
                  </label>
                  <span className="text-xs text-[var(--color-ink-faint)]">Slider: 250 - 50,000</span>
                </div>
                <input
                  type="range"
                  min="250"
                  max="50000"
                  step="250"
                  value={calcVolume}
                  onChange={(e) => setCalcVolume(Number(e.target.value))}
                  className="w-full"
                />
              </div>

              {pricingEstimate && (
                <div className="rounded-xl border border-blue-200 bg-blue-50/60 p-4 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-[var(--color-ink-muted)]">Rate per conversation:</span>
                    <span className="font-bold text-[var(--color-ink)]">
                      ${pricingEstimate.ratePerConversation.toFixed(4)} USD
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="font-bold text-slate-800">Total Estimated Meta Cost:</span>
                    <span className="text-lg font-extrabold text-blue-700">
                      ${pricingEstimate.estimatedCostUsd.toLocaleString()} USD
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-xs text-emerald-700 pt-1 border-t border-blue-200/60 font-semibold">
                    <span>Projected ROI (8.5x Industry Average):</span>
                    <span>~${pricingEstimate.estimatedRoiUsd.toLocaleString()} USD</span>
                  </div>
                  <p className="text-[11px] text-[var(--color-ink-muted)] pt-1">
                    💡 {pricingEstimate.freeServiceTierIncluded}
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* WhatsApp Campaigns FAQ Accordion ("WhatsApp campaigns, answered") */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-xs space-y-4">
        <div>
          <h2 className="text-base font-bold text-[var(--color-ink)]">WhatsApp campaigns, answered</h2>
          <p className="text-xs text-[var(--color-ink-muted)]">What teams ask before they send their first campaign.</p>
        </div>

        <div className="space-y-3 text-xs divide-y divide-slate-100">
          <details className="pt-2 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>What do I need to send WhatsApp campaigns?</span>
              <span className="text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform">⌄</span>
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              You need a verified WhatsApp Business account connected through the Meta Cloud API. In
              Settings → Connections, connect your WhatsApp Phone Number ID and Access Token.
            </p>
          </details>

          <details className="pt-3 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>Can I write any message I want in a WhatsApp campaign?</span>
              <span className="text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform">⌄</span>
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              WhatsApp requires bulk marketing messages sent outside the 24-hour service window to use
              pre-approved Message Templates. You can create templates in Settings and populate them
              with custom dynamic customer variables.
            </p>
          </details>

          <details className="pt-3 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>How do I choose who receives a WhatsApp campaign?</span>
              <span className="text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform">⌄</span>
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              Select one or multiple contact tags (e.g. <code>vip-customers</code>, <code>newsletter</code>).
              The system automatically calculates the deduplicated recipient list so nobody receives the
              same campaign twice.
            </p>
          </details>

          <details className="pt-3 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>Can I edit or reschedule a WhatsApp campaign?</span>
              <span className="text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform">⌄</span>
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              Yes. Until dispatch processing begins, you can update the audience, template parameters,
              or scheduled dispatch time directly from the campaign view.
            </p>
          </details>

          <details className="pt-3 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>What happens when a customer replies to a campaign?</span>
              <span className="text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform">⌄</span>
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              The reply lands directly in your shared omnichannel inbox. The conversation shows the
              originating campaign banner, delivery status, and our AI agent ("Captain") generates grounded
              draft replies to answer questions immediately.
            </p>
          </details>
        </div>
      </div>
    </div>
  )
}
