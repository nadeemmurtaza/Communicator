import { useEffect, useState } from 'react'
import {
  api,
  type AuditLogItem,
  type AuditStatistics,
} from '../lib/api'

export function AuditLogs() {
  const [logs, setLogs] = useState<AuditLogItem[]>([])
  const [stats, setStats] = useState<AuditStatistics | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filters
  const [search, setSearch] = useState('')
  const [selectedSeverity, setSelectedSeverity] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('')
  const [page, setPage] = useState(1)
  const [pageSize] = useState(25)
  const [totalCount, setTotalCount] = useState(0)
  const [totalPages, setTotalPages] = useState(1)

  // Selected Log for Inspection Modal
  const [inspectedLog, setInspectedLog] = useState<AuditLogItem | null>(null)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    loadAuditData()
  }, [page, selectedSeverity, selectedCategory])

  async function loadAuditData() {
    try {
      setLoading(true)
      setError(null)
      const [res, st] = await Promise.all([
        api.getAuditLogs({
          search: search.trim() || undefined,
          severity: selectedSeverity || undefined,
          compliance_category: selectedCategory || undefined,
          page,
          page_size: pageSize,
        }),
        api.getAuditStats().catch(() => null),
      ])

      setLogs(res.items)
      setTotalCount(res.total)
      setTotalPages(res.total_pages)
      if (st) setStats(st)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  function handleSearchSubmit(e: React.FormEvent) {
    e.preventDefault()
    setPage(1)
    loadAuditData()
  }

  function copyJsonDetails(details: Record<string, any>) {
    navigator.clipboard.writeText(JSON.stringify(details, null, 2))
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  function formatTimestamp(isoString: string) {
    if (!isoString) return '-'
    try {
      const d = new Date(isoString)
      return d.toLocaleString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        timeZoneName: 'short',
      })
    } catch {
      return isoString
    }
  }

  const criticalWarningCount =
    (stats?.severity_distribution?.warning || 0) +
    (stats?.severity_distribution?.critical || 0)

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-[var(--color-border)] pb-5">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
              Audit Logs & SOC 2 Compliance
            </h1>
            <span className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2.5 py-0.5 text-xs font-semibold text-emerald-700 border border-emerald-200">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
              SOC 2 Monitored
            </span>
          </div>
          <p className="text-xs text-[var(--color-ink-muted)] mt-1">
            Tamper-evident activity trail recording administrative decisions, data exports, permissions, and AI copilot actions.
          </p>
        </div>

        {/* Action Buttons: Export CSV / JSON */}
        <div className="flex items-center gap-2">
          <a
            href="/api/v1/audit-logs/export?format=csv"
            download="audit-trail-soc2.csv"
            className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] shadow-sm hover:bg-[var(--color-page)] flex items-center gap-1.5"
          >
            <span>📥 Export CSV</span>
          </a>
          <a
            href="/api/v1/audit-logs/export?format=json"
            target="_blank"
            rel="noreferrer"
            className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] shadow-sm hover:bg-[var(--color-page)] flex items-center gap-1.5"
          >
            <span>📦 Export JSON</span>
          </a>
          <button
            onClick={() => loadAuditData()}
            className="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white shadow hover:bg-blue-700 flex items-center gap-1.5"
          >
            <span>↺ Refresh</span>
          </button>
        </div>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
            Total Audited Events
          </div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-[var(--color-ink)]">
              {stats?.total_events ?? totalCount}
            </span>
            <span className="text-[11px] font-semibold text-emerald-600">● 100% Retained</span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">Immutable compliance ledger</p>
        </div>

        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
            Security & Alerts
          </div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-[var(--color-ink)]">
              {criticalWarningCount}
            </span>
            <span
              className={`text-[11px] font-semibold ${
                criticalWarningCount > 0 ? 'text-amber-600' : 'text-emerald-600'
              }`}
            >
              {criticalWarningCount > 0 ? '⚠ Requires Review' : '✓ Zero Critical Alerts'}
            </span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">Warnings & critical anomalies</p>
        </div>

        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
            Compliance Standard
          </div>
          <div className="mt-1 flex items-baseline gap-2">
            <span className="text-xl font-bold text-[var(--color-ink)]">SOC 2 Type II</span>
            <span className="text-[11px] font-semibold text-blue-600">ISO 27001</span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">Audit-ready export envelope</p>
        </div>

        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-sm">
          <div className="text-[11px] font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
            Top Actor
          </div>
          <div className="mt-1 flex items-baseline gap-2 truncate">
            <span className="text-xl font-bold text-[var(--color-ink)] truncate">
              {stats?.top_actors?.[0]?.actor || 'system'}
            </span>
            <span className="text-[11px] font-semibold text-purple-600">
              {stats?.top_actors?.[0]?.count ? `${stats.top_actors[0].count} actions` : ''}
            </span>
          </div>
          <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5 truncate">
            Last logged: {stats?.last_audit_event_at ? formatTimestamp(stats.last_audit_event_at) : 'Active'}
          </p>
        </div>
      </div>

      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-3.5 text-xs text-red-800 flex items-center justify-between">
          <span>⚠️ {error}</span>
          <button onClick={() => setError(null)} className="text-red-600 hover:text-red-900">✕</button>
        </div>
      )}

      {/* Filter and Search Bar */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
        <form onSubmit={handleSearchSubmit} className="flex flex-col md:flex-row items-center gap-3">
          <div className="relative flex-1 w-full">
            <span className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none text-[var(--color-ink-faint)] text-xs">
              🔍
            </span>
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search by action, actor, IP address, description..."
              className="w-full rounded-lg border border-slate-300 pl-8 pr-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
            />
          </div>

          <div className="flex items-center gap-2 w-full md:w-auto">
            <select
              value={selectedSeverity}
              onChange={(e) => {
                setSelectedSeverity(e.target.value)
                setPage(1)
              }}
              className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-2 text-xs text-[var(--color-ink)] focus:border-blue-500 focus:outline-none"
            >
              <option value="">All Severities</option>
              <option value="info">Info</option>
              <option value="notice">Notice</option>
              <option value="warning">Warning</option>
              <option value="critical">Critical</option>
            </select>

            <select
              value={selectedCategory}
              onChange={(e) => {
                setSelectedCategory(e.target.value)
                setPage(1)
              }}
              className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-3 py-2 text-xs text-[var(--color-ink)] focus:border-blue-500 focus:outline-none"
            >
              <option value="">All Categories</option>
              <option value="access_control">Access Control</option>
              <option value="data_modification">Data Modification</option>
              <option value="export_activity">Export Activity</option>
              <option value="ai_governance">AI Governance</option>
              <option value="security_event">Security Event</option>
              <option value="configuration_change">Config Change</option>
            </select>

            <button
              type="submit"
              className="rounded-lg bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shrink-0"
            >
              Filter
            </button>
          </div>
        </form>
      </div>

      {/* Audit Log Table */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
        {loading ? (
          <div className="p-12 text-center text-xs text-[var(--color-ink-muted)]">
            <div className="inline-block h-6 w-6 animate-spin rounded-full border-2 border-solid border-blue-600 border-r-transparent"></div>
            <p className="mt-2 font-medium">Loading compliance activity trail...</p>
          </div>
        ) : logs.length === 0 ? (
          <div className="p-12 text-center text-xs text-[var(--color-ink-muted)]">
            <span className="text-3xl">🛡️</span>
            <div className="font-semibold text-slate-800 mt-2">No Audit Log Entries Found</div>
            <p className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">
              Try broadening your search term or clearing active severity and category filters.
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse text-xs">
              <thead>
                <tr className="border-b border-[var(--color-border)] bg-[var(--color-page)]/75 text-[11px] font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider">
                  <th className="py-3 px-4">Timestamp (UTC)</th>
                  <th className="py-3 px-4">Actor</th>
                  <th className="py-3 px-4">Action</th>
                  <th className="py-3 px-4">Severity</th>
                  <th className="py-3 px-4">Target Entity</th>
                  <th className="py-3 px-4">Origin IP</th>
                  <th className="py-3 px-4 text-right">Details</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {logs.map((item) => {
                  const sev = item.severity?.toLowerCase() || 'info'
                  const sevClasses =
                    sev === 'critical'
                      ? 'bg-red-50 text-red-700 border-red-200'
                      : sev === 'warning'
                      ? 'bg-amber-50 text-amber-700 border-amber-200'
                      : sev === 'notice'
                      ? 'bg-slate-100 text-[var(--color-ink)] border-[var(--color-border)]'
                      : 'bg-blue-50 text-blue-700 border-blue-200'

                  return (
                    <tr
                      key={item.id}
                      className="hover:bg-[var(--color-page)]/60 transition-colors cursor-pointer"
                      onClick={() => setInspectedLog(item)}
                    >
                      {/* Timestamp */}
                      <td className="py-3 px-4 font-mono text-[11px] text-[var(--color-ink-muted)] whitespace-nowrap">
                        {formatTimestamp(item.created_at)}
                      </td>

                      {/* Actor */}
                      <td className="py-3 px-4 whitespace-nowrap">
                        <div className="flex items-center gap-2">
                          <div className="h-6 w-6 rounded-full bg-slate-800 text-white flex items-center justify-center font-bold text-[10px] shrink-0 uppercase">
                            {item.actor.charAt(0) || 'U'}
                          </div>
                          <div>
                            <div className="font-semibold text-[var(--color-ink)]">{item.actor}</div>
                            {item.actor_email && (
                              <div className="text-[10px] text-[var(--color-ink-faint)] truncate max-w-[140px]">
                                {item.actor_email}
                              </div>
                            )}
                          </div>
                        </div>
                      </td>

                      {/* Action */}
                      <td className="py-3 px-4">
                        <div>
                          <code className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[11px] text-slate-800 font-medium">
                            {item.action}
                          </code>
                          <div className="text-[10px] text-[var(--color-ink-muted)] mt-0.5 truncate max-w-[260px]">
                            {item.description}
                          </div>
                        </div>
                      </td>

                      {/* Severity */}
                      <td className="py-3 px-4 whitespace-nowrap">
                        <span className={`rounded px-2 py-0.5 text-[10px] font-bold uppercase border ${sevClasses}`}>
                          {item.severity}
                        </span>
                      </td>

                      {/* Entity */}
                      <td className="py-3 px-4 whitespace-nowrap text-[var(--color-ink-muted)]">
                        <span className="font-medium capitalize">{item.entity_type}</span>
                        {item.entity_id && item.entity_id !== '00000000-0000-0000-0000-000000000000' && (
                          <div className="font-mono text-[10px] text-[var(--color-ink-faint)]">
                            {item.entity_id.slice(0, 8)}...
                          </div>
                        )}
                      </td>

                      {/* IP */}
                      <td className="py-3 px-4 font-mono text-[11px] text-[var(--color-ink-muted)] whitespace-nowrap">
                        {item.ip_address || '127.0.0.1'}
                      </td>

                      {/* Details CTA */}
                      <td className="py-3 px-4 text-right whitespace-nowrap">
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation()
                            setInspectedLog(item)
                          }}
                          className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-[11px] font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]"
                        >
                          Inspect ↗
                        </button>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        )}

        {/* Pagination Bar */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 p-3.5 border-t border-[var(--color-border)] bg-[var(--color-page)]/50 text-xs text-[var(--color-ink-muted)]">
          <div>
            Showing <strong className="text-[var(--color-ink)]">{logs.length}</strong> of{' '}
            <strong className="text-[var(--color-ink)]">{totalCount}</strong> recorded events
          </div>
          <div className="flex items-center gap-1.5">
            <button
              onClick={() => setPage(Math.max(1, page - 1))}
              disabled={page <= 1}
              className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-40"
            >
              ← Previous
            </button>
            <span className="px-2 text-xs text-[var(--color-ink-muted)]">
              Page {page} of {totalPages}
            </span>
            <button
              onClick={() => setPage(Math.min(totalPages, page + 1))}
              disabled={page >= totalPages}
              className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-40"
            >
              Next →
            </button>
          </div>
        </div>
      </div>

      {/* Inspection Modal / Drawer */}
      {inspectedLog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/50 p-4 backdrop-blur-xs">
          <div className="w-full max-w-2xl rounded-2xl bg-[var(--color-surface)] shadow-2xl border border-[var(--color-border)] overflow-hidden flex flex-col max-h-[85vh]">
            <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)] bg-[var(--color-page)]">
              <div>
                <div className="flex items-center gap-2">
                  <h3 className="font-bold text-sm text-[var(--color-ink)]">Audit Trail Event Detail</h3>
                  <span className="rounded bg-blue-100 text-blue-700 px-2 py-0.5 text-[10px] font-bold uppercase">
                    {inspectedLog.severity}
                  </span>
                </div>
                <p className="text-[11px] font-mono text-[var(--color-ink-muted)] mt-0.5">ID: {inspectedLog.id}</p>
              </div>
              <button
                onClick={() => setInspectedLog(null)}
                className="h-7 w-7 rounded-full bg-slate-200 text-[var(--color-ink-muted)] hover:bg-slate-300 flex items-center justify-center text-xs font-bold"
              >
                ✕
              </button>
            </div>

            <div className="p-5 space-y-4 overflow-y-auto">
              <div className="grid grid-cols-2 gap-3 text-xs">
                <div className="rounded-lg bg-[var(--color-page)] p-3 border border-[var(--color-border)]">
                  <div className="text-[10px] text-[var(--color-ink-faint)] uppercase font-semibold">Action & Category</div>
                  <div className="font-mono font-bold text-slate-800 mt-0.5">{inspectedLog.action}</div>
                  <div className="text-[11px] text-[var(--color-ink-muted)] capitalize">{inspectedLog.compliance_category}</div>
                </div>

                <div className="rounded-lg bg-[var(--color-page)] p-3 border border-[var(--color-border)]">
                  <div className="text-[10px] text-[var(--color-ink-faint)] uppercase font-semibold">Actor</div>
                  <div className="font-bold text-slate-800 mt-0.5">{inspectedLog.actor}</div>
                  <div className="text-[11px] text-[var(--color-ink-muted)]">{inspectedLog.actor_email || 'System Account'}</div>
                </div>

                <div className="rounded-lg bg-[var(--color-page)] p-3 border border-[var(--color-border)]">
                  <div className="text-[10px] text-[var(--color-ink-faint)] uppercase font-semibold">Target Entity</div>
                  <div className="font-bold text-slate-800 capitalize mt-0.5">{inspectedLog.entity_type}</div>
                  <div className="font-mono text-[10px] text-[var(--color-ink-muted)] truncate">{inspectedLog.entity_id}</div>
                </div>

                <div className="rounded-lg bg-[var(--color-page)] p-3 border border-[var(--color-border)]">
                  <div className="text-[10px] text-[var(--color-ink-faint)] uppercase font-semibold">Network & Time</div>
                  <div className="font-mono text-slate-800 mt-0.5">{inspectedLog.ip_address || '127.0.0.1'}</div>
                  <div className="text-[10px] text-[var(--color-ink-muted)]">{formatTimestamp(inspectedLog.created_at)}</div>
                </div>
              </div>

              <div>
                <div className="text-xs font-semibold text-slate-800 mb-1">Event Narrative Description</div>
                <div className="rounded-lg bg-[var(--color-page)] p-3 text-xs text-[var(--color-ink)] border border-[var(--color-border)]">
                  {inspectedLog.description}
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <span className="text-xs font-semibold text-slate-800">Audit Metadata & State Snapshot (JSON)</span>
                  <button
                    onClick={() => copyJsonDetails(inspectedLog.details)}
                    className="text-[11px] font-semibold text-blue-600 hover:text-blue-800"
                  >
                    {copied ? '✓ Copied' : '📋 Copy JSON'}
                  </button>
                </div>
                <pre className="rounded-lg bg-slate-950 p-3.5 text-[11px] font-mono text-emerald-400 overflow-x-auto border border-slate-800 max-h-56 leading-relaxed">
                  {JSON.stringify(inspectedLog.details, null, 2)}
                </pre>
              </div>
            </div>

            <div className="p-3.5 bg-[var(--color-page)] border-t border-[var(--color-border)] flex justify-end">
              <button
                onClick={() => setInspectedLog(null)}
                className="rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)]"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
