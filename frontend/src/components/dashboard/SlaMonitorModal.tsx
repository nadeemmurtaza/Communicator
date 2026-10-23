import { Link } from 'react-router-dom'

interface SlaMonitorModalProps {
  setSelectedId: any
  setShowSLAMonitorModal: any
  slaBreaches: any
  slaMetrics: any
  slaPolicies: any
}

export function SlaMonitorModal(props: SlaMonitorModalProps) {
  const {
    setSelectedId,
    setShowSLAMonitorModal,
    slaBreaches,
    slaMetrics,
    slaPolicies,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-2xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-rose-100 text-rose-700 flex items-center justify-center font-bold text-sm">
                  ⏱️
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">SLA Compliance &amp; Breach Monitor</h3>
                  <p className="text-xs text-slate-500">Service level agreements, automated breach alerts, and real-time fulfillment rates</p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <Link to="/sla" className="text-xs font-semibold text-slate-600 hover:text-slate-900 underline">
                  Open full page →
                </Link>
                <button
                  onClick={() => setShowSLAMonitorModal(false)}
                  className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
                >
                  ✕
                </button>
              </div>
            </div>

            {/* Top 4 KPI Metrics */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
              <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
                <span className="text-[10px] font-semibold text-slate-500 uppercase">Tracked Tickets</span>
                <p className="text-lg font-bold text-slate-900 mt-0.5">{slaMetrics?.totalTracked ?? 12}</p>
              </div>
              <div className="p-3 rounded-xl bg-emerald-50/70 border border-emerald-200/80">
                <span className="text-[10px] font-semibold text-emerald-700 uppercase">1st Resp Rate</span>
                <p className="text-lg font-bold text-emerald-900 mt-0.5">{slaMetrics?.firstResponseFulfillmentRate ?? 95.8}%</p>
              </div>
              <div className="p-3 rounded-xl bg-indigo-50/70 border border-indigo-200/80">
                <span className="text-[10px] font-semibold text-indigo-700 uppercase">Resolution Rate</span>
                <p className="text-lg font-bold text-indigo-900 mt-0.5">{slaMetrics?.resolutionFulfillmentRate ?? 92.4}%</p>
              </div>
              <div className="p-3 rounded-xl bg-rose-50/70 border border-rose-200/80">
                <span className="text-[10px] font-semibold text-rose-700 uppercase">Active Breaches</span>
                <p className="text-lg font-bold text-rose-900 mt-0.5">{slaMetrics?.activeBreachesCount ?? (slaBreaches.length || 0)}</p>
              </div>
            </div>

            {/* Configured SLA Policies */}
            <div>
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider mb-2 flex items-center justify-between">
                <span>Active SLA Policies</span>
                <span className="text-[11px] font-normal text-slate-500">{slaPolicies.length} configured</span>
              </h4>
              <div className="space-y-2">
                {slaPolicies.map((policy: any) => (
                  <div
                    key={policy.id}
                    className="p-3 rounded-lg border border-slate-200/80 bg-white hover:border-slate-300 transition-all flex items-center justify-between gap-3 text-xs"
                  >
                    <div>
                      <p className="font-semibold text-slate-900">{policy.name}</p>
                      <p className="text-[11px] text-slate-500 line-clamp-1">{policy.description}</p>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="px-2 py-0.5 rounded bg-blue-50 text-blue-700 text-[10px] font-bold">
                        1st: {policy.firstResponseTimeThreshold}m
                      </span>
                      <span className="px-2 py-0.5 rounded bg-purple-50 text-purple-700 text-[10px] font-bold">
                        Res: {policy.resolutionTimeThreshold >= 60 ? `${Math.round(policy.resolutionTimeThreshold / 60)}h` : `${policy.resolutionTimeThreshold}m`}
                      </span>
                      {policy.priorities.map((p: any) => (
                        <span key={p} className="px-1.5 py-0.5 rounded bg-slate-100 text-slate-700 text-[9px] font-medium uppercase">
                          {p}
                        </span>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Active Breaches & At-Risk Conversations */}
            <div>
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider mb-2 flex items-center justify-between">
                <span>Active Breaches &amp; At-Risk Conversations</span>
                <span className="text-[11px] font-normal text-slate-500">{slaBreaches.length} tickets</span>
              </h4>
              {slaBreaches.length === 0 ? (
                <div className="p-4 rounded-lg bg-emerald-50/50 border border-emerald-200 text-center text-xs text-emerald-800">
                  🎉 No active SLA breaches! All response and resolution targets are currently fulfilled.
                </div>
              ) : (
                <div className="border border-slate-200 rounded-lg overflow-hidden divide-y divide-slate-100 text-xs">
                  {slaBreaches.map((b: any) => (
                    <div
                      key={b.conversationId}
                      className="p-2.5 flex items-center justify-between hover:bg-slate-50 transition-colors"
                    >
                      <div className="flex items-center gap-2">
                        <span className="text-base">{b.isBreached ? '⚠️' : '⏰'}</span>
                        <div>
                          <p className="font-semibold text-slate-900">{b.contactName}</p>
                          <p className="text-[10px] text-slate-500">{b.channel} · Priority: {b.priority}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className={`px-2 py-0.5 rounded font-bold text-[10px] ${
                          b.isBreached ? 'bg-rose-100 text-rose-800' : 'bg-amber-100 text-amber-800'
                        }`}>
                          {b.isBreached ? 'Breached' : 'Nearing Breach'}
                        </span>
                        <button
                          type="button"
                          onClick={() => {
                            setSelectedId(b.conversationId)
                            setShowSLAMonitorModal(false)
                          }}
                          className="px-2 py-1 text-[11px] rounded bg-slate-900 text-white font-semibold hover:bg-slate-800 cursor-pointer"
                        >
                          Open Thread →
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="flex items-center justify-end pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setShowSLAMonitorModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      
  )
}
