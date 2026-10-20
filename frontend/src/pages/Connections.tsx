import { useEffect, useState } from 'react'
import { api, type IntegrationSettings, type Template } from '../lib/api'

export function Connections() {
  const [data, setData] = useState<IntegrationSettings | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  function load() {
    api
      .getIntegrationSettings()
      .then(setData)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [])

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-xl font-semibold text-slate-900">Connections</h1>
        <p className="mt-1 text-sm text-slate-500">
          Connect WhatsApp, tell the agent about your business, and manage templates.
        </p>
      </div>

      {error && <Banner tone="error">{error}</Banner>}
      {notice && <Banner tone="success">{notice}</Banner>}

      {data === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : (
        <>
          <WhatsAppSection
            data={data}
            onSaved={(msg) => {
              setNotice(msg)
              load()
            }}
            onError={setError}
          />
          <BusinessProfileSection
            data={data}
            onSaved={(msg) => {
              setNotice(msg)
              load()
            }}
            onError={setError}
          />
          <TemplatesSection onError={setError} wabaConfigured={!!data.whatsapp.wabaId} />
        </>
      )}
    </div>
  )
}

function Banner({ tone, children }: { tone: 'error' | 'success'; children: React.ReactNode }) {
  const cls =
    tone === 'error'
      ? 'border-red-200 bg-red-50 text-red-700'
      : 'border-emerald-200 bg-emerald-50 text-emerald-700'
  return <div className={`rounded-md border px-4 py-3 text-sm ${cls}`}>{children}</div>
}

function SectionCard({
  title,
  description,
  children,
}: {
  title: string
  description: string
  children: React.ReactNode
}) {
  return (
    <section className="rounded-lg border border-slate-200 bg-white">
      <div className="border-b border-slate-100 px-5 py-4">
        <h2 className="text-sm font-semibold text-slate-900">{title}</h2>
        <p className="mt-0.5 text-xs text-slate-500">{description}</p>
      </div>
      <div className="p-5">{children}</div>
    </section>
  )
}

function Field({
  label,
  value,
  onChange,
  placeholder,
  secret = false,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  secret?: boolean
}) {
  return (
    <div>
      <label className="block text-xs font-medium text-slate-500">{label}</label>
      <input
        type={secret ? 'password' : 'text'}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
      />
    </div>
  )
}

function WhatsAppSection({
  data,
  onSaved,
  onError,
}: {
  data: IntegrationSettings
  onSaved: (msg: string) => void
  onError: (msg: string) => void
}) {
  const [accessToken, setAccessToken] = useState('')
  const [phoneNumberId, setPhoneNumberId] = useState(data.whatsapp.phoneNumberId)
  const [wabaId, setWabaId] = useState(data.whatsapp.wabaId)
  const [appSecret, setAppSecret] = useState('')
  const [verifyToken, setVerifyToken] = useState('')
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)

  const connected = !!data.whatsapp.verifiedAt

  async function save() {
    setSaving(true)
    try {
      await api.updateIntegrationSettings({
        whatsapp: {
          phone_number_id: phoneNumberId,
          waba_id: wabaId,
          ...(accessToken ? { access_token: accessToken } : {}),
          ...(appSecret ? { app_secret: appSecret } : {}),
          ...(verifyToken ? { webhook_verify_token: verifyToken } : {}),
        },
      })
      setAccessToken('')
      setAppSecret('')
      setVerifyToken('')
      onSaved('WhatsApp settings saved.')
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function verify() {
    setVerifying(true)
    try {
      const result = await api.verifyWhatsApp()
      onSaved(`Connected to ${result.verifiedName || result.displayNumber || 'WhatsApp'}.`)
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setVerifying(false)
    }
  }

  return (
    <SectionCard
      title="WhatsApp connection"
      description="Credentials from your Meta app's WhatsApp → API setup page."
    >
      <div className="mb-4 flex items-center gap-2">
        <span
          className={`inline-block h-2 w-2 rounded-full ${connected ? 'bg-emerald-500' : 'bg-slate-300'}`}
        />
        <span className="text-sm text-slate-700">
          {connected
            ? `Connected — ${data.whatsapp.verifiedName || data.whatsapp.displayNumber}`
            : 'Not verified yet'}
        </span>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Field
          label="Access token"
          value={accessToken}
          onChange={setAccessToken}
          placeholder={data.whatsapp.accessTokenMasked ?? 'Paste access token'}
          secret
        />
        <Field label="Phone number ID" value={phoneNumberId} onChange={setPhoneNumberId} />
        <Field
          label="WhatsApp Business Account ID"
          value={wabaId}
          onChange={setWabaId}
          placeholder="Needed for template messages"
        />
        <Field
          label="App secret"
          value={appSecret}
          onChange={setAppSecret}
          placeholder={data.whatsapp.appSecretMasked ?? 'From App settings → Basic'}
          secret
        />
      </div>

      <div className="mt-4 rounded-md border border-slate-200 bg-slate-50 px-4 py-3">
        <p className="text-xs font-medium text-slate-500">Webhook setup (paste into Meta's console)</p>
        <dl className="mt-2 space-y-1 text-sm">
          <div className="flex gap-2">
            <dt className="shrink-0 text-slate-500">Callback URL path:</dt>
            <dd className="font-mono text-xs text-slate-800">{data.whatsapp.webhookCallbackPath}</dd>
          </div>
        </dl>
        <div className="mt-3 max-w-xs">
          <Field
            label="Verify token"
            value={verifyToken}
            onChange={setVerifyToken}
            placeholder={data.whatsapp.webhookVerifyTokenMasked ?? 'Choose a shared secret'}
            secret
          />
        </div>
      </div>

      <div className="mt-4 flex gap-2">
        <button
          onClick={save}
          disabled={saving}
          className="rounded-md bg-slate-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
        >
          {saving ? 'Saving…' : 'Save'}
        </button>
        <button
          onClick={verify}
          disabled={verifying}
          className="rounded-md border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-50"
        >
          {verifying ? 'Verifying…' : 'Verify connection'}
        </button>
      </div>
    </SectionCard>
  )
}

function BusinessProfileSection({
  data,
  onSaved,
  onError,
}: {
  data: IntegrationSettings
  onSaved: (msg: string) => void
  onError: (msg: string) => void
}) {
  const [name, setName] = useState(data.business.name)
  const [description, setDescription] = useState(data.business.description)
  const [guidelines, setGuidelines] = useState(data.business.agentGuidelines)
  const [autoReply, setAutoReply] = useState(data.business.autoReplyEnabled)
  const [saving, setSaving] = useState(false)

  async function save() {
    setSaving(true)
    try {
      await api.updateIntegrationSettings({
        business: {
          name,
          description,
          agent_guidelines: guidelines,
          auto_reply_enabled: autoReply,
        },
      })
      onSaved('Business profile saved.')
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <SectionCard
      title="Business profile & agent guidelines"
      description="What the AI reply engine knows about your business, and how it should behave."
    >
      <div className="space-y-4">
        <Field label="Business name" value={name} onChange={setName} placeholder="Acme Inc." />
        <div>
          <label className="block text-xs font-medium text-slate-500">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
            placeholder="What you do, who you serve."
            className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
          />
        </div>
        <div>
          <label className="block text-xs font-medium text-slate-500">Agent guidelines</label>
          <textarea
            value={guidelines}
            onChange={(e) => setGuidelines(e.target.value)}
            rows={4}
            placeholder="Tone, things to never promise, when to escalate to a human, etc."
            className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            checked={autoReply}
            onChange={(e) => setAutoReply(e.target.checked)}
            className="h-4 w-4 rounded border-slate-300"
          />
          Auto-reply enabled
        </label>
      </div>
      <button
        onClick={save}
        disabled={saving}
        className="mt-4 rounded-md bg-slate-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
      >
        {saving ? 'Saving…' : 'Save'}
      </button>
    </SectionCard>
  )
}

function TemplatesSection({
  onError,
  wabaConfigured,
}: {
  onError: (msg: string) => void
  wabaConfigured: boolean
}) {
  const [templates, setTemplates] = useState<Template[] | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [name, setName] = useState('')
  const [bodyText, setBodyText] = useState('')
  const [category, setCategory] = useState('UTILITY')
  const [saving, setSaving] = useState(false)

  function load() {
    api
      .listTemplates()
      .then(setTemplates)
      .catch((e) => onError((e as Error).message))
  }

  useEffect(() => {
    if (wabaConfigured) load()
  }, [wabaConfigured])

  async function submit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      await api.createTemplate({ name, category, language: 'en_US', body_text: bodyText })
      setName('')
      setBodyText('')
      setShowForm(false)
      load()
    } catch (e) {
      onError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <SectionCard
      title="Template messages"
      description="Pre-approved messages for re-engaging a lead outside the 24h session window."
    >
      {!wabaConfigured ? (
        <p className="text-sm text-slate-500">
          Set a WhatsApp Business Account ID above to manage templates.
        </p>
      ) : (
        <>
          <div className="mb-3 flex justify-end">
            <button
              onClick={() => setShowForm((v) => !v)}
              className="rounded-md border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-50"
            >
              {showForm ? 'Cancel' : 'New template'}
            </button>
          </div>

          {showForm && (
            <form onSubmit={submit} className="mb-4 space-y-3 rounded-md border border-slate-200 p-4">
              <Field label="Name" value={name} onChange={setName} placeholder="order_update" />
              <div>
                <label className="block text-xs font-medium text-slate-500">Category</label>
                <select
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                >
                  <option value="UTILITY">Utility</option>
                  <option value="MARKETING">Marketing</option>
                  <option value="AUTHENTICATION">Authentication</option>
                </select>
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-500">Body text</label>
                <textarea
                  value={bodyText}
                  onChange={(e) => setBodyText(e.target.value)}
                  rows={3}
                  required
                  className="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5 text-sm"
                />
              </div>
              <button
                type="submit"
                disabled={saving}
                className="rounded-md bg-slate-900 px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
              >
                {saving ? 'Submitting…' : 'Submit for approval'}
              </button>
            </form>
          )}

          {templates === null ? (
            <p className="text-sm text-slate-500">Loading…</p>
          ) : templates.length === 0 ? (
            <p className="text-sm text-slate-500">No templates yet.</p>
          ) : (
            <ul className="divide-y divide-slate-100 rounded-md border border-slate-200">
              {templates.map((t) => (
                <li key={t.id} className="flex items-center justify-between px-3 py-2 text-sm">
                  <span className="font-medium text-slate-800">{t.name}</span>
                  <span className="text-xs text-slate-500">
                    {t.category} · {t.language} · {t.status}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </SectionCard>
  )
}
