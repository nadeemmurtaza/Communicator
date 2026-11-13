import { useEffect, useState, useCallback } from 'react'
import { api } from '../../lib/api'
import { WhatsAppIcon, X, Search, Phone, Send, Loader2, CheckCircle2 } from '../../components/icons'

// ---------------------------------------------------------------------------
// Contact type from API
// ---------------------------------------------------------------------------
interface Contact {
  id: string
  name?: string
  phone?: string
  email?: string
  channel?: string
}

// ---------------------------------------------------------------------------
// WhatsApp Start Conversation Modal
// ---------------------------------------------------------------------------
interface Props {
  open: boolean
  onClose: () => void
  onSuccess: () => void
}

export function WhatsAppStartConversation({ open, onClose, onSuccess }: Props) {
  const [step, setStep] = useState<'select' | 'compose' | 'sending' | 'sent'>('select')
  const [contacts, setContacts] = useState<Contact[]>([])
  const [loading, setLoading] = useState(false)
  const [search, setSearch] = useState('')
  const [selectedPhone, setSelectedPhone] = useState('')
  const [selectedName, setSelectedName] = useState('')
  const [selectedContactId, setSelectedContactId] = useState('')
  const [message, setMessage] = useState('')
  const [useTemplate, setUseTemplate] = useState(false)
  const [templates, setTemplates] = useState<any[]>([])
  const [selectedTemplate, setSelectedTemplate] = useState('')
  const [templateVars, setTemplateVars] = useState<Record<string, string>>({})
  const [error, setError] = useState<string | null>(null)
  const [brandId, setBrandId] = useState<string | null>(null)

  useEffect(() => {
    try {
      const b = localStorage.getItem('communicator.brand')
      if (b) {
        const parsed = JSON.parse(b)
        setBrandId(parsed.id || null)
      }
    } catch { /* ignore */ }
  }, [])

  useEffect(() => {
    if (open) {
      resetForm()
      loadContacts()
      if (brandId) loadTemplates()
    }
  }, [open, brandId])

  function resetForm() {
    setStep('select')
    setSearch('')
    setSelectedPhone('')
    setSelectedName('')
    setSelectedContactId('')
    setMessage('')
    setUseTemplate(false)
    setSelectedTemplate('')
    setTemplateVars({})
    setError(null)
  }

  async function loadContacts() {
    try {
      setLoading(true)
      const data = await api.listContacts({ channel: 'whatsapp' })
      setContacts(data as unknown as Contact[])
    } catch {
      // fallback: try without channel filter
      try {
        const data = await api.listContacts()
        setContacts(data as unknown as Contact[])
      } catch { /* ignore */ }
    } finally {
      setLoading(false)
    }
  }

  async function loadTemplates() {
    if (!brandId) return
    try {
      const data = await api.listWhatsAppTemplates(brandId)
      setTemplates(data.filter((t: any) => t.status === 'APPROVED'))
    } catch { /* ignore */ }
  }

  const filteredContacts = useCallback(() => {
    if (!search.trim()) return contacts
    const q = search.toLowerCase()
    return contacts.filter(
      (c) =>
        (c.name || '').toLowerCase().includes(q) ||
        (c.phone || '').includes(q) ||
        (c.email || '').toLowerCase().includes(q)
    )
  }, [contacts, search])

  function handleSelectContact(contact: Contact) {
    setSelectedPhone(contact.phone || '')
    setSelectedName(contact.name || '')
    setSelectedContactId(contact.id || '')
    setStep('compose')
  }

  function handleUsePhone() {
    if (selectedPhone.trim()) {
      setStep('compose')
    }
  }

  async function handleSend() {
    if (!selectedPhone.trim()) {
      setError('Phone number is required')
      return
    }
    if (!useTemplate && !message.trim()) {
      setError('Message is required')
      return
    }
    if (useTemplate && !selectedTemplate) {
      setError('Please select a template')
      return
    }

    setStep('sending')
    setError(null)

    try {
      const brand = localStorage.getItem('communicator.brand')
      const brandData = brand ? JSON.parse(brand) : null
      const bid = brandData?.id

      if (useTemplate && bid) {
        // Send template message
        const template = templates.find((t: any) => t.id === selectedTemplate)
        const variables = template?.variables || []
        const values = variables.map((v: string) => templateVars[v] || '')

        await api.sendWhatsAppTemplate(bid, {
          to: selectedPhone.trim(),
          templateId: selectedTemplate,
          templateName: template?.name || selectedTemplate,
          language: template?.language || 'en',
          variables: values.length > 0 ? values : undefined,
        })
      } else {
        // Send text message — try to create conversation first if contact exists
        if (selectedContactId) {
          try {
            await api.getOrCreateContactConversation(selectedContactId)
          } catch { /* conversation may already exist */ }
        }

        // Send the message via the send-test endpoint (reuses WhatsApp service)
        if (bid) {
          await api.sendWhatsAppTest(bid, {
            to: selectedPhone.trim(),
            body: message.trim(),
          })
        }
      }

      setStep('sent')
      setTimeout(() => {
        onSuccess()
        onClose()
      }, 1500)
    } catch (e: any) {
      setError(e.message || 'Failed to send message')
      setStep('compose')
    }
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-xs">
      <div className="w-full max-w-lg rounded-2xl bg-[var(--color-surface)] shadow-2xl border border-[var(--color-border)] overflow-hidden">
        {/* Header */}
        <div className="border-b border-[var(--color-border)] bg-[var(--color-page)]/70 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-[var(--radius-lg)] bg-emerald-500 text-white">
              <WhatsAppIcon className="h-5 w-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-[var(--color-ink)]">
                {step === 'sent' ? 'Message Sent' : step === 'select' ? 'Start WhatsApp Conversation' : 'Compose Message'}
              </h2>
              <p className="text-xs text-[var(--color-ink-faint)]">
                {step === 'sent' ? 'Waiting...' : step === 'select' ? 'Choose a contact or enter a phone number' : `To: ${selectedName || selectedPhone}`}
              </p>
            </div>
          </div>
          {step !== 'sent' && (
            <button onClick={onClose} className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]">
              <X className="h-5 w-5" />
            </button>
          )}
        </div>

        <div className="p-6">
          {/* Error */}
          {error && (
            <div className="mb-4 rounded-[var(--radius-md)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 flex items-center justify-between">
              <span>{error}</span>
              <button onClick={() => setError(null)} className="text-red-500"><X className="h-4 w-4" /></button>
            </div>
          )}

          {/* Step 1: Select Contact */}
          {step === 'select' && (
            <div className="space-y-4">
              {/* Search */}
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-[var(--color-ink-faint)]" />
                <input
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Search contacts by name, phone, or email..."
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] pl-10 pr-4 py-2.5 text-sm focus:border-[var(--color-accent)] focus:outline-hidden"
                  autoFocus
                />
              </div>

              {/* Or enter phone */}
              <div className="flex items-center gap-3">
                <span className="text-xs text-[var(--color-ink-faint)]">or</span>
                <input
                  type="text"
                  value={selectedPhone}
                  onChange={(e) => setSelectedPhone(e.target.value)}
                  placeholder="Enter phone number (e.g. +14155551234)"
                  className="flex-1 rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm font-mono focus:border-[var(--color-accent)] focus:outline-hidden"
                />
                <button
                  onClick={handleUsePhone}
                  disabled={!selectedPhone.trim()}
                  className="rounded-[var(--radius-md)] bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-40 flex items-center gap-1.5"
                >
                  <Phone className="h-3.5 w-3.5" />
                  Use
                </button>
              </div>

              {/* Contact List */}
              <div className="max-h-64 overflow-y-auto rounded-[var(--radius-md)] border border-[var(--color-border)] divide-y divide-[var(--color-border)]">
                {loading ? (
                  <div className="p-8 text-center">
                    <Loader2 className="h-6 w-6 text-[var(--color-ink-faint)] animate-spin mx-auto" />
                    <p className="mt-2 text-xs text-[var(--color-ink-faint)]">Loading contacts...</p>
                  </div>
                ) : filteredContacts().length === 0 ? (
                  <div className="p-8 text-center text-xs text-[var(--color-ink-faint)]">
                    No contacts found. Enter a phone number above.
                  </div>
                ) : (
                  filteredContacts().map((c) => (
                    <button
                      key={c.id}
                      onClick={() => handleSelectContact(c)}
                      className="w-full flex items-center gap-3 p-3 hover:bg-[var(--color-accent-muted)] text-left transition-colors"
                    >
                      <div className="flex h-9 w-9 items-center justify-center rounded-full bg-emerald-100 text-emerald-700 text-xs font-bold shrink-0">
                        {(c.name || '?').charAt(0).toUpperCase()}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-semibold text-[var(--color-ink)] truncate">
                          {c.name || 'Unknown'}
                        </p>
                        <p className="text-xs text-[var(--color-ink-faint)] font-mono">{c.phone || 'No phone'}</p>
                      </div>
                      {c.channel && (
                        <span className="text-[10px] rounded-full bg-emerald-100 text-emerald-800 px-2 py-0.5 font-semibold">
                          {c.channel}
                        </span>
                      )}
                    </button>
                  ))
                )}
              </div>
            </div>
          )}

          {/* Step 2: Compose Message */}
          {step === 'compose' && (
            <div className="space-y-4">
              {/* Selected contact info */}
              <div className="flex items-center gap-3 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-emerald-100 text-emerald-700 text-sm font-bold shrink-0">
                  {(selectedName || '?').charAt(0).toUpperCase()}
                </div>
                <div className="flex-1">
                  <p className="text-sm font-semibold text-[var(--color-ink)]">{selectedName || 'New Contact'}</p>
                  <p className="text-xs text-[var(--color-ink-faint)] font-mono">{selectedPhone}</p>
                </div>
                <button
                  onClick={() => setStep('select')}
                  className="text-xs text-[var(--color-ink-faint)] hover:text-[var(--color-ink)]"
                >
                  Change
                </button>
              </div>

              {/* Template toggle */}
              <div className="flex items-center justify-between rounded-[var(--radius-md)] border border-[var(--color-border)] p-3">
                <div>
                  <p className="text-sm font-semibold text-[var(--color-ink)]">Use Template</p>
                  <p className="text-xs text-[var(--color-ink-faint)]">Required if 24h window is closed</p>
                </div>
                <button
                  onClick={() => setUseTemplate(!useTemplate)}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${useTemplate ? 'bg-emerald-500' : 'bg-[var(--color-border)]'}`}
                >
                  <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${useTemplate ? 'translate-x-6' : 'translate-x-1'}`} />
                </button>
              </div>

              {useTemplate ? (
                /* Template Selection */
                <div className="space-y-3">
                  <select
                    value={selectedTemplate}
                    onChange={(e) => {
                      setSelectedTemplate(e.target.value)
                      const tpl = templates.find((t: any) => t.id === e.target.value)
                      const vars = tpl?.variables || []
                      setTemplateVars(Object.fromEntries(vars.map((v: string) => [v, ''])))
                    }}
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2.5 text-sm focus:border-[var(--color-accent)] focus:outline-hidden"
                  >
                    <option value="">Select a template...</option>
                    {templates.map((t: any) => (
                      <option key={t.id} value={t.id}>{t.name} ({t.category})</option>
                    ))}
                  </select>

                  {selectedTemplate && (
                    <>
                      {/* Template variables */}
                      {Object.keys(templateVars).map((v) => (
                        <div key={v}>
                          <label className="block text-xs font-semibold text-[var(--color-ink)] mb-1">{v}</label>
                          <input
                            type="text"
                            value={templateVars[v] || ''}
                            onChange={(e) => setTemplateVars({ ...templateVars, [v]: e.target.value })}
                            className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-hidden"
                            placeholder={`Value for ${v}`}
                          />
                        </div>
                      ))}

                      {/* Template preview */}
                      {(() => {
                        const tpl = templates.find((t: any) => t.id === selectedTemplate)
                        let preview = tpl?.body || tpl?.bodyText || ''
                        Object.entries(templateVars).forEach(([k, val]) => {
                          preview = preview.replace(k, val || k)
                        })
                        return (
                          <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-3">
                            <p className="text-xs font-semibold text-[var(--color-ink)] mb-1">Preview</p>
                            <p className="text-xs text-[var(--color-ink-muted)]">{preview}</p>
                          </div>
                        )
                      })()}
                    </>
                  )}
                </div>
              ) : (
                /* Free text message */
                <div>
                  <label className="block text-xs font-semibold text-[var(--color-ink)] mb-1">Message</label>
                  <textarea
                    value={message}
                    onChange={(e) => setMessage(e.target.value)}
                    rows={4}
                    className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-hidden resize-none"
                    placeholder="Type your message..."
                    autoFocus
                  />
                  <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
                    {message.length}/1024 characters
                  </p>
                </div>
              )}

              {/* Actions */}
              <div className="flex items-center justify-between">
                <button
                  onClick={() => setStep('select')}
                  className="rounded-[var(--radius-md)] border border-[var(--color-border)] px-4 py-2 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)]"
                >
                  Back
                </button>
                <button
                  onClick={handleSend}
                  className="rounded-[var(--radius-md)] bg-emerald-600 px-6 py-2 text-xs font-semibold text-white hover:bg-emerald-700 flex items-center gap-2"
                >
                  <Send className="h-3.5 w-3.5" />
                  Send Message
                </button>
              </div>
            </div>
          )}

          {/* Step 3: Sending */}
          {step === 'sending' && (
            <div className="py-12 text-center">
              <Loader2 className="h-10 w-10 text-emerald-500 animate-spin mx-auto" />
              <p className="mt-4 text-sm font-semibold text-[var(--color-ink)]">Sending message...</p>
              <p className="text-xs text-[var(--color-ink-faint)]">Connecting to WhatsApp Cloud API</p>
            </div>
          )}

          {/* Step 4: Sent */}
          {step === 'sent' && (
            <div className="py-12 text-center">
              <CheckCircle2 className="h-12 w-12 text-emerald-500 mx-auto" />
              <p className="mt-4 text-sm font-semibold text-[var(--color-ink)]">Message sent successfully!</p>
              <p className="text-xs text-[var(--color-ink-faint)]">Conversation started with {selectedName || selectedPhone}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
