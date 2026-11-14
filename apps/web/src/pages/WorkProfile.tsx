import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  getWorkProfileOverview,
  type WorkProfileOverview,
  type BrandSummary,
  type NidOrganization,
} from '../lib/api'
import {
  Building2,
  Briefcase,
  Users,
  ShoppingCart,
  TrendingUp,
  Phone,
  CheckCircle,
  AlertCircle,
  ArrowRight,
  Store,
} from '../components/icons'

function StatusBadge({ status }: { status: string }) {
  const s = status.toLowerCase()
  let colors = 'text-muted bg-accent-muted'
  if (['active', 'paid', 'succeeded'].includes(s)) colors = 'text-green-600 bg-green-50'
  else if (['pending', 'processing'].includes(s)) colors = 'text-amber-600 bg-amber-50'
  else if (['failed', 'overdue', 'void', 'canceled'].includes(s)) colors = 'text-red-600 bg-red-50'

  return (
    <span className={`text-xs font-medium px-2 py-0.5 rounded-4px ${colors}`}>
      {status}
    </span>
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

function BrandCard({
  brand,
  onClick,
}: {
  brand: BrandSummary
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className="w-full text-left bg-white border border-border rounded-8px p-5 hover:border-accent transition-colors group"
    >
      <div className="flex items-start gap-3">
        <div className="w-10 h-10 rounded-4px bg-black text-white flex items-center justify-center flex-shrink-0">
          <Store size={16} />
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-semibold text-sm truncate">{brand.name}</p>
          <p className="text-xs text-faint mt-0.5">{brand.slug}</p>
          <div className="grid grid-cols-3 gap-2 mt-3">
            <div>
              <p className="text-xs text-faint">Orders</p>
              <p className="text-sm font-semibold">{brand.totalOrders}</p>
            </div>
            <div>
              <p className="text-xs text-faint">Sales</p>
              <p className="text-sm font-semibold">
                {brand.totalSales.toLocaleString('en-US', {
                  style: 'currency',
                  currency: 'USD',
                  maximumFractionDigits: 0,
                })}
              </p>
            </div>
            <div>
              <p className="text-xs text-faint">Customers</p>
              <p className="text-sm font-semibold">{brand.totalCustomers}</p>
            </div>
          </div>
        </div>
        <ArrowRight
          size={14}
          className="text-faint group-hover:text-accent transition-colors flex-shrink-0 mt-1"
        />
      </div>
    </button>
  )
}

export function WorkProfile() {
  const { orgId } = useParams<{ orgId: string }>()
  const navigate = useNavigate()
  const [overview, setOverview] = useState<WorkProfileOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function load() {
      if (!orgId) return
      try {
        const session = localStorage.getItem('communicator.session')
        const teamMemberId = session ? JSON.parse(session).userId || 'me' : 'me'
        const data = await getWorkProfileOverview(teamMemberId, orgId)
        setOverview(data)
      } catch (err: any) {
        setError(err.message || 'Failed to load work profile')
      } finally {
        setLoading(false)
      }
    }
    load()
  }, [orgId])

  if (loading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-xl font-semibold">Work Profile</h1>
          <p className="text-sm text-muted mt-0.5">Loading organization data...</p>
        </div>
        <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className="bg-white border border-border rounded-8px p-5 animate-pulse">
              <div className="h-4 bg-accent-muted rounded w-1/2 mb-3" />
              <div className="h-6 bg-accent-muted rounded w-1/3" />
            </div>
          ))}
        </div>
      </div>
    )
  }

  if (error || !overview) {
    return (
      <div className="space-y-6">
        <h1 className="text-xl font-semibold">Work Profile</h1>
        <div className="bg-white border border-border rounded-8px p-8 text-center">
          <AlertCircle size={24} className="mx-auto text-red-600 mb-3" />
          <p className="text-sm text-muted">{error || 'Organization not found'}</p>
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

  const { organization, brands, totalCustomers, totalOrders, totalSales, totalContacts, activeSubscriptions } = overview

  return (
    <div className="space-y-6">
      {/* Organization Header */}
      <div className="bg-white border border-border rounded-8px p-6">
        <div className="flex items-start gap-4">
          <div className="w-14 h-14 rounded-8px bg-black text-white flex items-center justify-center flex-shrink-0">
            <Building2 size={24} />
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-semibold">{organization.nidLegalName}</h1>
              <span className="text-[10px] font-medium uppercase tracking-wider px-2 py-0.5 rounded-4px bg-black text-white">
                Work
              </span>
            </div>
            <div className="flex flex-wrap items-center gap-3 mt-2 text-xs text-faint">
              {organization.isVerified ? (
                <span className="flex items-center gap-1 text-green-600">
                  <CheckCircle size={12} /> Verified
                </span>
              ) : (
                <span className="flex items-center gap-1 text-amber-600">
                  <AlertCircle size={12} /> Pending Verification
                </span>
              )}
              {organization.nidCountryCode && (
                <>
                  <span>·</span>
                  <span>{organization.nidCountryCode}</span>
                </>
              )}
              {organization.nidRegistrationNumber && (
                <>
                  <span>·</span>
                  <span>Reg: {organization.nidRegistrationNumber}</span>
                </>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Stats Row */}
      <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
        <StatCard
          label="Customers"
          value={totalCustomers}
          icon={<Users size={14} />}
          color="bg-accent-muted text-accent"
        />
        <StatCard
          label="Orders"
          value={totalOrders}
          icon={<ShoppingCart size={14} />}
          color="bg-blue-50 text-blue-600"
        />
        <StatCard
          label="Total Sales"
          value={totalSales.toLocaleString('en-US', {
            style: 'currency',
            currency: 'USD',
            maximumFractionDigits: 0,
          })}
          icon={<TrendingUp size={14} />}
          color="bg-green-50 text-green-600"
        />
        <StatCard
          label="Contacts"
          value={totalContacts}
          icon={<Phone size={14} />}
          color="bg-purple-50 text-purple-600"
        />
        <StatCard
          label="Subscriptions"
          value={activeSubscriptions}
          icon={<Briefcase size={14} />}
          color="bg-amber-50 text-amber-600"
        />
      </div>

      {/* Brands Section */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-faint uppercase tracking-wider">
            Brands ({brands.length})
          </h2>
          <button
            onClick={() => navigate('/brands')}
            className="text-xs text-accent hover:underline"
          >
            View All →
          </button>
        </div>
        {brands.length === 0 ? (
          <div className="bg-white border border-border rounded-8px p-8 text-center">
            <Store size={24} className="mx-auto text-faint mb-3" />
            <p className="text-sm text-muted">No brands configured for this organization</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {brands.map((brand) => (
              <BrandCard
                key={brand.id}
                brand={brand}
                onClick={() => navigate(`/brands/${brand.id}`)}
              />
            ))}
          </div>
        )}
      </section>

      {/* Quick Links */}
      <section>
        <h2 className="text-sm font-medium text-faint uppercase tracking-wider mb-3">
          Quick Access
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <button
            onClick={() => navigate('/conversations')}
            className="w-full text-left bg-white border border-border rounded-8px p-4 hover:border-accent transition-colors group"
          >
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-4px bg-accent-muted text-accent flex items-center justify-center">
                <Phone size={14} />
              </div>
              <div>
                <p className="font-medium text-sm">Conversations</p>
                <p className="text-xs text-faint">View all customer conversations</p>
              </div>
              <ArrowRight
                size={14}
                className="text-faint group-hover:text-accent transition-colors ml-auto"
              />
            </div>
          </button>
          <button
            onClick={() => navigate('/contacts')}
            className="w-full text-left bg-white border border-border rounded-8px p-4 hover:border-accent transition-colors group"
          >
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-4px bg-blue-50 text-blue-600 flex items-center justify-center">
                <Users size={14} />
              </div>
              <div>
                <p className="font-medium text-sm">Contacts</p>
                <p className="text-xs text-faint">Manage customer contacts</p>
              </div>
              <ArrowRight
                size={14}
                className="text-faint group-hover:text-accent transition-colors ml-auto"
              />
            </div>
          </button>
          <button
            onClick={() => navigate('/commerce')}
            className="w-full text-left bg-white border border-border rounded-8px p-4 hover:border-accent transition-colors group"
          >
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-4px bg-green-50 text-green-600 flex items-center justify-center">
                <ShoppingCart size={14} />
              </div>
              <div>
                <p className="font-medium text-sm">Commerce</p>
                <p className="text-xs text-faint">Orders, products, payments</p>
              </div>
              <ArrowRight
                size={14}
                className="text-faint group-hover:text-accent transition-colors ml-auto"
              />
            </div>
          </button>
        </div>
      </section>
    </div>
  )
}
