import { useState } from 'react'
import {
  X, Image as ImageIcon, Video, FileText, Mic, Link as LinkIcon,
  Loader2, CheckCircle2, Send,
} from '../../components/icons'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type MediaType = 'image' | 'video' | 'document' | 'audio'

interface MediaAttachment {
  type: MediaType
  url: string
  filename?: string
  mimetype?: string
  caption?: string
}

interface Props {
  open: boolean
  onClose: () => void
  onSend: (attachment: MediaAttachment) => Promise<void>
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function WhatsAppMediaComposer({ open, onClose, onSend }: Props) {
  const [tab, setTab] = useState<MediaType>('image')
  const [url, setUrl] = useState('')
  const [filename, setFilename] = useState('')
  const [caption, setCaption] = useState('')
  const [sending, setSending] = useState(false)
  const [sent, setSent] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const tabs: { type: MediaType; icon: any; label: string; placeholder: string; mimetypes: string }[] = [
    { type: 'image', icon: ImageIcon, label: 'Image', placeholder: 'https://example.com/photo.jpg', mimetypes: 'image/jpeg, image/png, image/gif, image/webp' },
    { type: 'video', icon: Video, label: 'Video', placeholder: 'https://example.com/video.mp4', mimetypes: 'video/mp4, video/webm' },
    { type: 'document', icon: FileText, label: 'Document', placeholder: 'https://example.com/file.pdf', mimetypes: 'application/pdf, application/msword' },
    { type: 'audio', icon: Mic, label: 'Audio', placeholder: 'https://example.com/voice.ogg', mimetypes: 'audio/ogg, audio/mpeg, audio/wav' },
  ]

  function reset() {
    setUrl('')
    setFilename('')
    setCaption('')
    setError(null)
    setSent(false)
  }

  function handleOpen() {
    reset()
  }

  async function handleSend() {
    if (!url.trim()) {
      setError('Media URL is required')
      return
    }

    setSending(true)
    setError(null)
    try {
      const attachment: MediaAttachment = {
        type: tab,
        url: url.trim(),
        filename: filename.trim() || undefined,
        mimetype: tabs.find(t => t.type === tab)?.mimetypes.split(',')[0].trim(),
        caption: caption.trim() || undefined,
      }
      await onSend(attachment)
      setSent(true)
      setTimeout(() => {
        onClose()
        reset()
      }, 1500)
    } catch (e: any) {
      setError(e.message || 'Failed to send media')
    } finally {
      setSending(false)
    }
  }

  if (!open) return null

  const currentTab = tabs.find(t => t.type === tab) || tabs[0]
  const TabIcon = currentTab.icon

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="w-full max-w-lg rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl flex flex-col mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
          <div>
            <h2 className="text-sm font-semibold text-[var(--color-ink)]">
              Attach Media
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">
              Send media via URL link to the customer
            </p>
          </div>
          <button
            type="button"
            onClick={() => { onClose(); reset() }}
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
        {sent && (
          <div className="mx-4 mt-3 rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-700 flex items-center gap-2">
            <CheckCircle2 className="h-3.5 w-3.5" />
            Media sent successfully
          </div>
        )}

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* Tab selector */}
          <div className="grid grid-cols-4 gap-2">
            {tabs.map((t) => {
              const Icon = t.icon
              const isActive = t.type === tab
              return (
                <button
                  key={t.type}
                  type="button"
                  onClick={() => { setTab(t.type); reset() }}
                  className={`flex flex-col items-center gap-1.5 rounded-[var(--radius-md)] p-3 text-xs font-medium transition-colors cursor-pointer border ${
                    isActive
                      ? 'bg-[var(--color-accent)] text-white border-[var(--color-accent)]'
                      : 'bg-[var(--color-page)] text-[var(--color-ink-muted)] border-[var(--color-border)] hover:bg-[var(--color-accent-muted)] hover:text-[var(--color-accent)]'
                  }`}
                >
                  <Icon className="h-5 w-5" />
                  <span>{t.label}</span>
                </button>
              )
            })}
          </div>

          {/* URL Input */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Media URL <span className="text-rose-500">*</span>
            </label>
            <div className="relative">
              <input
                type="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder={currentTab.placeholder}
                className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] pl-9 pr-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
              />
              <LinkIcon className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-[var(--color-ink-faint)]" />
            </div>
            <p className="text-[10px] text-[var(--color-ink-faint)] mt-1">
              Supported: {currentTab.mimetypes}
            </p>
          </div>

          {/* Filename */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Filename <span className="text-[var(--color-ink-faint)] font-normal">(optional)</span>
            </label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder={`${currentTab.label.toLowerCase()}.jpg`}
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none"
            />
          </div>

          {/* Caption */}
          <div>
            <label className="text-[11px] font-semibold text-[var(--color-ink)] block mb-1">
              Caption <span className="text-[var(--color-ink-faint)] font-normal">(optional, max 1024 chars)</span>
            </label>
            <textarea
              rows={2}
              value={caption}
              onChange={(e) => setCaption(e.target.value.slice(0, 1024))}
              placeholder={`Caption for ${currentTab.label.toLowerCase()}...`}
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-ink)] placeholder-[var(--color-ink-faint)] focus:border-[var(--color-accent)] focus:outline-none resize-none"
            />
            <p className="text-[10px] text-[var(--color-ink-faint)] mt-1">{caption.length}/1024</p>
          </div>

          {/* Preview */}
          {url && tab === 'image' && (
            <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] overflow-hidden bg-[var(--color-page)]">
              <p className="text-[10px] font-semibold text-[var(--color-ink-muted)] px-3 pt-2">Preview</p>
              <img
                src={url}
                alt="Preview"
                className="w-full max-h-40 object-contain p-2"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = 'none'
                }}
              />
            </div>
          )}

          {url && tab === 'video' && (
            <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] overflow-hidden bg-[var(--color-page)]">
              <p className="text-[10px] font-semibold text-[var(--color-ink-muted)] px-3 pt-2">Preview</p>
              <video src={url} className="w-full max-h-40 p-2" controls />
            </div>
          )}

          {url && tab === 'audio' && (
            <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] overflow-hidden bg-[var(--color-page)]">
              <p className="text-[10px] font-semibold text-[var(--color-ink-muted)] px-3 pt-2">Preview</p>
              <audio src={url} className="w-full p-2" controls />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 p-4 border-t border-[var(--color-border)]">
          <button
            type="button"
            onClick={() => { onClose(); reset() }}
            className="rounded-[var(--radius-md)] px-4 py-2 text-xs font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-page)] transition-colors cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleSend}
            disabled={!url.trim() || sending}
            className="rounded-[var(--radius-md)] bg-[var(--color-accent)] px-4 py-2 text-xs font-semibold text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50 shadow-none transition-colors cursor-pointer flex items-center gap-2"
          >
            {sending ? (
              <>
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                <span>Sending...</span>
              </>
            ) : (
              <>
                <Send className="h-3.5 w-3.5" />
                <span>Send {currentTab.label}</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
