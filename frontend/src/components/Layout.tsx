import { NavLink, Outlet, useLocation } from 'react-router-dom'
import { Breadcrumbs } from './Breadcrumbs'
import { NAV_SECTIONS, findActiveSection } from './navSections'
import { CHANNEL_ICONS } from './icons/channels'

export function Layout() {
  const { pathname } = useLocation()
  const activeSection = findActiveSection(pathname)

  return (
    <div className="min-h-screen flex">
      {/* Primary nav -- narrow icon rail, Intercom style */}
      <aside className="hidden md:flex md:w-16 md:shrink-0 flex-col items-center gap-0.5 bg-[var(--color-surface)] border-r border-[var(--color-border)] py-4">
        <div className="mb-3 h-8 w-8 rounded-[var(--radius-sm)] bg-[var(--color-ink)] text-[var(--color-on-accent)] flex items-center justify-center font-bold text-sm">
          C
        </div>
        {NAV_SECTIONS.map((section) => {
          const Icon = section.icon
          const isActive = activeSection?.id === section.id
          return (
            <NavLink
              key={section.id}
              to={section.to}
              end={section.end}
              title={section.label}
              className={() =>
                `group relative flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] transition-colors duration-150 ${
                  isActive
                    ? 'bg-[var(--color-ink)] text-white'
                    : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-page)] hover:text-[var(--color-ink)]'
                }`
              }
            >
              <Icon className="h-5 w-5" strokeWidth={isActive ? 2.5 : 2} fill={isActive ? 'currentColor' : 'none'} />
              <span className="pointer-events-none absolute left-full ml-2 whitespace-nowrap rounded-[var(--radius-sm)] bg-[var(--color-ink)] px-2 py-1 text-xs font-medium text-white opacity-0 shadow-lg transition-opacity duration-150 group-hover:opacity-100 z-[var(--layer-dropdown)]">
                {section.label}
              </span>
            </NavLink>
          )
        })}
      </aside>

      {/* Secondary nav -- labeled panel, Intercom style */}
      {activeSection?.children && (
        <aside className="hidden md:flex md:w-56 md:shrink-0 flex-col bg-[var(--color-page)] border-r border-[var(--color-border)] py-5 px-2">
          <p className="px-2 pb-3 text-xs font-bold uppercase tracking-[0.6px] text-[var(--color-ink-faint)]">{activeSection.label}</p>
          <nav className="flex flex-col gap-0.5">
            {activeSection.children.map((child) => {
              const ChannelIcon = child.channel ? CHANNEL_ICONS[child.channel] : undefined
              return (
                <NavLink
                  key={child.to}
                  to={child.to}
                  end={child.end}
                  className={({ isActive }) =>
                    `flex items-center gap-2.5 rounded-[var(--radius-sm)] px-2.5 py-1.5 text-sm transition-colors duration-150 ${
                      isActive
                        ? 'bg-[var(--color-surface)] text-[var(--color-ink)] font-semibold shadow-sm border border-[var(--color-border)]'
                        : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface)] hover:text-[var(--color-ink)]'
                    }`
                  }
                >
                  {ChannelIcon && <ChannelIcon className="h-4 w-4 shrink-0" />}
                  {child.label}
                </NavLink>
              )
            })}
          </nav>
        </aside>
      )}

      {/* Mobile primary nav */}
      <header className="md:hidden fixed inset-x-0 top-0 z-[var(--layer-sticky)] bg-[var(--color-surface)] border-b border-[var(--color-border)]">
        <div className="flex items-center justify-between px-4 h-14">
          <span className="font-semibold tracking-tight text-[var(--color-ink)]">Communicator</span>
        </div>
        <nav className="flex gap-1 overflow-x-auto px-3 pb-2">
          {NAV_SECTIONS.map((section) => (
            <NavLink
              key={section.id}
              to={section.to}
              end={section.end}
              className={() =>
                `shrink-0 rounded-[var(--radius-sm)] px-3 py-1.5 text-sm font-medium ${
                  activeSection?.id === section.id
                    ? 'bg-[var(--color-ink)] text-white'
                    : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-page)] hover:text-[var(--color-ink)]'
                }`
              }
            >
              {section.label}
            </NavLink>
          ))}
        </nav>
      </header>

      <div className="flex-1 min-w-0">
        <main className="mx-auto max-w-7xl px-4 py-6 md:px-8 md:py-8 pt-24 md:pt-8">
          <Breadcrumbs />
          <Outlet />
        </main>
      </div>
    </div>
  )
}
