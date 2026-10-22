import { useEffect, useState } from 'react'
import { api } from '../lib/api'

export function RoutingCapacity() {
  const [agents, setAgents] = useState<any[]>([])
  const [overview, setOverview] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [toast, setToast] = useState<string | null>(null)

  async function refresh() {
    const [a, o] = await Promise.all([api.listRoutingAgents(), api.getCapacityOverview()])
    setAgents(a)
    setOverview(o)
  }

  useEffect(() => {
    refresh().finally(() => setLoading(false))
  }, [])

  async function handleUpdateCapacity(agentId: string, cap: number) {
    try {
      await api.updateAgentCapacity(agentId, cap)
      await refresh()
    } catch (err) {
      console.error('Failed to update capacity:', err)
    }
  }

  async function handleBatchAutoAssign() {
    try {
      const res = await api.batchAutoAssign('capacity_based')
      setToast(`Batch Routing Complete: ${res.assignedCount} assigned, ${res.overflowCount} overflow tickets`)
      setTimeout(() => setToast(null), 5000)
      await refresh()
    } catch (err) {
      console.error('Failed batch auto-assign:', err)
    }
  }

  if (loading) return <p className="text-sm text-[var(--color-ink-muted)]">Loading routing data...</p>

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-[var(--color-ink)]">Team Routing &amp; Capacity</h1>
        <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
          Live agent availability, concurrent capacity limits, and workload distribution.
        </p>
      </div>

      {toast && (
        <div className="glass-card p-3 text-sm text-[var(--color-ink)]">{toast}</div>
      )}

      <div className="grid grid-cols-2 sm:grid-cols-5 gap-4">
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Total Agents</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{overview?.totalAgents ?? agents.length}</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Online</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{overview?.onlineAgents ?? agents.filter((a) => a.availabilityStatus === 'online').length}</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Total Slots</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{overview?.totalMaxCapacity ?? 0}</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Open Slots</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{overview?.availableOnlineSlots ?? 0}</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Load %</span>
          <p className="text-lg font-bold text-[var(--color-ink)] mt-0.5">{overview?.utilizationRatePct ?? 0}%</p>
        </div>
      </div>

      <div className="glass-card p-4 flex items-center justify-between gap-3">
        <div>
          <p className="font-semibold text-sm text-[var(--color-ink)]">Auto-Route Unassigned Conversations</p>
          <p className="text-xs text-[var(--color-ink-muted)]">Evenly distributes all unassigned open tickets across online agents</p>
        </div>
        <button
          type="button"
          onClick={handleBatchAutoAssign}
          className="px-4 py-2 rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold text-sm hover:bg-[var(--color-accent-hover)]"
        >
          Run Batch Auto-Assign
        </button>
      </div>

      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3 flex items-center justify-between">
          <span>Agent Availability &amp; Capacity Limits</span>
          <span className="text-xs font-normal text-[var(--color-ink-faint)]">{agents.length} agents</span>
        </h2>
        <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)] text-sm">
          {agents.map((ag) => (
            <div key={ag.id} className="p-3 flex items-center justify-between hover:bg-black/5 transition-colors">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-full bg-[var(--color-accent-soft)] text-[var(--color-accent)] flex items-center justify-center font-bold text-xs uppercase">
                  {ag.name.slice(0, 2)}
                </div>
                <div>
                  <p className="font-medium text-[var(--color-ink)]">{ag.name}</p>
                  <p className="text-xs text-[var(--color-ink-muted)]">{ag.email}</p>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <select
                  value={ag.availabilityStatus}
                  onChange={async (e) => {
                    await api.updateAgentStatus(ag.id, e.target.value as any)
                    await refresh()
                  }}
                  className="text-xs border border-[var(--color-border)] rounded-[var(--radius-sm)] px-2 py-1 bg-[var(--color-surface)]"
                >
                  <option value="online">Online</option>
                  <option value="busy">Busy</option>
                  <option value="offline">Offline</option>
                </select>
                <input
                  type="number"
                  min={1}
                  max={50}
                  defaultValue={ag.maxCapacity}
                  onBlur={(e) => {
                    const val = parseInt(e.target.value, 10)
                    if (val > 0 && val !== ag.maxCapacity) handleUpdateCapacity(ag.id, val)
                  }}
                  className="w-16 text-xs border border-[var(--color-border)] rounded-[var(--radius-sm)] px-2 py-1 text-center bg-[var(--color-surface)]"
                />
                <span className="text-xs font-medium text-[var(--color-ink)] w-20 text-right">
                  {ag.activeConversationsCount}/{ag.maxCapacity} active
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
