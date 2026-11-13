import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  getLaunchpadOverview,
  type LaunchpadOverview,
  type NidProfile,
  type NidOrganization,
} from '../lib/api'
import {
  User,
  Briefcase,
  Building2,
  ShoppingCart,
  FileText,
  CreditCard,
  SparklesAnimated,
  ArrowRight,
  Globe,
  CheckCircle,
  AlertCircle,
} from '../components/icons'

function getStatusColor(status: string) {
  switch (status.toLowerCase()) {
    case 'active':
    case 'paid':
    case 'succeeded':
      return 'text-green-600 bg-green-50'
    case 'pending':
    case 'processing':
      return 'text-amber-600 bg-amber-50'
    case 'failed':
    case 'overdue':
    case 'void':
      return 'text-red-600 bg-red-50'
    default:
      return 'text-muted bg-accent-muted'
  }
}

function ProfileCard({
  profile,
  type,
  onClick,
}: {
  profile: NidProfile
  type: 'personal' | 'work'
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className="w-full text-left bg-white border border-border rounded-8px p-5 hover:border-accent transition-colors group"
    >
      <div className="flex items-start gap-4">
        <div
          className={`w-10 h-10 rounded-8px flex items-center justify-center flex-shrink-0 ${
            type === 'personal'
              ? 'bg-accent-muted text-accent'
              : 'bg-black text-white'
          }`}
        >
          {type === 'personal' ? <User size={18} /> : <Briefcase size={18} />}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold truncate">{profile.displayName}</h3>
            <span
              className={`text-[10px] font-medium uppercase tracking-wider px-2 py-0.5 rounded-4px ${
                type === 'personal'
                  ? 'bg-accent-muted text-accent'
                  : 'bg-black text-white'
              }`}
            >
              {type}
            </span>
          </div>
          {profile.headline && (
            <p className="text-sm text-muted mt-0.5 truncate">{profile.headline}</p>
          )}
          <div className="flex items-center gap-3 mt-2 text-xs text-faint">
            <span className="uppercase tracking-wider">{profile.category}</span>
            {profile.city && <span>· {profile.city}</span>}
            {profile.isPublished && (
              <span className="flex items-center gap-1 text-green-600">
                <CheckCircle size={12} /> Published
              </span>
            )}
          </div>
        </div>
        <ArrowRight
          size={16}
          className="text-faint group-hover:text-accent transition-colors flex-shrink-0 mt-1"
        />
      </div>
    </button>
  )
}

function StatCard({
  label,
  value,
  icon,
  color,
}: {
  label: string
  value: string | number
  icon: React.ReactNode
  color: string
}) {
  return (
    <div className="bg-white border border-border rounded-8px p-5">
      <div className="flex items-center gap-3">
        <div className={`w-8 h-8 rounded-4px flex items-center justify-center ${color}`}>
          {icon}
        </div>
        <div>
          <p className="text-xs text-faint uppercase tracking-wider">{label}</p>
          <p className="text-xl font-bold mt-0.5">{value}</p>
        </div>
      </div>
    </div>
  )
}

function OrganizationCard({
  org,
  onClick,
}: {
  org: NidOrganization
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className="w-full text-left bg-white border border-border rounded-8px p-4 hover:border-accent transition-colors group"
    >
      <div className="flex items-center gap-3">
        <div className="w-8 h-8 rounded-4px bg-black text-white flex items-center justify-center flex-shrink-0">
          <Building2 size={14} />
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-medium text-sm truncate">{org.nidLegalName}</p>
          <div className="flex items-center gap-2 mt-0.5">
            {org.isVerified ? (
              <span className="flex items-center gap-1 text-xs text-green-600">
                <CheckCircle size={10} /> Verified
              </span>
            ) : (
              <span className="flex items-center gap-1 text-xs text-amber-600">
                <AlertCircle size={10} /> Pending
              </span>
            )}
            {org.nidCountryCode && (
              <span className="text-xs text-faint">· {org.nidCountryCode}</span>
            )}
          </div>
        </div>
        <ArrowRight
          size={14}
          className="text-faint group-hover:text-accent transition-colors flex-shrink-0"
        />
      </div>
    </button>
  )
}

export function Launchpad() {
  const navigate = useNavigate()
  const [overview, setOverview] = useState<LaunchpadOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function load() {
      try {
        // Get team member ID from session
        const session = localStorage.getItem('communicator.session')
        const teamMemberId = session
          ? JSON.parse(session).userId || 'me'
          : 'me'
        const data = await getLaunchpadOverview(teamMemberId)
        setOverview(data)
      } catch (err: any) {
        setError(err.message || 'Failed to load launchpad')
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [])

  if (loading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-xl font-semibold">Launchpad</h1>
          <p className="text-sm text-muted mt-0.5">Loading your profiles and assets...</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="bg-white border border-border rounded-8px p-5 animate-pulse">
              <div className="h-4 bg-accent-muted rounded w-1/2 mb-3" />
              <div className="h-6 bg-accent-muted rounded w-1/3" />
            </div>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-xl font-semibold">Launchpad</h1>
          <p className="text-sm text-muted mt-0.5">Your profiles and assets</p>
        </div>
        <div className="bg-white border border-border rounded-8px p-8 text-center">
          <AlertCircle size={24} className="mx-auto text-red-600 mb-3" />
          <p className="text-sm text-muted">{error}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-black text-white text-sm rounded-4px hover:scale-[1.05] transition-transform"
          >
            Retry
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold flex items-center gap-2">
            <SparklesAnimated size={20} className="text-accent" />
            Launchpad
          </h1>
          <p className="text-sm text-muted mt-0.5">
            Access all your profiles, organizations, and assets
          </p>
        </div>
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Orders"
          value={overview?.totalOrders ?? 0}
          icon={<ShoppingCart size={14} />}
          color="bg-accent-muted text-accent"
        />
        <StatCard
          label="Invoices"
          value={overview?.totalInvoices ?? 0}
          icon={<FileText size={14} />}
          color="bg-blue-50 text-blue-600"
        />
        <StatCard
          label="Payments"
          value={overview?.totalPayments ?? 0}
          icon={<CreditCard size={14} />}
          color="bg-green-50 text-green-600"
        />
        <StatCard
          label="Subscriptions"
          value={overview?.activeSubscriptions ?? 0}
          icon={<Globe size={14} />}
          color="bg-purple-50 text-purple-600"
        />
      </div>

      {/* Personal Profile */}
      {overview?.personalProfile && (
        <section>
          <h2 className="text-sm font-medium text-faint uppercase tracking-wider mb-3">
            Personal Profile
          </h2>
          <ProfileCard
            profile={overview.personalProfile}
            type="personal"
            onClick={() =>
              navigate(`/personal-profile/${overview.personalProfile!.nidAccountId}`)
            }
          />
        </section>
      )}

      {/* Work Profiles */}
      {overview?.workProfiles && overview.workProfiles.length > 0 && (
        <section>
          <h2 className="text-sm font-medium text-faint uppercase tracking-wider mb-3">
            Work Profiles
          </h2>
          <div className="space-y-3">
            {overview.workProfiles.map((profile) => (
              <ProfileCard
                key={profile.id}
                profile={profile}
                type="work"
                onClick={() =>
                  navigate(
                    `/work-profile/${profile.nidOrganizationId}`
                  )
                }
              />
            ))}
          </div>
        </section>
      )}

      {/* Organizations */}
      {overview?.organizations && overview.organizations.length > 0 && (
        <section>
          <h2 className="text-sm font-medium text-faint uppercase tracking-wider mb-3">
            Organizations
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {overview.organizations.map((org) => (
              <OrganizationCard
                key={org.id}
                org={org}
                onClick={() => navigate(`/work-profile/${org.nidOrgId}`)}
              />
            ))}
          </div>
        </section>
      )}

      {/* Tenants */}
      {overview?.tenants && overview.tenants.length > 0 && (
        <section>
          <h2 className="text-sm font-medium text-faint uppercase tracking-wider mb-3">
            Tenants
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {overview.tenants.map((tenant) => (
              <div
                key={tenant.id}
                className="bg-white border border-border rounded-8px p-4"
              >
                <div className="flex items-center gap-2">
                  <Globe size={14} className="text-accent" />
                  <p className="font-medium text-sm">{tenant.nidTenantName}</p>
                </div>
                <p className="text-xs text-faint mt-1">{tenant.nidTenantSlug}</p>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Empty State */}
      {!overview?.personalProfile &&
        (!overview?.workProfiles || overview.workProfiles.length === 0) &&
        (!overview?.organizations || overview.organizations.length === 0) && (
          <div className="bg-white border border-border rounded-8px p-12 text-center">
            <User size={32} className="mx-auto text-faint mb-4" />
            <h3 className="font-semibold mb-1">No profiles linked yet</h3>
            <p className="text-sm text-muted max-w-md mx-auto">
              Link your nID account to see your personal and work profiles, organizations, and
              assets here. Sign in with nID to get started.
            </p>
          </div>
        )}
    </div>
  )
}
