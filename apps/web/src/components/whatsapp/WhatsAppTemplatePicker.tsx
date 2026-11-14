import { useEffect, useState } from 'react'
import { X, Send, Loader2, CheckCircle2 } from '../../components/icons'
import { api, type WhatsAppTemplate } from '../../lib/api'

interface Props {
  open: boolean
  onClose: () => void
  contactId: string
  contactPhone?: string
  conversationId: string
  onSent: () => void
}

export function WhatsAppTemplatePicker({ open, onClose, contactId, contactPhone, conversationId, onSent }: Props) {
  const [templates, setTemplates] = useState<WhatsAppTemplate[]>([])
  const [loading, setLoading] = useState(true)
  const [sending, setSending] = useState<string | null>(null)
  const [sentTemplate, setSentTemplate] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [categoryFilter, setCategoryFilter] = useState('all')

  useEffect(() => {
    if (!open) return
    loadTemplates()
  }, [open])

  async function loadTemplates() {
    setLoading(true)
    setError(null)
    try {
      const brandId = api.getBrandId()
      const data = await api.listWhatsAppTemplates(brandId || undefined)
      // Filter to approved templates only
      const approved = (data as WhatsAppTemplate[]).filter(
        (t) => t.status === 'APPROVED' || t.status === 'approved'
      )
      setTemplates(approved)
    } catch (e: any) {
      setError(e.message || 'Failed to load templates')
    } finally {
      setLoading(false)
    }
  }

  async function handleSend(template: WhatsAppTemplate) {
    setSending(template.id)
    setError(null)
    try {
      const brandId = api.getBrandId()
      if (!brandId) throw new Error('No brand selected')
      await api.sendWhatsAppTemplate(brandId, {
        contactId,
        templateName: template.name,
        language: template.language || 'en_US',
      })
      setSentTemplate(template.name)
      setTimeout(() => {
        onSent()
        onClose()
      }, 1500)
    } catch (e: any) {
      setError(e.message || 'Failed to send template')
    } finally {
      setSending(null)
    }
  }

  const categories = ['all', ...Array.from(new Set(templates.map((t) => t.category || 'other')))]
  const filtered = templates.filter((t) => {
    const matchesCategory = categoryFilter === 'all' || (t.category || 'other') === categoryFilter
    const matchesSearch = !search || t.name.toLowerCase().includes(search.toLowerCase()) || (t.bodyText || '').toLowerCase().includes(search.toLowerCase())
    return matchesCategory && matchesSearch
  })

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="w-full max-w-lg max-h-[80vh] rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl flex flex-col mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
          <div>
            <h2 className="text-sm font-semibold text-[var(--color-ink)]">Send WhatsApp Template</h2>
            <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
              Contact: {contactPhone || contactId}
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-[var(--radius-md)] p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-page)] hover:text-[var(--color-ink)] transition-colors cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Search & Filter */}
        <div className="p-3 border-b border-[var(--color-border)] space-y-2">
          <input
            type="text"
            placeholder="Search templates..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
          />
          <div className="flex gap-1.5 overflow-x-auto pb-1">
            {categories.map((cat) => (
              <button
                key={cat}
                type="button"
                onClick={() => setCategoryFilter(cat)}
                className={`px-2.5 py-1 rounded-full text-[11px] font-semibold capitalize whitespace-nowrap transition-colors cursor-pointer ${
                  categoryFilter === cat
                    ? 'bg-[var(--color-accent)] text-white'
                    : 'bg-[var(--color-page)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="mx-3 mt-3 rounded-[var(--radius-sm)] border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700">
            {error}
          </div>
        )}

        {/* Template List */}
        <div className="flex-1 overflow-y-auto p-3 space-y-2">
          {loading ? (
            <div className="flex items-center justify-center py-8 text-xs text-[var(--color-ink-muted)]">
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
              Loading templates...
            </div>
          ) : filtered.length === 0 ? (
            <div className="text-center py-8 text-xs text-[var(--color-ink-muted)]">
              No approved templates found.{' '}
              <button
                type="button"
                onClick={loadTemplates}
                className="text-[var(--color-accent)] underline cursor-pointer"
              >
                Refresh
              </button>
            </div>
          ) : (
            filtered.map((t) => {
              const isSending = sending === t.id
              const isSent = sentTemplate === t.name
              return (
                <div
                  key={t.id}
                  className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] p-3 transition-colors"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <p className="text-xs font-semibold text-[var(--color-ink)] truncate">{t.name}</p>
                        <span className="text-[10px] font-medium text-[var(--color-ink-faint)] bg-[var(--color-accent-muted)] px-1.5 py-0.25 rounded capitalize">
                          {t.category}
                        </span>
                        <span className="text-[10px] font-medium text-[var(--color-ink-faint)]">
                          {t.language}
                        </span>
                      </div>
                      <p className="text-xs text-[var(--color-ink-muted)] mt-1 line-clamp-2">
                        {t.bodyText}
                      </p>
                    </div>
                    <button
                      type="button"
                      onClick={() => handleSend(t)}
                      disabled={isSending || isSent}
                      className={`shrink-0 rounded-[var(--radius-md)] p-2 transition-colors cursor-pointer ${
                        isSent
                          ? 'bg-emerald-100 text-emerald-600'
                          : isSending
                          ? 'bg-[var(--color-accent-muted)] text-[var(--color-accent)]'
                          : 'bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)]'
                      }`}
                    >
                      {isSent ? (
                        <CheckCircle2 className="h-4 w-4" />
                      ) : isSending ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <Send className="h-4 w-4" />
                      )}
                    </button>
                  </div>
                </div>
              )
            })
          )}
        </div>

        {/* Footer */}
        <div className="border-t border-[var(--color-border)] px-4 py-3 bg-[var(--color-page)]/50 rounded-b-xl">
          <p className="text-[11px] text-[var(--color-ink-faint)]">
            Templates are sent via WhatsApp Business API. The customer service window will reopen when the contact replies.
          </p>
        </div>
      </div>
    </div>
  )
}
