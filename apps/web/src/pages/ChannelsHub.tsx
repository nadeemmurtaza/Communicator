import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { ArrowRight, RefreshCw } from '../components/icons'
import { CHANNEL_ICONS } from '../components/icons/channels'
import { api, type Inbox, type WhatsAppChannelData, type ChannelHealth } from '../lib/api'

interface BaseChannelDef {
  id: string
  name: string
  description: string
  icon: keyof typeof CHANNEL_ICONS
  link: string
  defaultConnectedStats: string
}

const BASE_CHANNELS: BaseChannelDef[] = [
  {
    id: 'whatsapp',
    name: 'WhatsApp Business API',
    description: 'Official WhatsApp Business Cloud API with interactive templates, 24h window, and live analytics.',
    icon: 'whatsapp',
    link: '/channels/whatsapp',
    defaultConnectedStats: '250 / day · Quality: High',
  },
  {
    id: 'live-chat',
    name: 'Website Live Chat',
    description: 'Embeddable customizable chat widget for your website with pre-chat forms, visitor presence, and HMAC identity verification.',
    icon: 'live-chat',
    link: '/channels/live-chat',
    defaultConnectedStats: 'SDK Embed · Pre-chat forms · HMAC-SHA256',
  },
  {
    id: 'email',
    name: 'Email (IMAP / SMTP / Postmark)',
    description: 'Connect shared support emails, parse threaded replies, and send branded email responses.',
    icon: 'email',
    link: '/channels/email',
    defaultConnectedStats: 'SMTP Outbound · IMAP Polling · Inbound Webhooks · SPF/DKIM',
  },
  {
    id: 'sms',
    name: 'SMS Messaging (Twilio)',
    description: 'Direct SMS outreach and inbound conversation handling via Twilio, A2P 10DLC verification, and carrier compliance.',
    icon: 'sms',
    link: '/channels/sms',
    defaultConnectedStats: '10DLC Verified · Automated STOP/START · Twilio REST API',
  },
  {
    id: 'facebook',
    name: 'Facebook Messenger',
    description: 'Manage Facebook business page messages, comments, and customer replies from one inbox.',
    icon: 'facebook',
    link: '/channels/facebook',
    defaultConnectedStats: 'Meta Graph API v21.0 · Page IDs · Icebreakers · 24h Window',
  },
  {
    id: 'instagram',
    name: 'Instagram Direct Messages',
    description: 'Handle Instagram DMs, story mentions, and post replies in your unified team inbox.',
    icon: 'instagram',
    link: '/channels/instagram',
    defaultConnectedStats: 'Professional DMs · Story Mentions Auto-Reply · Icebreakers',
  },
  {
    id: 'telegram',
    name: 'Telegram Bot API',
    description: 'Connect Telegram support bots and manage high-volume customer messages securely.',
    icon: 'telegram',
    link: '/channels/telegram',
    defaultConnectedStats: 'Bot Token & Secret · Commands · Live Webhook Streaming',
  },
  {
    id: 'line',
    name: 'LINE Messaging API',
    description: 'Support customers across Japan, Taiwan, and Thailand with official LINE channel integration.',
    icon: 'line',
    link: '/channels/line',
    defaultConnectedStats: 'Channel Secret & Push · Quick Replies · Webhook Verification',
  },
  {
    id: 'tiktok',
    name: 'TikTok Direct Messages & Comments',
    description: 'Manage incoming TikTok customer conversations and video comment moderation directly in your unified inbox.',
    icon: 'tiktok',
    link: '/channels/tiktok',
    defaultConnectedStats: 'TikTok Business v2 · DMs & Comment Auto-Replies · HMAC Verification',
  },
  {
    id: 'voice',
    name: 'Voice & WebRTC Calling',
    description: 'Phone calls and audio conferencing directly inside the agent inbox with Twilio Voice & WebRTC.',
    icon: 'voice',
    link: '/channels/voice',
    defaultConnectedStats: 'Browser Softphone · IVR · Dual-Track Audio & Transcripts',
  },
]

function checkChannelConfigured(
  channelId: string,
  inboxes: Inbox[],
  healthMap: Record<string, ChannelHealth>,
  waData?: WhatsAppChannelData | null
): boolean {
  if (healthMap[channelId]) {
    return healthMap[channelId].isConnected
  }

  if (channelId === 'whatsapp') {
    return Boolean(waData?.health?.isConnected)
  }

  const typeMap: Record<string, string[]> = {
    'live-chat': ['website', 'live-chat', 'live_chat'],
    email: ['email'],
    sms: ['sms'],
    facebook: ['facebook'],
    instagram: ['instagram'],
    telegram: ['telegram'],
    line: ['line'],
    tiktok: ['tiktok'],
    voice: ['voice', 'voice_sip'],
  }

  const targetTypes = typeMap[channelId] || [channelId]
  const inbox = inboxes.find((i) => targetTypes.includes(i.channelType || (i as any).channel))
  if (!inbox || !inbox.isActive) return false

  const creds = inbox.credentials || {}
  return Object.keys(creds).length > 0
}

export function ChannelsHub() {
  const [inboxes, setInboxes] = useState<Inbox[]>([])
  const [channelsHealth, setChannelsHealth] = useState<ChannelHealth[]>([])
  const [waData, setWaData] = useState<WhatsAppChannelData | null>(null)
  const [loading, setLoading] = useState(true)

  const loadChannels = async () => {
    try {
      setLoading(true)
      const [inboxList, whatsappRes, healthList] = await Promise.all([
        api.listInboxes().catch(() => []),
        api.getWhatsAppChannel().catch(() => null),
        api.getChannelsHealth().catch(() => []),
      ])
      setInboxes(inboxList)
      setWaData(whatsappRes)
      setChannelsHealth(healthList)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadChannels()
  }, [])

  const healthMap: Record<string, ChannelHealth> = {}
  for (const h of channelsHealth) {
    healthMap[h.channel] = h
  }

  const channels = BASE_CHANNELS.map((ch) => {
    const isConfigured = checkChannelConfigured(ch.id, inboxes, healthMap, waData)
    const health = healthMap[ch.id]

    if (ch.id === 'whatsapp') {
      if (isConfigured) {
        return {
          ...ch,
          status: 'connected',
          statusLabel: health?.statusLabel || 'Active & Verified',
          statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
          stats: `${waData?.health?.messagingLimitTier || '250 / day'} · Quality: ${waData?.health?.qualityRating || 'High'}`,
        }
      }
      return {
        ...ch,
        status: 'not_configured',
        statusLabel: 'Not Configured',
        statusColor: 'bg-amber-50 text-amber-800 border-amber-200',
        stats: 'Setup required · No active credentials',
      }
    }

    if (isConfigured) {
      return {
        ...ch,
        status: 'connected',
        statusLabel: health?.statusLabel || 'Active & Connected',
        statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
        stats: ch.defaultConnectedStats,
      }
    }

    return {
      ...ch,
      status: 'not_configured',
      statusLabel: 'Not Configured',
      statusColor: 'bg-amber-50 text-amber-800 border-amber-200',
      stats: 'Setup required · No active credentials',
    }
  })

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-[var(--color-ink)]">Channels &amp; Inboxes</h1>
            <span className="rounded-full bg-[var(--color-accent-muted)] px-2.5 py-0.5 text-xs font-semibold text-[var(--color-ink)]">
              Omnichannel Hub
            </span>
            {loading && (
              <span className="flex items-center gap-1 text-xs text-[var(--color-ink-faint)]">
                <RefreshCw className="h-3 w-3 animate-spin" /> Syncing channels...
              </span>
            )}
          </div>
          <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
            Connect all customer support channels and manage all conversations from a single, unified inbox.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Link
            to="/channels/whatsapp"
            className="btn-primary"
          >
            <CHANNEL_ICONS.whatsapp className="h-3.5 w-3.5" /> Configure WhatsApp
          </Link>
        </div>
      </div>

      {/* Grid of Channels */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {channels.map((ch) => {
          const Icon = CHANNEL_ICONS[ch.icon]
          const isConnected = ch.status === 'connected'
          return (
            <div
              key={ch.id}
              className={`rounded-[var(--radius-lg)] border bg-[var(--color-surface)] p-5 transition-all duration-150 flex flex-col justify-between hover:border-[var(--color-accent)] ${
                isConnected ? 'border-emerald-200' : 'border-[var(--color-border)]'
              }`}
            >
              <div>
                <div className="flex items-start justify-between gap-2">
                  <div className="flex items-center gap-3">
                    <span className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--color-page)] text-[var(--color-ink)]">
                      <Icon className="h-5 w-5" />
                    </span>
                    <div>
                      <h3 className="text-sm font-bold text-[var(--color-ink)]">{ch.name}</h3>
                      <span className="text-[11px] text-[var(--color-ink-faint)]">{ch.stats}</span>
                    </div>
                  </div>
                  <span
                    className={`shrink-0 rounded-full border px-2 py-0.5 text-[10px] font-bold uppercase tracking-[0.5px] ${ch.statusColor}`}
                  >
                    {ch.statusLabel}
                  </span>
                </div>

                <p className="mt-3 text-xs text-[var(--color-ink-muted)] leading-relaxed">{ch.description}</p>
              </div>

              <div className="mt-4 pt-3 border-t border-[var(--color-border)] flex items-center justify-between">
                <span className="text-[11px] text-[var(--color-ink-faint)]">
                  {isConnected ? 'Active' : 'Setup required'}
                </span>
                {ch.link && ch.link !== '#' ? (
                  <Link
                    to={ch.link}
                    className="btn-primary"
                  >
                    Configure <ArrowRight className="h-3 w-3" />
                  </Link>
                ) : (
                  <button
                    disabled
                    className="btn-ghost disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Configure
                  </button>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
