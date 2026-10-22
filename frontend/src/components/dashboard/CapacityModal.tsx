import { Link } from 'react-router-dom'
import { api } from '../../lib/api'

interface CapacityModalProps {
  api: any
  capacityOverview: any
  handleBatchAutoAssign: any
  handleUpdateCapacity: any
  routingAgents: any
  setCapacityOverview: any
  setRoutingAgents: any
  setShowCapacityModal: any
}

export function CapacityModal(props: CapacityModalProps) {
  const {
    api,
    capacityOverview,
    handleBatchAutoAssign,
    handleUpdateCapacity,
    routingAgents,
    setCapacityOverview,
    setRoutingAgents,
    setShowCapacityModal,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-3xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-teal-100 text-teal-700 flex items-center justify-center font-bold text-sm">
                  👥
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Team Routing &amp; Capacity Manager</h3>
                  <p className="text-xs text-slate-500">Live agent availability, concurrent capacity limits, and workload distribution</p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <Link to="/routing-capacity" className="text-xs font-semibold text-slate-600 hover:text-slate-900 underline">
                  Open full page →
                </Link>
                <button
                  onClick={() => setShowCapacityModal(false)}
                  className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
                >
                  ✕
                </button>
              </div>
            </div>

            {/* Top 5 KPI Metrics */}
            <div className="grid grid-cols-2 sm:grid-cols-5 gap-3">
              <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
                <span className="text-[10px] font-semibold text-slate-500 uppercase">Total Agents</span>
                <p className="text-lg font-bold text-slate-900 mt-0.5">{capacityOverview?.totalAgents ?? routingAgents.length}</p>
              </div>
              <div className="p-3 rounded-xl bg-emerald-50/70 border border-emerald-200/80">
                <span className="text-[10px] font-semibold text-emerald-700 uppercase">Online Agents</span>
                <p className="text-lg font-bold text-emerald-900 mt-0.5">{capacityOverview?.onlineAgents ?? routingAgents.filter(a => a.availabilityStatus === 'online').length}</p>
              </div>
              <div className="p-3 rounded-xl bg-blue-50/70 border border-blue-200/80">
                <span className="text-[10px] font-semibold text-blue-700 uppercase">Total Slots</span>
                <p className="text-lg font-bold text-blue-900 mt-0.5">{capacityOverview?.totalMaxCapacity ?? 18}</p>
              </div>
              <div className="p-3 rounded-xl bg-purple-50/70 border border-purple-200/80">
                <span className="text-[10px] font-semibold text-purple-700 uppercase">Open Slots</span>
                <p className="text-lg font-bold text-purple-900 mt-0.5">{capacityOverview?.availableOnlineSlots ?? 12}</p>
              </div>
              <div className="p-3 rounded-xl bg-amber-50/70 border border-amber-200/80">
                <span className="text-[10px] font-semibold text-amber-700 uppercase">Load %</span>
                <p className="text-lg font-bold text-amber-900 mt-0.5">{capacityOverview?.utilizationRatePct ?? 25.0}%</p>
              </div>
            </div>

            {/* Batch Routing Action Bar */}
            <div className="p-3 rounded-xl bg-gradient-to-r from-teal-50 to-emerald-50 border border-teal-200 flex items-center justify-between gap-3">
              <div>
                <p className="font-bold text-xs text-teal-950">Auto-Route Unassigned Conversations</p>
                <p className="text-[11px] text-teal-700">Evenly distributes all unassigned open tickets across online agents</p>
              </div>
              <button
                type="button"
                onClick={handleBatchAutoAssign}
                className="px-3.5 py-1.5 rounded-lg bg-teal-800 hover:bg-teal-900 text-white font-semibold text-xs shadow-xs transition-colors cursor-pointer"
              >
                ⚡ Run Batch Auto-Assign
              </button>
            </div>

            {/* Agents Table */}
            <div>
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider mb-2 flex items-center justify-between">
                <span>Agent Availability &amp; Capacity Limits</span>
                <span className="text-[11px] font-normal text-slate-500">{routingAgents.length} agents</span>
              </h4>
              <div className="border border-slate-200 rounded-lg overflow-hidden divide-y divide-slate-100 text-xs">
                {routingAgents.map((ag) => (
                  <div key={ag.id} className="p-3 flex items-center justify-between hover:bg-slate-50 transition-colors">
                    <div className="flex items-center gap-3">
                      <div className="relative">
                        <div className="w-8 h-8 rounded-full bg-slate-200 text-slate-700 flex items-center justify-center font-bold text-xs uppercase">
                          {ag.name.slice(0, 2)}
                        </div>
                        <span
                          className={`absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full ring-2 ring-white ${
                            ag.availabilityStatus === 'online'
                              ? 'bg-emerald-500'
                              : ag.availabilityStatus === 'busy'
                              ? 'bg-amber-500'
                              : 'bg-slate-400'
                          }`}
                        />
                      </div>
                      <div>
                        <p className="font-semibold text-slate-900">{ag.name}</p>
                        <p className="text-[11px] text-slate-500">{ag.email}</p>
                      </div>
                    </div>

                    <div className="flex items-center gap-4">
                      {/* Status Selector */}
                      <div>
                        <label className="block text-[9px] text-slate-400 font-semibold uppercase mb-0.5">Status</label>
                        <select
                          value={ag.availabilityStatus}
                          onChange={async (e) => {
                            const newSt = e.target.value as any
                            await api.updateAgentStatus(ag.id, newSt)
                            const [updated, ov] = await Promise.all([
                              api.listRoutingAgents(),
                              api.getCapacityOverview(),
                            ])
                            setRoutingAgents(updated)
                            setCapacityOverview(ov)
                          }}
                          className="text-xs border border-slate-300 rounded px-2 py-1 bg-white focus:outline-hidden cursor-pointer"
                        >
                          <option value="online">🟢 Online</option>
                          <option value="busy">🟡 Busy</option>
                          <option value="offline">⚪ Offline</option>
                        </select>
                      </div>

                      {/* Capacity Limit input */}
                      <div>
                        <label className="block text-[9px] text-slate-400 font-semibold uppercase mb-0.5">Max Slots</label>
                        <input
                          type="number"
                          min={1}
                          max={50}
                          defaultValue={ag.maxCapacity}
                          onBlur={(e) => {
                            const val = parseInt(e.target.value, 10)
                            if (val > 0 && val !== ag.maxCapacity) {
                              handleUpdateCapacity(ag.id, val)
                            }
                          }}
                          className="w-16 text-xs border border-slate-300 rounded px-2 py-1 text-center bg-white focus:outline-hidden"
                        />
                      </div>

                      {/* Active Load & Bar */}
                      <div className="w-28 text-right">
                        <span className="text-[11px] font-semibold text-slate-800">
                          {ag.activeConversationsCount}/{ag.maxCapacity} active
                        </span>
                        <div className="w-full bg-slate-200 h-1.5 rounded-full mt-1 overflow-hidden">
                          <div
                            className={`h-full rounded-full ${
                              ag.utilizationRatePct >= 100
                                ? 'bg-rose-500'
                                : ag.utilizationRatePct >= 70
                                ? 'bg-amber-500'
                                : 'bg-emerald-500'
                            }`}
                            style={{ width: `${Math.min(100, ag.utilizationRatePct)}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-end pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setShowCapacityModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      
  )
}
