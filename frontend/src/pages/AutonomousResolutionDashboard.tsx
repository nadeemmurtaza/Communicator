import { useState, useEffect } from 'react'
import {
  getResolutionSummary,
  getResolutionConfig,
  updateResolutionConfig,
  listEscalationRules,
  createEscalationRule,
  updateEscalationRule,
  deleteEscalationRule,
  listAgentPolicies,
  listResolutionLogs,
  generateAnalyticsSnapshot,
  type ResolutionSummary,
  type ResolutionConfig,
  type EscalationRule,
  type AgentPolicy,
  type ResolutionLog,
  type AnalyticsSnapshot,
} from '../lib/api'
import {
  Bot, CheckCircle, AlertTriangle, TrendingUp, Settings, Shield,
  Activity, Clock, Plus, Trash2, Edit2, Play, BarChart3, FileText,
} from '../components/icons'

export default function AutonomousResolutionDashboard() {
  const [tab, setTab] = useState<'overview' | 'rules' | 'policies' | 'logs'>('overview')
  const [summary, setSummary] = useState<ResolutionSummary | null>(null)
  const [config, setConfig] = useState<ResolutionConfig | null>(null)
  const [rules, setRules] = useState<EscalationRule[]>([])
  const [policies, setPolicies] = useState<AgentPolicy[]>([])
  const [logs, setLogs] = useState<ResolutionLog[]>([])
  const [loading, setLoading] = useState(true)
  const [editingRule, setEditingRule] = useState<EscalationRule | null>(null)
  const [showRuleForm, setShowRuleForm] = useState(false)
  const [snapshot, setSnapshot] = useState<AnalyticsSnapshot | null>(null)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    setLoading(true)
    try {
      const [s, c, r, p, l] = await Promise.all([
        getResolutionSummary(),
        getResolutionConfig(),
        listEscalationRules(),
        listAgentPolicies(),
        listResolutionLogs(undefined, undefined, 50, 0),
      ])
      setSummary(s)
      setConfig(c)
      setRules(r)
      setPolicies(p)
      setLogs(l)
    } catch (e) {
      console.error('Failed to load autonomous resolution data:', e)
    }
    setLoading(false)
  }

  const handleGenerateSnapshot = async () => {
    try {
      const snap = await generateAnalyticsSnapshot(7)
      setSnapshot(snap)
    } catch (e) {
      console.error('Failed to generate snapshot:', e)
    }
  }

  const handleUpdateConfig = async (field: keyof ResolutionConfig, value: any) => {
    if (!config) return
    const updated = { ...config, [field]: value }
    setConfig(updated)
    try {
      await updateResolutionConfig(updated)
    } catch (e) {
      console.error('Failed to update config:', e)
      setConfig(config)
    }
  }

  const handleSaveRule = async (rule: EscalationRule) => {
    try {
      if (rule.id) {
        await updateEscalationRule(rule.id, rule)
      } else {
        await createEscalationRule(rule)
      }
      setShowRuleForm(false)
      setEditingRule(null)
      const updated = await listEscalationRules()
      setRules(updated)
    } catch (e) {
      console.error('Failed to save rule:', e)
    }
  }

  const handleDeleteRule = async (id: string) => {
    try {
      await deleteEscalationRule(id)
      setRules(rules.filter((r) => r.id !== id))
    } catch (e) {
      console.error('Failed to delete rule:', e)
    }
  }

  if (loading) {
    return <div className="p-6 text-center text-gray-400">Loading autonomous resolution data...</div>
  }

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white flex items-center gap-2">
          <Bot className="w-6 h-6 text-blue-400" />
          Autonomous Resolution
        </h1>
        <p className="text-gray-400 mt-1">
          Configure AI agent to resolve conversations autonomously with smart escalation rules.
        </p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-slate-800 rounded-lg p-1">
        {(['overview', 'rules', 'policies', 'logs'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors capitalize ${
              tab === t ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {tab === 'overview' && (
        <div className="space-y-6">
          {/* Status Banner */}
          <div
            className={`p-4 rounded-lg border ${
              summary?.enabled
                ? 'bg-green-900/20 border-green-500/30'
                : 'bg-yellow-900/20 border-yellow-500/30'
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                {summary?.enabled ? (
                  <CheckCircle className="w-6 h-6 text-green-400" />
                ) : (
                  <AlertTriangle className="w-6 h-6 text-yellow-400" />
                )}
                <div>
                  <h3 className="font-semibold text-white">
                    {summary?.enabled ? 'Autonomous Resolution Active' : 'Autonomous Resolution Disabled'}
                  </h3>
                  <p className="text-sm text-gray-400">
                    {summary?.enabled
                      ? 'AI agent is resolving conversations autonomously'
                      : 'Enable autonomous resolution to allow AI to resolve without human approval'}
                  </p>
                </div>
              </div>
              {config && (
                <label className="flex items-center gap-2 cursor-pointer">
                  <span className="text-sm text-gray-400">Enable</span>
                  <input
                    type="checkbox"
                    checked={config.enabled}
                    onChange={(e) => handleUpdateConfig('enabled', e.target.checked)}
                    className="w-4 h-4 rounded"
                  />
                </label>
              )}
            </div>
          </div>

          {/* Metrics Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <MetricCard
              icon={<Activity className="w-5 h-5 text-blue-400" />}
              label="Attempts (7d)"
              value={summary?.totalAttempts7d ?? 0}
            />
            <MetricCard
              icon={<CheckCircle className="w-5 h-5 text-green-400" />}
              label="Resolved (7d)"
              value={summary?.successfulResolutions7d ?? 0}
            />
            <MetricCard
              icon={<TrendingUp className="w-5 h-5 text-purple-400" />}
              label="Resolution Rate"
              value={`${((summary?.resolutionRate7d ?? 0) * 100).toFixed(1)}%`}
            />
            <MetricCard
              icon={<AlertTriangle className="w-5 h-5 text-orange-400" />}
              label="Escalation Rate"
              value={`${((summary?.escalationRate7d ?? 0) * 100).toFixed(1)}%`}
            />
          </div>

          {/* Config */}
          {config && (
            <div className="bg-slate-800 rounded-lg p-4">
              <h3 className="font-semibold text-white mb-4 flex items-center gap-2">
                <Settings className="w-4 h-4" />
                Configuration
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <ConfigField
                  label="Auto-send Threshold"
                  type="number"
                  step="0.1"
                  min="0"
                  max="1"
                  value={config.autoSendThreshold}
                  onChange={(v) => handleUpdateConfig('autoSendThreshold', parseFloat(v))}
                  help="Minimum AI confidence to auto-send"
                />
                <ConfigField
                  label="Validation Threshold"
                  type="number"
                  step="0.1"
                  min="0"
                  max="1"
                  value={config.validationThreshold}
                  onChange={(v) => handleUpdateConfig('validationThreshold', parseFloat(v))}
                  help="Minimum validation score required"
                />
                <ConfigField
                  label="Max Attempts"
                  type="number"
                  step="1"
                  min="1"
                  max="10"
                  value={config.maxAutonomousAttempts}
                  onChange={(v) => handleUpdateConfig('maxAutonomousAttempts', parseInt(v))}
                  help="Max autonomous attempts before escalation"
                />
                <ConfigField
                  label="Low Confidence Threshold"
                  type="number"
                  step="0.1"
                  min="0"
                  max="1"
                  value={config.lowConfidenceThreshold}
                  onChange={(v) => handleUpdateConfig('lowConfidenceThreshold', parseFloat(v))}
                  help="Confidence below which to escalate"
                />
                <div className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={config.requireValidation}
                    onChange={(e) => handleUpdateConfig('requireValidation', e.target.checked)}
                    className="w-4 h-4 rounded"
                  />
                  <label className="text-sm text-gray-300">Require validation before auto-send</label>
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={config.escalateOnSensitiveTopic}
                    onChange={(e) =>
                      handleUpdateConfig('escalateOnSensitiveTopic', e.target.checked)
                    }
                    className="w-4 h-4 rounded"
                  />
                  <label className="text-sm text-gray-300">Escalate on sensitive topics</label>
                </div>
              </div>
            </div>
          )}

          {/* Stats */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm text-gray-400 mb-1">Avg Confidence (7d)</h4>
              <p className="text-2xl font-bold text-white">
                {((summary?.avgConfidence7d ?? 0) * 100).toFixed(1)}%
              </p>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm text-gray-400 mb-1">Active Escalation Rules</h4>
              <p className="text-2xl font-bold text-white">{summary?.activeEscalationRules ?? 0}</p>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm text-gray-400 mb-1">Active Policies</h4>
              <p className="text-2xl font-bold text-white">{summary?.activePolicies ?? 0}</p>
            </div>
          </div>

          {/* Snapshot */}
          <div className="bg-slate-800 rounded-lg p-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold text-white flex items-center gap-2">
                <BarChart3 className="w-4 h-4" />
                Analytics Snapshot
              </h3>
              <button
                onClick={handleGenerateSnapshot}
                className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-md"
              >
                Generate 7-day Snapshot
              </button>
            </div>
            {snapshot && (
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <span className="text-sm text-gray-400">Resolution Rate:</span>
                  <span className="ml-2 text-white font-semibold">
                    {(snapshot.resolutionRate * 100).toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span className="text-sm text-gray-400">Escalation Rate:</span>
                  <span className="ml-2 text-white font-semibold">
                    {(snapshot.escalationRate * 100).toFixed(1)}%
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* Top Escalation Reasons */}
          {summary?.topEscalationReasons.length && (
            <div className="bg-slate-800 rounded-lg p-4">
              <h3 className="font-semibold text-white mb-4">Top Escalation Reasons</h3>
              <div className="space-y-2">
                {summary.topEscalationReasons.map((r, i) => (
                  <div key={i} className="flex items-center justify-between">
                    <span className="text-sm text-gray-300">{r.reason}</span>
                    <span className="text-sm font-semibold text-white">{r.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Logs */}
          {summary?.recentLogs.length && (
            <div className="bg-slate-800 rounded-lg p-4">
              <h3 className="font-semibold text-white mb-4">Recent Resolution Attempts</h3>
              <div className="space-y-2">
                {summary.recentLogs.slice(0, 5).map((log) => (
                  <div
                    key={log.id}
                    className="flex items-center justify-between p-2 bg-slate-900 rounded"
                  >
                    <div className="flex items-center gap-2">
                      {log.escalated ? (
                        <AlertTriangle className="w-4 h-4 text-orange-400" />
                      ) : (
                        <CheckCircle className="w-4 h-4 text-green-400" />
                      )}
                      <span className="text-sm text-gray-300">{log.channel}</span>
                      <span className="text-xs text-gray-500">
                        {log.aiConfidence.toFixed(2)}
                      </span>
                    </div>
                    <span className="text-xs text-gray-500">
                      {new Date(log.createdAt).toLocaleDateString()}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Escalation Rules Tab */}
      {tab === 'rules' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold text-white">Escalation Rules</h3>
            <button
              onClick={() => {
                setEditingRule(null)
                setShowRuleForm(true)
              }}
              className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-md flex items-center gap-1"
            >
              <Plus className="w-4 h-4" />
              Add Rule
            </button>
          </div>

          {showRuleForm && (
            <RuleForm
              rule={editingRule}
              onSave={handleSaveRule}
              onCancel={() => {
                setShowRuleForm(false)
                setEditingRule(null)
              }}
            />
          )}

          <div className="space-y-2">
            {rules.map((rule) => (
              <div
                key={rule.id}
                className="bg-slate-800 rounded-lg p-4 flex items-center justify-between"
              >
                <div>
                  <h4 className="font-medium text-white">{rule.name}</h4>
                  <p className="text-sm text-gray-400">
                    {rule.conditionType} {rule.conditionValue} → {rule.actionType}
                  </p>
                  {rule.description && (
                    <p className="text-xs text-gray-500 mt-1">{rule.description}</p>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <label className="flex items-center gap-1 text-xs text-gray-400">
                    <input
                      type="checkbox"
                      checked={rule.isActive}
                      onChange={async () => {
                        const updated = { ...rule, isActive: !rule.isActive }
                        await updateEscalationRule(rule.id, updated)
                        setRules(rules.map((r) => (r.id === rule.id ? updated : r)))
                      }}
                      className="w-3 h-3 rounded"
                    />
                    Active
                  </label>
                  <button
                    onClick={() => {
                      setEditingRule(rule)
                      setShowRuleForm(true)
                    }}
                    className="p-1 text-gray-400 hover:text-white"
                  >
                    <Edit2 className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleDeleteRule(rule.id)}
                    className="p-1 text-gray-400 hover:text-red-400"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Policies Tab */}
      {tab === 'policies' && (
        <div className="space-y-4">
          <h3 className="font-semibold text-white">Agent Policies</h3>
          <div className="space-y-2">
            {policies.map((policy) => (
              <div
                key={policy.id}
                className="bg-slate-800 rounded-lg p-4 flex items-center justify-between"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <Shield className="w-4 h-4 text-blue-400" />
                    <h4 className="font-medium text-white">{policy.name}</h4>
                    <span className="text-xs px-2 py-0.5 bg-slate-700 text-gray-300 rounded">
                      {policy.policyType}
                    </span>
                  </div>
                  <p className="text-sm text-gray-400 mt-1">
                    Condition: <code className="text-xs bg-slate-900 px-1 rounded">{policy.condition}</code>
                  </p>
                  <p className="text-xs text-gray-500 mt-1">
                    Action: {policy.action}
                  </p>
                  {policy.description && (
                    <p className="text-xs text-gray-500 mt-1">{policy.description}</p>
                  )}
                </div>
                <span className="text-xs text-gray-500">Priority: {policy.priority}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Logs Tab */}
      {tab === 'logs' && (
        <div className="space-y-4">
          <h3 className="font-semibold text-white">Resolution Logs</h3>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-gray-400 border-b border-slate-700">
                  <th className="pb-2">Status</th>
                  <th className="pb-2">Channel</th>
                  <th className="pb-2">Confidence</th>
                  <th className="pb-2">Validation</th>
                  <th className="pb-2">Outcome</th>
                  <th className="pb-2">Reason</th>
                  <th className="pb-2">Time</th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <tr key={log.id} className="border-b border-slate-800">
                    <td className="py-2">
                      {log.escalated ? (
                        <AlertTriangle className="w-4 h-4 text-orange-400" />
                      ) : (
                        <CheckCircle className="w-4 h-4 text-green-400" />
                      )}
                    </td>
                    <td className="py-2 text-gray-300">{log.channel}</td>
                    <td className="py-2 text-gray-300">{(log.aiConfidence * 100).toFixed(1)}%</td>
                    <td className="py-2 text-gray-300">
                      {log.validationScore !== undefined
                        ? `${(log.validationScore * 100).toFixed(1)}%`
                        : 'N/A'}
                    </td>
                    <td className="py-2">
                      <span
                        className={`text-xs px-2 py-0.5 rounded ${
                          log.outcome === 'auto_sent'
                            ? 'bg-green-900/30 text-green-400'
                            : log.outcome === 'escalated'
                            ? 'bg-orange-900/30 text-orange-400'
                            : 'bg-gray-900/30 text-gray-400'
                        }`}
                      >
                        {log.outcome}
                      </span>
                    </td>
                    <td className="py-2 text-gray-400 max-w-xs truncate">
                      {log.escalationReason || '-'}
                    </td>
                    <td className="py-2 text-gray-500">
                      {new Date(log.createdAt).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  )
}

// ——— Sub-components ———

function MetricCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string | number }) {
  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <div className="flex items-center gap-2 mb-2">
        {icon}
        <span className="text-sm text-gray-400">{label}</span>
      </div>
      <p className="text-2xl font-bold text-white">{value}</p>
    </div>
  )
}

function ConfigField({
  label,
  type,
  step,
  min,
  max,
  value,
  onChange,
  help,
}: {
  label: string
  type: string
  step?: string
  min?: string
  max?: string
  value: number
  onChange: (v: string) => void
  help?: string
}) {
  return (
    <div>
      <label className="block text-sm text-gray-400 mb-1">{label}</label>
      <input
        type={type}
        step={step}
        min={min}
        max={max}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1 text-white text-sm"
      />
      {help && <p className="text-xs text-gray-500 mt-1">{help}</p>}
    </div>
  )
}

function RuleForm({
  rule,
  onSave,
  onCancel,
}: {
  rule: EscalationRule | null
  onSave: (rule: EscalationRule) => void
  onCancel: () => void
}) {
  const [form, setForm] = useState<EscalationRule>(
    rule ?? {
      id: '',
      organizationId: 'org-default',
      name: '',
      description: '',
      conditionType: 'confidence_below',
      conditionValue: 0.4,
      actionType: 'assign_to_agent',
      isActive: true,
      priority: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }
  )

  return (
    <div className="bg-slate-800 rounded-lg p-4 space-y-3">
      <h4 className="font-medium text-white">
        {rule ? 'Edit Rule' : 'New Escalation Rule'}
      </h4>
      <div>
        <label className="block text-sm text-gray-400 mb-1">Name</label>
        <input
          type="text"
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1 text-white text-sm"
          placeholder="e.g., Low Confidence Escalation"
        />
      </div>
      <div>
        <label className="block text-sm text-gray-400 mb-1">Description</label>
        <input
          type="text"
          value={form.description || ''}
          onChange={(e) => setForm({ ...form, description: e.target.value })}
          className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1 text-white text-sm"
          placeholder="What triggers this rule?"
        />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="block text-sm text-gray-400 mb-1">Condition Type</label>
          <select
            value={form.conditionType}
            onChange={(e) => setForm({ ...form, conditionType: e.target.value })}
            className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1 text-white text-sm"
          >
            <option value="confidence_below">Confidence Below</option>
            <option value="topic_match">Topic Match</option>
            <option value="keyword_match">Keyword Match</option>
          </select>
        </div>
        <div>
          <label className="block text-sm text-gray-400 mb-1">Condition Value</label>
          <input
            type="number"
            step="0.1"
            value={form.conditionValue}
            onChange={(e) => setForm({ ...form, conditionValue: parseFloat(e.target.value) })}
            className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1 text-white text-sm"
          />
        </div>
      </div>
      <div>
        <label className="block text-sm text-gray-400 mb-1">Action Type</label>
        <select
          value={form.actionType}
          onChange={(e) => setForm({ ...form, actionType: e.target.value })}
          className="w-full bg-slate-900 border border-slate-700 rounded px-3 py-1 text-white text-sm"
        >
          <option value="assign_to_agent">Assign to Agent</option>
          <option value="assign_to_team">Assign to Team</option>
          <option value="notify_webhook">Notify Webhook</option>
        </select>
      </div>
      <div className="flex gap-2 justify-end">
        <button
          onClick={onCancel}
          className="px-3 py-1 bg-slate-700 text-gray-300 text-sm rounded-md"
        >
          Cancel
        </button>
        <button
          onClick={() => onSave(form)}
          className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-md"
        >
          Save Rule
        </button>
      </div>
    </div>
  )
}
