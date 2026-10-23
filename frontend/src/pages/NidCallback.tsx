import { useEffect, useRef, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'

/**
 * nID's real (non-dev_mode) redirect target: the authorize screen sends the
 * browser here with ?code&state, not to the backend's JSON callback
 * endpoint directly, so the SPA gets a chance to store the session and
 * route the person onward instead of landing on raw JSON.
 */
export function NidCallback() {
  const { completeNidCallback } = useAuth()
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const ran = useRef(false)

  useEffect(() => {
    if (ran.current) return
    ran.current = true

    const code = params.get('code')
    const state = params.get('state')
    const nidError = params.get('error')

    if (nidError) {
      setError(`nID sign-in failed: ${nidError}`)
      return
    }
    if (!code || !state) {
      setError('Missing code or state from nID.')
      return
    }

    completeNidCallback(code, state)
      .then(() => navigate('/organizations', { replace: true }))
      .catch(() => setError('Could not complete sign-in. Please try again.'))
  }, [params, completeNidCallback, navigate])

  return (
    <div className="min-h-screen flex items-center justify-center bg-[var(--color-page)]">
      <div className="glass-card p-8 text-center space-y-3">
        {error ? (
          <>
            <p className="text-sm text-red-600">{error}</p>
            <button
              onClick={() => navigate('/login', { replace: true })}
              className="px-4 py-2 bg-[var(--color-accent)] text-[var(--color-on-accent)] text-sm font-medium rounded-lg"
            >
              Back to sign in
            </button>
          </>
        ) : (
          <p className="text-sm text-[var(--color-ink-muted)]">Signing you in...</p>
        )}
      </div>
    </div>
  )
}
