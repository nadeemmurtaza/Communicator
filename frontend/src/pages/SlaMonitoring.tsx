import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { api } from '../lib/api'

export function SlaMonitoring() {
  const [policies, setPolicies] = useState<any[]>([])
  const [metrics, setMetrics] = useState<any>(null)
  const [breaches, setBreaches] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const navigate = useNavigate()

  useEffect(() => {
    Promise.all([api.listSLAPolicies(), api.getSLAMetrics(), api.getSLABreaches()])
      .then(([p, m, b]) => {
        setPolicies(p)
        setMetrics(m)
        setBreaches(b)
      })
      .finally(() => setLoading(false))
  }, [])

  if (loading) return <p className="text-sm text-[var(--color-ink-muted)]">Loading SLA data...</p>

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-[var(--color-ink)]">SLA Compliance &amp; Breach Monitor</h1>
        <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
          Service level agreements, automated breach alerts, and real-time fulfillment rates.
        </p>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Tracked Tickets</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{metrics?.totalTracked ?? 0}</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">1st Resp Rate</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{metrics?.firstResponseFulfillmentRate ?? 0}%</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Resolution Rate</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{metrics?.resolutionFulfillmentRate ?? 0}%</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Active Breaches</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{metrics?.activeBreachesCount ?? breaches.length}</p>
        </div>
      </div>

      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3 flex items-center justify-between">
          <span>Active SLA Policies</span>
          <span className="text-xs font-normal text-[var(--color-ink-faint)]">{policies.length} configured</span>
        </h2>
        <div className="space-y-2">
          {policies.map((policy) => (
            <div
              key={policy.id}
              className="p-3 rounded-[var(--radius-sm)] border border-[var(--color-border)] flex items-center justify-between gap-3 text-sm"
            >
              <div>
                <p className="font-medium text-[var(--color-ink)]">{policy.name}</p>
                <p className="text-xs text-[var(--color-ink-muted)]">{policy.description}</p>
              </div>
              <div className="flex items-center gap-2 shrink-0 text-xs">
                <span className="px-2 py-0.5 rounded bg-[var(--color-accent-soft)] text-[var(--color-accent)] font-semibold">
                  1st: {policy.firstResponseTimeThreshold}m
                </span>
                <span className="px-2 py-0.5 rounded bg-[var(--color-accent-soft)] text-[var(--color-accent)] font-semibold">
                  Res: {policy.resolutionTimeThreshold >= 60 ? `${Math.round(policy.resolutionTimeThreshold / 60)}h` : `${policy.resolutionTimeThreshold}m`}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3 flex items-center justify-between">
          <span>Active Breaches &amp; At-Risk Conversations</span>
          <span className="text-xs font-normal text-[var(--color-ink-faint)]">{breaches.length} tickets</span>
        </h2>
        {breaches.length === 0 ? (
          <div className="p-4 rounded-[var(--radius-sm)] bg-[var(--color-accent-soft)] text-center text-sm text-[var(--color-ink)]">
            No active SLA breaches. All response and resolution targets are currently fulfilled.
          </div>
        ) : (
          <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)] text-sm">
            {breaches.map((b) => (
              <div key={b.conversationId} className="p-3 flex items-center justify-between hover:bg-black/5 transition-colors">
                <div>
                  <p className="font-medium text-[var(--color-ink)]">{b.contactName}</p>
                  <p className="text-xs text-[var(--color-ink-muted)]">{b.channel} · Priority: {b.priority}</p>
                </div>
                <div className="flex items-center gap-2">
                  <span className={`px-2 py-0.5 rounded text-xs font-semibold ${b.isBreached ? 'bg-red-100 text-red-700' : 'bg-amber-100 text-amber-700'}`}>
                    {b.isBreached ? 'Breached' : 'Nearing Breach'}
                  </span>
                  <button
                    type="button"
                    onClick={() => navigate('/')}
                    className="px-3 py-1 text-xs rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold hover:bg-[var(--color-accent-hover)]"
                  >
                    Open Inbox →
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
