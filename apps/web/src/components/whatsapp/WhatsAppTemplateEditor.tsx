import { useState } from 'react'
import {
  X,
  Send,
  Loader2,
  CheckCircle2,
  Plus,
  Trash2,
  Eye,
  Globe,
  Copy,
} from '../../components/icons'
import type { WhatsAppTemplate } from '../../lib/api'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface TemplateVariable {
  id: string
  placeholder: string // e.g. "1"
  hintText: string
}

export interface TemplateButton {
  id: string
  type: 'quick_reply' | 'url' | 'phone_number'
  text: string
  url?: string
  phoneNumber?: string
}

export interface CreateTemplateData {
  name: string
  category: 'UTILITY' | 'MARKETING' | 'AUTHENTICATION'
  language: string
  bodyText: string
  headerText?: string
  footerText?: string
  headerType?: 'text' | 'image' | 'video' | 'document' | 'location'
  headerImageUrl?: string
  buttons: TemplateButton[]
  variables: TemplateVariable[]
}

interface Props {
  open: boolean
  onClose: () => void
  onCreate: (data: CreateTemplateData) => Promise<void>
  onDuplicate?: (template: WhatsAppTemplate) => void
  duplicateSource?: WhatsAppTemplate | null
}

// ---------------------------------------------------------------------------
// Language list (common WhatsApp languages)
// ---------------------------------------------------------------------------

const LANGUAGES = [
  { code: 'en', label: 'English' },
  { code: 'es', label: 'Spanish' },
  { code: 'fr', label: 'French' },
  { code: 'de', label: 'German' },
  { code: 'pt', label: 'Portuguese' },
  { code: 'id', label: 'Indonesian' },
  { code: 'ar', label: 'Arabic' },
  { code: 'zh', label: 'Chinese' },
  { code: 'ja', label: 'Japanese' },
  { code: 'ko', label: 'Korean' },
  { code: 'hi', label: 'Hindi' },
  { code: 'ru', label: 'Russian' },
  { code: 'tr', label: 'Turkish' },
  { code: 'nl', label: 'Dutch' },
  { code: 'it', label: 'Italian' },
]

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function WhatsAppTemplateEditor({ open, onClose, onCreate, onDuplicate, duplicateSource }: Props) {
  const [creating, setCreating] = useState(false)
  const [created, setCreated] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showPreview, setShowPreview] = useState(false)

  // Form state
  const [name, setName] = useState(duplicateSource?.name || '')
  const [category, setCategory] = useState<'UTILITY' | 'MARKETING' | 'AUTHENTICATION'>(
    (duplicateSource?.category?.toUpperCase() as any) || 'UTILITY'
  )
  const [language, setLanguage] = useState(duplicateSource?.language || 'en')
  const [bodyText, setBodyText] = useState(duplicateSource?.body || duplicateSource?.bodyText || '')
  const [headerText, setHeaderText] = useState(duplicateSource?.headerText || '')
  const [footerText, setFooterText] = useState(duplicateSource?.footerText || '')
  const [headerType, setHeaderType] = useState<'text' | 'image' | 'video' | 'document' | 'location'>('text')
  const [headerImageUrl, setHeaderImageUrl] = useState('')

  const [buttons, setButtons] = useState<TemplateButton[]>(
    duplicateSource?.buttons?.map((b: any) => ({
      id: b.id || `btn_${Math.random()}`,
      type: b.type || 'quick_reply',
      text: b.text || b.title || '',
      url: b.url,
      phoneNumber: b.phoneNumber,
    })) || []
  )

  const [variables, setVariables] = useState<TemplateVariable[]>(
    duplicateSource?.variables?.map((v: string, i: number) => ({
      id: `var_${i}`,
      placeholder: v.replace(/[{}]/g, ''),
      hintText: `Variable ${v.replace(/[{}]/g, '')}`,
    })) || []
  )

  function resetState() {
    setName('')
    setCategory('UTILITY')
    setLanguage('en')
    setBodyText('')
    setHeaderText('')
    setFooterText('')
    setHeaderType('text')
    setHeaderImageUrl('')
    setButtons([])
    setVariables([])
    setCreated(false)
    setError(null)
    setShowPreview(false)
  }

  function handleOpen() {
    if (duplicateSource) {
      setName(`${duplicateSource.name} (copy)`)
      setBodyText(duplicateSource.body || duplicateSource.bodyText || '')
      setHeaderText(duplicateSource.headerText || '')
      setFooterText(duplicateSource.footerText || '')
      setLanguage(duplicateSource.language || 'en')
      setCategory((duplicateSource.category?.toUpperCase() as any) || 'UTILITY')
    } else {
      resetState()
    }
    setError(null)
    setCreated(false)
  }

  async function handleCreate() {
    if (!name.trim()) {
      setError('Template name is required')
      return
    }
    if (!bodyText.trim()) {
      setError('Body text is required')
      return
    }

    setCreating(true)
    setError(null)
    try {
      const data: CreateTemplateData = {
        name,
        category,
        language,
        bodyText,
        headerText: headerText || undefined,
        footerText: footerText || undefined,
        headerType,
        headerImageUrl: headerImageUrl || undefined,
        buttons,
        variables,
      }
      await onCreate(data)
      setCreated(true)
      setTimeout(() => {
        onClose()
        resetState()
      }, 1500)
    } catch (e: any) {
      setError(e.message || 'Failed to create template')
    } finally {
      setCreating(false)
    }
  }

  // Variable CRUD
  function addVariable() {
    const nextNum = variables.length + 1
    setVariables([...variables, { id: `var_${nextNum}`, placeholder: `${nextNum}`, hintText: `Variable ${nextNum}` }])
    // Insert {{N}} into body text at cursor or end
    setBodyText((prev) => prev + (prev ? ' ' : '') + `{{${nextNum}}}`)
  }

  function removeVariable(index: number) {
    const varToRemove = variables[index]
    setVariables(variables.filter((_, i) => i !== index))
    // Remove from body text
    setBodyText((prev) => prev.replace(new RegExp(`\\{\\{${varToRemove.placeholder}\\}\\}`, 'g'), '').trim())
  }

  function updateVariable(index: number, field: 'placeholder' | 'hintText', value: string) {
    const updated = [...variables]
    updated[index] = { ...updated[index], [field]: value }
    setVariables(updated)
  }

  // Button CRUD
  function addButton() {
    if (buttons.length >= 3) return
    setButtons([...buttons, { id: `btn_${buttons.length + 1}`, type: 'quick_reply', text: '' }])
  }

  function removeButton(index: number) {
    setButtons(buttons.filter((_, i) => i !== index))
  }

  function updateButton(index: number, field: keyof TemplateButton, value: string) {
    const updated = [...buttons]
    updated[index] = { ...updated[index], [field]: value }
    setButtons(updated)
  }

  // Preview
  function PreviewCard() {
    return (
      <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] p-4 space-y-3">
        {/* Header */}
        {headerType === 'text' && headerText && (
          <div className="text-sm font-semibold text-[var(--color-ink)] border-b border-[var(--color-border)] pb-2">
            {headerText}
          </div>
        )}
        {headerType === 'image' && (
          <div className="rounded-[var(--radius-sm)] bg-[var(--color-surface)] border border-[var(--color-border)] h-24 flex items-center justify-center text-xs text-[var(--color-ink-faint)]">
            🖼️ {headerImageUrl || 'Image Header'}
          </div>
        )}

        {/* Body */}
        <p className="text-sm text-[var(--color-ink)] whitespace-pre-wrap">
          {bodyText || '—'}
        </p>

        {/* Buttons */}
        {buttons.length > 0 && (
          <div className="space-y-2 pt-2">
            {buttons.map((btn) => (
              <div
                key={btn.id}
                className="rounded-[var(--radius-md)] border border-[var(--color-accent)] bg-[var(--color-accent-muted)] px-4 py-2 text-sm font-medium text-[var(--color-accent)] text-center"
              >
                {btn.text || 'Button'}
                {btn.type === 'url' && ` → ${btn.url}`}
              </div>
            ))}
          </div>
        )}

        {/* Footer */}
        {footerText && (
          <div className="text-xs text-[var(--color-ink-faint)] border-t border-[var(--color-border)] pt-2 italic">
            {footerText}
          </div>
        )}
      </div>
    )
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="w-full max-w-2xl max-h-[90vh] rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl flex flex-col mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
          <div>
            <h2 className="text-sm font-semibold text-[var(--color-ink)]">
              {duplicateSource ? 'Duplicate Template' : 'Create WhatsApp Template'}
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
              Templates require Meta approval before use
            </p>
          </div>
          <button
            type="button"
            onClick={() => { onClose(); resetState() }}
            className="rounded-[var(--radius-md)] p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-page)] hover:text-[var(--color-ink)] transition-colors cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Error */}
        {error && (
          <div className="mx-4 mt-3 rounded-[var(--radius-sm)] border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700">
            {error}
          </div>
        )}

        {/* Success */}
        {created && (
          <div className="mx-4 mt-3 rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-700 flex items-center gap-2">
            <CheckCircle2 className="h-3.5 w-3.5" />
            Template created successfully
          </div>
        )}

        {/* Scrollable Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* Name */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Template Name <span className="text-rose-500">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. order_confirmation"
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
            />
            <p className="text-[10px] text-[var(--color-ink-faint)] mt-1">
              Use lowercase letters, numbers, and underscores only
            </p>
          </div>

          {/* Category & Language */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                Category <span className="text-rose-500">*</span>
              </label>
              <select
                value={category}
                onChange={(e) => setCategory(e.target.value as any)}
                className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
              >
                <option value="UTILITY">Utility</option>
                <option value="MARKETING">Marketing</option>
                <option value="AUTHENTICATION">Authentication</option>
              </select>
            </div>
            <div>
              <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                Language <span className="text-rose-500">*</span>
              </label>
              <div className="relative">
                <select
                  value={language}
                  onChange={(e) => setLanguage(e.target.value)}
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none appearance-none"
                >
                  {LANGUAGES.map((lang) => (
                    <option key={lang.code} value={lang.code}>{lang.label}</option>
                  ))}
                </select>
                <Globe className="absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--color-ink-faint)] pointer-events-none" />
              </div>
            </div>
          </div>

          {/* Header Type */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Header Type <span className="text-[var(--color-ink-faint)] font-normal">(optional)</span>
            </label>
            <div className="flex gap-2">
              {(['text', 'image', 'video', 'document', 'location'] as const).map((type) => (
                <button
                  key={type}
                  type="button"
                  onClick={() => setHeaderType(type)}
                  className={`px-3 py-1.5 rounded-[var(--radius-md)] text-[10px] font-semibold capitalize transition-colors cursor-pointer ${
                    headerType === type
                      ? 'bg-[var(--color-accent)] text-white'
                      : 'bg-[var(--color-page)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]'
                  }`}
                >
                  {type}
                </button>
              ))}
            </div>
            {headerType === 'text' && (
              <input
                type="text"
                value={headerText}
                onChange={(e) => setHeaderText(e.target.value.slice(0, 60))}
                placeholder="Header text (max 60 chars)"
                className="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
              />
            )}
            {headerType === 'image' && (
              <input
                type="text"
                value={headerImageUrl}
                onChange={(e) => setHeaderImageUrl(e.target.value)}
                placeholder="Image URL (https://...)"
                className="mt-2 w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
              />
            )}
          </div>

          {/* Body Text */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Body Text <span className="text-rose-500">*</span> <span className="text-[var(--color-ink-faint)] font-normal">(max 1024 chars)</span>
            </label>
            <textarea
              rows={4}
              value={bodyText}
              onChange={(e) => setBodyText(e.target.value.slice(0, 1024))}
              placeholder="Hello {{1}}, your order {{2}} has been confirmed."
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none resize-none"
            />
            <p className="text-[10px] text-[var(--color-ink-faint)] mt-1">{bodyText.length}/1024</p>
          </div>

          {/* Variables */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                Variables <span className="text-[var(--color-ink-faint)] font-normal">({variables.length})</span>
              </label>
              <button
                type="button"
                onClick={addVariable}
                disabled={variables.length >= 10}
                className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2 py-1 text-[10px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] disabled:opacity-50 cursor-pointer flex items-center gap-1"
              >
                <Plus className="h-3 w-3" /> Add Variable
              </button>
            </div>
            {variables.map((v, i) => (
              <div key={v.id} className="flex items-start gap-2">
                <div className="flex-1 grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-[10px] text-[var(--color-ink-faint)] block mb-0.5">Placeholder</label>
                    <div className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs font-mono text-[var(--color-accent)]">
                      {`{{${v.placeholder}}}`}
                    </div>
                  </div>
                  <div>
                    <label className="text-[10px] text-[var(--color-ink-faint)] block mb-0.5">Example Value</label>
                    <input
                      type="text"
                      value={v.hintText}
                      onChange={(e) => updateVariable(i, 'hintText', e.target.value)}
                      placeholder="e.g. John Doe"
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => removeVariable(i)}
                  className="p-1.5 text-[var(--color-ink-faint)] hover:text-red-600 hover:bg-red-50 rounded transition-colors cursor-pointer mt-3"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
            ))}
            {variables.length === 0 && (
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Add variables to use dynamic content like <code className="text-[var(--color-accent)]">{'{{1}}'}</code> in your body text
              </p>
            )}
          </div>

          {/* Buttons */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                Buttons <span className="text-[var(--color-ink-faint)] font-normal">({buttons.length}/3)</span>
              </label>
              <button
                type="button"
                onClick={addButton}
                disabled={buttons.length >= 3}
                className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2 py-1 text-[10px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] disabled:opacity-50 cursor-pointer flex items-center gap-1"
              >
                <Plus className="h-3 w-3" /> Add Button
              </button>
            </div>
            {buttons.map((btn, i) => (
              <div key={btn.id} className="flex items-start gap-2">
                <div className="flex-1 space-y-1.5">
                  <div className="flex gap-2">
                    <select
                      value={btn.type}
                      onChange={(e) => updateButton(i, 'type', e.target.value as any)}
                      className="flex-1 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                    >
                      <option value="quick_reply">Quick Reply</option>
                      <option value="url">URL</option>
                      <option value="phone_number">Phone Number</option>
                    </select>
                    <input
                      type="text"
                      value={btn.text}
                      onChange={(e) => updateButton(i, 'text', e.target.value.slice(0, 20))}
                      placeholder="Button text (max 20)"
                      className="flex-1 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  </div>
                  {btn.type === 'url' && (
                    <input
                      type="text"
                      value={btn.url || ''}
                      onChange={(e) => updateButton(i, 'url', e.target.value)}
                      placeholder="https://example.com"
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  )}
                  {btn.type === 'phone_number' && (
                    <input
                      type="text"
                      value={btn.phoneNumber || ''}
                      onChange={(e) => updateButton(i, 'phoneNumber', e.target.value)}
                      placeholder="+1234567890"
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  )}
                </div>
                <button
                  type="button"
                  onClick={() => removeButton(i)}
                  className="p-1.5 text-[var(--color-ink-faint)] hover:text-red-600 hover:bg-red-50 rounded transition-colors cursor-pointer mt-1"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
            ))}
            {buttons.length === 0 && (
              <p className="text-[10px] text-[var(--color-ink-faint)]">
                Add up to 3 reply buttons for interactive templates
              </p>
            )}
          </div>

          {/* Footer */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Footer Text <span className="text-[var(--color-ink-faint)] font-normal">(optional, max 60 chars)</span>
            </label>
            <input
              type="text"
              value={footerText}
              onChange={(e) => setFooterText(e.target.value.slice(0, 60))}
              placeholder="Optional footer text"
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
            />
          </div>

          {/* Preview Toggle */}
          <div className="flex items-center justify-between pt-2">
            <button
              type="button"
              onClick={() => setShowPreview(!showPreview)}
              className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-1.5 text-[10px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] cursor-pointer flex items-center gap-1.5"
            >
              <Eye className="h-3 w-3" />
              {showPreview ? 'Hide Preview' : 'Show Preview'}
            </button>
          </div>

          {showPreview && <PreviewCard />}
        </div>

        {/* Footer Actions */}
        <div className="flex items-center justify-end gap-2 p-4 border-t border-[var(--color-border)]">
          <button
            type="button"
            onClick={() => { onClose(); resetState() }}
            className="px-4 py-2 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-page)] rounded-[var(--radius-md)] transition-colors cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleCreate}
            disabled={creating || created}
            className="px-4 py-2 text-xs font-semibold bg-[var(--color-accent)] text-white rounded-[var(--radius-md)] hover:bg-[var(--color-accent)]/90 disabled:opacity-50 flex items-center gap-1.5 cursor-pointer"
          >
            {creating ? (
              <>
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                Creating...
              </>
            ) : created ? (
              <>
                <CheckCircle2 className="h-3.5 w-3.5" />
                Created
              </>
            ) : (
              <>
                <Send className="h-3.5 w-3.5" />
                Submit for Approval
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
