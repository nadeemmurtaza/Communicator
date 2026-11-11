import { useState, useEffect } from 'react'
import {
  getNotificationSummary,
  listNotifications,
  queueNotification,
  broadcastNotification,
  markNotificationRead,
  dismissNotification,
  getNotificationPreferences,
  updateNotificationPreferences,
  getNotificationSettings,
  updateNotificationSettings,
  listWebSocketConnections,
  cleanupOldNotifications,
  type NotificationSummary,
  type NotificationItem,
  type NotificationPreferences,
  type NotificationSettings,
  type WebSocketConnection,
} from '../lib/api'
import { Bell, Megaphone, Settings, User, CheckCircle, XCircle, Trash2, RefreshCw, Send, Volume2, Monitor, Clock } from '../components/icons'

export default function NotificationsDashboard() {
  const [tab, setTab] = useState<'overview' | 'notifications' | 'preferences' | 'settings'>('overview')
  const [summary, setSummary] = useState<NotificationSummary | null>(null)
  const [notifications, setNotifications] = useState<NotificationItem[]>([])
  const [preferences, setPreferences] = useState<NotificationPreferences | null>(null)
  const [settings, setSettings] = useState<NotificationSettings | null>(null)
  const [connections, setConnections] = useState<WebSocketConnection[]>([])
  const [loading, setLoading] = useState(false)
  const [filter, setFilter] = useState('')
  const [statusFilter, setStatusFilter] = useState('')

  useEffect(() => {
    loadSummary()
  }, [])

  useEffect(() => {
    if (tab === 'notifications') loadNotifications()
    if (tab === 'preferences') loadPreferences()
    if (tab === 'settings') loadSettings()
    if (tab === 'overview') loadSummary()
  }, [tab])

  async function loadSummary() {
    try {
      const data = await getNotificationSummary()
      setSummary(data)
    } catch (e) {
      console.error('Failed to load notification summary', e)
    }
  }

  async function loadNotifications() {
    setLoading(true)
    try {
      const data = await listNotifications(undefined, undefined, statusFilter || undefined, 100)
      const filtered = filter
        ? data.filter(n =>
            n.title.toLowerCase().includes(filter.toLowerCase()) ||
            n.body.toLowerCase().includes(filter.toLowerCase()) ||
            n.notificationType.toLowerCase().includes(filter.toLowerCase())
          )
        : data
      setNotifications(filtered)
    } catch (e) {
      console.error('Failed to load notifications', e)
    }
    setLoading(false)
  }

  async function loadPreferences() {
    try {
      const data = await getNotificationPreferences('user-default')
      setPreferences(data)
    } catch (e) {
      console.error('Failed to load preferences', e)
    }
  }

  async function loadSettings() {
    try {
      const data = await getNotificationSettings()
      setSettings(data)
    } catch (e) {
      console.error('Failed to load settings', e)
    }
  }

  async function loadConnections() {
    try {
      const data = await listWebSocketConnections()
      setConnections(data)
    } catch (e) {
      console.error('Failed to load connections', e)
    }
  }

  async function handleMarkRead(id: string) {
    try {
      await markNotificationRead(id)
      loadNotifications()
      loadSummary()
    } catch (e) {
      console.error('Failed to mark read', e)
    }
  }

  async function handleDismiss(id: string) {
    try {
      await dismissNotification(id)
      loadNotifications()
      loadSummary()
    } catch (e) {
      console.error('Failed to dismiss', e)
    }
  }

  async function handleBroadcast() {
    try {
      const result = await broadcastNotification({
        organizationId: 'org-default',
        notificationType: 'system',
        title: 'Test Broadcast',
        body: 'This is a test broadcast from the dashboard.',
      })
      alert(result.message)
      loadSummary()
    } catch (e) {
      console.error('Failed to broadcast', e)
    }
  }

  async function handleCleanup() {
    try {
      const result = await cleanupOldNotifications()
      alert(result.message)
      loadSummary()
    } catch (e) {
      console.error('Failed to cleanup', e)
    }
  }

  async function handleSavePreferences() {
    if (!preferences) return
    try {
      await updateNotificationPreferences('user-default', preferences)
      alert('Preferences saved')
    } catch (e) {
      console.error('Failed to save preferences', e)
    }
  }

  async function handleSaveSettings() {
    if (!settings) return
    try {
      await updateNotificationSettings(settings)
      alert('Settings saved')
    } catch (e) {
      console.error('Failed to save settings', e)
    }
  }

  const togglePref = (field: keyof NotificationPreferences) => {
    if (!preferences) return
    setPreferences({ ...preferences, [field]: !preferences[field] as any })
  }

  return (
    <div style={{ padding: 24 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 24 }}>
        <Bell size={28} />
        <h1 style={{ margin: 0 }}>Real-time Notifications</h1>
      </div>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 24, borderBottom: '1px solid #e5e7eb', paddingBottom: 12 }}>
        {(['overview', 'notifications', 'preferences', 'settings'] as const).map(t => (
          <button
            key={t}
            onClick={() => setTab(t)}
            style={{
              padding: '8px 16px',
              border: 'none',
              borderRadius: 8,
              background: tab === t ? '#6366f1' : 'transparent',
              color: tab === t ? '#fff' : '#6b7280',
              fontWeight: tab === t ? 600 : 400,
              cursor: 'pointer',
              textTransform: 'capitalize',
            }}
          >
            {t}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {tab === 'overview' && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 16, marginBottom: 24 }}>
            <StatCard icon={<Bell />} label="Total Notifications" value={summary?.totalNotifications ?? 0} />
            <StatCard icon={<Clock />} label="Pending" value={summary?.pendingNotifications ?? 0} color="#f59e0b" />
            <StatCard icon={<Megaphone />} label="Unread" value={summary?.unreadNotifications ?? 0} color="#ef4444" />
            <StatCard icon={<Volume2 />} label="Active Connections" value={summary?.activeConnections ?? 0} color="#10b981" />
            <StatCard icon={<Send />} label="Broadcasts Today" value={summary?.broadcastsToday ?? 0} color="#6366f1" />
          </div>

          <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
            <button onClick={handleBroadcast} style={primaryBtnStyle}>
              <Send size={16} style={{ marginRight: 6 }} />
              Test Broadcast
            </button>
            <button onClick={loadConnections} style={secondaryBtnStyle}>
              <RefreshCw size={16} style={{ marginRight: 6 }} />
              Refresh Connections
            </button>
            <button onClick={handleCleanup} style={secondaryBtnStyle}>
              <Trash2 size={16} style={{ marginRight: 6 }} />
              Cleanup Old
            </button>
          </div>

          {/* By Type */}
          {summary?.byType && summary.byType.length > 0 && (
            <div style={{ marginBottom: 24 }}>
              <h3 style={{ marginTop: 0 }}>Notifications by Type</h3>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))', gap: 12 }}>
                {summary.byType.map(bt => (
                  <div key={bt.notificationType} style={cardStyle}>
                    <div style={{ fontSize: 13, color: '#6b7280', marginBottom: 4 }}>{bt.notificationType}</div>
                    <div style={{ fontSize: 24, fontWeight: 700 }}>{bt.count}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Notifications */}
          {summary?.recentNotifications && summary.recentNotifications.length > 0 && (
            <div style={{ marginBottom: 24 }}>
              <h3 style={{ marginTop: 0 }}>Recent Notifications</h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {summary.recentNotifications.slice(0, 5).map(n => (
                  <div key={n.id} style={{ ...cardStyle, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <div>
                      <div style={{ fontWeight: 600 }}>{n.title}</div>
                      <div style={{ fontSize: 13, color: '#6b7280' }}>{n.body}</div>
                    </div>
                    <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                      <span style={statusBadge(n.status)}>{n.status}</span>
                      <span style={{ fontSize: 12, color: '#9ca3af' }}>{n.notificationType}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Active Connections */}
          {connections.length > 0 && (
            <div>
              <h3 style={{ marginTop: 0 }}>Active WebSocket Connections ({connections.length})</h3>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ textAlign: 'left', borderBottom: '2px solid #e5e7eb' }}>
                    <th style={{ padding: '8px 12px' }}>User</th>
                    <th style={{ padding: '8px 12px' }}>Session</th>
                    <th style={{ padding: '8px 12px' }}>Path</th>
                    <th style={{ padding: '8px 12px' }}>Connected</th>
                  </tr>
                </thead>
                <tbody>
                  {connections.map(c => (
                    <tr key={c.id} style={{ borderBottom: '1px solid #f3f4f6' }}>
                      <td style={{ padding: '8px 12px' }}>{c.userId}</td>
                      <td style={{ padding: '8px 12px', fontSize: 12 }}>{c.sessionId.slice(0, 8)}</td>
                      <td style={{ padding: '8px 12px' }}>{c.path}</td>
                      <td style={{ padding: '8px 12px', fontSize: 12 }}>{new Date(c.connectedAt).toLocaleString()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Notifications Tab */}
      {tab === 'notifications' && (
        <div>
          <div style={{ display: 'flex', gap: 12, marginBottom: 16 }}>
            <input
              type="text"
              placeholder="Search notifications..."
              value={filter}
              onChange={e => setFilter(e.target.value)}
              style={{ ...inputStyle, flex: 1 }}
            />
            <select
              value={statusFilter}
              onChange={e => setStatusFilter(e.target.value)}
              style={{ ...inputStyle, width: 150 }}
            >
              <option value="">All Statuses</option>
              <option value="pending">Pending</option>
              <option value="sent">Sent</option>
              <option value="dismissed">Dismissed</option>
            </select>
            <button onClick={loadNotifications} style={primaryBtnStyle}>
              <RefreshCw size={16} style={{ marginRight: 6 }} />
              Refresh
            </button>
          </div>

          {loading ? (
            <div style={{ textAlign: 'center', padding: 40, color: '#6b7280' }}>Loading...</div>
          ) : notifications.length === 0 ? (
            <div style={{ textAlign: 'center', padding: 40, color: '#6b7280' }}>No notifications found.</div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {notifications.map(n => (
                <div key={n.id} style={{ ...cardStyle, padding: 16 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                    <div style={{ flex: 1 }}>
                      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 4 }}>
                        <span style={{ fontWeight: 600, fontSize: 15 }}>{n.title}</span>
                        <span style={statusBadge(n.status)}>{n.status}</span>
                        <span style={{ fontSize: 11, color: '#9ca3af', background: '#f3f4f6', padding: '2px 6px', borderRadius: 4 }}>
                          {n.notificationType}
                        </span>
                      </div>
                      {n.body && <div style={{ fontSize: 14, color: '#4b5563', marginBottom: 4 }}>{n.body}</div>}
                      <div style={{ fontSize: 12, color: '#9ca3af' }}>
                        {n.channel} &middot; {new Date(n.createdAt).toLocaleString()}
                      </div>
                    </div>
                    <div style={{ display: 'flex', gap: 6, marginLeft: 12 }}>
                      {n.status !== 'dismissed' && !n.readAt && (
                        <button onClick={() => handleMarkRead(n.id)} title="Mark as read" style={iconBtnStyle}>
                          <CheckCircle size={16} />
                        </button>
                      )}
                      {n.status !== 'dismissed' && (
                        <button onClick={() => handleDismiss(n.id)} title="Dismiss" style={iconBtnStyle}>
                          <XCircle size={16} />
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Preferences Tab */}
      {tab === 'preferences' && preferences && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
            <h3 style={{ margin: 0 }}>Notification Preferences for {preferences.userId}</h3>
            <button onClick={handleSavePreferences} style={primaryBtnStyle}>
              Save Preferences
            </button>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: 16 }}>
            <PrefToggle label="New Message Push" icon={<Bell size={14} />} checked={preferences.newMessagePush} onChange={() => togglePref('newMessagePush')} />
            <PrefToggle label="New Message Email" icon={<Bell size={14} />} checked={preferences.newMessageEmail} onChange={() => togglePref('newMessageEmail')} />
            <PrefToggle label="Draft Approved Push" icon={<CheckCircle size={14} />} checked={preferences.draftApprovedPush} onChange={() => togglePref('draftApprovedPush')} />
            <PrefToggle label="Draft Rejected Push" icon={<XCircle size={14} />} checked={preferences.draftRejectedPush} onChange={() => togglePref('draftRejectedPush')} />
            <PrefToggle label="Alert Triggered Push" icon={<Megaphone size={14} />} checked={preferences.alertTriggeredPush} onChange={() => togglePref('alertTriggeredPush')} />
            <PrefToggle label="Alert Triggered Email" icon={<Megaphone size={14} />} checked={preferences.alertTriggeredEmail} onChange={() => togglePref('alertTriggeredEmail')} />
            <PrefToggle label="Conversation Assigned" icon={<User size={14} />} checked={preferences.conversationAssignedPush} onChange={() => togglePref('conversationAssignedPush')} />
            <PrefToggle label="Conversation Escalated" icon={<User size={14} />} checked={preferences.conversationEscalatedPush} onChange={() => togglePref('conversationEscalatedPush')} />
            <PrefToggle label="Workflow Completed" icon={<CheckCircle size={14} />} checked={preferences.workflowCompletedPush} onChange={() => togglePref('workflowCompletedPush')} />
            <PrefToggle label="CSAT Response" icon={<Bell size={14} />} checked={preferences.csatResponsePush} onChange={() => togglePref('csatResponsePush')} />
            <PrefToggle label="Suggestion Ready" icon={<Bell size={14} />} checked={preferences.suggestionReadyPush} onChange={() => togglePref('suggestionReadyPush')} />
            <PrefToggle label="Operator Proposal" icon={<Bell size={14} />} checked={preferences.operatorProposalPush} onChange={() => togglePref('operatorProposalPush')} />
            <PrefToggle label="Sound Enabled" icon={<Volume2 size={14} />} checked={preferences.soundEnabled} onChange={() => togglePref('soundEnabled')} />
            <PrefToggle label="Desktop Notifications" icon={<Monitor size={14} />} checked={preferences.desktopNotifications} onChange={() => togglePref('desktopNotifications')} />
          </div>

          <div style={{ ...cardStyle, marginTop: 16, padding: 16 }}>
            <h4 style={{ margin: '0 0 12px 0' }}>Quiet Hours</h4>
            <div style={{ display: 'flex', gap: 16 }}>
              <div>
                <label style={{ fontSize: 13, color: '#6b7280', display: 'block', marginBottom: 4 }}>Start Time</label>
                <input
                  type="time"
                  value={preferences.quietHoursStart || ''}
                  onChange={e => setPreferences({ ...preferences, quietHoursStart: e.target.value || undefined })}
                  style={inputStyle}
                />
              </div>
              <div>
                <label style={{ fontSize: 13, color: '#6b7280', display: 'block', marginBottom: 4 }}>End Time</label>
                <input
                  type="time"
                  value={preferences.quietHoursEnd || ''}
                  onChange={e => setPreferences({ ...preferences, quietHoursEnd: e.target.value || undefined })}
                  style={inputStyle}
                />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {tab === 'settings' && settings && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
            <h3 style={{ margin: 0 }}>Global Notification Settings</h3>
            <button onClick={handleSaveSettings} style={primaryBtnStyle}>
              Save Settings
            </button>
          </div>

          <div style={{ ...cardStyle, padding: 24, maxWidth: 600 }}>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              <PrefToggle label="Notifications Enabled" icon={<Bell size={14} />} checked={settings.enabled} onChange={() => setSettings({ ...settings, enabled: !settings.enabled })} />
              <PrefToggle label="WebSocket Enabled" icon={<Volume2 size={14} />} checked={settings.websocketEnabled} onChange={() => setSettings({ ...settings, websocketEnabled: !settings.websocketEnabled })} />
              <PrefToggle label="Email Notifications" icon={<Megaphone size={14} />} checked={settings.emailNotificationsEnabled} onChange={() => setSettings({ ...settings, emailNotificationsEnabled: !settings.emailNotificationsEnabled })} />
              <PrefToggle label="Digest Mode" icon={<Clock size={14} />} checked={settings.digestMode} onChange={() => setSettings({ ...settings, digestMode: !settings.digestMode })} />

              <div>
                <label style={{ fontSize: 13, color: '#6b7280', display: 'block', marginBottom: 4 }}>Email From Address</label>
                <input
                  type="text"
                  value={settings.emailFromAddress}
                  onChange={e => setSettings({ ...settings, emailFromAddress: e.target.value })}
                  style={{ ...inputStyle, width: '100%' }}
                />
              </div>

              <div>
                <label style={{ fontSize: 13, color: '#6b7280', display: 'block', marginBottom: 4 }}>Max Notifications Per Hour</label>
                <input
                  type="number"
                  value={settings.maxNotificationsPerHour}
                  onChange={e => setSettings({ ...settings, maxNotificationsPerHour: parseInt(e.target.value) || 100 })}
                  style={{ ...inputStyle, width: '100%' }}
                />
              </div>

              <div>
                <label style={{ fontSize: 13, color: '#6b7280', display: 'block', marginBottom: 4 }}>Digest Interval (minutes)</label>
                <input
                  type="number"
                  value={settings.digestIntervalMinutes}
                  onChange={e => setSettings({ ...settings, digestIntervalMinutes: parseInt(e.target.value) || 60 })}
                  style={{ ...inputStyle, width: '100%' }}
                />
              </div>

              <div>
                <label style={{ fontSize: 13, color: '#6b7280', display: 'block', marginBottom: 4 }}>Retention Days</label>
                <input
                  type="number"
                  value={settings.retentionDays}
                  onChange={e => setSettings({ ...settings, retentionDays: parseInt(e.target.value) || 90 })}
                  style={{ ...inputStyle, width: '100%' }}
                />
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

// ——— Sub-components ———

function StatCard({ icon, label, value, color = '#6366f1' }: { icon: React.ReactNode; label: string; value: number; color?: string }) {
  return (
    <div style={{ ...cardStyle, padding: 20, display: 'flex', alignItems: 'center', gap: 16 }}>
      <div style={{ color, fontSize: 24 }}>{icon}</div>
      <div>
        <div style={{ fontSize: 13, color: '#6b7280', marginBottom: 2 }}>{label}</div>
        <div style={{ fontSize: 28, fontWeight: 700, color }}>{value}</div>
      </div>
    </div>
  )
}

function PrefToggle({ label, icon, checked, onChange }: { label: string; icon: React.ReactNode; checked: boolean; onChange: () => void }) {
  return (
    <label style={{ display: 'flex', alignItems: 'center', gap: 10, cursor: 'pointer', padding: '8px 12px', background: '#f9fafb', borderRadius: 8 }}>
      <input type="checkbox" checked={checked} onChange={onChange} />
      <span style={{ color: '#6b7280', fontSize: 14 }}>{icon}</span>
      <span style={{ fontSize: 14, fontWeight: 500 }}>{label}</span>
    </label>
  )
}

// ——— Styles ———

const cardStyle: React.CSSProperties = {
  background: '#fff',
  border: '1px solid #e5e7eb',
  borderRadius: 12,
  padding: 12,
}

const inputStyle: React.CSSProperties = {
  padding: '8px 12px',
  border: '1px solid #d1d5db',
  borderRadius: 8,
  fontSize: 14,
  outline: 'none',
}

const primaryBtnStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  padding: '8px 16px',
  background: '#6366f1',
  color: '#fff',
  border: 'none',
  borderRadius: 8,
  fontSize: 14,
  fontWeight: 500,
  cursor: 'pointer',
}

const secondaryBtnStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  padding: '8px 16px',
  background: '#f3f4f6',
  color: '#374151',
  border: '1px solid #d1d5db',
  borderRadius: 8,
  fontSize: 14,
  fontWeight: 500,
  cursor: 'pointer',
}

const iconBtnStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: 32,
  height: 32,
  border: 'none',
  borderRadius: 8,
  background: 'transparent',
  cursor: 'pointer',
  color: '#6b7280',
}

function statusBadge(status: string): React.CSSProperties {
  const colors: Record<string, { bg: string; color: string }> = {
    pending: { bg: '#fef3c7', color: '#92400e' },
    sent: { bg: '#d1fae5', color: '#065f46' },
    dismissed: { bg: '#fee2e2', color: '#991b1b' },
  }
  const c = colors[status] || { bg: '#f3f4f6', color: '#374151' }
  return {
    fontSize: 11,
    fontWeight: 600,
    padding: '2px 8px',
    borderRadius: 9999,
    background: c.bg,
    color: c.color,
    textTransform: 'capitalize',
  }
}
