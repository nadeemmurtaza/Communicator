import { useEffect, useState } from 'react'
import { AlertTriangle, Check, CheckCheck, ChevronDown, DollarSign, History, Lightbulb, Megaphone, Plus, Tag, X } from '../components/icons'
import { Link } from 'react-router-dom'
import {
  api,
  type AudiencePreview,
  type Broadcast,
  type CampaignAnalytics,
  type PricingEstimate,
  type WhatsAppTemplate,
} from '../lib/api'

/** Formats with the currency the rate row actually stores — never a assumed "$". */
function formatMoney(value: number, currency: string, digits: number): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: 'currency',
      currency,
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    }).format(value)
  } catch {
    return `${value.toFixed(digits)} ${currency || 'unknown currency'}`
  }
}

export function Broadcasts() {
  const [campaigns, setCampaigns] = useState<Broadcast[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Creation Wizard Modal
  const [showWizard, setShowWizard] = useState(false)
  const [name, setName] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [tagInput, setTagInput] = useState('')
  const [audiencePreview, setAudiencePreview] = useState<AudiencePreview | null>(null)
  const [audienceError, setAudienceError] = useState<string | null>(null)
  // Tags are read from the contact table — never suggested from a list baked into this page.
  const [availableTags, setAvailableTags] = useState<string[]>([])
  const [availableTagsError, setAvailableTagsError] = useState<string | null>(null)
  // Templates come from Meta, never from a list written into this page.
  const [templates, setTemplates] = useState<WhatsAppTemplate[]>([])
  const [templatesError, setTemplatesError] = useState<string | null>(null)
  const [templateKey, setTemplateKey] = useState('')
  const [messageText, setMessageText] = useState('')
  const [paramsText, setParamsText] = useState('')
  const [scheduleTime, setScheduleTime] = useState('')
  const [sendImmediately, setSendImmediately] = useState(false)
  const [creating, setCreating] = useState(false)

  // Analytics View Modal
  const [selectedCampaignId, setSelectedCampaignId] = useState<string | null>(null)
  const [analytics, setAnalytics] = useState<CampaignAnalytics | null>(null)
  const [loadingAnalytics, setLoadingAnalytics] = useState(false)

  // Pricing Calculator Modal
  const [showPricingCalculator, setShowPricingCalculator] = useState(false)
  const [calcRegion, setCalcRegion] = useState('')
  const [calcCategory, setCalcCategory] = useState('marketing')
  const [calcVolume, setCalcVolume] = useState(1000)
  const [pricingEstimate, setPricingEstimate] = useState<PricingEstimate | null>(null)
  const [pricingError, setPricingError] = useState<string | null>(null)
  // Markets that actually have a row in `whatsapp_pricing_rates`.
  const [pricingRegions, setPricingRegions] = useState<Array<{ code: string; name: string }>>([])

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
    if (!showWizard) return
    if (selectedTags.length === 0) {
      setAudiencePreview(null)
      return
    }
    api
      .getAudiencePreview(selectedTags.join(','))
      .then((preview) => {
        setAudiencePreview(preview)
        setAudienceError(null)
      })
      .catch((e) => {
        setAudiencePreview(null)
        setAudienceError((e as Error).message)
      })
  }, [showWizard, selectedTags])

  useEffect(() => {
    if (!showWizard || templates.length > 0) return
    api
      .listWhatsAppTemplates()
      .then((rows) => {
        setTemplates(rows)
        setTemplatesError(null)
      })
      .catch((e) => {
        setTemplates([])
        setTemplatesError((e as Error).message)
      })
  }, [showWizard, templates.length])

  useEffect(() => {
    if (!showWizard || availableTags.length > 0) return
    api
      .listContactTags()
      .then((rows) => {
        setAvailableTags(rows.map((r) => r.name).filter(Boolean).sort())
        setAvailableTagsError(null)
      })
      .catch((e) => {
        setAvailableTags([])
        setAvailableTagsError((e as Error).message)
      })
  }, [showWizard, availableTags.length])

  // Recalculate WhatsApp pricing estimate
  useEffect(() => {
    if (!showPricingCalculator || !calcRegion) return
    api
      .getWhatsAppPricing(calcRegion, calcCategory, calcVolume)
      .then((estimate) => {
        setPricingEstimate(estimate)
        setPricingRegions(estimate.supportedRegions ?? [])
        setPricingError(null)
      })
      .catch((e) => {
        setPricingEstimate(null)
        setPricingError((e as Error).message)
      })
  }, [showPricingCalculator, calcRegion, calcCategory, calcVolume])

  const selectedTemplate = templates.find((t) => `${t.name}:${t.language}` === templateKey) ?? null

  function parseTemplateParams(): Record<string, string> | undefined {
    const entries: Record<string, string> = {}
    for (const line of paramsText.split(/\r?\n/)) {
      const trimmed = line.trim()
      if (!trimmed) continue
      const split = trimmed.indexOf('=')
      if (split < 1) continue
      entries[trimmed.slice(0, split).trim()] = trimmed.slice(split + 1).trim()
    }
    return Object.keys(entries).length > 0 ? entries : undefined
  }

  async function handleCreateCampaign(e: React.FormEvent) {
    e.preventDefault()
    setCreating(true)
    setError(null)

    if (!messageText.trim()) {
      setError('Write the message. The backend refuses to send a campaign with no body text.')
      setCreating(false)
      return
    }
    if (selectedTags.length === 0) {
      setError('Add at least one audience tag. Campaigns are never sent to every contact.')
      setCreating(false)
      return
    }

    try {
      const created = await api.createBroadcast({
        name,
        template_name: selectedTemplate?.name,
        template_language: selectedTemplate?.language,
        body_preview: messageText,
        template_params: parseTemplateParams(),
        tag_name: selectedTags.join(','),
        scheduled_at: sendImmediately ? null : new Date(scheduleTime).toISOString(),
      })
      if (sendImmediately && created?.id) {
        const outcome = await api.dispatchBroadcast(created.id)
        const sent = outcome.sentCount ?? 0
        const failed = outcome.failedCount ?? 0
        const skipped = outcome.skippedCount ?? 0
        if (sent > 0) {
          setSuccess(
            `Campaign "${name}" dispatched: ${sent} sent, ${failed} failed, ${skipped} not sent.`,
          )
        } else {
          setError(
            `Campaign "${name}" was created but nothing was delivered (${failed} failed, ${skipped} not sent). ` +
              'Open Analytics for the per-recipient reason.',
          )
        }
      } else {
        setSuccess(`Campaign "${name}" scheduled successfully!`)
      }
      setShowWizard(false)
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
      const outcome = await api.dispatchBroadcast(id)
      const sent = outcome.sentCount ?? 0
      const failed = outcome.failedCount ?? 0
      const skipped = outcome.skippedCount ?? 0
      if (sent > 0) {
        setSuccess(
          `Campaign dispatched: ${sent} sent, ${failed} refused by the provider, ${skipped} not sent.`,
        )
      } else {
        setError(
          `Nothing was delivered (${failed} refused, ${skipped} not sent). Open Analytics for the \
per-recipient reason.`,
        )
      }
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

  async function handleProcessScheduled() {
    try {
      setError(null)
      const res = await api.processScheduledCampaigns()
      const blocked = (res.campaigns ?? []).filter((c: any) => c.status === 'not_dispatched')
      setSuccess(
        `${res.processed} campaign(s) dispatched${blocked.length ? `, ${blocked.length} refused by the dispatcher` : ''}.`,
      )
      if (blocked.length) {
        setError(blocked.map((c: any) => c.error).filter(Boolean).join(' | '))
      }
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
            <span className="rounded-full bg-slate-100 px-2 py-0.5 text-xs font-semibold text-[var(--color-ink-muted)]">
              Sends via the WhatsApp Cloud API channel
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
            <span className="flex items-center gap-1.5"><DollarSign className="h-3.5 w-3.5 text-emerald-600" /> Pricing Calculator</span>
          </button>

          <button
            onClick={handleProcessScheduled}
            className="rounded-md border border-blue-200 bg-blue-50 px-3 py-1.5 text-xs font-semibold text-blue-800 hover:bg-blue-100 shadow-xs"
            title="Checks and sends any campaigns whose scheduled time has arrived"
          >
            <span className="flex items-center gap-1.5"><History className="h-3.5 w-3.5" /> Process Due</span>
          </button>

          <button
            onClick={() => setShowWizard(true)}
            className="rounded-md bg-[var(--color-accent)] px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow-xs"
          >
            <span className="flex items-center gap-1"><Plus className="h-3.5 w-3.5" /> Create campaign</span>
          </button>
        </div>
      </div>

      {/* Notifications */}
      {error && (
        <div className="flex items-center justify-between rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="font-bold">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="font-bold">
            <X className="h-4 w-4" />
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
            <Megaphone className="h-10 w-10 text-indigo-600 mx-auto" />
            <h3 className="mt-2 text-sm font-semibold text-[var(--color-ink)]">No campaigns yet</h3>
            <p className="mt-1 text-xs text-[var(--color-ink-muted)] max-w-sm mx-auto">
              Create your first WhatsApp campaign in 3 easy steps: audience, message, and schedule.
            </p>
            <div className="mt-4 flex justify-center gap-2">
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
                    {c.status === 'failed' && (
                      <span className="rounded-full bg-rose-100 px-2.5 py-0.5 text-[11px] font-semibold text-rose-800">
                        No message accepted
                      </span>
                    )}
                  </div>

                  <p className="mt-1 text-xs text-[var(--color-ink-muted)] flex flex-wrap items-center gap-2">
                    <span>
                      {c.templateName
                        ? `Template ${c.templateName}${
                            c.templateLanguage ? ` (${c.templateLanguage})` : ''
                          }`
                        : 'Free-form message (no Meta template)'}
                    </span>
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
                    <span className="font-medium text-[var(--color-ink-muted)]">
                      Tags: {c.tagName || 'none recorded'}
                    </span>
                  </p>

                  {/* Real Delivery Stats Bar with Delivery Rates */}
                  {(() => {
                    const audience = c.audienceCount || c.sentCount || 0
                    const delivered = c.deliveredCount || 0
                    const sent = c.sentCount || 0
                    const read = c.readCount || 0
                    const failed = c.failedCount || 0
                    const deliveryRate = audience > 0 ? Math.round((delivered / audience) * 100) : (sent > 0 ? Math.round((delivered / sent) * 100) : 0)
                    const readRate = delivered > 0 ? Math.round((read / delivered) * 100) : 0
                    return (
                      <div className="mt-2.5 space-y-2">
                        <div className="flex flex-wrap items-center gap-3 text-xs text-[var(--color-ink-muted)]">
                          <span>
                            Audience: <strong className="text-[var(--color-ink)]">{audience}</strong>
                          </span>
                          <span>·</span>
                          <span>
                            Sent: <strong className="text-[var(--color-ink)]">{sent}</strong>
                          </span>
                          <span>·</span>
                          <span>
                            Delivered: <strong className="text-emerald-700">{delivered}</strong>
                          </span>
                          <span>·</span>
                          <span>
                            Read: <strong className="text-indigo-700">{read}</strong>
                          </span>
                          {failed > 0 && (
                            <>
                              <span>·</span>
                              <span className="text-rose-600 font-medium">
                                Failed: <strong>{failed}</strong>
                              </span>
                            </>
                          )}
                          <span
                            className={`rounded-full px-2 py-0.5 text-[11px] font-bold ${
                              delivered > 0
                                ? 'bg-emerald-100 text-emerald-800'
                                : 'bg-slate-100 text-[var(--color-ink-muted)]'
                            }`}
                            title={
                              delivered > 0
                                ? 'Delivery receipts recorded by the Meta status webhook'
                                : 'No delivery receipt has arrived from the Meta status webhook yet'
                            }
                          >
                            {deliveryRate}% Delivered
                          </span>
                          {delivered > 0 && (
                            <span className="rounded-full bg-indigo-100 px-2 py-0.5 text-[11px] font-bold text-indigo-800">
                              {readRate}% Read Rate
                            </span>
                          )}
                        </div>

                        {/* Mini Delivery Breakdown Bar */}
                        {audience > 0 && (
                          <div className="h-1.5 w-full max-w-xs rounded-full bg-slate-200 overflow-hidden flex">
                            <div
                              style={{ width: `${Math.min(100, Math.round((read / audience) * 100))}%` }}
                              className="bg-indigo-600 h-full"
                              title={`Read: ${read}`}
                            />
                            <div
                              style={{ width: `${Math.min(100, Math.max(0, Math.round(((delivered - read) / audience) * 100)))}%` }}
                              className="bg-emerald-500 h-full"
                              title={`Delivered (unread): ${delivered - read}`}
                            />
                            {failed > 0 && (
                              <div
                                style={{ width: `${Math.min(100, Math.round((failed / audience) * 100))}%` }}
                                className="bg-rose-500 h-full"
                                title={`Failed: ${failed}`}
                              />
                            )}
                          </div>
                        )}
                      </div>
                    )
                  })()}
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
                <X className="h-4 w-4" />
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
                    {audiencePreview?.dailyLimit !== undefined && audiencePreview?.dailyLimit !== null
                      ? `Daily cap from Meta: ${audiencePreview.dailyLimit.toLocaleString()} conversations`
                      : 'Daily cap: not reported by Meta'}
                  </span>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Pick one or more labels that already exist on your contacts. You see how many
                  people will get the message before you send it, and nobody gets it twice.
                </p>

                {availableTagsError ? (
                  <p className="flex items-start gap-1.5 text-xs font-semibold text-rose-600">
                    <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
                    <span>{availableTagsError}</span>
                  </p>
                ) : null}

                {availableTags.length > 0 ? (
                  <div className="flex flex-wrap items-center gap-1.5">
                    <span className="text-[10px] font-bold uppercase tracking-wider text-[var(--color-ink-faint)]">
                      Tags in your contacts
                    </span>
                    {availableTags
                      .filter((t) => !selectedTags.includes(t))
                      .map((t) => (
                        <button
                          key={t}
                          type="button"
                          onClick={() => setSelectedTags([...selectedTags, t])}
                          className="rounded border border-slate-300 bg-[var(--color-surface)] px-2 py-0.5 text-xs text-[var(--color-ink)] hover:border-blue-400 hover:bg-blue-50"
                        >
                          + {t}
                        </button>
                      ))}
                    {availableTags.filter((t) => !selectedTags.includes(t)).length === 0 ? (
                      <span className="text-xs text-[var(--color-ink-faint)]">
                        all {availableTags.length} tag(s) are already in this audience
                      </span>
                    ) : null}
                  </div>
                ) : availableTagsError ? null : (
                  <p className="text-xs text-[var(--color-ink-faint)]">
                    No tags exist on your contacts yet, so there is no audience to pick. Type one
                    below only if you have already labelled contacts with it.
                  </p>
                )}

                {/* Tags Picker */}
                <div className="flex flex-wrap items-center gap-2">
                  {selectedTags.map((t) => (
                    <span
                      key={t}
                      className="inline-flex items-center gap-1 rounded bg-blue-100 px-2.5 py-1 text-xs font-semibold text-blue-800"
                    >
                      <Tag className="h-3 w-3" />
                      <span>{t}</span>
                      <button
                        type="button"
                        onClick={() => removeTag(t)}
                        className="ml-1 text-blue-600 hover:text-blue-900 font-bold"
                      >
                        <X className="h-4 w-4" />
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
                <div className="mt-2 rounded-lg bg-[var(--color-surface)] p-3 border border-[var(--color-border)] text-xs space-y-2">
                  {audienceError && selectedTags.length > 0 ? (
                    <p className="flex items-start gap-1.5 font-semibold text-rose-600">
                      <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
                      <span>{audienceError}</span>
                    </p>
                  ) : audiencePreview ? (
                    <>
                      <div className="flex items-center justify-between">
                        <span className="flex items-center gap-2 font-bold text-[var(--color-ink)]">
                          <span className="h-2 w-2 rounded-full bg-emerald-500" />
                          <span>{audiencePreview.totalSelected} contacts matched these tags</span>
                        </span>
                        <span className="text-[var(--color-ink-muted)]">
                          Deduplicated across all tags
                        </span>
                      </div>
                      <div className="flex flex-wrap gap-x-4 gap-y-1 text-[var(--color-ink-muted)]">
                        <span>Consent recorded as opt-in: {audiencePreview.optedInCount}</span>
                        <span>Opted out: {audiencePreview.optedOutCount}</span>
                        <span>No consent record: {audiencePreview.unknownConsentCount}</span>
                        <span>
                          Daily cap:{' '}
                          {audiencePreview.dailyLimit === null
                            ? `not reported${
                                audiencePreview.dailyLimitSource
                                  ? ` (${audiencePreview.dailyLimitSource})`
                                  : ' by Meta'
                              }`
                            : `${audiencePreview.dailyLimit} conversations`
                          }
                        </span>
                      </div>
                      {audiencePreview.warnings && audiencePreview.warnings.length > 0 ? (
                        <ul className="space-y-1 border-t border-[var(--color-border)] pt-2">
                          {audiencePreview.warnings.map((w) => (
                            <li key={w} className="flex items-start gap-1.5 text-amber-700">
                              <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
                              <span>{w}</span>
                            </li>
                          ))}
                        </ul>
                      ) : null}
                    </>
                  ) : selectedTags.length === 0 ? (
                    <span className="text-[var(--color-ink-muted)]">
                      Add a tag to see who is actually in this audience.
                    </span>
                  ) : (
                    <span className="text-[var(--color-ink-muted)]">Calculating audience...</span>
                  )}
                </div>
              </div>

              {/* Step 2: Message */}
              <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)]/40 p-4 space-y-3">
                <div className="flex items-center gap-2">
                  <span className="flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-[11px] font-bold text-white">
                    2
                  </span>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Message: what each customer will receive
                  </h3>
                </div>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  The list below is read live from your Meta WhatsApp Business account. A template
                  that has not been approved by Meta will not appear here, and sending one is
                  refused by the provider rather than quietly falling back to a made-up message.
                </p>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-3">
                    <div>
                      <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">
                        Template from Meta
                      </label>
                      <select
                        value={templateKey}
                        onChange={(e) => setTemplateKey(e.target.value)}
                        className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium"
                      >
                        <option value="">
                          {templatesError
                            ? 'No templates available (see the error below)'
                            : templates.length === 0
                              ? 'No templates synced yet'
                              : 'None — send as a free-form message'}
                        </option>
                        {templates.map((t) => (
                          <option key={`${t.name}:${t.language}`} value={`${t.name}:${t.language}`}>
                            {t.name} · {t.language} · {t.status}
                          </option>
                        ))}
                      </select>
                      {templatesError ? (
                        <p className="mt-1 flex items-start gap-1.5 text-[11px] font-semibold text-rose-600">
                          <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
                          <span>{templatesError}</span>
                        </p>
                      ) : (
                        <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
                          {templates.length} template(s) returned by the Meta sync. Only
                          APPROVED templates are sendable on WhatsApp.
                        </p>
                      )}
                    </div>

                    {selectedTemplate?.bodyText ? (
                      <div className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] p-2">
                        <span className="block text-[10px] font-bold uppercase tracking-wider text-[var(--color-ink-muted)]">
                          Approved body text
                        </span>
                        <p className="mt-1 whitespace-pre-wrap text-xs text-[var(--color-ink)]">
                          {selectedTemplate.bodyText}
                        </p>
                      </div>
                    ) : null}

                    <div>
                      <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">
                        Message body
                      </label>
                      <textarea
                        rows={5}
                        value={messageText}
                        onChange={(e) => setMessageText(e.target.value)}
                        placeholder={'Hi {{name}}, …'}
                        className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-2 text-xs font-medium"
                      />
                      <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
                        Stored as the campaign body and sent as-is. {'{{name}}'}, {'{{first_name}}'},
                        {'{{last_name}}'} and {'{{phone}}'} are filled from each contact; an
                        unresolved placeholder skips that recipient instead of sending the raw token.
                      </p>
                    </div>

                    <div>
                      <label className="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">
                        Template variables (one per line, name=value)
                      </label>
                      <textarea
                        rows={3}
                        value={paramsText}
                        onChange={(e) => setParamsText(e.target.value)}
                        placeholder={'discount=20%'}
                        className="w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-2 font-mono text-xs"
                      />
                      <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
                        Sent to Meta as the template payload in the order the approved body expects
                        them. Leave empty if the template has no variables.
                      </p>
                    </div>
                  </div>

                  {/* WhatsApp Mobile Balloon Preview */}
                  <div className="rounded-lg bg-[#efeae2] p-4 flex flex-col justify-end border border-[#d1d7db]">
                    <span className="text-[10px] font-bold text-[var(--color-ink-muted)] uppercase tracking-wider mb-2">
                      Preview of the message body
                    </span>
                    {messageText.trim() ? (
                      <div className="rounded-lg bg-[var(--color-surface)] p-3 shadow-xs max-w-[90%] text-xs text-[var(--color-ink)] relative">
                        <p className="whitespace-pre-wrap leading-relaxed">{messageText}</p>
                        <div className="mt-2 flex items-center justify-end gap-1 text-[10px] text-[var(--color-ink-faint)]">
                          <span>not sent yet</span>
                        </div>
                      </div>
                    ) : (
                      <p className="text-xs text-[var(--color-ink-muted)]">
                        Nothing to show yet. Delivery ticks and timestamps appear in Analytics once
                        Meta posts status webhooks back for real recipients.
                      </p>
                    )}
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
                <X className="h-4 w-4" />
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
                                {r.contactName?.charAt(0) || '·'}
                              </span>
                              <div>
                                <span className="font-semibold text-[var(--color-ink)]">
                                  {r.contactName || 'No name stored on this contact'}
                                </span>
                                <span className="ml-2 text-[var(--color-ink-faint)]">{r.contactPhone}</span>
                                {r.error && (
                                  <p className="text-[11px] text-rose-600 font-medium mt-0.5">
                                    <span className="flex items-center gap-1"><AlertTriangle className="h-3 w-3 text-rose-500 shrink-0" /> {r.error}</span>
                                  </p>
                                )}
                              </div>
                            </div>

                            <div className="flex items-center gap-3">
                              {r.readAt ? (
                                <span className="rounded bg-indigo-100 px-2 py-0.5 font-semibold text-indigo-800">
                                  <span className="flex items-center gap-1">Read <CheckCheck className="h-3 w-3 text-indigo-700" /></span>
                                </span>
                              ) : null}
                              {r.deliveredAt && !r.readAt ? (
                                <span className="rounded bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-800">
                                  <span className="flex items-center gap-1">Delivered <CheckCheck className="h-3 w-3 text-emerald-700" /></span>
                                </span>
                              ) : null}
                              {r.status === 'sent' && !r.deliveredAt ? (
                                <span className="rounded bg-blue-100 px-2 py-0.5 font-semibold text-blue-800">
                                  <span className="flex items-center gap-1">Sent <Check className="h-3 w-3 text-blue-700" /></span>
                                </span>
                              ) : null}
                              {r.status === 'failed' ? (
                                <span className="rounded bg-rose-100 px-2 py-0.5 font-semibold text-rose-800">
                                  Failed
                                </span>
                              ) : null}
                              {r.status === 'not_sent' ? (
                                <span className="rounded bg-slate-100 px-2 py-0.5 font-semibold text-[var(--color-ink-muted)]">
                                  Not sent
                                </span>
                              ) : null}

                              <Link
                                to={`/contacts/${r.contactId}`}
                                className="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] font-semibold text-[var(--color-ink)] hover:bg-slate-100"
                              >
                                Open contact ↗
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
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="p-6 space-y-4">
              <p className="text-[11px] text-[var(--color-ink-muted)]">
                Rates come from the <code>whatsapp_pricing_rates</code> table, which you fill from
                your own Meta WhatsApp Business Platform invoice. Nothing here is estimated when a
                rate row is missing — the calculator says so instead of inventing a number.
              </p>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-[var(--color-ink-muted)] mb-1">
                    Country / Market
                  </label>
                  {pricingRegions.length > 0 ? (
                    <select
                      value={calcRegion}
                      onChange={(e) => setCalcRegion(e.target.value)}
                      className="w-full rounded-md border border-slate-300 p-2 text-xs font-medium"
                    >
                      <option value="">Select a configured market</option>
                      {pricingRegions.map((r) => (
                        <option key={r.code} value={r.code}>
                          {r.name} ({r.code})
                        </option>
                      ))}
                    </select>
                  ) : (
                    <input
                      value={calcRegion}
                      onChange={(e) => setCalcRegion(e.target.value.toUpperCase())}
                      placeholder="e.g. DE"
                      className="w-full rounded-md border border-slate-300 p-2 text-xs font-medium"
                    />
                  )}
                  {pricingRegions.length === 0 ? (
                    <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
                      No market has a stored rate yet, so this is a free-text code. Type one and the
                      response lists exactly which rows exist.
                    </p>
                  ) : null}
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

              {!calcRegion ? (
                <p className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)]/60 p-3 text-xs text-[var(--color-ink-muted)]">
                  Choose a market to price. Rates are only shown for the rows that exist in
                  <code>whatsapp_pricing_rates</code>.
                </p>
              ) : pricingError ? (
                <div className="rounded-xl border border-rose-200 bg-rose-50/70 p-4 space-y-1">
                  <p className="flex items-start gap-1.5 text-xs font-bold text-rose-700">
                    <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
                    <span>{pricingError}</span>
                  </p>
                  <p className="text-[11px] text-rose-600">
                    No figure is shown. When a rate row is missing the calculator reports that
                    instead of substituting an industry average.
                  </p>
                </div>
              ) : pricingEstimate ? (
                <div className="rounded-xl border border-blue-200 bg-blue-50/60 p-4 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-[var(--color-ink-muted)]">Rate per conversation:</span>
                    <span className="font-bold text-[var(--color-ink)]">
                      {formatMoney(pricingEstimate.ratePerConversation, pricingEstimate.currency, 4)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="font-bold text-slate-800">
                      Total for {pricingEstimate.volume.toLocaleString()} conversations:
                    </span>
                    <span className="text-lg font-extrabold text-blue-700">
                      {formatMoney(pricingEstimate.estimatedCost, pricingEstimate.currency, 2)}
                    </span>
                  </div>
                  <p className="text-[11px] text-[var(--color-ink-muted)]">
                    Rate row source: {pricingEstimate.rateSource || 'not recorded on the row'}
                    {pricingEstimate.rateUpdatedAt
                      ? ` · last updated ${pricingEstimate.rateUpdatedAt}`
                      : ''}
                  </p>
                  {(pricingEstimate.warnings ?? []).map((w) => (
                    <p key={w} className="flex items-start gap-1.5 text-[11px] text-amber-700">
                      <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
                      <span>{w}</span>
                    </p>
                  ))}
                  <p className="text-[11px] text-[var(--color-ink-muted)] pt-1 border-t border-blue-200/60">
                    <span className="flex items-start gap-1.5">
                      <Lightbulb className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
                      {pricingEstimate.roiNote}
                    </span>
                  </p>
                </div>
              ) : null}
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
              <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform" />
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              You need a verified WhatsApp Business account connected through the Meta Cloud API. In
              Settings → Connections, connect your WhatsApp Phone Number ID and Access Token.
            </p>
          </details>

          <details className="pt-3 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>Can I write any message I want in a WhatsApp campaign?</span>
              <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform" />
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
              <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform" />
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
              <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform" />
            </summary>
            <p className="mt-2 text-[var(--color-ink-muted)] leading-relaxed pl-2">
              Yes. Until dispatch processing begins, you can update the audience, template parameters,
              or scheduled dispatch time directly from the campaign view.
            </p>
          </details>

          <details className="pt-3 group cursor-pointer">
            <summary className="font-semibold text-slate-800 flex items-center justify-between list-none">
              <span>What happens when a customer replies to a campaign?</span>
              <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)] group-open:rotate-180 transition-transform" />
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
