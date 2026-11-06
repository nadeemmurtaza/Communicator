import { useEffect, useRef, useState, type RefObject } from 'react'
import { Link } from 'react-router-dom'
import {
  Sparkles,
  Radio,
  ArrowRight,
  CheckCircle2,
  XCircle,
  RefreshCw,
  Zap,
  Users,
  Send,
  AlertTriangle,
  FlameAnimated,
  MessageSquareAnimated,
  ShieldCheckAnimated,
  SparklesAnimated,
  type AnimatedIconHandle,
} from '../components/icons'
import {
  api,
  type Conversation,
  type Draft,
  type SLAMetrics,
  type IntegrationSettings,
  type Inbox,
} from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

// A ref hands hover to the caller, so the card -- not the 32px chip inside it --
// decides when its icon moves.
const hoverIcon = (icon: RefObject<AnimatedIconHandle | null>) => ({
  onMouseEnter: () => icon.current?.startAnimation(),
  onMouseLeave: () => icon.current?.stopAnimation(),
})

export function Dashboard() {
  const [drafts, setDrafts] = useState<Draft[] | null>(null)
  const [conversations, setConversations] = useState<Conversation[] | null>(null)
  const [slaMetrics, setSlaMetrics] = useState<SLAMetrics | null>(null)
  const [settings, setSettings] = useState<IntegrationSettings | null>(null)
  const [inboxes, setInboxes] = useState<Inbox[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [busyDraftId, setBusyDraftId] = useState<string | null>(null)
  const [firstLoadComplete, setFirstLoadComplete] = useState(false)
  const draftsIcon = useRef<AnimatedIconHandle>(null)
  const leadsIcon = useRef<AnimatedIconHandle>(null)
  const conversationsIcon = useRef<AnimatedIconHandle>(null)
  const slaIcon = useRef<AnimatedIconHandle>(null)

  const loadData = async () => {
    const isFirstLoad = !firstLoadComplete
    setIsRefreshing(true)
    setError(null)
    try {
      const [draftsRes, convsRes, slaRes, settingsRes, inboxesRes] = await Promise.allSettled([
        api.listDrafts(),
        api.listConversations(),
        api.getSLAMetrics(),
        api.getIntegrationSettings(),
        api.listInboxes(),
      ])

      const anyFulfilled =
        draftsRes.status === 'fulfilled' ||
        convsRes.status === 'fulfilled' ||
        slaRes.status === 'fulfilled' ||
        settingsRes.status === 'fulfilled' ||
        inboxesRes.status === 'fulfilled'

      if (draftsRes.status === 'fulfilled') setDrafts(draftsRes.value)
      if (convsRes.status === 'fulfilled') setConversations(convsRes.value)
      if (slaRes.status === 'fulfilled') setSlaMetrics(slaRes.value)
      if (settingsRes.status === 'fulfilled') setSettings(settingsRes.value)
      if (inboxesRes.status === 'fulfilled') setInboxes(inboxesRes.value)

      setFirstLoadComplete(true)

      // Never show error during the first load — only on manual refresh
      if (!isFirstLoad && !anyFulfilled) {
        setError('Unable to connect to Communicator API backend')
      }
    } catch (err: unknown) {
      setFirstLoadComplete(true)
      // Never show error during the first load — only on manual refresh
      if (!isFirstLoad) {
        setError(err instanceof Error ? err.message : 'Failed to load dashboard data')
      }
    } finally {
      setIsRefreshing(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [])

  const handleDraftAction = async (id: string, action: 'approve' | 'reject') => {
    setBusyDraftId(id)
    try {
      if (action === 'approve') {
        await api.approveDraft(id)
      } else {
        await api.rejectDraft(id)
      }
      setDrafts((prev) => (prev ? prev.filter((d) => d.id !== id) : null))
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Action failed')
    } finally {
      setBusyDraftId(null)
    }
  }

  // Derived metrics
  const hotLeads = conversations?.filter((c) => c.state === 'hot_lead') ?? []
  const awaitingReply = conversations?.filter((c) => c.state === 'awaiting_reply') ?? []

  const isWaConnected = Boolean(settings?.whatsapp?.phoneNumberId || inboxes?.some((i) => i.channelType === 'whatsapp' && i.isActive))
  const isLcConnected = Boolean(inboxes?.some((i) => (i.channelType === 'live_chat' || i.channelType === 'live-chat') && i.isActive))
  const isEmailConnected = Boolean(inboxes?.some((i) => i.channelType === 'email' && i.isActive))
  const isSmsConnected = Boolean(inboxes?.some((i) => i.channelType === 'sms' && i.isActive))

  const slaFulfillment = (slaMetrics?.firstResponseFulfillmentRate !== undefined && slaMetrics?.firstResponseFulfillmentRate !== null)
    ? Math.round(slaMetrics.firstResponseFulfillmentRate)
    : null

  return (
    <div className="space-y-8 max-w-7xl mx-auto">
      {/* Top Header & Quick Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-[var(--color-border)] pb-5">
        <div>
          <div className="flex items-center gap-2.5">
            <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
              Communicator Dashboard
            </h1>
            <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700 border border-emerald-200">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
              Live Operations
            </span>
          </div>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Outbound &amp; inbound omnichannel orchestration, AI drafting queue, and lead pipeline.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2.5">
          <button
            onClick={loadData}
            disabled={isRefreshing}
            className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] shadow-xs hover:bg-[var(--color-page)] disabled:opacity-50 cursor-pointer"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
            Refresh
          </button>
          <Link
            to="/broadcasts"
            className="inline-flex items-center gap-1.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] shadow-xs hover:bg-[var(--color-page)]"
          >
            <Send className="w-3.5 h-3.5 text-[var(--color-ink-muted)]" />
            New Broadcast
          </Link>
          <Link
            to="/drafts"
            className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-3.5 py-1.5 text-xs font-medium text-white shadow-xs hover:bg-[var(--color-accent-hover)]"
          >
            <Sparkles className="w-3.5 h-3.5" />
            Review Drafts {drafts && drafts.length > 0 && `(${drafts.length})`}
          </Link>
        </div>
      </div>

      {error && firstLoadComplete && (
        <div className="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          <AlertTriangle className="w-4 h-4 shrink-0 text-red-600" />
          <span>{error}</span>
        </div>
      )}

      {/* KPI Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Pending Drafts */}
        <Link
          to="/drafts"
          className="group block rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs transition hover:border-amber-300 hover:shadow-sm"
          {...hoverIcon(draftsIcon)}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
              Pending AI Drafts
            </span>
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-50 text-amber-600">
              <SparklesAnimated ref={draftsIcon} size={20} />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold tracking-tight text-[var(--color-ink)]">
              {drafts === null ? '—' : drafts.length}
            </span>
            {drafts && drafts.length > 0 && (
              <span className="text-xs font-semibold text-amber-600">Needs review</span>
            )}
          </div>
          <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
            High-confidence AI drafted replies waiting for approval
          </p>
        </Link>

        {/* Hot Leads */}
        <Link
          to="/conversations"
          className="group block rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs transition hover:border-rose-300 hover:shadow-sm"
          {...hoverIcon(leadsIcon)}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
              Hot Leads
            </span>
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-rose-50 text-rose-600">
              <FlameAnimated ref={leadsIcon} size={20} />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold tracking-tight text-[var(--color-ink)]">
              {conversations === null ? '—' : hotLeads.length}
            </span>
            {hotLeads.length > 0 && (
              <span className="text-xs font-semibold text-rose-600">High intent</span>
            )}
          </div>
          <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
            Strong buying intent detected by AI analysis
          </p>
        </Link>

        {/* Active Conversations */}
        <Link
          to="/conversations"
          className="group block rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs transition hover:border-indigo-300 hover:shadow-sm"
          {...hoverIcon(conversationsIcon)}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
              Open Conversations
            </span>
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-50 text-indigo-600">
              <MessageSquareAnimated ref={conversationsIcon} size={20} />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold tracking-tight text-[var(--color-ink)]">
              {conversations === null ? '—' : conversations.length}
            </span>
            <span className="text-xs text-[var(--color-ink-muted)]">
              ({awaitingReply.length} waiting reply)
            </span>
          </div>
          <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
            Omnichannel threads across WhatsApp, SMS &amp; Email
          </p>
        </Link>

        {/* SLA & Automation Health */}
        <Link
          to="/sla"
          className="group block rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs transition hover:border-emerald-300 hover:shadow-sm"
          {...hoverIcon(slaIcon)}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
              SLA Health
            </span>
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-50 text-emerald-600">
              <ShieldCheckAnimated ref={slaIcon} size={20} />
            </div>
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-3xl font-bold tracking-tight text-[var(--color-ink)]">
              {slaFulfillment !== null ? `${slaFulfillment}%` : '100%'}
            </span>
            <span className={`text-xs font-medium ${slaFulfillment !== null && slaFulfillment < 90 ? 'text-amber-600' : 'text-emerald-600'}`}>
              {slaFulfillment === null ? 'No Breaches' : slaFulfillment >= 90 ? 'Compliant' : 'Needs Review'}
            </span>
          </div>
          <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
            Response cadence within configured business hours
          </p>
        </Link>
      </div>

      {/* Main Operational Split: Drafts Queue & Pipeline Summary */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Left 2 Cols: AI Drafts Awaiting Review */}
        <section className="lg:col-span-2 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-base font-semibold text-[var(--color-ink)] flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-amber-500" />
                Drafts Awaiting Review
              </h2>
              <p className="text-xs text-[var(--color-ink-muted)]">
                Autonomous Claude-drafted responses pending 1-click human verification
              </p>
            </div>
            <Link
              to="/drafts"
              className="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-accent)] hover:underline"
            >
              View all ({drafts?.length ?? 0})
              <ArrowRight className="w-3.5 h-3.5" />
            </Link>
          </div>

          {drafts === null ? (
            <div className="space-y-3">
              {[1, 2, 3].map((i) => (
                <div
                  key={i}
                  className="h-24 animate-pulse rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]"
                />
              ))}
            </div>
          ) : drafts.length === 0 ? (
            <div className="rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center">
              <CheckCircle2 className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
              <h3 className="text-sm font-semibold text-[var(--color-ink)]">All caught up!</h3>
              <p className="text-xs text-[var(--color-ink-muted)] mt-1">
                No pending drafts requiring review. New responses will land here when leads reply.
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {drafts.slice(0, 4).map((d) => (
                <div
                  key={d.id}
                  className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-xs transition hover:border-slate-300"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0 space-y-1.5 flex-1">
                      <div className="flex items-center gap-2">
                        <Link
                          to={`/conversations/${d.conversationId}`}
                          className="font-mono text-xs font-medium text-[var(--color-ink-muted)] hover:underline"
                        >
                          conv/{d.conversationId.slice(0, 8)}
                        </Link>
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium bg-amber-50 text-amber-700 border border-amber-200/60">
                          {Math.round(d.confidence * 100)}% confidence
                        </span>
                        <span className="text-[11px] text-[var(--color-ink-faint)]">
                          {timeAgo(d.createdAt)}
                        </span>
                      </div>
                      <p className="line-clamp-2 text-sm text-slate-800 leading-relaxed font-normal">
                        {d.body}
                      </p>
                    </div>

                    <div className="flex shrink-0 items-center gap-2">
                      <button
                        type="button"
                        disabled={busyDraftId === d.id}
                        onClick={() => handleDraftAction(d.id, 'reject')}
                        className="rounded-lg border border-[var(--color-border)] px-2.5 py-1.5 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50 cursor-pointer"
                        title="Reject draft"
                      >
                        <XCircle className="w-4 h-4 text-slate-400 hover:text-rose-500" />
                      </button>
                      <button
                        type="button"
                        disabled={busyDraftId === d.id}
                        onClick={() => handleDraftAction(d.id, 'approve')}
                        className="inline-flex items-center gap-1.5 rounded-lg bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white shadow-xs hover:bg-[var(--color-accent-hover)] disabled:opacity-50 cursor-pointer"
                      >
                        <Send className="w-3 h-3" />
                        Approve &amp; Send
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Right Col: Channel Hub & Quick Gateway Status */}
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-base font-semibold text-[var(--color-ink)] flex items-center gap-2">
                <Radio className="w-4 h-4 text-emerald-500" />
                Channel Integrations
              </h2>
              <p className="text-xs text-[var(--color-ink-muted)]">Active omnichannel gateways</p>
            </div>
            <Link
              to="/channels"
              className="text-xs font-medium text-[var(--color-accent)] hover:underline"
            >
              All channels
            </Link>
          </div>

          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] divide-y divide-[var(--color-border)] shadow-xs">
            {/* WhatsApp */}
            <div className="p-3.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-50 text-emerald-600 font-bold text-xs">
                  WA
                </div>
                <div>
                  <h4 className="text-xs font-semibold text-[var(--color-ink)]">WhatsApp Business</h4>
                  <p className="text-[11px] text-[var(--color-ink-faint)]">
                    {settings?.whatsapp.provider === 'meta'
                      ? 'Meta Cloud Graph API'
                      : 'Twilio Gateway'}
                  </p>
                </div>
              </div>
              {isWaConnected ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-50 text-emerald-700 border border-emerald-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                  Connected
                </span>
              ) : (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-slate-100 text-slate-500 border border-slate-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                  Not Configured
                </span>
              )}
            </div>

            {/* Live Chat Widget */}
            <div className="p-3.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-50 text-blue-600 font-bold text-xs">
                  LC
                </div>
                <div>
                  <h4 className="text-xs font-semibold text-[var(--color-ink)]">Live Chat Widget</h4>
                  <p className="text-[11px] text-[var(--color-ink-faint)]">Web SDK &amp; Pre-chat</p>
                </div>
              </div>
              {isLcConnected ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-50 text-emerald-700 border border-emerald-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                  Connected
                </span>
              ) : (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-slate-100 text-slate-500 border border-slate-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                  Not Configured
                </span>
              )}
            </div>

            {/* Email */}
            <div className="p-3.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-50 text-amber-600 font-bold text-xs">
                  EM
                </div>
                <div>
                  <h4 className="text-xs font-semibold text-[var(--color-ink)]">Email Relay</h4>
                  <p className="text-[11px] text-[var(--color-ink-faint)]">Inbound &amp; Outbound</p>
                </div>
              </div>
              {isEmailConnected ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-50 text-emerald-700 border border-emerald-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                  Connected
                </span>
              ) : (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-slate-100 text-slate-500 border border-slate-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                  Not Configured
                </span>
              )}
            </div>

            {/* SMS */}
            <div className="p-3.5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-purple-50 text-purple-600 font-bold text-xs">
                  SMS
                </div>
                <div>
                  <h4 className="text-xs font-semibold text-[var(--color-ink)]">SMS Gateway</h4>
                  <p className="text-[11px] text-[var(--color-ink-faint)]">Universal Multi-Provider</p>
                </div>
              </div>
              {isSmsConnected ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-50 text-emerald-700 border border-emerald-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                  Connected
                </span>
              ) : (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-slate-100 text-slate-500 border border-slate-200">
                  <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                  Not Configured
                </span>
              )}
            </div>
          </div>

          {/* Quick Automation & AI Rule Summary */}
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-xs space-y-3">
            <h3 className="text-xs font-semibold text-[var(--color-ink)] uppercase tracking-wider flex items-center gap-1.5">
              <Zap className="w-3.5 h-3.5 text-amber-500" />
              Autonomous Engine Status
            </h3>
            <div className="space-y-2 text-xs text-[var(--color-ink-muted)]">
              <div className="flex justify-between">
                <span>AI Reply Engine</span>
                <span className="font-medium text-emerald-600">Active (Claude 3.5)</span>
              </div>
              <div className="flex justify-between">
                <span>Quiet Hours Guard</span>
                <span className="font-medium text-[var(--color-ink)]">Enforced</span>
              </div>
              <div className="flex justify-between">
                <span>Follow-up Cadence</span>
                <span className="font-medium text-[var(--color-ink)]">Automated</span>
              </div>
            </div>
          </div>
        </section>
      </div>

      {/* Bottom Row: Recent Lead Threads */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-base font-semibold text-[var(--color-ink)] flex items-center gap-2">
              <Users className="w-4 h-4 text-indigo-500" />
              Recent Lead Conversations
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Latest omnichannel interactions across your pipeline
            </p>
          </div>
          <Link
            to="/conversations"
            className="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-accent)] hover:underline"
          >
            All conversations
            <ArrowRight className="w-3.5 h-3.5" />
          </Link>
        </div>

        {conversations === null ? (
          <div className="h-40 animate-pulse rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]" />
        ) : conversations.length === 0 ? (
          <div className="rounded-xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center text-sm text-[var(--color-ink-muted)]">
            No active conversations found. Inbound leads from Harvester will appear here.
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs">
            <table className="w-full text-left text-sm">
              <thead className="border-b border-[var(--color-border)] bg-[var(--color-page)] text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider">
                <tr>
                  <th className="px-4 py-3">Phone Number</th>
                  <th className="px-4 py-3">Customer / Lead Name</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="hidden sm:table-cell px-4 py-3">Last Updated</th>
                  <th className="px-4 py-3 text-right">Action</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[var(--color-border)]">
                {conversations.slice(0, 6).map((c) => (
                  <tr key={c.id} className="hover:bg-[var(--color-page)] transition-colors">
                    <td className="px-4 py-3 font-mono text-xs font-semibold text-slate-800">
                      {c.contactPhone || c.phone || '—'}
                    </td>
                    <td className="px-4 py-3">
                      <Link
                        to={`/conversations/${c.id}`}
                        className="font-semibold text-[var(--color-ink)] hover:text-emerald-700 hover:underline"
                      >
                        {c.contactName || c.customerName || c.leadName || c.contactPhone || 'Customer'}
                      </Link>
                    </td>
                    <td className="px-4 py-3">
                      <span className={`badge ${stateBadgeClass(c.state)}`}>
                        {stateLabel(c.state)}
                      </span>
                    </td>
                    <td className="hidden sm:table-cell px-4 py-3 text-xs text-[var(--color-ink-muted)]">
                      {timeAgo(c.updatedAt)}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <Link
                        to={`/conversations/${c.id}`}
                        className="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-accent)] hover:underline"
                      >
                        Open thread
                        <ArrowRight className="w-3 h-3" />
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  )
}
