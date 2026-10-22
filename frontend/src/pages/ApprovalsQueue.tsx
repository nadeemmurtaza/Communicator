import { useEffect, useState } from 'react'
import { api } from '../lib/api'

export function ApprovalsQueue() {
  const [approvals, setApprovals] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [statusFilter, setStatusFilter] = useState('all')
  const [processingId, setProcessingId] = useState<string | null>(null)

  function refresh() {
    api
      .listCopilotApprovals(statusFilter === 'all' ? undefined : { status: statusFilter })
      .then(setApprovals)
      .finally(() => setLoading(false))
  }

  useEffect(refresh, [statusFilter])

  async function handleApprove(app: any) {
    setProcessingId(app.id)
    try {
      await api.approveCopilotRequest(app.id, { reviewer_name: 'Riley Brooks', execute_send: true })
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
      await api.rejectCopilotRequest(id, { reviewer_name: 'Riley Brooks', feedback: 'Rejected from queue' })
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to reject request')
    } finally {
      setProcessingId(null)
    }
  }

  if (loading) return <p className="text-sm text-[var(--color-ink-muted)]">Loading approvals...</p>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-[var(--color-ink)]">Copilot Teammate Approvals</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
            AI-proposed actions (refunds, policy exceptions) waiting on human sign-off before they execute.
          </p>
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-1.5 bg-[var(--color-surface)]"
        >
          <option value="all">All</option>
          <option value="pending">Pending</option>
          <option value="approved">Approved</option>
          <option value="rejected">Rejected</option>
        </select>
      </div>

      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3">{approvals.length} requests</h2>
        <div className="space-y-3">
          {approvals.map((app) => (
            <div key={app.id} className="p-4 rounded-[var(--radius-sm)] border border-[var(--color-border)]">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="font-medium text-sm text-[var(--color-ink)]">{app.title}</p>
                  <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">{app.internalNote || app.internal_note}</p>
                </div>
                <span className="text-xs font-semibold uppercase px-2 py-0.5 rounded-full bg-[var(--color-accent-soft)] text-[var(--color-accent)] shrink-0">
                  {app.status}
                </span>
              </div>
              {(app.proposedReply || app.proposed_reply) && (
                <p className="text-xs text-[var(--color-ink-muted)] mt-2 p-2 rounded bg-black/5 italic">
                  "{app.proposedReply || app.proposed_reply}"
                </p>
              )}
              {app.status === 'pending' && (
                <div className="flex items-center gap-2 mt-3">
                  <button
                    type="button"
                    onClick={() => handleApprove(app)}
                    disabled={processingId === app.id}
                    className="px-3 py-1.5 text-xs rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold disabled:opacity-50"
                  >
                    Approve &amp; Send
                  </button>
                  <button
                    type="button"
                    onClick={() => handleReject(app.id)}
                    disabled={processingId === app.id}
                    className="px-3 py-1.5 text-xs rounded-[var(--radius-sm)] border border-[var(--color-border)] text-[var(--color-ink-muted)] font-semibold hover:bg-black/5 disabled:opacity-50"
                  >
                    Reject
                  </button>
                </div>
              )}
            </div>
          ))}
          {approvals.length === 0 && (
            <p className="text-sm text-[var(--color-ink-faint)] text-center py-6">No approvals in this filter.</p>
          )}
        </div>
      </div>
    </div>
  )
}
