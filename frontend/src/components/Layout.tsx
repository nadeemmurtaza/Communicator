import { NavLink, Outlet, useLocation } from 'react-router-dom'
import { Breadcrumbs } from './Breadcrumbs'
import { NAV_SECTIONS, findActiveSection } from './navSections'
import { CHANNEL_ICONS } from './icons/BrandIcons'

export function Layout() {
  const { pathname } = useLocation()
  const activeSection = findActiveSection(pathname)

  return (
    <div className="min-h-screen flex">
      {/* Primary nav -- narrow icon rail, Meta Business Suite style */}
      <aside className="hidden md:flex md:w-16 md:shrink-0 flex-col items-center gap-1 glass-panel-strong rounded-none border-y-0 border-l-0 py-4">
        <div className="mb-3 h-8 w-8 rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] flex items-center justify-center font-bold text-sm">
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
                `group relative flex h-11 w-11 items-center justify-center rounded-[var(--radius-sm)] transition-colors duration-[var(--motion-fast)] ${
                  isActive
                    ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                    : 'text-[var(--color-ink-muted)] hover:bg-black/5 hover:text-[var(--color-ink)]'
                }`
              }
            >
              <Icon className="h-[22px] w-[22px]" strokeWidth={2} />
              <span className="pointer-events-none absolute left-full ml-2 whitespace-nowrap rounded-[var(--radius-sm)] bg-[var(--color-ink)] px-2 py-1 text-xs font-medium text-white opacity-0 shadow-lg transition-opacity duration-[var(--motion-fast)] group-hover:opacity-100 z-[var(--layer-dropdown)]">
                {section.label}
              </span>
            </NavLink>
          )
        })}
      </aside>

      {/* Secondary nav -- labeled panel, children of the active primary section */}
      {activeSection?.children && (
        <aside className="hidden md:flex md:w-56 md:shrink-0 flex-col glass-panel-strong rounded-none border-y-0 py-5 px-3">
          <p className="px-2 pb-3 text-sm font-semibold text-[var(--color-ink)]">{activeSection.label}</p>
          <nav className="flex flex-col gap-0.5">
            {activeSection.children.map((child) => {
              const ChannelIcon = child.channel ? CHANNEL_ICONS[child.channel] : undefined
              return (
                <NavLink
                  key={child.to}
                  to={child.to}
                  end={child.end}
                  className={({ isActive }) =>
                    `flex items-center gap-2.5 rounded-[var(--radius-sm)] px-2.5 py-2 text-sm font-medium transition-colors duration-[var(--motion-fast)] ${
                      isActive
                        ? 'bg-[var(--color-accent)] text-[var(--color-on-accent)]'
                        : 'text-[var(--color-ink-muted)] hover:bg-black/5 hover:text-[var(--color-ink)]'
                    }`
                  }
                >
                  {ChannelIcon && <ChannelIcon className="h-[18px] w-[18px] shrink-0" />}
                  {child.label}
                </NavLink>
              )
            })}
          </nav>
        </aside>
      )}

      {/* Mobile primary nav */}
      <header className="md:hidden fixed inset-x-0 top-0 z-[var(--layer-sticky)] glass-panel-strong rounded-none border-x-0 border-t-0">
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
                    ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                    : 'text-[var(--color-ink-muted)] hover:bg-black/5'
                }`
              }
            >
              {section.label}
            </NavLink>
          ))}
        </nav>
      </header>

      <div className="flex-1 min-w-0">
        <main className="mx-auto max-w-6xl px-4 py-6 md:px-8 md:py-8 pt-24 md:pt-8">
          <Breadcrumbs />
          <Outlet />
        </main>
      </div>
    </div>
  )
}
