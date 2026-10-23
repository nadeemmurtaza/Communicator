import { useEffect, useState } from 'react'
import {
  api,
  type AgentPolicy,
  type SandboxChatResult,
  type SimulationRun,
  type SimulationScenario,
  type SimulationStats,
} from '../lib/api'

export function Simulations() {
  const [policies, setPolicies] = useState<AgentPolicy[]>([])
  const [scenarios, setScenarios] = useState<SimulationScenario[]>([])
  const [stats, setStats] = useState<SimulationStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Tabs: 'sandbox' | 'scenarios' | 'policies'
  const [activeTab, setActiveTab] = useState<'sandbox' | 'scenarios' | 'policies'>('sandbox')

  // Interactive Sandbox state
  const [sandboxMessage, setSandboxMessage] = useState('')
  const [sandboxHistory, setSandboxHistory] = useState<
    Array<{ role: 'customer' | 'agent'; text: string; details?: SandboxChatResult }>
  >([
    {
      role: 'agent',
      text: "Hello! I am your Autonomous Agent in Training. Test me on your refund policies, shipping delays, VIP protocols, or complex returns. What's on your mind?",
    },
  ])
  const [sendingSandbox, setSendingSandbox] = useState(false)
  const [selectedSandboxPolicy, setSelectedSandboxPolicy] = useState<string>('')
  const [latestSandboxResult, setLatestSandboxResult] = useState<SandboxChatResult | null>(null)

  // Active Simulation Run Result Modal
  const [activeRunResult, setActiveRunResult] = useState<SimulationRun | null>(null)
  const [runningScenarioId, setRunningScenarioId] = useState<string | null>(null)
  const [batchRunning, setBatchRunning] = useState(false)

  // Create Policy Modal
  const [showCreatePolicyModal, setShowCreatePolicyModal] = useState(false)
  const [policyName, setPolicyName] = useState('')
  const [policyCategory, setPolicyCategory] = useState('refunds')
  const [policyDescription, setPolicyDescription] = useState('')
  const [policyVersion, setPolicyVersion] = useState('1.0')
  const [guardrailText, setGuardrailText] = useState('')
  const [creatingPolicy, setCreatingPolicy] = useState(false)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [policiesData, scenariosData, statsData] = await Promise.all([
        api.listAgentPolicies(),
        api.listSimulationScenarios(),
        api.getSimulationStats(),
      ])
      setPolicies(policiesData)
      setScenarios(scenariosData)
      setStats(statsData)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load simulation environment')
    } finally {
      setLoading(false)
    }
  }

  const handleSendSandboxMessage = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!sandboxMessage.trim() || sendingSandbox) return

    const userText = sandboxMessage.trim()
    setSandboxMessage('')
    setSandboxHistory((prev) => [...prev, { role: 'customer', text: userText }])
    setSendingSandbox(true)

    try {
      const res = await api.sandboxChat(
        userText,
        [],
        selectedSandboxPolicy || undefined
      )
      setLatestSandboxResult(res)
      setSandboxHistory((prev) => [
        ...prev,
        { role: 'agent', text: res.reply, details: res },
      ])
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to get sandbox reply')
    } finally {
      setSendingSandbox(false)
    }
  }

  const handleRunSimulation = async (scenarioId: string) => {
    setRunningScenarioId(scenarioId)
    try {
      const run = await api.runSimulation(scenarioId)
      setActiveRunResult(run)
      // Refresh scenarios to reflect latest run
      const updatedScenarios = await api.listSimulationScenarios()
      setScenarios(updatedScenarios)
      const updatedStats = await api.getSimulationStats()
      setStats(updatedStats)
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to execute simulation')
    } finally {
      setRunningScenarioId(null)
    }
  }

  const handleRunBatchSuite = async () => {
    setBatchRunning(true)
    try {
      const res = await api.runBatchSimulations()
      alert(
        `Batch Simulation Suite Completed!\n• Total Scenarios: ${res.total_scenarios_tested}\n• Pass Rate: ${res.pass_rate_percentage}%\n• Average Score: ${res.average_score}%`
      )
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to run batch simulations')
    } finally {
      setBatchRunning(false)
    }
  }

  const handleCreatePolicy = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!policyName.trim()) return
    setCreatingPolicy(true)
    try {
      const guardrailsList = guardrailText
        .split('\n')
        .map((g) => g.trim())
        .filter(Boolean)

      await api.createAgentPolicy({
        name: policyName.trim(),
        category: policyCategory,
        description: policyDescription.trim(),
        version: policyVersion.trim() || '1.0',
        guardrails: guardrailsList,
        process_steps: [
          {
            step: 1,
            title: 'Initial Greeting & Assessment',
            instruction: 'Verify customer issue and check applicable policy constraints.',
          },
          {
            step: 2,
            title: 'Resolution Execution',
            instruction: 'Execute appropriate solution according to business thresholds.',
          },
        ],
      })
      setShowCreatePolicyModal(false)
      setPolicyName('')
      setPolicyDescription('')
      setGuardrailText('')
      loadData()
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to create policy')
    } finally {
      setCreatingPolicy(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
              Agent Training & AI Conversation Simulations
            </h1>
            <span className="rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-semibold text-blue-800">
              Our Own Agent
            </span>
          </div>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Train our autonomous Agent on your SOPs and policies. Use AI-driven Simulations to test fully simulated customer conversations before going live.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRunBatchSuite}
            disabled={batchRunning}
            className="inline-flex items-center justify-center rounded-lg bg-emerald-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-emerald-700 disabled:opacity-50"
          >
            {batchRunning ? 'Running Test Suite...' : '⚡ Run Batch Test Suite'}
          </button>
        </div>
      </div>

      {/* KPI Stats Cards */}
      {stats && (
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
          <div className="rounded-xl border border-emerald-200 bg-emerald-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-emerald-800 uppercase tracking-wider">
              Simulation Pass Rate
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-emerald-950">
                {stats.pass_rate_percentage}%
              </span>
              <span className="rounded-full bg-emerald-200 px-2 py-0.5 text-xs font-semibold text-emerald-900">
                Verified
              </span>
            </div>
            <div className="mt-2 text-xs text-emerald-700">
              {stats.total_simulation_runs} test runs executed
            </div>
          </div>

          <div className="rounded-xl border border-blue-200 bg-blue-50/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-blue-800 uppercase tracking-wider">
              Overall Benchmark Score
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-blue-950">
                {stats.average_overall_score}%
              </span>
              <span className="text-xs text-blue-700">avg quality</span>
            </div>
            <div className="mt-2 text-xs text-blue-700">Adherence, Empathy & Accuracy</div>
          </div>

          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-accent-soft)]/50 p-4 shadow-sm">
            <div className="text-xs font-medium text-indigo-800 uppercase tracking-wider">
              Trained SOP Policies
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-indigo-950">{stats.total_policies}</span>
              <span className="text-xs text-[var(--color-accent)]">active SOPs</span>
            </div>
            <div className="mt-2 text-xs text-[var(--color-accent)]">Refunds, Shipping, Returns, VIP</div>
          </div>

          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
            <div className="text-xs font-medium text-[var(--color-ink-muted)] uppercase tracking-wider">
              Pre-flight Test Scenarios
            </div>
            <div className="mt-2 flex items-baseline gap-2">
              <span className="text-2xl font-bold text-[var(--color-ink)]">{stats.total_scenarios}</span>
              <span className="text-xs text-[var(--color-ink-faint)]">scenarios</span>
            </div>
            <div className="mt-2 text-xs text-[var(--color-ink-muted)]">Autonomous & Escalation cases</div>
          </div>
        </div>
      )}

      {/* Tabs Navigation */}
      <div className="flex border-b border-[var(--color-border)] gap-2">
        <button
          onClick={() => setActiveTab('sandbox')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeTab === 'sandbox'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
          }`}
        >
          <span>🧪 Interactive Policy Sandbox</span>
        </button>
        <button
          onClick={() => setActiveTab('scenarios')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeTab === 'scenarios'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
          }`}
        >
          <span>🎯 Scenarios & Test Suite ({scenarios.length})</span>
        </button>
        <button
          onClick={() => setActiveTab('policies')}
          className={`px-4 py-2.5 text-sm font-semibold transition-colors border-b-2 flex items-center gap-2 ${
            activeTab === 'policies'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-slate-800'
          }`}
        >
          <span>📋 Agent Training SOPs ({policies.length})</span>
        </button>
      </div>

      {loading ? (
        <div className="py-12 text-center text-sm text-[var(--color-ink-muted)]">
          Loading simulation environment...
        </div>
      ) : error ? (
        <div className="rounded-lg bg-rose-50 p-4 text-sm text-rose-700">{error}</div>
      ) : (
        <>
          {/* TAB 1: INTERACTIVE POLICY SANDBOX */}
          {activeTab === 'sandbox' && (
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              {/* Left 2 Cols: Chat Console */}
              <div className="lg:col-span-2 flex flex-col h-[580px] rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
                <div className="flex items-center justify-between border-b border-[var(--color-border)] px-4 py-3 bg-[var(--color-page)]">
                  <div className="flex items-center gap-2">
                    <span className="h-2.5 w-2.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    <span className="text-xs font-bold text-slate-800">
                      Captain AI Agent - Live Sandbox Runner
                    </span>
                  </div>
                  <select
                    value={selectedSandboxPolicy}
                    onChange={(e) => setSelectedSandboxPolicy(e.target.value)}
                    className="rounded-lg border border-slate-300 bg-[var(--color-surface)] px-2.5 py-1 text-xs text-[var(--color-ink)] focus:outline-none"
                  >
                    <option value="">Auto-detect matching SOP</option>
                    {policies.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Conversation History */}
                <div className="flex-1 overflow-y-auto p-4 space-y-3 bg-[var(--color-page)]/30">
                  {sandboxHistory.map((item, idx) => (
                    <div
                      key={idx}
                      className={`flex flex-col ${
                        item.role === 'customer' ? 'items-end' : 'items-start'
                      }`}
                    >
                      <span className="text-[10px] font-semibold text-[var(--color-ink-faint)] mb-1">
                        {item.role === 'customer' ? 'Customer (You in Test Mode)' : 'Captain AI Agent'}
                      </span>
                      <div
                        className={`max-w-lg rounded-2xl px-4 py-2.5 text-xs leading-relaxed ${
                          item.role === 'customer'
                            ? 'bg-blue-600 text-white rounded-br-xs'
                            : 'bg-[var(--color-surface)] border border-[var(--color-border)] text-slate-800 shadow-2xs rounded-bl-xs'
                        }`}
                      >
                        <p className="whitespace-pre-wrap">{item.text}</p>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Quick Test Prompt Shortcuts */}
                <div className="px-4 py-2 bg-slate-100/70 border-t border-[var(--color-border)] flex flex-wrap items-center gap-1.5 text-[11px] text-[var(--color-ink-muted)]">
                  <span className="font-semibold text-[var(--color-ink-muted)]">Quick Prompts:</span>
                  <button
                    type="button"
                    onClick={() =>
                      setSandboxMessage('I need a refund for $35 perfume that arrived broken.')
                    }
                    className="rounded bg-[var(--color-surface)] border border-[var(--color-border)] px-2 py-0.5 hover:bg-[var(--color-page)] text-blue-700 font-medium"
                  >
                    Refund &lt;$50
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      setSandboxMessage(
                        'I want a refund for a $280 engraved gold watch from order #ORD-3321.'
                      )
                    }
                    className="rounded bg-[var(--color-surface)] border border-[var(--color-border)] px-2 py-0.5 hover:bg-[var(--color-page)] text-blue-700 font-medium"
                  >
                    Refund &gt;$150 (Approval)
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      setSandboxMessage('My express delivery is 3 days delayed. Can I get a refund?')
                    }
                    className="rounded bg-[var(--color-surface)] border border-[var(--color-border)] px-2 py-0.5 hover:bg-[var(--color-page)] text-blue-700 font-medium"
                  >
                    Express Shipping Delay
                  </button>
                </div>

                {/* Input form */}
                <form
                  onSubmit={handleSendSandboxMessage}
                  className="p-3 border-t border-[var(--color-border)] bg-[var(--color-surface)] flex gap-2"
                >
                  <input
                    type="text"
                    value={sandboxMessage}
                    onChange={(e) => setSandboxMessage(e.target.value)}
                    placeholder="Type a test customer message or scenario query..."
                    className="flex-1 rounded-xl border border-slate-300 px-3.5 py-2 text-xs focus:border-blue-500 focus:outline-none"
                  />
                  <button
                    type="submit"
                    disabled={!sandboxMessage.trim() || sendingSandbox}
                    className="rounded-xl bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
                  >
                    {sendingSandbox ? 'Evaluating...' : 'Send Test'}
                  </button>
                </form>
              </div>

              {/* Right Col: Real-time Agent Policy Inspector */}
              <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm space-y-4">
                <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
                  <h3 className="text-xs font-bold text-slate-800 uppercase tracking-wider flex items-center gap-1.5">
                    <span>🔍 Real-time Policy Inspector</span>
                  </h3>
                  <span className="rounded bg-slate-100 px-2 py-0.5 text-[10px] font-semibold text-[var(--color-ink-muted)]">
                    Live Diagnostics
                  </span>
                </div>

                {latestSandboxResult ? (
                  <div className="space-y-3 text-xs">
                    <div>
                      <span className="text-[var(--color-ink-faint)] font-medium block mb-1">
                        Matched SOP Policy:
                      </span>
                      <div className="rounded-lg bg-blue-50 border border-blue-200 p-2.5 font-semibold text-blue-900">
                        {latestSandboxResult.policy_matched}
                      </div>
                    </div>

                    <div>
                      <span className="text-[var(--color-ink-faint)] font-medium block mb-1">
                        Active Rules Triggered:
                      </span>
                      <div className="space-y-1">
                        {latestSandboxResult.matched_rules.map((rule, i) => (
                          <div
                            key={i}
                            className="rounded bg-[var(--color-page)] border border-[var(--color-border)] p-2 text-slate-800 font-medium"
                          >
                            ✓ {rule}
                          </div>
                        ))}
                      </div>
                    </div>

                    <div>
                      <span className="text-[var(--color-ink-faint)] font-medium block mb-1">
                        Guardrails Enforced:
                      </span>
                      <div className="space-y-1">
                        {latestSandboxResult.guardrails_applied.map((g, i) => (
                          <div
                            key={i}
                            className="rounded bg-emerald-50 border border-emerald-200 p-2 text-emerald-900 font-medium"
                          >
                            🛡️ {g}
                          </div>
                        ))}
                      </div>
                    </div>

                    <div>
                      <span className="text-[var(--color-ink-faint)] font-medium block mb-1">
                        Escalation Flag:
                      </span>
                      <span
                        className={`inline-block rounded px-2 py-1 text-xs font-semibold ${
                          latestSandboxResult.escalated_for_approval
                            ? 'bg-amber-100 text-amber-800'
                            : 'bg-emerald-100 text-emerald-800'
                        }`}
                      >
                        {latestSandboxResult.escalated_for_approval
                          ? '⚠️ Teammate Approval Triggered'
                          : '✅ Autonomous Resolution Allowed'}
                      </span>
                    </div>
                  </div>
                ) : (
                  <div className="py-12 text-center text-xs text-[var(--color-ink-faint)]">
                    Send a test message on the left to see live policy evaluation and guardrails.
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 2: AI SCENARIOS & BATCH RUNNER */}
          {activeTab === 'scenarios' && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Pre-flight AI Simulation Scenarios
                  </h3>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    Test full customer dialogues end-to-end to verify resolution accuracy before going live.
                  </p>
                </div>
                <button
                  onClick={handleRunBatchSuite}
                  disabled={batchRunning}
                  className="rounded-lg bg-emerald-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-emerald-700 disabled:opacity-50"
                >
                  {batchRunning ? 'Running Test Suite...' : '⚡ Run All Scenarios'}
                </button>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {scenarios.map((s) => (
                  <div
                    key={s.id}
                    className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-3"
                  >
                    <div className="flex items-start justify-between">
                      <div>
                        <span className="rounded bg-slate-100 px-2 py-0.5 text-[10px] font-semibold text-[var(--color-ink-muted)] uppercase">
                          {s.category}
                        </span>
                        <h4 className="mt-1 text-sm font-bold text-[var(--color-ink)]">{s.name}</h4>
                      </div>
                      {s.latest_run && (
                        <span
                          className={`rounded-full px-2.5 py-0.5 text-xs font-bold ${
                            s.latest_run.verdict === 'passed'
                              ? 'bg-emerald-100 text-emerald-800'
                              : 'bg-amber-100 text-amber-800'
                          }`}
                        >
                          {s.latest_run.overall_score}% {s.latest_run.verdict.toUpperCase()}
                        </span>
                      )}
                    </div>

                    <p className="text-xs text-[var(--color-ink-muted)] leading-relaxed">{s.description}</p>

                    {/* Customer Persona Box */}
                    <div className="rounded-xl bg-[var(--color-page)] p-3 border border-[var(--color-border)] text-xs space-y-1">
                      <div className="flex items-center justify-between text-[var(--color-ink-muted)]">
                        <span>
                          Customer Persona:{' '}
                          <strong className="text-slate-800">{s.customer_persona.name}</strong>
                        </span>
                        <span className="capitalize">
                          Sentiment: <strong>{s.customer_persona.sentiment}</strong>
                        </span>
                      </div>
                      <div className="text-[11px] text-[var(--color-ink-faint)]">
                        Expected Outcome:{' '}
                        <span className="font-mono text-[var(--color-ink)]">{s.expected_outcome}</span>
                      </div>
                    </div>

                    {/* Multi-turn Prompts preview */}
                    <div className="space-y-1 text-xs">
                      <span className="text-[11px] font-semibold text-[var(--color-ink-muted)]">
                        Multi-turn Prompts ({s.customer_messages.length}):
                      </span>
                      <div className="rounded bg-[var(--color-page)] p-2 text-[var(--color-ink)] line-clamp-2 italic text-[11px]">
                        "{s.customer_messages[0]}"
                      </div>
                    </div>

                    <div className="pt-2 flex items-center justify-between border-t border-[var(--color-border)]">
                      <span className="text-[11px] text-[var(--color-ink-faint)]">
                        Target Policy: {s.policy_name || 'Autonomous Default'}
                      </span>
                      <button
                        onClick={() => handleRunSimulation(s.id)}
                        disabled={runningScenarioId === s.id}
                        className="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
                      >
                        {runningScenarioId === s.id ? 'Simulating...' : 'Run Simulation'}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* TAB 3: AGENT POLICIES & TRAINING SOPS */}
          {activeTab === 'policies' && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-sm font-bold text-[var(--color-ink)]">
                    Agent Training Guidelines & SOPs
                  </h3>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    Defines process steps, numeric thresholds, and guardrails for our autonomous Agent.
                  </p>
                </div>
                <button
                  onClick={() => setShowCreatePolicyModal(true)}
                  className="rounded-lg bg-blue-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700"
                >
                  + Add New Policy
                </button>
              </div>

              <div className="space-y-4">
                {policies.map((p) => (
                  <div
                    key={p.id}
                    className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm space-y-4"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="rounded bg-blue-100 px-2 py-0.5 text-xs font-bold text-blue-800">
                          {p.version}
                        </span>
                        <h4 className="text-base font-bold text-[var(--color-ink)]">{p.name}</h4>
                        <span className="rounded bg-slate-100 px-2 py-0.5 text-xs font-medium text-[var(--color-ink-muted)] uppercase">
                          {p.category}
                        </span>
                      </div>
                      <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800">
                        {p.status}
                      </span>
                    </div>

                    <p className="text-xs text-[var(--color-ink-muted)]">{p.description}</p>

                    {/* Step-by-Step Procedure */}
                    <div>
                      <span className="text-xs font-bold text-slate-800 block mb-2">
                        Step-by-Step Process Flow:
                      </span>
                      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
                        {p.process_steps.map((st) => (
                          <div
                            key={st.step}
                            className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)] p-2.5 text-xs"
                          >
                            <span className="font-bold text-blue-700 block mb-0.5">
                              Step {st.step}: {st.title}
                            </span>
                            <p className="text-[var(--color-ink-muted)] text-[11px] leading-relaxed">
                              {st.instruction}
                            </p>
                          </div>
                        ))}
                      </div>
                    </div>

                    {/* Guardrails */}
                    <div>
                      <span className="text-xs font-bold text-slate-800 block mb-1">
                        Active Guardrails:
                      </span>
                      <div className="flex flex-wrap gap-1.5">
                        {p.guardrails.map((g, idx) => (
                          <span
                            key={idx}
                            className="rounded-lg bg-rose-50 border border-rose-200 px-2.5 py-1 text-xs text-rose-900 font-medium"
                          >
                            🛡️ {g}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      )}

      {/* Simulation Result Modal */}
      {activeRunResult && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="relative flex max-h-[90vh] w-full max-w-2xl flex-col rounded-2xl bg-[var(--color-surface)] shadow-2xl overflow-hidden">
            {/* Modal Header */}
            <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4 bg-[var(--color-page)]">
              <div className="flex items-center gap-2">
                <span className="text-sm font-bold text-[var(--color-ink)]">
                  Simulation Run Evaluation
                </span>
                <span
                  className={`rounded-full px-2.5 py-0.5 text-xs font-bold ${
                    activeRunResult.verdict === 'passed'
                      ? 'bg-emerald-100 text-emerald-800'
                      : 'bg-amber-100 text-amber-800'
                  }`}
                >
                  {activeRunResult.verdict.toUpperCase()} ({activeRunResult.overall_score}%)
                </span>
              </div>
              <button
                onClick={() => setActiveRunResult(null)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] font-bold"
              >
                ✕
              </button>
            </div>

            {/* Modal Body */}
            <div className="flex-1 overflow-y-auto p-6 space-y-5">
              {/* Scorecard */}
              <div className="grid grid-cols-3 gap-3 text-center">
                <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)] p-3">
                  <div className="text-[10px] font-semibold text-[var(--color-ink-muted)] uppercase">
                    Policy Adherence
                  </div>
                  <div className="text-xl font-bold text-[var(--color-ink)] mt-1">
                    {activeRunResult.policy_adherence_score}%
                  </div>
                </div>
                <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)] p-3">
                  <div className="text-[10px] font-semibold text-[var(--color-ink-muted)] uppercase">
                    Empathy & Tone
                  </div>
                  <div className="text-xl font-bold text-[var(--color-ink)] mt-1">
                    {activeRunResult.empathy_score}%
                  </div>
                </div>
                <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-page)] p-3">
                  <div className="text-[10px] font-semibold text-[var(--color-ink-muted)] uppercase">
                    Factual Accuracy
                  </div>
                  <div className="text-xl font-bold text-[var(--color-ink)] mt-1">
                    {activeRunResult.accuracy_score}%
                  </div>
                </div>
              </div>

              {/* Evaluation Summary */}
              <div className="rounded-xl bg-blue-50/70 border border-blue-200 p-3.5 text-xs text-blue-900 leading-relaxed">
                <span className="font-bold block mb-1">Coach Evaluation:</span>
                {activeRunResult.evaluation_summary}
              </div>

              {/* Turn-by-turn Transcript */}
              <div className="space-y-3">
                <h4 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">
                  Turn-by-Turn Simulated Dialogue ({activeRunResult.transcript.length} turns)
                </h4>
                <div className="space-y-2 border border-[var(--color-border)] rounded-xl p-3 bg-[var(--color-page)]/30 max-h-72 overflow-y-auto">
                  {activeRunResult.transcript.map((t, idx) => (
                    <div
                      key={idx}
                      className={`rounded-lg p-3 text-xs ${
                        t.role === 'customer'
                          ? 'bg-blue-50/60 border border-blue-100 text-blue-950'
                          : 'bg-[var(--color-surface)] border border-[var(--color-border)] text-slate-800'
                      }`}
                    >
                      <div className="flex items-center justify-between text-[11px] font-semibold mb-1 text-[var(--color-ink-muted)]">
                        <span>{t.speaker}</span>
                        <span>Turn {t.turn}</span>
                      </div>
                      <p className="whitespace-pre-wrap">{t.content}</p>
                      {t.policy_check && (
                        <div className="mt-1.5 pt-1.5 border-t border-[var(--color-border)] text-[10.5px] text-emerald-700 font-medium">
                          ✓ {t.policy_check}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>

              {/* Recommendations */}
              {activeRunResult.recommendations.length > 0 && (
                <div className="space-y-1">
                  <span className="text-xs font-bold text-[var(--color-ink)]">Coaching Takeaways:</span>
                  <ul className="list-disc list-inside text-xs text-[var(--color-ink-muted)] space-y-0.5">
                    {activeRunResult.recommendations.map((rec, i) => (
                      <li key={i}>{rec}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>

            <div className="border-t border-[var(--color-border)] px-6 py-3 bg-[var(--color-page)] flex justify-end">
              <button
                onClick={() => setActiveRunResult(null)}
                className="rounded-lg bg-[var(--color-accent)] px-4 py-1.5 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)]"
              >
                Close Evaluation
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Create Policy Modal */}
      {showCreatePolicyModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl bg-[var(--color-surface)] p-6 shadow-2xl space-y-4">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
              <h3 className="text-lg font-bold text-[var(--color-ink)]">Create Agent SOP Policy</h3>
              <button
                onClick={() => setShowCreatePolicyModal(false)}
                className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] font-bold"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleCreatePolicy} className="space-y-4">
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Policy Name:
                </label>
                <input
                  type="text"
                  required
                  value={policyName}
                  onChange={(e) => setPolicyName(e.target.value)}
                  placeholder="e.g. VIP Concierge Escalation Protocol"
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                    Category:
                  </label>
                  <select
                    value={policyCategory}
                    onChange={(e) => setPolicyCategory(e.target.value)}
                    className="w-full rounded-lg border border-slate-300 bg-[var(--color-surface)] p-2 text-xs focus:border-blue-500 focus:outline-none"
                  >
                    <option value="refunds">Refunds & Returns</option>
                    <option value="shipping">Shipping & Delays</option>
                    <option value="vip_support">VIP Concierge</option>
                    <option value="general">General Support</option>
                  </select>
                </div>
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                    Version:
                  </label>
                  <input
                    type="text"
                    value={policyVersion}
                    onChange={(e) => setPolicyVersion(e.target.value)}
                    placeholder="1.0"
                    className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-blue-500 focus:outline-none"
                  />
                </div>
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Description:
                </label>
                <textarea
                  rows={2}
                  value={policyDescription}
                  onChange={(e) => setPolicyDescription(e.target.value)}
                  placeholder="Objectives and scope of this policy..."
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink)] block mb-1">
                  Guardrails & Negative Constraints (one per line):
                </label>
                <textarea
                  rows={3}
                  value={guardrailText}
                  onChange={(e) => setGuardrailText(e.target.value)}
                  placeholder="Never promise refunds exceeding $150 without approval&#10;Always maintain empathetic tone&#10;Do not mention internal supplier names"
                  className="w-full rounded-lg border border-slate-300 p-2 text-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
                <button
                  type="button"
                  onClick={() => setShowCreatePolicyModal(false)}
                  className="rounded-lg px-3 py-1.5 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-slate-100"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creatingPolicy}
                  className="rounded-lg bg-blue-600 px-4 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
                >
                  {creatingPolicy ? 'Creating...' : 'Save Policy'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
