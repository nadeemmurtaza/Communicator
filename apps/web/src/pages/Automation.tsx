import React, { useEffect, useState } from 'react'
import {
  MessageSquare,
  CircleDot,
  List,
  Image,
  Link as LinkIcon,
  FileText,
  User,
  Tag,
  Scissors,
  Users,
  Clock,
  Star,
  Zap,
  FlaskConical,
  CheckCircle2,
  Info,
  Plus,
  X,
  Loader2,
} from '../components/icons'
import { api } from '../lib/api'
import type {
  AutomationRule,
  AutomationStats,
  RuleTestResult,
} from '../lib/api'

const ACTION_LABELS: Record<string, { label: string; icon: React.ReactNode; color: string }> = {
  auto_reply: { label: 'Text Reply', icon: <MessageSquare className="h-4 w-4" />, color: 'bg-blue-50 text-blue-700' },
  interactive_buttons: { label: 'Quick Buttons', icon: <CircleDot className="h-4 w-4" />, color: 'bg-emerald-50 text-emerald-700' },
  interactive_list: { label: 'List Menu', icon: <List className="h-4 w-4" />, color: 'bg-purple-50 text-purple-700' },
  send_media: { label: 'Send Media', icon: <Image className="h-4 w-4" />, color: 'bg-amber-50 text-amber-700' },
  send_cta_url: { label: 'CTA Link', icon: <LinkIcon className="h-4 w-4" />, color: 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' },
  send_template: { label: 'Meta Template', icon: <FileText className="h-4 w-4" />, color: 'bg-cyan-50 text-cyan-700' },
  handoff: { label: 'Agent Handoff', icon: <User className="h-4 w-4" />, color: 'bg-rose-50 text-rose-700' },
  add_tag: { label: 'Add Tag', icon: <Tag className="h-4 w-4" />, color: 'bg-teal-50 text-teal-700' },
  remove_tag: { label: 'Remove Tag', icon: <Scissors className="h-4 w-4" />, color: 'bg-[var(--color-page)] text-[var(--color-ink)]' },
  assign_team: { label: 'Assign Team', icon: <Users className="h-4 w-4" />, color: 'bg-orange-50 text-orange-700' },
  enroll_sequence: { label: 'Enroll Drip', icon: <Clock className="h-4 w-4" />, color: 'bg-sky-50 text-sky-700' },
  send_csat: { label: 'CSAT Survey', icon: <Star className="h-4 w-4 text-amber-500 fill-amber-500" />, color: 'bg-yellow-50 text-yellow-800' },
  trigger_webhook: { label: 'Webhook', icon: <Zap className="h-4 w-4" />, color: 'bg-fuchsia-50 text-fuchsia-700' },
}

export function Automation() {
  const [rules, setRules] = useState<AutomationRule[] | null>(null)
  const [stats, setStats] = useState<AutomationStats | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Loading states
  const [seeding, setSeeding] = useState(false)
  const [runningNudges, setRunningNudges] = useState(false)

  // Form state
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [keyword, setKeyword] = useState('')
  const [matchType, setMatchType] = useState('contains')
  const [actionType, setActionType] = useState('auto_reply')
  const [actionValue, setActionValue] = useState('')
  const [priority, setPriority] = useState(0)
  const [saving, setSaving] = useState(false)

  // Interactive buttons builder helpers
  const [button1, setButton1] = useState('')
  const [button2, setButton2] = useState('')
  const [button3, setButton3] = useState('')

  // Simulator / Test state
  const [showTester, setShowTester] = useState(false)
  const [testInput, setTestInput] = useState('')
  const [testPayload, setTestPayload] = useState('')
  const [testResult, setTestResult] = useState<RuleTestResult | null>(null)
  const [testing, setTesting] = useState(false)

  function load() {
    api
      .listAutomationRules()
      .then(setRules)
      .catch((e) => setError(e.message))

    api
      .getAutomationStats()
      .then(setStats)
      .catch(() => {})
  }

  useEffect(load, [])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)

    let finalActionValue = actionValue

    if (actionType === 'interactive_buttons') {
      const btns = [
        button1 ? { id: `btn_${button1.toLowerCase().replace(/\s+/g, '_')}`, title: button1 } : null,
        button2 ? { id: `btn_${button2.toLowerCase().replace(/\s+/g, '_')}`, title: button2 } : null,
        button3 ? { id: `btn_${button3.toLowerCase().replace(/\s+/g, '_')}`, title: button3 } : null,
      ].filter(Boolean)

      finalActionValue = JSON.stringify({
        body: actionValue || 'Please choose an option:',
        buttons: btns,
      })
    }

    try {
      const created = await api.createAutomationRule({
        name,
        description,
        trigger_keyword: keyword,
        match_type: matchType,
        action_type: actionType,
        action_value: finalActionValue,
        priority: Number(priority) || 0,
        is_active: true,
      })
      setName('')
      setDescription('')
      setKeyword('')
      setActionValue('')
      setPriority(0)
      setButton1('')
      setButton2('')
      setButton3('')
      setShowForm(false)
      setSuccess('Automation rule created successfully!')
      if (created && created.id) {
        setRules((prev) => (prev ? [created, ...prev] : [created]))
      }
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleSeedDefaults() {
    setError(null)
    setSeeding(true)
    try {
      const res = await api.seedDefaultAutomations()
      setSuccess(res.message || '11 WhatsApp automations loaded successfully!')
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSeeding(false)
    }
  }

  async function handleRunNudges() {
    setError(null)
    setRunningNudges(true)
    try {
      const res = await api.runInactivityNudges()
      setSuccess(`Ran 24h window watcher: ${res.nudgedCount} inactive conversations nudged.`)
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setRunningNudges(false)
    }
  }

  async function handleRunTest(e: React.FormEvent) {
    e.preventDefault()
    setTesting(true)
    setTestResult(null)
    try {
      const res = await api.testAutomationRule({
        text: testInput,
        payload_id: testPayload || undefined,
      })
      setTestResult(res)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setTesting(false)
    }
  }

  async function toggleActive(rule: AutomationRule) {
    const nextActive = !rule.isActive
    // Optimistic immediate UI update
    setRules((prev) =>
      prev ? prev.map((r) => (r.id === rule.id ? { ...r, isActive: nextActive } : r)) : null
    )
    setStats((prev) =>
      prev
        ? {
            ...prev,
            activeRules: nextActive ? prev.activeRules + 1 : Math.max(0, prev.activeRules - 1),
          }
        : null
    )
    try {
      await api.updateAutomationRule(rule.id, {
        name: rule.name,
        description: rule.description ?? '',
        trigger_keyword: rule.triggerKeyword,
        match_type: rule.matchType,
        action_type: rule.actionType,
        action_value: rule.actionValue,
        priority: rule.priority,
        is_active: nextActive,
      })
      load()
    } catch (e) {
      // Revert on error
      setRules((prev) =>
        prev ? prev.map((r) => (r.id === rule.id ? { ...r, isActive: rule.isActive } : r)) : null
      )
      load()
      setError((e as Error).message)
    }
  }

  async function remove(id: string) {
    const prevRules = rules
    // Optimistic immediate removal
    setRules((prev) => prev?.filter((r) => r.id !== id) ?? null)
    setStats((prev) =>
      prev
        ? {
            ...prev,
            totalRules: Math.max(0, prev.totalRules - 1),
            activeRules: prevRules?.find((r) => r.id === id)?.isActive
              ? Math.max(0, prev.activeRules - 1)
              : prev.activeRules,
          }
        : null
    )
    try {
      await api.deleteAutomationRule(id)
      setSuccess('Automation rule deleted.')
      load()
    } catch (e) {
      setRules(prevRules)
      load()
      setError((e as Error).message)
    }
  }

  return (
    <div className="space-y-6">
      {/* Top Header & Action Controls */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
            <span>WhatsApp Automation Suite</span>
            <span className="rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-semibold text-emerald-800">
              Meta Cloud API Ready
            </span>
          </h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Enterprise WhatsApp automations: interactive buttons, catalog menus, 24h session window recovery, live agent handoff, and CSAT surveys.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <button
            onClick={handleSeedDefaults}
            disabled={seeding}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-sm disabled:opacity-50"
            title="Install 11 standard WhatsApp automations in one click"
          >
            {seeding ? <Loader2 className="h-4 w-4 animate-spin" /> : <Zap className="h-4 w-4 text-amber-500" />}
            <span>{seeding ? 'Loading Pre-Built...' : 'Load Pre-Built Rules'}</span>
          </button>
          <button
            onClick={handleRunNudges}
            disabled={runningNudges}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-sm disabled:opacity-50"
            title="Check for conversations nearing the 24h Meta session window cutoff"
          >
            {runningNudges ? <Loader2 className="h-4 w-4 animate-spin" /> : <Clock className="h-4 w-4" />}
            <span>{runningNudges ? 'Running Nudges...' : 'Run 24h Nudges'}</span>
          </button>
          <button
            onClick={() => setShowTester((v) => !v)}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-sm"
          >
            <FlaskConical className="h-4 w-4 text-indigo-600" />
            <span>{showTester ? 'Hide Simulator' : 'Test Simulator'}</span>
          </button>
          <button
            onClick={() => setShowForm((v) => !v)}
            className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-xs font-medium text-white hover:bg-[var(--color-accent-hover)] shadow-sm"
          >
            {showForm ? (
              <span>Cancel</span>
            ) : (
              <>
                <Plus className="h-4 w-4" />
                <span>Add Custom Rule</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Alert Notices */}
      {error && (
        <div className="rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 flex justify-between items-center">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-red-500 hover:text-red-700"><X className="h-4 w-4" /></button>
        </div>
      )}
      {success && (
        <div className="rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 flex justify-between items-center">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="text-emerald-500 hover:text-[var(--color-accent)]"><X className="h-4 w-4" /></button>
        </div>
      )}

      {/* Stats Summary Cards */}
      {stats && (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">Active Rules</p>
            <p className="mt-1 text-2xl font-bold text-[var(--color-ink)]">
              {stats.activeRules ?? 0} <span className="text-xs text-[var(--color-ink-faint)] font-normal">/ {stats.totalRules ?? 0}</span>
            </p>
          </div>
          <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">Total Executions</p>
            <p className="mt-1 text-2xl font-bold text-[var(--color-ink)]">{(stats.totalExecutions ?? 0).toLocaleString()}</p>
          </div>
          <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">Avg CSAT Rating</p>
            <div className="mt-1 flex items-center gap-1.5 text-2xl font-bold text-amber-500">
              {stats.csatAverage ? (
                <>
                  <Star className="h-5 w-5 fill-amber-500 text-amber-500 inline" />
                  <span>{stats.csatAverage}</span>
                </>
              ) : (
                '-'
              )}
            </div>
          </div>
          <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">CSAT Feedback Count</p>
            <p className="mt-1 text-2xl font-bold text-[var(--color-ink)]">{stats.csatResponsesCount ?? 0}</p>
          </div>
        </div>
      )}

      {/* Simulator / Tester Drawer */}
      {showTester && (
        <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-soft)]/50 p-5 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-indigo-900 flex items-center gap-2">
              <FlaskConical className="h-4 w-4 text-indigo-600" />
              <span>WhatsApp Message & Button Simulator</span>
            </h2>
            <span className="text-xs text-[var(--color-accent)]">Simulate incoming customer actions without sending real texts</span>
          </div>
          <form onSubmit={handleRunTest} className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div className="sm:col-span-2">
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Incoming Message Text</label>
              <input
                value={testInput}
                onChange={(e) => setTestInput(e.target.value)}
                placeholder='e.g. "menu", "pricing", "agent", "quote", "#bot"'
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Button / List Payload ID (Optional)</label>
              <input
                value={testPayload}
                onChange={(e) => setTestPayload(e.target.value)}
                placeholder='e.g. "csat:5", "menu:services"'
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-sm"
              />
            </div>
            <div className="sm:col-span-3">
              <button
                type="submit"
                disabled={testing || (!testInput && !testPayload)}
                className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-1.5 text-xs font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
              >
                {testing && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
                <span>{testing ? 'Testing...' : 'Simulate WhatsApp Rule'}</span>
              </button>
            </div>
          </form>

          {testResult && (
            <div className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] p-4 space-y-2 text-sm">
              {testResult.matched ? (
                <>
                  <div className="flex items-center gap-2 font-medium text-emerald-700">
                    <CheckCircle2 className="h-4 w-4 text-emerald-600" />
                    <span>Matched Rule:</span>
                    <span className="font-semibold">{testResult.rule?.name || 'Automation Rule'}</span>
                    <span className="rounded bg-[var(--color-accent-muted)] px-2 py-0.5 text-xs text-[var(--color-ink-muted)]">
                      Priority: {testResult.rule?.priority ?? 0}
                    </span>
                  </div>
                  <div className="text-xs text-[var(--color-ink-muted)]">
                    <strong>Action:</strong> {testResult.actionType}
                  </div>
                  <div className="rounded bg-[var(--color-page)] p-3 font-mono text-xs text-[var(--color-ink)] whitespace-pre-wrap">
                    {testResult.actionValue}
                  </div>
                </>
              ) : (
                <div className="text-[var(--color-ink-muted)]">
                  <div className="flex items-center gap-1.5 font-medium text-amber-600">
                    <Info className="h-4 w-4 text-amber-600" />
                    <span>No deterministic rule matched.</span>
                  </div>
                  <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
                    The autonomous AI reply engine will handle this message using your service catalog, business profile, and knowledge base.
                  </p>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* New Rule Creation Form */}
      {showForm && (
        <form onSubmit={submit} className="space-y-4 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-semibold text-[var(--color-ink)] border-b border-[var(--color-border)] pb-2">
            Create WhatsApp Automation Rule
          </h2>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Rule Name</label>
              <input
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Services Catalog Menu"
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Trigger Keyword / Payload</label>
              <input
                required
                value={keyword}
                onChange={(e) => setKeyword(e.target.value)}
                placeholder="e.g. services, menu, or agent"
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Match Type</label>
              <select
                value={matchType}
                onChange={(e) => setMatchType(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm bg-[var(--color-surface)]"
              >
                <option value="contains">Contains Keyword</option>
                <option value="exact">Exact Match</option>
                <option value="starts_with">Starts With</option>
                <option value="regex">Regular Expression (Regex)</option>
                <option value="button_click">Interactive Button Click Payload</option>
                <option value="list_select">Interactive List Row Select</option>
                <option value="intent">Classified Intent</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Action Type</label>
              <select
                value={actionType}
                onChange={(e) => setActionType(e.target.value)}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm bg-[var(--color-surface)]"
              >
                <option value="auto_reply">Plain Text Reply</option>
                <option value="interactive_buttons">Quick-Reply Buttons</option>
                <option value="interactive_list">Interactive List Menu</option>
                <option value="send_media">Send Media</option>
                <option value="send_cta_url">Call-To-Action Link</option>
                <option value="handoff">Live Agent Handoff</option>
                <option value="add_tag">Add Contact Tag</option>
                <option value="remove_tag">Remove Contact Tag</option>
                <option value="assign_team">Assign Team Member</option>
                <option value="send_csat">CSAT Survey</option>
                <option value="trigger_webhook">Dispatch Webhook</option>
              </select>
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Description (Optional)</label>
              <input
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Brief internal note about this automation"
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Priority (Higher runs first)</label>
              <input
                type="number"
                value={priority}
                onChange={(e) => setPriority(Number(e.target.value))}
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
          </div>

          {/* Action configuration details */}
          {actionType === 'interactive_buttons' ? (
            <div className="space-y-3 rounded-[var(--radius-sm)] bg-[var(--color-page)] p-4 border border-[var(--color-border)]">
              <label className="block text-xs font-medium text-[var(--color-ink)]">Message Body Text</label>
              <textarea
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                rows={2}
                placeholder="Please select an option below:"
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] p-2 text-sm"
              />
              <p className="text-xs font-medium text-[var(--color-ink-muted)]">Buttons (up to 3 for WhatsApp Cloud API):</p>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <input
                  value={button1}
                  onChange={(e) => setButton1(e.target.value)}
                  placeholder="Button 1 (e.g. Services)"
                  className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-2.5 py-1.5 text-sm"
                />
                <input
                  value={button2}
                  onChange={(e) => setButton2(e.target.value)}
                  placeholder="Button 2 (e.g. Pricing)"
                  className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-2.5 py-1.5 text-sm"
                />
                <input
                  value={button3}
                  onChange={(e) => setButton3(e.target.value)}
                  placeholder="Button 3 (e.g. Talk to Agent)"
                  className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-2.5 py-1.5 text-sm"
                />
              </div>
            </div>
          ) : actionType === 'handoff' ? (
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Internal Handoff Note (Optional)</label>
              <input
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                placeholder="Reason or instructions for live agent"
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
          ) : actionType === 'add_tag' || actionType === 'remove_tag' ? (
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Tag Name</label>
              <input
                required
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                placeholder="e.g. high_intent, inquiry, vip"
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
          ) : actionType === 'send_csat' ? (
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Custom Survey Question (Optional)</label>
              <input
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                placeholder="How was your experience today?"
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm"
              />
            </div>
          ) : (
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Action Value / Payload</label>
              <textarea
                required
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                rows={3}
                placeholder={
                  actionType === 'send_media'
                    ? '{"media_type": "image|document", "url": "https://...", "caption": "..."}'
                    : actionType === 'send_cta_url'
                    ? '{"body": "Book a demo now", "display_text": "Book Call", "url": "https://calendly.com"}'
                    : 'Reply text or configuration JSON'
                }
                className="mt-1 w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm font-mono text-xs"
              />
            </div>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={() => setShowForm(false)}
              className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-1.5 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
            >
              {saving && <Loader2 className="h-4 w-4 animate-spin" />}
              <span>{saving ? 'Saving...' : 'Save Rule'}</span>
            </button>
          </div>
        </form>
      )}

      {/* Rules Listing */}
      {rules === null ? (
        <div className="p-8 text-center text-sm text-[var(--color-ink-muted)] flex items-center justify-center gap-2">
          <Loader2 className="h-4 w-4 animate-spin" />
          <span>Loading automation suite...</span>
        </div>
      ) : rules.length === 0 ? (
        <div className="rounded-[var(--radius-md)] border border-dashed border-[var(--color-border)] bg-[var(--color-surface)] p-12 text-center">
          <p className="text-sm font-medium text-[var(--color-ink)]">No automation rules configured yet.</p>
          <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
            Click below to instantly install all 11 battle-tested WhatsApp automations!
          </p>
          <button
            onClick={handleSeedDefaults}
            disabled={seeding}
            className="mt-4 flex items-center gap-1.5 mx-auto rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow disabled:opacity-50"
          >
            {seeding ? <Loader2 className="h-4 w-4 animate-spin" /> : <Zap className="h-4 w-4" />}
            <span>{seeding ? 'Loading Pre-Built Rules...' : 'Load All Pre-Built WhatsApp Automations'}</span>
          </button>
        </div>
      ) : (
        <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
          <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/75 px-4 py-3 flex items-center justify-between text-xs font-medium text-[var(--color-ink-muted)]">
            <span>Automation Rule ({rules.length})</span>
            <span>Priority & Status</span>
          </div>
          <ul className="divide-y divide-slate-100">
            {rules.map((r) => {
              const meta = ACTION_LABELS[r.actionType] ?? {
                label: r.actionType,
                icon: <Zap className="h-4 w-4" />,
                color: 'bg-[var(--color-accent-muted)] text-[var(--color-ink)]',
              }
              return (
                <li key={r.id} className="p-4 hover:bg-[var(--color-page)]/50 transition-colors">
                  <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                    <div className="min-w-0 space-y-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-semibold text-sm text-[var(--color-ink)]">{r.name}</span>
                        <span className={`inline-flex items-center gap-1.5 rounded px-2 py-0.5 text-xs font-medium ${meta.color}`}>
                          <span className="flex items-center">{meta.icon}</span>
                          <span>{meta.label}</span>
                        </span>
                        {r.executionCount ? (
                          <span className="rounded-full bg-[var(--color-accent-muted)] px-2 py-0.5 text-xs text-[var(--color-ink-muted)] font-mono">
                            {r.executionCount} fired
                          </span>
                        ) : null}
                      </div>

                      <p className="text-xs text-[var(--color-ink-muted)]">
                        When input <span className="font-semibold">{r.matchType}</span>:{' '}
                        <button
                          type="button"
                          onClick={() => {
                            setTestInput(r.triggerKeyword)
                            setShowTester(true)
                          }}
                          className="font-mono text-[var(--color-accent)] hover:underline inline cursor-pointer"
                          title="Click to test this trigger in simulator"
                        >
                          "{r.triggerKeyword}"
                        </button>
                      </p>

                      {r.description && (
                        <p className="text-xs text-[var(--color-ink-faint)] italic">{r.description}</p>
                      )}

                      {r.actionValue && (
                        <div className="mt-1 line-clamp-2 max-w-xl text-xs font-mono text-[var(--color-ink-muted)] bg-[var(--color-page)] p-1.5 rounded border border-[var(--color-border)]">
                          {r.actionValue}
                        </div>
                      )}
                    </div>

                    <div className="flex shrink-0 items-center gap-2 pt-2 sm:pt-0">
                      <span className="text-xs font-mono text-[var(--color-ink-faint)]" title="Execution priority">
                        p:{r.priority}
                      </span>
                      <button
                        onClick={() => toggleActive(r)}
                        className={`rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors ${
                          r.isActive
                            ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                            : 'bg-[var(--color-accent-muted)] text-[var(--color-ink-muted)] border border-[var(--color-border)]'
                        }`}
                      >
                        {r.isActive ? 'Active' : 'Paused'}
                      </button>
                      <button
                        onClick={() => remove(r.id)}
                        className="rounded px-2 py-1 text-xs text-[var(--color-ink-faint)] hover:bg-red-50 hover:text-red-600"
                        title="Delete Rule"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </li>
              )
            })}
          </ul>
        </div>
      )}
    </div>
  )
}
