import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { SparklesAnimated } from '../components/icons/animated'

const NID_LOGIN_UI = 'https://id.newax.co'

export function Login() {
  const { loginWithNid } = useAuth()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()

  async function handleLogin() {
    setLoading(true)
    setError(null)
    try {
      await loginWithNid()
      navigate('/organizations')
    } catch {
      setError('Login failed. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-[var(--color-page)]">
      <div className="w-full max-w-sm bg-white rounded-xl border border-[var(--color-border)] shadow-md p-8 text-center space-y-6">
        {/* Logo & Brand */}
        <div className="flex flex-col items-center gap-2">
          <div className="w-12 h-12 rounded-full bg-[var(--color-accent)] flex items-center justify-center">
            <SparklesAnimated size={24} className="text-white" />
          </div>
          <p className="text-xs font-semibold tracking-widest uppercase text-[var(--color-ink-faint)]">
            NEWAX
          </p>
        </div>

        {/* Headline */}
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-ink)]">Your work, connected.</h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-2 leading-relaxed">
            One identity for your personal and business spaces.
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-col gap-3">
          {/* Sign In — uses nID OAuth flow */}
          <button
            onClick={handleLogin}
            disabled={loading}
            className="w-full px-4 py-3 bg-[var(--color-accent)] text-white text-sm font-semibold rounded-lg hover:bg-[var(--color-accent-hover)] transition disabled:opacity-50"
          >
            {loading ? 'Signing in...' : 'Sign in to Communicator'}
          </button>

          {/* Create ID — links to nID login-ui create page */}
          <a
            href={`${NID_LOGIN_UI}/create`}
            target="_blank"
            rel="noreferrer"
            className="w-full px-4 py-3 bg-transparent text-[var(--color-accent)] text-sm font-semibold rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-accent-muted)] transition"
          >
            Create a NEWAX ID
          </a>
        </div>

        {error && <p className="text-sm text-red-600" role="alert">{error}</p>}

        {/* Subtitle */}
        <p className="text-xs text-[var(--color-ink-faint)] leading-relaxed">
          Secure access to the spaces you are authorized for.
        </p>

        {/* Legal links */}
        <div className="flex items-center justify-center gap-2 text-xs text-[var(--color-ink-faint)]">
          <a href={`${NID_LOGIN_UI}/privacy`} target="_blank" rel="noreferrer" className="underline hover:text-[var(--color-ink-muted)]">Privacy</a>
          <span aria-hidden="true">·</span>
          <a href={`${NID_LOGIN_UI}/security`} target="_blank" rel="noreferrer" className="underline hover:text-[var(--color-ink-muted)]">Security</a>
          <span aria-hidden="true">·</span>
          <a href={`${NID_LOGIN_UI}/terms`} target="_blank" rel="noreferrer" className="underline hover:text-[var(--color-ink-muted)]">Terms</a>
        </div>
      </div>
    </div>
  )
}
