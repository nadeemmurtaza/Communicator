import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ArrowRight, Building2, Loader2, Plus } from 'lucide-react'
import { api, type OrganizationItem } from '../lib/api'
import { useAuth } from '../context/AuthContext'

export function Organizations() {
  const { session, selectOrganization } = useAuth()
  const [orgs, setOrgs] = useState<OrganizationItem[]>([])
  const [loading, setLoading] = useState(true)
  const [creating, setCreating] = useState(false)
  const [name, setName] = useState('')
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()

  useEffect(() => {
    if (!session) return
    api
      .listOrganizations()
      .then(setOrgs)
      .finally(() => setLoading(false))
  }, [session])

  function enter(org: OrganizationItem) {
    selectOrganization(org)
    navigate('/')
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault()
    if (!session) return
    setCreating(true)
    setError(null)
    try {
      const org = await api.createOrganization(name.trim())
      enter(org)
    } catch {
      setError('Could not create organization. Try a different name.')
    } finally {
      setCreating(false)
    }
  }

  if (!session) {
    navigate('/login')
    return null
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <div className="w-full max-w-md space-y-6">
        {loading ? (
          <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-8 text-center text-gray-500">
            <Loader2 className="h-6 w-6 animate-spin mx-auto text-gray-700" />
            <p className="mt-2 text-xs font-medium">Loading organizations...</p>
          </div>
        ) : (
          orgs.length > 0 && (
            <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-6 space-y-3">
              <div className="flex items-center gap-2">
                <Building2 className="h-4 w-4 text-gray-700" />
                <h2 className="text-base font-semibold text-gray-900">Your organizations</h2>
              </div>
              <div className="space-y-2 pt-1">
                {orgs.map((org) => (
                  <button
                    key={org.id}
                    onClick={() => enter(org)}
                    className="w-full text-left px-4 py-3 border border-gray-200 rounded-lg hover:border-gray-900 hover:bg-gray-50/50 transition flex items-center justify-between group"
                  >
                    <div>
                      <div className="text-sm font-medium text-gray-900 group-hover:text-black">{org.name}</div>
                      <div className="text-xs text-gray-400 font-mono mt-0.5">{org.tenantKey}</div>
                    </div>
                    <ArrowRight className="h-4 w-4 text-gray-400 group-hover:text-gray-900 transition-transform group-hover:translate-x-0.5" />
                  </button>
                ))}
              </div>
            </div>
          )
        )}

        <form onSubmit={handleCreate} className="bg-white rounded-xl border border-gray-200 shadow-sm p-6 space-y-4">
          <div className="flex items-start gap-3">
            <div className="h-9 w-9 rounded-lg bg-gray-900 text-white flex items-center justify-center shrink-0 mt-0.5">
              <Plus className="h-5 w-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-gray-900">Create an organization</h2>
              <p className="text-sm text-gray-500 mt-0.5">
                Each organization manages its own channels, templates, and customers.
              </p>
            </div>
          </div>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Acme Inc."
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-gray-900 focus:outline-none"
            required
          />
          <button
            type="submit"
            disabled={creating || !name.trim()}
            className="w-full px-4 py-2.5 bg-gray-900 text-white text-sm font-medium rounded-lg hover:bg-gray-800 transition disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {creating ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                <span>Creating organization...</span>
              </>
            ) : (
              <span>Create organization</span>
            )}
          </button>
          {error && <p className="text-sm text-red-600">{error}</p>}
        </form>
      </div>
    </div>
  )
}
