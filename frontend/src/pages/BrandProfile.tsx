import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { api } from '../lib/api'
import { listIndustries } from '../lib/api'
import {
  Loader2, Save, Plus, Trash2, Check, X, AlertTriangle,
  Key, Brain, BookOpen, Globe
} from '../components/icons'

type Tab = 'identity' | 'products' | 'policies' | 'ai' | 'keys' | 'knowledge'

// Map route tab names to internal tab keys
const routeTabMap: Record<string, Tab> = {
  'profile': 'identity',
  'keys': 'keys',
  'ai-config': 'ai',
  'knowledge': 'knowledge',
}

export function BrandProfile({ tab: initialTab }: { tab?: string }) {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { brand } = useAuth()
  const brandId = id || brand?.id
  const resolvedTab: Tab = initialTab ? (routeTabMap[initialTab] ?? 'identity') : 'identity'
  const [tab, setTab] = useState<Tab>(resolvedTab)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [profile, setProfile] = useState<any>(null)

  // Identity form
  const [name, setName] = useState('')
  const [slug, setSlug] = useState('')
  const [description, setDescription] = useState('')
  const [tagline, setTagline] = useState('')
  const [bio, setBio] = useState('')
  const [website, setWebsite] = useState('')
  const [email, setEmail] = useState('')
  const [phone, setPhone] = useState('')
  const [address, setAddress] = useState('')
  const [industryId, setIndustryId] = useState('')

  // Industries
  const [industries, setIndustries] = useState<any[]>([])

  // AI config
  const [aiProvider, setAiProvider] = useState('anthropic')
  const [aiModel, setAiModel] = useState('claude-sonnet-4-20250514')
  const [aiTemperature, setAiTemperature] = useState(0.7)
  const [aiMaxTokens, setAiMaxTokens] = useState(4096)
  const [aiSystemPrompt, setAiSystemPrompt] = useState('')
  const [aiApiKey, setAiApiKey] = useState('')
  const [ragEnabled, setRagEnabled] = useState(false)

  // Credentials
  const [credentials, setCredentials] = useState<any[]>([])
  const [credChannel, setCredChannel] = useState('whatsapp')
  const [credFields, setCredFields] = useState<Record<string, string>>({})
  const [verifying, setVerifying] = useState<string | null>(null)

  // Knowledge base
  const [entries, setEntries] = useState<any[]>([])
  const [newTitle, setNewTitle] = useState('')
  const [newContent, setNewContent] = useState('')
  const [newTopic, setNewTopic] = useState('')

  useEffect(() => {
    if (!brandId) return
    loadProfile()
  }, [brandId])

  async function loadProfile() {
    if (!brandId) return
    try {
      setLoading(true)
      setError(null)
      const [prof, creds, kb, ai, inds] = await Promise.allSettled([
        api.getBrandProfile(brandId),
        api.listBrandCredentials(brandId),
        api.listBrandKnowledge(brandId),
        api.getBrandAIConfig(brandId),
        listIndustries(),
      ])
      if (prof.status === 'fulfilled') {
        const b = prof.value.brand
        setProfile(prof.value)
        setName(b.name || '')
        setSlug(b.slug || '')
        setDescription(b.description || '')
        setTagline(b.tagline || '')
        setBio(b.bio || '')
        setWebsite(b.website || '')
        setEmail(b.email || '')
        setPhone(b.phone || '')
        setAddress(b.address || '')
        setIndustryId(b.industryId || '')
      }
      if (creds.status === 'fulfilled') setCredentials(creds.value)
      if (kb.status === 'fulfilled') setEntries(kb.value)
      if (ai.status === 'fulfilled') {
        const a = ai.value
        setAiProvider(a.aiProvider || 'anthropic')
        setAiModel(a.modelId || 'claude-sonnet-4-20250514')
        setAiTemperature(a.temperature ?? 0.7)
        setAiMaxTokens(a.maxTokens ?? 4096)
        setAiSystemPrompt(a.systemPrompt || '')
        setRagEnabled(a.ragEnabled ?? false)
      }
      if (inds.status === 'fulfilled') setIndustries(inds.value)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  async function saveIdentity() {
    if (!brandId) return
    setSaving(true)
    try {
      await api.updateBrand(brandId, { name, slug, description, tagline, bio, website, email, phone, address, industry_id: industryId || undefined })
      setError(null)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setSaving(false)
    }
  }

  async function saveAIConfig() {
    if (!brandId) return
    setSaving(true)
    try {
      await api.updateBrandAIConfig(brandId, {
        aiProvider, modelId: aiModel, temperature: aiTemperature,
        maxTokens: aiMaxTokens, systemPrompt: aiSystemPrompt,
        ragEnabled, apiKey: aiApiKey,
      })
      setError(null)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setSaving(false)
    }
  }

  async function saveCredentials() {
    if (!brandId) return
    setSaving(true)
    try {
      await api.upsertBrandCredentials(brandId, credChannel, credFields)
      setCredFields({})
      loadProfile()
      setError(null)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setSaving(false)
    }
  }

  async function verifyCred(channel: string) {
    if (!brandId) return
    setVerifying(channel)
    try {
      const r = await api.verifyBrandCredentials(brandId, channel)
      if (!r.connected) setError(r.error || 'Verification failed')
      else setError(null)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setVerifying(null)
    }
  }

  async function createKnowledgeEntry() {
    if (!brandId || !newTitle.trim()) return
    setSaving(true)
    try {
      await api.createBrandKnowledge(brandId, { title: newTitle, content: newContent, topic: newTopic })
      setNewTitle('')
      setNewContent('')
      setNewTopic('')
      loadProfile()
      setError(null)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setSaving(false)
    }
  }

  async function deleteKnowledgeEntry(entryId: string) {
    if (!brandId) return
    try {
      await api.deleteBrandKnowledge(brandId, entryId)
      loadProfile()
    } catch (e: any) {
      setError(e.message)
    }
  }

  const channels = ['whatsapp', 'sms', 'telegram', 'line', 'facebook', 'instagram', 'tiktok', 'email', 'voice', 'ai']
  const credFieldMap: Record<string, string[]> = {
    whatsapp: ['accessToken', 'phoneNumberId'],
    sms: ['accountSid', 'authToken', 'fromPhone'],
    telegram: ['botToken'],
    line: ['channelAccessToken'],
    facebook: ['pageAccessToken', 'pageId'],
    instagram: ['accessToken', 'igAccountId'],
    tiktok: ['accessToken', 'pageId'],
    email: ['smtpHost', 'smtpPort', 'smtpUser', 'smtpPass'],
    voice: ['accountSid', 'authToken', 'fromNumber'],
    ai: ['apiKey', 'baseUrl'],
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-6 h-6 animate-text text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">Loading brand profile...</span>
      </div>
    )
  }

  const tabs: { key: Tab; label: string; icon: any }[] = [
    { key: 'identity', label: 'Identity', icon: Globe },
    { key: 'products', label: 'Products & Services', icon: Brain },
    { key: 'policies', label: 'Policies & Rules', icon: BookOpen },
    { key: 'ai', label: 'AI Config', icon: Brain },
    { key: 'keys', label: 'Channel Keys', icon: Key },
    { key: 'knowledge', label: 'Knowledge Base', icon: BookOpen },
  ]

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">{profile?.brand?.name || 'Brand Profile'}</h1>
          <p className="text-muted-foreground text-sm mt-1">
            Manage brand identity, API keys, AI configuration, and knowledge base
          </p>
        </div>
        {!brandId && (
          <button
            onClick={() => navigate('/brands/select')}
            className="px-3 py-1.5 text-sm border rounded-[var(--radius-md)] hover:bg-muted"
          >
            Select Brand
          </button>
        )}
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-[var(--radius-md)] border border-red-200 bg-red-50 p-3 text-sm text-red-700">
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span className="flex-1">{error}</span>
          <button onClick={() => setError(null)}><X className="w-4 h-4" /></button>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 border-b">
        {tabs.map((t) => (
          <button
            key={t.key}
            onClick={() => setTab(t.key)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              tab === t.key
                ? 'border-foreground text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* Identity Tab */}
      {tab === 'identity' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Field label="Brand Name" value={name} onChange={setName} />
          <Field label="Slug" value={slug} onChange={setSlug} />
          <Field label="Tagline" value={tagline} onChange={setTagline} />
          <Field label="Website" value={website} onChange={setWebsite} />
          <Field label="Email" value={email} onChange={setEmail} />
          <Field label="Phone" value={phone} onChange={setPhone} />
          <div className="md:col-span-2">
            <label className="text-sm font-medium">Description</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
              rows={3}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>
          <div className="md:col-span-2">
            <label className="text-sm font-medium">Bio</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
              rows={5}
              value={bio}
              onChange={(e) => setBio(e.target.value)}
            />
          </div>
          <Field label="Address" value={address} onChange={setAddress} />
          <div>
            <label className="text-sm font-medium">Industry</label>
            <select
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
              value={industryId}
              onChange={(e) => setIndustryId(e.target.value)}
            >
              <option value="">Select industry...</option>
              {industries.map((ind) => (
                <option key={ind.id} value={ind.id}>
                  {ind.icon ? ind.icon + ' ' : ''}{ind.name}
                </option>
              ))}
            </select>
            <p className="text-xs text-muted-foreground mt-1">
              Industry selection helps group WhatsApp templates and flows
            </p>
          </div>
          <div className="md:col-span-2 flex justify-end">
            <button
              onClick={saveIdentity}
              disabled={saving}
              className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-[var(--radius-md)] text-sm hover:opacity-90 disabled:opacity-50"
            >
              {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
              Save Changes
            </button>
          </div>
        </div>
      )}

      {/* Products & Services Tab */}
      {tab === 'products' && (
        <div className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Products, services, and prices are stored as JSON in the brand profile.
            Edit them directly below.
          </p>
          <div>
            <label className="text-sm font-medium">Products (JSON array)</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm font-mono"
              rows={8}
              defaultValue={profile?.brand?.products || '[]'}
            />
          </div>
          <div>
            <label className="text-sm font-medium">Services (JSON array)</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm font-mono"
              rows={8}
              defaultValue={profile?.brand?.services || '[]'}
            />
          </div>
          <div>
            <label className="text-sm font-medium">Prices (JSON object)</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm font-mono"
              rows={8}
              defaultValue={profile?.brand?.prices || '{}'}
            />
          </div>
        </div>
      )}

      {/* Policies & Rules Tab */}
      {tab === 'policies' && (
        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium">Policies (JSON)</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm font-mono"
              rows={8}
              defaultValue={profile?.brand?.policies || '{}'}
            />
          </div>
          <div>
            <label className="text-sm font-medium">Rules (JSON)</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm font-mono"
              rows={8}
              defaultValue={profile?.brand?.rules || '{}'}
            />
          </div>
          <div>
            <label className="text-sm font-medium">Tone Guidelines</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
              rows={6}
              defaultValue={profile?.brand?.toneGuidelines || ''}
            />
          </div>
        </div>
      )}

      {/* AI Config Tab */}
      {tab === 'ai' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="text-sm font-medium">AI Provider</label>
              <select
                className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
                value={aiProvider}
                onChange={(e) => setAiProvider(e.target.value)}
              >
                <option value="anthropic">Anthropic</option>
                <option value="openai">OpenAI</option>
                <option value="google">Google</option>
                <option value="custom">Custom</option>
              </select>
            </div>
            <Field label="Model" value={aiModel} onChange={setAiModel} />
            <Field label="API Key" value={aiApiKey} onChange={setAiApiKey} type="password" />
            <div>
              <label className="text-sm font-medium">Temperature: {aiTemperature}</label>
              <input
                type="range" min="0" max="2" step="0.1"
                className="w-full mt-1"
                value={aiTemperature}
                onChange={(e) => setAiTemperature(parseFloat(e.target.value))}
              />
            </div>
            <div>
              <label className="text-sm font-medium">Max Tokens</label>
              <input
                type="number"
                className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
                value={aiMaxTokens}
                onChange={(e) => setAiMaxTokens(parseInt(e.target.value) || 0)}
              />
            </div>
            <div className="flex items-center gap-2 mt-6">
              <input
                type="checkbox" id="ragEnabled"
                checked={ragEnabled}
                onChange={(e) => setRagEnabled(e.target.checked)}
              />
              <label htmlFor="ragEnabled" className="text-sm">Enable RAG</label>
            </div>
          </div>
          <div>
            <label className="text-sm font-medium">System Prompt</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
              rows={8}
              value={aiSystemPrompt}
              onChange={(e) => setAiSystemPrompt(e.target.value)}
              placeholder="You are a helpful assistant for {brand_name}..."
            />
          </div>
          <div className="flex justify-end">
            <button
              onClick={saveAIConfig}
              disabled={saving}
              className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-[var(--radius-md)] text-sm hover:opacity-90 disabled:opacity-50"
            >
              {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
              Save AI Config
            </button>
          </div>
        </div>
      )}

      {/* Channel Keys Tab */}
      {tab === 'keys' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="text-sm font-medium">Channel</label>
              <select
                className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
                value={credChannel}
                onChange={(e) => {
                  setCredChannel(e.target.value)
                  setCredFields({})
                }}
              >
                {channels.map((ch) => (
                  <option key={ch} value={ch}>{ch}</option>
                ))}
              </select>
            </div>
          </div>
          {credFieldMap[credChannel]?.map((field) => (
            <Field
              key={field}
              label={field}
              value={credFields[field] || ''}
              onChange={(v) => setCredFields((prev) => ({ ...prev, [field]: v }))}
              type={field.includes('Key') || field.includes('Token') || field.includes('Pass') ? 'password' : 'text'}
            />
          ))}
          <div className="flex gap-2">
            <button
              onClick={saveCredentials}
              disabled={saving}
              className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-[var(--radius-md)] text-sm hover:opacity-90 disabled:opacity-50"
            >
              {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
              Save Credentials
            </button>
          </div>
          <div className="border-t pt-4">
            <h3 className="text-sm font-medium mb-2">Connected Channels</h3>
            <div className="space-y-2">
              {credentials.map((c) => (
                <div key={c.id} className="flex items-center justify-between p-3 border rounded-[var(--radius-md)]">
                  <div className="flex items-center gap-3">
                    {c.isActive ? <Check className="w-4 h-4 text-green-600" /> : <X className="w-4 h-4 text-red-500" />}
                    <span className="text-sm capitalize">{c.channelType}</span>
                  </div>
                  <button
                    onClick={() => verifyCred(c.channelType)}
                    disabled={verifying === c.channelType}
                    className="text-xs px-2 py-1 border rounded hover:bg-muted"
                  >
                    {verifying === c.channelType ? 'Verifying...' : 'Verify'}
                  </button>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Knowledge Base Tab */}
      {tab === 'knowledge' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Field label="Title" value={newTitle} onChange={setNewTitle} />
            <Field label="Topic" value={newTopic} onChange={setNewTopic} />
          </div>
          <div>
            <label className="text-sm font-medium">Content</label>
            <textarea
              className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
              rows={6}
              value={newContent}
              onChange={(e) => setNewContent(e.target.value)}
            />
          </div>
          <button
            onClick={createKnowledgeEntry}
            disabled={saving || !newTitle.trim()}
            className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-[var(--radius-md)] text-sm hover:opacity-90 disabled:opacity-50"
          >
            {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
            Add Entry
          </button>
          <div className="border-t pt-4">
            <h3 className="text-sm font-medium mb-2">
              Knowledge Entries ({entries.length})
            </h3>
            <div className="space-y-2">
              {entries.map((entry) => (
                <div key={entry.id} className="flex items-start justify-between p-3 border rounded-[var(--radius-md)]">
                  <div>
                    <p className="text-sm font-medium">{entry.title}</p>
                    <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                      {entry.content.slice(0, 200)}
                    </p>
                    {entry.topic && (
                      <span className="inline-block mt-1 text-xs px-2 py-0.5 bg-muted rounded">
                        {entry.topic}
                      </span>
                    )}
                  </div>
                  <button
                    onClick={() => deleteKnowledgeEntry(entry.id)}
                    className="text-red-500 hover:text-red-700 p-1"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              ))}
              {entries.length === 0 && (
                <p className="text-sm text-muted-foreground text-center py-8">
                  No knowledge entries yet. Add your first entry above.
                </p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function Field({ label, value, onChange, type = 'text' }: {
  label: string
  value: string
  onChange: (v: string) => void
  type?: string
}) {
  return (
    <div>
      <label className="text-sm font-medium">{label}</label>
      <input
        type={type}
        className="w-full mt-1 px-3 py-2 border rounded-[var(--radius-md)] text-sm"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  )
}
