import { Link } from 'react-router-dom'
import { api } from '../../lib/api'

interface ApprovalsModalProps {
  approvalFilterStatus: any
  copilotApprovals: any
  handleApproveRequest: any
  handleRejectRequest: any
  handleTriggerRefundApprovalDemo: any
  setApprovalFilterStatus: any
  setShowApprovalsModal: any
}

export function ApprovalsModal(props: ApprovalsModalProps) {
  const {
    approvalFilterStatus,
    copilotApprovals,
    handleApproveRequest,
    handleRejectRequest,
    handleTriggerRefundApprovalDemo,
    setApprovalFilterStatus,
    setShowApprovalsModal,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-4xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center font-bold text-sm">
                  🤝
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Copilot Teammate Approvals Center</h3>
                  <p className="text-xs text-slate-500">
                    Agent and your team work from the same inbox and record. When Agent needs approval or info, it asks a teammate and finishes the conversation itself.
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <Link to="/approvals" className="text-xs font-semibold text-slate-600 hover:text-slate-900 underline">
                  Open full page →
                </Link>
                <button
                  onClick={() => setShowApprovalsModal(false)}
                  className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
                >
                  ✕
                </button>
              </div>
            </div>

            {/* Filter Tabs */}
            <div className="flex items-center gap-2 border-b border-slate-100 pb-2">
              {['all', 'pending', 'approved', 'rejected'].map((st) => {
                const count = st === 'all'
                  ? copilotApprovals.length
                  : copilotApprovals.filter((a) => a.status === st).length
                return (
                  <button
                    key={st}
                    type="button"
                    onClick={() => setApprovalFilterStatus(st)}
                    className={`px-3 py-1.5 rounded-lg text-xs font-semibold capitalize transition-colors flex items-center gap-1.5 cursor-pointer ${
                      approvalFilterStatus === st
                        ? 'bg-amber-600 text-white shadow-xs'
                        : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                    }`}
                  >
                    <span>{st}</span>
                    <span className={`text-[10px] px-1.5 py-0.2 rounded-full ${
                      approvalFilterStatus === st ? 'bg-amber-700 text-white' : 'bg-slate-200 text-slate-700'
                    }`}>
                      {count}
                    </span>
                  </button>
                )
              })}

              <button
                type="button"
                onClick={handleTriggerRefundApprovalDemo}
                className="ml-auto text-xs font-bold text-amber-700 bg-amber-50 hover:bg-amber-100 border border-amber-300 px-3 py-1.5 rounded-lg flex items-center gap-1 cursor-pointer transition-colors"
              >
                <span>➕</span>
                <span>Simulate AI Refund Request</span>
              </button>
            </div>

            {/* Approvals List */}
            <div className="space-y-3">
              {copilotApprovals
                .filter((a) => approvalFilterStatus === 'all' || a.status === approvalFilterStatus)
                .map((app) => (
                  <div
                    key={app.id}
                    className={`rounded-xl border p-4 transition-all ${
                      app.status === 'pending'
                        ? 'border-amber-300 bg-amber-50/40 shadow-xs'
                        : app.status === 'approved' || app.status === 'amended'
                        ? 'border-emerald-200 bg-emerald-50/20'
                        : 'border-rose-200 bg-rose-50/20'
                    }`}
                  >
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-100 pb-2.5">
                      <div className="flex items-center gap-2">
                        <span className="text-base">
                          {app.status === 'pending' ? '⏳' : app.status === 'approved' || app.status === 'amended' ? '✅' : '❌'}
                        </span>
                        <div>
                          <h4 className="font-bold text-slate-900 text-xs">{app.title}</h4>
                          <span className="text-[11px] text-slate-500 font-mono">
                            Conversation ID: {app.conversationId || app.conversation_id}
                          </span>
                        </div>
                      </div>

                      <div className="flex items-center gap-2">
                        <span className="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-slate-100 text-slate-700 border border-slate-200">
                          {app.requestType || app.request_type}
                        </span>
                        <span
                          className={`text-[10px] font-black uppercase tracking-wider px-2 py-0.5 rounded-full ${
                            app.status === 'pending'
                              ? 'bg-amber-200 text-amber-900 border border-amber-300'
                              : app.status === 'approved' || app.status === 'amended'
                              ? 'bg-emerald-100 text-emerald-800 border border-emerald-300'
                              : 'bg-rose-100 text-rose-800 border border-rose-300'
                          }`}
                        >
                          {app.status}
                        </span>
                      </div>
                    </div>

                    <div className="mt-2.5 space-y-2 text-xs">
                      <p className="text-slate-600 bg-white/70 p-2 rounded-lg border border-slate-200/60 font-medium">
                        <strong className="text-slate-700">AI Context: </strong>
                        {app.internalNote || app.internal_note}
                      </p>

                      {app.actionData && Object.keys(app.actionData).length > 0 && (
                        <div className="flex flex-wrap items-center gap-2 text-[11px]">
                          {app.actionData.amount && (
                            <span className="bg-amber-100 text-amber-900 font-bold px-2 py-0.5 rounded-md">
                              Amount: ${Number(app.actionData.amount).toFixed(2)}
                            </span>
                          )}
                          {app.actionData.order_id && (
                            <span className="bg-slate-100 text-slate-800 font-mono px-2 py-0.5 rounded-md">
                              Order #{app.actionData.order_id}
                            </span>
                          )}
                          {app.actionData.policy_rule && (
                            <span className="text-slate-500 italic">
                              Policy: {app.actionData.policy_rule}
                            </span>
                          )}
                        </div>
                      )}

                      {/* Proposed reply display */}
                      <div className="bg-white rounded-lg p-2.5 border border-slate-200">
                        <span className="block text-[10px] font-bold uppercase text-slate-400 mb-1">
                          {app.status === 'pending' ? 'Proposed Customer Reply:' : 'Final Dispatched Reply:'}
                        </span>
                        <p className="text-slate-800 leading-relaxed font-normal">
                          "{app.amendedReply || app.amended_reply || app.proposedReply || app.proposed_reply}"
                        </p>
                      </div>

                      {/* Review details if resolved */}
                      {app.reviewerName && (
                        <div className="text-[11px] text-slate-500 pt-1">
                          Reviewed by <strong className="text-slate-700">{app.reviewerName}</strong>
                          {app.reviewerFeedback && ` • Feedback: "${app.reviewerFeedback}"`}
                        </div>
                      )}

                      {/* Actions for pending items */}
                      {app.status === 'pending' && (
                        <div className="flex items-center justify-end gap-2 pt-2 border-t border-amber-200/60">
                          <button
                            type="button"
                            onClick={() => handleRejectRequest(app.id, 'Declined by teammate')}
                            className="text-xs text-rose-600 hover:text-rose-800 font-semibold px-2.5 py-1 rounded hover:bg-rose-50"
                          >
                            Reject
                          </button>
                          <button
                            type="button"
                            onClick={() => handleApproveRequest(app)}
                            className="bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs px-3.5 py-1.5 rounded-lg shadow-xs cursor-pointer transition-colors"
                          >
                            Approve &amp; Let AI Dispatch Reply
                          </button>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
            </div>

            <div className="flex items-center justify-between pt-2 border-t border-slate-100 text-xs text-slate-500">
              <span>🤖 Once approved, the AI Agent dispatches the message directly to the customer and finalizes the ticket.</span>
              <button
                type="button"
                onClick={() => setShowApprovalsModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      
  )
}
