import {
  MessageSquare,
} from '../../components/icons'

interface Props {
  onComposeInteractive: () => void
}

export function WhatsAppInteractiveToolbar({ onComposeInteractive }: Props) {
  return (
    <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-none px-3 py-2">
      <div className="flex items-center gap-2">
        <span className="text-[11px] font-semibold text-[var(--color-ink-muted)] flex items-center gap-1.5">
          <MessageSquare className="h-3.5 w-3.5 text-emerald-600" />
          Interactive
        </span>
        <div className="h-4 w-px bg-[var(--color-border)]" />
        <button
          type="button"
          onClick={onComposeInteractive}
          className="rounded-[var(--radius-md)] bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-white hover:bg-[var(--color-accent-hover)] shadow-none transition-colors cursor-pointer flex items-center gap-1.5"
        >
          <span>🔘</span>
          <span>Buttons</span>
        </button>
        <button
          type="button"
          onClick={onComposeInteractive}
          className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] transition-colors cursor-pointer flex items-center gap-1.5"
        >
          <span>📋</span>
          <span>List</span>
        </button>
        <button
          type="button"
          onClick={onComposeInteractive}
          className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] transition-colors cursor-pointer flex items-center gap-1.5"
        >
          <span>📍</span>
          <span>Location</span>
        </button>
        <button
          type="button"
          onClick={onComposeInteractive}
          className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] transition-colors cursor-pointer flex items-center gap-1.5"
        >
          <span>🖼️</span>
          <span>Media</span>
        </button>
        <button
          type="button"
          onClick={onComposeInteractive}
          className="rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] transition-colors cursor-pointer flex items-center gap-1.5"
        >
          <span>🔗</span>
          <span>CTA URL</span>
        </button>
      </div>
    </div>
  )
}
