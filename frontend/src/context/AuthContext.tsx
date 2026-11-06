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

interface Brand {
  id: string
  organizationId: string
  name: string
  slug: string
}

interface AuthContextValue {
  session: Session | null
  organization: Organization | null
  brand: Brand | null
  loading: boolean
  loginWithNid: () => Promise<void>
  completeNidCallback: (
    code: string,
    state: string,
    options?: { skipStateCheck?: boolean }
  ) => Promise<void>
  selectOrganization: (org: Organization) => void
  selectBrand: (brand: Brand) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

const SESSION_KEY = 'communicator.session'
const ORG_KEY = 'communicator.organization'
const BRAND_KEY = 'communicator.brand'
// Tab-scoped, not persisted to localStorage: proves *this browser*
// initiated the OAuth attempt whose state comes back on the callback, so a
// state value from a link an attacker completed for their own account
// (then sent to a victim) is rejected instead of silently adopted.
const PENDING_STATE_KEY = 'communicator.nid_pending_state'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<Session | null>(null)
  const [organization, setOrganization] = useState<Organization | null>(null)
  const [brand, setBrand] = useState<Brand | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    try {
      const s = localStorage.getItem(SESSION_KEY)
      const o = localStorage.getItem(ORG_KEY)
      const b = localStorage.getItem(BRAND_KEY)
      if (s) setSession(JSON.parse(s))
      if (o) setOrganization(JSON.parse(o))
      if (b) setBrand(JSON.parse(b))
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
      await completeNidCallback(`test_${Date.now()}`, data.state, { skipStateCheck: true })
    } else {
      sessionStorage.setItem(PENDING_STATE_KEY, data.state)
      window.location.href = data.authorize_url
    }
  }

  async function completeNidCallback(
    code: string,
    state: string,
    options?: { skipStateCheck?: boolean }
  ) {
    if (!options?.skipStateCheck) {
      const expected = sessionStorage.getItem(PENDING_STATE_KEY)
      if (!expected || expected !== state) {
        throw new Error('This sign-in link was not started from this browser.')
      }
    }
    sessionStorage.removeItem(PENDING_STATE_KEY)
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

  function selectBrand(brand: Brand) {
    localStorage.setItem(BRAND_KEY, JSON.stringify(brand))
    setBrand(brand)
  }

  function logout() {
    localStorage.removeItem(SESSION_KEY)
    localStorage.removeItem(ORG_KEY)
    localStorage.removeItem(BRAND_KEY)
    setSession(null)
    setOrganization(null)
    setBrand(null)
  }

  return (
    <AuthContext.Provider
      value={{ session, organization, brand, loading, loginWithNid, completeNidCallback, selectOrganization, selectBrand, logout }}
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
