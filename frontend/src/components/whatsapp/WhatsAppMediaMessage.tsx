import { useState, useRef, useEffect } from 'react'
import {
  Play, Pause, Download, ExternalLink, Image as ImageIcon,
  FileText, Video, Mic, Loader2, X, Eye,
} from '../../components/icons'
import type { Message } from '../../lib/api'

// ---------------------------------------------------------------------------
// Format seconds to MM:SS
// ---------------------------------------------------------------------------
function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

// ---------------------------------------------------------------------------
// Image Message
// ---------------------------------------------------------------------------
function ImageMessage({ url, filename, mimetype }: { url: string; filename?: string | null; mimetype?: string | null }) {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(false)
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="space-y-2">
      <div className="relative rounded-[var(--radius-md)] overflow-hidden bg-[var(--color-page)] border border-[var(--color-border)]">
        {loading && !error && (
          <div className="flex items-center justify-center h-48">
            <Loader2 className="h-6 w-6 text-[var(--color-ink-muted)] animate-spin" />
          </div>
        )}
        {error ? (
          <div className="flex flex-col items-center justify-center h-48 gap-2 text-[var(--color-ink-muted)]">
            <ImageIcon className="h-8 w-8" />
            <span className="text-xs">Failed to load image</span>
            <a
              href={url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-[var(--color-accent)] hover:underline inline-flex items-center gap-1"
            >
              <ExternalLink className="h-3 w-3" /> Open externally
            </a>
          </div>
        ) : (
          <img
            src={url}
            alt={filename || 'WhatsApp image'}
            className={`w-full max-h-80 object-contain cursor-pointer transition-opacity ${loading ? 'opacity-0' : 'opacity-100'}`}
            onClick={() => setExpanded(true)}
            onLoad={() => setLoading(false)}
            onError={() => { setError(true); setLoading(false) }}
          />
        )}
      </div>
      {filename && <p className="text-[10px] text-[var(--color-ink-faint)] truncate">{filename}</p>}

      {/* Expanded modal */}
      {expanded && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70" onClick={() => setExpanded(false)}>
          <div className="relative max-w-4xl max-h-[90vh] p-4" onClick={(e) => e.stopPropagation()}>
            <button
              onClick={() => setExpanded(false)}
              className="absolute top-0 right-0 z-10 rounded-full bg-black/50 text-white p-2 hover:bg-black/70 cursor-pointer"
            >
              <X className="h-4 w-4" />
            </button>
            <img src={url} alt={filename || 'WhatsApp image'} className="max-h-[85vh] w-auto rounded-[var(--radius-md)]" />
          </div>
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Video Message
// ---------------------------------------------------------------------------
function VideoMessage({ url, filename }: { url: string; filename?: string | null }) {
  return (
    <div className="space-y-2">
      <div className="rounded-[var(--radius-md)] overflow-hidden bg-[var(--color-page)] border border-[var(--color-border)]">
        <video
          controls
          className="w-full max-h-80"
          poster=""
        >
          <source src={url} />
          Your browser does not support video playback.
        </video>
      </div>
      {filename && <p className="text-[10px] text-[var(--color-ink-faint)] truncate">{filename}</p>}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Document Message
// ---------------------------------------------------------------------------
function DocumentMessage({ url, filename, mimetype }: { url: string; filename?: string | null; mimetype?: string | null }) {
  const ext = filename?.split('.').pop()?.toLowerCase() || ''
  const isPdf = ext === 'pdf' || mimetype?.includes('pdf')

  return (
    <div className="flex items-center gap-3 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] p-3">
      <div className="rounded-[var(--radius-md)] bg-[var(--color-accent-muted)] p-2.5 shrink-0">
        <FileText className="h-5 w-5 text-[var(--color-accent)]" />
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-xs font-medium text-[var(--color-ink)] truncate">
          {filename || 'Document'}
        </p>
        <p className="text-[10px] text-[var(--color-ink-faint)]">
          {mimetype || 'application/pdf'}
        </p>
      </div>
      <a
        href={url}
        target="_blank"
        rel="noopener noreferrer"
        download={filename || undefined}
        className="rounded-[var(--radius-md)] p-2 text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] hover:text-[var(--color-accent)] transition-colors"
        title="Download"
      >
        <Download className="h-4 w-4" />
      </a>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Audio / Voice Message
// ---------------------------------------------------------------------------
function AudioMessage({
  url, duration, transcription, isVoice,
}: {
  url: string
  duration?: number | null
  transcription?: string | null
  isVoice?: boolean
}) {
  const [playing, setPlaying] = useState(false)
  const [progress, setProgress] = useState(0)
  const audioRef = useRef<HTMLAudioElement>(null)

  useEffect(() => {
    const audio = audioRef.current
    if (!audio) return

    const onUpdate = () => {
      if (audio.duration) {
        setProgress((audio.currentTime / audio.duration) * 100)
      }
    }
    const onEnd = () => setPlaying(false)
    const onPlay = () => setPlaying(true)
    const onPause = () => setPlaying(false)

    audio.addEventListener('timeupdate', onUpdate)
    audio.addEventListener('ended', onEnd)
    audio.addEventListener('play', onPlay)
    audio.addEventListener('pause', onPause)

    return () => {
      audio.removeEventListener('timeupdate', onUpdate)
      audio.removeEventListener('ended', onEnd)
      audio.removeEventListener('play', onPlay)
      audio.removeEventListener('pause', onPause)
    }
  }, [])

  const handlePlayPause = () => {
    if (audioRef.current) {
      if (playing) {
        audioRef.current.pause()
      } else {
        audioRef.current.play()
        setPlaying(true)
      }
    }
  }

  const currentSec = audioRef.current?.currentTime || 0
  const durSec = duration || audioRef.current?.duration || 0

  return (
    <div className="space-y-2">
      <audio ref={audioRef} src={url} preload="metadata" />

      <div className={`rounded-[var(--radius-md)] border p-3 ${
        isVoice
          ? 'border-emerald-200 bg-emerald-50/50'
          : 'border-[var(--color-border)] bg-[var(--color-page)]'
      }`}>
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={handlePlayPause}
            className={`rounded-full p-2.5 shrink-0 transition-colors cursor-pointer ${
              isVoice
                ? 'bg-emerald-500 text-white hover:bg-emerald-600'
                : 'bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)]'
            }`}
          >
            {playing ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
          </button>

          <div className="flex-1 min-w-0">
            {/* Waveform-like progress bar */}
            <div className="relative h-8 flex items-center gap-px">
              {Array.from({ length: 40 }).map((_, i) => {
                const h = isVoice
                  ? `${8 + Math.sin(i * 0.5) * 6 + Math.random() * 8}px`
                  : '4px'
                const filled = (i / 40) * 100 <= progress
                return (
                  <div
                    key={i}
                    className={`flex-1 rounded-full transition-colors ${
                      filled
                        ? isVoice ? 'bg-emerald-400' : 'bg-[var(--color-accent)]'
                        : 'bg-[var(--color-border)]'
                    }`}
                    style={{ height: h }}
                  />
                )
              })}
            </div>

            <div className="flex items-center justify-between mt-1">
              <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">
                {formatDuration(currentSec)}
              </span>
              <span className="text-[10px] text-[var(--color-ink-faint)] font-mono">
                {formatDuration(durSec)}
              </span>
            </div>
          </div>

          {isVoice && (
            <div className="rounded-full bg-emerald-100 px-2 py-0.5">
              <Mic className="h-3 w-3 text-emerald-600" />
            </div>
          )}
        </div>
      </div>

      {/* Transcription */}
      {transcription && (
        <div className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] p-3">
          <p className="text-[10px] font-semibold text-[var(--color-ink-muted)] mb-1">Transcription</p>
          <p className="text-xs text-[var(--color-ink)] italic">"{transcription}"</p>
        </div>
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main Component
// ---------------------------------------------------------------------------
export function WhatsAppMediaMessage({ message }: { message: Message }) {
  const { mediaType, mediaUrl, mediaFilename, mediaMimetype, voiceDuration, transcriptionText, body } = message

  if (!mediaType || !mediaUrl) {
    return null
  }

  switch (mediaType) {
    case 'image':
      return <ImageMessage url={mediaUrl} filename={mediaFilename} mimetype={mediaMimetype} />

    case 'video':
      return <VideoMessage url={mediaUrl} filename={mediaFilename} />

    case 'document':
      return <DocumentMessage url={mediaUrl} filename={mediaFilename} mimetype={mediaMimetype} />

    case 'audio':
      return (
        <AudioMessage
          url={mediaUrl}
          duration={voiceDuration}
          transcription={transcriptionText || body || undefined}
          isVoice={mediaFilename?.toLowerCase().includes('voice') || !!voiceDuration}
        />
      )

    default:
      // Fallback: link to media
      return (
        <a
          href={mediaUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-2 text-xs text-[var(--color-accent)] hover:bg-[var(--color-accent-muted)] transition-colors"
        >
          <ExternalLink className="h-3.5 w-3.5" />
          <span className="truncate max-w-[200px]">{mediaFilename || mediaUrl}</span>
        </a>
      )
  }
}
