import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import {
  Bot, Loader2, CheckCircle2, AlertCircle, Save, Brain, MessageSquare,
  Zap, Shield, Settings
} from '../components/icons'

export function OrganizationAiAgent() {
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [data, setData] = useState<any>({})
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    api.getOrganizationSettings()
      .then(setData)
      .catch(() => setError('Failed to load AI agent settings'))
      .finally(() => setLoading(false))
  }, [])

  async function handleSave() {
    setSaving(true)
    setSaved(false)
    try {
      await api.updateOrganizationSettings(data)
      setSaved(true)
      setTimeout(() => setSaved(false), 3000)
    } catch {
      setError('Failed to save AI agent settings')
    } finally {
      setSaving(false)
    }
  }

  function update(key: string, value: any) {
    setData((prev: any) => ({ ...prev, [key]: value }))
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
        <span className="ml-2 text-sm text-gray-500">Loading AI agent settings...</span>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-ink)]">AI Agent Configuration</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-1">
            Configure the shared AI agent that powers auto-replies across all channels.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {saved && (
            <div className="flex items-center gap-2 text-sm text-green-600 bg-green-50 px-3 py-1.5 rounded-lg">
              <CheckCircle2 className="h-4 w-4" />
              <span>Saved</span>
            </div>
          )}
          <button onClick={handleSave} disabled={saving}
            className="px-4 py-2 bg-[var(--color-accent)] text-[var(--color-on-accent)] text-sm font-medium rounded-lg hover:opacity-90 transition disabled:opacity-50 flex items-center gap-2">
            {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
            <span>Save</span>
          </button>
        </div>
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Enable/Disable */}
      <Section title="AI Agent Status" icon={Bot}>
        <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
          <div>
            <p className="text-sm font-medium text-[var(--color-ink)]">Enable AI Agent</p>
            <p className="text-xs text-[var(--color-ink-muted)] mt-1">
              When enabled, the AI agent processes inbound messages from all connected channels.
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" checked={data?.aiAgentEnabled || false}
              onChange={(e) => update('aiAgentEnabled', e.target.checked)} className="sr-only peer" />
            <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
          </label>
        </div>
      </Section>

      {/* Agent Behavior */}
      <Section title="Agent Behavior" icon={Brain}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Tone</label>
            <select value={data?.aiAgentTone || 'professional'}
              onChange={(e) => update('aiAgentTone', e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none">
              <option value="professional">Professional</option>
              <option value="friendly">Friendly</option>
              <option value="empathetic">Empathetic</option>
              <option value="casual">Casual</option>
              <option value="formal">Formal</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Default Language</label>
            <select value={data?.aiAgentLanguage || 'en'}
              onChange={(e) => update('aiAgentLanguage', e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none">
              <option value="en">English</option>
              <option value="es">Spanish</option>
              <option value="fr">French</option>
              <option value="de">German</option>
              <option value="ar">Arabic</option>
              <option value="hi">Hindi</option>
              <option value="auto">Auto-detect</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">
              Max Context Messages
            </label>
            <input type="number" min={10} max={100} value={data?.aiAgentMaxContextMessages || 40}
              onChange={(e) => update('aiAgentMaxContextMessages', parseInt(e.target.value))}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none" />
            <p className="text-xs text-[var(--color-ink-muted)] mt-1">
              How many past messages to include in the AI context window.
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">
              Auto-reply Confidence Threshold
            </label>
            <input type="number" min={0} max={1} step={0.05}
              value={data?.aiAgentAutoReplyConfidence ?? 0.85}
              onChange={(e) => update('aiAgentAutoReplyConfidence', parseFloat(e.target.value))}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none" />
            <p className="text-xs text-[var(--color-ink-muted)] mt-1">
              Replies above this confidence are sent automatically. Below requires human review.
            </p>
          </div>
        </div>
      </Section>

      {/* Escalation */}
      <Section title="Escalation Keywords" icon={Shield}>
        <div>
          <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">
            Escalation Keywords (JSON array)
          </label>
          <textarea rows={3}
            value={data?.aiAgentEscalationKeywords || '[]'}
            onChange={(e) => update('aiAgentEscalationKeywords', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none resize-none font-mono"
            placeholder='["complaint", "manager", "legal", "refund"]' />
          <p className="text-xs text-[var(--color-ink-muted)] mt-1">
            Messages containing these keywords are escalated to a human agent regardless of confidence.
          </p>
        </div>
      </Section>

      {/* Custom System Prompt */}
      <Section title="Custom System Prompt" icon={Settings}>
        <div>
          <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">
            System Prompt Override
          </label>
          <textarea rows={6}
            value={data?.aiAgentSystemPrompt || ''}
            onChange={(e) => update('aiAgentSystemPrompt', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none resize-none font-mono"
            placeholder="You are a helpful assistant for [Business Name]. Always be polite and reference our services catalog..." />
          <p className="text-xs text-[var(--color-ink-muted)] mt-1">
            Leave empty to use the default prompt. This prompt is combined with your business profile, knowledge base, and conversation history.
          </p>
        </div>
      </Section>

      {/* Memory & RAG */}
      <Section title="Memory & RAG" icon={Brain}>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
            <div>
              <p className="text-sm font-medium text-[var(--color-ink)]">Enable Memory</p>
              <p className="text-xs text-[var(--color-ink-muted)] mt-1">
                Store key facts extracted from conversations for future context.
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" checked={data?.aiMemoryEnabled || false}
                onChange={(e) => update('aiMemoryEnabled', e.target.checked)} className="sr-only peer" />
              <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
            </label>
          </div>
          <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
            <div>
              <p className="text-sm font-medium text-[var(--color-ink)]">Enable RAG</p>
              <p className="text-xs text-[var(--color-ink-muted)] mt-1">
                Use Retrieval-Augmented Generation to search knowledge base for relevant context.
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" checked={data?.aiRagEnabled || false}
                onChange={(e) => update('aiRagEnabled', e.target.checked)} className="sr-only peer" />
              <div className="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
            </label>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">Max Memory Entries</label>
              <input type="number" min={10} max={500} value={data?.aiMemoryMaxEntries || 100}
                onChange={(e) => update('aiMemoryMaxEntries', parseInt(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none" />
            </div>
            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">RAG Top-K Results</label>
              <input type="number" min={1} max={20} value={data?.aiRagTopK || 5}
                onChange={(e) => update('aiRagTopK', parseInt(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none" />
            </div>
            <div>
              <label className="block text-sm font-medium text-[var(--color-ink)] mb-1">RAG Similarity Threshold</label>
              <input type="number" min={0} max={1} step={0.05}
                value={data?.aiRagSimilarityThreshold ?? 0.75}
                onChange={(e) => update('aiRagSimilarityThreshold', parseFloat(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-[var(--color-accent)] focus:outline-none" />
            </div>
          </div>
        </div>
      </Section>

      {/* Per-Channel Auto-Reply */}
      <Section title="Per-Channel Auto-Reply" icon={MessageSquare}>
        <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
          {[
            { key: 'autoReplyWhatsapp', label: 'WhatsApp' },
            { key: 'autoReplySms', label: 'SMS' },
            { key: 'autoReplyEmail', label: 'Email' },
            { key: 'autoReplyTelegram', label: 'Telegram' },
            { key: 'autoReplyFacebook', label: 'Facebook' },
            { key: 'autoReplyInstagram', label: 'Instagram' },
            { key: 'autoReplyLine', label: 'LINE' },
            { key: 'autoReplyTiktok', label: 'TikTok' },
            { key: 'autoReplyVoice', label: 'Voice' },
            { key: 'autoReplyLiveChat', label: 'Live Chat' },
          ].map(ch => (
            <label key={ch.key} className="flex items-center gap-2 p-3 border border-gray-200 rounded-lg cursor-pointer hover:bg-gray-50">
              <input type="checkbox"
                checked={data?.[ch.key] !== false}
                onChange={(e) => update(ch.key, e.target.checked)}
                className="rounded border-gray-300 text-[var(--color-accent)] focus:ring-[var(--color-accent)]" />
              <span className="text-sm text-[var(--color-ink)]">{ch.label}</span>
            </label>
          ))}
        </div>
      </Section>

      {/* Info Box */}
      <div className="p-4 bg-purple-50 border border-purple-200 rounded-lg">
        <div className="flex items-start gap-3">
          <Zap className="h-5 w-5 text-purple-600 mt-0.5 shrink-0" />
          <div>
            <p className="text-sm font-medium text-purple-900">Shared AI Agent Architecture</p>
            <p className="text-xs text-purple-700 mt-1">
              One AI agent handles all channels. It reads from your organization profile, knowledge base,
              and client conversation history to generate contextually aware replies. Each channel can
              independently enable/disable auto-reply while sharing the same AI brain.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

function Section({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) {
  return (
    <div className="border border-gray-200 rounded-lg bg-white">
      <div className="flex items-center gap-2 px-4 py-3 border-b border-gray-200">
        <Icon className="h-4 w-4 text-[var(--color-accent)]" />
        <h2 className="text-sm font-semibold text-[var(--color-ink)]">{title}</h2>
      </div>
      <div className="p-4">{children}</div>
    </div>
  )
}
