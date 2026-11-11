import { useState, useEffect } from 'react'
import {
  getAudienceSummary,
  getAudienceSettings,
  updateAudienceSettings,
  listAudienceLinks,
  linkKbToSegment,
  unlinkKbFromSegment,
  listAudienceMemberships,
  addContactToSegment,
  removeContactFromSegment,
  getAudienceUsageLogs,
} from '../lib/api'
import type {
  AudienceSummary,
  AudienceKBSettings,
  AudienceKBLink,
  SegmentContactMembership,
  AudienceKBUsageLog,
} from '../lib/api'
import {
  Users,
  Link as LinkIcon,
  Settings,
  BarChart3,
  Plus,
  Trash2,
  X,
  User,
  Eye,
} from '../components/icons'

export default function AudienceKBDashboard() {
  const [tab, setTab] = useState<'overview' | 'links' | 'memberships' | 'settings'>('overview')
  const [summary, setSummary] = useState<AudienceSummary | null>(null)
  const [settings, setSettings] = useState<AudienceKBSettings | null>(null)
  const [links, setLinks] = useState<AudienceKBLink[]>([])
  const [memberships, setMemberships] = useState<SegmentContactMembership[]>([])
  const [usageLogs, setUsageLogs] = useState<AudienceKBUsageLog[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Link modal
  const [showLinkModal, setShowLinkModal] = useState(false)
  const [linkSegmentId, setLinkSegmentId] = useState('')
  const [linkKbEntryId, setLinkKbEntryId] = useState('')
  const [linkPriority, setLinkPriority] = useState(0)

  // Membership modal
  const [showMembershipModal, setShowMembershipModal] = useState(false)
  const [memberSegmentId, setMemberSegmentId] = useState('')
  const [memberContactId, setMemberContactId] = useState('')
  const [memberType, setMemberType] = useState('static')

  // Filter state
  const [filterSegment, setFilterSegment] = useState('')

  // Settings form
  const [settingsForm, setSettingsForm] = useState({
    enabled: true,
    fallbackToGlobalKb: true,
    mergeSegments: true,
    defaultScope: 'all',
  })

  async function loadAll() {
    setLoading(true)
    setError(null)
    try {
      const [sum, sett, lnks, mems] = await Promise.all([
        getAudienceSummary(),
        getAudienceSettings(),
        listAudienceLinks(filterSegment || undefined),
        listAudienceMemberships(undefined, undefined, 50),
      ])
      setSummary(sum)
      setSettings(sett)
      setLinks(lnks.links)
      setMemberships(mems.memberships)
      setSettingsForm({
        enabled: sett.enabled,
        fallbackToGlobalKb: sett.fallbackToGlobalKb,
        mergeSegments: sett.mergeSegments,
        defaultScope: sett.defaultScope,
      })
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadAll()
  }, [filterSegment])

  async function handleLinkKb() {
    setError(null)
    setSuccess(null)
    try {
      await linkKbToSegment({
        segmentId: linkSegmentId,
        kbEntryId: linkKbEntryId,
        priority: linkPriority,
      })
      setSuccess('KB entry linked to segment')
      setShowLinkModal(false)
      setLinkSegmentId('')
      setLinkKbEntryId('')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleUnlinkKb(segmentId: string, kbEntryId: string) {
    if (!confirm('Remove this KB link from the segment?')) return
    try {
      await unlinkKbFromSegment(segmentId, kbEntryId)
      setSuccess('KB entry unlinked')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleAddMembership() {
    setError(null)
    setSuccess(null)
    try {
      await addContactToSegment({
        segmentId: memberSegmentId,
        contactId: memberContactId,
        membershipType: memberType,
      })
      setSuccess('Contact added to segment')
      setShowMembershipModal(false)
      setMemberSegmentId('')
      setMemberContactId('')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleRemoveMembership(segmentId: string, contactId: string) {
    if (!confirm('Remove this contact from the segment?')) return
    try {
      await removeContactFromSegment(segmentId, contactId)
      setSuccess('Contact removed from segment')
      loadAll()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleUpdateSettings() {
    setError(null)
    setSuccess(null)
    try {
      const updated = await updateAudienceSettings(settingsForm)
      setSettings(updated)
      setSuccess('Settings updated')
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleViewUsage(segmentId: string) {
    setError(null)
    try {
      const result = await getAudienceUsageLogs(segmentId, 20)
      setUsageLogs(result.logs)
      setFilterSegment(segmentId)
      setTab('memberships')
    } catch (e) {
      setError((e as Error).message)
    }
  }

  if (loading) {
    return (
      <div style={{ padding: 32, textAlign: 'center', color: '#94a3b8' }}>
        Loading Audience KB Scoping...
      </div>
    )
  }

  return (
    <div style={{ padding: 24 }}>
      <div style={{ marginBottom: 24 }}>
        <h1 style={{ margin: 0, fontSize: 24, color: '#f1f5f9' }}>
          Audience KB Scoping
        </h1>
        <p style={{ margin: '8px 0 0', color: '#94a3b8' }}>
          Control which knowledge base entries each customer segment can access.
        </p>
      </div>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
        {(['overview', 'links', 'memberships', 'settings'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            style={{
              padding: '8px 16px',
              borderRadius: 8,
              border: 'none',
              background: tab === t ? '#6366f1' : '#1e293b',
              color: tab === t ? '#fff' : '#94a3b8',
              cursor: 'pointer',
              textTransform: 'capitalize',
            }}
          >
            {t}
          </button>
        ))}
      </div>

      {/* Status messages */}
      {error && (
        <div
          style={{
            padding: '12px 16px',
            borderRadius: 8,
            background: '#7f1d1d',
            color: '#fecaca',
            marginBottom: 16,
          }}
        >
          {error}
          <button
            onClick={() => setError(null)}
            style={{
              marginLeft: 12,
              background: 'none',
              border: 'none',
              color: '#fecaca',
              cursor: 'pointer',
            }}
          >
            <X />
          </button>
        </div>
      )}
      {success && (
        <div
          style={{
            padding: '12px 16px',
            borderRadius: 8,
            background: '#14532d',
            color: '#bbf7d0',
            marginBottom: 16,
          }}
        >
          {success}
          <button
            onClick={() => setSuccess(null)}
            style={{
              marginLeft: 12,
              background: 'none',
              border: 'none',
              color: '#bbf7d0',
              cursor: 'pointer',
            }}
          >
            <X />
          </button>
        </div>
      )}

      {/* Overview Tab */}
      {tab === 'overview' && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, marginBottom: 24 }}>
            <StatCard
              icon={<Users />}
              label="Segments with KB"
              value={summary?.totalSegmentsWithKb ?? 0}
            />
            <StatCard
              icon={<LinkIcon />}
              label="Total Links"
              value={summary?.totalLinks ?? 0}
            />
            <StatCard
              icon={<BarChart3 />}
              label="Usage (7d)"
              value={summary?.totalUsage7d ?? 0}
            />
            <StatCard
              icon={<Eye />}
              label="Avg Relevance"
              value={((summary?.avgRelevanceScore ?? 0) * 100).toFixed(0) + '%'}
            />
          </div>

          {/* Top Segments */}
          {summary?.topSegments.length && summary.topSegments.length > 0 && (
            <div style={{ marginBottom: 24 }}>
              <h3 style={{ color: '#f1f5f9', marginBottom: 12 }}>Top Segments by KB Links</h3>
              <div style={{ display: 'grid', gap: 8 }}>
                {summary.topSegments.map((seg, i) => (
                  <div
                    key={i}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      padding: '12px 16px',
                      background: '#1e293b',
                      borderRadius: 8,
                    }}
                  >
                    <span style={{ color: '#e2e8f0' }}>{seg.segmentName}</span>
                    <span style={{ color: '#6366f1', fontWeight: 600 }}>
                      {seg.linkCount} links
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Top KB Entries */}
          {summary?.topKbEntries.length && summary.topKbEntries.length > 0 && (
            <div>
              <h3 style={{ color: '#f1f5f9', marginBottom: 12 }}>
                Top KB Entries by Usage
              </h3>
              <div style={{ display: 'grid', gap: 8 }}>
                {summary.topKbEntries.map((entry, i) => (
                  <div
                    key={i}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      padding: '12px 16px',
                      background: '#1e293b',
                      borderRadius: 8,
                    }}
                  >
                    <span style={{ color: '#e2e8f0' }}>{entry.kbTitle}</span>
                    <span style={{ color: '#10b981', fontWeight: 600 }}>
                      {entry.usageCount} uses
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Links Tab */}
      {tab === 'links' && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16 }}>
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              <input
                type="text"
                placeholder="Filter by segment ID..."
                value={filterSegment}
                onChange={(e) => setFilterSegment(e.target.value)}
                style={{
                  padding: '8px 12px',
                  borderRadius: 8,
                  border: '1px solid #334155',
                  background: '#0f172a',
                  color: '#e2e8f0',
                }}
              />
            </div>
            <button
              onClick={() => setShowLinkModal(true)}
              style={{
                padding: '8px 16px',
                borderRadius: 8,
                border: 'none',
                background: '#6366f1',
                color: '#fff',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              <Plus /> Link KB Entry
            </button>
          </div>

          <div style={{ display: 'grid', gap: 8 }}>
            {links.map((link) => (
              <div
                key={link.id}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '12px 16px',
                  background: '#1e293b',
                  borderRadius: 8,
                }}
              >
                <div>
                  <div style={{ color: '#e2e8f0', fontWeight: 500 }}>
                    {link.segmentName || link.segmentId}
                  </div>
                  <div style={{ color: '#94a3b8', fontSize: 13 }}>
                    {link.kbTitle || link.kbEntryId}
                  </div>
                  <div style={{ color: '#64748b', fontSize: 12 }}>
                    Priority: {link.priority}
                  </div>
                </div>
                <div style={{ display: 'flex', gap: 8 }}>
                  <button
                    onClick={() => handleViewUsage(link.segmentId)}
                    style={{
                      padding: '6px 10px',
                      borderRadius: 6,
                      border: '1px solid #334155',
                      background: 'transparent',
                      color: '#94a3b8',
                      cursor: 'pointer',
                    }}
                  >
                    <Eye />
                  </button>
                  <button
                    onClick={() => handleUnlinkKb(link.segmentId, link.kbEntryId)}
                    style={{
                      padding: '6px 10px',
                      borderRadius: 6,
                      border: '1px solid #7f1d1d',
                      background: 'transparent',
                      color: '#f87171',
                      cursor: 'pointer',
                    }}
                  >
                    <Trash2 />
                  </button>
                </div>
              </div>
            ))}
            {links.length === 0 && (
              <div style={{ color: '#64748b', textAlign: 'center', padding: 32 }}>
                No KB links found. Create one to link a knowledge base entry to a segment.
              </div>
            )}
          </div>

          {/* Link Modal */}
          {showLinkModal && (
            <Modal onClose={() => setShowLinkModal(false)}>
              <h3 style={{ margin: '0 0 16px', color: '#f1f5f9' }}>Link KB Entry to Segment</h3>
              <input
                type="text"
                placeholder="Segment ID"
                value={linkSegmentId}
                onChange={(e) => setLinkSegmentId(e.target.value)}
                style={inputStyle}
              />
              <input
                type="text"
                placeholder="KB Entry ID"
                value={linkKbEntryId}
                onChange={(e) => setLinkKbEntryId(e.target.value)}
                style={inputStyle}
              />
              <input
                type="number"
                placeholder="Priority (0 = default)"
                value={linkPriority}
                onChange={(e) => setLinkPriority(Number(e.target.value))}
                style={inputStyle}
              />
              <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
                <button onClick={handleLinkKb} style={primaryBtnStyle}>
                  Link
                </button>
                <button
                  onClick={() => setShowLinkModal(false)}
                  style={secondaryBtnStyle}
                >
                  Cancel
                </button>
              </div>
            </Modal>
          )}
        </div>
      )}

      {/* Memberships Tab */}
      {tab === 'memberships' && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16 }}>
            <h3 style={{ color: '#f1f5f9', margin: 0 }}>
              {usageLogs.length > 0
                ? `Usage Logs for Segment: ${filterSegment}`
                : 'Segment Memberships'}
            </h3>
            <button
              onClick={() => setShowMembershipModal(true)}
              style={{
                padding: '8px 16px',
                borderRadius: 8,
                border: 'none',
                background: '#6366f1',
                color: '#fff',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 6,
              }}
            >
              <User /> Add Membership
            </button>
          </div>

          {/* Usage Logs */}
          {usageLogs.length > 0 && (
            <div style={{ marginBottom: 24 }}>
              {usageLogs.map((log) => (
                <div
                  key={log.id}
                  style={{
                    padding: '12px 16px',
                    background: '#1e293b',
                    borderRadius: 8,
                    marginBottom: 8,
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span style={{ color: '#e2e8f0' }}>{log.queryText}</span>
                    <span style={{ color: '#6366f1' }}>
                      {(log.relevanceScore * 100).toFixed(0)}%
                    </span>
                  </div>
                  <div style={{ color: '#64748b', fontSize: 12, marginTop: 4 }}>
                    Contact: {log.contactId} | Used: {log.usedInResponse ? 'Yes' : 'No'}
                  </div>
                </div>
              ))}
              <button
                onClick={() => {
                  setUsageLogs([])
                  setFilterSegment('')
                }}
                style={{
                  marginTop: 8,
                  padding: '6px 12px',
                  borderRadius: 6,
                  border: '1px solid #334155',
                  background: 'transparent',
                  color: '#94a3b8',
                  cursor: 'pointer',
                }}
              >
                Back to Memberships
              </button>
            </div>
          )}

          {/* Memberships List */}
          {usageLogs.length === 0 && (
            <div style={{ display: 'grid', gap: 8 }}>
              {memberships.map((mem) => (
                <div
                  key={mem.id}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '12px 16px',
                    background: '#1e293b',
                    borderRadius: 8,
                  }}
                >
                  <div>
                    <div style={{ color: '#e2e8f0', fontWeight: 500 }}>
                      {mem.contactName || mem.contactId}
                    </div>
                    <div style={{ color: '#94a3b8', fontSize: 13 }}>
                      {mem.segmentName || mem.segmentId}
                    </div>
                    <div style={{ color: '#64748b', fontSize: 12 }}>
                      Type: {mem.membershipType}
                    </div>
                  </div>
                  <button
                    onClick={() => handleRemoveMembership(mem.segmentId, mem.contactId)}
                    style={{
                      padding: '6px 10px',
                      borderRadius: 6,
                      border: '1px solid #7f1d1d',
                      background: 'transparent',
                      color: '#f87171',
                      cursor: 'pointer',
                    }}
                  >
                    <Trash2 />
                  </button>
                </div>
              ))}
              {memberships.length === 0 && (
                <div style={{ color: '#64748b', textAlign: 'center', padding: 32 }}>
                  No memberships found. Add contacts to segments to enable audience-based
                  knowledge scoping.
                </div>
              )}
            </div>
          )}

          {/* Membership Modal */}
          {showMembershipModal && (
            <Modal onClose={() => setShowMembershipModal(false)}>
              <h3 style={{ margin: '0 0 16px', color: '#f1f5f9' }}>
                Add Contact to Segment
              </h3>
              <input
                type="text"
                placeholder="Segment ID"
                value={memberSegmentId}
                onChange={(e) => setMemberSegmentId(e.target.value)}
                style={inputStyle}
              />
              <input
                type="text"
                placeholder="Contact ID"
                value={memberContactId}
                onChange={(e) => setMemberContactId(e.target.value)}
                style={inputStyle}
              />
              <select
                value={memberType}
                onChange={(e) => setMemberType(e.target.value)}
                style={{ ...inputStyle, appearance: 'auto' }}
              >
                <option value="static">Static</option>
                <option value="dynamic">Dynamic</option>
              </select>
              <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
                <button onClick={handleAddMembership} style={primaryBtnStyle}>
                  Add
                </button>
                <button
                  onClick={() => setShowMembershipModal(false)}
                  style={secondaryBtnStyle}
                >
                  Cancel
                </button>
              </div>
            </Modal>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {tab === 'settings' && (
        <div style={{ maxWidth: 480 }}>
          <h3 style={{ color: '#f1f5f9', marginBottom: 16 }}>Audience KB Settings</h3>

          <label style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16, alignItems: 'center' }}>
            <span style={{ color: '#e2e8f0' }}>Enabled</span>
            <Toggle
              checked={settingsForm.enabled}
              onChange={(v) => setSettingsForm((f) => ({ ...f, enabled: v }))}
            />
          </label>

          <label style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16, alignItems: 'center' }}>
            <span style={{ color: '#e2e8f0' }}>Fallback to Global KB</span>
            <Toggle
              checked={settingsForm.fallbackToGlobalKb}
              onChange={(v) =>
                setSettingsForm((f) => ({ ...f, fallbackToGlobalKb: v }))
              }
            />
          </label>

          <label style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16, alignItems: 'center' }}>
            <span style={{ color: '#e2e8f0' }}>Merge Segments</span>
            <Toggle
              checked={settingsForm.mergeSegments}
              onChange={(v) => setSettingsForm((f) => ({ ...f, mergeSegments: v }))}
            />
          </label>

          <div style={{ marginBottom: 16 }}>
            <label style={{ color: '#e2e8f0', display: 'block', marginBottom: 6 }}>
              Default Scope
            </label>
            <select
              value={settingsForm.defaultScope}
              onChange={(e) =>
                setSettingsForm((f) => ({ ...f, defaultScope: e.target.value }))
              }
              style={{ ...inputStyle, appearance: 'auto', width: '100%' }}
            >
              <option value="all">All</option>
              <option value="scoped">Scoped Only</option>
            </select>
          </div>

          <button onClick={handleUpdateSettings} style={primaryBtnStyle}>
            <Settings /> Save Settings
          </button>

          <div
            style={{
              marginTop: 24,
              padding: 16,
              background: '#1e293b',
              borderRadius: 8,
              color: '#94a3b8',
              fontSize: 13,
              lineHeight: 1.6,
            }}
          >
            <strong style={{ color: '#e2e8f0' }}>How it works:</strong>
            <br />
            • <strong>Enabled</strong> — Toggle audience-based KB scoping on/off
            <br />
            • <strong>Fallback to Global KB</strong> — When a segment has no linked KB
            entries, fall back to all published KB entries
            <br />
            • <strong>Merge Segments</strong> — When a contact belongs to multiple segments,
            merge all their KB entries (union) instead of using only the highest-priority
            segment
          </div>
        </div>
      )}
    </div>
  )
}

// ——— Shared Components ———

function StatCard({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode
  label: string
  value: string | number
}) {
  return (
    <div
      style={{
        padding: 20,
        background: '#1e293b',
        borderRadius: 12,
        display: 'flex',
        alignItems: 'center',
        gap: 16,
      }}
    >
      <div style={{ color: '#6366f1', fontSize: 24 }}>{icon}</div>
      <div>
        <div style={{ color: '#94a3b8', fontSize: 13 }}>{label}</div>
        <div style={{ color: '#f1f5f9', fontSize: 24, fontWeight: 700 }}>{value}</div>
      </div>
    </div>
  )
}

function Modal({
  children,
  onClose,
}: {
  children: React.ReactNode
  onClose: () => void
}) {
  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0,0,0,0.6)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: '#0f172a',
          borderRadius: 12,
          padding: 24,
          minWidth: 360,
          maxWidth: 480,
          border: '1px solid #334155',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </div>
  )
}

function Toggle({
  checked,
  onChange,
}: {
  checked: boolean
  onChange: (v: boolean) => void
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      style={{
        width: 44,
        height: 24,
        borderRadius: 12,
        border: 'none',
        background: checked ? '#6366f1' : '#334155',
        cursor: 'pointer',
        position: 'relative',
        padding: 0,
      }}
    >
      <div
        style={{
          width: 18,
          height: 18,
          borderRadius: '50%',
          background: '#fff',
          position: 'absolute',
          top: 3,
          left: checked ? 23 : 3,
          transition: 'left 0.2s',
        }}
      />
    </button>
  )
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  padding: '8px 12px',
  borderRadius: 8,
  border: '1px solid #334155',
  background: '#0f172a',
  color: '#e2e8f0',
  marginBottom: 12,
  boxSizing: 'border-box',
}

const primaryBtnStyle: React.CSSProperties = {
  padding: '8px 16px',
  borderRadius: 8,
  border: 'none',
  background: '#6366f1',
  color: '#fff',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  gap: 6,
}

const secondaryBtnStyle: React.CSSProperties = {
  padding: '8px 16px',
  borderRadius: 8,
  border: '1px solid #334155',
  background: 'transparent',
  color: '#94a3b8',
  cursor: 'pointer',
}
