import { useEffect, useState, useCallback } from 'react'
import { Link } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { api } from '../lib/api'
import {
  Loader2, Plus, Users, Search, AlertTriangle, X,
  User, Mail, Phone, Building2, Star, TrendingUp,
  ChevronRight
} from '../components/icons'

type ClientTier = 'all' | 'bronze' | 'silver' | 'gold' | 'platinum'

export function ClientProfiles() {
  useAuth()
  const [clients, setClients] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [tierFilter, setTierFilter] = useState<ClientTier>('all')
  const [sortBy, setSortBy] = useState<'name' | 'value' | 'orders' | 'recent'>('name')

  useEffect(() => {
    loadClients()
  }, [])

  const loadClients = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      // Fetch contacts as clients — the backend resolves unified customer IDs
      const data = await api.listContacts({ search: '' })
      setClients(Array.isArray(data) ? data.map((c: any) => ({
        ...c,
        tags: Array.isArray(c.tags) ? JSON.stringify(c.tags) : (c.tags || ''),
      })) : [])
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }, [])

  const filteredClients = clients
    .filter(c => {
      const name = `${c.firstName || ''} ${c.lastName || ''} ${c.displayName || ''}`.toLowerCase()
      const matchesSearch = !search ||
        name.includes(search.toLowerCase()) ||
        (c.email || '').toLowerCase().includes(search.toLowerCase()) ||
        (c.phone || '').toLowerCase().includes(search.toLowerCase()) ||
        (c.company || '').toLowerCase().includes(search.toLowerCase())
      const matchesTier = tierFilter === 'all' || (c.customerTier || '') === tierFilter
      return matchesSearch && matchesTier
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'value': return (b.lifetimeValue || 0) - (a.lifetimeValue || 0)
        case 'orders': return (b.totalOrders || 0) - (a.totalOrders || 0)
        case 'recent': return (b.lastOrderDate || '').localeCompare(a.lastOrderDate || '')
        default: return `${a.firstName || ''} ${a.lastName || ''}`.localeCompare(`${b.firstName || ''} ${b.lastName || ''}`)
      }
    })

  // Stats
  const totalClients = clients.length
  const totalValue = clients.reduce((sum, c) => sum + (c.lifetimeValue || 0), 0)
  const avgEngagement = clients.length
    ? (clients.reduce((sum, c) => sum + (c.engagementScore || 0), 0) / clients.length).toFixed(1)
    : '0'

  const tierColors: Record<string, string> = {
    platinum: 'bg-purple-100 text-purple-700 border-purple-200',
    gold: 'bg-yellow-100 text-yellow-700 border-yellow-200',
    silver: 'bg-gray-100 text-gray-700 border-gray-200',
    bronze: 'bg-orange-100 text-orange-700 border-orange-200',
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Client Profiles</h1>
          <p className="text-muted-foreground text-sm mt-1">
            Unified customer data across all channels with AI learning insights
          </p>
        </div>
        <Link
          to="/contacts"
          className="flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-lg text-sm hover:opacity-90"
        >
          <Plus className="w-4 h-4" />
          Add Contact
        </Link>
      </div>

      {/* Error */}
      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span className="flex-1">{error}</span>
          <button onClick={() => setError(null)}><X className="w-4 h-4" /></button>
        </div>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="rounded-xl border p-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-blue-50 flex items-center justify-center">
              <Users className="w-5 h-5 text-blue-600" />
            </div>
            <div>
              <p className="text-2xl font-bold">{totalClients}</p>
              <p className="text-xs text-muted-foreground">Total Clients</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border p-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-green-50 flex items-center justify-center">
              <TrendingUp className="w-5 h-5 text-green-600" />
            </div>
            <div>
              <p className="text-2xl font-bold">${totalValue.toLocaleString()}</p>
              <p className="text-xs text-muted-foreground">Total Lifetime Value</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border p-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-purple-50 flex items-center justify-center">
              <Star className="w-5 h-5 text-purple-600" />
            </div>
            <div>
              <p className="text-2xl font-bold">{avgEngagement}</p>
              <p className="text-xs text-muted-foreground">Avg Engagement Score</p>
            </div>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-col md:flex-row gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search by name, email, phone, or company..."
            className="w-full pl-10 pr-4 py-2 border rounded-lg text-sm"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="flex items-center gap-2">
          <Search className="w-4 h-4 text-muted-foreground" />
          <select
            className="px-3 py-2 border rounded-lg text-sm"
            value={tierFilter}
            onChange={(e) => setTierFilter(e.target.value as ClientTier)}
          >
            <option value="all">All Tiers</option>
            <option value="platinum">Platinum</option>
            <option value="gold">Gold</option>
            <option value="silver">Silver</option>
            <option value="bronze">Bronze</option>
          </select>
          <select
            className="px-3 py-2 border rounded-lg text-sm"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
          >
            <option value="name">Sort by Name</option>
            <option value="value">Sort by Value</option>
            <option value="orders">Sort by Orders</option>
            <option value="recent">Sort by Recent</option>
          </select>
        </div>
      </div>

      {/* Loading */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
          <span className="ml-2 text-muted-foreground">Loading clients...</span>
        </div>
      )}

      {/* Empty State */}
      {!loading && filteredClients.length === 0 && (
        <div className="text-center py-12">
          <Users className="w-12 h-12 mx-auto text-muted-foreground/50" />
          <h3 className="mt-4 text-lg font-medium">No clients found</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            {search ? 'Try adjusting your search or filters.' : 'Start by adding contacts from your channels.'}
          </p>
          {!search && (
            <Link
              to="/contacts"
              className="mt-4 inline-flex items-center gap-2 px-4 py-2 bg-foreground text-background rounded-lg text-sm hover:opacity-90"
            >
              <Plus className="w-4 h-4" />
              Add Contact
            </Link>
          )}
        </div>
      )}

      {/* Client List */}
      {!loading && filteredClients.length > 0 && (
        <div className="space-y-2">
          {/* Table Header */}
          <div className="hidden md:grid grid-cols-12 gap-4 px-4 py-2 text-xs font-medium text-muted-foreground uppercase tracking-wider">
            <div className="col-span-4">Client</div>
            <div className="col-span-2">Company</div>
            <div className="col-span-1">Tier</div>
            <div className="col-span-2">Lifetime Value</div>
            <div className="col-span-1">Orders</div>
            <div className="col-span-1">Engagement</div>
            <div className="col-span-1"></div>
          </div>

          {/* Client Rows */}
          {filteredClients.map((client) => {
            const name = `${client.firstName || ''} ${client.lastName || ''}`.trim() || client.displayName || 'Unknown'
            const tier = client.customerTier || 'bronze'
            const tierColor = tierColors[tier] || tierColors.bronze

            return (
              <Link
                key={client.id}
                to={`/clients/${client.id}`}
                className="grid grid-cols-1 md:grid-cols-12 gap-2 md:gap-4 px-4 py-3 rounded-xl border hover:border-foreground/30 hover:bg-muted/50 transition-all items-center group"
              >
                {/* Client Info */}
                <div className="md:col-span-4 flex items-center gap-3">
                  <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center shrink-0">
                    <User className="w-5 h-5 text-muted-foreground" />
                  </div>
                  <div className="min-w-0">
                    <p className="font-medium truncate">{name}</p>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      {client.email && (
                        <span className="flex items-center gap-1">
                          <Mail className="w-3 h-3" />
                          <span className="truncate max-w-[120px]">{client.email}</span>
                        </span>
                      )}
                      {client.phone && (
                        <span className="flex items-center gap-1">
                          <Phone className="w-3 h-3" />
                          <span className="truncate max-w-[80px]">{client.phone}</span>
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                {/* Company */}
                <div className="md:col-span-2 flex items-center gap-1 text-sm text-muted-foreground">
                  {client.company ? (
                    <>
                      <Building2 className="w-3 h-3 shrink-0" />
                      <span className="truncate">{client.company}</span>
                    </>
                  ) : (
                    <span className="text-muted-foreground/50">—</span>
                  )}
                </div>

                {/* Tier */}
                <div className="md:col-span-1">
                  <span className={`inline-block px-2 py-0.5 text-xs font-medium rounded-full border ${tierColor}`}>
                    {tier}
                  </span>
                </div>

                {/* Lifetime Value */}
                <div className="md:col-span-2 text-sm">
                  {client.lifetimeValue ? (
                    <span className="font-medium">${client.lifetimeValue.toLocaleString()}</span>
                  ) : (
                    <span className="text-muted-foreground/50">—</span>
                  )}
                </div>

                {/* Orders */}
                <div className="md:col-span-1 text-sm">
                  {client.totalOrders ? (
                    <span>{client.totalOrders}</span>
                  ) : (
                    <span className="text-muted-foreground/50">—</span>
                  )}
                </div>

                {/* Engagement */}
                <div className="md:col-span-1">
                  {client.engagementScore != null ? (
                    <div className="flex items-center gap-1">
                      <div className="w-8 h-1.5 bg-muted rounded-full overflow-hidden">
                        <div
                          className="h-full bg-green-500 rounded-full"
                          style={{ width: `${Math.min(client.engagementScore * 10, 100)}%` }}
                        />
                      </div>
                      <span className="text-xs text-muted-foreground">{client.engagementScore.toFixed(0)}%</span>
                    </div>
                  ) : (
                    <span className="text-muted-foreground/50">—</span>
                  )}
                </div>

                {/* Action */}
                <div className="md:col-span-1 flex justify-end">
                  <ChevronRight className="w-4 h-4 text-muted-foreground group-hover:text-foreground transition-colors" />
                </div>
              </Link>
            )
          })}
        </div>
      )}

      {/* Results count */}
      {!loading && filteredClients.length > 0 && (
        <p className="text-xs text-muted-foreground text-center">
          Showing {filteredClients.length} of {totalClients} clients
        </p>
      )}
    </div>
  )
}
