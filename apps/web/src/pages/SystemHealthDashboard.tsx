import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  CheckCircle2,
  XCircle,
  AlertTriangle,
  RefreshCw,
  Shield,
  Settings,
  ArrowRight,
  Zap,
  Users,
  MessageSquare,
  Sparkles,
  BarChart3,
  Lock,
  Plug,
} from '../components/icons'
import {
  getOnboardingSummary,
  listOnboardingSteps,
  getOnboardingProgress,
  completeOnboardingStep,
  runHealthCheck,
  getHealthCheckSummary,
  getOnboardingSettings,
  updateOnboardingSettings,
  type OnboardingSummary,
  type OnboardingStep,
  type OnboardingProgress,
  type HealthCheckSummary,
  type OnboardingSettings,
} from '../lib/api'

const categoryIcons: Record<string, typeof Zap> = {
  general: Sparkles,
  channels: MessageSquare,
  ai: Sparkles,
  analytics: BarChart3,
  compliance: Lock,
  integrations: Plug,
}

const categoryColors: Record<string, string> = {
  general: 'bg-purple-50 text-purple-600',
  channels: 'bg-emerald-50 text-emerald-600',
  ai: 'bg-blue-50 text-blue-600',
  analytics: 'bg-amber-50 text-amber-600',
  compliance: 'bg-red-50 text-red-600',
  integrations: 'bg-indigo-50 text-indigo-600',
}

export function SystemHealthDashboard() {
  const [tab, setTab] = useState<'onboarding' | 'health' | 'settings'>('onboarding')
  const [summary, setSummary] = useState<OnboardingSummary | null>(null)
  const [steps, setSteps] = useState<OnboardingStep[]>([])
  const [progress, setProgress] = useState<OnboardingProgress[]>([])
  const [health, setHealth] = useState<HealthCheckSummary | null>(null)
  const [settings, setSettings] = useState<OnboardingSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [busyStep, setBusyStep] = useState<string | null>(null)
  const [healthLoading, setHealthLoading] = useState(false)

  const loadAll = async () => {
    setLoading(true)
    try {
      const [s, st, p, h, se] = await Promise.allSettled([
        getOnboardingSummary(),
        listOnboardingSteps(),
        getOnboardingProgress(),
        getHealthCheckSummary(),
        getOnboardingSettings(),
      ])
      if (s.status === 'fulfilled') setSummary(s.value)
      if (st.status === 'fulfilled') setSteps(st.value)
      if (p.status === 'fulfilled') setProgress(p.value)
      if (h.status === 'fulfilled') setHealth(h.value)
      if (se.status === 'fulfilled') setSettings(se.value)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadAll()
  }, [])

  const handleCompleteStep = async (stepId: string) => {
    setBusyStep(stepId)
    try {
      const result = await completeOnboardingStep(stepId)
      if (result.success) {
        setSummary(result.summary)
        setProgress(result.summary.categories.flatMap((c) => []))
        loadAll()
      }
    } finally {
      setBusyStep(null)
    }
  }

  const handleRunHealthCheck = async () => {
    setHealthLoading(true)
    try {
      const result = await runHealthCheck()
      if (result.success) setHealth(result.summary)
    } finally {
      setHealthLoading(false)
    }
  }

  const handleUpdateSettings = async (updates: Partial<OnboardingSettings>) => {
    const updated = await updateOnboardingSettings(updates)
    setSettings(updated)
  }

  const completedStepIds = new Set(progress.filter((p) => p.isCompleted).map((p) => p.stepId))

  const healthStatusColor = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'text-emerald-600 bg-emerald-50'
      case 'degraded':
        return 'text-amber-600 bg-amber-50'
      case 'unhealthy':
        return 'text-red-600 bg-red-50'
      default:
        return 'text-gray-600 bg-gray-50'
    }
  }

  return (
    <div className="space-y-8 max-w-7xl mx-auto">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-[var(--color-border)] pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-[var(--color-ink)]">
            System Health & Onboarding
          </h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Set up your Communicator instance and monitor system component health.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={loadAll}
            disabled={loading}
            className="btn-secondary disabled:opacity-50"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-[var(--color-border)]">
        {(['onboarding', 'health', 'settings'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              tab === t
                ? 'border-[var(--color-accent)] text-[var(--color-accent)]'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            {t === 'onboarding' ? 'Onboarding Wizard' : t === 'health' ? 'System Health' : 'Settings'}
          </button>
        ))}
      </div>

      {/* Onboarding Tab */}
      {tab === 'onboarding' && (
        <div className="space-y-6">
          {/* Progress Overview */}
          {summary && (
            <div className="grid grid-cols-1 sm:grid-cols-4 gap-4">
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">
                  Overall Progress
                </div>
                <div className="mt-3 flex items-baseline gap-2">
                  <span className="text-3xl font-bold text-[var(--color-ink)]">
                    {Math.round(summary.completionPercentage)}%
                  </span>
                </div>
                <div className="mt-2 w-full bg-gray-100 rounded-full h-2">
                  <div
                    className="bg-[var(--color-accent)] h-2 rounded-full transition-all"
                    style={{ width: `${summary.completionPercentage}%` }}
                  />
                </div>
                <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
                  {summary.completedSteps} of {summary.totalSteps} steps completed
                </p>
              </div>

              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">
                  Required Steps
                </div>
                <div className="mt-3 flex items-baseline gap-2">
                  <span className="text-3xl font-bold text-[var(--color-ink)]">
                    {summary.requiredCompleted}/{summary.requiredSteps}
                  </span>
                </div>
                <div className="mt-2 w-full bg-gray-100 rounded-full h-2">
                  <div
                    className="bg-emerald-500 h-2 rounded-full transition-all"
                    style={{ width: `${summary.requiredCompletionPercentage}%` }}
                  />
                </div>
                <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
                  Must-complete steps
                </p>
              </div>

              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">
                  Categories
                </div>
                <div className="mt-3 flex items-baseline gap-2">
                  <span className="text-3xl font-bold text-[var(--color-ink)]">
                    {summary.categories.length}
                  </span>
                </div>
                <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
                  Setup categories to complete
                </p>
              </div>

              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">
                  Next Steps
                </div>
                <div className="mt-3 flex items-baseline gap-2">
                  <span className="text-3xl font-bold text-[var(--color-ink)]">
                    {summary.nextSteps.length}
                  </span>
                </div>
                <p className="mt-2 text-xs text-[var(--color-ink-faint)]">
                  Recommended actions ahead
                </p>
              </div>
            </div>
          )}

          {/* Category Progress */}
          {summary?.categories.length && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
              <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">
                Progress by Category
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {summary.categories.map((cat) => {
                  const Icon = categoryIcons[cat.category] || Zap
                  const color = categoryColors[cat.category] || 'bg-gray-50 text-gray-600'
                  return (
                    <div key={cat.category} className="flex items-center gap-3">
                      <div className={`flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] ${color}`}>
                        <Icon className="w-5 h-5" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between">
                          <span className="text-sm font-medium text-[var(--color-ink)] capitalize">
                            {cat.category}
                          </span>
                          <span className="text-xs text-[var(--color-ink-muted)]">
                            {cat.completed}/{cat.total}
                          </span>
                        </div>
                        <div className="mt-1.5 w-full bg-gray-100 rounded-full h-1.5">
                          <div
                            className="bg-[var(--color-accent)] h-1.5 rounded-full transition-all"
                            style={{ width: `${cat.percentage}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          )}

          {/* Next Steps */}
          {summary?.nextSteps.length && (
            <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
              <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">
                Recommended Next Steps
              </h3>
              <div className="space-y-3">
                {summary.nextSteps.map((step) => (
                  <div
                    key={step.id}
                    className="flex items-center justify-between gap-4 p-3 rounded-[var(--radius-md)] border border-[var(--color-border)]"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-[var(--color-ink)]">
                          {step.title}
                        </span>
                        {step.isRequired && (
                          <span className="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-red-50 text-red-600">
                            Required
                          </span>
                        )}
                      </div>
                      <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
                        {step.description}
                      </p>
                    </div>
                    <Link
                      to={step.route}
                      className="shrink-0 btn-secondary text-xs"
                    >
                      Go to setup
                      <ArrowRight className="w-3 h-3" />
                    </Link>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* All Steps */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
            <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">
              All Onboarding Steps
            </h3>
            <div className="space-y-2">
              {steps.map((step) => {
                const isCompleted = completedStepIds.has(step.id)
                const Icon = categoryIcons[step.category] || Zap
                return (
                  <div
                    key={step.id}
                    className={`flex items-center justify-between gap-4 p-3 rounded-[var(--radius-md)] border ${
                      isCompleted
                        ? 'border-emerald-200 bg-emerald-50/50'
                        : 'border-[var(--color-border)]'
                    }`}
                  >
                    <div className="flex items-center gap-3 flex-1 min-w-0">
                      <div
                        className={`flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] ${
                          categoryColors[step.category] || 'bg-gray-50 text-gray-600'
                        }`}
                      >
                        {isCompleted ? (
                          <CheckCircle2 className="w-4 h-4 text-emerald-600" />
                        ) : (
                          <Icon className="w-4 h-4" />
                        )}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span
                            className={`text-sm font-medium ${
                              isCompleted ? 'text-[var(--color-ink-muted)] line-through' : 'text-[var(--color-ink)]'
                            }`}
                          >
                            {step.title}
                          </span>
                          {step.isRequired && (
                            <span className="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-red-50 text-red-600">
                              Required
                            </span>
                          )}
                        </div>
                        <p className="text-xs text-[var(--color-ink-faint)]">
                          {step.description}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      {isCompleted ? (
                        <span className="text-xs text-emerald-600 font-medium">Completed</span>
                      ) : (
                        <button
                          onClick={() => handleCompleteStep(step.id)}
                          disabled={busyStep === step.id}
                          className="btn-primary text-xs disabled:opacity-50"
                        >
                          Mark done
                        </button>
                      )}
                      <Link
                        to={step.route}
                        className="btn-ghost text-xs"
                      >
                        <ArrowRight className="w-3 h-3" />
                      </Link>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        </div>
      )}

      {/* Health Tab */}
      {tab === 'health' && (
        <div className="space-y-6">
          {/* Health Overview */}
          {health && (
            <>
              <div className="grid grid-cols-1 sm:grid-cols-5 gap-4">
                <div className="sm:col-span-1 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                  <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">
                    Overall Status
                  </div>
                  <div className="mt-3">
                    <span
                      className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-sm font-bold ${healthStatusColor(
                        health.overallStatus
                      )}`}
                    >
                      {health.overallStatus === 'healthy' ? (
                        <CheckCircle2 className="w-4 h-4" />
                      ) : health.overallStatus === 'degraded' ? (
                        <AlertTriangle className="w-4 h-4" />
                      ) : (
                        <XCircle className="w-4 h-4" />
                      )}
                      {health.overallStatus.toUpperCase()}
                    </span>
                  </div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                  <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">
                    Components
                  </div>
                  <div className="mt-3 text-3xl font-bold text-[var(--color-ink)]">
                    {health.totalComponents}
                  </div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-emerald-200 bg-emerald-50/50 p-5">
                  <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-emerald-600">
                    Healthy
                  </div>
                  <div className="mt-3 text-3xl font-bold text-emerald-600">{health.healthy}</div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-amber-200 bg-amber-50/50 p-5">
                  <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-amber-600">
                    Degraded
                  </div>
                  <div className="mt-3 text-3xl font-bold text-amber-600">{health.degraded}</div>
                </div>

                <div className="rounded-[var(--radius-lg)] border border-red-200 bg-red-50/50 p-5">
                  <div className="text-[11px] font-bold uppercase tracking-[0.6px] text-red-600">
                    Unhealthy
                  </div>
                  <div className="mt-3 text-3xl font-bold text-red-600">{health.unhealthy}</div>
                </div>
              </div>

              {/* Component Details */}
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-sm font-semibold text-[var(--color-ink)]">
                    Component Health Details
                  </h3>
                  <button
                    onClick={handleRunHealthCheck}
                    disabled={healthLoading}
                    className="btn-secondary text-xs disabled:opacity-50"
                  >
                    <RefreshCw className={`w-3 h-3 ${healthLoading ? 'animate-spin' : ''}`} />
                    Run Check
                  </button>
                </div>
                <div className="space-y-2">
                  {health.results.map((result) => (
                    <div
                      key={result.id}
                      className="flex items-center justify-between gap-4 p-3 rounded-[var(--radius-md)] border border-[var(--color-border)]"
                    >
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-mono font-medium text-[var(--color-ink)]">
                            {result.component}
                          </span>
                          <span
                            className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-bold ${healthStatusColor(
                              result.status
                            )}`}
                          >
                            {result.status === 'healthy' ? (
                              <CheckCircle2 className="w-3 h-3" />
                            ) : result.status === 'degraded' ? (
                              <AlertTriangle className="w-3 h-3" />
                            ) : (
                              <XCircle className="w-3 h-3" />
                            )}
                            {result.status}
                          </span>
                        </div>
                        {result.message && (
                          <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
                            {result.message}
                          </p>
                        )}
                      </div>
                      <div className="text-right shrink-0">
                        {result.checkDurationMs !== null && (
                          <span className="text-xs text-[var(--color-ink-faint)]">
                            {result.checkDurationMs}ms
                          </span>
                        )}
                        <p className="text-[10px] text-[var(--color-ink-faint)]">
                          {new Date(result.lastCheckedAt).toLocaleTimeString()}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </>
          )}

          {!health && (
            <div className="rounded-[var(--radius-lg)] border border-dashed border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center">
              <Shield className="w-8 h-8 text-[var(--color-ink-faint)] mx-auto mb-2" />
              <h3 className="text-sm font-semibold text-[var(--color-ink)]">
                No health check data yet
              </h3>
              <p className="text-xs text-[var(--color-ink-muted)] mt-1 mb-4">
                Run a health check to see the status of all system components.
              </p>
              <button
                onClick={handleRunHealthCheck}
                disabled={healthLoading}
                className="btn-primary disabled:opacity-50"
              >
                <RefreshCw className={`w-3.5 h-3.5 ${healthLoading ? 'animate-spin' : ''}`} />
                Run Health Check
              </button>
            </div>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {tab === 'settings' && settings && (
        <div className="space-y-6">
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
            <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-4">
              Onboarding & Health Check Settings
            </h3>
            <div className="space-y-4 max-w-lg">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm font-medium text-[var(--color-ink)]">
                    Show Onboarding
                  </div>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    Display onboarding wizard on the dashboard
                  </p>
                </div>
                <button
                  onClick={() =>
                    handleUpdateSettings({ showOnboarding: !settings.showOnboarding })
                  }
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    settings.showOnboarding ? 'bg-[var(--color-accent)]' : 'bg-gray-300'
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      settings.showOnboarding ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm font-medium text-[var(--color-ink)]">
                    Auto Health Check
                  </div>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    Automatically run health checks at intervals
                  </p>
                </div>
                <button
                  onClick={() =>
                    handleUpdateSettings({ autoCheck: !settings.autoCheck })
                  }
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    settings.autoCheck ? 'bg-[var(--color-accent)]' : 'bg-gray-300'
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      settings.autoCheck ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>

              <div>
                <div className="text-sm font-medium text-[var(--color-ink)]">
                  Health Check Interval
                </div>
                <p className="text-xs text-[var(--color-ink-muted)] mb-2">
                  Minutes between automatic health checks
                </p>
                <input
                  type="number"
                  value={settings.healthCheckIntervalMinutes}
                  onChange={(e) =>
                    handleUpdateSettings({
                      healthCheckIntervalMinutes: parseInt(e.target.value) || 60,
                    })
                  }
                  className="w-32 px-3 py-2 text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                  min={5}
                  max={1440}
                />
              </div>
            </div>
          </div>

          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5">
            <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-2">
              How It Works
            </h3>
            <div className="space-y-3 text-sm text-[var(--color-ink-muted)]">
              <p>
                <strong className="text-[var(--color-ink)]">Onboarding Wizard</strong> guides new
                organizations through setting up all Communicator features in a structured order.
                Each step links to the relevant dashboard for configuration.
              </p>
              <p>
                <strong className="text-[var(--color-ink)]">System Health Check</strong> verifies
                that all components — database, Claude API, channel inboxes — are functioning
                correctly. Run checks manually or schedule them automatically.
              </p>
              <p>
                <strong className="text-[var(--color-ink)]">Required Steps</strong> must be
                completed before the system is considered fully configured. Optional steps enhance
                functionality but are not mandatory.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
