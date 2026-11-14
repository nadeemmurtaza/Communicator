import { useEffect, useState } from 'react'
import { api } from '../lib/api'

interface AccessGrant {
  id: string
  team_member_id: string
  tenant_id: string
  org_id: string | null
  brand_id: string | null
  access_type: string
  partnership_percentage: number | null
  granted_by: string
  status: string
  granted_at: string
}

interface Department {
  id: string
  tenant_id: string
  name: string
  slug: string
  default_permissions: any
  icon: string | null
  color: string | null
  inherited_from_tenant: boolean
  created_at: string
}

interface RoleDefinition {
  id: string
  tenant_id: string
  name: string
  slug: string
  description: string | null
  permissions: string[]
  is_built_in: boolean
  created_at: string
}

interface PermissionCatalogGroup {
  group_name: string
  services: {
    service_name: string
    permissions: {
      id: string
      group_name: string
      service_name: string
      permission_key: string
      label: string
      description: string | null
    }[]
  }[]
}

interface Invitation {
  id: string
  tenant_id: string
  email: string | null
  username: string | null
  invited_by: string
  scope: string
  status: string
  expires_at: string
  created_at: string
}

interface EffectivePermissions {
  team_member_id: string
  tenant_id: string
  permissions: string[]
  access_type: string
  roles: string[]
  departments: string[]
}

export function AccessManagement() {
  const [activeTab, setActiveTab] = useState<'grants' | 'departments' | 'roles' | 'invitations' | 'my-permissions'>('grants')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(false)
  }, [])

  if (loading) return <div className="page-loading">Loading access management…</div>
  if (error) return <div className="page-error">{error}</div>

  return (
    <div className="page">
      <div className="page-header">
        <h1>Access Management</h1>
        <p className="page-subtitle">Three-layer hierarchical access control: entity grants, departments, and custom roles</p>
      </div>

      <div className="tab-nav">
        <button
          className={activeTab === 'grants' ? 'active' : ''}
          onClick={() => setActiveTab('grants')}
        >
          Access Grants
        </button>
        <button
          className={activeTab === 'departments' ? 'active' : ''}
          onClick={() => setActiveTab('departments')}
        >
          Departments
        </button>
        <button
          className={activeTab === 'roles' ? 'active' : ''}
          onClick={() => setActiveTab('roles')}
        >
          Roles & Permissions
        </button>
        <button
          className={activeTab === 'invitations' ? 'active' : ''}
          onClick={() => setActiveTab('invitations')}
        >
          Invitations
        </button>
        <button
          className={activeTab === 'my-permissions' ? 'active' : ''}
          onClick={() => setActiveTab('my-permissions')}
        >
          My Permissions
        </button>
      </div>

      <div className="tab-content">
        {activeTab === 'grants' && <AccessGrantsTab />}
        {activeTab === 'departments' && <DepartmentsTab />}
        {activeTab === 'roles' && <RolesTab />}
        {activeTab === 'invitations' && <InvitationsTab />}
        {activeTab === 'my-permissions' && <MyPermissionsTab />}
      </div>
    </div>
  )
}

function AccessGrantsTab() {
  const [grants, setGrants] = useState<AccessGrant[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getAccessGrants().then(setGrants).finally(() => setLoading(false))
  }, [])

  if (loading) return <div>Loading grants…</div>

  return (
    <div>
      <h2>Access Grants</h2>
      <table className="data-table">
        <thead>
          <tr>
            <th>Member ID</th>
            <th>Access Type</th>
            <th>Partnership %</th>
            <th>Status</th>
            <th>Granted</th>
          </tr>
        </thead>
        <tbody>
          {grants.map(grant => (
            <tr key={grant.id}>
              <td>{grant.team_member_id.slice(0, 8)}</td>
              <td>
                <span className={`badge badge-${grant.access_type}`}>
                  {grant.access_type}
                </span>
              </td>
              <td>{grant.partnership_percentage ? `${grant.partnership_percentage}%` : '—'}</td>
              <td>
                <span className={`badge badge-${grant.status}`}>
                  {grant.status}
                </span>
              </td>
              <td>{new Date(grant.granted_at).toLocaleDateString()}</td>
            </tr>
          ))}
          {grants.length === 0 && (
            <tr><td colSpan={5}>No access grants yet.</td></tr>
          )}
        </tbody>
      </table>
    </div>
  )
}

function DepartmentsTab() {
  const [departments, setDepartments] = useState<Department[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getDepartments().then(setDepartments).finally(() => setLoading(false))
  }, [])

  if (loading) return <div>Loading departments…</div>

  return (
    <div>
      <h2>Departments</h2>
      <div className="card-grid">
        {departments.map(dept => (
          <div key={dept.id} className="card">
            <div className="card-icon" style={{ color: dept.color || '#3b82f6' }}>
              {dept.icon || '📁'}
            </div>
            <h3>{dept.name}</h3>
            <p className="text-muted">{dept.slug}</p>
            {dept.inherited_from_tenant && (
              <span className="badge badge-info">Inherited</span>
            )}
          </div>
        ))}
        {departments.length === 0 && <p>No departments yet.</p>}
      </div>
    </div>
  )
}

function RolesTab() {
  const [roles, setRoles] = useState<RoleDefinition[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getRoles().then(setRoles).finally(() => setLoading(false))
  }, [])

  if (loading) return <div>Loading roles…</div>

  return (
    <div>
      <h2>Roles & Permissions</h2>
      <div className="card-grid">
        {roles.map(role => (
          <div key={role.id} className="card">
            {role.is_built_in && <span className="badge badge-info">Built-in</span>}
            <h3>{role.name}</h3>
            <p className="text-muted">{role.slug}</p>
            {role.description && <p>{role.description}</p>}
            <p className="text-muted">{role.permissions.length} permissions</p>
          </div>
        ))}
        {roles.length === 0 && <p>No roles yet.</p>}
      </div>
    </div>
  )
}

function InvitationsTab() {
  const [invitations, setInvitations] = useState<Invitation[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getInvitations().then(setInvitations).finally(() => setLoading(false))
  }, [])

  if (loading) return <div>Loading invitations…</div>

  return (
    <div>
      <h2>Invitations</h2>
      <table className="data-table">
        <thead>
          <tr>
            <th>Email / Username</th>
            <th>Scope</th>
            <th>Status</th>
            <th>Expires</th>
          </tr>
        </thead>
        <tbody>
          {invitations.map(inv => (
            <tr key={inv.id}>
              <td>{inv.email || inv.username || '—'}</td>
              <td><span className="badge badge-info">{inv.scope}</span></td>
              <td><span className={`badge badge-${inv.status}`}>{inv.status}</span></td>
              <td>{new Date(inv.expires_at).toLocaleDateString()}</td>
            </tr>
          ))}
          {invitations.length === 0 && (
            <tr><td colSpan={4}>No invitations yet.</td></tr>
          )}
        </tbody>
      </table>
    </div>
  )
}

function MyPermissionsTab() {
  const [perms, setPerms] = useState<EffectivePermissions | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.getMyPermissions().then(setPerms).finally(() => setLoading(false))
  }, [])

  if (loading) return <div>Loading your permissions…</div>

  return (
    <div>
      <h2>My Effective Permissions</h2>
      {perms && (
        <div className="info-card">
          <div className="info-row">
            <span className="label">Access Type</span>
            <span className="value">
              <span className={`badge badge-${perms.access_type}`}>
                {perms.access_type}
              </span>
            </span>
          </div>
          <div className="info-row">
            <span className="label">Roles</span>
            <span className="value">
              {perms.roles.map(r => <span key={r} className="badge badge-info">{r}</span>)}
            </span>
          </div>
          <div className="info-row">
            <span className="label">Departments</span>
            <span className="value">
              {perms.departments.map(d => <span key={d} className="badge badge-success">{d}</span>)}
            </span>
          </div>
          <div className="info-row">
            <span className="label">Permissions</span>
            <span className="value">{perms.permissions.length} total</span>
          </div>
        </div>
      )}
    </div>
  )
}
