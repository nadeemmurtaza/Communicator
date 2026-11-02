import { Link, useLocation } from 'react-router-dom'
import { findActiveChild, findActiveSection } from './navSections'

export function Breadcrumbs() {
  const { pathname } = useLocation()
  const section = findActiveSection(pathname)
  const child = findActiveChild(section, pathname)

  const trail: { label: string; to?: string }[] = [{ label: 'Home', to: '/' }]
  if (section && section.id !== 'dashboard') trail.push({ label: section.label, to: section.children ? undefined : section.to })
  if (child && child.to !== section?.to) trail.push({ label: child.label, to: child.to })

  if (trail.length <= 1) return null

  return (
    <nav aria-label="Breadcrumb" className="flex items-center gap-1.5 text-sm text-[var(--color-ink-muted)] mb-4">
      {trail.map((item, i) => (
        <span key={`${item.label}-${i}`} className="flex items-center gap-1.5">
          {i > 0 && <span className="text-[var(--color-ink-faint)]">/</span>}
          {item.to && i < trail.length - 1 ? (
            <Link to={item.to} className="hover:text-[var(--color-ink)] transition-colors">
              {item.label}
            </Link>
          ) : (
            <span className={i === trail.length - 1 ? 'text-[var(--color-ink)] font-medium' : ''}>{item.label}</span>
          )}
        </span>
      ))}
    </nav>
  )
}
