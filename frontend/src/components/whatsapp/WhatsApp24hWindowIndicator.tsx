import { AlertTriangle, Clock, CheckCircle2 } from '../../components/icons'
import type { WhatsApp24hWindow } from '../../lib/api'

interface Props {
  window: WhatsApp24hWindow
}

function formatRemaining(minutes: number): string {
  if (minutes <= 0) return 'Closed'
  if (minutes < 60) return `${minutes}m remaining`
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60
  return mins > 0 ? `${hours}h ${mins}m remaining` : `${hours}h remaining`
}

export function WhatsApp24hWindowIndicator({ window: w }: Props) {
  if (w.isOpen) {
    if (w.isClosingSoon) {
      return (
        <div className="rounded-[var(--radius-md)] border border-amber-200 bg-amber-50 px-4 py-3 flex items-center gap-3">
          <AlertTriangle className="h-5 w-5 text-amber-600 shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-semibold text-amber-800">
              24h window closing soon
            </p>
            <p className="text-xs text-amber-700">
              You can send free-text replies for <span className="font-semibold">{formatRemaining(w.remainingMinutes)}</span>. After that, only approved templates will be delivered.
            </p>
          </div>
          <Clock className="h-5 w-5 text-amber-500 shrink-0" />
        </div>
      )
    }
    return (
      <div className="rounded-[var(--radius-md)] border border-emerald-200 bg-emerald-50 px-4 py-2 flex items-center gap-2">
        <CheckCircle2 className="h-4 w-4 text-emerald-600 shrink-0" />
        <p className="text-xs font-medium text-emerald-800">
          24h customer service window is open — <span className="font-semibold">{formatRemaining(w.remainingMinutes)}</span>
        </p>
      </div>
    )
  }

  return (
    <div className="rounded-[var(--radius-md)] border border-rose-200 bg-rose-50 px-4 py-3 flex items-center gap-3">
      <Clock className="h-5 w-5 text-rose-600 shrink-0" />
      <div className="flex-1 min-w-0">
        <p className="text-sm font-semibold text-rose-800">
          24h window closed
        </p>
        <p className="text-xs text-rose-700">
          Customer service window has expired. You can only send pre-approved WhatsApp templates to this contact.
        </p>
      </div>
    </div>
  )
}
