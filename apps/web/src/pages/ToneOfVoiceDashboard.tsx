import { useState, useEffect } from 'react'
import {
  getToneSummary, getToneConfig, updateToneConfig, applyTonePreset,
  listGuidanceRules, createGuidanceRule, updateGuidanceRule, deleteGuidanceRule,
  getPromptAdditions
} from '../lib/api'
import type { ToneSummary, ToneConfig, GuidanceRule, TonePreset, PromptAdditions } from '../lib/api'
import {
  MessageSquare, BookOpen, Sparkles, Settings, Play, Trash2, Plus,
  CheckCircle, XCircle, Eye, Save, RefreshCw
} from '../components/icons'

export default function ToneOfVoiceDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'config' | 'presets' | 'guidance' | 'preview'>('overview')
  const [summary, setSummary] = useState<ToneSummary | null>(null)
  const [config, setConfig] = useState<ToneConfig | null>(null)
  const [promptAdditions, setPromptAdditions] = useState<PromptAdditions | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  // Guidance form state
  const [editingRule, setEditingRule] = useState<GuidanceRule | null>(null)
  const [ruleName, setRuleName] = useState('')
  const [ruleInstruction, setRuleInstruction] = useState('')
  const [ruleType, setRuleType] = useState('style')
  const [rulePriority, setRulePriority] = useState(0)

  useEffect(() => {
    loadSummary()
  }, [])

  async function loadSummary() {
    setLoading(true)
    try {
      const [s, c, p] = await Promise.all([
        getToneSummary(),
        getToneConfig(),
        getPromptAdditions()
      ])
      setSummary(s)
      setConfig(c)
      setPromptAdditions(p)
    } catch (e) {
      console.error('Failed to load tone summary:', e)
    }
    setLoading(false)
  }

  async function handleSaveConfig() {
    if (!config) return
    setSaving(true)
    try {
      await updateToneConfig(config)
      await loadSummary()
    } catch (e) {
      console.error('Failed to save config:', e)
    }
    setSaving(false)
  }

  async function handleApplyPreset(presetId: string) {
    try {
      await applyTonePreset(presetId)
      await loadSummary()
    } catch (e) {
      console.error('Failed to apply preset:', e)
    }
  }

  async function handleSaveRule() {
    try {
      if (editingRule) {
        await updateGuidanceRule(editingRule.id, {
          name: ruleName,
          instruction: ruleInstruction,
          ruleType,
          priority: rulePriority
        })
      } else {
        await createGuidanceRule({
          name: ruleName,
          instruction: ruleInstruction,
          ruleType,
          priority: rulePriority
        })
      }
      setEditingRule(null)
      setRuleName('')
      setRuleInstruction('')
      setRuleType('style')
      setRulePriority(0)
      await loadSummary()
    } catch (e) {
      console.error('Failed to save rule:', e)
    }
  }

  async function handleDeleteRule(id: string) {
    if (!confirm('Delete this guidance rule?')) return
    try {
      await deleteGuidanceRule(id)
      await loadSummary()
    } catch (e) {
      console.error('Failed to delete rule:', e)
    }
  }

  async function handleToggleRule(rule: GuidanceRule) {
    try {
      await updateGuidanceRule(rule.id, { enabled: !rule.enabled })
      await loadSummary()
    } catch (e) {
      console.error('Failed to toggle rule:', e)
    }
  }

  function openEditRule(rule: GuidanceRule) {
    setEditingRule(rule)
    setRuleName(rule.name)
    setRuleInstruction(rule.instruction)
    setRuleType(rule.ruleType)
    setRulePriority(rule.priority)
  }

  function openNewRule() {
    setEditingRule(null)
    setRuleName('')
    setRuleInstruction('')
    setRuleType('style')
    setRulePriority(0)
  }

  if (loading) {
    return <div className="p-6 text-center text-gray-400">Loading...</div>
  }

  const tabs = [
    { id: 'overview' as const, label: 'Overview', icon: MessageSquare },
    { id: 'config' as const, label: 'Configuration', icon: Settings },
    { id: 'presets' as const, label: 'Presets', icon: Sparkles },
    { id: 'guidance' as const, label: 'Guidance Rules', icon: BookOpen },
    { id: 'preview' as const, label: 'Prompt Preview', icon: Eye },
  ]

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white">Tone of Voice & Guidance</h1>
        <p className="text-gray-400 mt-1">Configure AI personality, tone, and custom instructions</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-slate-800 rounded-lg p-1">
        {tabs.map(tab => {
          const Icon = tab.icon
          const isActive = activeTab === tab.id
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                isActive
                  ? 'bg-indigo-600 text-white'
                  : 'text-gray-400 hover:text-white hover:bg-slate-700'
              }`}
            >
              <Icon className="w-4 h-4" />
              {tab.label}
            </button>
          )
        })}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && summary && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Status</div>
              <div className="mt-1">
                {summary.enabled ? (
                  <span className="text-emerald-400 font-semibold flex items-center gap-1">
                    <CheckCircle className="w-4 h-4" /> Enabled
                  </span>
                ) : (
                  <span className="text-gray-500 font-semibold flex items-center gap-1">
                    <XCircle className="w-4 h-4" /> Disabled
                  </span>
                )}
              </div>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Tone Style</div>
              <div className="mt-1 text-white font-semibold capitalize">{summary.currentToneStyle}</div>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Personality</div>
              <div className="mt-1 text-white font-semibold capitalize">{summary.currentPersonality}</div>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Answer Length</div>
              <div className="mt-1 text-white font-semibold capitalize">{summary.currentAnswerLength}</div>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Guidance Rules</div>
              <div className="mt-1 text-2xl font-bold text-white">
                {summary.enabledGuidanceRules}<span className="text-sm text-gray-400"> / {summary.totalGuidanceRules}</span>
              </div>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Available Presets</div>
              <div className="mt-1 text-2xl font-bold text-white">{summary.totalPresets}</div>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <div className="text-sm text-gray-400">Analytics Snapshots</div>
              <div className="mt-1 text-2xl font-bold text-white">{summary.recentAnalytics.length}</div>
            </div>
          </div>

          {summary.guidanceRules.length > 0 && (
            <div className="bg-slate-800 rounded-lg p-4">
              <h3 className="text-sm font-semibold text-gray-300 mb-3">Active Guidance Rules</h3>
              <div className="space-y-2">
                {summary.guidanceRules.filter(r => r.enabled).map(rule => (
                  <div key={rule.id} className="flex items-start gap-3 bg-slate-700 rounded p-3">
                    <CheckCircle className="w-4 h-4 text-emerald-400 mt-0.5 flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-white text-sm font-medium">{rule.name}</span>
                        <span className="text-xs px-2 py-0.5 rounded bg-slate-600 text-gray-300">{rule.ruleType}</span>
                        <span className="text-xs text-gray-400">Priority: {rule.priority}</span>
                      </div>
                      <p className="text-gray-400 text-sm mt-1">{rule.instruction}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Configuration Tab */}
      {activeTab === 'config' && config && (
        <div className="space-y-6">
          <div className="bg-slate-800 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-white mb-4">Tone Configuration</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Tone Style</label>
                <select
                  value={config.toneStyle}
                  onChange={e => setConfig({ ...config, toneStyle: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                >
                  <option value="professional">Professional</option>
                  <option value="friendly">Friendly</option>
                  <option value="humorous">Humorous</option>
                  <option value="casual">Casual</option>
                  <option value="formal">Formal</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Personality</label>
                <select
                  value={config.personality}
                  onChange={e => setConfig({ ...config, personality: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                >
                  <option value="helpful">Helpful</option>
                  <option value="enthusiastic">Enthusiastic</option>
                  <option value="playful">Playful</option>
                  <option value="direct">Direct</option>
                  <option value="thorough">Thorough</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Answer Length</label>
                <select
                  value={config.answerLength}
                  onChange={e => setConfig({ ...config, answerLength: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                >
                  <option value="short">Short (1-3 sentences)</option>
                  <option value="medium">Medium (3-5 sentences)</option>
                  <option value="long">Long (detailed)</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Language</label>
                <input
                  type="text"
                  value={config.language}
                  onChange={e => setConfig({ ...config, language: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Formality Level (1-10)</label>
                <input
                  type="range"
                  min="1"
                  max="10"
                  value={config.formalityLevel}
                  onChange={e => setConfig({ ...config, formalityLevel: parseInt(e.target.value) })}
                  className="w-full"
                />
                <div className="text-xs text-gray-400 text-center">{config.formalityLevel} (1=casual, 10=formal)</div>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Empathy Level (1-10)</label>
                <input
                  type="range"
                  min="1"
                  max="10"
                  value={config.empathyLevel}
                  onChange={e => setConfig({ ...config, empathyLevel: parseInt(e.target.value) })}
                  className="w-full"
                />
                <div className="text-xs text-gray-400 text-center">{config.empathyLevel} (1=direct, 10=empathetic)</div>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Conciseness Level (1-10)</label>
                <input
                  type="range"
                  min="1"
                  max="10"
                  value={config.concisenessLevel}
                  onChange={e => setConfig({ ...config, concisenessLevel: parseInt(e.target.value) })}
                  className="w-full"
                />
                <div className="text-xs text-gray-400 text-center">{config.concisenessLevel} (1=elaborate, 10=concise)</div>
              </div>
              <div className="flex items-center gap-3">
                <input
                  type="checkbox"
                  id="humorEnabled"
                  checked={config.humorEnabled}
                  onChange={e => setConfig({ ...config, humorEnabled: e.target.checked })}
                  className="w-4 h-4 rounded"
                />
                <label htmlFor="humorEnabled" className="text-sm text-gray-300">Allow Humor</label>
              </div>
              <div className="flex items-center gap-3">
                <input
                  type="checkbox"
                  id="enabled"
                  checked={config.enabled}
                  onChange={e => setConfig({ ...config, enabled: e.target.checked })}
                  className="w-4 h-4 rounded"
                />
                <label htmlFor="enabled" className="text-sm text-gray-300">Enable Tone of Voice</label>
              </div>
            </div>

            <div className="mt-4 space-y-3">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Greeting Template</label>
                <input
                  type="text"
                  value={config.greetingTemplate}
                  onChange={e => setConfig({ ...config, greetingTemplate: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Closing Template</label>
                <input
                  type="text"
                  value={config.closingTemplate}
                  onChange={e => setConfig({ ...config, closingTemplate: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Escalation Template</label>
                <input
                  type="text"
                  value={config.escalationTemplate}
                  onChange={e => setConfig({ ...config, escalationTemplate: e.target.value })}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Custom Instructions</label>
                <textarea
                  value={config.customInstructions || ''}
                  onChange={e => setConfig({ ...config, customInstructions: e.target.value })}
                  rows={3}
                  className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                  placeholder="Additional custom instructions for the AI..."
                />
              </div>
            </div>

            <div className="mt-4 flex gap-2">
              <button
                onClick={handleSaveConfig}
                disabled={saving}
                className="flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50"
              >
                <Save className="w-4 h-4" />
                {saving ? 'Saving...' : 'Save Configuration'}
              </button>
              <button
                onClick={loadSummary}
                className="flex items-center gap-2 px-4 py-2 bg-slate-700 text-white rounded-md text-sm hover:bg-slate-600"
              >
                <RefreshCw className="w-4 h-4" />
                Reset
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Presets Tab */}
      {activeTab === 'presets' && summary && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Tone Presets</h3>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {summary.presets.map(preset => (
              <div key={preset.id} className="bg-slate-800 rounded-lg p-4">
                <div className="flex items-start justify-between">
                  <div>
                    <h4 className="text-white font-semibold">{preset.name}</h4>
                    {preset.description && (
                      <p className="text-gray-400 text-sm mt-1">{preset.description}</p>
                    )}
                  </div>
                  <button
                    onClick={() => handleApplyPreset(preset.id)}
                    className="px-3 py-1 bg-indigo-600 text-white rounded text-xs hover:bg-indigo-700"
                  >
                    Apply
                  </button>
                </div>
                <div className="mt-3 grid grid-cols-2 gap-2 text-xs">
                  <div className="bg-slate-700 rounded px-2 py-1">
                    <span className="text-gray-400">Style:</span>
                    <span className="text-white ml-1 capitalize">{preset.toneStyle}</span>
                  </div>
                  <div className="bg-slate-700 rounded px-2 py-1">
                    <span className="text-gray-400">Personality:</span>
                    <span className="text-white ml-1 capitalize">{preset.personality}</span>
                  </div>
                  <div className="bg-slate-700 rounded px-2 py-1">
                    <span className="text-gray-400">Length:</span>
                    <span className="text-white ml-1 capitalize">{preset.answerLength}</span>
                  </div>
                  <div className="bg-slate-700 rounded px-2 py-1">
                    <span className="text-gray-400">Formality:</span>
                    <span className="text-white ml-1">{preset.formalityLevel}/10</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Guidance Rules Tab */}
      {activeTab === 'guidance' && summary && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Guidance Rules</h3>
            <button
              onClick={openNewRule}
              className="flex items-center gap-2 px-3 py-1.5 bg-indigo-600 text-white rounded text-sm hover:bg-indigo-700"
            >
              <Plus className="w-4 h-4" />
              New Rule
            </button>
          </div>

          {editingRule || ruleName || ruleInstruction ? (
            <div className="bg-slate-800 rounded-lg p-4 mb-4">
              <h4 className="text-sm font-semibold text-gray-300 mb-3">
                {editingRule ? 'Edit Rule' : 'New Rule'}
              </h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Name</label>
                  <input
                    type="text"
                    value={ruleName}
                    onChange={e => setRuleName(e.target.value)}
                    className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                    placeholder="Rule name..."
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Type</label>
                  <select
                    value={ruleType}
                    onChange={e => setRuleType(e.target.value)}
                    className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                  >
                    <option value="style">Style</option>
                    <option value="policy">Policy</option>
                    <option value="safety">Safety</option>
                    <option value="escalation">Escalation</option>
                  </select>
                </div>
                <div className="md:col-span-2">
                  <label className="block text-xs text-gray-400 mb-1">Instruction</label>
                  <textarea
                    value={ruleInstruction}
                    onChange={e => setRuleInstruction(e.target.value)}
                    rows={3}
                    className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                    placeholder="What should the AI do?"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Priority (higher = first)</label>
                  <input
                    type="number"
                    value={rulePriority}
                    onChange={e => setRulePriority(parseInt(e.target.value) || 0)}
                    className="w-full bg-slate-700 text-white rounded px-3 py-2 text-sm"
                  />
                </div>
              </div>
              <div className="mt-3 flex gap-2">
                <button
                  onClick={handleSaveRule}
                  disabled={!ruleName || !ruleInstruction}
                  className="flex items-center gap-2 px-3 py-1.5 bg-indigo-600 text-white rounded text-sm hover:bg-indigo-700 disabled:opacity-50"
                >
                  <Save className="w-3 h-3" />
                  Save Rule
                </button>
                <button
                  onClick={openNewRule}
                  className="px-3 py-1.5 bg-slate-700 text-white rounded text-sm hover:bg-slate-600"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : null}

          <div className="space-y-2">
            {summary.guidanceRules.map(rule => (
              <div key={rule.id} className="bg-slate-800 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className={`text-sm font-medium ${rule.enabled ? 'text-white' : 'text-gray-500'}`}>
                        {rule.name}
                      </span>
                      <span className="text-xs px-2 py-0.5 rounded bg-slate-600 text-gray-300">{rule.ruleType}</span>
                      <span className="text-xs text-gray-400">Priority: {rule.priority}</span>
                      {rule.enabled ? (
                        <CheckCircle className="w-4 h-4 text-emerald-400" />
                      ) : (
                        <XCircle className="w-4 h-4 text-gray-500" />
                      )}
                    </div>
                    <p className="text-gray-400 text-sm mt-1">{rule.instruction}</p>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => handleToggleRule(rule)}
                      className="p-1.5 text-gray-400 hover:text-white rounded hover:bg-slate-700"
                      title={rule.enabled ? 'Disable' : 'Enable'}
                    >
                      {rule.enabled ? <CheckCircle className="w-4 h-4" /> : <XCircle className="w-4 h-4" />}
                    </button>
                    <button
                      onClick={() => openEditRule(rule)}
                      className="p-1.5 text-gray-400 hover:text-white rounded hover:bg-slate-700"
                      title="Edit"
                    >
                      <Settings className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handleDeleteRule(rule.id)}
                      className="p-1.5 text-gray-400 hover:text-red-400 rounded hover:bg-slate-700"
                      title="Delete"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Prompt Preview Tab */}
      {activeTab === 'preview' && promptAdditions && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Prompt Additions Preview</h3>
            <button
              onClick={loadSummary}
              className="flex items-center gap-2 px-3 py-1.5 bg-slate-700 text-white rounded text-sm hover:bg-slate-600"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
          </div>

          {promptAdditions.toneInstructions && (
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm font-semibold text-indigo-400 mb-2">Tone Instructions (injected into AI prompt)</h4>
              <pre className="bg-slate-900 rounded p-3 text-sm text-gray-300 whitespace-pre-wrap font-mono">
                {promptAdditions.toneInstructions}
              </pre>
            </div>
          )}

          {promptAdditions.guidanceInstructions && (
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm font-semibold text-emerald-400 mb-2">Guidance Rules (injected into AI prompt)</h4>
              <pre className="bg-slate-900 rounded p-3 text-sm text-gray-300 whitespace-pre-wrap font-mono">
                {promptAdditions.guidanceInstructions}
              </pre>
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm font-semibold text-amber-400 mb-2">Greeting Template</h4>
              <p className="text-white text-sm">{promptAdditions.greetingTemplate}</p>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm font-semibold text-amber-400 mb-2">Closing Template</h4>
              <p className="text-white text-sm">{promptAdditions.closingTemplate}</p>
            </div>
            <div className="bg-slate-800 rounded-lg p-4">
              <h4 className="text-sm font-semibold text-amber-400 mb-2">Escalation Template</h4>
              <p className="text-white text-sm">{promptAdditions.escalationTemplate}</p>
            </div>
          </div>

          {!promptAdditions.toneInstructions && !promptAdditions.guidanceInstructions && (
            <div className="bg-slate-800 rounded-lg p-6 text-center">
              <Sparkles className="w-8 h-8 text-gray-500 mx-auto mb-2" />
              <p className="text-gray-400">No prompt additions configured. Enable tone of voice and add guidance rules to see the preview.</p>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
