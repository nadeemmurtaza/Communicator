import { useEffect, useState } from 'react'
import {
  api,
  type AutomationRule,
  type AutomationStats,
  type RuleTestResult,
} from '../lib/api'

const ACTION_LABELS: Record<string, { label: string; icon: string; color: string }> = {
  auto_reply: { label: 'Text Reply', icon: '💬', color: 'bg-blue-50 text-blue-700' },
  interactive_buttons: { label: 'Quick Buttons', icon: '🔘', color: 'bg-emerald-50 text-emerald-700' },
  interactive_list: { label: 'List Menu', icon: '📋', color: 'bg-purple-50 text-purple-700' },
  send_media: { label: 'Send Media', icon: '🖼️', color: 'bg-amber-50 text-amber-700' },
  send_cta_url: { label: 'CTA Link', icon: '🔗', color: 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' },
  send_template: { label: 'Meta Template', icon: '📄', color: 'bg-cyan-50 text-cyan-700' },
  handoff: { label: 'Agent Handoff', icon: '👤', color: 'bg-rose-50 text-rose-700' },
  add_tag: { label: 'Add Tag', icon: '🏷️', color: 'bg-teal-50 text-teal-700' },
  remove_tag: { label: 'Remove Tag', icon: '✂️', color: 'bg-[var(--color-page)] text-[var(--color-ink)]' },
  assign_team: { label: 'Assign Team', icon: '👥', color: 'bg-orange-50 text-orange-700' },
  enroll_sequence: { label: 'Enroll Drip', icon: '⏳', color: 'bg-sky-50 text-sky-700' },
  send_csat: { label: 'CSAT Survey', icon: '⭐', color: 'bg-yellow-50 text-yellow-800' },
  trigger_webhook: { label: 'Webhook', icon: '⚡', color: 'bg-fuchsia-50 text-fuchsia-700' },
}

export function Automation() {
  const [rules, setRules] = useState<AutomationRule[] | null>(null)
  const [stats, setStats] = useState<AutomationStats | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

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
      await api.createAutomationRule({
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
      setButton1('')
      setButton2('')
      setButton3('')
      setShowForm(false)
      setSuccess('Automation rule created successfully!')
      load()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleSeedDefaults() {
    setError(null)
    try {
      const res = await api.seedDefaultAutomations()
      setSuccess(res.message || '11 WhatsApp automations loaded successfully!')
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleRunNudges() {
    setError(null)
    try {
      const res = await api.runInactivityNudges()
      setSuccess(`Ran 24h window watcher: ${res.nudgedCount} inactive conversations nudged.`)
      load()
    } catch (e) {
      setError((e as Error).message)
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
    try {
      await api.updateAutomationRule(rule.id, {
        name: rule.name,
        description: rule.description ?? '',
        trigger_keyword: rule.triggerKeyword,
        match_type: rule.matchType,
        action_type: rule.actionType,
        action_value: rule.actionValue,
        priority: rule.priority,
        is_active: !rule.isActive,
      })
      load()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteAutomationRule(id)
      setRules((prev) => prev?.filter((r) => r.id !== id) ?? null)
      setSuccess('Automation rule deleted.')
    } catch (e) {
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
            className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-sm"
            title="Install 11 standard WhatsApp automations in one click"
          >
            ⚡ Load Pre-Built Rules
          </button>
          <button
            onClick={handleRunNudges}
            className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-sm"
            title="Check for conversations nearing the 24h Meta session window cutoff"
          >
            ⏰ Run 24h Nudges
          </button>
          <button
            onClick={() => setShowTester((v) => !v)}
            className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-sm"
          >
            🧪 {showTester ? 'Hide Simulator' : 'Test Simulator'}
          </button>
          <button
            onClick={() => setShowForm((v) => !v)}
            className="rounded-md bg-[var(--color-accent)] px-3 py-2 text-xs font-medium text-white hover:bg-[var(--color-accent-hover)] shadow-sm"
          >
            {showForm ? 'Cancel' : '+ Add Custom Rule'}
          </button>
        </div>
      </div>

      {/* Alert Notices */}
      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 flex justify-between items-center">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-red-500 font-bold">×</button>
        </div>
      )}
      {success && (
        <div className="rounded-md border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 flex justify-between items-center">
          <span>{success}</span>
          <button onClick={() => setSuccess(null)} className="text-emerald-500 font-bold">×</button>
        </div>
      )}

      {/* Stats Summary Cards */}
      {stats && (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">Active Rules</p>
            <p className="mt-1 text-2xl font-bold text-[var(--color-ink)]">
              {stats.activeRules} <span className="text-xs text-[var(--color-ink-faint)] font-normal">/ {stats.totalRules}</span>
            </p>
          </div>
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">Total Executions</p>
            <p className="mt-1 text-2xl font-bold text-[var(--color-ink)]">{stats.totalExecutions.toLocaleString()}</p>
          </div>
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">Avg CSAT Rating</p>
            <p className="mt-1 text-2xl font-bold text-amber-500">
              {stats.csatAverage ? `⭐ ${stats.csatAverage}` : '—'}
            </p>
          </div>
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <p className="text-xs font-medium text-[var(--color-ink-muted)]">CSAT Feedback Count</p>
            <p className="mt-1 text-2xl font-bold text-[var(--color-ink)]">{stats.csatResponsesCount}</p>
          </div>
        </div>
      )}

      {/* Simulator / Tester Drawer */}
      {showTester && (
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-accent-soft)]/50 p-5 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-indigo-900 flex items-center gap-2">
              <span>🧪 WhatsApp Message & Button Simulator</span>
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
                className="mt-1 w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Button / List Payload ID (Optional)</label>
              <input
                value={testPayload}
                onChange={(e) => setTestPayload(e.target.value)}
                placeholder='e.g. "csat:5", "menu:services"'
                className="mt-1 w-full rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-sm"
              />
            </div>
            <div className="sm:col-span-3">
              <button
                type="submit"
                disabled={testing || (!testInput && !testPayload)}
                className="rounded-md bg-[var(--color-accent)] px-4 py-1.5 text-xs font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
              >
                {testing ? 'Testing…' : 'Simulate WhatsApp Rule'}
              </button>
            </div>
          </form>

          {testResult && (
            <div className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] p-4 space-y-2 text-sm">
              {testResult.matched ? (
                <>
                  <div className="flex items-center gap-2 font-medium text-emerald-700">
                    <span>✅ Matched Rule:</span>
                    <span className="font-semibold">{testResult.rule?.name}</span>
                    <span className="rounded bg-slate-100 px-2 py-0.5 text-xs text-[var(--color-ink-muted)]">
                      Priority: {testResult.rule?.priority}
                    </span>
                  </div>
                  <div className="text-xs text-[var(--color-ink-muted)]">
                    <strong>Action:</strong> {testResult.actionType}
                  </div>
                  <div className="rounded bg-[var(--color-page)] p-3 font-mono text-xs text-slate-800 whitespace-pre-wrap">
                    {testResult.actionValue}
                  </div>
                </>
              ) : (
                <div className="text-[var(--color-ink-muted)]">
                  <span className="font-medium text-amber-600">ℹ️ No deterministic rule matched.</span>
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
        <form onSubmit={submit} className="space-y-4 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
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
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Trigger Keyword / Payload</label>
              <input
                required
                value={keyword}
                onChange={(e) => setKeyword(e.target.value)}
                placeholder="e.g. services, menu, or agent"
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Match Type</label>
              <select
                value={matchType}
                onChange={(e) => setMatchType(e.target.value)}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm bg-[var(--color-surface)]"
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
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm bg-[var(--color-surface)]"
              >
                <option value="auto_reply">💬 Plain Text Reply</option>
                <option value="interactive_buttons">🔘 Quick-Reply Interactive Buttons</option>
                <option value="interactive_list">📋 Interactive List Menu (JSON)</option>
                <option value="send_media">🖼️ Send Media (Image/PDF Document)</option>
                <option value="send_cta_url">🔗 Call-To-Action Link (URL)</option>
                <option value="handoff">👤 Live Agent Handoff (Pause Bot)</option>
                <option value="add_tag">🏷️ Add Contact Tag</option>
                <option value="remove_tag">✂️ Remove Contact Tag</option>
                <option value="assign_team">👥 Assign to Team Member</option>
                <option value="send_csat">⭐ Send CSAT Rating Survey</option>
                <option value="trigger_webhook">⚡ Dispatch Webhook</option>
              </select>
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Description (Optional)</label>
              <input
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Brief internal note about this automation"
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Priority (Higher runs first)</label>
              <input
                type="number"
                value={priority}
                onChange={(e) => setPriority(Number(e.target.value))}
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
          </div>

          {/* Action configuration details */}
          {actionType === 'interactive_buttons' ? (
            <div className="space-y-3 rounded-md bg-[var(--color-page)] p-4 border border-[var(--color-border)]">
              <label className="block text-xs font-medium text-[var(--color-ink)]">Message Body Text</label>
              <textarea
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                rows={2}
                placeholder="Please select an option below:"
                className="w-full rounded-md border border-slate-300 p-2 text-sm"
              />
              <p className="text-xs font-medium text-[var(--color-ink-muted)]">Buttons (up to 3 for WhatsApp Cloud API):</p>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <input
                  value={button1}
                  onChange={(e) => setButton1(e.target.value)}
                  placeholder="Button 1 (e.g. Services)"
                  className="rounded-md border border-slate-300 px-2.5 py-1.5 text-sm"
                />
                <input
                  value={button2}
                  onChange={(e) => setButton2(e.target.value)}
                  placeholder="Button 2 (e.g. Pricing)"
                  className="rounded-md border border-slate-300 px-2.5 py-1.5 text-sm"
                />
                <input
                  value={button3}
                  onChange={(e) => setButton3(e.target.value)}
                  placeholder="Button 3 (e.g. Talk to Agent)"
                  className="rounded-md border border-slate-300 px-2.5 py-1.5 text-sm"
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
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
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
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
              />
            </div>
          ) : actionType === 'send_csat' ? (
            <div>
              <label className="block text-xs font-medium text-[var(--color-ink-muted)]">Custom Survey Question (Optional)</label>
              <input
                value={actionValue}
                onChange={(e) => setActionValue(e.target.value)}
                placeholder="How was your experience today?"
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
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
                className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm font-mono text-xs"
              />
            </div>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={() => setShowForm(false)}
              className="rounded-md border border-slate-300 px-4 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="rounded-md bg-[var(--color-accent)] px-4 py-1.5 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
            >
              {saving ? 'Saving…' : 'Save Rule'}
            </button>
          </div>
        </form>
      )}

      {/* Rules Listing */}
      {rules === null ? (
        <div className="p-8 text-center text-sm text-[var(--color-ink-muted)]">Loading automation suite…</div>
      ) : rules.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-[var(--color-surface)] p-12 text-center">
          <p className="text-sm font-medium text-[var(--color-ink)]">No automation rules configured yet.</p>
          <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
            Click below to instantly install all 11 battle-tested WhatsApp automations!
          </p>
          <button
            onClick={handleSeedDefaults}
            className="mt-4 rounded-md bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow"
          >
            ⚡ Load All Pre-Built WhatsApp Automations
          </button>
        </div>
      ) : (
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
          <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/75 px-4 py-3 flex items-center justify-between text-xs font-medium text-[var(--color-ink-muted)]">
            <span>Automation Rule ({rules.length})</span>
            <span>Priority & Status</span>
          </div>
          <ul className="divide-y divide-slate-100">
            {rules.map((r) => {
              const meta = ACTION_LABELS[r.actionType] ?? {
                label: r.actionType,
                icon: '⚡',
                color: 'bg-slate-100 text-[var(--color-ink)]',
              }
              return (
                <li key={r.id} className="p-4 hover:bg-[var(--color-page)]/50 transition-colors">
                  <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                    <div className="min-w-0 space-y-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-semibold text-sm text-[var(--color-ink)]">{r.name}</span>
                        <span className={`inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs font-medium ${meta.color}`}>
                          <span>{meta.icon}</span>
                          <span>{meta.label}</span>
                        </span>
                        {r.executionCount ? (
                          <span className="rounded-full bg-slate-100 px-2 py-0.5 text-xs text-[var(--color-ink-muted)] font-mono">
                            {r.executionCount} fired
                          </span>
                        ) : null}
                      </div>

                      <p className="text-xs text-[var(--color-ink-muted)]">
                        When input <span className="font-semibold">{r.matchType}</span>: "{r.triggerKeyword}"
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
                            : 'bg-slate-100 text-[var(--color-ink-muted)] border border-[var(--color-border)]'
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
