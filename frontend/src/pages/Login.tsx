import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'

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
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="w-full max-w-sm bg-white rounded-xl border border-gray-200 shadow-sm p-8 text-center space-y-6">
        <div>
          <h1 className="text-xl font-semibold text-gray-900">Communicator</h1>
          <p className="text-sm text-gray-500 mt-1">Sign in to manage your organization's conversations.</p>
        </div>

        <button
          onClick={handleLogin}
          disabled={loading}
          className="w-full px-4 py-2.5 bg-gray-900 text-white text-sm font-medium rounded-lg hover:bg-gray-800 transition disabled:opacity-50"
        >
          {loading ? 'Signing in...' : 'Continue with nID'}
        </button>

        {error && <p className="text-sm text-red-600">{error}</p>}

        <p className="text-xs text-gray-400">
          nID (<a href="https://id.newax.co" className="underline hover:text-gray-600" target="_blank" rel="noreferrer">id.newax.co</a>) is the single sign-on for the NEWAX ecosystem. Your account works across every connected product.
        </p>
      </div>
    </div>
  )
}
