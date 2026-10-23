import { Link } from 'react-router-dom'
import { CHANNEL_ICONS } from '../components/icons/BrandIcons'

const CHANNELS = [
  {
    id: 'whatsapp',
    name: 'WhatsApp Business API',
    description: 'Official WhatsApp Business Cloud API with interactive templates, 24h window, and live analytics.',
    icon: 'whatsapp',
    status: 'connected',
    statusLabel: 'Active & Verified',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/whatsapp',
    stats: '100K daily limit · 🟢 Quality Rating: High',
  },
  {
    id: 'live-chat',
    name: 'Website Live Chat',
    description: 'Embeddable customizable chat widget for your website with pre-chat forms, visitor presence, and HMAC identity verification.',
    icon: 'live-chat',
    status: 'connected',
    statusLabel: 'Active & Configured',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/live-chat',
    stats: 'SDK Embed · Pre-chat forms · HMAC-SHA256',
  },
  {
    id: 'email',
    name: 'Email (IMAP / SMTP / Postmark)',
    description: 'Connect shared support emails, parse threaded replies, and send branded email responses.',
    icon: 'email',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/email',
    stats: 'SMTP Outbound · IMAP Polling · Inbound Webhooks · SPF/DKIM',
  },
  {
    id: 'sms',
    name: 'SMS Messaging (Twilio)',
    description: 'Direct SMS outreach and inbound conversation handling via Twilio, A2P 10DLC verification, and carrier compliance.',
    icon: 'sms',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/sms',
    stats: '10DLC Verified · Automated STOP/START · Twilio REST API',
  },
  {
    id: 'facebook',
    name: 'Facebook Messenger',
    description: 'Manage Facebook business page messages, comments, and customer replies from one inbox.',
    icon: 'facebook',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/facebook',
    stats: 'Meta Graph API v21.0 · Page IDs · Icebreakers · 24h Window',
  },
  {
    id: 'instagram',
    name: 'Instagram Direct Messages',
    description: 'Handle Instagram DMs, story mentions, and post replies in your unified team inbox.',
    icon: 'instagram',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/instagram',
    stats: 'Professional DMs · Story Mentions Auto-Reply · Icebreakers',
  },
  {
    id: 'telegram',
    name: 'Telegram Bot API',
    description: 'Connect Telegram support bots and manage high-volume customer messages securely.',
    icon: 'telegram',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-sky-100 text-sky-800 border-sky-200',
    link: '/channels/telegram',
    stats: 'Bot Token & Secret · Commands · Live Webhook Streaming',
  },
  {
    id: 'line',
    name: 'LINE Messaging API',
    description: 'Support customers across Japan, Taiwan, and Thailand with official LINE channel integration.',
    icon: 'line',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/line',
    stats: 'Channel Secret & Push · Quick Replies · Webhook Verification',
  },
  {
    id: 'tiktok',
    name: 'TikTok Direct Messages & Comments',
    description: 'Manage incoming TikTok customer conversations and video comment moderation directly in your unified inbox.',
    icon: 'tiktok',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/tiktok',
    stats: 'TikTok Business v2 · DMs & Comment Auto-Replies · HMAC Verification',
  },
  {
    id: 'voice',
    name: 'Voice & WebRTC Calling',
    description: 'Phone calls and audio conferencing directly inside the agent inbox with Twilio Voice & WebRTC.',
    icon: 'voice',
    status: 'connected',
    statusLabel: 'Active & Connected',
    statusColor: 'bg-emerald-100 text-emerald-800 border-emerald-200',
    link: '/channels/voice',
    stats: 'Browser Softphone · IVR · Dual-Track Audio & Transcripts',
  },
]

export function ChannelsHub() {
  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-slate-900">Channels &amp; Inboxes</h1>
            <span className="rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-semibold text-slate-700">
              Omnichannel Hub
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-500">
            Connect all customer support channels and manage all conversations from a single, unified inbox.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Link
            to="/channels/whatsapp"
            className="rounded-md bg-emerald-600 px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-emerald-700 shadow-xs flex items-center gap-1.5"
          >
            <CHANNEL_ICONS.whatsapp className="h-3.5 w-3.5" /> Configure WhatsApp
          </Link>
        </div>
      </div>

      {/* Grid of Channels */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {CHANNELS.map((ch) => {
          const Icon = CHANNEL_ICONS[ch.icon]
          return (
          <div
            key={ch.id}
            className={`glass-card p-5 transition-all flex flex-col justify-between ${
              ch.status === 'connected' ? 'ring-1 ring-emerald-200' : ''
            }`}
          >
            <div>
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-3">
                  <span className="flex h-11 w-11 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
                    <Icon className="h-6 w-6" />
                  </span>
                  <div>
                    <h3 className="text-sm font-bold text-slate-900">{ch.name}</h3>
                    <span className="text-[11px] text-slate-400 font-mono">{ch.stats}</span>
                  </div>
                </div>
                <span className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider ${ch.statusColor}`}>
                  {ch.statusLabel}
                </span>
              </div>

              <p className="mt-3 text-xs text-slate-600 leading-relaxed">
                {ch.description}
              </p>
            </div>

            <div className="mt-4 pt-3 border-t border-slate-100 flex items-center justify-between">
              <span className="text-[11px] text-slate-400">
                {ch.status === 'connected' ? 'Primary Active Channel' : 'Unified Inbox Ready'}
              </span>
              {ch.link && ch.link !== '#' ? (
                <Link
                  to={ch.link}
                  className="rounded-md bg-slate-900 px-3 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs"
                >
                  Configure →
                </Link>
              ) : (
                <button
                  disabled
                  className="rounded-md border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-400 cursor-not-allowed"
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
