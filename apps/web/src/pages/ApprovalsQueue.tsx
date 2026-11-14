import { useEffect, useState } from 'react'
import {
  AlertCircle,
  CheckCircle2,
  Clock,
  Edit3,
  HelpCircle,
  MessageSquare,
  Send,
  ShieldAlert,
  XCircle,
} from '../components/icons'
import { api, type CopilotApprovalRequest } from '../lib/api'

export function ApprovalsQueue() {
  const [approvals, setApprovals] = useState<CopilotApprovalRequest[]>([])
  const [loading, setLoading] = useState(true)
  const [statusFilter, setStatusFilter] = useState('all')
  const [processingId, setProcessingId] = useState<string | null>(null)

  // In-line editing & answers state
  const [editingReplyId, setEditingReplyId] = useState<string | null>(null)
  const [amendedReplies, setAmendedReplies] = useState<Record<string, string>>({})
  const [teammateAnswers, setTeammateAnswers] = useState<Record<string, string>>({})
  const [rejectFeedback, setRejectFeedback] = useState<Record<string, string>>({})
  const [showRejectInputId, setShowRejectInputId] = useState<string | null>(null)

  function refresh() {
    api
      .listCopilotApprovals(statusFilter === 'all' ? undefined : { status: statusFilter })
      .then((data) => setApprovals(data || []))
      .catch((err) => console.error('Failed to load approvals queue', err))
      .finally(() => setLoading(false))
  }

  useEffect(refresh, [statusFilter])

  async function handleApprove(app: CopilotApprovalRequest) {
    setProcessingId(app.id)
    try {
      const amended = amendedReplies[app.id] || app.amendedReply || app.amended_reply
      await api.approveCopilotRequest(app.id, {
        reviewer_name: 'Riley Brooks',
        amended_reply: amended || undefined,
        execute_send: true,
      })
      setEditingReplyId(null)
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to approve request')
    } finally {
      setProcessingId(null)
    }
  }

  async function handleReject(id: string) {
    setProcessingId(id)
    try {
      const feedback = rejectFeedback[id] || 'Rejected from approvals queue'
      await api.rejectCopilotRequest(id, {
        reviewer_name: 'Riley Brooks',
        feedback,
      })
      setShowRejectInputId(null)
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to reject request')
    } finally {
      setProcessingId(null)
    }
  }

  async function handleProvideInfo(id: string) {
    const answer = teammateAnswers[id]?.trim()
    if (!answer) return
    setProcessingId(id)
    try {
      await api.provideCopilotInfo(id, {
        reviewer_name: 'Riley Brooks',
        teammate_answer: answer,
      })
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to submit teammate answer')
    } finally {
      setProcessingId(null)
    }
  }

  if (loading) {
    return (
      <div className="py-12 text-center text-sm text-[var(--color-ink-muted)]">
        Loading approvals queue...
      </div>
    )
  }

  const pendingCount = approvals.filter((a) => a.status === 'pending').length

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-[var(--color-ink)]">
              Copilot Teammate Approvals
            </h1>
            {pendingCount > 0 && (
              <span className="rounded-full bg-amber-100 px-2.5 py-0.5 text-xs font-semibold text-amber-900 flex items-center gap-1">
                <Clock className="h-3 w-3 text-amber-700" />
                <span>{pendingCount} Pending</span>
              </span>
            )}
          </div>
          <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
            AI-proposed high-value actions (refunds exceeding autonomous threshold, carrier exceptions) awaiting human sign-off before dispatch.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="text-xs border border-[var(--color-border)] rounded-[var(--radius-md)] px-3 py-1.5 bg-[var(--color-surface)] text-[var(--color-ink)] font-medium focus:outline-none"
          >
            <option value="all">All Statuses ({approvals.length})</option>
            <option value="pending">Pending Review</option>
            <option value="approved">Approved</option>
            <option value="rejected">Rejected</option>
          </select>
        </div>
      </div>

      <div className="glass-card p-5 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)]">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
            <ShieldAlert className="h-4 w-4 text-[var(--color-accent)]" />
            <span>Active Sign-off Requests ({approvals.length})</span>
          </h2>
        </div>

        <div className="space-y-4">
          {approvals.map((app) => {
            const rawAction = app.actionData || app.action_data || {}
            const note = app.internalNote || app.internal_note
            const propReply = app.proposedReply || app.proposed_reply
            const amendedReply = amendedReplies[app.id] ?? (app.amendedReply || app.amended_reply || propReply)
            const question = app.questionToTeammate || app.question_to_teammate
            const answer = app.teammateAnswer || app.teammate_answer
            const reviewer = app.reviewerName || app.reviewer_name
            const feedback = app.reviewerFeedback || app.reviewer_feedback
            const isEditing = editingReplyId === app.id
            const isShowingReject = showRejectInputId === app.id

            return (
              <div
                key={app.id}
                className="p-5 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-page)] space-y-3.5 transition-all"
              >
                {/* Header Row */}
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="rounded bg-[var(--color-border)]/80 px-2 py-0.5 text-[10.5px] font-semibold text-[var(--color-ink)] uppercase tracking-wider">
                        {app.requestType || app.request_type || 'exception'}
                      </span>
                      <h3 className="font-bold text-sm text-[var(--color-ink)]">{app.title}</h3>
                    </div>
                    {note && (
                      <p className="text-xs text-[var(--color-ink-muted)] mt-1 leading-relaxed">
                        {note}
                      </p>
                    )}
                  </div>
                  <span
                    className={`text-xs font-bold uppercase px-2.5 py-0.5 rounded-full shrink-0 flex items-center gap-1 ${
                      app.status === 'approved'
                        ? 'bg-emerald-100 text-emerald-800'
                        : app.status === 'rejected'
                        ? 'bg-rose-100 text-rose-800'
                        : 'bg-amber-100 text-amber-800'
                    }`}
                  >
                    {app.status === 'approved' && <CheckCircle2 className="h-3.5 w-3.5 text-emerald-700" />}
                    {app.status === 'rejected' && <XCircle className="h-3.5 w-3.5 text-rose-700" />}
                    {app.status === 'pending' && <AlertCircle className="h-3.5 w-3.5 text-amber-700" />}
                    <span>{app.status}</span>
                  </span>
                </div>

                {/* Action Data Badges */}
                {typeof rawAction === 'object' && Object.keys(rawAction).length > 0 && (
                  <div className="flex flex-wrap gap-2 text-xs">
                    {Object.entries(rawAction).map(([k, v]) => (
                      <span
                        key={k}
                        className="inline-flex items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--color-surface)] border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-ink)] font-medium"
                      >
                        <span className="text-[var(--color-ink-faint)] uppercase text-[9.5px]">
                          {k.replace(/_/g, ' ')}:
                        </span>
                        <strong className="text-[var(--color-ink)]">{String(v)}</strong>
                      </span>
                    ))}
                  </div>
                )}

                {/* Proposed / Amended Reply Box */}
                {propReply && (
                  <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-3 text-xs space-y-2">
                    <div className="flex items-center justify-between text-[11px] text-[var(--color-ink-muted)]">
                      <span className="font-semibold flex items-center gap-1.5">
                        <MessageSquare className="h-3.5 w-3.5 text-[var(--color-accent)]" />
                        <span>Proposed Customer Response:</span>
                      </span>
                      {app.status === 'pending' && !isEditing && (
                        <button
                          type="button"
                          onClick={() => setEditingReplyId(app.id)}
                          className="text-[var(--color-accent)] hover:underline inline-flex items-center gap-1"
                        >
                          <Edit3 className="h-3 w-3" />
                          <span>Amend Reply</span>
                        </button>
                      )}
                    </div>

                    {isEditing ? (
                      <div className="space-y-2">
                        <textarea
                          rows={3}
                          value={amendedReply}
                          onChange={(e) =>
                            setAmendedReplies((prev) => ({ ...prev, [app.id]: e.target.value }))
                          }
                          className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] p-2.5 text-xs text-[var(--color-ink)] focus:outline-none focus:border-[var(--color-accent)]"
                        />
                        <div className="flex items-center justify-end gap-2">
                          <button
                            type="button"
                            onClick={() => setEditingReplyId(null)}
                            className="px-2.5 py-1 text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
                          >
                            Cancel Edit
                          </button>
                        </div>
                      </div>
                    ) : (
                      <p className="text-xs text-[var(--color-ink)] italic leading-relaxed">
                        "{amendedReply}"
                      </p>
                    )}
                  </div>
                )}

                {/* Question to Teammate Box */}
                {question && (
                  <div className="rounded-[var(--radius-lg)] border border-indigo-200 bg-indigo-50/60 p-3 text-xs space-y-2 text-indigo-950">
                    <div className="flex items-center gap-1.5 font-bold text-[11px] text-indigo-900">
                      <HelpCircle className="h-3.5 w-3.5 text-indigo-700" />
                      <span>AI Copilot Question for Reviewer:</span>
                    </div>
                    <p className="text-xs">{question}</p>

                    {answer ? (
                      <div className="mt-2 pt-2 border-t border-indigo-200/60 text-xs">
                        <span className="text-[10.5px] font-bold text-indigo-800 block">
                          Teammate Answer:
                        </span>
                        <p className="text-[var(--color-ink)]">{answer}</p>
                      </div>
                    ) : app.status === 'pending' ? (
                      <div className="flex items-center gap-2 pt-1">
                        <input
                          type="text"
                          value={teammateAnswers[app.id] || ''}
                          onChange={(e) =>
                            setTeammateAnswers((prev) => ({ ...prev, [app.id]: e.target.value }))
                          }
                          placeholder="Provide answer or clarifying instructions..."
                          className="flex-1 rounded-[var(--radius-md)] border border-indigo-200 bg-white px-2.5 py-1.5 text-xs text-[var(--color-ink)] focus:outline-none"
                        />
                        <button
                          type="button"
                          onClick={() => handleProvideInfo(app.id)}
                          disabled={processingId === app.id || !teammateAnswers[app.id]?.trim()}
                          className="px-3 py-1.5 rounded-[var(--radius-md)] bg-indigo-700 text-white font-semibold text-xs shadow-sm hover:bg-indigo-800 disabled:opacity-50"
                        >
                          Submit Info
                        </button>
                      </div>
                    ) : null}
                  </div>
                )}

                {/* Reviewer Note for approved/rejected items */}
                {(reviewer || feedback) && (
                  <div className="text-[11px] text-[var(--color-ink-muted)] pt-1 flex items-center justify-between border-t border-[var(--color-border)]">
                    <span>
                      Reviewed by: <strong className="text-[var(--color-ink)]">{reviewer || 'Operations Lead'}</strong>
                    </span>
                    {feedback && <span>Feedback: "{feedback}"</span>}
                  </div>
                )}

                {/* Rejection input when triggered */}
                {isShowingReject && (
                  <div className="pt-2 border-t border-[var(--color-border)] space-y-2">
                    <label className="text-xs font-semibold text-rose-800 block">
                      Rejection Reason / Guidance for AI:
                    </label>
                    <div className="flex items-center gap-2">
                      <input
                        type="text"
                        value={rejectFeedback[app.id] || ''}
                        onChange={(e) =>
                          setRejectFeedback((prev) => ({ ...prev, [app.id]: e.target.value }))
                        }
                        placeholder="e.g. Return label required first, or exceeds customer tier limit..."
                        className="flex-1 rounded-[var(--radius-md)] border border-rose-300 bg-white px-2.5 py-1.5 text-xs text-[var(--color-ink)] focus:outline-none"
                      />
                      <button
                        type="button"
                        onClick={() => handleReject(app.id)}
                        disabled={processingId === app.id}
                        className="px-3 py-1.5 rounded-[var(--radius-md)] bg-rose-600 text-white font-semibold text-xs shadow-sm hover:bg-rose-700 disabled:opacity-50"
                      >
                        Confirm Rejection
                      </button>
                      <button
                        type="button"
                        onClick={() => setShowRejectInputId(null)}
                        className="px-2 py-1 text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                )}

                {/* Action Buttons for Pending */}
                {app.status === 'pending' && !isShowingReject && (
                  <div className="flex items-center gap-2 pt-2 border-t border-[var(--color-border)]">
                    <button
                      type="button"
                      onClick={() => handleApprove(app)}
                      disabled={processingId === app.id}
                      className="inline-flex items-center gap-1.5 px-4 py-2 text-xs rounded-[var(--radius-md)] bg-emerald-600 text-white font-semibold shadow-sm hover:bg-emerald-700 disabled:opacity-50"
                    >
                      <Send className="h-3.5 w-3.5" />
                      <span>{processingId === app.id ? 'Processing...' : 'Approve & Dispatch'}</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setShowRejectInputId(app.id)}
                      disabled={processingId === app.id}
                      className="px-3.5 py-2 text-xs rounded-[var(--radius-md)] border border-rose-200 text-rose-700 font-semibold hover:bg-rose-50 disabled:opacity-50"
                    >
                      Reject Request
                    </button>
                  </div>
                )}
              </div>
            )
          })}

          {approvals.length === 0 && (
            <div className="py-12 text-center text-sm text-[var(--color-ink-faint)]">
              No approval requests match the current status filter.
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
