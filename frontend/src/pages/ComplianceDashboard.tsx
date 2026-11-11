import { useState, useEffect } from 'react'
import {
  getComplianceSummary, listSuppressions, addSuppression, removeSuppression,
  listGDPRRequests, createGDPRRequest, updateGDPRRequest,
  listPolicies, createPolicy, updatePolicy, deletePolicy,
  listAuditLogs, getComplianceSettings, updateComplianceSettings
} from '../lib/api'
import type {
  ComplianceSummary as ComplianceSummaryType, SuppressionEntry, GDPRRequest,
  CompliancePolicy, AuditLogEntry, ComplianceSettings
} from '../lib/api'
import { Shield, Users, FileText, Scale, AlertTriangle, CheckCircle, XCircle, Plus, Search, Filter } from '../components/icons'

export default function ComplianceDashboard() {
  const [tab, setTab] = useState<'overview' | 'suppressions' | 'gdpr' | 'policies' | 'audit' | 'settings'>('overview')
  const [summary, setSummary] = useState<ComplianceSummaryType | null>(null)
  const [suppressions, setSuppressions] = useState<SuppressionEntry[]>([])
  const [gdprRequests, setGDPRRequests] = useState<GDPRRequest[]>([])
  const [policies, setPolicies] = useState<CompliancePolicy[]>([])
  const [auditLogs, setAuditLogs] = useState<AuditLogEntry[]>([])
  const [settings, setSettings] = useState<ComplianceSettings | null>(null)
  const [loading, setLoading] = useState(true)

  // Suppression form
  const [showAddSuppression, setShowAddSuppression] = useState(false)
  const [supForm, setSupForm] = useState({ contact_id: '', channel: 'email', reason: 'opt_out', note: '' })

  // GDPR form
  const [showAddGDPR, setShowAddGDPR] = useState(false)
  const [gdprForm, setGDPRForm] = useState({ contact_id: '', email: '', request_type: 'access', notes: '' })

  // Policy form
  const [showAddPolicy, setShowAddPolicy] = useState(false)
  const [policyForm, setPolicyForm] = useState({ policy_name: '', regulation: 'gdpr', channel: 'all', description: '' })

  // Filters
  const [supChannel, setSupChannel] = useState('')
  const [supReason, setSupReason] = useState('')
  const [gdprStatus, setGDPRStatus] = useState('')
  const [auditSeverity, setAuditSeverity] = useState('')

  const loadAll = async () => {
    setLoading(true)
    try {
      const [s, sup, gdpr, pol, audit, set] = await Promise.all([
        getComplianceSummary(),
        listSuppressions(true, supChannel || undefined, supReason || undefined),
        listGDPRRequests(gdprStatus || undefined),
        listPolicies(true),
        listAuditLogs(undefined, auditSeverity || undefined, 100),
        getComplianceSettings(),
      ])
      setSummary(s)
      setSuppressions(sup)
      setGDPRRequests(gdpr)
      setPolicies(pol)
      setAuditLogs(audit)
      setSettings(set)
    } catch (e) {
      console.error('Failed to load compliance data:', e)
    }
    setLoading(false)
  }

  useEffect(() => { loadAll() }, [tab, supChannel, supReason, gdprStatus, auditSeverity])

  const handleAddSuppression = async () => {
    if (!supForm.contact_id || !supForm.channel) return
    await addSuppression({
      contact_id: supForm.contact_id,
      channel: supForm.channel,
      reason: supForm.reason,
      note: supForm.note || undefined,
    })
    setShowAddSuppression(false)
    setSupForm({ contact_id: '', channel: 'email', reason: 'opt_out', note: '' })
    loadAll()
  }

  const handleAddGDPR = async () => {
    if (!gdprForm.request_type) return
    await createGDPRRequest({
      contact_id: gdprForm.contact_id || undefined,
      email: gdprForm.email || undefined,
      request_type: gdprForm.request_type,
      notes: gdprForm.notes || undefined,
    })
    setShowAddGDPR(false)
    setGDPRForm({ contact_id: '', email: '', request_type: 'access', notes: '' })
    loadAll()
  }

  const handleAddPolicy = async () => {
    if (!policyForm.policy_name) return
    await createPolicy({
      policy_name: policyForm.policy_name,
      regulation: policyForm.regulation,
      channel: policyForm.channel,
      description: policyForm.description || undefined,
    })
    setShowAddPolicy(false)
    setPolicyForm({ policy_name: '', regulation: 'gdpr', channel: 'all', description: '' })
    loadAll()
  }

  const handleSettingsUpdate = async () => {
    if (!settings) return
    await updateComplianceSettings(settings)
    loadAll()
  }

  const statusBadge = (status: string) => {
    const colors: Record<string, string> = {
      pending: 'bg-yellow-100 text-yellow-800',
      in_progress: 'bg-blue-100 text-blue-800',
      completed: 'bg-green-100 text-green-800',
      denied: 'bg-red-100 text-red-800',
      expired: 'bg-gray-100 text-gray-800',
    }
    return (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[status] || 'bg-gray-100 text-gray-800'}`}>
        {status}
      </span>
    )
  }

  const severityBadge = (severity: string) => {
    const colors: Record<string, string> = {
      info: 'bg-blue-100 text-blue-800',
      warning: 'bg-yellow-100 text-yellow-800',
      critical: 'bg-red-100 text-red-800',
    }
    return (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[severity] || 'bg-gray-100 text-gray-800'}`}>
        {severity}
      </span>
    )
  }

  if (loading) return <div className="p-8 text-center text-gray-500">Loading compliance data...</div>

  return (
    <div className="p-6">
      <div className="flex items-center gap-3 mb-6">
        <Shield className="w-8 h-8 text-indigo-600" />
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Compliance & Data Privacy</h1>
          <p className="text-sm text-gray-500">Manage consent, suppression lists, GDPR requests, and regulatory policies</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-100 rounded-lg p-1">
        {(['overview', 'suppressions', 'gdpr', 'policies', 'audit', 'settings'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 rounded-md text-sm font-medium transition ${
              tab === t ? 'bg-white text-gray-900 shadow-sm' : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {tab === 'overview' && summary && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <div className="flex items-center gap-2 mb-2">
                <Users className="w-5 h-5 text-red-500" />
                <span className="text-sm text-gray-500">Suppressed</span>
              </div>
              <div className="text-3xl font-bold text-gray-900">{summary.active_suppressed}</div>
              <div className="text-xs text-gray-400 mt-1">of {summary.total_suppressed} total</div>
            </div>
            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <div className="flex items-center gap-2 mb-2">
                <FileText className="w-5 h-5 text-blue-500" />
                <span className="text-sm text-gray-500">Pending GDPR</span>
              </div>
              <div className="text-3xl font-bold text-gray-900">{summary.pending_gdpr_requests}</div>
              <div className="text-xs text-gray-400 mt-1">of {summary.total_gdpr_requests} total</div>
            </div>
            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <div className="flex items-center gap-2 mb-2">
                <Scale className="w-5 h-5 text-green-500" />
                <span className="text-sm text-gray-500">Active Policies</span>
              </div>
              <div className="text-3xl font-bold text-gray-900">{summary.active_policies}</div>
            </div>
            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="w-5 h-5 text-orange-500" />
                <span className="text-sm text-gray-500">Critical Violations</span>
              </div>
              <div className="text-3xl font-bold text-gray-900">{summary.critical_violations}</div>
              <div className="text-xs text-gray-400 mt-1">{summary.total_audit_entries} audit entries</div>
            </div>
          </div>

          {summary.upcoming_gdpr_expiries > 0 && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 flex items-center gap-3">
              <AlertTriangle className="w-5 h-5 text-yellow-600" />
              <span className="text-sm text-yellow-800">
                <strong>{summary.upcoming_gdpr_expiries}</strong> GDPR request(s) expiring within 7 days
              </span>
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <h3 className="font-semibold text-gray-900 mb-3">Suppression by Reason</h3>
              {summary.suppression_by_reason.length === 0 ? (
                <p className="text-sm text-gray-400">No suppressions yet</p>
              ) : (
                <div className="space-y-2">
                  {summary.suppression_by_reason.map((r) => (
                    <div key={r.reason} className="flex items-center gap-3">
                      <div className="flex-1">
                        <div className="flex justify-between text-sm mb-1">
                          <span className="text-gray-700">{r.reason}</span>
                          <span className="text-gray-500">{r.count}</span>
                        </div>
                        <div className="h-2 bg-gray-100 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-red-400 rounded-full"
                            style={{ width: `${Math.min(100, (r.count / Math.max(1, summary.active_suppressed)) * 100)}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <h3 className="font-semibold text-gray-900 mb-3">GDPR Requests by Type</h3>
              {summary.gdpr_by_type.length === 0 ? (
                <p className="text-sm text-gray-400">No GDPR requests yet</p>
              ) : (
                <div className="space-y-2">
                  {summary.gdpr_by_type.map((t) => (
                    <div key={t.request_type} className="flex items-center gap-3">
                      <div className="flex-1">
                        <div className="flex justify-between text-sm mb-1">
                          <span className="text-gray-700">{t.request_type}</span>
                          <span className="text-gray-500">{t.count}</span>
                        </div>
                        <div className="h-2 bg-gray-100 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-blue-400 rounded-full"
                            style={{ width: `${Math.min(100, (t.count / Math.max(1, summary.total_gdpr_requests)) * 100)}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {summary.recent_audit_logs.length > 0 && (
            <div className="bg-white rounded-xl p-5 border border-gray-200">
              <h3 className="font-semibold text-gray-900 mb-3">Recent Audit Activity</h3>
              <div className="space-y-2">
                {summary.recent_audit_logs.slice(0, 5).map((log) => (
                  <div key={log.id} className="flex items-center gap-3 text-sm py-2 border-b border-gray-100 last:border-0">
                    {severityBadge(log.severity)}
                    <span className="text-gray-700 font-medium">{log.action}</span>
                    <span className="text-gray-400">by {log.performed_by}</span>
                    <span className="text-gray-400 ml-auto">{new Date(log.performed_at).toLocaleString()}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Suppressions Tab */}
      {tab === 'suppressions' && (
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 bg-gray-100 rounded-lg px-3 py-2">
              <Filter className="w-4 h-4 text-gray-500" />
              <select value={supChannel} onChange={(e) => setSupChannel(e.target.value)} className="bg-transparent text-sm outline-none">
                <option value="">All Channels</option>
                <option value="email">Email</option>
                <option value="sms">SMS</option>
                <option value="whatsapp">WhatsApp</option>
                <option value="all">All</option>
              </select>
              <select value={supReason} onChange={(e) => setSupReason(e.target.value)} className="bg-transparent text-sm outline-none">
                <option value="">All Reasons</option>
                <option value="opt_out">Opt Out</option>
                <option value="bounce">Bounce</option>
                <option value="complaint">Complaint</option>
                <option value="gdpr_deletion">GDPR Deletion</option>
                <option value="manual">Manual</option>
                <option value="blocked">Blocked</option>
              </select>
            </div>
            <button
              onClick={() => setShowAddSuppression(true)}
              className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm hover:bg-indigo-700"
            >
              <Plus className="w-4 h-4" /> Add Suppression
            </button>
          </div>

          {showAddSuppression && (
            <div className="bg-white rounded-xl p-5 border border-indigo-200 space-y-3">
              <h3 className="font-semibold text-gray-900">Add to Suppression List</h3>
              <input
                placeholder="Contact ID"
                value={supForm.contact_id}
                onChange={(e) => setSupForm({ ...supForm, contact_id: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <div className="flex gap-3">
                <select
                  value={supForm.channel}
                  onChange={(e) => setSupForm({ ...supForm, channel: e.target.value })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="email">Email</option>
                  <option value="sms">SMS</option>
                  <option value="whatsapp">WhatsApp</option>
                  <option value="all">All Channels</option>
                </select>
                <select
                  value={supForm.reason}
                  onChange={(e) => setSupForm({ ...supForm, reason: e.target.value })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="opt_out">Opt Out</option>
                  <option value="bounce">Bounce</option>
                  <option value="complaint">Complaint</option>
                  <option value="gdpr_deletion">GDPR Deletion</option>
                  <option value="manual">Manual</option>
                  <option value="blocked">Blocked</option>
                </select>
              </div>
              <input
                placeholder="Note (optional)"
                value={supForm.note}
                onChange={(e) => setSupForm({ ...supForm, note: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <div className="flex gap-2">
                <button onClick={handleAddSuppression} className="px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm">Add</button>
                <button onClick={() => setShowAddSuppression(false)} className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm">Cancel</button>
              </div>
            </div>
          )}

          <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-gray-50">
                <tr>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Contact ID</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Channel</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Reason</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Added By</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Date</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Action</th>
                </tr>
              </thead>
              <tbody>
                {suppressions.map((s) => (
                  <tr key={s.id} className="border-t border-gray-100">
                    <td className="px-4 py-3 font-mono text-xs text-gray-600">{s.contact_id.slice(0, 8)}...</td>
                    <td className="px-4 py-3 capitalize">{s.channel}</td>
                    <td className="px-4 py-3">{s.reason}</td>
                    <td className="px-4 py-3 text-gray-500">{s.added_by}</td>
                    <td className="px-4 py-3 text-gray-500">{new Date(s.added_at).toLocaleDateString()}</td>
                    <td className="px-4 py-3">
                      <button onClick={() => { removeSuppression(s.id); loadAll() }} className="text-red-600 hover:text-red-800 text-xs">
                        Remove
                      </button>
                    </td>
                  </tr>
                ))}
                {suppressions.length === 0 && (
                  <tr><td colSpan={6} className="px-4 py-8 text-center text-gray-400">No suppressions found</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* GDPR Requests Tab */}
      {tab === 'gdpr' && (
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 bg-gray-100 rounded-lg px-3 py-2">
              <Filter className="w-4 h-4 text-gray-500" />
              <select value={gdprStatus} onChange={(e) => setGDPRStatus(e.target.value)} className="bg-transparent text-sm outline-none">
                <option value="">All Statuses</option>
                <option value="pending">Pending</option>
                <option value="in_progress">In Progress</option>
                <option value="completed">Completed</option>
                <option value="denied">Denied</option>
              </select>
            </div>
            <button
              onClick={() => setShowAddGDPR(true)}
              className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm hover:bg-indigo-700"
            >
              <Plus className="w-4 h-4" /> New Request
            </button>
          </div>

          {showAddGDPR && (
            <div className="bg-white rounded-xl p-5 border border-indigo-200 space-y-3">
              <h3 className="font-semibold text-gray-900">Create GDPR Request</h3>
              <input
                placeholder="Contact ID (optional)"
                value={gdprForm.contact_id}
                onChange={(e) => setGDPRForm({ ...gdprForm, contact_id: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <input
                placeholder="Email (optional)"
                value={gdprForm.email}
                onChange={(e) => setGDPRForm({ ...gdprForm, email: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <select
                value={gdprForm.request_type}
                onChange={(e) => setGDPRForm({ ...gdprForm, request_type: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              >
                <option value="access">Right of Access</option>
                <option value="deletion">Right to Deletion</option>
                <option value="portability">Data Portability</option>
                <option value="rectification">Rectification</option>
                <option value="objection">Objection</option>
              </select>
              <textarea
                placeholder="Notes (optional)"
                value={gdprForm.notes}
                onChange={(e) => setGDPRForm({ ...gdprForm, notes: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                rows={2}
              />
              <div className="flex gap-2">
                <button onClick={handleAddGDPR} className="px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm">Create</button>
                <button onClick={() => setShowAddGDPR(false)} className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm">Cancel</button>
              </div>
            </div>
          )}

          <div className="space-y-3">
            {gdprRequests.map((req) => (
              <div key={req.id} className="bg-white rounded-xl p-5 border border-gray-200">
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-gray-900 capitalize">{req.request_type}</span>
                      {statusBadge(req.status)}
                    </div>
                    <div className="text-sm text-gray-500 mt-1">
                      {req.email || `Contact: ${req.contact_id?.slice(0, 8)}...`}
                    </div>
                  </div>
                  <span className="text-xs text-gray-400">{new Date(req.requested_at).toLocaleDateString()}</span>
                </div>
                {req.notes && <p className="text-sm text-gray-600 mb-3">{req.notes}</p>}
                {req.expires_at && (
                  <p className="text-xs text-orange-600 mb-3">Expires: {new Date(req.expires_at).toLocaleDateString()}</p>
                )}
                {req.status !== 'completed' && req.status !== 'denied' && (
                  <div className="flex gap-2">
                    <button
                      onClick={() => { updateGDPRRequest(req.id, { status: 'in_progress' }); loadAll() }}
                      className="px-3 py-1 bg-blue-100 text-blue-700 rounded text-xs hover:bg-blue-200"
                    >
                      Start Processing
                    </button>
                    <button
                      onClick={() => { updateGDPRRequest(req.id, { status: 'completed' }); loadAll() }}
                      className="px-3 py-1 bg-green-100 text-green-700 rounded text-xs hover:bg-green-200"
                    >
                      Mark Completed
                    </button>
                    <button
                      onClick={() => { updateGDPRRequest(req.id, { status: 'denied' }); loadAll() }}
                      className="px-3 py-1 bg-red-100 text-red-700 rounded text-xs hover:bg-red-200"
                    >
                      Deny
                    </button>
                  </div>
                )}
              </div>
            ))}
            {gdprRequests.length === 0 && (
              <div className="bg-white rounded-xl p-8 border border-gray-200 text-center text-gray-400">
                No GDPR requests found
              </div>
            )}
          </div>
        </div>
      )}

      {/* Policies Tab */}
      {tab === 'policies' && (
        <div className="space-y-4">
          <button
            onClick={() => setShowAddPolicy(true)}
            className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm hover:bg-indigo-700"
          >
            <Plus className="w-4 h-4" /> Add Policy
          </button>

          {showAddPolicy && (
            <div className="bg-white rounded-xl p-5 border border-indigo-200 space-y-3">
              <h3 className="font-semibold text-gray-900">Add Compliance Policy</h3>
              <input
                placeholder="Policy Name"
                value={policyForm.policy_name}
                onChange={(e) => setPolicyForm({ ...policyForm, policy_name: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
              <div className="flex gap-3">
                <select
                  value={policyForm.regulation}
                  onChange={(e) => setPolicyForm({ ...policyForm, regulation: e.target.value })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="can_spam">CAN-SPAM</option>
                  <option value="tcpa">TCPA</option>
                  <option value="gdpr">GDPR</option>
                  <option value="whatsapp_business">WhatsApp Business</option>
                  <option value="ccpa">CCPA</option>
                </select>
                <select
                  value={policyForm.channel}
                  onChange={(e) => setPolicyForm({ ...policyForm, channel: e.target.value })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="email">Email</option>
                  <option value="sms">SMS</option>
                  <option value="whatsapp">WhatsApp</option>
                  <option value="all">All Channels</option>
                </select>
              </div>
              <textarea
                placeholder="Description"
                value={policyForm.description}
                onChange={(e) => setPolicyForm({ ...policyForm, description: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                rows={2}
              />
              <div className="flex gap-2">
                <button onClick={handleAddPolicy} className="px-4 py-2 bg-indigo-600 text-white rounded-lg text-sm">Add</button>
                <button onClick={() => setShowAddPolicy(false)} className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm">Cancel</button>
              </div>
            </div>
          )}

          <div className="space-y-3">
            {policies.map((p) => (
              <div key={p.id} className="bg-white rounded-xl p-5 border border-gray-200">
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-gray-900">{p.policy_name}</span>
                      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${p.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'}`}>
                        {p.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </div>
                    <div className="flex gap-2 mt-1">
                      <span className="text-xs text-gray-500 capitalize bg-gray-100 px-2 py-0.5 rounded">{p.regulation}</span>
                      <span className="text-xs text-gray-500 capitalize bg-gray-100 px-2 py-0.5 rounded">{p.channel}</span>
                    </div>
                    {p.description && <p className="text-sm text-gray-600 mt-2">{p.description}</p>}
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => { updatePolicy(p.id, !p.is_active); loadAll() }}
                      className={`px-3 py-1 rounded text-xs ${p.is_active ? 'bg-yellow-100 text-yellow-700' : 'bg-green-100 text-green-700'}`}
                    >
                      {p.is_active ? 'Disable' : 'Enable'}
                    </button>
                    <button
                      onClick={() => { deletePolicy(p.id); loadAll() }}
                      className="px-3 py-1 bg-red-100 text-red-700 rounded text-xs hover:bg-red-200"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Audit Log Tab */}
      {tab === 'audit' && (
        <div className="space-y-4">
          <div className="flex items-center gap-2 bg-gray-100 rounded-lg px-3 py-2">
            <Filter className="w-4 h-4 text-gray-500" />
            <select value={auditSeverity} onChange={(e) => setAuditSeverity(e.target.value)} className="bg-transparent text-sm outline-none">
              <option value="">All Severities</option>
              <option value="info">Info</option>
              <option value="warning">Warning</option>
              <option value="critical">Critical</option>
            </select>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-gray-50">
                <tr>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Severity</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Action</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Entity</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Performed By</th>
                  <th className="text-left px-4 py-3 text-gray-500 font-medium">Date</th>
                </tr>
              </thead>
              <tbody>
                {auditLogs.map((log) => (
                  <tr key={log.id} className="border-t border-gray-100">
                    <td className="px-4 py-3">{severityBadge(log.severity)}</td>
                    <td className="px-4 py-3 text-gray-700">{log.action}</td>
                    <td className="px-4 py-3 text-gray-500">{log.entity_type}: {log.entity_id.slice(0, 8)}...</td>
                    <td className="px-4 py-3 text-gray-500">{log.performed_by}</td>
                    <td className="px-4 py-3 text-gray-500">{new Date(log.performed_at).toLocaleString()}</td>
                  </tr>
                ))}
                {auditLogs.length === 0 && (
                  <tr><td colSpan={5} className="px-4 py-8 text-center text-gray-400">No audit logs found</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {tab === 'settings' && settings && (
        <div className="bg-white rounded-xl p-6 border border-gray-200 space-y-4">
          <h3 className="font-semibold text-gray-900">Compliance Settings</h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-gray-600 mb-1">GDPR Response Days</label>
              <input
                type="number"
                value={settings.gdpr_response_days}
                onChange={(e) => setSettings({ ...settings, gdpr_response_days: parseInt(e.target.value) || 30 })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">Data Retention Days</label>
              <input
                type="number"
                value={settings.data_retention_days}
                onChange={(e) => setSettings({ ...settings, data_retention_days: parseInt(e.target.value) || 365 })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>
          </div>

          <div className="space-y-3">
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.auto_expire_gdpr_requests}
                onChange={(e) => setSettings({ ...settings, auto_expire_gdpr_requests: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">Auto-expire GDPR requests after response period</span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.require_consent_before_send}
                onChange={(e) => setSettings({ ...settings, require_consent_before_send: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">Require consent before sending messages</span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.check_suppression_before_send}
                onChange={(e) => setSettings({ ...settings, check_suppression_before_send: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">Check suppression list before sending</span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.log_all_compliance_actions}
                onChange={(e) => setSettings({ ...settings, log_all_compliance_actions: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">Log all compliance actions to audit trail</span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.can_spam_unsubscribe_link}
                onChange={(e) => setSettings({ ...settings, can_spam_unsubscribe_link: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">CAN-SPAM: Include unsubscribe link in emails</span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.tcpa_consent_required}
                onChange={(e) => setSettings({ ...settings, tcpa_consent_required: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">TCPA: Require consent before SMS</span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={settings.whatsapp_opt_in_required}
                onChange={(e) => setSettings({ ...settings, whatsapp_opt_in_required: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm text-gray-700">WhatsApp: Require opt-in before messaging</span>
            </label>
          </div>

          <div>
            <label className="block text-sm text-gray-600 mb-1">CAN-SPAM Physical Address</label>
            <input
              value={settings.can_spam_physical_address || ''}
              onChange={(e) => setSettings({ ...settings, can_spam_physical_address: e.target.value || null })}
              placeholder="123 Main St, City, State ZIP"
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            />
          </div>

          <button
            onClick={handleSettingsUpdate}
            className="px-6 py-2 bg-indigo-600 text-white rounded-lg text-sm hover:bg-indigo-700"
          >
            Save Settings
          </button>
        </div>
      )}
    </div>
  )
}
