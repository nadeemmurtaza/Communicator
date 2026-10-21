import { useEffect, useState } from 'react'
import {
  api,
  type AgentInsight,
  type AgentRetrainingEvent,
  type AIConnection,
  type AICustomPolicy,
  type JobModelMapping,
  type SelfImprovingStats,
} from '../lib/api'

export function SelfImproving() {
  const [insights, setInsights] = useState<AgentInsight[]>([])
  const [history, setHistory] = useState<AgentRetrainingEvent[]>([])
  const [stats, setStats] = useState<SelfImprovingStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filter tab for insights: 'all' | 'human_exemplar' | 'trend' | 'policy_refinement' | 'team_coaching'
  const [activeCategory, setActiveCategory] = useState<string>('all')
  // Page tab: 'insights' | 'history' | 'coaching' | 'model_hub'
  const [activeView, setActiveView] = useState<'insights' | 'history' | 'coaching' | 'model_hub'>('insights')

  const [scanning, setScanning] = useState(false)
  const [applyingId, setApplyingId] = useState<string | null>(null)

  // AI Model Hub & Custom Policies state
  const [connections, setConnections] = useState<AIConnection[]>([])
  const [jobMapping, setJobMapping] = useState<JobModelMapping>({})
  const [selectedPolicyJob, setSelectedPolicyJob] = useState<string>('self_improving')
  const [currentCustomPolicy, setCurrentCustomPolicy] = useState<AICustomPolicy | null>(null)
  const [loadingPolicy, setLoadingPolicy] = useState(false)
  const [savingPolicy, setSavingPolicy] = useState(false)
  const [testingConnId, setTestingConnId] = useState<string | null>(null)

  // Modal: Connect New AI Model
  const [showConnectModal, setShowConnectModal] = useState(false)
  const [newConnName, setNewConnName] = useState('')
  const [newConnProvider, setNewConnProvider] = useState('anthropic')
  const [newConnModelId, setNewConnModelId] = useState('claude-3-5-sonnet-20241022')
  const [newConnApiKey, setNewConnApiKey] = useState('')
  const [newConnBaseUrl, setNewConnBaseUrl] = useState('')
  const [newConnTemp, setNewConnTemp] = useState(0.7)
  const [connectingModel, setConnectingModel] = useState(false)

  // Custom Rules & Guardrails Inputs
  const [newRuleText, setNewRuleText] = useState('')
  const [newGuardrailText, setNewGuardrailText] = useState('')

  useEffect(() => {
    loadData()
  }, [])

  useEffect(() => {
    if (activeView === 'model_hub') {
      loadModelHubData()
    }
  }, [activeView])

  useEffect(() => {
    if (activeView === 'model_hub' && selectedPolicyJob) {
      loadCustomPolicy(selectedPolicyJob)
    }
  }, [selectedPolicyJob, activeView])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [insightsData, historyData, statsData] = await Promise.all([
        api.listInsights(),
        api.listRetrainingHistory(),
        api.getSelfImprovingStats(),
      ])
      setInsights(insightsData)
      setHistory(historyData)
      setStats(statsData)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load self-improving suite')
    } finally {
      setLoading(false)
    }
  }

  const loadModelHubData = async () => {
    try {
      const [conns, mapping] = await Promise.all([
        api.listAIConnections(),
        api.getJobModelMapping(),
      ])
      setConnections(conns)
      setJobMapping(mapping)
    } catch (err) {
      console.error('Failed to load model hub data', err)
    }
  }

  const loadCustomPolicy = async (jobType: string) => {
    setLoadingPolicy(true)
    try {
      const pol = await api.getCustomPolicy(jobType)
      setCurrentCustomPolicy(pol)
    } catch (err) {
      console.error('Failed to load policy for ' + jobType, err)
    } finally {
      setLoadingPolicy(false)
    }
  }

  const handleApplyRetraining = async (insightId: string) => {
    setApplyingId(insightId)
    try {
      const res = await api.applyInsightRetraining(insightId, 'Operations Manager')
      alert(
        `Continuous Retraining Applied Successfully!\n• Upgraded to Version: ${res.version_to}\n• New Resolution Rate: ${res.post_training_resolution_rate}%\n• Resolution Lift: +${res.resolution_lift}%`
      )
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to apply retraining')
    } finally {
      setApplyingId(null)
    }
  }

  const handleDismiss = async (insightId: string) => {
    try {
      await api.dismissInsight(insightId)
      setInsights((prev) =>
        prev.map((i) => (i.id === insightId ? { ...i, status: 'dismissed' } : i))
      )
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to dismiss insight')
    }
  }

  const handleTriggerScan = async () => {
    setScanning(true)
    try {
      const res = await api.triggerAutoLearningScan()
      alert(
        `Learning Scan Completed!\n• Conversations Analyzed: ${res.conversations_scanned}\n• High-CSAT Patterns Analyzed: ${res.high_csat_patterns_analyzed}\n• New Insights Surfaced: ${res.new_insights_surfaced}`
      )
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Scan failed')
    } finally {
      setScanning(false)
    }
  }

  const handleTestConnection = async (connId: string) => {
    setTestingConnId(connId)
    try {
      const res = await api.testAIConnection(connId)
      alert(res.message || 'Connection verified successfully!')
      loadModelHubData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Connection test failed')
    } finally {
      setTestingConnId(null)
    }
  }

  const handleAssignJobModel = async (jobType: string, connId: string) => {
    try {
      await api.assignModelToJob(jobType, connId)
      loadModelHubData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to route model')
    }
  }

  const handleConnectModel = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newConnName.trim() || !newConnModelId.trim()) return
    setConnectingModel(true)
    try {
      await api.createAIConnection({
        name: newConnName.trim(),
        provider: newConnProvider,
        model_id: newConnModelId.trim(),
        api_key: newConnApiKey.trim(),
        base_url: newConnBaseUrl.trim() || null,
        temperature: newConnTemp,
        is_active: true,
      })
      setShowConnectModal(false)
      setNewConnName('')
      setNewConnApiKey('')
      setNewConnBaseUrl('')
      loadModelHubData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to connect AI model')
    } finally {
      setConnectingModel(false)
    }
  }

  const handleAddCustomRule = () => {
    if (!newRuleText.trim() || !currentCustomPolicy) return
    const updatedRules = [
      ...currentCustomPolicy.rules,
      {
        id: `rule_${Date.now()}`,
        rule: newRuleText.trim(),
        enabled: true,
        category: 'custom',
      },
    ]
    setCurrentCustomPolicy({ ...currentCustomPolicy, rules: updatedRules })
    setNewRuleText('')
  }

  const handleToggleRule = (ruleId: string) => {
    if (!currentCustomPolicy) return
    const updated = currentCustomPolicy.rules.map((r) =>
      r.id === ruleId ? { ...r, enabled: !r.enabled } : r
    )
    setCurrentCustomPolicy({ ...currentCustomPolicy, rules: updated })
  }

  const handleDeleteRule = (ruleId: string) => {
    if (!currentCustomPolicy) return
    const updated = currentCustomPolicy.rules.filter((r) => r.id !== ruleId)
    setCurrentCustomPolicy({ ...currentCustomPolicy, rules: updated })
  }

  const handleAddGuardrail = () => {
    if (!newGuardrailText.trim() || !currentCustomPolicy) return
    const updated = [...currentCustomPolicy.guardrails, newGuardrailText.trim()]
    setCurrentCustomPolicy({ ...currentCustomPolicy, guardrails: updated })
    setNewGuardrailText('')
  }

  const handleDeleteGuardrail = (idx: number) => {
    if (!currentCustomPolicy) return
    const updated = currentCustomPolicy.guardrails.filter((_, i) => i !== idx)
    setCurrentCustomPolicy({ ...currentCustomPolicy, guardrails: updated })
  }

  const handleSavePolicy = async () => {
    if (!currentCustomPolicy) return
    setSavingPolicy(true)
    try {
      const saved = await api.updateCustomPolicy(selectedPolicyJob, {
        tone: currentCustomPolicy.tone,
        guidelines: currentCustomPolicy.guidelines,
        rules: currentCustomPolicy.rules,
        guardrails: currentCustomPolicy.guardrails,
        auto_approval_mode: currentCustomPolicy.auto_approval_mode,
        min_csat_learning_threshold: currentCustomPolicy.min_csat_learning_threshold,
        max_autonomous_refund: currentCustomPolicy.max_autonomous_refund,
      })
      setCurrentCustomPolicy(saved)
      alert(`Custom policy for ${selectedPolicyJob.replace(/_/g, ' ')} updated successfully!`)
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to save policy')
    } finally {
      setSavingPolicy(false)
    }
  }

  // Filtered insights
  const filteredInsights = insights.filter((i) => {
    if (activeCategory !== 'all' && i.category !== activeCategory) return false
    return true
  })

  const coachingInsights = insights.filter((i) => i.category === 'team_coaching')

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-slate-900">
              Self-Improving AI, Model Hub & Custom Policies
            </h1>
            <span className="rounded-full bg-indigo-100 px-2.5 py-0.5 text-xs font-semibold text-indigo-800">
              Our Own Agent
            </span>
          </div>
          <p className="mt-1 text-sm text-slate-500">
            Set custom policies, rules, and guidelines for self-improving AI and other AI jobs, and connect any AI model you want.
          </p>
        </div>

        <div className="flex items-center gap-2">
          {activeView === 'model_hub' ? (
            <button
              onClick={() => setShowConnectModal(true)}
              className="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-700"
            >
              + Connect Any AI Model
            </button>
          ) : (
            <button
              onClick={handleTriggerScan}
              disabled={scanning}
              className="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-700 disabled:opacity-50"
            >
              {scanning ? 'Scanning Conversations...' : '⚡ Scan Conversations for Learnings'}
            </button>
          )}
        </div>
      </div>

      {/* KPI Stats Cards */}
      {stats && (
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
          <div className="rounded-xl border border-indigo-200 bg-indigo-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-indigo-800 uppercase tracking-wider">
              Cumulative Resolution Lift
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-indigo-950">
                +{stats.cumulative_resolution_lift}%
              </span>
              <span className="rounded-full bg-indigo-200 px-2 py-0.5 text-xs font-semibold text-indigo-900">
                Compounding
              </span>
            </div>
            <div className="mt-2 text-xs text-indigo-700">
              {stats.applied_retrainings} retraining cycles deployed
            </div>
          </div>

          <div className="rounded-xl border border-emerald-200 bg-emerald-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-emerald-800 uppercase tracking-wider">
              Active Agent Model
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-emerald-950">
                {jobMapping?.autonomous_agent?.model_name?.split(' ')[0] || 'Claude 3.5'}
              </span>
              <span className="rounded-full bg-emerald-200 px-2 py-0.5 text-xs font-semibold text-emerald-900">
                {stats.current_agent_version}
              </span>
            </div>
            <div className="mt-2 text-xs text-emerald-700">Connected & Configurable</div>
          </div>

          <div className="rounded-xl border border-purple-200 bg-purple-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-purple-800 uppercase tracking-wider">
              Human Rep Exemplars
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-purple-950">
                {stats.human_exemplars_learned}
              </span>
              <span className="text-xs text-purple-700">winning patterns</span>
            </div>
            <div className="mt-2 text-xs text-purple-700">Extracted from 5-star human threads</div>
          </div>

          <div className="rounded-xl border border-amber-200 bg-amber-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-amber-800 uppercase tracking-wider">
              Open AI Suggestions
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-amber-950">
                {stats.open_suggestions}
              </span>
              <span className="text-xs text-amber-700">ready to apply</span>
            </div>
            <div className="mt-2 text-xs text-amber-700">1-click optimization backlog</div>
          </div>
        </div>
      )}

      {/* Main View Tabs */}
      <div className="flex border-b border-slate-200 gap-2">
        <button
          onClick={() => setActiveView('insights')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeView === 'insights'
              ? 'border-indigo-600 text-indigo-600'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span>💡 AI Insights & Rep Exemplars ({insights.length})</span>
        </button>
        <button
          onClick={() => setActiveView('model_hub')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeView === 'model_hub'
              ? 'border-indigo-600 text-indigo-600'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span>⚙️ Model Hub & Custom Policies ({connections.length})</span>
        </button>
        <button
          onClick={() => setActiveView('history')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeView === 'history'
              ? 'border-indigo-600 text-indigo-600'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span>📈 Retraining Timeline ({history.length})</span>
        </button>
        <button
          onClick={() => setActiveView('coaching')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeView === 'coaching'
              ? 'border-indigo-600 text-indigo-600'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span>🤝 Team Coaching ({coachingInsights.length})</span>
        </button>
      </div>

      {loading ? (
        <div className="py-12 text-center text-sm text-slate-500">
          Loading self-improving suite...
        </div>
      ) : error ? (
        <div className="rounded-lg bg-rose-50 p-4 text-sm text-rose-700">{error}</div>
      ) : (
        <>
          {/* VIEW: MODEL HUB & CUSTOM POLICIES */}
          {activeView === 'model_hub' && (
            <div className="space-y-6">
              {/* SECTION 1: PER-JOB MODEL ROUTING */}
              <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
                <div className="flex items-center justify-between border-b border-slate-100 pb-3">
                  <div>
                    <h3 className="text-sm font-bold text-slate-900">
                      AI Job Model Routing (Connect & Route Any Model)
                    </h3>
                    <p className="text-xs text-slate-500">
                      Choose which AI model powers each distinct capability across your inbox and back-office.
                    </p>
                  </div>
                  <span className="rounded bg-indigo-100 px-2 py-0.5 text-xs font-semibold text-indigo-800">
                    Live Router
                  </span>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                  {[
                    {
                      key: 'self_improving',
                      label: '🧠 Self-Improving AI Engine',
                      desc: 'Analyzes human rep threads & continuous retraining',
                    },
                    {
                      key: 'autonomous_agent',
                      label: '🤖 Autonomous Customer Agent',
                      desc: 'Handles live chat, WhatsApp, and multi-channel inquiries',
                    },
                    {
                      key: 'copilot_drafts',
                      label: '✍️ Copilot & Draft Assistant',
                      desc: 'Assists human reps with replies & refund approvals',
                    },
                    {
                      key: 'simulations',
                      label: '🧪 AI Simulation Runner',
                      desc: 'Tests customer dialogues & policy compliance pre-flight',
                    },
                    {
                      key: 'quality_triage',
                      label: '🎯 Quality & Sentiment Triage',
                      desc: 'Monitors CSAT risk, intent classification & SLAs',
                    },
                  ].map((job) => {
                    const current = jobMapping[job.key]
                    return (
                      <div
                        key={job.key}
                        className="rounded-xl border border-slate-200 bg-slate-50 p-3.5 space-y-2"
                      >
                        <div className="flex items-center justify-between">
                          <span className="text-xs font-bold text-slate-900">{job.label}</span>
                        </div>
                        <p className="text-[11px] text-slate-500 line-clamp-2">{job.desc}</p>

                        <div className="pt-1">
                          <label className="text-[10px] font-semibold text-slate-400 block mb-1">
                            Assigned Model:
                          </label>
                          <select
                            value={current?.connection_id || ''}
                            onChange={(e) => handleAssignJobModel(job.key, e.target.value)}
                            className="w-full rounded-lg border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-800 focus:border-indigo-500 focus:outline-none"
                          >
                            {connections.map((c) => (
                              <option key={c.id} value={c.id}>
                                {c.name} ({c.provider})
                              </option>
                            ))}
                          </select>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>

              {/* SECTION 2: CONNECTED AI MODELS GRID */}
              <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
                <div className="flex items-center justify-between border-b border-slate-100 pb-3">
                  <div>
                    <h3 className="text-sm font-bold text-slate-900">
                      Connected AI Models & Inference Providers ({connections.length})
                    </h3>
                    <p className="text-xs text-slate-500">
                      Connect models from Anthropic, OpenAI, Google Gemini, DeepSeek, Groq, Ollama, or custom OpenAI-compatible endpoints.
                    </p>
                  </div>
                  <button
                    onClick={() => setShowConnectModal(true)}
                    className="rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-indigo-700"
                  >
                    + Connect New Model
                  </button>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                  {connections.map((c) => (
                    <div
                      key={c.id}
                      className="rounded-xl border border-slate-200 bg-white p-4 shadow-2xs space-y-3"
                    >
                      <div className="flex items-start justify-between">
                        <div>
                          <span className="rounded bg-slate-100 px-2 py-0.5 text-[10px] font-semibold text-slate-700 uppercase">
                            {c.provider}
                          </span>
                          <h4 className="text-sm font-bold text-slate-900 mt-1">{c.name}</h4>
                          <span className="font-mono text-[11px] text-slate-500 block">
                            {c.model_id}
                          </span>
                        </div>
                        <span
                          className={`rounded-full px-2 py-0.5 text-[10px] font-bold ${
                            c.test_status === 'connected'
                              ? 'bg-emerald-100 text-emerald-800'
                              : 'bg-slate-100 text-slate-600'
                          }`}
                        >
                          {c.test_status === 'connected'
                            ? `✓ ${c.latency_ms ?? 150}ms`
                            : 'Untested'}
                        </span>
                      </div>

                      {/* Assigned Jobs */}
                      <div className="text-[11px] space-y-1">
                        <span className="text-slate-400">Assigned Jobs:</span>
                        <div className="flex flex-wrap gap-1">
                          {c.assigned_jobs && c.assigned_jobs.length > 0 ? (
                            c.assigned_jobs.map((j) => (
                              <span
                                key={j}
                                className="rounded bg-indigo-50 border border-indigo-200 px-1.5 py-0.5 text-[10px] font-medium text-indigo-700"
                              >
                                {j.replace(/_/g, ' ')}
                              </span>
                            ))
                          ) : (
                            <span className="text-slate-400 italic text-[10px]">
                              None (available in pool)
                            </span>
                          )}
                        </div>
                      </div>

                      <div className="flex items-center justify-between pt-2 border-t border-slate-100 text-xs">
                        <span className="text-slate-400 font-mono text-[10px]">
                          Key: {c.api_key_masked}
                        </span>
                        <button
                          onClick={() => handleTestConnection(c.id)}
                          disabled={testingConnId === c.id}
                          className="rounded bg-slate-100 hover:bg-slate-200 px-2 py-1 text-[11px] font-semibold text-slate-700 transition-colors disabled:opacity-50"
                        >
                          {testingConnId === c.id ? 'Testing...' : 'Test Connection'}
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* SECTION 3: CUSTOM POLICIES, RULES & GUIDELINES EDITOR */}
              <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-5">
                <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between border-b border-slate-100 pb-3 gap-2">
                  <div>
                    <h3 className="text-sm font-bold text-slate-900">
                      Custom Policies, Rules & Guidelines Editor
                    </h3>
                    <p className="text-xs text-slate-500">
                      Configure custom operational guidelines, dynamic rules, and guardrails for each AI job.
                    </p>
                  </div>

                  {/* Job Type Selector Pills */}
                  <div className="flex flex-wrap gap-1">
                    {[
                      { key: 'self_improving', label: 'Self-Improving AI' },
                      { key: 'autonomous_agent', label: 'Autonomous Agent' },
                      { key: 'copilot_drafts', label: 'Copilot Drafts' },
                      { key: 'simulations', label: 'Simulations' },
                    ].map((j) => (
                      <button
                        key={j.key}
                        onClick={() => setSelectedPolicyJob(j.key)}
                        className={`rounded-lg px-2.5 py-1 text-xs font-semibold transition-colors ${
                          selectedPolicyJob === j.key
                            ? 'bg-indigo-600 text-white'
                            : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                        }`}
                      >
                        {j.label}
                      </button>
                    ))}
                  </div>
                </div>

                {loadingPolicy || !currentCustomPolicy ? (
                  <div className="py-8 text-center text-xs text-slate-400">Loading policy...</div>
                ) : (
                  <div className="space-y-4 text-xs">
                    {/* Tone of Voice & Guidelines */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <div>
                        <label className="font-semibold text-slate-700 block mb-1">
                          Tone of Voice & Persona:
                        </label>
                        <select
                          value={currentCustomPolicy.tone}
                          onChange={(e) =>
                            setCurrentCustomPolicy({ ...currentCustomPolicy, tone: e.target.value })
                          }
                          className="w-full rounded-lg border border-slate-300 bg-white p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-none"
                        >
                          <option value="empathetic, helpful & professional">
                            Empathetic & Professional (Recommended)
                          </option>
                          <option value="analytical & constructive">
                            Analytical & Constructive (Ideal for Self-Improving)
                          </option>
                          <option value="concise, direct & friendly">
                            Concise & Friendly (Fast replies)
                          </option>
                          <option value="casual & conversational">Casual & Conversational</option>
                          <option value="strict benchmark evaluator">Strict Benchmark Evaluator</option>
                        </select>
                      </div>

                      {/* Self-improving controls */}
                      {selectedPolicyJob === 'self_improving' && (
                        <>
                          <div>
                            <label className="font-semibold text-slate-700 block mb-1">
                              Min CSAT for Human Exemplar Learning:
                            </label>
                            <input
                              type="number"
                              step="0.1"
                              min="3.0"
                              max="5.0"
                              value={currentCustomPolicy.min_csat_learning_threshold}
                              onChange={(e) =>
                                setCurrentCustomPolicy({
                                  ...currentCustomPolicy,
                                  min_csat_learning_threshold: parseFloat(e.target.value) || 4.0,
                                })
                              }
                              className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
                            />
                          </div>

                          <div>
                            <label className="font-semibold text-slate-700 block mb-1">
                              Retraining Approval Mode:
                            </label>
                            <select
                              value={currentCustomPolicy.auto_approval_mode}
                              onChange={(e) =>
                                setCurrentCustomPolicy({
                                  ...currentCustomPolicy,
                                  auto_approval_mode: e.target.value,
                                })
                              }
                              className="w-full rounded-lg border border-slate-300 bg-white p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-none"
                            >
                              <option value="require_manager_review">
                                Require Manager Review (Recommended)
                              </option>
                              <option value="auto_apply_verified">
                                Auto-Apply 5-Star Verified Exemplars
                              </option>
                            </select>
                          </div>
                        </>
                      )}
                    </div>

                    {/* Operational Guidelines Textarea */}
                    <div>
                      <label className="font-semibold text-slate-700 block mb-1">
                        Operational Instructions & Custom Guidelines:
                      </label>
                      <textarea
                        rows={3}
                        value={currentCustomPolicy.guidelines}
                        onChange={(e) =>
                          setCurrentCustomPolicy({
                            ...currentCustomPolicy,
                            guidelines: e.target.value,
                          })
                        }
                        className="w-full rounded-xl border border-slate-300 p-2.5 text-xs text-slate-800 focus:border-indigo-500 focus:outline-none"
                        placeholder="Define operational guidelines for this AI job..."
                      />
                    </div>

                    {/* Dynamic Rules Manager */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="font-semibold text-slate-700">
                          Active Custom Rules ({currentCustomPolicy.rules.length}):
                        </label>
                        <span className="text-[11px] text-slate-400">
                          Toggle or add specific rules
                        </span>
                      </div>

                      <div className="space-y-1.5 max-h-48 overflow-y-auto rounded-xl border border-slate-200 p-3 bg-slate-50">
                        {currentCustomPolicy.rules.map((rule) => (
                          <div
                            key={rule.id}
                            className="flex items-center justify-between rounded-lg bg-white p-2 border border-slate-200"
                          >
                            <div className="flex items-center gap-2">
                              <input
                                type="checkbox"
                                checked={rule.enabled}
                                onChange={() => handleToggleRule(rule.id)}
                                className="rounded text-indigo-600 focus:ring-indigo-500"
                              />
                              <span
                                className={`text-xs ${
                                  rule.enabled ? 'text-slate-800' : 'text-slate-400 line-through'
                                }`}
                              >
                                {rule.rule}
                              </span>
                            </div>
                            <button
                              onClick={() => handleDeleteRule(rule.id)}
                              className="text-slate-400 hover:text-rose-600 text-xs px-1"
                            >
                              ✕
                            </button>
                          </div>
                        ))}
                      </div>

                      <div className="flex gap-2">
                        <input
                          type="text"
                          value={newRuleText}
                          onChange={(e) => setNewRuleText(e.target.value)}
                          placeholder="Type a new custom rule (e.g. Always address customer by first name)..."
                          className="flex-1 rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
                        />
                        <button
                          type="button"
                          onClick={handleAddCustomRule}
                          className="rounded-lg bg-slate-900 px-3 py-1.5 text-xs font-semibold text-white hover:bg-slate-800"
                        >
                          + Add Rule
                        </button>
                      </div>
                    </div>

                    {/* Guardrails Manager */}
                    <div className="space-y-2">
                      <label className="font-semibold text-slate-700 block">
                        Forbidden Actions & Guardrails ({currentCustomPolicy.guardrails.length}):
                      </label>
                      <div className="space-y-1.5 max-h-40 overflow-y-auto rounded-xl border border-rose-200 p-3 bg-rose-50/30">
                        {currentCustomPolicy.guardrails.map((g, idx) => (
                          <div
                            key={idx}
                            className="flex items-center justify-between rounded-lg bg-white p-2 border border-rose-200 text-xs text-rose-950 font-medium"
                          >
                            <span>🛡️ {g}</span>
                            <button
                              onClick={() => handleDeleteGuardrail(idx)}
                              className="text-slate-400 hover:text-rose-600 text-xs px-1"
                            >
                              ✕
                            </button>
                          </div>
                        ))}
                      </div>

                      <div className="flex gap-2">
                        <input
                          type="text"
                          value={newGuardrailText}
                          onChange={(e) => setNewGuardrailText(e.target.value)}
                          placeholder="Type a guardrail (e.g. Never disclose internal supplier costs)..."
                          className="flex-1 rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
                        />
                        <button
                          type="button"
                          onClick={handleAddGuardrail}
                          className="rounded-lg bg-rose-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-rose-700"
                        >
                          + Add Guardrail
                        </button>
                      </div>
                    </div>

                    <div className="pt-3 border-t border-slate-200 flex justify-end">
                      <button
                        onClick={handleSavePolicy}
                        disabled={savingPolicy}
                        className="rounded-xl bg-indigo-600 px-5 py-2 text-xs font-bold text-white shadow-sm hover:bg-indigo-700 disabled:opacity-50"
                      >
                        {savingPolicy ? 'Saving Policy...' : 'Save Policy & Recompile Prompt'}
                      </button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* VIEW: INSIGHTS & EXEMPLARS */}
          {activeView === 'insights' && (
            <div className="space-y-4">
              {/* Category Filter Pills */}
              <div className="flex flex-wrap items-center gap-1.5">
                <button
                  onClick={() => setActiveCategory('all')}
                  className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
                    activeCategory === 'all'
                      ? 'bg-slate-900 text-white'
                      : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-100'
                  }`}
                >
                  All Suggestions ({insights.length})
                </button>
                <button
                  onClick={() => setActiveCategory('human_exemplar')}
                  className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors flex items-center gap-1 ${
                    activeCategory === 'human_exemplar'
                      ? 'bg-purple-700 text-white'
                      : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-100'
                  }`}
                >
                  <span>⭐ Human Rep Exemplars</span>
                </button>
                <button
                  onClick={() => setActiveCategory('trend')}
                  className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
                    activeCategory === 'trend'
                      ? 'bg-blue-700 text-white'
                      : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-100'
                  }`}
                >
                  Emerging Customer Trends
                </button>
                <button
                  onClick={() => setActiveCategory('policy_refinement')}
                  className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
                    activeCategory === 'policy_refinement'
                      ? 'bg-emerald-700 text-white'
                      : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-100'
                  }`}
                >
                  Policy Refinements
                </button>
                <button
                  onClick={() => setActiveCategory('team_coaching')}
                  className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
                    activeCategory === 'team_coaching'
                      ? 'bg-amber-700 text-white'
                      : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-100'
                  }`}
                >
                  Team Coaching
                </button>
              </div>

              {/* Insights List */}
              <div className="space-y-4">
                {filteredInsights.map((ins) => (
                  <div
                    key={ins.id}
                    className={`rounded-2xl border p-5 shadow-sm transition-all space-y-3 ${
                      ins.category === 'human_exemplar'
                        ? 'border-purple-200 bg-purple-50/20'
                        : ins.category === 'trend'
                        ? 'border-blue-200 bg-blue-50/20'
                        : 'border-slate-200 bg-white'
                    }`}
                  >
                    <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                      <div className="flex items-center gap-2">
                        {ins.category === 'human_exemplar' ? (
                          <span className="rounded-md bg-purple-100 px-2 py-0.5 text-xs font-bold text-purple-800">
                            ⭐ Human Rep Exemplar
                          </span>
                        ) : ins.category === 'trend' ? (
                          <span className="rounded-md bg-blue-100 px-2 py-0.5 text-xs font-bold text-blue-800">
                            📊 Emerging Trend
                          </span>
                        ) : ins.category === 'team_coaching' ? (
                          <span className="rounded-md bg-amber-100 px-2 py-0.5 text-xs font-bold text-amber-800">
                            🤝 Team Coaching
                          </span>
                        ) : (
                          <span className="rounded-md bg-emerald-100 px-2 py-0.5 text-xs font-bold text-emerald-800">
                            📋 Policy Refinement
                          </span>
                        )}

                        <span className="rounded-md bg-emerald-50 border border-emerald-200 px-2 py-0.5 text-xs font-semibold text-emerald-800">
                          +{ins.estimated_resolution_lift}% Est. Resolution Lift
                        </span>

                        <span className="text-xs text-slate-400">
                          Impact: {ins.impact_score}/10
                        </span>
                      </div>

                      <div className="flex items-center gap-2">
                        {ins.status === 'applied' ? (
                          <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-bold text-emerald-800">
                            ✓ Retrained into Agent
                          </span>
                        ) : ins.status === 'dismissed' ? (
                          <span className="rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-medium text-slate-500">
                            Dismissed
                          </span>
                        ) : (
                          <div className="flex items-center gap-2">
                            <button
                              onClick={() => handleDismiss(ins.id)}
                              className="text-xs text-slate-400 hover:text-slate-600 px-2 py-1"
                            >
                              Dismiss
                            </button>
                            <button
                              onClick={() => handleApplyRetraining(ins.id)}
                              disabled={applyingId === ins.id}
                              className="rounded-lg bg-indigo-600 px-3.5 py-1.5 text-xs font-bold text-white shadow-sm hover:bg-indigo-700 disabled:opacity-50"
                            >
                              {applyingId === ins.id
                                ? 'Retraining...'
                                : ins.category === 'human_exemplar'
                                ? '⚡ Retrain Agent with this Pattern'
                                : '⚡ Apply & Retrain Agent'}
                            </button>
                          </div>
                        )}
                      </div>
                    </div>

                    <div>
                      <h3 className="text-base font-bold text-slate-900">{ins.title}</h3>
                      <p className="mt-1 text-xs text-slate-600 leading-relaxed">
                        {ins.description}
                      </p>
                    </div>

                    {/* Human Rep Exemplar Highlight Quote Box */}
                    {ins.exemplar_content && (
                      <div className="rounded-xl border border-purple-200 bg-purple-50/70 p-3.5 text-xs space-y-1.5">
                        <div className="flex items-center justify-between text-purple-900 font-semibold text-[11px]">
                          <span>
                            Top Rep Winning Response (from {ins.human_rep_name || 'Top Human Rep'}):
                          </span>
                          <span className="bg-purple-200 px-1.5 py-0.2 rounded text-[10px]">
                            5.0 ★ CSAT Verified
                          </span>
                        </div>
                        <p className="text-purple-950 font-medium italic">
                          "{ins.exemplar_content}"
                        </p>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* VIEW: RETRAINING HISTORY */}
          {activeView === 'history' && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-slate-900">
                    Agent Retraining Iteration Timeline
                  </h3>
                  <p className="text-xs text-slate-500">
                    Chronological audit log of how our Agent has improved across versions with resolution rate lift metrics.
                  </p>
                </div>
              </div>

              <div className="relative border-l-2 border-indigo-200 ml-4 space-y-6 pl-6 py-2">
                {history.map((ev) => (
                  <div key={ev.id} className="relative space-y-2">
                    <div className="absolute -left-[31px] top-1.5 h-3.5 w-3.5 rounded-full border-2 border-white bg-indigo-600 shadow-xs"></div>

                    <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-3">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="rounded bg-indigo-100 px-2 py-0.5 text-xs font-bold text-indigo-800">
                            v{ev.version_from} ➔ v{ev.version_to}
                          </span>
                          <h4 className="text-sm font-bold text-slate-900">{ev.title}</h4>
                        </div>
                        <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-bold text-emerald-800">
                          +{ev.resolution_lift}% Lift
                        </span>
                      </div>

                      <div className="rounded-xl bg-slate-50 border border-slate-200 p-3 flex items-center justify-between text-xs">
                        <div className="flex items-center gap-4">
                          <div>
                            <span className="text-slate-400 block text-[11px]">
                              Baseline Resolution Rate:
                            </span>
                            <span className="font-bold text-slate-700">
                              {ev.baseline_resolution_rate}%
                            </span>
                          </div>
                          <span className="text-slate-300 text-lg">➔</span>
                          <div>
                            <span className="text-slate-400 block text-[11px]">
                              Post-Retraining Resolution Rate:
                            </span>
                            <span className="font-bold text-emerald-700">
                              {ev.post_training_resolution_rate}%
                            </span>
                          </div>
                        </div>

                        <span className="text-slate-400 text-xs">
                          Deployed on{' '}
                          {new Date(ev.created_at).toLocaleDateString([], {
                            month: 'short',
                            day: 'numeric',
                            year: 'numeric',
                          })}
                        </span>
                      </div>

                      <div>
                        <span className="text-xs font-semibold text-slate-700 block mb-1">
                          Changes & Learnings Incorporated:
                        </span>
                        <div className="space-y-1">
                          {ev.changes_applied.map((ch, idx) => (
                            <div
                              key={idx}
                              className="rounded bg-slate-50 p-2 text-xs text-slate-700 flex items-center gap-2"
                            >
                              <span className="text-indigo-600 font-bold">✓</span>
                              <span>{ch.detail || ch.title || JSON.stringify(ch)}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* VIEW: HUMAN TEAM COACHING */}
          {activeView === 'coaching' && (
            <div className="space-y-4">
              <div>
                <h3 className="text-sm font-bold text-slate-900">
                  Human Team Coaching & Assistance
                </h3>
                <p className="text-xs text-slate-500">
                  Suggestions and data assistance from our Agent make your human team better, creating a mutual improvement loop.
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {coachingInsights.map((c) => (
                  <div
                    key={c.id}
                    className="rounded-2xl border border-amber-200 bg-amber-50/20 p-5 shadow-sm space-y-3"
                  >
                    <div className="flex items-center justify-between">
                      <span className="rounded bg-amber-100 px-2 py-0.5 text-xs font-bold text-amber-800">
                        🤝 Human Team Recommendation
                      </span>
                      <span className="rounded bg-emerald-100 px-2 py-0.5 text-xs font-bold text-emerald-800">
                        +{c.estimated_resolution_lift}% CSAT Lift
                      </span>
                    </div>

                    <h4 className="text-sm font-bold text-slate-900">{c.title}</h4>
                    <p className="text-xs text-slate-600 leading-relaxed">{c.description}</p>

                    <div className="rounded-xl bg-white border border-amber-200 p-3 text-xs text-slate-800">
                      <span className="font-semibold text-amber-900 block mb-1">
                        Coaching Action for Reps:
                      </span>
                      <p>{c.action_payload?.coaching_tip || c.description}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      )}

      {/* Modal: Connect Any AI Model */}
      {showConnectModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl bg-white p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between border-b border-slate-200 pb-3">
              <h3 className="text-lg font-bold text-slate-900">Connect Any AI Model</h3>
              <button
                onClick={() => setShowConnectModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleConnectModel} className="space-y-4">
              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">
                  Connection Display Name:
                </label>
                <input
                  type="text"
                  required
                  value={newConnName}
                  onChange={(e) => setNewConnName(e.target.value)}
                  placeholder="e.g. DeepSeek R1 via OpenRouter, Groq Llama 3.3 70B"
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-slate-700 block mb-1">
                    Provider:
                  </label>
                  <select
                    value={newConnProvider}
                    onChange={(e) => setNewConnProvider(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 bg-white p-2 text-xs focus:border-indigo-500 focus:outline-none"
                  >
                    <option value="anthropic">Anthropic Claude</option>
                    <option value="openai">OpenAI (GPT-4o / o1 / o3)</option>
                    <option value="google">Google Gemini</option>
                    <option value="deepseek">DeepSeek (V3 / R1)</option>
                    <option value="groq">Groq (Ultra-fast)</option>
                    <option value="ollama">Ollama / Local LLM</option>
                    <option value="custom">Custom OpenAI-Compatible</option>
                  </select>
                </div>

                <div>
                  <label className="text-xs font-semibold text-slate-700 block mb-1">
                    Model Identifier:
                  </label>
                  <input
                    type="text"
                    required
                    value={newConnModelId}
                    onChange={(e) => setNewConnModelId(e.target.value)}
                    placeholder="e.g. deepseek-reasoner, gpt-4o, llama3.3:70b"
                    className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none"
                  />
                </div>
              </div>

              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">
                  API Key (leave blank for local Ollama):
                </label>
                <input
                  type="password"
                  value={newConnApiKey}
                  onChange={(e) => setNewConnApiKey(e.target.value)}
                  placeholder="sk-..."
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none font-mono"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">
                  Custom Base URL (optional):
                </label>
                <input
                  type="text"
                  value={newConnBaseUrl}
                  onChange={(e) => setNewConnBaseUrl(e.target.value)}
                  placeholder="e.g. http://localhost:11434/v1 or https://openrouter.ai/api/v1"
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-indigo-500 focus:outline-none font-mono"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-slate-700 block mb-1">
                  Temperature: {newConnTemp}
                </label>
                <input
                  type="range"
                  min="0.0"
                  max="1.5"
                  step="0.05"
                  value={newConnTemp}
                  onChange={(e) => setNewConnTemp(parseFloat(e.target.value))}
                  className="w-full"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-200">
                <button
                  type="button"
                  onClick={() => setShowConnectModal(false)}
                  className="rounded-lg px-3 py-1.5 text-xs font-semibold text-slate-600 hover:bg-slate-100"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={connectingModel}
                  className="rounded-lg bg-indigo-600 px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-indigo-700 disabled:opacity-50"
                >
                  {connectingModel ? 'Connecting...' : 'Connect Model'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
