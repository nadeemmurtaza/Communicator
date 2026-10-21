import { NavLink, Outlet } from 'react-router-dom'

const NAV = [
  { to: '/', label: 'Dashboard', end: true },
  { to: '/reports', label: 'Reports' },
  { to: '/channels', label: 'Channels' },
  { to: '/drafts', label: 'Draft review' },
  { to: '/conversations', label: 'Conversations' },
  { to: '/tickets', label: 'Tickets' },
  { to: '/contacts', label: 'Contacts' },
  { to: '/broadcasts', label: 'Broadcasts' },
  { to: '/automation', label: 'Automation' },
  { to: '/knowledge-base', label: 'Knowledge base' },
  { to: '/simulations', label: 'AI Simulations' },
  { to: '/self-improving', label: 'Self-Improving AI' },
  { to: '/services', label: 'Service catalog' },
  { to: '/sequences', label: 'Sequences' },
  { to: '/team', label: 'Team' },
  { to: '/connections', label: 'Connections' },
  { to: '/webhooks', label: 'Webhooks' },
]

export function Layout() {
  return (
    <div className="min-h-screen bg-slate-50 text-slate-900">
      <div className="flex">
        <aside className="hidden md:flex md:w-56 md:flex-col md:border-r md:border-slate-200 md:bg-white md:min-h-screen">
          <div className="px-5 py-5 border-b border-slate-200">
            <span className="font-semibold tracking-tight">Communicator</span>
          </div>
          <nav className="flex flex-col gap-1 p-3">
            {NAV.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  `rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                    isActive
                      ? 'bg-slate-900 text-white'
                      : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
                  }`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </aside>

        <div className="flex-1 min-w-0">
          <header className="md:hidden flex items-center justify-between border-b border-slate-200 bg-white px-4 py-3">
            <span className="font-semibold">Communicator</span>
          </header>
          <nav className="md:hidden flex gap-1 overflow-x-auto border-b border-slate-200 bg-white px-3 py-2">
            {NAV.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  `shrink-0 rounded-md px-3 py-1.5 text-sm font-medium ${
                    isActive ? 'bg-slate-900 text-white' : 'text-slate-600 hover:bg-slate-100'
                  }`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>

          <main className="mx-auto max-w-5xl px-4 py-6 md:px-8 md:py-8">
            <Outlet />
          </main>
        </div>
      </div>
    </div>
  )
}
