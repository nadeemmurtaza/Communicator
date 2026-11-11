import { useEffect, useState, useMemo } from 'react'
import { api } from '../../lib/api'
import {
  Activity, TrendingUp, Minus, AlertTriangle,
  CheckCircle2, AlertCircle, Info, Shield, MessageSquare,
  BarChart3, RefreshCw, Clock, ArrowUpRight, ArrowDown,
} from '../../components/icons'

// ---------------------------------------------------------------------------
// Quality score color helpers
// ---------------------------------------------------------------------------
function qualityColor(score?: number): string {
  if (score == null) return 'text-[var(--color-ink-faint)]'
  if (score >= 70) return 'text-emerald-600'
  if (score >= 40) return 'text-amber-600'
  return 'text-rose-600'
}

function qualityBg(score?: number): string {
  if (score == null) return 'bg-[var(--color-ink-faint)]'
  if (score >= 70) return 'bg-emerald-500'
  if (score >= 40) return 'bg-amber-500'
  return 'bg-rose-500'
}

function qualityRatingColor(rating?: string): string {
  switch (rating?.toUpperCase()) {
    case 'HIGH': return 'text-emerald-600'
    case 'MEDIUM': return 'text-amber-600'
    case 'LOW': return 'text-rose-600'
    default: return 'text-[var(--color-ink-faint)]'
  }
}

function trendIcon(trend: string) {
  switch (trend) {
    case 'improving': return <ArrowUpRight className="h-4 w-4 text-emerald-600" />
    case 'declining': return <ArrowDown className="h-4 w-4 text-rose-600" />
    default: return <Minus className="h-4 w-4 text-[var(--color-ink-faint)]" />
  }
}

function trendLabel(trend: string): string {
  switch (trend) {
    case 'improving': return 'Improving'
    case 'declining': return 'Declining'
    case 'stable': return 'Stable'
    default: return 'Insufficient Data'
  }
}

// ---------------------------------------------------------------------------
// Mini bar chart component (pure CSS, no dependencies)
// ---------------------------------------------------------------------------
function QualityChart({ data }: { data: Array<{ score?: number; date?: string }> }) {
  const maxScore = 100
  const scores = data.map(d => d.score ?? 0)
  const max = Math.max(maxScore, ...scores)

  if (data.length === 0) {
    return (
      <div className="flex h-48 items-center justify-center text-sm text-[var(--color-ink-faint)]">
        No quality data available yet. Refresh the health check to begin tracking.
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {/* Chart area */}
      <div className="relative h-48 flex items-end gap-px">
        {/* Grid lines */}
        <div className="absolute inset-0 flex flex-col justify-between pointer-events-none">
          {[100, 75, 50, 25, 0].map((val) => (
            <div key={val} className="border-b border-[var(--color-border)]/50 h-0" />
          ))}
        </div>

        {/* Bars */}
        <div className="relative flex items-end gap-px flex-1 h-full">
          {data.map((d, i) => {
            const height = max > 0 ? ((d.score ?? 0) / max) * 100 : 0
            const color = d.score != null
              ? d.score >= 70 ? 'bg-emerald-400' : d.score >= 40 ? 'bg-amber-400' : 'bg-rose-400'
              : 'bg-[var(--color-ink-faint)]'

            return (
              <div
                key={i}
                className={`flex-1 rounded-t transition-all hover:opacity-80 cursor-pointer relative group ${color}`}
                style={{ height: `${Math.max(height, 2)}%` }}
                title={`${d.date || `Day ${i + 1}`}: Score ${d.score ?? 'N/A'}`}
              >
                {/* Tooltip */}
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-gray-900 text-white text-[10px] rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-10">
                  {d.date || `Day ${i + 1}`}: {d.score ?? 'N/A'}/100
                </div>
              </div>
            )
          })}
        </div>

        {/* Y-axis labels */}
        <div className="absolute left-0 top-0 bottom-0 flex flex-col justify-between text-[9px] text-[var(--color-ink-faint)] pointer-events-none -ml-1 -ml-8 pr-1">
          <span>100</span>
          <span>75</span>
          <span>50</span>
          <span>25</span>
          <span>0</span>
        </div>
      </div>

      {/* Legend */}
      <div className="flex items-center gap-4 text-[10px] text-[var(--color-ink-faint)]">
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-emerald-400"></span> High (70+)</span>
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-amber-400"></span> Medium (40-69)</span>
        <span className="flex items-center gap-1"><span className="w-2 h-2 rounded-full bg-rose-400"></span> Low (&lt;40)</span>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Metric card
// ---------------------------------------------------------------------------
function MetricCard({
  title, value, subtitle, icon, colorClass,
}: {
  title: string
  value: string | number
  subtitle?: string
  icon: React.ReactNode
  colorClass: string
}) {
  return (
    <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-4 shadow-none">
      <div className="flex items-center justify-between mb-2">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-faint)]">{title}</span>
        <div className={`p-1.5 rounded-[var(--radius-md)] ${colorClass}`}>{icon}</div>
      </div>
      <p className="text-2xl font-bold text-[var(--color-ink)]">{value}</p>
      {subtitle && <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">{subtitle}</p>}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Recommendation card
// ---------------------------------------------------------------------------
function RecommendationCard({ rec }: { rec: { type?: string; severity?: string; title?: string; description?: string; createdAt?: string } }) {
  const severityColor = rec.severity === 'critical'
    ? 'border-rose-200 bg-rose-50/50'
    : rec.severity === 'warning'
    ? 'border-amber-200 bg-amber-50/50'
    : 'border-blue-200 bg-blue-50/50'

  const severityIcon = rec.severity === 'critical'
    ? <AlertCircle className="h-4 w-4 text-rose-600" />
    : rec.severity === 'warning'
    ? <AlertTriangle className="h-4 w-4 text-amber-600" />
    : <Info className="h-4 w-4 text-blue-600" />

  const typeIcon = rec.type === 'spam'
    ? <Shield className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
    : rec.type === 'messaging_limit'
    ? <MessageSquare className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
    : <Activity className="h-3.5 w-3.5 text-[var(--color-ink-faint)]" />

  return (
    <div className={`rounded-[var(--radius-md)] border p-3.5 ${severityColor}`}>
      <div className="flex items-start gap-2.5">
        <div className="mt-0.5">{severityIcon}</div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs font-semibold text-[var(--color-ink)]">{rec.title || 'Recommendation'}</span>
            <span className="flex items-center gap-1 text-[10px] text-[var(--color-ink-faint)]">
              {typeIcon}
              {rec.type || 'general'}
            </span>
          </div>
          <p className="text-xs text-[var(--color-ink-muted)] leading-relaxed">{rec.description}</p>
          {rec.createdAt && (
            <p className="text-[10px] text-[var(--color-ink-faint)] mt-1.5 flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {new Date(rec.createdAt).toLocaleDateString()}
            </p>
          )}
        </div>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------
export function WhatsAppQualityDashboard({ brandId }: { brandId: string }) {
  const [loading, setLoading] = useState(true)
  const [data, setData] = useState<any>(null)
  const [error, setError] = useState<string | null>(null)

  async function load() {
    try {
      setLoading(true)
      setError(null)
      const result = await api.getWhatsAppQualityHistory(brandId)
      setData(result)
    } catch (e: any) {
      setError(e.message || 'Failed to load quality history')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [brandId])

  // Process history data for chart
  const chartData = useMemo(() => {
    if (!data?.history) return []
    return data.history.map((h: any) => ({
      score: h.qualityScore,
      date: h.createdAt ? new Date(h.createdAt).toLocaleDateString('en-US', { month: 'short', day: 'numeric' }) : undefined,
    }))
  }, [data])

  // Collect all block reasons
  const allBlockReasons = useMemo(() => {
    if (!data?.history) return []
    const reasons = new Set<string>()
    data.history.forEach((h: any) => {
      h.blockReasons?.forEach((r: string) => reasons.add(r))
    })
    return Array.from(reasons)
  }, [data])

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="text-center">
          <Activity className="h-8 w-8 text-[var(--color-accent)] animate-pulse mx-auto" />
          <p className="mt-2 text-xs text-[var(--color-ink-faint)]">Loading quality history...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="rounded-[var(--radius-lg)] border border-red-200 bg-red-50/50 p-6 text-center">
        <AlertCircle className="h-8 w-8 text-red-500 mx-auto mb-2" />
        <p className="text-sm text-red-700">{error}</p>
        <button onClick={load} className="mt-3 text-xs text-red-600 hover:underline">Retry</button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
            <BarChart3 className="h-5 w-5 text-[var(--color-accent)]" />
            30-Day Quality Dashboard
          </h2>
          <p className="text-xs text-[var(--color-ink-faint)] mt-0.5">
            Track your WhatsApp Business API quality rating, spam flags, and messaging limits over time.
          </p>
        </div>
        <button
          onClick={load}
          className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-white px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] flex items-center gap-1.5 cursor-pointer"
        >
          <RefreshCw className="h-3.5 w-3.5" />
          Refresh
        </button>
      </div>

      {/* Metric Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <MetricCard
          title="Current Score"
          value={data?.currentScore ?? 'N/A'}
          subtitle={`Rating: ${data?.currentRating ?? 'N/A'}`}
          icon={<Activity className="h-4 w-4" />}
          colorClass={`${qualityBg(data?.currentScore)} bg-opacity-10`}
        />
        <MetricCard
          title="Average Score"
          value={data?.averageScore ?? 0}
          subtitle={`${data?.totalSnapshots ?? 0} data points`}
          icon={<BarChart3 className="h-4 w-4" />}
          colorClass="bg-blue-50"
        />
        <MetricCard
          title="Spam Flags"
          value={data?.currentSpamFlags ?? 0}
          subtitle="Last 14 days"
          icon={<Shield className="h-4 w-4" />}
          colorClass={(data?.currentSpamFlags ?? 0) > 10 ? 'bg-rose-50' : (data?.currentSpamFlags ?? 0) > 0 ? 'bg-amber-50' : 'bg-emerald-50'}
        />
        <MetricCard
          title="Messaging Tier"
          value={data?.currentTier ?? 'N/A'}
          subtitle={`Trend: ${trendLabel(data?.trend ?? '')}`}
          icon={<MessageSquare className="h-4 w-4" />}
          colorClass="bg-purple-50"
        />
      </div>

      {/* Trend indicator */}
      <div className={`rounded-[var(--radius-lg)] border p-4 flex items-center gap-3 ${
        data?.trend === 'improving' ? 'border-emerald-200 bg-emerald-50/50' :
        data?.trend === 'declining' ? 'border-rose-200 bg-rose-50/50' :
        'border-[var(--color-border)] bg-white'
      }`}>
        <div className="p-2 rounded-[var(--radius-md)] bg-white border border-[var(--color-border)]">
          {trendIcon(data?.trend ?? '')}
        </div>
        <div>
          <p className="text-sm font-semibold text-[var(--color-ink)]">
            Quality Trend: {trendLabel(data?.trend ?? '')}
          </p>
          <p className="text-xs text-[var(--color-ink-faint)]">
            {data?.trend === 'improving' && 'Your quality score is improving compared to the previous period.'}
            {data?.trend === 'declining' && 'Your quality score is declining. Review your messaging practices.'}
            {data?.trend === 'stable' && 'Your quality score remains stable. Continue current practices.'}
            {data?.trend === 'insufficient_data' && 'Not enough data points yet. Refresh health checks to build history.'}
          </p>
        </div>
      </div>

      {/* Quality Score Chart */}
      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5 shadow-none">
        <h3 className="text-sm font-bold text-[var(--color-ink)] mb-4 flex items-center gap-2">
          <BarChart3 className="h-4 w-4 text-[var(--color-accent)]" />
          Quality Score History
        </h3>
        <QualityChart data={chartData} />
      </div>

      {/* Two-column: Block Reasons + Recommendations */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Block Reasons */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5 shadow-none">
          <h3 className="text-sm font-bold text-[var(--color-ink)] mb-3 flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 text-amber-600" />
            Block Reasons
            {allBlockReasons.length > 0 && (
              <span className="ml-auto rounded-full bg-amber-100 text-amber-800 px-2 py-0.5 text-[10px] font-semibold">
                {allBlockReasons.length} detected
              </span>
            )}
          </h3>
          {allBlockReasons.length === 0 ? (
            <div className="flex items-center gap-2 text-xs text-emerald-600 py-4">
              <CheckCircle2 className="h-4 w-4" />
              No block reasons detected — great job!
            </div>
          ) : (
            <div className="space-y-2">
              {allBlockReasons.map((reason, i) => (
                <div key={i} className="flex items-start gap-2 rounded-[var(--radius-md)] border border-amber-200 bg-amber-50/50 p-3">
                  <AlertTriangle className="h-4 w-4 text-amber-600 mt-0.5 shrink-0" />
                  <div>
                    <p className="text-xs font-medium text-[var(--color-ink)]">{reason}</p>
                    <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">
                      Users have blocked messages for this reason. Address it to improve quality.
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Recommendations */}
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-5 shadow-none">
          <h3 className="text-sm font-bold text-[var(--color-ink)] mb-3 flex items-center gap-2">
            <Info className="h-4 w-4 text-blue-600" />
            Recommendations
            {(data?.recommendations?.length ?? 0) > 0 && (
              <span className="ml-auto rounded-full bg-blue-100 text-blue-800 px-2 py-0.5 text-[10px] font-semibold">
                {data.recommendations.length} active
              </span>
            )}
          </h3>
          {(data?.recommendations?.length ?? 0) === 0 ? (
            <div className="flex items-center gap-2 text-xs text-emerald-600 py-4">
              <CheckCircle2 className="h-4 w-4" />
              No active recommendations — everything looks good!
            </div>
          ) : (
            <div className="space-y-2.5 max-h-80 overflow-y-auto">
              {data.recommendations.map((rec: any, i: number) => (
                <RecommendationCard key={i} rec={rec} />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Info footer */}
      <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-muted)]/40 p-4 text-xs text-[var(--color-ink-muted)]">
        <p className="flex items-start gap-2">
          <Info className="h-4 w-4 text-[var(--color-accent)] mt-0.5 shrink-0" />
          <span>
            Quality data is recorded each time the WhatsApp health check runs. To build a complete 30-day history,
            ensure health checks are performed regularly. Meta evaluates quality based on user feedback (spam reports, blocks)
            over a rolling 14-day window. Maintain HIGH quality for 30 days to upgrade your messaging tier.
          </span>
        </p>
      </div>
    </div>
  )
}
