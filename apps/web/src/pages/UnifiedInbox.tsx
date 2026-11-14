import { useEffect, useState } from 'react'
import {
  getUnifiedInbox,
  getUnifiedCustomer,
  type UnifiedInboxSummary,
  type UnifiedInboxEntry,
  type UnifiedMessage,
} from '../lib/api'
import { Users, MessageSquare, Phone, Mail, MessageCircle, Globe, ChevronRight, Search } from '../components/icons'

const channelIcons: Record<string, typeof Phone> = {
  whatsapp: MessageCircle,
  sms: Phone,
  email: Mail,
  website: Globe,
  facebook: Globe,
  instagram: Globe,
  telegram: MessageCircle,
  line: MessageCircle,
  voice: Phone,
  tiktok: Globe,
}

const channelColors: Record<string, string> = {
  whatsapp: '#25D366',
  sms: '#3498db',
  email: '#e74c3c',
  website: '#9b59b6',
  facebook: '#1877f2',
  instagram: '#e4405f',
  telegram: '#0088cc',
  line: '#00c300',
  voice: '#2ecc71',
  tiktok: '#000000',
}

function ChannelBadge({ channel }: { channel: string }) {
  const Icon = channelIcons[channel] || MessageSquare
  const color = channelColors[channel] || '#95a5a6'
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 4,
        padding: '2px 8px',
        borderRadius: 12,
        fontSize: 11,
        fontWeight: 600,
        color,
        border: `1px solid ${color}33`,
        backgroundColor: `${color}11`,
      }}
    >
      <Icon style={{ width: 12, height: 12 }} />
      {channel}
    </span>
  )
}

function formatTime(dateStr?: string) {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}d ago`
  return d.toLocaleDateString()
}

export function UnifiedInbox() {
  const [summary, setSummary] = useState<UnifiedInboxSummary | null>(null)
  const [selectedCustomer, setSelectedCustomer] = useState<UnifiedInboxEntry | null>(null)
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [channelFilter, setChannelFilter] = useState('')
  const [stateFilter, setStateFilter] = useState('')
  const [timelineView, setTimelineView] = useState(false)

  useEffect(() => {
    loadInbox()
  }, [search, channelFilter, stateFilter])

  async function loadInbox() {
    setLoading(true)
    try {
      const data = await getUnifiedInbox(20, 0, channelFilter || undefined, stateFilter || undefined, search || undefined)
      setSummary(data)
    } catch (err) {
      console.error('Failed to load unified inbox:', err)
    } finally {
      setLoading(false)
    }
  }

  async function selectCustomer(customerId: string) {
    try {
      const data = await getUnifiedCustomer(customerId)
      setSelectedCustomer(data)
      setTimelineView(false)
    } catch (err) {
      console.error('Failed to load customer:', err)
    }
  }

  function backToList() {
    setSelectedCustomer(null)
    setTimelineView(false)
  }

  // Customer detail view
  if (selectedCustomer) {
    const { customer, conversations, mergedTimeline } = selectedCustomer

    if (timelineView) {
      return (
        <div style={{ padding: 24 }}>
          <button
            onClick={() => setTimelineView(false)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '8px 16px',
              border: 'none',
              background: '#f3f4f6',
              borderRadius: 8,
              cursor: 'pointer',
              marginBottom: 16,
            }}
          >
            ← Back to Customer
          </button>

          <h2 style={{ marginBottom: 16 }}>
            Merged Timeline — {customer.displayName}
          </h2>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {mergedTimeline.map((msg) => (
              <TimelineMessage key={msg.id} message={msg} />
            ))}
          </div>
        </div>
      )
    }

    return (
      <div style={{ padding: 24 }}>
        <button
          onClick={backToList}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            padding: '8px 16px',
            border: 'none',
            background: '#f3f4f6',
            borderRadius: 8,
            cursor: 'pointer',
            marginBottom: 16,
          }}
        >
          ← Back to Inbox
        </button>

        {/* Customer Header */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 16,
            padding: 24,
            background: '#f8f9fa',
            borderRadius: 12,
            marginBottom: 24,
          }}
        >
          <div
            style={{
              width: 56,
              height: 56,
              borderRadius: '50%',
              background: '#6366f1',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'white',
              fontSize: 24,
              fontWeight: 700,
            }}
          >
            {customer.displayName.charAt(0).toUpperCase()}
          </div>

          <div style={{ flex: 1 }}>
            <h2 style={{ margin: 0 }}>{customer.displayName}</h2>
            <div style={{ display: 'flex', gap: 8, marginTop: 4, flexWrap: 'wrap' }}>
              {customer.channels.map((ch) => (
                <ChannelBadge key={ch} channel={ch} />
              ))}
            </div>
          </div>

          <div style={{ textAlign: 'right' }}>
            <div style={{ fontSize: 14, color: '#6b7280' }}>
              {customer.conversationCount} conversations
            </div>
            {customer.customerTier && (
              <span
                style={{
                  display: 'inline-block',
                  padding: '2px 8px',
                  borderRadius: 12,
                  fontSize: 11,
                  fontWeight: 600,
                  background: '#fef3c7',
                  color: '#92400e',
                }}
              >
                {customer.customerTier}
              </span>
            )}
          </div>
        </div>

        {/* Customer Info Grid */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: 16,
            marginBottom: 24,
          }}
        >
          {customer.email && (
            <InfoCard label="Email" value={customer.email} />
          )}
          {customer.phone && (
            <InfoCard label="Phone" value={customer.phone} />
          )}
          {customer.company && (
            <InfoCard label="Company" value={customer.company} />
          )}
          <InfoCard label="Lifetime Value" value={`$${customer.lifetimeValue.toFixed(2)}`} />
        </div>

        {/* Conversations */}
        <div style={{ marginBottom: 24 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
            <h3 style={{ margin: 0 }}>Conversations ({conversations.length})</h3>
            <button
              onClick={() => setTimelineView(true)}
              style={{
                padding: '6px 12px',
                border: '1px solid #6366f1',
                background: '#6366f1',
                color: 'white',
                borderRadius: 6,
                cursor: 'pointer',
                fontSize: 13,
              }}
            >
              View Merged Timeline ({mergedTimeline.length} messages)
            </button>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {conversations.map((conv) => (
              <div
                key={conv.conversationId}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 12,
                  padding: 12,
                  border: '1px solid #e5e7eb',
                  borderRadius: 8,
                  cursor: 'pointer',
                }}
              >
                <ChannelBadge channel={conv.channel} />
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 14 }}>
                    {conv.lastMessagePreview || 'No messages yet'}
                  </div>
                  <div style={{ fontSize: 12, color: '#6b7280', marginTop: 2 }}>
                    {formatTime(conv.lastMessageAt)} · {conv.messageCount} messages
                  </div>
                </div>
                <span
                  style={{
                    padding: '2px 8px',
                    borderRadius: 12,
                    fontSize: 11,
                    fontWeight: 600,
                    background: conv.state === 'awaiting_reply' ? '#fef3c7' : '#d1fae5',
                    color: conv.state === 'awaiting_reply' ? '#92400e' : '#065f46',
                  }}
                >
                  {conv.state}
                </span>
                <ChevronRight style={{ width: 16, height: 16, color: '#9ca3af' }} />
              </div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  // Main inbox list view
  return (
    <div style={{ padding: 24 }}>
      <h1 style={{ marginBottom: 24 }}>Unified Inbox</h1>

      {/* Summary Cards */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: 16,
          marginBottom: 24,
        }}
      >
        <SummaryCard
          icon={<Users style={{ width: 24, height: 24 }} />}
          label="Total Customers"
          value={summary?.totalCustomers ?? 0}
          color="#6366f1"
        />
        <SummaryCard
          icon={<MessageSquare style={{ width: 24, height: 24 }} />}
          label="Active Customers"
          value={summary?.activeCustomers ?? 0}
          color="#10b981"
        />
        {summary?.conversationsByChannel && (
          <div
            style={{
              padding: 16,
              border: '1px solid #e5e7eb',
              borderRadius: 12,
              gridColumn: 'span 2',
            }}
          >
            <div style={{ fontSize: 13, color: '#6b7280', marginBottom: 8 }}>
              Conversations by Channel
            </div>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
              {Object.entries(summary.conversationsByChannel).map(([ch, cnt]) => (
                <div
                  key={ch}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 4,
                    padding: '4px 12px',
                    borderRadius: 16,
                    background: `${channelColors[ch] || '#95a5a6'}11`,
                    border: `1px solid ${channelColors[ch] || '#95a5a6'}33`,
                  }}
                >
                  <ChannelBadge channel={ch} />
                  <span style={{ fontSize: 13, fontWeight: 600 }}>{cnt}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Filters */}
      <div style={{ display: 'flex', gap: 12, marginBottom: 24 }}>
        <div style={{ position: 'relative', flex: 1 }}>
          <Search
            style={{
              position: 'absolute',
              left: 12,
              top: '50%',
              transform: 'translateY(-50%)',
              width: 16,
              height: 16,
              color: '#9ca3af',
            }}
          />
          <input
            type="text"
            placeholder="Search customers..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              width: '100%',
              padding: '8px 12px 8px 36px',
              border: '1px solid #e5e7eb',
              borderRadius: 8,
              fontSize: 14,
              boxSizing: 'border-box',
            }}
          />
        </div>
        <select
          value={channelFilter}
          onChange={(e) => setChannelFilter(e.target.value)}
          style={{
            padding: '8px 12px',
            border: '1px solid #e5e7eb',
            borderRadius: 8,
            fontSize: 14,
          }}
        >
          <option value="">All Channels</option>
          <option value="whatsapp">WhatsApp</option>
          <option value="email">Email</option>
          <option value="sms">SMS</option>
          <option value="website">Website</option>
          <option value="facebook">Facebook</option>
          <option value="instagram">Instagram</option>
          <option value="telegram">Telegram</option>
        </select>
        <select
          value={stateFilter}
          onChange={(e) => setStateFilter(e.target.value)}
          style={{
            padding: '8px 12px',
            border: '1px solid #e5e7eb',
            borderRadius: 8,
            fontSize: 14,
          }}
        >
          <option value="">All States</option>
          <option value="awaiting_reply">Awaiting Reply</option>
          <option value="engaged">Engaged</option>
          <option value="hot_lead">Hot Lead</option>
          <option value="handed_off">Handed Off</option>
          <option value="closed_won">Closed Won</option>
          <option value="closed_lost">Closed Lost</option>
        </select>
      </div>

      {/* Customer List */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: 48, color: '#6b7280' }}>
          Loading...
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {summary?.recentEntries.map((entry) => (
            <div
              key={entry.customer.customerId}
              onClick={() => selectCustomer(entry.customer.customerId)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 16,
                padding: 16,
                border: '1px solid #e5e7eb',
                borderRadius: 12,
                cursor: 'pointer',
                transition: 'border-color 0.15s',
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLDivElement).style.borderColor = '#6366f1'
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLDivElement).style.borderColor = '#e5e7eb'
              }}
            >
              {/* Avatar */}
              <div
                style={{
                  width: 44,
                  height: 44,
                  borderRadius: '50%',
                  background: '#6366f1',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: 'white',
                  fontSize: 18,
                  fontWeight: 700,
                  flexShrink: 0,
                }}
              >
                {entry.customer.displayName.charAt(0).toUpperCase()}
              </div>

              {/* Info */}
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontWeight: 600, fontSize: 15 }}>
                    {entry.customer.displayName}
                  </span>
                  <div style={{ display: 'flex', gap: 4 }}>
                    {entry.customer.channels.slice(0, 3).map((ch) => (
                      <ChannelBadge key={ch} channel={ch} />
                    ))}
                    {entry.customer.channels.length > 3 && (
                      <span style={{ fontSize: 11, color: '#6b7280' }}>
                        +{entry.customer.channels.length - 3}
                      </span>
                    )}
                  </div>
                </div>
                <div style={{ fontSize: 13, color: '#6b7280', marginTop: 2 }}>
                  {entry.customer.lastMessagePreview && (
                    <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', display: 'block', maxWidth: 400 }}>
                      {entry.customer.lastMessagePreview}
                    </span>
                  )}
                </div>
              </div>

              {/* Meta */}
              <div style={{ textAlign: 'right', flexShrink: 0 }}>
                <div style={{ fontSize: 12, color: '#6b7280' }}>
                  {entry.customer.conversationCount} convs
                </div>
                <div style={{ fontSize: 12, color: '#9ca3af' }}>
                  {formatTime(entry.customer.lastMessageAt)}
                </div>
              </div>

              <ChevronRight style={{ width: 20, height: 20, color: '#9ca3af', flexShrink: 0 }} />
            </div>
          ))}

          {summary?.recentEntries.length === 0 && (
            <div style={{ textAlign: 'center', padding: 48, color: '#6b7280' }}>
              No customers found
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function SummaryCard({ icon, label, value, color }: { icon: React.ReactNode; label: string; value: number; color: string }) {
  return (
    <div
      style={{
        padding: 16,
        border: '1px solid #e5e7eb',
        borderRadius: 12,
        display: 'flex',
        alignItems: 'center',
        gap: 12,
      }}
    >
      <div
        style={{
          width: 40,
          height: 40,
          borderRadius: 10,
          background: `${color}11`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color,
        }}
      >
        {icon}
      </div>
      <div>
        <div style={{ fontSize: 13, color: '#6b7280' }}>{label}</div>
        <div style={{ fontSize: 24, fontWeight: 700 }}>{value}</div>
      </div>
    </div>
  )
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        padding: 12,
        border: '1px solid #e5e7eb',
        borderRadius: 8,
      }}
    >
      <div style={{ fontSize: 12, color: '#6b7280', marginBottom: 4 }}>{label}</div>
      <div style={{ fontSize: 14, fontWeight: 500 }}>{value}</div>
    </div>
  )
}

function TimelineMessage({ message }: { message: UnifiedMessage }) {
  const isInbound = message.direction === 'inbound'
  const Icon = channelIcons[message.channel] || MessageSquare

  return (
    <div
      style={{
        display: 'flex',
        gap: 12,
        padding: 12,
        background: isInbound ? '#f8f9fa' : 'white',
        border: '1px solid #e5e7eb',
        borderRadius: 8,
        borderLeft: `3px solid ${channelColors[message.channel] || '#95a5a6'}`,
      }}
    >
      <div
        style={{
          width: 32,
          height: 32,
          borderRadius: '50%',
          background: `${channelColors[message.channel] || '#95a5a6'}22`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0,
        }}
      >
        <Icon style={{ width: 16, height: 16, color: channelColors[message.channel] || '#95a5a6' }} />
      </div>

      <div style={{ flex: 1 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
          <ChannelBadge channel={message.channel} />
          <span
            style={{
              fontSize: 11,
              fontWeight: 600,
              padding: '1px 6px',
              borderRadius: 4,
              background: isInbound ? '#dbeafe' : '#d1fae5',
              color: isInbound ? '#1e40af' : '#065f46',
            }}
          >
            {isInbound ? 'in' : 'out'}
          </span>
          {message.senderName && (
            <span style={{ fontSize: 12, color: '#6b7280' }}>{message.senderName}</span>
          )}
          <span style={{ fontSize: 11, color: '#9ca3af' }}>
            {new Date(message.sentAt).toLocaleString()}
          </span>
        </div>
        <div style={{ fontSize: 14, lineHeight: 1.5 }}>{message.body}</div>
      </div>
    </div>
  )
}
