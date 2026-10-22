import { useEffect, useState } from 'react'
import {
  api,
  type PermissionCatalogItem,
  type RoleItem,
  type SSOConfigItem,
  type SSOTestResult,
  type TeamMember,
} from '../lib/api'

export function Team() {
  const [activeTab, setActiveTab] = useState<'members' | 'roles' | 'sso'>('members')

  // SSO State
  const [ssoConfig, setSsoConfig] = useState<SSOConfigItem | null>(null)
  const [ssoProvider, setSsoProvider] = useState<'saml' | 'okta' | 'azure_ad' | 'google_workspace' | 'onelogin'>('saml')
  const [ssoEnabled, setSsoEnabled] = useState(false)
  const [idpEntityId, setIdpEntityId] = useState('')
  const [idpSsoUrl, setIdpSsoUrl] = useState('')
  const [idpCert, setIdpCert] = useState('')
  const [spEntityId, setSpEntityId] = useState('urn:communicator:sp')
  const [spAcsUrl, setSpAcsUrl] = useState('/api/v1/sso/saml/acs')
  const [enforcedDomainsText, setEnforcedDomainsText] = useState('')
  const [jitEnabled, setJitEnabled] = useState(true)
  const [defaultSsoRole, setDefaultSsoRole] = useState('agent')
  const [allowUnlinkedPasswords, setAllowUnlinkedPasswords] = useState(true)
  const [savingSso, setSavingSso] = useState(false)
  const [testingSso, setTestingSso] = useState(false)
  const [ssoTestResult, setSsoTestResult] = useState<SSOTestResult | null>(null)
  const [showTestModal, setShowTestModal] = useState(false)
  const [copiedField, setCopiedField] = useState<string | null>(null)
  const [members, setMembers] = useState<TeamMember[] | null>(null)
  const [roles, setRoles] = useState<RoleItem[]>([])
  const [catalog, setCatalog] = useState<PermissionCatalogItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // New Member Form
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const [memberRole, setMemberRole] = useState('agent')
  const [savingMember, setSavingMember] = useState(false)

  // Role Modal State
  const [showRoleModal, setShowRoleModal] = useState(false)
  const [editingRole, setEditingRole] = useState<RoleItem | null>(null)
  const [roleSlug, setRoleSlug] = useState('')
  const [roleDisplayName, setRoleDisplayName] = useState('')
  const [roleDesc, setRoleDesc] = useState('')
  const [selectedPerms, setSelectedPerms] = useState<string[]>([])
  const [savingRole, setSavingRole] = useState(false)

  // Member Permission Inspection Modal
  const [inspectingMember, setInspectingMember] = useState<TeamMember | null>(null)

  useEffect(() => {
    loadAll()
  }, [])

  async function loadAll() {
    try {
      setLoading(true)
      setError(null)
      const [mList, rList, pCatalog, sso] = await Promise.all([
        api.listTeam(),
        api.getRoles().catch(() => []),
        api.getPermissionsCatalog().catch(() => []),
        api.getSSOConfig().catch(() => null),
      ])
      setMembers(mList)
      setRoles(rList)
      setCatalog(pCatalog)
      if (sso) {
        setSsoConfig(sso)
        setSsoProvider(sso.provider_type)
        setSsoEnabled(sso.is_enabled)
        setIdpEntityId(sso.idp_entity_id)
        setIdpSsoUrl(sso.idp_sso_url)
        setIdpCert(sso.idp_x509_cert)
        setSpEntityId(sso.sp_entity_id)
        setSpAcsUrl(sso.sp_acs_url)
        setEnforcedDomainsText(sso.enforced_domains.join(', '))
        setJitEnabled(sso.jit_provisioning_enabled)
        setDefaultSsoRole(sso.default_role || 'agent')
        setAllowUnlinkedPasswords(sso.allow_unlinked_passwords)
      }
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveSSO(e: React.FormEvent) {
    e.preventDefault()
    setSavingSso(true)
    setError(null)
    try {
      const domains = enforcedDomainsText
        .split(',')
        .map((d) => d.trim().replace(/^@/, '').toLowerCase())
        .filter(Boolean)

      const updated = await api.updateSSOConfig({
        is_enabled: ssoEnabled,
        provider_type: ssoProvider,
        idp_entity_id: idpEntityId.trim(),
        idp_sso_url: idpSsoUrl.trim(),
        idp_x509_cert: idpCert.trim(),
        sp_entity_id: spEntityId.trim(),
        sp_acs_url: spAcsUrl.trim(),
        enforced_domains: domains,
        jit_provisioning_enabled: jitEnabled,
        default_role: defaultSsoRole,
        allow_unlinked_passwords: allowUnlinkedPasswords,
      })

      setSsoConfig(updated)
      setSuccess('Enterprise SSO & SAML 2.0 configuration saved and logged to SOC 2 compliance trail!')
      setTimeout(() => setSuccess(null), 3500)
    } catch (err) {
      setError((err as Error).message)
    } finally {
      setSavingSso(false)
    }
  }

  async function handleRunSSOTest() {
    setTestingSso(true)
    setError(null)
    try {
      const result = await api.testSSOConnection()
      setSsoTestResult(result)
      setShowTestModal(true)
      if (result.status === 'healthy') {
        setSuccess('All SAML 2.0 connectivity and assertion parsing checks passed successfully!')
      } else {
        setError(result.message)
      }
      setTimeout(() => {
        setSuccess(null)
        setError(null)
      }, 4000)
    } catch (err) {
      setError((err as Error).message)
    } finally {
      setTestingSso(false)
    }
  }

  function handleCopy(text: string, fieldName: string) {
    navigator.clipboard.writeText(text)
    setCopiedField(fieldName)
    setTimeout(() => setCopiedField(null), 2000)
  }

  async function submitMember(e: React.FormEvent) {
    e.preventDefault()
    setSavingMember(true)
    setError(null)
    try {
      const created = await api.createTeamMember({ name, email })
      if (memberRole !== 'agent') {
        await api.assignMemberRole(created.id, { role: memberRole })
      }
      setSuccess(`Team member ${name} added successfully!`)
      setName('')
      setEmail('')
      setMemberRole('agent')
      setTimeout(() => setSuccess(null), 3000)
      const updatedList = await api.listTeam()
      setMembers(updatedList)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSavingMember(false)
    }
  }

  async function handleRoleChange(memberId: string, newRole: string) {
    try {
      await api.assignMemberRole(memberId, { role: newRole })
      setMembers((prev) =>
        prev
          ? prev.map((m) => (m.id === memberId ? { ...m, role: newRole } : m))
          : null
      )
      setSuccess('Role updated and recorded in SOC 2 compliance trail!')
      setTimeout(() => setSuccess(null), 3000)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function removeMember(id: string) {
    if (!window.confirm('Are you sure you want to remove this team member?')) return
    try {
      await api.deleteTeamMember(id)
      setMembers((prev) => prev?.filter((m) => m.id !== id) ?? null)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function openCreateRoleModal() {
    setEditingRole(null)
    setRoleSlug('')
    setRoleDisplayName('')
    setRoleDesc('')
    setSelectedPerms([
      'conversations:view',
      'conversations:reply',
      'contacts:view',
      'inboxes:view',
    ])
    setShowRoleModal(true)
  }

  function openEditRoleModal(role: RoleItem) {
    setEditingRole(role)
    setRoleSlug(role.name)
    setRoleDisplayName(role.display_name)
    setRoleDesc(role.description)
    setSelectedPerms([...role.permissions])
    setShowRoleModal(true)
  }

  async function handleSaveRole(e: React.FormEvent) {
    e.preventDefault()
    setSavingRole(true)
    setError(null)
    try {
      if (editingRole) {
        await api.updateRole(editingRole.id, {
          display_name: roleDisplayName,
          description: roleDesc,
          permissions: selectedPerms,
        })
        setSuccess(`Role '${roleDisplayName}' updated successfully!`)
      } else {
        await api.createRole({
          name: roleSlug,
          display_name: roleDisplayName,
          description: roleDesc,
          permissions: selectedPerms,
        })
        setSuccess(`Custom role '${roleDisplayName}' created successfully!`)
      }
      setShowRoleModal(false)
      setTimeout(() => setSuccess(null), 3500)
      const rList = await api.getRoles()
      setRoles(rList)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSavingRole(false)
    }
  }

  async function handleDeleteRole(roleName: string) {
    if (!window.confirm(`Delete role '${roleName}'? Any assigned members will be migrated to the default Agent role.`)) {
      return
    }
    try {
      await api.deleteRole(roleName)
      setSuccess(`Role '${roleName}' deleted successfully!`)
      setTimeout(() => setSuccess(null), 3000)
      const [rList, mList] = await Promise.all([api.getRoles(), api.listTeam()])
      setRoles(rList)
      setMembers(mList)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function togglePermission(key: string) {
    setSelectedPerms((prev) =>
      prev.includes(key) ? prev.filter((p) => p !== key) : [...prev, key]
    )
  }

  // Group catalog items by category
  const categories = Array.from(new Set(catalog.map((c) => c.category)))

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-slate-200 pb-5">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-slate-900">
              Team & Role-Based Access Control (RBAC)
            </h1>
            <span className="inline-flex items-center gap-1 rounded-full bg-blue-50 px-2.5 py-0.5 text-xs font-semibold text-blue-700 border border-blue-200">
              Enterprise Security
            </span>
          </div>
          <p className="text-xs text-slate-500 mt-1">
            Manage agent capacity routing, enforce principle of least privilege, and define custom enterprise permissions matrices.
          </p>
        </div>

        {/* Tab Switcher */}
        <div className="flex items-center gap-1 rounded-lg bg-slate-200/80 p-1 text-xs font-semibold">
          <button
            onClick={() => setActiveTab('members')}
            className={`rounded-md px-3.5 py-1.5 transition-all ${
              activeTab === 'members'
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-slate-600 hover:text-slate-900'
            }`}
          >
            👥 Team Members ({members?.length ?? 0})
          </button>
          <button
            onClick={() => setActiveTab('roles')}
            className={`rounded-md px-3.5 py-1.5 transition-all ${
              activeTab === 'roles'
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-slate-600 hover:text-slate-900'
            }`}
          >
            🛡️ RBAC Roles ({roles.length})
          </button>
          <button
            onClick={() => setActiveTab('sso')}
            className={`rounded-md px-3.5 py-1.5 transition-all ${
              activeTab === 'sso'
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-slate-600 hover:text-slate-900'
            }`}
          >
            🔑 Enterprise SSO & SAML
          </button>
        </div>
      </div>

      {success && (
        <div className="rounded-lg border border-emerald-200 bg-emerald-50 p-3.5 text-xs text-emerald-800 flex items-center justify-between">
          <span>✅ {success}</span>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-900">✕</button>
        </div>
      )}

      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-3.5 text-xs text-red-800 flex items-center justify-between">
          <span>⚠️ {error}</span>
          <button onClick={() => setError(null)} className="text-red-600 hover:text-red-900">✕</button>
        </div>
      )}

      {/* TAB 1: TEAM MEMBERS */}
      {activeTab === 'members' && (
        <div className="space-y-6">
          {/* Add Team Member Card */}
          <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
            <h2 className="text-sm font-semibold text-slate-900">Invite New Team Member</h2>
            <form onSubmit={submitMember} className="grid grid-cols-1 sm:grid-cols-4 gap-3">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Full Name</label>
                <input
                  required
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. Jane Doe"
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Email Address</label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="jane@company.com"
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Assigned RBAC Role</label>
                <select
                  value={memberRole}
                  onChange={(e) => setMemberRole(e.target.value)}
                  className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-xs focus:border-blue-500 focus:outline-none capitalize"
                >
                  {roles.map((r) => (
                    <option key={r.name} value={r.name}>
                      {r.display_name} {r.is_system_role ? '(System)' : '(Custom)'}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex items-end">
                <button
                  type="submit"
                  disabled={savingMember || !name.trim()}
                  className="w-full rounded-lg bg-blue-600 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                >
                  {savingMember ? 'Adding...' : '+ Invite Member'}
                </button>
              </div>
            </form>
          </div>

          {/* Members Table */}
          <div className="rounded-xl border border-slate-200 bg-white shadow-sm overflow-hidden">
            {loading ? (
              <div className="p-12 text-center text-xs text-slate-500">
                <div className="inline-block h-6 w-6 animate-spin rounded-full border-2 border-solid border-blue-600 border-r-transparent"></div>
                <p className="mt-2 font-medium">Loading team directory...</p>
              </div>
            ) : !members || members.length === 0 ? (
              <div className="p-12 text-center text-xs text-slate-500">
                <span className="text-3xl">👥</span>
                <div className="font-semibold text-slate-800 mt-2">No Team Members Found</div>
                <p className="text-[11px] text-slate-400 mt-0.5">
                  Add team members above to route conversations to human specialists.
                </p>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left border-collapse text-xs">
                  <thead>
                    <tr className="border-b border-slate-200 bg-slate-50 text-[11px] font-semibold text-slate-600 uppercase tracking-wider">
                      <th className="py-3 px-4">Member</th>
                      <th className="py-3 px-4">Role & Permissions</th>
                      <th className="py-3 px-4">Status & Capacity</th>
                      <th className="py-3 px-4 text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {members.map((m) => {
                      const currentRoleName = m.role || 'agent'
                      const currentRoleObj = roles.find((r) => r.name === currentRoleName)

                      return (
                        <tr key={m.id} className="hover:bg-slate-50/60 transition-colors">
                          <td className="py-3.5 px-4 whitespace-nowrap">
                            <div className="flex items-center gap-3">
                              <div className="h-8 w-8 rounded-full bg-slate-900 text-white font-bold text-xs flex items-center justify-center shrink-0 uppercase">
                                {m.name.charAt(0) || 'U'}
                              </div>
                              <div>
                                <div className="font-semibold text-slate-900">{m.name}</div>
                                <div className="text-[11px] text-slate-500">{m.email || 'No email registered'}</div>
                              </div>
                            </div>
                          </td>

                          <td className="py-3.5 px-4 whitespace-nowrap">
                            <div className="flex items-center gap-2">
                              <select
                                value={currentRoleName}
                                onChange={(e) => handleRoleChange(m.id, e.target.value)}
                                className="rounded-md border border-slate-300 bg-white px-2.5 py-1 text-xs font-semibold text-slate-800 focus:border-blue-500 focus:outline-none"
                              >
                                {roles.map((r) => (
                                  <option key={r.name} value={r.name}>
                                    {r.display_name}
                                  </option>
                                ))}
                              </select>
                              <button
                                type="button"
                                onClick={() => setInspectingMember(m)}
                                className="rounded bg-slate-100 px-2 py-1 text-[10px] font-semibold text-slate-600 hover:bg-slate-200"
                              >
                                {currentRoleObj ? `${currentRoleObj.permissions_count} perms` : 'View Perms'}
                              </button>
                            </div>
                          </td>

                          <td className="py-3.5 px-4 whitespace-nowrap">
                            <div className="flex items-center gap-2">
                              <span className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-0.5 text-[10px] font-semibold text-emerald-700 border border-emerald-200">
                                <span className="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
                                {m.availability_status || 'online'}
                              </span>
                              <span className="text-[11px] text-slate-500">
                                Max {m.max_concurrent_conversations || 5} concurrent chats
                              </span>
                            </div>
                          </td>

                          <td className="py-3.5 px-4 text-right whitespace-nowrap">
                            <button
                              type="button"
                              onClick={() => removeMember(m.id)}
                              className="text-xs font-semibold text-red-600 hover:text-red-800"
                            >
                              Remove
                            </button>
                          </td>
                        </tr>
                      )
                    })}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}

      {/* TAB 2: RBAC ROLES & PERMISSIONS */}
      {activeTab === 'roles' && (
        <div className="space-y-6">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <h2 className="text-base font-semibold text-slate-900">Enterprise Roles Matrix</h2>
              <p className="text-xs text-slate-500 mt-0.5">
                Pre-configured system security roles plus customized privilege profiles.
              </p>
            </div>
            <button
              onClick={openCreateRoleModal}
              className="rounded-lg bg-blue-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow hover:bg-blue-700 flex items-center gap-1.5"
            >
              <span>+ Create Custom Role</span>
            </button>
          </div>

          {/* Roles Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {roles.map((role) => (
              <div
                key={role.id}
                className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm flex flex-col justify-between hover:border-slate-300 transition-all space-y-4"
              >
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <span
                      className={`rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider ${
                        role.is_system_role
                          ? 'bg-slate-100 text-slate-700 border border-slate-200'
                          : 'bg-purple-50 text-purple-700 border border-purple-200'
                      }`}
                    >
                      {role.is_system_role ? 'System Role' : 'Custom Role'}
                    </span>
                    <span className="text-[11px] font-semibold text-slate-500">
                      👥 {role.assigned_members_count} member{role.assigned_members_count === 1 ? '' : 's'}
                    </span>
                  </div>

                  <h3 className="text-sm font-bold text-slate-900">{role.display_name}</h3>
                  <code className="text-[10px] font-mono text-slate-400">slug: {role.name}</code>

                  <p className="text-xs text-slate-500 mt-2 line-clamp-2">
                    {role.description || 'No description provided.'}
                  </p>

                  {/* Permissions Preview */}
                  <div className="mt-3 pt-3 border-t border-slate-100 space-y-1.5">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="font-semibold text-slate-700">Privileges</span>
                      <span className="font-bold text-blue-600">{role.permissions_count} granted</span>
                    </div>
                    <div className="flex flex-wrap gap-1">
                      {role.permissions.slice(0, 4).map((p) => (
                        <span key={p} className="rounded bg-slate-50 border border-slate-200 px-1.5 py-0.2 text-[10px] font-mono text-slate-600">
                          {p}
                        </span>
                      ))}
                      {role.permissions.length > 4 && (
                        <span className="rounded bg-slate-100 px-1.5 py-0.2 text-[10px] font-semibold text-slate-500">
                          +{role.permissions.length - 4} more
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                {/* Card Footer Actions */}
                <div className="pt-2 border-t border-slate-100 flex items-center justify-between text-xs">
                  <button
                    type="button"
                    onClick={() => openEditRoleModal(role)}
                    className="font-semibold text-blue-600 hover:text-blue-800"
                  >
                    Edit Privileges ↗
                  </button>

                  {!role.is_system_role && (
                    <button
                      type="button"
                      onClick={() => handleDeleteRole(role.name)}
                      className="text-xs font-semibold text-red-600 hover:text-red-800"
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* TAB 3: ENTERPRISE SSO & SAML 2.0 */}
      {activeTab === 'sso' && (
        <div className="space-y-6">
          {/* SSO Top Status Banner */}
          <div className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm flex flex-col md:flex-row md:items-center md:justify-between gap-5">
            <div className="space-y-1">
              <div className="flex items-center gap-2.5">
                <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-blue-100 text-blue-600 font-bold text-lg">
                  🔑
                </div>
                <div>
                  {ssoConfig?.last_tested_at && (
                    <div className="text-[10px] font-mono text-slate-400">
                      Config ID: {ssoConfig.id.slice(0, 8)}... | Last verified: {new Date(ssoConfig.last_tested_at).toLocaleString()}
                    </div>
                  )}
                  <h2 className="text-base font-bold text-slate-900">
                    SAML 2.0 Identity Provider (IdP) & SSO Enforcement
                  </h2>
                  <p className="text-xs text-slate-500">
                    Connect Okta, Microsoft Entra ID (Azure AD), Google Workspace, OneLogin, or custom SAML 2.0 federations.
                  </p>
                </div>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <span
                className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-bold border ${
                  ssoEnabled
                    ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                    : 'bg-slate-100 text-slate-600 border-slate-200'
                }`}
              >
                <span
                  className={`h-2 w-2 rounded-full ${ssoEnabled ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'}`}
                />
                {ssoEnabled ? 'SSO Enforced' : 'SSO Inactive'}
              </span>

              <button
                type="button"
                onClick={handleRunSSOTest}
                disabled={testingSso}
                className="inline-flex items-center gap-1.5 rounded-lg border border-slate-300 bg-white px-3.5 py-1.5 text-xs font-semibold text-slate-700 shadow-xs hover:bg-slate-50 disabled:opacity-50"
              >
                {testingSso ? 'Testing Connection...' : '⚡ Test Connection'}
              </button>
            </div>
          </div>

          <form onSubmit={handleSaveSSO} className="space-y-6">
            {/* Identity Provider Presets */}
            <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
              <div>
                <h3 className="text-sm font-bold text-slate-900">1. Select Identity Provider</h3>
                <p className="text-xs text-slate-500 mt-0.5">
                  Choose your organization's enterprise directory or select Generic SAML 2.0.
                </p>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-5 gap-3">
                {[
                  { id: 'okta', name: 'Okta', icon: '🔵', desc: 'Okta Verify & SAML' },
                  { id: 'azure_ad', name: 'Microsoft Entra ID', icon: '🔷', desc: 'Azure AD / M365' },
                  { id: 'google_workspace', name: 'Google Workspace', icon: '🔴', desc: 'Google Cloud Identity' },
                  { id: 'onelogin', name: 'OneLogin', icon: '🟣', desc: 'OneLogin Cloud Directory' },
                  { id: 'saml', name: 'Generic SAML 2.0', icon: '🌐', desc: 'Ping / Keycloak / Custom' },
                ].map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => setSsoProvider(item.id as any)}
                    className={`flex flex-col items-center justify-center p-3.5 rounded-xl border text-center transition-all ${
                      ssoProvider === item.id
                        ? 'border-blue-500 bg-blue-50/60 shadow-xs ring-2 ring-blue-500/20'
                        : 'border-slate-200 bg-slate-50/50 hover:bg-slate-100 hover:border-slate-300'
                    }`}
                  >
                    <span className="text-2xl mb-1">{item.icon}</span>
                    <span className="text-xs font-bold text-slate-800">{item.name}</span>
                    <span className="text-[10px] text-slate-400 mt-0.5">{item.desc}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* IdP Settings */}
              <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
                <div className="border-b border-slate-100 pb-3">
                  <h3 className="text-sm font-bold text-slate-900">2. Identity Provider (IdP) Credentials</h3>
                  <p className="text-xs text-slate-500 mt-0.5">
                    Enter the SSO endpoints provided by your identity directory.
                  </p>
                </div>

                <div className="space-y-3 text-xs">
                  <div>
                    <label className="block font-semibold text-slate-700 mb-1">
                      IdP Single Sign-On URL (HTTP-POST Binding) *
                    </label>
                    <input
                      type="url"
                      value={idpSsoUrl}
                      onChange={(e) => setIdpSsoUrl(e.target.value)}
                      placeholder={
                        ssoProvider === 'okta'
                          ? 'https://yourcompany.okta.com/app/yourcompany_communicator/sso/saml'
                          : ssoProvider === 'azure_ad'
                          ? 'https://login.microsoftonline.com/{tenant-id}/saml2'
                          : 'https://idp.example.com/sso/saml'
                      }
                      className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none font-mono text-[11px]"
                    />
                  </div>

                  <div>
                    <label className="block font-semibold text-slate-700 mb-1">
                      IdP Entity ID / Issuer URI *
                    </label>
                    <input
                      type="text"
                      value={idpEntityId}
                      onChange={(e) => setIdpEntityId(e.target.value)}
                      placeholder={
                        ssoProvider === 'okta'
                          ? 'http://www.okta.com/exk123456789'
                          : ssoProvider === 'azure_ad'
                          ? 'https://sts.windows.net/{tenant-id}/'
                          : 'https://idp.example.com/entity-id'
                      }
                      className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none font-mono text-[11px]"
                    />
                  </div>

                  <div>
                    <label className="block font-semibold text-slate-700 mb-1">
                      X.509 Public Certificate (PEM format) *
                    </label>
                    <textarea
                      rows={5}
                      value={idpCert}
                      onChange={(e) => setIdpCert(e.target.value)}
                      placeholder="-----BEGIN CERTIFICATE-----
MIIDpDCCAoygAwIBAgIGAX...
-----END CERTIFICATE-----"
                      className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none font-mono text-[10px] leading-relaxed"
                    />
                    <p className="text-[10px] text-slate-400 mt-1">
                      Used to verify XML digital signatures on SAML authentication assertions.
                    </p>
                  </div>
                </div>
              </div>

              {/* Service Provider (SP) Metadata */}
              <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
                <div className="border-b border-slate-100 pb-3 flex items-center justify-between">
                  <div>
                    <h3 className="text-sm font-bold text-slate-900">3. Service Provider (SP) Settings</h3>
                    <p className="text-xs text-slate-500 mt-0.5">
                      Configure these values in your IdP application setup.
                    </p>
                  </div>
                  <a
                    href="/api/v1/sso/metadata.xml"
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-1 rounded-md bg-blue-50 border border-blue-200 px-2.5 py-1 text-[11px] font-semibold text-blue-700 hover:bg-blue-100"
                  >
                    📥 Download XML
                  </a>
                </div>

                <div className="space-y-3 text-xs">
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="font-semibold text-slate-700">Assertion Consumer Service (ACS) URL</label>
                      <button
                        type="button"
                        onClick={() => handleCopy(`${window.location.origin}/api/v1/sso/saml/acs`, 'acs')}
                        className="text-blue-600 hover:text-blue-800 text-[11px] font-semibold"
                      >
                        {copiedField === 'acs' ? '✓ Copied' : 'Copy'}
                      </button>
                    </div>
                    <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 font-mono text-[11px] text-slate-700 select-all">
                      {window.location.origin}/api/v1/sso/saml/acs
                    </div>
                  </div>

                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="font-semibold text-slate-700">SP Entity ID (Audience URI)</label>
                      <button
                        type="button"
                        onClick={() => handleCopy(spEntityId, 'entityId')}
                        className="text-blue-600 hover:text-blue-800 text-[11px] font-semibold"
                      >
                        {copiedField === 'entityId' ? '✓ Copied' : 'Copy'}
                      </button>
                    </div>
                    <input
                      type="text"
                      value={spEntityId}
                      onChange={(e) => setSpEntityId(e.target.value)}
                      className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none font-mono text-[11px]"
                    />
                  </div>

                  <div className="rounded-lg border border-amber-200 bg-amber-50/60 p-3 text-[11px] text-amber-800 space-y-1">
                    <div className="font-bold">Required SAML Attribute Claims:</div>
                    <ul className="list-disc pl-4 space-y-0.5 text-[10px]">
                      <li><strong className="font-mono">NameID</strong> or <strong className="font-mono">email</strong>: Team member email</li>
                      <li><strong className="font-mono">displayName</strong> or <strong className="font-mono">name</strong>: Full member name</li>
                      <li><strong className="font-mono">role</strong> (Optional): Mapped RBAC role slug</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>

            {/* Access Policies & Provisioning */}
            <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm space-y-4">
              <div className="border-b border-slate-100 pb-3">
                <h3 className="text-sm font-bold text-slate-900">4. Domain Enforcement & Just-in-Time (JIT) Provisioning</h3>
                <p className="text-xs text-slate-500 mt-0.5">
                  Control domain restrictions, automated account creation, and default RBAC privilege assignments.
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-5 text-xs">
                <div className="space-y-4">
                  <div>
                    <label className="block font-semibold text-slate-700 mb-1">
                      Enforced Email Domains
                    </label>
                    <input
                      type="text"
                      value={enforcedDomainsText}
                      onChange={(e) => setEnforcedDomainsText(e.target.value)}
                      placeholder="e.g. bayworks.com, acme.io, enterprise.org"
                      className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none"
                    />
                    <p className="text-[10px] text-slate-400 mt-1">
                      Separate multiple domains with commas. Team members with these domains will be forced to authenticate via SSO.
                    </p>
                  </div>

                  <div>
                    <label className="block font-semibold text-slate-700 mb-1">
                      Default RBAC Role for Provisioned Members
                    </label>
                    <select
                      value={defaultSsoRole}
                      onChange={(e) => setDefaultSsoRole(e.target.value)}
                      className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none"
                    >
                      {roles.map((r) => (
                        <option key={r.name} value={r.name}>
                          {r.display_name} ({r.name})
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                <div className="space-y-3 rounded-xl border border-slate-100 bg-slate-50/70 p-4">
                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={ssoEnabled}
                      onChange={(e) => setSsoEnabled(e.target.checked)}
                      className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 mt-0.5"
                    />
                    <div>
                      <div className="font-semibold text-slate-800">Enforce Enterprise SSO</div>
                      <div className="text-[11px] text-slate-500">
                        Require team members with matching domains to log in through your Identity Provider.
                      </div>
                    </div>
                  </label>

                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={jitEnabled}
                      onChange={(e) => setJitEnabled(e.target.checked)}
                      className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 mt-0.5"
                    />
                    <div>
                      <div className="font-semibold text-slate-800">Just-in-Time (JIT) User Provisioning</div>
                      <div className="text-[11px] text-slate-500">
                        Automatically create team accounts on first successful SAML login.
                      </div>
                    </div>
                  </label>

                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={allowUnlinkedPasswords}
                      onChange={(e) => setAllowUnlinkedPasswords(e.target.checked)}
                      className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 mt-0.5"
                    />
                    <div>
                      <div className="font-semibold text-slate-800">Allow Break-Glass Password Sign-In</div>
                      <div className="text-[11px] text-slate-500">
                        Enable emergency local password logins if your Identity Provider is down.
                      </div>
                    </div>
                  </label>
                </div>
              </div>

              <div className="pt-4 border-t border-slate-100 flex items-center justify-end gap-3">
                <button
                  type="button"
                  onClick={handleRunSSOTest}
                  disabled={testingSso}
                  className="rounded-lg border border-slate-300 bg-white px-4 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50"
                >
                  {testingSso ? 'Testing Connection...' : '⚡ Validate Setup'}
                </button>
                <button
                  type="submit"
                  disabled={savingSso}
                  className="rounded-lg bg-blue-600 px-6 py-2 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
                >
                  {savingSso ? 'Saving Configuration...' : 'Save Enterprise SSO Settings'}
                </button>
              </div>
            </div>
          </form>
        </div>
      )}

      {/* SSO DIAGNOSTIC TEST RESULTS MODAL */}
      {showTestModal && ssoTestResult && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4 backdrop-blur-xs">
          <div className="w-full max-w-lg rounded-2xl bg-white shadow-2xl border border-slate-200 overflow-hidden flex flex-col max-h-[85vh]">
            <div className="flex items-center justify-between p-4 border-b border-slate-200 bg-slate-50">
              <div className="flex items-center gap-2">
                <span className="text-xl">
                  {ssoTestResult.status === 'healthy' ? '✅' : '⚠️'}
                </span>
                <div>
                  <h3 className="font-bold text-sm text-slate-900">SAML 2.0 Connection Diagnostics</h3>
                  <p className="text-[11px] text-slate-500">
                    Tested at: {new Date(ssoTestResult.tested_at).toLocaleTimeString()}
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowTestModal(false)}
                className="h-7 w-7 rounded-full bg-slate-200 text-slate-600 hover:bg-slate-300 flex items-center justify-center text-xs font-bold"
              >
                ✕
              </button>
            </div>

            <div className="p-4 space-y-3 overflow-y-auto flex-1">
              <div
                className={`rounded-lg p-3 text-xs border ${
                  ssoTestResult.status === 'healthy'
                    ? 'bg-emerald-50 text-emerald-800 border-emerald-200'
                    : 'bg-amber-50 text-amber-800 border-amber-200'
                }`}
              >
                {ssoTestResult.message}
              </div>

              <div className="divide-y divide-slate-100 border border-slate-200 rounded-lg overflow-hidden text-xs">
                {ssoTestResult.checks.map((check, idx) => (
                  <div key={idx} className="p-3 bg-white flex items-start justify-between gap-3">
                    <div>
                      <div className="font-semibold text-slate-800">{check.name}</div>
                      <div className="text-[11px] text-slate-500 mt-0.5">{check.message}</div>
                    </div>
                    <span
                      className={`shrink-0 rounded px-2 py-0.5 text-[10px] font-bold uppercase border ${
                        check.passed
                          ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                          : 'bg-amber-50 text-amber-700 border-amber-200'
                      }`}
                    >
                      {check.passed ? '✓ Pass' : 'Warning'}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            <div className="p-3 bg-slate-50 border-t border-slate-200 flex justify-end">
              <button
                onClick={() => setShowTestModal(false)}
                className="rounded-lg bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white hover:bg-slate-800"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* CREATE / EDIT ROLE MODAL */}
      {showRoleModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4 backdrop-blur-xs">
          <div className="w-full max-w-2xl rounded-2xl bg-white shadow-2xl border border-slate-200 overflow-hidden flex flex-col max-h-[90vh]">
            <div className="flex items-center justify-between p-4 border-b border-slate-200 bg-slate-50">
              <div>
                <h3 className="font-bold text-sm text-slate-900">
                  {editingRole ? `Edit Role: ${editingRole.display_name}` : 'Create Custom Enterprise Role'}
                </h3>
                <p className="text-[11px] text-slate-500">
                  Define granular access controls across conversations, CRM, and system settings.
                </p>
              </div>
              <button
                onClick={() => setShowRoleModal(false)}
                className="h-7 w-7 rounded-full bg-slate-200 text-slate-600 hover:bg-slate-300 flex items-center justify-center text-xs font-bold"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleSaveRole} className="p-5 space-y-4 overflow-y-auto flex-1">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Role Display Name</label>
                  <input
                    required
                    value={roleDisplayName}
                    onChange={(e) => {
                      setRoleDisplayName(e.target.value)
                      if (!editingRole && !roleSlug) {
                        setRoleSlug(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, '_'))
                      }
                    }}
                    placeholder="e.g. Tier 2 Support Lead"
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                  />
                </div>

                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Role Identifier (Slug)</label>
                  <input
                    required
                    disabled={!!editingRole}
                    value={roleSlug}
                    onChange={(e) => setRoleSlug(e.target.value)}
                    placeholder="e.g. tier2_lead"
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs font-mono disabled:bg-slate-100 disabled:text-slate-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Description</label>
                <input
                  value={roleDesc}
                  onChange={(e) => setRoleDesc(e.target.value)}
                  placeholder="Summarize responsibilities and intended team..."
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 text-xs focus:border-blue-500 focus:outline-none"
                />
              </div>

              {/* Permissions Checkbox Grid by Category */}
              <div className="space-y-4 pt-2">
                <div className="flex items-center justify-between border-b border-slate-200 pb-2">
                  <span className="text-xs font-bold text-slate-900 uppercase tracking-wider">
                    Permissions Matrix ({selectedPerms.length} selected)
                  </span>
                  <div className="flex items-center gap-2 text-xs">
                    <button
                      type="button"
                      onClick={() => setSelectedPerms(catalog.map((c) => c.key))}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      Select All
                    </button>
                    <span>•</span>
                    <button
                      type="button"
                      onClick={() => setSelectedPerms([])}
                      className="text-slate-500 hover:text-slate-700 font-medium"
                    >
                      Deselect All
                    </button>
                  </div>
                </div>

                {categories.map((cat) => {
                  const catItems = catalog.filter((c) => c.category === cat)
                  return (
                    <div key={cat} className="rounded-lg border border-slate-200 p-3 bg-slate-50/50 space-y-2">
                      <div className="text-xs font-bold text-slate-800">{cat}</div>
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                        {catItems.map((item) => {
                          const isChecked = selectedPerms.includes(item.key)
                          return (
                            <label
                              key={item.key}
                              className={`flex items-start gap-2 p-2 rounded-md border text-xs cursor-pointer transition-colors ${
                                isChecked
                                  ? 'bg-blue-50/70 border-blue-200 text-slate-900'
                                  : 'bg-white border-slate-200 text-slate-600 hover:bg-slate-50'
                              }`}
                            >
                              <input
                                type="checkbox"
                                checked={isChecked}
                                onChange={() => togglePermission(item.key)}
                                className="mt-0.5 h-3.5 w-3.5 rounded border-slate-300 text-blue-600"
                              />
                              <div>
                                <div className="font-semibold">{item.label}</div>
                                <div className="text-[10px] text-slate-500 leading-tight">{item.description}</div>
                              </div>
                            </label>
                          )
                        })}
                      </div>
                    </div>
                  )
                })}
              </div>

              <div className="pt-3 border-t border-slate-200 flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowRoleModal(false)}
                  className="rounded-lg border border-slate-300 bg-white px-4 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={savingRole || !roleDisplayName.trim()}
                  className="rounded-lg bg-blue-600 px-5 py-2 text-xs font-semibold text-white shadow hover:bg-blue-700 disabled:opacity-50"
                >
                  {savingRole ? 'Saving Role...' : 'Save Role Permissions'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* INSPECT MEMBER PERMISSIONS MODAL */}
      {inspectingMember && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4 backdrop-blur-xs">
          <div className="w-full max-w-lg rounded-2xl bg-white shadow-2xl border border-slate-200 overflow-hidden flex flex-col max-h-[85vh]">
            <div className="flex items-center justify-between p-4 border-b border-slate-200 bg-slate-50">
              <div>
                <h3 className="font-bold text-sm text-slate-900">
                  Effective Privileges: {inspectingMember.name}
                </h3>
                <p className="text-[11px] text-slate-500">
                  Role: <strong className="text-slate-800 capitalize">{inspectingMember.role || 'agent'}</strong> ({inspectingMember.email})
                </p>
              </div>
              <button
                onClick={() => setInspectingMember(null)}
                className="h-7 w-7 rounded-full bg-slate-200 text-slate-600 hover:bg-slate-300 flex items-center justify-center text-xs font-bold"
              >
                ✕
              </button>
            </div>

            <div className="p-4 space-y-3 overflow-y-auto flex-1">
              <div className="rounded-lg bg-blue-50 border border-blue-200 p-3 text-xs text-blue-900">
                Granted based on role assignment and SOC 2 RBAC governance.
              </div>

              <div className="divide-y divide-slate-100 border border-slate-200 rounded-lg overflow-hidden text-xs">
                {catalog.map((perm) => {
                  const roleObj = roles.find((r) => r.name === (inspectingMember.role || 'agent'))
                  const isRoleGranted = roleObj?.permissions.includes(perm.key) ?? false
                  const isCustomGranted = inspectingMember.custom_permissions?.includes(perm.key) ?? false
                  const isGranted = inspectingMember.role === 'super_admin' || isRoleGranted || isCustomGranted

                  return (
                    <div key={perm.key} className="flex items-center justify-between p-2.5 bg-white">
                      <div>
                        <div className="font-semibold text-slate-800">{perm.label}</div>
                        <div className="text-[10px] font-mono text-slate-400">{perm.key}</div>
                      </div>
                      <span
                        className={`rounded px-2 py-0.5 text-[10px] font-bold uppercase border ${
                          isGranted
                            ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                            : 'bg-slate-50 text-slate-400 border-slate-200'
                        }`}
                      >
                        {isGranted ? '✓ Allowed' : 'Denied'}
                      </span>
                    </div>
                  )
                })}
              </div>
            </div>

            <div className="p-3 bg-slate-50 border-t border-slate-200 flex justify-end">
              <button
                onClick={() => setInspectingMember(null)}
                className="rounded-lg bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white hover:bg-slate-800"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
