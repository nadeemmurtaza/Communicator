import { useState } from 'react'
import {
  X,
  Send,
  Loader2,
  CheckCircle2,
  Plus,
  Trash2,
  Eye,
} from '../../components/icons'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type InteractiveType = 'buttons' | 'list' | 'location' | 'media' | 'cta_url'

export interface InteractiveButton {
  id: string
  title: string
}

export interface ListRow {
  id: string
  title: string
  description: string
}

export interface ListSection {
  title: string
  rows: ListRow[]
}

export interface InteractiveMessageData {
  type: InteractiveType
  bodyText: string
  headerText?: string
  footerText?: string
  // Buttons
  buttons?: InteractiveButton[]
  // List
  listButtonTitle?: string
  listSections?: ListSection[]
  // Location request
  // (uses bodyText)
  // Media
  mediaUrl?: string
  mediaType?: 'image' | 'video' | 'audio' | 'document'
  mediaFilename?: string
  // CTA URL
  ctaDisplayText?: string
  ctaUrl?: string
}

interface Props {
  open: boolean
  onClose: () => void
  onSend: (data: InteractiveMessageData) => Promise<void>
}

// ---------------------------------------------------------------------------
// Default values
// ---------------------------------------------------------------------------

const defaultButtons: InteractiveButton[] = [
  { id: 'btn_1', title: 'Yes' },
  { id: 'btn_2', title: 'No' },
]

const defaultListSections: ListSection[] = [
  {
    title: 'Options',
    rows: [
      { id: 'row_1', title: 'Option 1', description: 'Description for option 1' },
      { id: 'row_2', title: 'Option 2', description: 'Description for option 2' },
    ],
  },
]

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function WhatsAppInteractiveBuilder({ open, onClose, onSend }: Props) {
  const [activeType, setActiveType] = useState<InteractiveType>('buttons')
  const [sending, setSending] = useState(false)
  const [sent, setSent] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showPreview, setShowPreview] = useState(false)

  // Body text (shared across all types)
  const [bodyText, setBodyText] = useState('')
  const [headerText, setHeaderText] = useState('')
  const [footerText, setFooterText] = useState('')

  // Buttons state
  const [buttons, setButtons] = useState<InteractiveButton[]>(defaultButtons)

  // List state
  const [listButtonTitle, setListButtonTitle] = useState('View Options')
  const [listSections, setListSections] = useState<ListSection[]>(defaultListSections)

  // Media state
  const [mediaUrl, setMediaUrl] = useState('')
  const [mediaType, setMediaType] = useState<'image' | 'video' | 'audio' | 'document'>('image')
  const [mediaFilename, setMediaFilename] = useState('')

  // CTA URL state
  const [ctaDisplayText, setCtaDisplayText] = useState('Visit Website')
  const [ctaUrl, setCtaUrl] = useState('')

  function resetState() {
    setBodyText('')
    setHeaderText('')
    setFooterText('')
    setButtons(defaultButtons)
    setListButtonTitle('View Options')
    setListSections(defaultListSections)
    setMediaUrl('')
    setMediaType('image')
    setMediaFilename('')
    setCtaDisplayText('Visit Website')
    setCtaUrl('')
    setSent(false)
    setError(null)
    setShowPreview(false)
  }

  function handleTypeChange(type: InteractiveType) {
    setActiveType(type)
    resetState()
  }

  function buildData(): InteractiveMessageData {
    const base: InteractiveMessageData = {
      type: activeType,
      bodyText,
      headerText: headerText || undefined,
      footerText: footerText || undefined,
    }

    switch (activeType) {
      case 'buttons':
        return { ...base, buttons }
      case 'list':
        return { ...base, listButtonTitle, listSections }
      case 'media':
        return { ...base, mediaUrl, mediaType, mediaFilename: mediaFilename || undefined }
      case 'cta_url':
        return { ...base, ctaDisplayText, ctaUrl }
      default:
        return base
    }
  }

  async function handleSend() {
    if (!bodyText.trim()) {
      setError('Body text is required')
      return
    }

    if (activeType === 'buttons' && buttons.length < 1) {
      setError('At least 1 button is required')
      return
    }

    if (activeType === 'list' && listSections.flat().length < 1) {
      setError('At least 1 list row is required')
      return
    }

    if (activeType === 'media' && !mediaUrl.trim()) {
      setError('Media URL is required')
      return
    }

    if (activeType === 'cta_url' && !ctaUrl.trim()) {
      setError('URL is required')
      return
    }

    setSending(true)
    setError(null)
    try {
      await onSend(buildData())
      setSent(true)
      setTimeout(() => {
        onClose()
      }, 1500)
    } catch (e: any) {
      setError(e.message || 'Failed to send interactive message')
    } finally {
      setSending(false)
    }
  }

  // ---------------------------------------------------------------------------
  // Button CRUD
  // ---------------------------------------------------------------------------

  function addButton() {
    if (buttons.length >= 3) return
    setButtons([...buttons, { id: `btn_${buttons.length + 1}`, title: `Button ${buttons.length + 1}` }])
  }

  function removeButton(index: number) {
    if (buttons.length <= 1) return
    setButtons(buttons.filter((_, i) => i !== index))
  }

  function updateButton(index: number, field: 'id' | 'title', value: string) {
    const updated = [...buttons]
    updated[index] = { ...updated[index], [field]: value }
    setButtons(updated)
  }

  // ---------------------------------------------------------------------------
  // List CRUD
  // ---------------------------------------------------------------------------

  function addSection() {
    setListSections([...listSections, { title: 'New Section', rows: [] }])
  }

  function removeSection(index: number) {
    setListSections(listSections.filter((_, i) => i !== index))
  }

  function updateSectionTitle(index: number, title: string) {
    const updated = [...listSections]
    updated[index] = { ...updated[index], title }
    setListSections(updated)
  }

  function addRow(sectionIndex: number) {
    const totalRows = listSections.reduce((sum, s) => sum + s.rows.length, 0)
    if (totalRows >= 10) return
    const updated = [...listSections]
    const section = { ...updated[sectionIndex] }
    section.rows = [...section.rows, { id: `row_${totalRows + 1}`, title: `Row ${totalRows + 1}`, description: '' }]
    updated[sectionIndex] = section
    setListSections(updated)
  }

  function removeRow(sectionIndex: number, rowIndex: number) {
    const updated = [...listSections]
    const section = { ...updated[sectionIndex] }
    section.rows = section.rows.filter((_, i) => i !== rowIndex)
    updated[sectionIndex] = section
    setListSections(updated)
  }

  function updateRow(sectionIndex: number, rowIndex: number, field: 'id' | 'title' | 'description', value: string) {
    const updated = [...listSections]
    const section = { ...updated[sectionIndex] }
    const rows = [...section.rows]
    rows[rowIndex] = { ...rows[rowIndex], [field]: value }
    section.rows = rows
    updated[sectionIndex] = section
    setListSections(updated)
  }

  const totalRows = listSections.reduce((sum, s) => sum + s.rows.length, 0)

  // ---------------------------------------------------------------------------
  // Preview Component
  // ---------------------------------------------------------------------------

  function PreviewCard() {
    const data = buildData()
    return (
      <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] p-4 space-y-3">
        {/* Header */}
        {data.headerText && (
          <div className="text-sm font-semibold text-[var(--color-ink)] border-b border-[var(--color-border)] pb-2">
            {data.headerText}
          </div>
        )}

        {/* Body */}
        <p className="text-sm text-[var(--color-ink)] whitespace-pre-wrap">{data.bodyText || '—'}</p>

        {/* Buttons */}
        {data.type === 'buttons' && data.buttons && data.buttons.length > 0 && (
          <div className="space-y-2 pt-2">
            {data.buttons.map((btn) => (
              <div
                key={btn.id}
                className="rounded-[var(--radius-md)] border border-[var(--color-accent)] bg-[var(--color-accent-muted)] px-4 py-2 text-sm font-medium text-[var(--color-accent)] text-center"
              >
                {btn.title}
              </div>
            ))}
          </div>
        )}

        {/* List */}
        {data.type === 'list' && data.listSections && (
          <div className="space-y-2 pt-2">
            {data.listSections.map((section, si) => (
              <div key={si} className="space-y-1">
                <p className="text-xs font-semibold text-[var(--color-ink)] uppercase">{section.title}</p>
                {section.rows.map((row, ri) => (
                  <div key={ri} className="rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2">
                    <p className="text-sm font-medium text-[var(--color-ink)]">{row.title}</p>
                    {row.description && (
                      <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">{row.description}</p>
                    )}
                  </div>
                ))}
              </div>
            ))}
            <div className="rounded-[var(--radius-md)] border border-[var(--color-accent)] bg-[var(--color-accent-muted)] px-4 py-2 text-sm font-medium text-[var(--color-accent)] text-center">
              {data.listButtonTitle}
            </div>
          </div>
        )}

        {/* Location Request */}
        {data.type === 'location' && (
          <div className="rounded-[var(--radius-md)] border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-700 flex items-center gap-2">
            <span className="text-lg">📍</span>
            <span>Tap to share your location</span>
          </div>
        )}

        {/* Media */}
        {data.type === 'media' && (
          <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-3">
            <div className="rounded-[var(--radius-sm)] bg-[var(--color-page)] border border-[var(--color-border)] h-32 flex items-center justify-center text-xs text-[var(--color-ink-faint)]">
              {data.mediaType === 'image' && '🖼️ Image Preview'}
              {data.mediaType === 'video' && '🎬 Video Preview'}
              {data.mediaType === 'audio' && '🎵 Audio Preview'}
              {data.mediaType === 'document' && '📄 Document Preview'}
            </div>
            {data.mediaFilename && (
              <p className="text-xs text-[var(--color-ink-muted)] mt-2">{data.mediaFilename}</p>
            )}
          </div>
        )}

        {/* CTA URL */}
        {data.type === 'cta_url' && (
          <div className="rounded-[var(--radius-md)] border border-[var(--color-accent)] bg-[var(--color-accent-muted)] px-4 py-2 text-sm font-medium text-[var(--color-accent)] text-center">
            {data.ctaDisplayText} → {data.ctaUrl}
          </div>
        )}

        {/* Footer */}
        {data.footerText && (
          <div className="text-xs text-[var(--color-ink-faint)] border-t border-[var(--color-border)] pt-2 italic">
            {data.footerText}
          </div>
        )}
      </div>
    )
  }

  if (!open) return null

  const typeTabs: { key: InteractiveType; label: string; icon: string; hint: string }[] = [
    { key: 'buttons', label: 'Buttons', icon: '🔘', hint: 'Up to 3 reply buttons' },
    { key: 'list', label: 'List', icon: '📋', hint: 'Up to 10 menu items' },
    { key: 'location', label: 'Location', icon: '📍', hint: 'Request customer location' },
    { key: 'media', label: 'Media', icon: '🖼️', hint: 'Image, video, audio, doc' },
    { key: 'cta_url', label: 'CTA URL', icon: '🔗', hint: 'Link button to website' },
  ]

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="w-full max-w-2xl max-h-[90vh] rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl flex flex-col mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
          <div>
            <h2 className="text-sm font-semibold text-[var(--color-ink)]">
              Compose Interactive WhatsApp Message
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
              Only available within the 24h customer service window
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

        {/* Type Tabs */}
        <div className="border-b border-[var(--color-border)] px-4 pt-3">
          <div className="flex gap-1 overflow-x-auto pb-2">
            {typeTabs.map((tab) => (
              <button
                key={tab.key}
                type="button"
                onClick={() => handleTypeChange(tab.key)}
                title={tab.hint}
                className={`px-3 py-1.5 rounded-[var(--radius-md)] text-xs font-semibold whitespace-nowrap transition-colors cursor-pointer flex items-center gap-1.5 ${
                  activeType === tab.key
                    ? 'bg-[var(--color-accent)] text-white'
                    : 'bg-[var(--color-page)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]'
                }`}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="mx-4 mt-3 rounded-[var(--radius-sm)] border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700">
            {error}
          </div>
        )}

        {/* Scrollable Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* Header Text (optional, for types that support it) */}
          {activeType !== 'location' && (
            <>
              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  Header Text <span className="text-[var(--color-ink-faint)] font-normal">(optional, max 60 chars)</span>
                </label>
                <input
                  type="text"
                  value={headerText}
                  onChange={(e) => setHeaderText(e.target.value.slice(0, 60))}
                  placeholder="Optional header text"
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                />
              </div>
            </>
          )}

          {/* Body Text (required) */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Body Text <span className="text-rose-500">*</span> <span className="text-[var(--color-ink-faint)] font-normal">(max 1024 chars)</span>
            </label>
            <textarea
              rows={3}
              value={bodyText}
              onChange={(e) => setBodyText(e.target.value.slice(0, 1024))}
              placeholder={
                activeType === 'buttons'
                  ? 'Ask a question the buttons will answer...'
                  : activeType === 'list'
                  ? 'Describe what the list contains...'
                  : activeType === 'location'
                  ? 'Please share your location so we can...'
                  : activeType === 'media'
                  ? 'Caption for the media...'
                  : 'Message before the CTA button...'
              }
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none resize-none"
            />
            <p className="text-[10px] text-[var(--color-ink-faint)] mt-1">{bodyText.length}/1024</p>
          </div>

          {/* Type-Specific Fields */}

          {/* Buttons Builder */}
          {activeType === 'buttons' && (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                  Reply Buttons <span className="text-[var(--color-ink-faint)] font-normal">({buttons.length}/3)</span>
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
                <div key={i} className="flex items-start gap-2">
                  <div className="flex-1 space-y-1.5">
                    <input
                      type="text"
                      value={btn.title}
                      onChange={(e) => updateButton(i, 'title', e.target.value.slice(0, 20))}
                      placeholder="Button text (max 20 chars)"
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                    <input
                      type="text"
                      value={btn.id}
                      onChange={(e) => updateButton(i, 'id', e.target.value.slice(0, 256))}
                      placeholder="Button ID (received when tapped)"
                      className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                  </div>
                  <button
                    type="button"
                    onClick={() => removeButton(i)}
                    disabled={buttons.length <= 1}
                    className="rounded-[var(--radius-sm)] p-1.5 text-rose-400 hover:bg-rose-50 hover:text-rose-600 disabled:opacity-30 cursor-pointer"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* List Builder */}
          {activeType === 'list' && (
            <div className="space-y-3">
              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  List Button Text <span className="text-[var(--color-ink-faint)] font-normal">(max 20 chars)</span>
                </label>
                <input
                  type="text"
                  value={listButtonTitle}
                  onChange={(e) => setListButtonTitle(e.target.value.slice(0, 20))}
                  placeholder="View Options"
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                />
              </div>

              <div className="flex items-center justify-between">
                <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                  Sections & Rows <span className="text-[var(--color-ink-faint)] font-normal">({totalRows}/10 rows)</span>
                </label>
                <button
                  type="button"
                  onClick={addSection}
                  className="rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-page)] px-2 py-1 text-[10px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] cursor-pointer flex items-center gap-1"
                >
                  <Plus className="h-3 w-3" /> Add Section
                </button>
              </div>

              {listSections.map((section, si) => (
                <div key={si} className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] p-3 space-y-2">
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={section.title}
                      onChange={(e) => updateSectionTitle(si, e.target.value.slice(0, 24))}
                      placeholder="Section title"
                      className="flex-1 rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs font-semibold text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                    />
                    <button
                      type="button"
                      onClick={() => removeSection(si)}
                      className="rounded-[var(--radius-sm)] p-1.5 text-rose-400 hover:bg-rose-50 hover:text-rose-600 cursor-pointer"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  </div>

                  {section.rows.map((row, ri) => (
                    <div key={ri} className="flex items-start gap-2 ml-2">
                      <div className="flex-1 space-y-1.5">
                        <input
                          type="text"
                          value={row.title}
                          onChange={(e) => updateRow(si, ri, 'title', e.target.value.slice(0, 24))}
                          placeholder="Row title (max 24 chars)"
                          className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                        />
                        <input
                          type="text"
                          value={row.description}
                          onChange={(e) => updateRow(si, ri, 'description', e.target.value.slice(0, 72))}
                          placeholder="Description (optional, max 72 chars)"
                          className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                        />
                        <input
                          type="text"
                          value={row.id}
                          onChange={(e) => updateRow(si, ri, 'id', e.target.value.slice(0, 200))}
                          placeholder="Row ID (received when tapped)"
                          className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                        />
                      </div>
                      <button
                        type="button"
                        onClick={() => removeRow(si, ri)}
                        className="rounded-[var(--radius-sm)] p-1.5 text-rose-400 hover:bg-rose-50 hover:text-rose-600 cursor-pointer"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  ))}

                  <button
                    type="button"
                    onClick={() => addRow(si)}
                    disabled={totalRows >= 10}
                    className="rounded-[var(--radius-sm)] border border-dashed border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-[10px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] disabled:opacity-50 cursor-pointer flex items-center gap-1"
                  >
                    <Plus className="h-3 w-3" /> Add Row
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Media Builder */}
          {activeType === 'media' && (
            <div className="space-y-3">
              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  Media Type
                </label>
                <div className="flex gap-2">
                  {(['image', 'video', 'audio', 'document'] as const).map((t) => (
                    <button
                      key={t}
                      type="button"
                      onClick={() => setMediaType(t)}
                      className={`px-3 py-1.5 rounded-[var(--radius-md)] text-xs font-semibold capitalize transition-colors cursor-pointer ${
                        mediaType === t
                          ? 'bg-[var(--color-accent)] text-white'
                          : 'bg-[var(--color-page)] text-[var(--color-ink-muted)] border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)]'
                      }`}
                    >
                      {t === 'image' && '🖼️ '}
                      {t === 'video' && '🎬 '}
                      {t === 'audio' && '🎵 '}
                      {t === 'document' && '📄 '}
                      {t}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  Media URL <span className="text-rose-500">*</span>
                </label>
                <input
                  type="url"
                  value={mediaUrl}
                  onChange={(e) => setMediaUrl(e.target.value)}
                  placeholder="https://example.com/image.jpg"
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                />
                <p className="text-[10px] text-[var(--color-ink-faint)] mt-1">
                  Must be a publicly accessible URL. Max sizes: Image 5MB, Video 16MB, Audio 16MB, Document 100MB
                </p>
              </div>

              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  Filename <span className="text-[var(--color-ink-faint)] font-normal">(optional, for documents)</span>
                </label>
                <input
                  type="text"
                  value={mediaFilename}
                  onChange={(e) => setMediaFilename(e.target.value)}
                  placeholder="document.pdf"
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                />
              </div>
            </div>
          )}

          {/* CTA URL Builder */}
          {activeType === 'cta_url' && (
            <div className="space-y-3">
              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  Button Display Text <span className="text-[var(--color-ink-faint)] font-normal">(max 20 chars)</span>
                </label>
                <input
                  type="text"
                  value={ctaDisplayText}
                  onChange={(e) => setCtaDisplayText(e.target.value.slice(0, 20))}
                  placeholder="Visit Website"
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                />
              </div>

              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
                  URL <span className="text-rose-500">*</span>
                </label>
                <input
                  type="url"
                  value={ctaUrl}
                  onChange={(e) => setCtaUrl(e.target.value)}
                  placeholder="https://example.com"
                  className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
                />
              </div>
            </div>
          )}

          {/* Footer Text (optional, for types that support it) */}
          {activeType !== 'location' && (
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
          )}

          {/* Preview Toggle */}
          <div className="flex items-center justify-between pt-2 border-t border-[var(--color-border)]">
            <span className="text-[11px] font-semibold text-[var(--color-ink-muted)]">Preview</span>
            <button
              type="button"
              onClick={() => setShowPreview(!showPreview)}
              className={`rounded-[var(--radius-sm)] px-3 py-1.5 text-[10px] font-semibold flex items-center gap-1.5 cursor-pointer transition-colors ${
                showPreview
                  ? 'bg-[var(--color-accent)] text-white'
                  : 'border border-[var(--color-border)] bg-[var(--color-page)] text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)]'
              }`}
            >
              <Eye className="h-3 w-3" />
              {showPreview ? 'Hide Preview' : 'Show Preview'}
            </button>
          </div>

          {showPreview && <PreviewCard />}
        </div>

        {/* Footer */}
        <div className="border-t border-[var(--color-border)] px-4 py-3 bg-[var(--color-page)]/50 rounded-b-xl flex items-center justify-between">
          <p className="text-[11px] text-[var(--color-ink-faint)]">
            Interactive messages require an open 24h customer service window.
          </p>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleSend}
              disabled={sending || sent}
              className={`rounded-[var(--radius-md)] px-4 py-1.5 text-xs font-semibold flex items-center gap-2 shadow-none transition-colors cursor-pointer ${
                sent
                  ? 'bg-emerald-100 text-emerald-600 border border-emerald-300'
                  : 'bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50'
              }`}
            >
              {sent ? (
                <>
                  <CheckCircle2 className="h-4 w-4" />
                  <span>Sent!</span>
                </>
              ) : sending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span>Sending...</span>
                </>
              ) : (
                <>
                  <Send className="h-4 w-4" />
                  <span>Send Interactive Message</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
