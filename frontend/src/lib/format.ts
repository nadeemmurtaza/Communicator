export function timeAgo(iso: string): string {
  const diffMs = Date.now() - new Date(iso).getTime()
  const mins = Math.round(diffMs / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hours = Math.round(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.round(hours / 24)
  return `${days}d ago`
}

export function stateLabel(state: string): string {
  return state
    .split('_')
    .map((w) => w[0].toUpperCase() + w.slice(1))
    .join(' ')
}

export function stateBadgeClass(state: string): string {
  switch (state) {
    case 'hot_lead':
      return 'badge-error'
    case 'engaged':
    case 'awaiting_reply':
      return 'badge-info'
    case 'closed_won':
      return 'badge-success'
    case 'closed_lost':
    case 'opted_out':
      return 'badge-neutral'
    case 'handed_off':
      return 'badge-warning'
    default:
      return 'badge-ghost'
  }
}
