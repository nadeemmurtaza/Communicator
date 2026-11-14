import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  AlertTriangle,
  ArrowRight,
  Check,
  CheckCircle2,
  Clock,
  Edit2,
  Plus,
  RefreshCw,
  Search,
  ShieldAlert,
  Trash2,
  X,
} from '../components/icons'
import { api } from '../lib/api'
import type {
  SLAPolicy,
  SLAMetrics,
  SLABreachedConversation,
  SLAEvaluationResult,
} from '../lib/api'

export function SlaMonitoring() {
  const [policies, setPolicies] = useState<SLAPolicy[]>([])
  const [metrics, setMetrics] = useState<SLAMetrics | null>(null)
  const [breaches, setBreaches] = useState<SLABreachedConversation[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const navigate = useNavigate()

  // Add Policy State
  const [showAddModal, setShowAddModal] = useState(false)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [firstResponseThreshold, setFirstResponseThreshold] = useState(30)
  const [resolutionThreshold, setResolutionThreshold] = useState(360)
  const [onlyDuringBusinessHours, setOnlyDuringBusinessHours] = useState(false)
  const [priorities, setPriorities] = useState<string[]>(['medium'])
  const [creatingPolicy, setCreatingPolicy] = useState(false)

  // Edit Policy State
  const [editingPolicy, setEditingPolicy] = useState<SLAPolicy | null>(null)
  const [editName, setEditName] = useState('')
  const [editDescription, setEditDescription] = useState('')
  const [editFirstResponse, setEditFirstResponse] = useState(30)
  const [editResolution, setEditResolution] = useState(360)
  const [editBusinessHours, setEditBusinessHours] = useState(false)
  const [editPriorities, setEditPriorities] = useState<string[]>([])
  const [updatingPolicy, setUpdatingPolicy] = useState(false)

  // Evaluate SLA Modal State
  const [showEvalModal, setShowEvalModal] = useState(false)
  const [evalConvId, setEvalConvId] = useState('')
  const [selectedPolicyId, setSelectedPolicyId] = useState('')
  const [evaluating, setEvaluating] = useState(false)
  const [evalResult, setEvalResult] = useState<SLAEvaluationResult | null>(null)

  function loadData() {
    setRefreshing(true)
    return Promise.all([api.listSLAPolicies(), api.getSLAMetrics(), api.getSLABreaches()])
      .then(([p, m, b]) => {
        setPolicies(p)
        setMetrics(m)
        setBreaches(b)
      })
      .finally(() => {
        setLoading(false)
        setRefreshing(false)
      })
  }

  useEffect(() => {
    loadData()
  }, [])

  function togglePriority(
    prio: string,
    current: string[],
    setter: (val: string[]) => void
  ) {
    if (current.includes(prio)) {
      setter(current.filter((p) => p !== prio))
    } else {
      setter([...current, prio])
    }
  }

  async function handleCreatePolicy() {
    if (!name.trim()) return
    setCreatingPolicy(true)
    try {
      await api.createSLAPolicy({
        name: name.trim(),
        description: description.trim(),
        first_response_time_threshold: Number(firstResponseThreshold) || 60,
        resolution_time_threshold: Number(resolutionThreshold) || 1440,
        only_during_business_hours: onlyDuringBusinessHours,
        priorities: priorities.length > 0 ? priorities : ['medium'],
      })
      setShowAddModal(false)
      setName('')
      setDescription('')
      setFirstResponseThreshold(30)
      setResolutionThreshold(360)
      setOnlyDuringBusinessHours(false)
      setPriorities(['medium'])
      await loadData()
    } catch (e: any) {
      alert(e.message || 'Failed to create SLA policy')
    } finally {
      setCreatingPolicy(false)
    }
  }

  function startEdit(policy: SLAPolicy) {
    setEditingPolicy(policy)
    setEditName(policy.name || '')
    setEditDescription(policy.description || '')
    setEditFirstResponse(policy.firstResponseTimeThreshold || 60)
    setEditResolution(policy.resolutionTimeThreshold || 1440)
    setEditBusinessHours(Boolean(policy.onlyDuringBusinessHours))
    setEditPriorities(Array.isArray(policy.priorities) ? policy.priorities : [])
  }

  async function handleUpdatePolicy() {
    if (!editingPolicy || !editName.trim()) return
    setUpdatingPolicy(true)
    try {
      await api.updateSLAPolicy(editingPolicy.id, {
        name: editName.trim(),
        description: editDescription.trim(),
        firstResponseTimeThreshold: Number(editFirstResponse) || 60,
        resolutionTimeThreshold: Number(editResolution) || 1440,
        onlyDuringBusinessHours: editBusinessHours,
        priorities: editPriorities,
      })
      setEditingPolicy(null)
      await loadData()
    } catch (e: any) {
      alert(e.message || 'Failed to update SLA policy')
    } finally {
      setUpdatingPolicy(false)
    }
  }

  async function handleDeletePolicy(id: string) {
    if (!confirm('Are you sure you want to delete this SLA policy?')) return
    try {
      await api.deleteSLAPolicy(id)
      await loadData()
    } catch (e: any) {
      alert(e.message || 'Failed to delete SLA policy')
    }
  }

  async function handleEvaluateSLA() {
    if (!evalConvId.trim()) {
      alert('Please enter a conversation ID.')
      return
    }
    setEvaluating(true)
    try {
      const res = await api.evaluateConversationSLA(
        evalConvId.trim(),
        selectedPolicyId.trim() || undefined
      )
      setEvalResult(res)
      await loadData()
    } catch (e: any) {
      alert(e.message || 'Failed to evaluate conversation SLA')
    } finally {
      setEvaluating(false)
    }
  }

  if (loading && policies.length === 0) {
    return <p className="text-sm text-[var(--color-ink-muted)]">Loading SLA monitoring data...</p>
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div>
          <h1 className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
            <Clock className="w-5 h-5 text-[var(--color-accent)]" />
            SLA Compliance &amp; Breach Monitor
          </h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
            Service level agreements, automated breach alerts, and dynamic real-time fulfillment rates.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={loadData}
            disabled={refreshing}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] text-xs font-semibold hover:bg-[var(--color-accent-muted)] transition-colors disabled:opacity-50"
            title="Refresh metrics"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} />
            Refresh
          </button>
          <button
            type="button"
            onClick={() => {
              setShowEvalModal(true)
              setEvalResult(null)
            }}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] text-xs font-semibold hover:bg-[var(--color-accent-muted)] transition-colors"
          >
            <Search className="w-3.5 h-3.5 text-[var(--color-accent)]" />
            Evaluate Conversation
          </button>
          <button
            type="button"
            onClick={() => setShowAddModal(true)}
            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold text-xs hover:bg-[var(--color-accent-hover)] transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            Add Policy
          </button>
        </div>
      </div>

      {/* SLA Metrics Cards */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Tracked Tickets</span>
          <p className="text-xl font-bold text-[var(--color-ink)] mt-1">{metrics?.totalTracked ?? 0}</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">1st Resp Rate</span>
          <p className="text-xl font-bold text-[var(--color-ink)] mt-1">{metrics?.firstResponseFulfillmentRate ?? 100}%</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Resolution Rate</span>
          <p className="text-xl font-bold text-[var(--color-ink)] mt-1">{metrics?.resolutionFulfillmentRate ?? 100}%</p>
        </div>
        <div className="glass-card p-4">
          <span className="text-xs font-semibold text-[var(--color-ink-faint)] uppercase">Active Breaches</span>
          <p className="text-xl font-bold text-red-600 mt-1">{metrics?.activeBreachesCount ?? breaches.length}</p>
        </div>
      </div>

      {/* Active SLA Policies Section */}
      <div className="glass-card p-5">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-semibold text-[var(--color-ink)] flex items-center gap-2">
            <ShieldAlert className="w-4 h-4 text-[var(--color-accent)]" />
            Active SLA Policies
          </h2>
          <span className="text-xs text-[var(--color-ink-faint)]">{policies.length} configured</span>
        </div>

        <div className="space-y-2.5">
          {policies.map((policy) => {
            const prios = Array.isArray(policy.priorities) ? policy.priorities : []
            return (
              <div
                key={policy.id}
                className="p-3.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-sm hover:bg-[var(--color-accent-muted)] transition-colors"
              >
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <p className="font-semibold text-[var(--color-ink)]">{policy.name}</p>
                    {policy.onlyDuringBusinessHours && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-black/5 text-[var(--color-ink-muted)] uppercase tracking-wider font-semibold">
                        Business Hours
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-[var(--color-ink-muted)]">{policy.description || 'No description provided'}</p>
                  <div className="flex items-center gap-1 pt-0.5">
                    <span className="text-[11px] text-[var(--color-ink-muted)]">Applies to:</span>
                    {prios.map((pr) => (
                      <span key={pr} className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-[var(--color-accent-soft)] text-[var(--color-accent)] font-semibold">
                        {pr}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="flex items-center gap-3 shrink-0 self-end sm:self-center">
                  <div className="flex items-center gap-1.5 text-xs">
                    <span className="px-2 py-0.5 rounded bg-[var(--color-accent-soft)] text-[var(--color-accent)] font-semibold">
                      1st: {policy.firstResponseTimeThreshold}m
                    </span>
                    <span className="px-2 py-0.5 rounded bg-[var(--color-accent-soft)] text-[var(--color-accent)] font-semibold">
                      Res:{' '}
                      {policy.resolutionTimeThreshold >= 60
                        ? `${Math.round(policy.resolutionTimeThreshold / 60)}h`
                        : `${policy.resolutionTimeThreshold}m`}
                    </span>
                  </div>

                  <div className="flex items-center gap-1 border-l border-[var(--color-border)] pl-2">
                    <button
                      type="button"
                      onClick={() => startEdit(policy)}
                      className="p-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] rounded hover:bg-[var(--color-accent-muted)]"
                      title="Edit Policy"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDeletePolicy(policy.id)}
                      className="p-1 text-[var(--color-ink-muted)] hover:text-red-600 rounded hover:bg-red-50"
                      title="Delete Policy"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      </div>

      {/* Active Breaches & At-Risk Conversations */}
      <div className="glass-card p-5">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-semibold text-[var(--color-ink)] flex items-center gap-2">
            <AlertTriangle className="w-4 h-4 text-amber-600" />
            Active Breaches &amp; At-Risk Conversations
          </h2>
          <span className="text-xs text-[var(--color-ink-faint)]">{breaches.length} tickets</span>
        </div>

        {breaches.length === 0 ? (
          <div className="p-4 rounded-[var(--radius-sm)] bg-[var(--color-accent-soft)] text-center text-sm text-[var(--color-ink)] flex items-center justify-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-emerald-600" />
            No active SLA breaches. All response and resolution targets are currently fulfilled.
          </div>
        ) : (
          <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)] text-sm">
            {breaches.map((b) => (
              <div
                key={b.conversationId}
                className="p-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-3 hover:bg-[var(--color-accent-muted)] transition-colors"
              >
                <div className="space-y-1">
                  <p className="font-semibold text-[var(--color-ink)]">{b.contactName || 'Anonymous Visitor'}</p>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    {b.channel} · Priority: <span className="font-semibold uppercase">{b.priority}</span> · Policy: {b.policyName}
                  </p>
                  <p className="text-[11px] text-[var(--color-ink-muted)]">
                    1st Resp: <span className="font-medium">{b.firstResponseStatus}</span> · Res:{' '}
                    <span className="font-medium">{b.resolutionStatus}</span>
                  </p>
                </div>
                <div className="flex items-center gap-2 shrink-0 self-end sm:self-center">
                  <span
                    className={`px-2 py-0.5 rounded text-xs font-semibold ${
                      b.isBreached
                        ? 'bg-red-100 text-red-700'
                        : 'bg-amber-100 text-amber-700'
                    }`}
                  >
                    {b.isBreached ? 'Breached' : 'Nearing Breach'}
                  </span>
                  <button
                    type="button"
                    onClick={() => navigate('/')}
                    className="flex items-center gap-1.5 px-3 py-1 text-xs rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold hover:bg-[var(--color-accent-hover)] transition-colors"
                  >
                    Open Inbox
                    <ArrowRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Add Policy Modal */}
      {showAddModal && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-lg w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <h3 className="font-semibold text-base text-[var(--color-ink)] flex items-center gap-2">
                <Plus className="w-4 h-4 text-[var(--color-accent)]" />
                Create New SLA Policy
              </h3>
              <button
                type="button"
                onClick={() => setShowAddModal(false)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">Policy Name</label>
                <input
                  type="text"
                  placeholder="e.g. Critical Enterprise VIP Tier"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">Description</label>
                <textarea
                  rows={2}
                  placeholder="Targets and escalation protocols for this SLA policy..."
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                    First Response Target (minutes)
                  </label>
                  <input
                    type="number"
                    min="1"
                    value={firstResponseThreshold}
                    onChange={(e) => setFirstResponseThreshold(Number(e.target.value))}
                    className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-1.5 bg-[var(--color-surface)]"
                  />
                </div>
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                    Resolution Target (minutes)
                  </label>
                  <input
                    type="number"
                    min="1"
                    value={resolutionThreshold}
                    onChange={(e) => setResolutionThreshold(Number(e.target.value))}
                    className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-1.5 bg-[var(--color-surface)]"
                  />
                </div>
              </div>

              <div className="space-y-1 pt-1">
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block">
                  Applicable Priorities
                </label>
                <div className="flex flex-wrap gap-2">
                  {['urgent', 'high', 'medium', 'low'].map((p) => (
                    <label key={p} className="flex items-center gap-1.5 text-xs text-[var(--color-ink)] cursor-pointer">
                      <input
                        type="checkbox"
                        checked={priorities.includes(p)}
                        onChange={() => togglePriority(p, priorities, setPriorities)}
                      />
                      <span className="uppercase font-mono">{p}</span>
                    </label>
                  ))}
                </div>
              </div>

              <label className="flex items-center gap-2 text-xs text-[var(--color-ink)] cursor-pointer pt-1">
                <input
                  type="checkbox"
                  checked={onlyDuringBusinessHours}
                  onChange={(e) => setOnlyDuringBusinessHours(e.target.checked)}
                />
                Calculate target times during business hours only
              </label>
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setShowAddModal(false)}
                className="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleCreatePolicy}
                disabled={creatingPolicy || !name.trim()}
                className="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
              >
                <Plus className="w-3.5 h-3.5" />
                {creatingPolicy ? 'Creating...' : 'Save Policy'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Policy Modal */}
      {editingPolicy && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-lg w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <h3 className="font-semibold text-base text-[var(--color-ink)] flex items-center gap-2">
                <Edit2 className="w-4 h-4 text-[var(--color-accent)]" />
                Edit SLA Policy: {editingPolicy.name}
              </h3>
              <button
                type="button"
                onClick={() => setEditingPolicy(null)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">Policy Name</label>
                <input
                  type="text"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">Description</label>
                <textarea
                  rows={2}
                  value={editDescription}
                  onChange={(e) => setEditDescription(e.target.value)}
                  className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                    First Response Target (minutes)
                  </label>
                  <input
                    type="number"
                    min="1"
                    value={editFirstResponse}
                    onChange={(e) => setEditFirstResponse(Number(e.target.value))}
                    className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-1.5 bg-[var(--color-surface)]"
                  />
                </div>
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                    Resolution Target (minutes)
                  </label>
                  <input
                    type="number"
                    min="1"
                    value={editResolution}
                    onChange={(e) => setEditResolution(Number(e.target.value))}
                    className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-1.5 bg-[var(--color-surface)]"
                  />
                </div>
              </div>

              <div className="space-y-1 pt-1">
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block">
                  Applicable Priorities
                </label>
                <div className="flex flex-wrap gap-2">
                  {['urgent', 'high', 'medium', 'low'].map((p) => (
                    <label key={p} className="flex items-center gap-1.5 text-xs text-[var(--color-ink)] cursor-pointer">
                      <input
                        type="checkbox"
                        checked={editPriorities.includes(p)}
                        onChange={() => togglePriority(p, editPriorities, setEditPriorities)}
                      />
                      <span className="uppercase font-mono">{p}</span>
                    </label>
                  ))}
                </div>
              </div>

              <label className="flex items-center gap-2 text-xs text-[var(--color-ink)] cursor-pointer pt-1">
                <input
                  type="checkbox"
                  checked={editBusinessHours}
                  onChange={(e) => setEditBusinessHours(e.target.checked)}
                />
                Calculate target times during business hours only
              </label>
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setEditingPolicy(null)}
                className="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleUpdatePolicy}
                disabled={updatingPolicy || !editName.trim()}
                className="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
              >
                <Check className="w-3.5 h-3.5" />
                {updatingPolicy ? 'Updating...' : 'Save Changes'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Evaluate SLA Modal */}
      {showEvalModal && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-lg w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <h3 className="font-semibold text-base text-[var(--color-ink)] flex items-center gap-2">
                <Search className="w-4 h-4 text-[var(--color-accent)]" />
                Evaluate SLA for Conversation
              </h3>
              <button
                type="button"
                onClick={() => setShowEvalModal(false)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                  Conversation UUID
                </label>
                <input
                  type="text"
                  placeholder="Paste conversation UUID to evaluate..."
                  value={evalConvId}
                  onChange={(e) => setEvalConvId(e.target.value)}
                  className="w-full text-xs font-mono border border-[var(--color-border)] rounded px-2.5 py-2 bg-[var(--color-surface)]"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                  Optional Policy Override
                </label>
                <select
                  value={selectedPolicyId}
                  onChange={(e) => setSelectedPolicyId(e.target.value)}
                  className="w-full text-xs border border-[var(--color-border)] rounded px-2.5 py-1.5 bg-[var(--color-surface)]"
                >
                  <option value="">Auto-match based on priority and inbox</option>
                  {policies.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} ({p.firstResponseTimeThreshold}m / {p.resolutionTimeThreshold}m)
                    </option>
                  ))}
                </select>
              </div>

              {evalResult && (
                <div className="p-3 bg-[var(--color-accent-soft)]/50 border border-[var(--color-accent)]/20 rounded space-y-2 text-xs">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-[var(--color-ink)]">
                      Matched Policy: {evalResult.sla.policyName}
                    </span>
                    <span
                      className={`px-2 py-0.5 rounded text-[11px] font-semibold ${
                        evalResult.sla.isBreached
                          ? 'bg-red-100 text-red-700'
                          : 'bg-emerald-100 text-emerald-700'
                      }`}
                    >
                      {evalResult.sla.isBreached ? 'Breached' : 'Compliant'}
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-2 text-[var(--color-ink)]">
                    <div>
                      <p className="text-[11px] text-[var(--color-ink-muted)]">First Response</p>
                      <p className="font-semibold capitalize">{evalResult.sla.firstResponseStatus}</p>
                      {evalResult.sla.firstResponseStatus === 'active' && (
                        <p className="text-[11px] text-[var(--color-ink-muted)]">
                          {evalResult.sla.minutesToFirstResponseBreach}m remaining
                        </p>
                      )}
                    </div>
                    <div>
                      <p className="text-[11px] text-[var(--color-ink-muted)]">Resolution</p>
                      <p className="font-semibold capitalize">{evalResult.sla.resolutionStatus}</p>
                      {evalResult.sla.resolutionStatus === 'active' && (
                        <p className="text-[11px] text-[var(--color-ink-muted)]">
                          {evalResult.sla.minutesToResolutionBreach}m remaining
                        </p>
                      )}
                    </div>
                  </div>

                  <div className="pt-1 border-t border-[var(--color-accent)]/20">
                    <span className="text-[11px] text-[var(--color-ink-muted)]">Auto-applied tags: </span>
                    <span className="font-mono font-semibold text-[var(--color-ink)]">
                      {evalResult.tags.join(', ') || 'none'}
                    </span>
                  </div>
                </div>
              )}
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setShowEvalModal(false)}
                className="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]"
              >
                Close
              </button>
              <button
                type="button"
                onClick={handleEvaluateSLA}
                disabled={evaluating || !evalConvId.trim()}
                className="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
              >
                <Search className="w-3.5 h-3.5" />
                {evaluating ? 'Evaluating...' : 'Evaluate SLA'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
