import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'

interface Session {
  token: string
  accountId: string
  email?: string
}

interface Organization {
  id: string
  tenantKey: string
  name: string
}

interface AuthContextValue {
  session: Session | null
  organization: Organization | null
  loading: boolean
  loginWithNid: () => Promise<void>
  completeNidCallback: (code: string, state: string) => Promise<void>
  selectOrganization: (org: Organization) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

const SESSION_KEY = 'communicator.session'
const ORG_KEY = 'communicator.organization'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<Session | null>(null)
  const [organization, setOrganization] = useState<Organization | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    try {
      const s = localStorage.getItem(SESSION_KEY)
      const o = localStorage.getItem(ORG_KEY)
      if (s) setSession(JSON.parse(s))
      if (o) setOrganization(JSON.parse(o))
    } catch {
      // ignore corrupted storage
    }
    setLoading(false)
  }, [])

  async function loginWithNid() {
    const res = await fetch('/api/v1/auth/nid/start')
    const data = await res.json()
    if (data.dev_mode) {
      // No live nID instance configured -- complete the exchange directly
      // with a dev test code instead of redirecting to an unreachable IdP.
      await completeNidCallback(`test_${Date.now()}`, data.state)
    } else {
      window.location.href = data.authorize_url
    }
  }

  async function completeNidCallback(code: string, state: string) {
    const res = await fetch(`/api/v1/auth/nid/callback?code=${encodeURIComponent(code)}&state=${encodeURIComponent(state)}`)
    if (!res.ok) throw new Error('nID login failed')
    const data = await res.json()
    const s: Session = { token: data.token, accountId: data.account_id, email: data.email }
    localStorage.setItem(SESSION_KEY, JSON.stringify(s))
    setSession(s)
  }

  function selectOrganization(org: Organization) {
    localStorage.setItem(ORG_KEY, JSON.stringify(org))
    setOrganization(org)
  }

  function logout() {
    localStorage.removeItem(SESSION_KEY)
    localStorage.removeItem(ORG_KEY)
    setSession(null)
    setOrganization(null)
  }

  return (
    <AuthContext.Provider
      value={{ session, organization, loading, loginWithNid, completeNidCallback, selectOrganization, logout }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}
