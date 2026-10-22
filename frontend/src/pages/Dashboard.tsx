import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type AgentPresenceItem,
  type CannedResponse,
  type CaptainMonitorsData,
  type CaptainSuggestedReply,
  type CaptainSummary,
  type CaptainTriageResult,
  type CollisionCheckResult,
  type CopilotApprovalRequest,
  type CapacityOverview,
  type MacroItem,
  type RoutingAgent,
  type SLABreachedConversation,
  type SLAData,
  type SLAMetrics,
  type SLAPolicy,
  type LinkedIssue,
  type SlackSyncStatus,
  type StripeBillingLog,
  type StripeBillingProfile,
  type StripeInvoice,
} from '../lib/api'

interface MessageItem {
  id: string
  sender: 'customer' | 'agent' | 'note'
  text: string
  time: string
  status?: 'sent' | 'delivered' | 'read'
  agentName?: string
  agentAvatar?: string
}

interface ConversationItem {
  id: string
  name: string
  avatar: string
  channel: string
  channelType: 'facebook' | 'instagram' | 'whatsapp' | 'website' | 'email' | 'telegram' | 'line' | 'sms' | 'voice'
  assignedAgent: string
  agentAvatar: string
  timeAgo: string
  preview: string
  unreadCount?: number
  sla?: string
  slaData?: SLAData
  tags: Array<{ name: string; color: string }>
  email: string
  phone: string
  company: string
  location: string
  title: string
  bio: string
  priority: 'Low' | 'Medium' | 'High' | 'Urgent'
  team: string
  status: 'Open' | 'Pending' | 'Resolved'
  resolutionType?: 'autonomous_no_handoff' | 'human_assisted' | null
  campaign?: {
    name: string
    templateName: string
    sentAt: string
    status: string
    messageText: string
  } | null
  captainDraft?: string | null
  memories?: Array<{
    id: string
    fact: string
    category: string
    notedAt: string
  }>
  messages: MessageItem[]
}

const INITIAL_CONVERSATIONS: ConversationItem[] = [
  {
    id: 'conv-maya',
    name: 'Maya Chen',
    avatar: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80',
    channel: 'WhatsApp Support',
    channelType: 'whatsapp',
    assignedAgent: 'Captain (AI Agent)',
    agentAvatar: 'https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=150&auto=format&fit=crop&q=80',
    timeAgo: 'Just now',
    preview: 'Does the offer include the new linen collection?',
    unreadCount: 1,
    sla: '3h 10m',
    tags: [
      { name: 'vip-customers', color: 'bg-purple-500' },
      { name: 'campaign-reply', color: 'bg-emerald-500' },
    ],
    email: 'maya.chen@luxuryliving.com',
    phone: '+14155552480',
    company: 'Luxury Living Decor',
    location: 'Seattle, United States 🇺🇸',
    title: 'Lead Buyer & VIP Member',
    bio: 'VIP loyalty member since 2023. High purchase volume customer who prefers prompt WhatsApp support.',
    priority: 'High',
    team: 'customer support',
    status: 'Open',
    resolutionType: 'autonomous_no_handoff',
    campaign: {
      name: 'Spring sale early access',
      templateName: 'spring_sale_offer',
      sentAt: 'Mar 12, 10:00 AM',
      status: 'Delivered · Read',
      messageText: 'Hi Maya, our spring sale starts Friday. As a VIP customer you get early access and 20% off until Sunday.',
    },
    captainDraft: 'Yes, the 20% early access discount covers the full linen collection until Sunday.',
    memories: [
      {
        id: 'mem-1',
        fact: 'Their team prefers email over calls',
        category: 'channel_preference',
        notedAt: 'Mar 12',
      },
      {
        id: 'mem-2',
        fact: 'VIP loyalty tier; requires expedited courier delivery',
        category: 'constraint',
        notedAt: 'Feb 10',
      },
    ],
    messages: [
      {
        id: 'cm-camp',
        sender: 'agent',
        text: 'Hi Maya, our spring sale starts Friday. As a VIP customer you get early access and 20% off until Sunday.',
        time: 'Mar 12, 10:00 AM',
        status: 'read',
        agentName: 'WhatsApp Campaign: Spring sale early access',
      },
      {
        id: 'cm-reply',
        sender: 'customer',
        text: 'Does the offer include the new linen collection?',
        time: 'Mar 12, 10:05 AM',
      },
    ],
  },
  {
    id: 'conv-4',
    name: 'Lorrie Trosdall',
    avatar: 'https://images.unsplash.com/photo-1544005313-94ddf0286df2?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works Website',
    channelType: 'website',
    assignedAgent: 'Riley Brooks',
    agentAvatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80',
    timeAgo: '2h · 35m',
    preview: "↩ Anytime — let us know if it happens...",
    sla: '5h 51m',
    tags: [{ name: 'login-issue', color: 'bg-amber-500' }],
    email: 'lorrie@northwind.inc',
    phone: '+14155550100',
    company: 'Northwind Traders',
    location: 'San Francisco, United States 🇺🇸',
    title: 'Head of Operations at Northwind Traders',
    bio: 'Head of Operations at Northwind Traders. Customer since 2023 and an internal product champion.',
    priority: 'High',
    team: 'technical support',
    status: 'Open',
    messages: [
      {
        id: 'm1',
        sender: 'customer',
        text: "Hi, I changed my password yesterday and now I can't sign in on the web. The mobile app still works.",
        time: 'Aug 5, 8:55 PM',
      },
      {
        id: 'm2',
        sender: 'agent',
        text: "Hi Lorrie, I'm Riley from support. It sounds like the browser kept an old session. Could you sign out, clear the site data, and try again?",
        time: 'Aug 5, 9:14 PM',
        status: 'read',
        agentName: 'Riley Brooks',
        agentAvatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80',
      },
      {
        id: 'm3',
        sender: 'customer',
        text: 'That worked — I can sign in now. Do I need to do this on my other devices?',
        time: 'Aug 5, 9:24 PM',
      },
      {
        id: 'm4',
        sender: 'agent',
        text: 'Only if they still show the login error. Since the mobile app is working, you can leave it as is.',
        time: 'Aug 5, 9:46 PM',
        status: 'read',
        agentName: 'Riley Brooks',
        agentAvatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80',
      },
      {
        id: 'm5',
        sender: 'customer',
        text: 'Perfect. Thanks for the quick help!',
        time: 'Aug 5, 10:02 PM',
      },
    ],
  },
  {
    id: 'conv-1',
    name: 'Candice Matherson',
    avatar: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works Facebook',
    channelType: 'facebook',
    assignedAgent: 'Jamie Cole',
    agentAvatar: 'https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=150&auto=format&fit=crop&q=80',
    timeAgo: '3h · 6m',
    preview: "↩ Will do — I'll follow up here.",
    unreadCount: 1,
    tags: [
      { name: 'bug', color: 'bg-rose-500' },
      { name: 'feature-request', color: 'bg-purple-500' },
    ],
    email: 'candice.m@acme.org',
    phone: '+14155550199',
    company: 'Acme Systems',
    location: 'Austin, United States 🇺🇸',
    title: 'Senior Systems Architect',
    bio: 'Lead architect evaluating webhook integrations and enterprise WhatsApp APIs.',
    priority: 'Medium',
    team: 'developer support',
    status: 'Open',
    messages: [
      {
        id: 'cm1',
        sender: 'customer',
        text: 'Can we schedule an engineering review on webhook delivery retries?',
        time: 'Aug 5, 7:10 PM',
      },
      {
        id: 'cm2',
        sender: 'agent',
        text: "Will do — I'll follow up here.",
        time: 'Aug 5, 7:22 PM',
        status: 'read',
      },
    ],
  },
  {
    id: 'conv-2',
    name: 'Ophelia Folkard',
    avatar: 'https://images.unsplash.com/photo-1517841905240-472988babdf9?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works Instagram',
    channelType: 'instagram',
    assignedAgent: 'Morgan Hayes',
    agentAvatar: 'https://images.unsplash.com/photo-1570295999919-56ceb5ecca61?w=150&auto=format&fit=crop&q=80',
    timeAgo: '3h · 13m',
    preview: '↩ My pleasure — enjoy the new tier.',
    tags: [{ name: 'billing', color: 'bg-emerald-500' }],
    email: 'ophelia@folkarddesign.co',
    phone: '+14155550144',
    company: 'Folkard Design Studio',
    location: 'New York, United States 🇺🇸',
    title: 'Founder & Principal Designer',
    bio: 'Creative director managing enterprise brand communication accounts.',
    priority: 'Low',
    team: 'customer success',
    status: 'Open',
    messages: [
      {
        id: 'of1',
        sender: 'customer',
        text: 'Thanks for upgrading our quota ahead of the campaign launch!',
        time: 'Aug 5, 7:05 PM',
      },
      {
        id: 'of2',
        sender: 'agent',
        text: 'My pleasure — enjoy the new tier.',
        time: 'Aug 5, 7:15 PM',
        status: 'read',
      },
    ],
  },
  {
    id: 'conv-telegram-1',
    name: 'Pavel Durov',
    avatar: 'https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=150&auto=format&fit=crop&q=80',
    channel: 'Telegram Support Bot',
    channelType: 'telegram',
    assignedAgent: 'Captain (AI Agent)',
    agentAvatar: 'https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=150&auto=format&fit=crop&q=80',
    timeAgo: '12m · ago',
    preview: '/help Can I integrate inline query bots with Chatwoot?',
    unreadCount: 1,
    sla: '4h 15m',
    tags: [
      { name: 'telegram-bot', color: 'bg-sky-500' },
      { name: 'api-support', color: 'bg-indigo-500' },
    ],
    email: 'pavel@telegram-users.net',
    phone: '+447911123456',
    company: 'Fintech Europe',
    location: 'Dubai, UAE 🇦🇪',
    title: 'Lead Community Engineer',
    bio: 'Telegram power user integrating custom bot commands and Webhook notifications.',
    priority: 'Medium',
    team: 'customer support',
    status: 'Open',
    messages: [
      {
        id: 'tg-1',
        sender: 'customer',
        text: '/help Can I integrate inline query bots with Chatwoot?',
        time: '11:42 AM',
      },
      {
        id: 'tg-2',
        sender: 'agent',
        text: 'Hi Pavel! Yes, Telegram Bot API commands (/start, /help, /status) stream straight into our shared inbox, and inline queries or callbacks trigger interactive replies seamlessly.',
        time: '11:45 AM',
        status: 'read',
        agentName: 'Captain (AI Agent)',
      },
    ],
  },
  {
    id: 'conv-line-1',
    name: 'Kenji Sato',
    avatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=150&auto=format&fit=crop&q=80',
    channel: 'LINE Official Channel',
    channelType: 'line',
    assignedAgent: 'Riley Brooks',
    agentAvatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80',
    timeAgo: '28m · ago',
    preview: 'Selected quick reply: [Check Order Status]',
    sla: '2h 45m',
    tags: [
      { name: 'line-official', color: 'bg-emerald-500' },
      { name: 'japan-market', color: 'bg-rose-500' },
    ],
    email: 'kenji.sato@tokyotech.co.jp',
    phone: '+819012345678',
    company: 'Tokyo Logistics K.K.',
    location: 'Tokyo, Japan 🇯🇵',
    title: 'Supply Chain Operations Director',
    bio: 'LINE Official Account enterprise customer using automated push notifications and rich menus.',
    priority: 'High',
    team: 'technical support',
    status: 'Open',
    messages: [
      {
        id: 'ln-1',
        sender: 'customer',
        text: 'Selected quick reply: [Check Order Status]',
        time: '11:20 AM',
      },
      {
        id: 'ln-2',
        sender: 'agent',
        text: 'Konnichiwa Kenji! Your order #TYO-9821 is currently out for delivery via Yamato Transport. Expected arrival by 4:00 PM today.',
        time: '11:22 AM',
        status: 'read',
        agentName: 'Riley Brooks',
      },
    ],
  },
  {
    id: 'conv-voice-1',
    name: 'Harrison Sterling',
    avatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=150&auto=format&fit=crop&q=80',
    channel: 'Voice Hotline (+1 415 555-0199)',
    channelType: 'voice',
    assignedAgent: 'Morgan Hayes',
    agentAvatar: 'https://images.unsplash.com/photo-1570295999919-56ceb5ecca61?w=150&auto=format&fit=crop&q=80',
    timeAgo: '4m · ago',
    preview: '📞 Call Completed (3m 42s) · "Need to update our carrier SIP trunk configuration."',
    unreadCount: 0,
    sla: '1h 20m',
    tags: [
      { name: 'voice-call', color: 'bg-teal-500' },
      { name: 'enterprise-sip', color: 'bg-indigo-500' },
    ],
    email: 'harrison@sterlingcap.com',
    phone: '+14155550188',
    company: 'Sterling Capital Advisors',
    location: 'Chicago, United States 🇺🇸',
    title: 'Managing Director & Partner',
    bio: 'Enterprise client utilizing high-concurrency SIP trunking and in-inbox phone support.',
    priority: 'Urgent',
    team: 'technical support',
    status: 'Open',
    messages: [
      {
        id: 'vc-1',
        sender: 'customer',
        text: '📞 Inbound Call from +14155550188 (Selected option [2: Support] · Connected via WebRTC)',
        time: '11:38 AM',
      },
      {
        id: 'vc-2',
        sender: 'agent',
        text: 'Morgan Hayes answered call. Duration: 3m 42s · 🎙️ Dual-Channel Recording Saved',
        time: '11:42 AM',
        status: 'read',
        agentName: 'Morgan Hayes',
      },
      {
        id: 'vc-3',
        sender: 'agent',
        text: '📝 AI Transcript: "Customer requested an emergency update to their carrier SIP trunk IP whitelist to enable 50 concurrent outbound channels. Whitelist applied successfully during call."',
        time: '11:43 AM',
        status: 'read',
        agentName: 'Captain AI (Voice Summary)',
      },
    ],
  },
  {
    id: 'conv-3',
    name: 'Willy Castelot',
    avatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works WhatsApp',
    channelType: 'whatsapp',
    assignedAgent: 'Taylor Quinn',
    agentAvatar: 'https://images.unsplash.com/photo-1522075469751-3a6694fb2f61?w=150&auto=format&fit=crop&q=80',
    timeAgo: '4h · 22m',
    preview: '↩ Anytime — glad you got everything.',
    tags: [{ name: 'how-to', color: 'bg-blue-500' }],
    email: 'willy@castelot.fr',
    phone: '+33612345678',
    company: 'Castelot Logistics',
    location: 'Paris, France 🇫🇷',
    title: 'Supply Chain Operations Manager',
    bio: 'Oversees fleet tracking notifications and automated WhatsApp dispatch updates.',
    priority: 'Medium',
    team: 'technical support',
    status: 'Open',
    messages: [
      {
        id: 'wc1',
        sender: 'customer',
        text: 'Does WhatsApp Cloud API support multi-product catalog messages in Europe?',
        time: 'Aug 5, 6:00 PM',
      },
      {
        id: 'wc2',
        sender: 'agent',
        text: 'Yes! It is fully supported across all Meta business regions.',
        time: 'Aug 5, 6:10 PM',
        status: 'read',
      },
      {
        id: 'wc3',
        sender: 'agent',
        text: 'Anytime — glad you got everything.',
        time: 'Aug 5, 6:12 PM',
        status: 'read',
      },
    ],
  },
  {
    id: 'conv-5',
    name: 'Elisabeth Derington',
    avatar: 'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works Email',
    channelType: 'email',
    assignedAgent: 'Casey Reed',
    agentAvatar: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80',
    timeAgo: '11h · 1h',
    preview: 'Re: Error when saving my settings',
    tags: [{ name: 'bug', color: 'bg-rose-500' }],
    email: 'e.derington@globalventures.com',
    phone: '+442079460912',
    company: 'Global Ventures',
    location: 'London, United Kingdom 🇬🇧',
    title: 'VP Operations',
    bio: 'Oversees operational workflows across European subsidiaries.',
    priority: 'High',
    team: 'developer support',
    status: 'Open',
    messages: [
      {
        id: 'ed1',
        sender: 'customer',
        text: 'Re: Error when saving my settings. The modal throws 403 on tenant configuration.',
        time: 'Aug 5, 11:15 AM',
      },
    ],
  },
  {
    id: 'conv-6',
    name: 'Olia Olenchenko',
    avatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works Website',
    channelType: 'website',
    assignedAgent: 'Alex Morgan',
    agentAvatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=150&auto=format&fit=crop&q=80',
    timeAgo: '4h · 2h',
    preview: 'Appreciate the quick explanation!',
    tags: [
      { name: 'bug', color: 'bg-rose-500' },
      { name: 'escalation', color: 'bg-amber-500' },
    ],
    email: 'olia.o@polaris.tech',
    phone: '+14155550188',
    company: 'Polaris Technologies',
    location: 'Seattle, United States 🇺🇸',
    title: 'Infrastructure Lead',
    bio: 'Leads cloud reliability engineering and notification gateways.',
    priority: 'Urgent',
    team: 'technical support',
    status: 'Open',
    messages: [
      {
        id: 'oo1',
        sender: 'customer',
        text: 'Appreciate the quick explanation! The patch resolves our failover route.',
        time: 'Aug 5, 6:30 PM',
      },
    ],
  },
  {
    id: 'conv-7',
    name: 'Nathaniel Vannuchi',
    avatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=150&auto=format&fit=crop&q=80',
    channel: 'Bay Works Facebook',
    channelType: 'facebook',
    assignedAgent: 'Sam Carter',
    agentAvatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=150&auto=format&fit=crop&q=80',
    timeAgo: '5h · 3h',
    preview: '↩ Of course — enable auto-update ...',
    unreadCount: 1,
    tags: [{ name: 'bug', color: 'bg-rose-500' }],
    email: 'nathaniel.v@vannuchigroup.com',
    phone: '+14155550177',
    company: 'Vannuchi Consulting',
    location: 'Chicago, United States 🇺🇸',
    title: 'Managing Director',
    bio: 'Client engagement partner managing high-volume service inquiries.',
    priority: 'Medium',
    team: 'customer success',
    status: 'Open',
    messages: [
      {
        id: 'nv1',
        sender: 'customer',
        text: 'Should we enable auto-update on the communication daemon?',
        time: 'Aug 5, 5:10 PM',
      },
      {
        id: 'nv2',
        sender: 'agent',
        text: 'Of course — enable auto-update to ensure certificate rotations succeed.',
        time: 'Aug 5, 5:18 PM',
        status: 'read',
      },
    ],
  },
]

const DEFAULT_MACROS: MacroItem[] = [
  {
    id: 'default-macro-resolve',
    name: 'Resolve Ticket + Send CSAT Survey',
    description: 'Marks conversation as resolved, appends resolved tag, sends customer satisfaction survey link, and thanks customer.',
    actions: [
      { type: 'send_message', body: 'Thank you for reaching out! Your issue has been resolved. Please let us know how we did.' },
      { type: 'send_csat_survey' },
      { type: 'add_tags', tags: ['resolved', 'csat-sent'] },
      { type: 'resolve_conversation', resolution_type: 'human_assisted' },
    ],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'default-macro-escalate',
    name: 'Escalate to Tier 2 Engineering',
    description: 'Bumps priority to urgent, assigns to engineering team, tags as escalated, and logs internal handover note.',
    actions: [
      { type: 'change_priority', priority: 'urgent' },
      { type: 'assign_team', team_name: 'Engineering' },
      { type: 'add_tags', tags: ['escalated', 'tier-2'] },
      { type: 'add_private_note', note: 'Escalated by agent for Tier 2 technical diagnosis and log inspection.' },
    ],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'default-macro-refund',
    name: 'Refund Approved & Order Credited',
    description: 'Dispatches refund notification, tags as refund-processed, sets priority to low, and closes ticket.',
    actions: [
      { type: 'send_message', body: 'Your refund has been approved and issued to your original payment method within 3-5 business days.' },
      { type: 'add_tags', tags: ['refund-processed', 'finance-verified'] },
      { type: 'change_priority', priority: 'low' },
      { type: 'resolve_conversation', resolution_type: 'human_assisted' },
    ],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'default-macro-snooze',
    name: 'Request Info & Snooze 24h',
    description: 'Asks customer for additional reproduction steps, sets status to pending, and snoozes for 24 hours.',
    actions: [
      { type: 'send_message', body: 'Could you please provide your device model, browser version, and a screenshot so we can investigate further?' },
      { type: 'snooze_conversation', duration_hours: 24 },
      { type: 'add_tags', tags: ['waiting-customer', 'snoozed-24h'] },
    ],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
]

export function Dashboard() {
  const [conversations, setConversations] = useState<ConversationItem[]>(INITIAL_CONVERSATIONS)
  const [selectedId, setSelectedId] = useState<string>('conv-maya')
  const [activeTab, setActiveTab] = useState<'Mine' | 'Unassigned' | 'All' | 'Autonomous' | 'Approvals'>('All')
  const [composerMode, setComposerMode] = useState<'reply' | 'note'>('reply')
  const [replyText, setReplyText] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [copiedField, setCopiedField] = useState<string | null>(null)
  const [showActionsAccordion, setShowActionsAccordion] = useState(true)
  const [showMacrosAccordion, setShowMacrosAccordion] = useState(true)
  const [showNavFolders, setShowNavFolders] = useState(true)
  const [showNavTeams, setShowNavTeams] = useState(true)
  const [showNavChannels, setShowNavChannels] = useState(true)
  const [showRightSidebar, setShowRightSidebar] = useState(true)

  // One-Click Macros & Workflows State
  const [macros, setMacros] = useState<MacroItem[]>(DEFAULT_MACROS)
  const [macroLoading, setMacroLoading] = useState(false)
  const [activeExecutingMacroId, setActiveExecutingMacroId] = useState<string | null>(null)
  const [macroBanner, setMacroBanner] = useState<{
    name: string
    count: number
    tags?: string[]
    priority?: string
    state?: string
  } | null>(null)
  const [showCreateMacroModal, setShowCreateMacroModal] = useState(false)
  const [newMacroName, setNewMacroName] = useState('')
  const [newMacroDesc, setNewMacroDesc] = useState('')
  const [newMacroPriority, setNewMacroPriority] = useState<'low' | 'medium' | 'high' | 'urgent'>('high')
  const [newMacroTag, setNewMacroTag] = useState('')
  const [newMacroNote, setNewMacroNote] = useState('')
  const [newMacroMessage, setNewMacroMessage] = useState('')
  const [newMacroResolve, setNewMacroResolve] = useState(false)
  const [newMacroCSAT, setNewMacroCSAT] = useState(false)

  // Dynamic SLA Management & Breach Detection State
  const [showSLAAccordion, setShowSLAAccordion] = useState(true)
  const [slaPolicies, setSlaPolicies] = useState<SLAPolicy[]>([])
  const [slaMetrics, setSlaMetrics] = useState<SLAMetrics | null>(null)
  const [slaBreaches, setSlaBreaches] = useState<SLABreachedConversation[]>([])
  const [showSLAMonitorModal, setShowSLAMonitorModal] = useState(false)
  const [isEvaluatingSLA, setIsEvaluatingSLA] = useState(false)

  // Auto-Assignment & Capacity Routing State
  const [routingAgents, setRoutingAgents] = useState<RoutingAgent[]>([])
  const [capacityOverview, setCapacityOverview] = useState<CapacityOverview | null>(null)
  const [myAgentStatus, setMyAgentStatus] = useState<'online' | 'busy' | 'offline'>('online')
  const [isAutoAssigning, setIsAutoAssigning] = useState(false)
  const [showCapacityModal, setShowCapacityModal] = useState(false)
  const [routingToast, setRoutingToast] = useState<{ message: string; isOverflow?: boolean } | null>(null)

  // Omnichannel & Collaboration State
  const [cannedResponses, setCannedResponses] = useState<CannedResponse[]>([
    { id: '1', shortCode: 'greet', short_code: 'greet', content: 'Hi {{customer.first_name}}! Thanks for reaching out to Bay Works. How can I assist you today?', category: 'greetings', createdAt: '' },
    { id: '2', shortCode: 'pricing', short_code: 'pricing', content: 'Hi {{customer.first_name}}, our plans start with our Standard tier with dedicated support. Would you like a breakdown of our rates or a custom quote?', category: 'sales', createdAt: '' },
    { id: '3', shortCode: 'refund', short_code: 'refund', content: 'Standard refund requests for ticket #{{ticket.id}} are processed within 3 to 5 business days back to your original payment method.', category: 'billing', createdAt: '' },
    { id: '4', shortCode: 'hours', short_code: 'hours', content: "Our support team is available Monday through Friday from 9:00 AM to 6:00 PM. We'll follow up on your request promptly!", category: 'support', createdAt: '' },
    { id: '5', shortCode: 'followup', short_code: 'followup', content: 'Just checking in to make sure everything is working well for you! Feel free to reply here anytime if you need more help.', category: 'general', createdAt: '' },
    { id: '6', shortCode: 'closing', short_code: 'closing', content: 'Thanks for contacting us, {{customer.first_name}}! If you need anything else, feel free to reply right here. Have a great day! -- {{agent.name}}', category: 'support', createdAt: '' },
  ])
  const [showCannedPicker, setShowCannedPicker] = useState(false)
  const [cannedCategory, setCannedCategory] = useState<string>('all')
  const [cannedSelectedIndex, setCannedSelectedIndex] = useState<number>(0)
  const [showCannedModal, setShowCannedModal] = useState(false)
  const [newCannedCode, setNewCannedCode] = useState('')
  const [newCannedCategory, setNewCannedCategory] = useState('support')
  const [newCannedContent, setNewCannedContent] = useState('')
  const [isSavingCanned, setIsSavingCanned] = useState(false)

  // Copilot Teammate Approvals & Human-in-the-Loop Workflows
  const [copilotApprovals, setCopilotApprovals] = useState<CopilotApprovalRequest[]>([
    {
      id: 'app-seed-1',
      conversationId: 'conv-maya',
      conversation_id: 'conv-maya',
      requestType: 'refund',
      request_type: 'refund',
      status: 'pending',
      title: 'Approve $89.00 Refund for Maya Lin',
      proposedReply: "Hi Maya, I've reviewed your request and approved a full refund of $89.00 for order #ORD-4912. The funds will appear on your original payment card within 3-5 business days. Please let me know if there's anything else I can do for you!",
      proposed_reply: "Hi Maya, I've reviewed your request and approved a full refund of $89.00 for order #ORD-4912. The funds will appear on your original payment card within 3-5 business days. Please let me know if there's anything else I can do for you!",
      actionData: {
        amount: 89.00,
        currency: 'USD',
        order_id: 'ORD-4912',
        reason: 'Courier delivered parcel with damaged exterior box',
        policy_rule: 'Refunds > $50.00 require human teammate sign-off prior to execution',
      },
      internalNote: 'Customer reported product damage on arrival with photos. AI confirmed VIP Platinum status. Company policy requires human teammate sign-off before payment gateway refund execution.',
      createdAt: new Date().toISOString(),
      created_at: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  ])
  const [showApprovalsModal, setShowApprovalsModal] = useState(false)
  const [approvalFilterStatus, setApprovalFilterStatus] = useState<string>('all')
  const [editingApprovalReplyId, setEditingApprovalReplyId] = useState<string | null>(null)
  const [editedApprovalReply, setEditedApprovalReply] = useState<string>('')
  const [teammateInfoInput, setTeammateInfoInput] = useState<string>('')
  const [rejectingApprovalId, setRejectingApprovalId] = useState<string | null>(null)
  const [rejectionFeedback, setRejectionFeedback] = useState<string>('')
  const [isProcessingApproval, setIsProcessingApproval] = useState(false)

  const [snoozeMenuOpen, setSnoozeMenuOpen] = useState(false)
  // Live Collision Detection & Agent Presence State
  const [collisionInfo, setCollisionInfo] = useState<CollisionCheckResult | null>(null)
  const [activePresences, setActivePresences] = useState<AgentPresenceItem[]>([])
  const [collisionWarningDismissed, setCollisionWarningDismissed] = useState(false)
  const [showCollisionInterlock, setShowCollisionInterlock] = useState(false)
  const [contactSidebarTab, setContactSidebarTab] = useState<'overview' | 'notes' | 'attributes' | 'history' | 'billing'>('overview')
  // Stripe Billing & Sidebar Integration State
  const [stripeBilling, setStripeBilling] = useState<StripeBillingProfile | null>(null)
  const [stripeInvoices, setStripeInvoices] = useState<StripeInvoice[]>([])
  const [stripeLogs, setStripeLogs] = useState<StripeBillingLog[]>([])
  const [stripeLoading, setStripeLoading] = useState(false)
  const [refundModalOpen, setRefundModalOpen] = useState(false)
  const [selectedInvoiceForRefund, setSelectedInvoiceForRefund] = useState<StripeInvoice | null>(null)
  const [refundAmountInput, setRefundAmountInput] = useState<string>('')
  const [refundReason, setRefundReason] = useState<string>('requested_by_customer')
  const [refundSubmitting, setRefundSubmitting] = useState(false)
  const [refundFeedback, setRefundFeedback] = useState<string | null>(null)
  const [portalCopied, setPortalCopied] = useState(false)
  // Gap 3: Issue Trackers & Slack Sync State
  const [linkedIssues, setLinkedIssues] = useState<LinkedIssue[]>([])
  const [slackSync, setSlackSync] = useState<SlackSyncStatus | null>(null)
  const [issueModalOpen, setIssueModalOpen] = useState(false)
  const [issueProvider, setIssueProvider] = useState<'linear' | 'github' | 'jira'>('linear')
  const [issueTitle, setIssueTitle] = useState('')
  const [issueDescription, setIssueDescription] = useState('')
  const [issuePriority, setIssuePriority] = useState<string>('high')
  const [issueType, setIssueType] = useState<string>('bug')
  const [issueSubmitting, setIssueSubmitting] = useState(false)
  const [issueNotice, setIssueNotice] = useState<string | null>(null)
  const [slackSyncing, setSlackSyncing] = useState(false)
  const [newNoteText, setNewNoteText] = useState('')
  const [contactNotes, setContactNotes] = useState<Record<string, Array<{ id: string; author: string; body: string; time: string }>>>({
    'conv-maya': [
      { id: 'cn-1', author: 'Riley Brooks', body: 'VIP customer since 2023. Always approve expedited courier delivery.', time: 'Mar 10, 11:30 AM' },
      { id: 'cn-2', author: 'Jamie Cole', body: 'Attended the Spring Preview webinar last week.', time: 'Mar 11, 4:15 PM' },
    ],
  })
  const [contactAttributes, setContactAttributes] = useState<Record<string, Record<string, string>>>({
    'conv-maya': {
      account_tier: 'VIP Platinum',
      preferred_channel: 'WhatsApp',
      language: 'English (US)',
      total_spend: '$18,450',
      nps_score: '10/10',
      whatsapp_optin: 'true',
    },
    'conv-4': {
      account_tier: 'Enterprise',
      preferred_channel: 'Website Live Chat',
      language: 'English (US)',
      total_spend: '$42,000',
      nps_score: '9/10',
      whatsapp_optin: 'false',
    },
  })

  // Captain AI Copilot, Triage & Monitors State
  const [captainSuggestedReply, setCaptainSuggestedReply] = useState<CaptainSuggestedReply | null>(null)
  const [isGeneratingReply, setIsGeneratingReply] = useState(false)
  const [captainSummary, setCaptainSummary] = useState<CaptainSummary | null>(null)
  const [isSummarizing, setIsSummarizing] = useState(false)
  const [showSummaryModal, setShowSummaryModal] = useState(false)
  const [isRewriting, setIsRewriting] = useState(false)
  const [isTriaging, setIsTriaging] = useState(false)
  const [triageResult, setTriageResult] = useState<CaptainTriageResult | null>(null)
  const [showMonitorsDrawer, setShowMonitorsDrawer] = useState(false)
  const [monitorsData, setMonitorsData] = useState<CaptainMonitorsData | null>(null)
  const [newMonitorName, setNewMonitorName] = useState('')
  const [newMonitorTopic, setNewMonitorTopic] = useState('support')
  const [newMonitorKeywords, setNewMonitorKeywords] = useState('')

  // Load live canned responses and macros from API
  useEffect(() => {
    api.listCannedResponses()
      .then((data) => {
        if (data && data.length > 0) setCannedResponses(data)
      })
      .catch(() => {})

    api.listMacros()
      .then((data) => {
        if (data && data.length > 0) setMacros(data)
      })
      .catch(() => {})

    api.listSLAPolicies()
      .then((data) => {
        if (data && data.length > 0) setSlaPolicies(data)
      })
      .catch(() => {})

    api.getSLAMetrics()
      .then((data) => {
        if (data) setSlaMetrics(data)
      })
      .catch(() => {})

    api.listRoutingAgents()
      .then((data) => {
        if (data && data.length > 0) setRoutingAgents(data)
      })
      .catch(() => {})

    api.getCapacityOverview()
      .then((data) => {
        if (data) setCapacityOverview(data)
      })
      .catch(() => {})

    api.listCopilotApprovals()
      .then((data) => {
        if (data && data.length > 0) {
          setCopilotApprovals((prev) => {
            const ids = new Set(data.map((d) => d.id))
            const remaining = prev.filter((p) => !ids.has(p.id))
            return [...data, ...remaining]
          })
        }
      })
      .catch(() => {})
  }, [])

  const activeConv = conversations.find((c) => c.id === selectedId) || conversations[0]

  // Live Presence & Real-Time Collision Heartbeat
  useEffect(() => {
    if (!activeConv?.id) return

    setCollisionWarningDismissed(false)

    // Register initial viewing state
    api.updateConversationPresence(activeConv.id, 'Riley Brooks', 'viewing')
      .catch(() => {})

    const checkCollision = () => {
      api.checkConversationCollision(activeConv.id, 'Riley Brooks')
        .then((res) => {
          setCollisionInfo(res)
          setActivePresences(res.allPresences || [])
        })
        .catch(() => {})
    }

    checkCollision()
    const timer = setInterval(checkCollision, 3500)

    return () => {
      clearInterval(timer)
      api.leaveConversationPresence(activeConv.id, 'Riley Brooks').catch(() => {})
    }
  }, [activeConv?.id])

  // Fetch Stripe Billing Profile, Invoices, and Action Logs for Active Contact
  useEffect(() => {
    if (!activeConv) return
    const identifier = activeConv.email || activeConv.phone || activeConv.id
    setStripeLoading(true)
    setRefundFeedback(null)

    Promise.all([
      api.getStripeBilling(identifier).catch(() => null),
      api.getStripeInvoices(identifier, 10).catch(() => []),
      api.getStripeBillingLogs(identifier, 20).catch(() => []),
    ]).then(([billing, invoices, logs]) => {
      setStripeBilling(billing)
      setStripeInvoices(invoices || [])
      setStripeLogs(logs || [])
      setStripeLoading(false)
    })
  }, [activeConv?.id, activeConv?.email, activeConv?.phone])

  async function handleCreatePortalLink() {
    if (!activeConv) return
    const identifier = activeConv.email || activeConv.phone || activeConv.id
    try {
      const res = await api.createStripePortalSession({
        identifier,
        actor_email: 'riley.brooks@newax.com',
      })
      await navigator.clipboard.writeText(res.url)
      setPortalCopied(true)
      setTimeout(() => setPortalCopied(false), 3000)
      api.getStripeBillingLogs(identifier, 20).then(setStripeLogs).catch(() => {})
    } catch (e: any) {
      alert(`Could not generate portal link: ${e.message}`)
    }
  }

  async function handleExecuteRefund() {
    if (!selectedInvoiceForRefund || !activeConv) return
    setRefundSubmitting(true)
    setRefundFeedback(null)
    const identifier = activeConv.email || activeConv.phone || activeConv.id

    try {
      const parsedAmount = refundAmountInput ? Math.round(parseFloat(refundAmountInput) * 100) : selectedInvoiceForRefund.amount_paid_cents
      const res = await api.processStripeRefund({
        charge_or_invoice_id: selectedInvoiceForRefund.id,
        amount_cents: parsedAmount,
        reason: refundReason,
        actor_email: 'riley.brooks@newax.com',
        customer_identifier: identifier,
      })

      setRefundFeedback(`Refund ${res.amount_formatted} processed successfully (Ref: ${res.id})`)
      setRefundSubmitting(false)
      setRefundModalOpen(false)

      api.getStripeInvoices(identifier, 10).then(setStripeInvoices).catch(() => {})
      api.getStripeBillingLogs(identifier, 20).then(setStripeLogs).catch(() => {})
    } catch (e: any) {
      setRefundSubmitting(false)
      setRefundFeedback(`Error issuing refund: ${e.message}`)
    }
  }

  // Gap 3: Fetch Linked Issues & Slack Sync on Active Conversation Change
  useEffect(() => {
    if (!activeConv?.id) return
    api.listLinkedIssues(activeConv.id)
      .then(setLinkedIssues)
      .catch(() => setLinkedIssues([]))

    api.getSlackSyncStatus(activeConv.id)
      .then(setSlackSync)
      .catch(() => setSlackSync(null))
  }, [activeConv?.id])

  function handleOpenCreateIssueModal(provider: 'linear' | 'github' | 'jira' = 'linear') {
    if (!activeConv) return
    setIssueProvider(provider)
    const lastMsg = activeConv.messages?.[activeConv.messages.length - 1]?.text || activeConv.preview
    setIssueTitle(`[${activeConv.channel.toUpperCase()}] Technical issue reported by ${activeConv.name}`)
    setIssueDescription(
      `Customer: ${activeConv.name} (${activeConv.email || activeConv.phone})
Channel: ${activeConv.channel}
Reported snippet:
"${lastMsg}"

Ticket Context: Conversation ID ${activeConv.id}`
    )
    setIssueNotice(null)
    setIssueModalOpen(true)
  }

  async function handleCreateLinkedIssue() {
    if (!activeConv || !issueTitle.trim()) return
    setIssueSubmitting(true)
    setIssueNotice(null)
    try {
      const created = await api.createLinkedIssue({
        provider: issueProvider,
        title: issueTitle,
        description: issueDescription,
        priority: issuePriority,
        issue_type: issueType,
        conversation_id: activeConv.id,
        actor_email: 'riley.brooks@newax.com',
        assigned_to: 'Riley Brooks',
      })
      setLinkedIssues((prev) => [created, ...prev])
      setIssueNotice(`Created and linked ${created.issue_key} in ${created.provider.toUpperCase()}!`)
      setIssueSubmitting(false)
      setIssueModalOpen(false)
    } catch (e: any) {
      setIssueSubmitting(false)
      setIssueNotice(`Failed to create issue: ${e.message}`)
    }
  }

  async function handleUnlinkIssue(issueId: string) {
    try {
      await api.unlinkIssue(issueId)
      setLinkedIssues((prev) => prev.filter((i) => i.id !== issueId))
    } catch (e: any) {
      alert(`Could not unlink issue: ${e.message}`)
    }
  }

  async function handleSyncIssueStatus(issueId: string) {
    try {
      const updated = await api.syncIssueStatus(issueId)
      setLinkedIssues((prev) => prev.map((i) => (i.id === issueId ? updated : i)))
    } catch (e: any) {
      alert(`Could not sync issue: ${e.message}`)
    }
  }

  async function handleSyncToSlack() {
    if (!activeConv) return
    setSlackSyncing(true)
    try {
      const res = await api.syncConversationToSlack({
        conversation_id: activeConv.id,
        channel_name: '#customer-escalations',
        note: `Escalation requested for ${activeConv.name} (${activeConv.channel})`,
        author: 'Riley Brooks',
      })
      setSlackSync({
        conversation_id: activeConv.id,
        slack_channel_name: res.slack_channel_name,
        slack_thread_ts: res.slack_thread_ts,
        thread_url: res.thread_url,
        is_synced: true,
      })
      alert(`Conversation bridged to Slack in ${res.slack_channel_name}!`)
    } catch (e: any) {
      alert(`Failed to sync to Slack: ${e.message}`)
    } finally {
      setSlackSyncing(false)
    }
  }

  async function simulateTeammate(action: 'viewing' | 'typing' | 'left', name: string = 'Jamie Cole') {
    if (!activeConv?.id) return
    setCollisionWarningDismissed(false)
    try {
      if (action === 'left') {
        await api.leaveConversationPresence(activeConv.id, name)
      } else {
        await api.updateConversationPresence(
          activeConv.id,
          name,
          action,
          'https://images.unsplash.com/photo-1570295999919-56ceb5ecca61?w=150&auto=format&fit=crop&q=80'
        )
      }
      const res = await api.checkConversationCollision(activeConv.id, 'Riley Brooks')
      setCollisionInfo(res)
      setActivePresences(res.allPresences || [])
    } catch (err) {
      console.error('Failed to simulate teammate presence:', err)
    }
  }

  async function handleExecuteMacro(macro: MacroItem) {
    if (!activeConv?.id) return
    setActiveExecutingMacroId(macro.id)
    setMacroLoading(true)
    try {
      const res = await api.executeMacro(activeConv.id, macro.id, 'Riley Brooks')

      setConversations((prev) =>
        prev.map((c) => {
          if (c.id === activeConv.id) {
            let updatedStatus = c.status
            let updatedPriority = c.priority
            const updatedTags = [...c.tags]
            const updatedMessages = [...c.messages]

            if (res.conversationState === 'closed_won' || res.conversationState === 'resolved') {
              updatedStatus = 'Resolved'
            }
            if (res.conversationPriority) {
              const capPrio = (res.conversationPriority.charAt(0).toUpperCase() + res.conversationPriority.slice(1).toLowerCase()) as any
              if (['Low', 'Medium', 'High', 'Urgent'].includes(capPrio)) {
                updatedPriority = capPrio
              }
            }
            if (res.conversationTags && res.conversationTags.length > 0) {
              res.conversationTags.forEach((t) => {
                if (!updatedTags.some((existing) => existing.name === t)) {
                  updatedTags.push({ name: t, color: 'bg-teal-500' })
                }
              })
            }

            // Append outgoing message or private note if in macro
            macro.actions.forEach((a) => {
              if (a.type === 'send_message' && a.body) {
                updatedMessages.push({
                  id: `m-macro-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
                  sender: 'agent',
                  text: a.body,
                  time: 'Just now',
                  status: 'sent',
                  agentName: 'Riley Brooks (via Macro)',
                  agentAvatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80',
                })
              } else if (a.type === 'add_private_note' && a.note) {
                updatedMessages.push({
                  id: `m-note-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
                  sender: 'note',
                  text: `🔒 [Private Note]: ${a.note}`,
                  time: 'Just now',
                  agentName: 'Riley Brooks (via Macro)',
                })
              }
            })

            return {
              ...c,
              status: updatedStatus,
              priority: updatedPriority,
              tags: updatedTags,
              messages: updatedMessages,
            }
          }
          return c
        })
      )

      setMacroBanner({
        name: macro.name,
        count: res.executedActionsCount || macro.actions.length,
        tags: res.conversationTags,
        priority: res.conversationPriority,
        state: res.conversationState,
      })
      setTimeout(() => setMacroBanner(null), 6000)
    } catch (err) {
      console.error('Failed to execute macro:', err)
    } finally {
      setMacroLoading(false)
      setActiveExecutingMacroId(null)
    }
  }

  async function handleCreateMacro(e: React.FormEvent) {
    e.preventDefault()
    if (!newMacroName.trim()) return

    const actions: any[] = []
    if (newMacroResolve) actions.push({ type: 'resolve_conversation', resolution_type: 'human_assisted' })
    if (newMacroCSAT) actions.push({ type: 'send_csat_survey' })
    if (newMacroPriority) actions.push({ type: 'change_priority', priority: newMacroPriority })
    if (newMacroTag.trim()) actions.push({ type: 'add_tags', tags: newMacroTag.split(',').map((t) => t.trim().toLowerCase()).filter(Boolean) })
    if (newMacroMessage.trim()) actions.push({ type: 'send_message', body: newMacroMessage.trim() })
    if (newMacroNote.trim()) actions.push({ type: 'add_private_note', note: newMacroNote.trim() })

    if (actions.length === 0) {
      actions.push({ type: 'add_tags', tags: ['custom-macro'] })
    }

    try {
      const created = await api.createMacro({
        name: newMacroName.trim(),
        description: newMacroDesc.trim(),
        actions,
      })
      setMacros((prev) => [created, ...prev])
      setShowCreateMacroModal(false)
      setNewMacroName('')
      setNewMacroDesc('')
      setNewMacroTag('')
      setNewMacroNote('')
      setNewMacroMessage('')
      setNewMacroResolve(false)
      setNewMacroCSAT(false)
    } catch (err) {
      console.error('Failed to create macro:', err)
    }
  }

  async function handleEvaluateSLA(convId: string = activeConv.id) {
    setIsEvaluatingSLA(true)
    try {
      const res = await api.evaluateConversationSLA(convId)
      setConversations((prev) =>
        prev.map((c) => {
          if (c.id === convId) {
            return {
              ...c,
              slaData: res.sla,
              sla: res.sla.isBreached
                ? 'Breached'
                : `${res.sla.minutesToFirstResponseBreach || res.sla.minutesToResolutionBreach}m left`,
              tags: res.tags.map((t) => ({
                name: t,
                color: t === 'sla-breached' ? 'bg-rose-600' : t === 'sla-warning' ? 'bg-amber-500' : 'bg-slate-500',
              })),
            }
          }
          return c
        })
      )
      api.getSLAMetrics().then(setSlaMetrics).catch(() => {})
    } catch (err) {
      console.error('Failed to evaluate SLA:', err)
    } finally {
      setIsEvaluatingSLA(false)
    }
  }

  async function openSLAMonitorModal() {
    setShowSLAMonitorModal(true)
    try {
      const [metrics, breaches, policies] = await Promise.all([
        api.getSLAMetrics(),
        api.getSLABreaches(),
        api.listSLAPolicies(),
      ])
      if (metrics) setSlaMetrics(metrics)
      if (breaches) setSlaBreaches(breaches)
      if (policies) setSlaPolicies(policies)
    } catch (err) {
      console.error('Failed to load SLA monitor data:', err)
    }
  }

  async function handleAutoAssign(strategy: 'capacity_based' | 'round_robin' = 'capacity_based') {
    setIsAutoAssigning(true)
    try {
      const res = await api.autoAssignConversation(activeConv.id, strategy)
      if (res.assigned && res.agentName) {
        setConversations((prev) =>
          prev.map((c) =>
            c.id === activeConv.id
              ? {
                  ...c,
                  assignedAgent: res.agentName!,
                  tags: c.tags.filter((t) => t.name !== 'team-overflow'),
                }
              : c
          )
        )
        setRoutingToast({
          message: `⚡ Assigned to ${res.agentName} via ${
            strategy === 'round_robin' ? 'Round-Robin' : 'Capacity-Based Routing'
          } (${res.activeCount}/${res.maxCapacity} active slots)`,
        })
      } else {
        setConversations((prev) =>
          prev.map((c) =>
            c.id === activeConv.id
              ? {
                  ...c,
                  tags: c.tags.some((t) => t.name === 'team-overflow')
                    ? c.tags
                    : [...c.tags, { name: 'team-overflow', color: 'bg-rose-600' }],
                }
              : c
          )
        )
        setRoutingToast({
          message: `⚠️ Team Overflow: All online agents are currently at capacity. Ticket placed in overflow queue.`,
          isOverflow: true,
        })
      }
      setTimeout(() => setRoutingToast(null), 5000)

      api.listRoutingAgents().then(setRoutingAgents).catch(() => {})
      api.getCapacityOverview().then(setCapacityOverview).catch(() => {})
    } catch (err) {
      console.error('Failed to auto-assign:', err)
    } finally {
      setIsAutoAssigning(false)
    }
  }

  async function handleUpdateMyStatus(newStatus: 'online' | 'busy' | 'offline') {
    setMyAgentStatus(newStatus)
    const myAgent = routingAgents.find((a) => a.name === 'Riley Brooks')
    if (myAgent) {
      try {
        await api.updateAgentStatus(myAgent.id, newStatus)
        const updated = await api.listRoutingAgents()
        setRoutingAgents(updated)
      } catch (err) {
        console.error('Failed to update status:', err)
      }
    }
  }

  async function handleUpdateCapacity(agentId: string, cap: number) {
    try {
      await api.updateAgentCapacity(agentId, cap)
      const [updatedAgents, overview] = await Promise.all([
        api.listRoutingAgents(),
        api.getCapacityOverview(),
      ])
      setRoutingAgents(updatedAgents)
      setCapacityOverview(overview)
    } catch (err) {
      console.error('Failed to update capacity:', err)
    }
  }

  async function handleBatchAutoAssign() {
    try {
      const res = await api.batchAutoAssign('capacity_based')
      setRoutingToast({
        message: `⚡ Batch Routing Complete: ${res.assignedCount} assigned, ${res.overflowCount} overflow tickets`,
      })
      setTimeout(() => setRoutingToast(null), 5000)
      const [updatedAgents, overview] = await Promise.all([
        api.listRoutingAgents(),
        api.getCapacityOverview(),
      ])
      setRoutingAgents(updatedAgents)
      setCapacityOverview(overview)
    } catch (err) {
      console.error('Failed batch auto-assign:', err)
    }
  }

  function handleComposerTextChange(val: string) {
    setReplyText(val)
    if (activeConv?.id) {
      const action = val.trim().length > 0 ? 'typing' : 'viewing'
      api.updateConversationPresence(activeConv.id, 'Riley Brooks', action).catch(() => {})
    }
  }

  // Canned autocomplete detection on "/"
  const slashMatch = replyText.match(/(?:^|\s)\/([a-zA-Z0-9_-]*)$/)
  const isTypingSlash = Boolean(slashMatch)
  const slashQuery = slashMatch ? slashMatch[1].toLowerCase() : ''
  const cannedCategories = ['all', ...Array.from(new Set(cannedResponses.map((c) => (c.category || 'general').toLowerCase())))]

  const filteredCanned = cannedResponses.filter((c) => {
    const code = (c.shortCode || c.short_code || '').toLowerCase()
    const content = c.content.toLowerCase()
    const matchesQuery = !slashQuery || code.includes(slashQuery) || content.includes(slashQuery)
    const matchesCategory =
      cannedCategory === 'all' || (c.category || 'general').toLowerCase() === cannedCategory.toLowerCase()
    return matchesQuery && matchesCategory
  })

  useEffect(() => {
    if (cannedSelectedIndex >= filteredCanned.length) {
      setCannedSelectedIndex(0)
    }
  }, [filteredCanned.length, cannedSelectedIndex])

  function resolveCannedVariables(template: string): string {
    const custName = activeConv?.name || 'Customer'
    const custFirstName = custName.split(' ')[0] || 'there'
    const agentName = activeConv?.assignedAgent || 'Riley Brooks'
    const channelName = activeConv?.channel ? activeConv.channel.charAt(0).toUpperCase() + activeConv.channel.slice(1) : 'Support'
    const ticketId = activeConv?.id ? activeConv.id.slice(0, 8) : 'TICKET'

    return template
      .replace(/\{\{\s*customer\.name\s*\}\}/gi, custName)
      .replace(/\{\{\s*customer_name\s*\}\}/gi, custName)
      .replace(/\{\{\s*customer\.first_name\s*\}\}/gi, custFirstName)
      .replace(/\{\{\s*customer\.firstName\s*\}\}/gi, custFirstName)
      .replace(/\{\{\s*first_name\s*\}\}/gi, custFirstName)
      .replace(/\{\{\s*agent\.name\s*\}\}/gi, agentName)
      .replace(/\{\{\s*agent_name\s*\}\}/gi, agentName)
      .replace(/\{\{\s*channel\s*\}\}/gi, channelName)
      .replace(/\{\{\s*ticket\.id\s*\}\}/gi, ticketId)
      .replace(/\{\{\s*ticket_id\s*\}\}/gi, ticketId)
      .replace(/\{\{\s*conversation_id\s*\}\}/gi, activeConv?.id || ticketId)
  }

  function insertCanned(canned: CannedResponse) {
    const resolved = resolveCannedVariables(canned.content)
    if (slashMatch) {
      const idx = replyText.lastIndexOf('/')
      const prefix = replyText.slice(0, idx)
      setReplyText(prefix + resolved + ' ')
    } else {
      setReplyText((prev) => (prev ? prev + ' ' : '') + resolved + ' ')
    }
    setShowCannedPicker(false)
    setCannedSelectedIndex(0)
  }

  async function handleCreateCanned() {
    if (!newCannedCode.trim() || !newCannedContent.trim()) return
    setIsSavingCanned(true)
    try {
      const code = newCannedCode.trim().replace(/^\//, '')
      const created = await api.createCannedResponse({
        short_code: code,
        content: newCannedContent.trim(),
        category: newCannedCategory.trim().toLowerCase() || 'general',
      })
      setCannedResponses((prev) => [...prev, created])
      setNewCannedCode('')
      setNewCannedContent('')
      setNewCannedCategory('support')
      setShowCannedModal(false)
    } catch (e: any) {
      alert(e.message || 'Failed to create canned response')
    } finally {
      setIsSavingCanned(false)
    }
  }

  async function handleDeleteCanned(id: string) {
    if (!confirm('Are you sure you want to delete this canned response?')) return
    try {
      await api.deleteCannedResponse(id)
      setCannedResponses((prev) => prev.filter((c) => c.id !== id))
    } catch (e: any) {
      alert(e.message || 'Failed to delete canned response')
    }
  }

  // Copilot Teammate Approvals Handlers
  const pendingApprovalsCount = copilotApprovals.filter((a) => a.status === 'pending').length
  const activeApproval = copilotApprovals.find(
    (a) => (a.conversationId === activeConv?.id || a.conversation_id === activeConv?.id) && a.status === 'pending'
  )

  async function handleApproveRequest(app: CopilotApprovalRequest, amendedText?: string) {
    setIsProcessingApproval(true)
    try {
      const updated = await api.approveCopilotRequest(app.id, {
        reviewer_name: 'Riley Brooks',
        amended_reply: amendedText,
        execute_send: true,
      })
      setCopilotApprovals((prev) => prev.map((a) => (a.id === app.id ? updated : a)))
      setEditingApprovalReplyId(null)

      const replyBody = amendedText || app.proposedReply || app.proposed_reply
      if (replyBody && activeConv?.id) {
        const newMsg: MessageItem = {
          id: 'msg-' + Date.now(),
          sender: 'agent',
          agentName: 'Captain AI (Approved)',
          text: replyBody,
          time: 'Just now',
          status: 'sent',
        }
        const auditMsg: MessageItem = {
          id: 'audit-' + Date.now(),
          sender: 'note',
          text: `✅ [Copilot Approved] Riley Brooks approved ${(app.requestType || app.request_type || 'action').toUpperCase()}. AI Agent dispatched reply to customer and finished conversation.`,
          time: 'Just now',
        }
        setConversations((prev) =>
          prev.map((c) =>
            c.id === activeConv.id
              ? {
                  ...c,
                  messages: [...c.messages, auditMsg, newMsg],
                  preview: replyBody.slice(0, 50) + '...',
                }
              : c
          )
        )
      }
    } catch (e: any) {
      alert(e.message || 'Failed to approve request')
    } finally {
      setIsProcessingApproval(false)
    }
  }

  async function handleRejectRequest(approvalId: string, feedback: string) {
    setIsProcessingApproval(true)
    try {
      const updated = await api.rejectCopilotRequest(approvalId, {
        reviewer_name: 'Riley Brooks',
        feedback: feedback || 'Rejected by teammate',
      })
      setCopilotApprovals((prev) => prev.map((a) => (a.id === approvalId ? updated : a)))
      setRejectingApprovalId(null)
      setRejectionFeedback('')

      if (activeConv?.id) {
        const auditMsg: MessageItem = {
          id: 'audit-rej-' + Date.now(),
          sender: 'note',
          text: `❌ [Copilot Rejected] Riley Brooks rejected the request. Reason: ${feedback || 'No feedback'}. AI Agent paused and will not send reply.`,
          time: 'Just now',
        }
        setConversations((prev) =>
          prev.map((c) =>
            c.id === activeConv.id
              ? { ...c, messages: [...c.messages, auditMsg] }
              : c
          )
        )
      }
    } catch (e: any) {
      alert(e.message || 'Failed to reject request')
    } finally {
      setIsProcessingApproval(false)
    }
  }

  async function handleProvideTeammateInfo(approvalId: string, infoText: string) {
    if (!infoText.trim()) return
    setIsProcessingApproval(true)
    try {
      const updated = await api.provideCopilotInfo(approvalId, {
        reviewer_name: 'Riley Brooks',
        teammate_answer: infoText.trim(),
      })
      setCopilotApprovals((prev) => prev.map((a) => (a.id === approvalId ? updated : a)))
      setTeammateInfoInput('')

      const finalReply = updated.amendedReply || updated.amended_reply
      if (finalReply && activeConv?.id) {
        const newMsg: MessageItem = {
          id: 'msg-info-' + Date.now(),
          sender: 'agent',
          agentName: 'Captain AI (Finished)',
          text: finalReply,
          time: 'Just now',
          status: 'sent',
        }
        const auditMsg: MessageItem = {
          id: 'audit-info-' + Date.now(),
          sender: 'note',
          text: `✅ [Teammate Info Provided] Riley Brooks provided answer: "${infoText.trim()}". AI Agent finished the conversation itself.`,
          time: 'Just now',
        }
        setConversations((prev) =>
          prev.map((c) =>
            c.id === activeConv.id
              ? {
                  ...c,
                  messages: [...c.messages, auditMsg, newMsg],
                  preview: finalReply.slice(0, 50) + '...',
                }
              : c
          )
        )
      }
    } catch (e: any) {
      alert(e.message || 'Failed to submit info')
    } finally {
      setIsProcessingApproval(false)
    }
  }

  async function handleTriggerRefundApprovalDemo() {
    if (!activeConv?.id) return
    try {
      const res = await api.createCopilotApproval({
        conversation_id: activeConv.id,
        title: `Approve $89.00 Refund for ${activeConv.name}`,
        proposed_reply: `Hi ${activeConv.name.split(' ')[0]}, I've reviewed your request and approved a full refund of $89.00 for order #ORD-4912. The funds have been initiated back to your card and will appear in 3-5 business days.`,
        request_type: 'refund',
        action_data: {
          amount: 89.0,
          currency: 'USD',
          order_id: 'ORD-4912',
          reason: 'Courier reported damaged packaging on delivery',
          policy_rule: 'Refunds > $50.00 require teammate sign-off prior to execution',
        },
        internal_note: 'Customer requested refund. AI verified ticket history. Policy requires teammate approval before payment gateway return.',
      })
      setCopilotApprovals((prev) => [res, ...prev.filter((a) => a.id !== res.id)])
    } catch (e: any) {
      alert(e.message || 'Failed to trigger approval')
    }
  }

  function handlePriorityChange(newPriority: 'Low' | 'Medium' | 'High' | 'Urgent') {
    setConversations((prev) =>
      prev.map((c) => (c.id === activeConv.id ? { ...c, priority: newPriority } : c))
    )
    api.updatePriority(activeConv.id, newPriority.toLowerCase()).catch(() => {})
  }

  function handleTeamChange(newTeam: string) {
    setConversations((prev) =>
      prev.map((c) => (c.id === activeConv.id ? { ...c, team: newTeam } : c))
    )
    api.assignTeam(activeConv.id, newTeam).catch(() => {})
  }

  function handleAgentChange(newAgent: string) {
    setConversations((prev) =>
      prev.map((c) => (c.id === activeConv.id ? { ...c, assignedAgent: newAgent } : c))
    )
    api.assignAgent(activeConv.id, newAgent).catch(() => {})
  }

  function handleSnooze(duration: string) {
    setConversations((prev) =>
      prev.map((c) =>
        c.id === activeConv.id ? { ...c, status: 'Pending', sla: `Snoozed (${duration})` } : c
      )
    )
    setSnoozeMenuOpen(false)
    api.snoozeConversation(activeConv.id, duration).catch(() => {})
  }

  function handleAddContactNote() {
    if (!newNoteText.trim()) return
    const noteObj = {
      id: `cn-${Date.now()}`,
      author: 'Riley Brooks',
      body: newNoteText.trim(),
      time: 'Just now',
    }
    setContactNotes((prev) => ({
      ...prev,
      [activeConv.id]: [noteObj, ...(prev[activeConv.id] || [])],
    }))
    api.addContactNote(activeConv.id, newNoteText.trim(), 'Riley Brooks').catch(() => {})
    setNewNoteText('')
  }

  function executeSend() {
    if (!replyText.trim()) return

    const isNote = composerMode === 'note'
    const newMsg: MessageItem = {
      id: `m-${Date.now()}`,
      sender: isNote ? 'note' : 'agent',
      text: replyText.trim(),
      time: 'Just now',
      status: 'read',
      agentName: 'Riley Brooks',
      agentAvatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80',
    }

    setConversations((prev) =>
      prev.map((c) => {
        if (c.id === activeConv.id) {
          return {
            ...c,
            preview: isNote ? c.preview : `↩ ${replyText.trim().slice(0, 32)}...`,
            messages: [...c.messages, newMsg],
          }
        }
        return c
      })
    )

    api.sendConversationMessage(activeConv.id, replyText.trim(), isNote, 'Riley Brooks').catch(() => {})
    api.updateConversationPresence(activeConv.id, 'Riley Brooks', 'viewing').catch(() => {})
    setReplyText('')
    setShowCollisionInterlock(false)
  }

  function handleSend() {
    if (!replyText.trim()) return
    if (collisionInfo?.severity === 'danger' && composerMode !== 'note') {
      setShowCollisionInterlock(true)
      return
    }
    executeSend()
  }

  function handleCopy(text: string, field: string) {
    navigator.clipboard.writeText(text)
    setCopiedField(field)
    setTimeout(() => setCopiedField(null), 2000)
  }

  function handleRemoveLabel(labelName: string) {
    setConversations((prev) =>
      prev.map((c) => {
        if (c.id === activeConv.id) {
          return {
            ...c,
            tags: c.tags.filter((t) => t.name !== labelName),
          }
        }
        return c
      })
    )
  }

  function handleAddLabel() {
    const label = prompt('Enter new conversation label (e.g. billing, feature-request, bug):')
    if (!label) return
    setConversations((prev) =>
      prev.map((c) => {
        if (c.id === activeConv.id) {
          if (c.tags.some((t) => t.name === label.toLowerCase())) return c
          return {
            ...c,
            tags: [...c.tags, { name: label.toLowerCase(), color: 'bg-indigo-500' }],
          }
        }
        return c
      })
    )
  }

  function handleAddMemory(factText: string) {
    if (!factText.trim()) return
    const newMem = {
      id: `mem-${Date.now()}`,
      fact: factText.trim(),
      category: 'preference',
      notedAt: 'Just now',
    }
    setConversations((prev) =>
      prev.map((c) =>
        c.id === activeConv.id
          ? { ...c, memories: [newMem, ...(c.memories || [])] }
          : c
      )
    )
  }

  function handleRemoveMemory(memId: string) {
    setConversations((prev) =>
      prev.map((c) =>
        c.id === activeConv.id
          ? { ...c, memories: (c.memories || []).filter((m) => m.id !== memId) }
          : c
      )
    )
  }

  async function handleGenerateCaptainReply(customInstruction?: string) {
    setIsGeneratingReply(true)
    try {
      const res = await api.getCaptainSuggestedReply(activeConv.id, customInstruction)
      setCaptainSuggestedReply(res)
      setConversations((prev) =>
        prev.map((c) =>
          c.id === activeConv.id ? { ...c, captainDraft: res.suggestedReply } : c
        )
      )
    } catch (err) {
      console.error('Failed to generate captain reply:', err)
      // Fallback
      setConversations((prev) =>
        prev.map((c) =>
          c.id === activeConv.id
            ? {
                ...c,
                captainDraft:
                  `Hi ${activeConv.name}, thank you for contacting us! I am reviewing your request and will provide full details immediately.`,
              }
            : c
        )
      )
    } finally {
      setIsGeneratingReply(false)
    }
  }

  function handleAIDraft() {
    handleGenerateCaptainReply()
  }

  async function handleSummarizeThread() {
    setIsSummarizing(true)
    try {
      const res = await api.summarizeConversationThread(activeConv.id)
      setCaptainSummary(res)
      setShowSummaryModal(true)
    } catch (err) {
      console.error('Failed to summarize thread:', err)
      setCaptainSummary({
        conversationId: activeConv.id,
        summary: `• Conversation with ${activeConv.name} on ${activeConv.channel}.\n• Customer inquiries reviewed.\n• Ready for agent follow-up.`,
        bulletPoints: [
          `Inquiry initiated by ${activeConv.name} on ${activeConv.channel}.`,
          `Customer exchanged ${activeConv.messages.length} messages.`,
          `Current SLA status: ${activeConv.sla || 'Active'}.`,
        ],
        sentiment: 'positive',
        keyTopics: ['Customer Support', activeConv.channel],
        actionRequired: true,
      })
      setShowSummaryModal(true)
    } finally {
      setIsSummarizing(false)
    }
  }

  async function handleRewriteTone(
    tone: 'professional' | 'friendly' | 'shorten' | 'fix_spelling' | 'expand'
  ) {
    if (!replyText.trim()) return
    setIsRewriting(true)
    try {
      const res = await api.rewriteDraftText(replyText, tone)
      setReplyText(res.rewritten)
    } catch (err) {
      console.error('Failed to rewrite draft:', err)
    } finally {
      setIsRewriting(false)
    }
  }

  async function handleRunTriage(apply: boolean = true) {
    setIsTriaging(true)
    try {
      const res = await api.triageConversation(activeConv.id, apply)
      setTriageResult(res)
      if (apply) {
        setConversations((prev) =>
          prev.map((c) => {
            if (c.id === activeConv.id) {
              const newTags = [
                ...c.tags,
                ...res.suggestedTags
                  .filter((t) => !c.tags.some((ct) => ct.name === t))
                  .map((t) => ({ name: t, color: 'bg-purple-600' })),
              ]
              const priorityMap: Record<string, 'Low' | 'Medium' | 'High' | 'Urgent'> = {
                urgent: 'Urgent',
                high: 'High',
                medium: 'Medium',
                low: 'Low',
              }
              return {
                ...c,
                priority: priorityMap[res.priority] || c.priority,
                team: res.suggestedTeam || c.team,
                tags: newTags,
              }
            }
            return c
          })
        )
      }
    } catch (err) {
      console.error('Failed to triage conversation:', err)
    } finally {
      setIsTriaging(false)
    }
  }

  async function handleOpenMonitors() {
    setShowMonitorsDrawer(true)
    try {
      const data = await api.getCaptainMonitors()
      setMonitorsData(data)
    } catch (err) {
      console.error('Failed to fetch monitors:', err)
    }
  }

  async function handleCreateMonitor() {
    if (!newMonitorName.trim() || !newMonitorKeywords.trim()) return
    const keywords = newMonitorKeywords.split(',').map((k) => k.trim()).filter(Boolean)
    try {
      await api.createCaptainMonitor(newMonitorName.trim(), newMonitorTopic, keywords)
      const data = await api.getCaptainMonitors()
      setMonitorsData(data)
      setNewMonitorName('')
      setNewMonitorKeywords('')
    } catch (err) {
      console.error('Failed to create monitor:', err)
    }
  }

  const filteredConversations = conversations.filter((c) => {
    if (activeTab === 'Mine' && c.assignedAgent !== 'Riley Brooks') return false
    if (activeTab === 'Unassigned' && c.assignedAgent !== '') return false
    if (activeTab === 'Autonomous' && c.resolutionType !== 'autonomous_no_handoff') return false
    if (activeTab === 'Approvals') {
      const hasPending = copilotApprovals.some(
        (a) => (a.conversationId === c.id || a.conversation_id === c.id) && a.status === 'pending'
      )
      if (!hasPending) return false
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase()
      return (
        c.name.toLowerCase().includes(q) ||
        c.preview.toLowerCase().includes(q) ||
        c.channel.toLowerCase().includes(q)
      )
    }
    return true
  })

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-white text-slate-800 antialiased font-sans select-none">
      {/* ========================================================================= */}
      {/* COLUMN 1: LEFT NAVIGATION SIDEBAR                                        */}
      {/* ========================================================================= */}
      <aside className="flex w-[240px] shrink-0 flex-col border-r border-[#eaecf0] bg-[#fafafb] text-[13px]">
        {/* Workspace Brand Switcher */}
        <div className="flex items-center justify-between border-b border-[#eaecf0] px-4 py-3.5">
          <div className="flex items-center gap-2.5">
            {/* Bay Works Logo */}
            <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#1570ef] text-white shadow-xs">
              <div className="h-2.5 w-2.5 rounded-full border-2 border-white" />
            </div>
            <span className="font-semibold text-slate-900 tracking-tight text-[13.5px]">Bay Works</span>
            <span className="text-slate-400 text-xs">⌄</span>
          </div>
          <Link
            to="/connections"
            title="Workspace Settings"
            className="text-slate-400 hover:text-slate-600 p-1"
          >
            ⚙️
          </Link>
        </div>

        {/* Search & Compose Action Bar */}
        <div className="px-3 py-2.5">
          <div className="flex items-center gap-1.5">
            <div className="relative flex-1">
              <span className="absolute left-2.5 top-2 text-slate-400 text-xs">🔍</span>
              <input
                type="text"
                placeholder="Search..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full rounded-md border border-[#eaecf0] bg-white py-1.5 pl-7 pr-2 text-xs text-slate-700 placeholder-slate-400 focus:border-blue-500 focus:outline-hidden"
              />
            </div>
            <button
              title="Compose message"
              className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-[#eaecf0] bg-white text-slate-600 hover:bg-slate-50 shadow-xs"
            >
              ✏️
            </button>
          </div>
        </div>

        {/* Navigation Sections Scrollable Area */}
        <div className="flex-1 overflow-y-auto px-2 py-1 space-y-4">
          {/* Main Inbox & Conversations Group */}
          <div className="space-y-0.5">
            <button
              onClick={() => setActiveTab('Mine')}
              className={`flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left font-medium transition-colors ${
                activeTab === 'Mine' ? 'bg-[#f2f4f7] text-slate-900' : 'text-slate-600 hover:bg-[#f2f4f7]'
              }`}
            >
              <div className="flex items-center gap-2">
                <span>📥</span>
                <span>My Inbox</span>
              </div>
              <span className="rounded-full bg-slate-200/80 px-2 py-0.5 text-[11px] font-semibold text-slate-600">
                3
              </span>
            </button>

            {/* Conversations Tree */}
            <div className="pt-1">
              <div className="flex items-center gap-2 px-2.5 py-1 text-slate-700 font-medium">
                <span>💬</span>
                <span>Conversations</span>
              </div>
              <div className="ml-4 pl-2 border-l border-slate-200/80 space-y-0.5 pt-0.5">
                <button
                  onClick={() => setActiveTab('All')}
                  className={`flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-xs transition-colors ${
                    activeTab === 'All'
                      ? 'bg-[#eaecf0] font-semibold text-slate-900'
                      : 'text-slate-600 hover:bg-[#f2f4f7]'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <span>🗂️</span>
                    <span>All Conversations</span>
                  </div>
                </button>
                <button
                  onClick={() => setActiveTab('Mine')}
                  className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs text-slate-600 hover:bg-[#f2f4f7]"
                >
                  <span>@</span>
                  <span>Mentions</span>
                </button>
                <button
                  onClick={() => setActiveTab('Mine')}
                  className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs text-slate-600 hover:bg-[#f2f4f7]"
                >
                  <span>👥</span>
                  <span>Participating</span>
                </button>
                <button
                  onClick={() => setActiveTab('Unassigned')}
                  className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs text-slate-600 hover:bg-[#f2f4f7]"
                >
                  <span>⏳</span>
                  <span>Unattended</span>
                </button>
              </div>
            </div>
          </div>

          {/* Folders Section */}
          <div>
            <div
              onClick={() => setShowNavFolders((v) => !v)}
              className="flex items-center justify-between px-2.5 py-1 text-slate-500 font-semibold text-[11.5px] uppercase tracking-wider cursor-pointer hover:text-slate-800"
            >
              <div className="flex items-center gap-1.5">
                <span>📁</span>
                <span>Folders</span>
              </div>
              <span>{showNavFolders ? '⌃' : '⌄'}</span>
            </div>
            {showNavFolders && (
              <div className="space-y-0.5 pt-1">
                {[
                  { name: 'Tech queue', icon: '🛠️' },
                  { name: 'API', icon: '⚙️' },
                  { name: 'Login', icon: '🔑' },
                  { name: 'Escalations', icon: '🚨' },
                  { name: 'Billing queue', icon: '💳' },
                  { name: 'Open bugs', icon: '🐞' },
                ].map((f) => (
                  <button
                    key={f.name}
                    className="flex w-full items-center gap-2 rounded-md px-2.5 py-1 text-left text-xs text-slate-600 hover:bg-[#f2f4f7]"
                  >
                    <span>{f.icon}</span>
                    <span>{f.name}</span>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Teams Section */}
          <div>
            <div
              onClick={() => setShowNavTeams((v) => !v)}
              className="flex items-center justify-between px-2.5 py-1 text-slate-500 font-semibold text-[11.5px] uppercase tracking-wider cursor-pointer hover:text-slate-800"
            >
              <div className="flex items-center gap-1.5">
                <span>👥</span>
                <span>Teams</span>
              </div>
              <span>{showNavTeams ? '⌃' : '⌄'}</span>
            </div>
            {showNavTeams && (
              <div className="space-y-0.5 pt-1">
                {[
                  { name: 'developer su...', icon: '💻' },
                  { name: 'customer suc...', icon: '🤝' },
                  { name: 'technical sup...', icon: '🔧' },
                ].map((t) => (
                  <button
                    key={t.name}
                    className="flex w-full items-center gap-2 rounded-md px-2.5 py-1 text-left text-xs text-slate-600 hover:bg-[#f2f4f7]"
                  >
                    <span>{t.icon}</span>
                    <span>{t.name}</span>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Channels Section */}
          <div>
            <div
              onClick={() => setShowNavChannels((v) => !v)}
              className="flex items-center justify-between px-2.5 py-1 text-slate-500 font-semibold text-[11.5px] uppercase tracking-wider cursor-pointer hover:text-slate-800"
            >
              <div className="flex items-center gap-1.5">
                <span>📡</span>
                <span>Channels</span>
              </div>
              <span>{showNavChannels ? '⌃' : '⌄'}</span>
            </div>
            {showNavChannels && (
              <div className="space-y-1 pt-1">
                <Link
                  to="/channels/whatsapp"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-emerald-50 border border-emerald-200 text-emerald-950 font-medium hover:bg-emerald-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>💬</span>
                    <span className="truncate">WhatsApp Business</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-emerald-500 ring-2 ring-emerald-200" title="Active & Verified" />
                </Link>
                <Link
                  to="/channels/live-chat"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-blue-50 border border-blue-200 text-blue-950 font-medium hover:bg-blue-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>🌐</span>
                    <span className="truncate">Website Live Chat</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-blue-500 ring-2 ring-blue-200" title="Active & Configured" />
                </Link>
                <Link
                  to="/channels/sms"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-red-50 border border-red-200 text-red-950 font-medium hover:bg-red-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>📱</span>
                    <span className="truncate">Twilio SMS</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-red-500 ring-2 ring-red-200" title="Active & 10DLC Verified" />
                </Link>
                <Link
                  to="/channels/facebook"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-indigo-50 border border-indigo-200 text-indigo-950 font-medium hover:bg-indigo-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>👥</span>
                    <span className="truncate">Facebook Messenger</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-indigo-500 ring-2 ring-indigo-200" title="Active & Graph v21.0" />
                </Link>
                <Link
                  to="/channels/instagram"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-pink-50 border border-pink-200 text-pink-950 font-medium hover:bg-pink-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>📸</span>
                    <span className="truncate">Instagram Direct</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-pink-500 ring-2 ring-pink-200" title="Active & Story Mentions" />
                </Link>
                <Link
                  to="/channels/telegram"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-sky-50 border border-sky-200 text-sky-950 font-medium hover:bg-sky-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>✈️</span>
                    <span className="truncate">Telegram Bot</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-sky-500 ring-2 ring-sky-200" title="Active & Webhook Verified" />
                </Link>
                <Link
                  to="/channels/line"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-emerald-50 border border-emerald-200 text-emerald-950 font-medium hover:bg-emerald-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>🟢</span>
                    <span className="truncate">LINE Official</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-emerald-500 ring-2 ring-emerald-200" title="Active & Push Messaging" />
                </Link>
                <Link
                  to="/channels/voice"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-teal-50 border border-teal-200 text-teal-950 font-medium hover:bg-teal-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>📞</span>
                    <span className="truncate">Voice &amp; WebRTC</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-teal-500 ring-2 ring-teal-200" title="Active & Softphone Ready" />
                </Link>
                <Link
                  to="/channels/email"
                  className="flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-left text-xs bg-amber-50 border border-amber-200 text-amber-950 font-medium hover:bg-amber-100/70 transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span>✉️</span>
                    <span className="truncate">Support Email</span>
                  </div>
                  <span className="flex h-2 w-2 rounded-full bg-amber-500 ring-2 ring-amber-200" title="Active & IMAP/SMTP" />
                </Link>
                <Link
                  to="/channels"
                  className="mt-1 flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold text-blue-600 hover:text-blue-800"
                >
                  <span>+ Manage All Channels ↗</span>
                </Link>
              </div>
            )}
          </div>

          {/* Management Shortcuts to other views */}
          <div className="border-t border-[#eaecf0] pt-2">
            <div className="px-2.5 py-1 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
              Management
            </div>
            <div className="space-y-0.5">
              <Link
                to="/automation"
                className="flex items-center gap-2 rounded-md px-2.5 py-1 text-xs text-slate-600 hover:bg-[#f2f4f7]"
              >
                <span>⚡</span>
                <span>Automations</span>
              </Link>
              <Link
                to="/broadcasts"
                className="flex items-center gap-2 rounded-md px-2.5 py-1 text-xs text-slate-600 hover:bg-[#f2f4f7]"
              >
                <span>📢</span>
                <span>Broadcasts</span>
              </Link>
              <Link
                to="/contacts"
                className="flex items-center gap-2 rounded-md px-2.5 py-1 text-xs text-slate-600 hover:bg-[#f2f4f7]"
              >
                <span>📇</span>
                <span>Contacts</span>
              </Link>
              <Link
                to="/connections"
                className="flex items-center gap-2 rounded-md px-2.5 py-1 text-xs text-slate-600 hover:bg-[#f2f4f7]"
              >
                <span>🔌</span>
                <span>Connections</span>
              </Link>
            </div>
          </div>

          {/* Captain AI Suite */}
          <div className="border-t border-[#eaecf0] pt-2">
            <div className="px-2.5 py-1 text-[11px] font-semibold text-purple-600 uppercase tracking-wider flex items-center justify-between">
              <span>Captain AI</span>
              <span className="text-[10px] bg-purple-100 text-purple-700 px-1.5 py-0.2 rounded font-bold">Live</span>
            </div>
            <div className="space-y-0.5">
              <button
                onClick={() => setShowApprovalsModal(true)}
                className="flex w-full items-center justify-between rounded-md px-2.5 py-1 text-xs text-amber-900 font-semibold hover:bg-amber-50 transition-colors cursor-pointer"
              >
                <div className="flex items-center gap-2">
                  <span>🤝</span>
                  <span>Teammate Approvals</span>
                </div>
                {pendingApprovalsCount > 0 && (
                  <span className="text-[10px] bg-amber-500 text-white font-bold px-1.5 py-0.2 rounded-full animate-pulse">
                    {pendingApprovalsCount}
                  </span>
                )}
              </button>
              <button
                onClick={handleOpenMonitors}
                className="flex w-full items-center gap-2 rounded-md px-2.5 py-1 text-xs text-purple-900 font-semibold hover:bg-purple-50 transition-colors"
              >
                <span>✨</span>
                <span>Topic Monitors &amp; Insights</span>
              </button>
              <Link
                to="/reports"
                className="flex items-center gap-2 rounded-md px-2.5 py-1 text-xs text-slate-600 hover:bg-[#f2f4f7]"
              >
                <span>📊</span>
                <span>Reports &amp; Analytics</span>
              </Link>
            </div>
          </div>
        </div>

        {/* Bottom Current Agent Account Card */}
        <div className="border-t border-[#eaecf0] p-3 bg-white">
          <div className="flex items-center gap-2.5">
            <div className="relative">
              <img
                src="https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80"
                alt="Riley Brooks"
                className="h-8 w-8 rounded-full object-cover ring-1 ring-slate-200"
              />
              <span
                className={`absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full ring-2 ring-white ${
                  myAgentStatus === 'online'
                    ? 'bg-emerald-500'
                    : myAgentStatus === 'busy'
                    ? 'bg-amber-500'
                    : 'bg-slate-400'
                }`}
              />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-center justify-between">
                <p className="truncate text-xs font-semibold text-slate-900 leading-tight">Riley Brooks</p>
                <select
                  value={myAgentStatus}
                  onChange={(e) => handleUpdateMyStatus(e.target.value as any)}
                  className="text-[10px] bg-transparent font-medium text-slate-600 border border-slate-200 rounded px-1 py-0.2 focus:outline-hidden cursor-pointer"
                >
                  <option value="online">🟢 Online</option>
                  <option value="busy">🟡 Busy</option>
                  <option value="offline">⚪ Offline</option>
                </select>
              </div>
              <div className="flex items-center justify-between mt-0.5">
                <p className="truncate text-[10.5px] text-slate-500">
                  Capacity: {routingAgents.find((a) => a.name === 'Riley Brooks')?.activeConversationsCount || 1}/
                  {routingAgents.find((a) => a.name === 'Riley Brooks')?.maxCapacity || 5}
                </p>
                <button
                  type="button"
                  onClick={() => setShowCapacityModal(true)}
                  className="text-[10px] text-brand-600 hover:underline font-semibold cursor-pointer"
                >
                  Capacity
                </button>
              </div>
            </div>
          </div>
        </div>
      </aside>

      {/* ========================================================================= */}
      {/* COLUMN 2: CONVERSATIONS LIST PANE                                        */}
      {/* ========================================================================= */}
      <section className="flex w-[320px] shrink-0 flex-col border-r border-[#eaecf0] bg-white text-[13px]">
        {/* Top Header Controls */}
        <div className="border-b border-[#eaecf0] px-3.5 pt-3.5 pb-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <h2 className="font-semibold text-slate-900 text-[15px]">Conversations</h2>
              <span className="inline-flex items-center gap-1 rounded bg-[#f2f4f7] px-2 py-0.5 text-[11px] font-medium text-slate-700">
                Open ⌄
              </span>
            </div>
            <div className="flex items-center gap-2 text-slate-400">
              <button title="Filter" className="hover:text-slate-600">
                ⚡
              </button>
              <button title="Sort" className="hover:text-slate-600">
                ⇅
              </button>
              <button title="Toggle view" className="hover:text-slate-600">
                ⊞
              </button>
            </div>
          </div>

          {/* Filter Tabs (Mine, Unassigned, All) */}
          <div className="mt-3 flex items-center gap-4 text-xs">
            <button
              onClick={() => setActiveTab('Mine')}
              className={`pb-1.5 transition-colors ${
                activeTab === 'Mine'
                  ? 'border-b-2 border-blue-600 font-semibold text-blue-600'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              Mine <span className="font-normal text-slate-400">2</span>
            </button>
            <button
              onClick={() => setActiveTab('Unassigned')}
              className={`pb-1.5 transition-colors ${
                activeTab === 'Unassigned'
                  ? 'border-b-2 border-blue-600 font-semibold text-blue-600'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              Unassigned <span className="font-normal text-slate-400">0</span>
            </button>
            <button
              onClick={() => setActiveTab('All')}
              className={`pb-1.5 transition-colors ${
                activeTab === 'All'
                  ? 'border-b-2 border-blue-600 font-semibold text-blue-600'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              All <span className="font-semibold text-blue-600">20</span>
            </button>
            <button
              onClick={() => setActiveTab('Autonomous')}
              className={`pb-1.5 transition-colors flex items-center gap-1 ${
                activeTab === 'Autonomous'
                  ? 'border-b-2 border-emerald-600 font-semibold text-emerald-700'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              <span>✨ Autonomous</span>
            </button>
            <button
              onClick={() => setActiveTab('Approvals')}
              className={`pb-1.5 transition-colors flex items-center gap-1.5 cursor-pointer ${
                activeTab === 'Approvals'
                  ? 'border-b-2 border-amber-600 font-bold text-amber-700'
                  : 'text-slate-500 hover:text-slate-800'
              }`}
            >
              <span>🤝 Approvals</span>
              {pendingApprovalsCount > 0 && (
                <span className="text-[10px] bg-amber-500 text-white font-black px-1.5 py-0.2 rounded-full animate-pulse">
                  {pendingApprovalsCount}
                </span>
              )}
            </button>
          </div>
        </div>

        {/* Scrollable Conversations List */}
        <div className="flex-1 overflow-y-auto divide-y divide-[#f2f4f7]">
          {filteredConversations.map((item) => {
            const isSelected = item.id === activeConv.id
            return (
              <div
                key={item.id}
                onClick={() => setSelectedId(item.id)}
                className={`cursor-pointer px-3.5 py-3 transition-colors ${
                  isSelected ? 'bg-[#f8fafc]' : 'hover:bg-[#f8fafc]/60'
                }`}
              >
                {/* Meta Row: Channel + Assigned Agent */}
                <div className="flex items-center justify-between text-[11px] text-slate-500">
                  <div className="flex items-center gap-1.5 truncate">
                    <span>
                      {item.channelType === 'facebook'
                        ? '💬'
                        : item.channelType === 'instagram'
                        ? '📸'
                        : item.channelType === 'whatsapp'
                        ? '📱'
                        : item.channelType === 'telegram'
                        ? '✈️'
                        : item.channelType === 'line'
                        ? '🟢'
                        : item.channelType === 'sms'
                        ? '📲'
                        : item.channelType === 'voice'
                        ? '📞'
                        : item.channelType === 'email'
                        ? '✉️'
                        : '🌐'}
                    </span>
                    <span className="truncate">{item.channel}</span>
                  </div>
                  <div className="flex items-center gap-1 shrink-0">
                    <span className="text-[10px]">👤</span>
                    <span className="truncate text-slate-600 font-medium">{item.assignedAgent}</span>
                    {item.channelType === 'website' && <span className="text-amber-500 text-[10px]">📶</span>}
                  </div>
                </div>

                {/* Sender Row: Avatar, Name, Time, Snippet */}
                <div className="mt-1.5 flex items-start gap-2.5">
                  <img
                    src={item.avatar}
                    alt={item.name}
                    className="h-8 w-8 shrink-0 rounded-full object-cover ring-1 ring-slate-100"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center justify-between">
                      <span className="font-semibold text-slate-900 text-xs truncate leading-tight">
                        {item.name}
                      </span>
                      <span className="text-[11px] text-slate-400 shrink-0 font-normal">{item.timeAgo}</span>
                    </div>
                    <p className="mt-0.5 truncate text-xs text-slate-600">{item.preview}</p>
                  </div>
                  {item.unreadCount ? (
                    <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-emerald-600 text-[10px] font-bold text-white">
                      {item.unreadCount}
                    </span>
                  ) : null}
                </div>

                {/* Bottom Tags Row */}
                <div className="mt-2 flex flex-wrap items-center gap-1.5 text-[11px]">
                  {item.resolutionType === 'autonomous_no_handoff' && (
                    <span className="inline-flex items-center gap-1 rounded bg-emerald-50 px-1.5 py-0.5 font-bold text-emerald-800 border border-emerald-200 shadow-2xs">
                      <span>✨</span>
                      <span>Resolved, no handoff</span>
                    </span>
                  )}
                  {item.sla && (
                    <span
                      className={`inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] font-semibold border ${
                        item.slaData?.isBreached || item.sla.toLowerCase().includes('breached')
                          ? 'bg-rose-50 text-rose-700 border-rose-300 animate-pulse'
                          : item.slaData?.nearingBreach || item.sla.toLowerCase().includes('warning')
                          ? 'bg-amber-50 text-amber-800 border-amber-300'
                          : 'bg-slate-50 text-slate-700 border-slate-200'
                      }`}
                    >
                      <span>
                        {item.slaData?.isBreached || item.sla.toLowerCase().includes('breached')
                          ? '⚠️'
                          : item.slaData?.nearingBreach
                          ? '⏰'
                          : '⏱️'}
                      </span>
                      <span>SLA: {item.sla}</span>
                    </span>
                  )}
                  {item.tags.map((t) => (
                    <span
                      key={t.name}
                      className="inline-flex items-center gap-1 rounded bg-[#f2f4f7] px-2 py-0.5 text-slate-700 font-medium text-[11px]"
                    >
                      <span className={`h-1.5 w-1.5 rounded-full ${t.color}`} />
                      <span>{t.name}</span>
                    </span>
                  ))}
                </div>
              </div>
            )
          })}
        </div>
      </section>

      {/* ========================================================================= */}
      {/* COLUMN 3: ACTIVE CONVERSATION WORKSPACE                                   */}
      {/* ========================================================================= */}
      <main className="flex flex-1 flex-col overflow-hidden bg-white">
        {/* Top Header of Chat Area */}
        <header className="flex h-14 shrink-0 items-center justify-between border-b border-[#eaecf0] px-6">
          <div className="flex items-center gap-3">
            <img
              src={activeConv.avatar}
              alt={activeConv.name}
              className="h-9 w-9 rounded-full object-cover ring-1 ring-slate-200"
            />
            <div>
              <div className="flex items-center gap-2">
                <h2 className="font-semibold text-slate-900 text-sm">{activeConv.name}</h2>

                {/* Live Teammates Presence Avatars */}
                {activePresences.filter((p) => p.agentName !== 'Riley Brooks').length > 0 && (
                  <div className="flex items-center gap-1.5 ml-2 pl-2 border-l border-slate-200">
                    <div className="flex -space-x-1.5 overflow-hidden">
                      {activePresences
                        .filter((p) => p.agentName !== 'Riley Brooks')
                        .map((p) => (
                          <div
                            key={p.agentName}
                            className="relative group cursor-pointer"
                            title={`${p.agentName} is ${p.action === 'typing' ? 'actively typing...' : 'viewing'}`}
                          >
                            <img
                              src={p.agentAvatar}
                              alt={p.agentName}
                              className={`h-6 w-6 rounded-full object-cover ring-2 ${
                                p.action === 'typing'
                                  ? 'ring-purple-500 animate-pulse'
                                  : 'ring-emerald-400'
                              }`}
                            />
                            <span
                              className={`absolute -bottom-0.5 -right-0.5 h-2 w-2 rounded-full ring-1 ring-white ${
                                p.action === 'typing'
                                  ? 'bg-purple-600 animate-ping'
                                  : 'bg-emerald-500'
                              }`}
                            />
                          </div>
                        ))}
                    </div>
                    <span className="text-[11px] font-semibold text-slate-600">
                      {activePresences
                        .filter((p) => p.agentName !== 'Riley Brooks')
                        .map((p) => p.agentName.split(' ')[0])
                        .join(', ')}{' '}
                      {activePresences.some(
                        (p) => p.agentName !== 'Riley Brooks' && p.action === 'typing'
                      )
                        ? 'is typing...'
                        : 'is viewing'}
                    </span>
                  </div>
                )}
              </div>
              <div className="flex items-center gap-2 text-xs text-slate-500">
                <span>#1 · {activeConv.channel}</span>

                {/* Presence Simulation Pill for quick verification */}
                <div className="inline-flex items-center gap-1 rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-medium text-slate-600 border border-slate-200">
                  <span className="text-slate-400">Simulate:</span>
                  <button
                    onClick={() => simulateTeammate('viewing')}
                    className="hover:text-slate-900 transition-colors font-semibold"
                    title="Simulate Jamie Cole viewing this conversation"
                  >
                    👀 View
                  </button>
                  <span>·</span>
                  <button
                    onClick={() => simulateTeammate('typing')}
                    className="text-purple-700 hover:text-purple-900 transition-colors font-semibold"
                    title="Simulate Jamie Cole typing a reply"
                  >
                    ✍️ Type
                  </button>
                  <span>·</span>
                  <button
                    onClick={() => simulateTeammate('left')}
                    className="hover:text-rose-600 transition-colors font-semibold"
                    title="Clear Jamie Cole presence"
                  >
                    Clear
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {activeConv.sla && (
              <span className="inline-flex items-center gap-1 rounded-full bg-rose-50 px-2.5 py-1 text-xs font-medium text-rose-600">
                <span>🔥</span>
                <span>{activeConv.sla}</span>
              </span>
            )}

            <div className="relative">
              <button
                onClick={() => setSnoozeMenuOpen((v) => !v)}
                className="inline-flex items-center gap-1.5 rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 text-xs font-semibold text-slate-700 shadow-xs hover:bg-slate-50"
              >
                <span>⏰ Snooze</span>
                <span className="text-[10px] text-slate-400">⌄</span>
              </button>
              {snoozeMenuOpen && (
                <div className="absolute right-0 top-9 z-30 w-48 rounded-lg border border-slate-200 bg-white py-1 shadow-lg text-xs">
                  <button
                    onClick={() => handleSnooze('1 hour')}
                    className="flex w-full items-center px-3 py-1.5 text-slate-700 hover:bg-slate-50"
                  >
                    ⏰ Snooze for 1 hour
                  </button>
                  <button
                    onClick={() => handleSnooze('4 hours')}
                    className="flex w-full items-center px-3 py-1.5 text-slate-700 hover:bg-slate-50"
                  >
                    ⏰ Snooze for 4 hours
                  </button>
                  <button
                    onClick={() => handleSnooze('tomorrow 9:00 AM')}
                    className="flex w-full items-center px-3 py-1.5 text-slate-700 hover:bg-slate-50"
                  >
                    ⏰ Snooze until tomorrow 9 AM
                  </button>
                  <button
                    onClick={() => handleSnooze('next Monday')}
                    className="flex w-full items-center px-3 py-1.5 text-slate-700 hover:bg-slate-50"
                  >
                    ⏰ Snooze until next Monday
                  </button>
                </div>
              )}
            </div>

            {/* Captain AI Action Buttons */}
            <button
              onClick={handleSummarizeThread}
              disabled={isSummarizing}
              className="inline-flex items-center gap-1.5 rounded-md border border-purple-200 bg-purple-50 px-2.5 py-1.5 text-xs font-semibold text-purple-700 hover:bg-purple-100 shadow-xs transition-colors disabled:opacity-50"
              title="Generate executive bulleted summary of this conversation thread"
            >
              <span>{isSummarizing ? '⏳' : '✨'}</span>
              <span>{isSummarizing ? 'Summarizing...' : 'Summarize'}</span>
            </button>

            <button
              onClick={() => handleRunTriage(true)}
              disabled={isTriaging}
              className="inline-flex items-center gap-1.5 rounded-md border border-indigo-200 bg-indigo-50 px-2.5 py-1.5 text-xs font-semibold text-indigo-700 hover:bg-indigo-100 shadow-xs transition-colors disabled:opacity-50"
              title="Captain AI parses customer intent, tags, and routes to appropriate team"
            >
              <span>{isTriaging ? '⏳' : '🎯'}</span>
              <span>{isTriaging ? 'Triaging...' : 'Auto-Triage'}</span>
            </button>

            {/* Auto-Assignment Routing Button */}
            <div className="relative group">
              <button
                onClick={() => handleAutoAssign('capacity_based')}
                disabled={isAutoAssigning}
                className="inline-flex items-center gap-1.5 rounded-md border border-teal-200 bg-teal-50 px-2.5 py-1.5 text-xs font-semibold text-teal-800 hover:bg-teal-100 shadow-xs transition-colors disabled:opacity-50 cursor-pointer"
                title="Auto-assigns to the online agent with the most available capacity slots"
              >
                <span>{isAutoAssigning ? '⏳' : '⚡'}</span>
                <span>{isAutoAssigning ? 'Routing...' : 'Auto-Assign'}</span>
                <span className="text-[10px] text-teal-600">⌄</span>
              </button>
              <div className="absolute right-0 top-9 z-30 hidden group-hover:block w-48 rounded-lg border border-slate-200 bg-white py-1 shadow-lg text-xs">
                <button
                  type="button"
                  onClick={() => handleAutoAssign('capacity_based')}
                  className="w-full text-left px-3 py-1.5 hover:bg-slate-50 text-slate-700 flex flex-col cursor-pointer"
                >
                  <span className="font-semibold text-slate-900">Capacity Routing</span>
                  <span className="text-[10px] text-slate-500">Least-loaded agent gets next ticket</span>
                </button>
                <button
                  type="button"
                  onClick={() => handleAutoAssign('round_robin')}
                  className="w-full text-left px-3 py-1.5 hover:bg-slate-50 text-slate-700 flex flex-col border-t border-slate-100 cursor-pointer"
                >
                  <span className="font-semibold text-slate-900">Round-Robin Rotation</span>
                  <span className="text-[10px] text-slate-500">Even circular distribution</span>
                </button>
              </div>
            </div>

            <button
              onClick={() => {
                const next = activeConv.status === 'Resolved' ? 'Open' : 'Resolved'
                setConversations((prev) =>
                  prev.map((c) => (c.id === activeConv.id ? { ...c, status: next } : c))
                )
              }}
              className="inline-flex items-center gap-1.5 rounded-md border border-[#d0d5dd] bg-white px-3 py-1.5 text-xs font-semibold text-slate-700 shadow-xs hover:bg-slate-50"
            >
              <span>{activeConv.status === 'Resolved' ? 'Re-open' : 'Resolve'}</span>
              <span className="text-[10px] text-slate-400">⌄</span>
            </button>

            <button
              onClick={() => setShowRightSidebar((v) => !v)}
              title="Toggle right contact sidebar"
              className="text-slate-400 hover:text-slate-600 p-1 font-bold text-sm"
            >
              ⋮
            </button>
          </div>
        </header>

        {/* Dynamic Collision Detection Alert Banner */}
        {collisionInfo?.hasCollision && !collisionWarningDismissed && (
          <div
            className={`flex items-center justify-between border-b px-6 py-2.5 text-xs font-medium transition-colors ${
              collisionInfo.severity === 'danger'
                ? 'bg-rose-50 border-rose-300 text-rose-950 animate-pulse'
                : 'bg-amber-50 border-amber-200 text-amber-900'
            }`}
          >
            <div className="flex items-center gap-2.5">
              <span className="text-base">
                {collisionInfo.severity === 'danger' ? '✍️' : '👀'}
              </span>
              <span>{collisionInfo.warningMessage}</span>
            </div>
            <div className="flex items-center gap-2">
              {collisionInfo.severity === 'danger' && (
                <button
                  onClick={() => setComposerMode('note')}
                  className="rounded border border-rose-300 bg-white px-2 py-0.5 text-[11px] font-bold text-rose-800 hover:bg-rose-100 shadow-2xs"
                >
                  Switch to Private Note 🔒
                </button>
              )}
              <button
                onClick={() => setCollisionWarningDismissed(true)}
                className="text-slate-500 hover:text-slate-800 font-bold px-1"
                title="Dismiss warning"
              >
                ✕
              </button>
            </div>
          </div>
        )}

        {/* Macro Execution Notification Banner */}
        {macroBanner && (
          <div className="flex items-center justify-between border-b border-emerald-200 bg-emerald-50 px-6 py-2.5 text-xs font-medium text-emerald-900 animate-in fade-in slide-in-from-top-1">
            <div className="flex items-center gap-2">
              <span className="flex h-5 w-5 items-center justify-center rounded-md bg-emerald-600 text-white font-bold text-[10px] shadow-2xs">
                ⚡
              </span>
              <span>
                Executed Macro: <strong className="text-emerald-950 font-bold">{macroBanner.name}</strong>
              </span>
              <span className="text-emerald-700 text-[11px]">
                ({macroBanner.count} automated steps applied
                {macroBanner.state ? ` · State: ${macroBanner.state}` : ''}
                {macroBanner.priority ? ` · Priority: ${macroBanner.priority}` : ''})
              </span>
            </div>
            <button
              onClick={() => setMacroBanner(null)}
              className="text-emerald-600 hover:text-emerald-950 font-bold px-1"
              title="Dismiss"
            >
              ✕
            </button>
          </div>
        )}

        {/* Routing Notification Toast Banner */}
        {routingToast && (
          <div
            className={`flex items-center justify-between border-b px-6 py-2.5 text-xs font-medium animate-in fade-in slide-in-from-top-1 ${
              routingToast.isOverflow
                ? 'bg-rose-50 border-rose-300 text-rose-950'
                : 'bg-teal-50 border-teal-200 text-teal-950'
            }`}
          >
            <div className="flex items-center gap-2">
              <span
                className={`flex h-5 w-5 items-center justify-center rounded-md text-white font-bold text-[10px] shadow-2xs ${
                  routingToast.isOverflow ? 'bg-rose-600' : 'bg-teal-600'
                }`}
              >
                {routingToast.isOverflow ? '⚠️' : '⚡'}
              </span>
              <span>{routingToast.message}</span>
            </div>
            <button
              onClick={() => setRoutingToast(null)}
              className="text-slate-500 hover:text-slate-800 font-bold px-1 cursor-pointer"
              title="Dismiss"
            >
              ✕
            </button>
          </div>
        )}

        {/* Sub-header Tabs (Messages / Customer Dashboard) */}
        <div className="flex items-center justify-between border-b border-[#eaecf0] px-6 pt-2">
          <div className="flex items-center gap-6 text-xs font-medium">
            <button className="border-b-2 border-blue-600 pb-2 text-blue-600 font-semibold">
              Messages
            </button>
            <button className="pb-2 text-slate-500 hover:text-slate-800">
              Customer Dashboard
            </button>
            {activeConv.channelType === 'whatsapp' && (
              <div className="flex items-center gap-2 pb-2">
                <span className="flex items-center gap-1.5 rounded-full bg-emerald-50 px-2.5 py-0.5 text-[11px] font-semibold text-emerald-700 border border-emerald-200">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
                  <span>24h WhatsApp Session: Active</span>
                </span>
                <button
                  onClick={() => {
                    const choice = prompt(
                      'Select Approved Meta Template to insert:\n1. spring_sale_offer (Marketing)\n2. order_confirmation (Utility)\n3. auth_otp_code (Authentication)\n\nEnter 1, 2, or 3:'
                    )
                    if (choice === '1') {
                      setReplyText('Hi Maya, our spring sale starts Friday. As a VIP customer you get early access and 20% off until Sunday.')
                    } else if (choice === '2') {
                      setReplyText('Hello Maya, thank you for your order #49281. Your items are being prepared for shipping.')
                    } else if (choice === '3') {
                      setReplyText('849201 is your one-time verification code. For your security, do not share this code.')
                    }
                  }}
                  className="rounded border border-slate-300 bg-white px-2 py-0.5 text-[11px] font-semibold text-slate-700 hover:bg-slate-50"
                  title="Send an Approved WhatsApp Template (required after 24h window expires)"
                >
                  📋 Re-engagement Template
                </button>
              </div>
            )}
          </div>
          <div className="flex items-center gap-3 pb-2 text-slate-400 text-xs">
            <button title="Customer Info" className="hover:text-slate-600">
              👤
            </button>
            <button title="Video Call" className="hover:text-slate-600">
              📹
            </button>
          </div>
        </div>

        {/* Chat Stream Timeline */}
        <div className="flex-1 overflow-y-auto px-6 py-6 space-y-4 bg-white">
          {/* Pinned WhatsApp Campaign Info Banner */}
          {activeConv.campaign && (
            <div className="rounded-xl border border-blue-200 bg-blue-50/70 p-3.5 text-xs text-blue-950 flex items-center justify-between shadow-xs">
              <div className="flex items-center gap-3">
                <span className="text-2xl">📢</span>
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-slate-900">
                      WhatsApp Campaign: {activeConv.campaign.name}
                    </span>
                    <span className="rounded bg-blue-100 text-blue-800 px-1.5 py-0.5 text-[10px] font-semibold">
                      {activeConv.campaign.status}
                    </span>
                  </div>
                  <p className="text-[11px] text-blue-700 mt-0.5">
                    Sent {activeConv.campaign.sentAt} · Customer replied directly to this campaign message
                  </p>
                </div>
              </div>
              <Link
                to="/broadcasts"
                className="rounded-md border border-blue-300 bg-white px-2.5 py-1 text-xs font-semibold text-blue-700 hover:bg-blue-50 shadow-xs"
              >
                View Campaign Analytics ↗
              </Link>
            </div>
          )}

          {activeConv.messages.map((msg) => {
            const isAgent = msg.sender === 'agent'
            const isNote = msg.sender === 'note'

            if (isNote) {
              return (
                <div key={msg.id} className="mx-auto max-w-lg rounded-xl border border-amber-200 bg-amber-50 p-3 text-xs text-amber-900 shadow-xs">
                  <div className="flex items-center justify-between pb-1 font-semibold text-amber-800">
                    <span>📝 Private Team Note</span>
                    <span className="font-normal text-amber-600">{msg.time}</span>
                  </div>
                  <p>{msg.text}</p>
                </div>
              )
            }

            if (isAgent) {
              return (
                <div key={msg.id} className="flex items-end justify-end gap-2">
                  <div className="max-w-[520px]">
                    <div className="rounded-2xl rounded-br-xs bg-[#e0f2fe] px-4 py-3 text-[13.5px] leading-relaxed text-slate-900 shadow-xs">
                      {msg.text}
                    </div>
                    <div className="mt-1 flex items-center justify-end gap-1.5 text-[11px] text-slate-400">
                      <span>{msg.time}</span>
                      <span className="text-blue-600 font-bold">✓✓</span>
                    </div>
                  </div>
                  <img
                    src="https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80"
                    alt="Riley Brooks"
                    className="h-6 w-6 rounded-full object-cover mb-4"
                  />
                </div>
              )
            }

            // Customer message
            return (
              <div key={msg.id} className="flex items-end justify-start gap-2">
                <div className="max-w-[520px]">
                  <div className="rounded-2xl rounded-bl-xs bg-[#f2f4f7] px-4 py-3 text-[13.5px] leading-relaxed text-slate-900 shadow-xs">
                    {msg.text}
                  </div>
                  <div className="mt-1 text-[11px] text-slate-400">{msg.time}</div>
                </div>
              </div>
            )
          })}
        </div>

        {/* Message Composer Area */}
        <div className="p-4 pt-0 bg-white">
          {activeConv.captainDraft && (
            <div className="mb-2.5 rounded-xl border border-purple-200 bg-purple-50/80 p-3 text-xs text-purple-950 shadow-xs">
              <div className="flex items-center justify-between mb-1.5">
                <div className="flex items-center gap-1.5 font-bold text-purple-900">
                  <span>✨</span>
                  <span>Captain AI Suggested Reply</span>
                  <span className="rounded-full bg-purple-200 text-purple-800 px-2 py-0.5 text-[10px]">
                    {captainSuggestedReply ? `${Math.round(captainSuggestedReply.confidence * 100)}% Confidence` : 'Grounded in Knowledge Base'}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setReplyText(activeConv.captainDraft || '')}
                    className="rounded border border-purple-300 bg-white px-2 py-0.5 font-semibold text-purple-700 hover:bg-purple-100"
                  >
                    Insert Draft
                  </button>
                  <button
                    onClick={() => {
                      if (!activeConv.captainDraft) return
                      const newMsg: MessageItem = {
                        id: `m-${Date.now()}`,
                        sender: 'agent',
                        text: activeConv.captainDraft,
                        time: 'Just now',
                        status: 'read',
                        agentName: 'Captain AI',
                      }
                      setConversations((prev) =>
                        prev.map((c) =>
                          c.id === activeConv.id
                            ? {
                                ...c,
                                preview: `↩ ${activeConv.captainDraft?.slice(0, 32)}...`,
                                messages: [...c.messages, newMsg],
                                captainDraft: null,
                              }
                            : c
                        )
                      )
                    }}
                    className="rounded bg-purple-700 px-2.5 py-0.5 font-bold text-white hover:bg-purple-800 shadow-xs"
                  >
                    Send Immediately
                  </button>
                </div>
              </div>
              <p className="rounded-md border border-purple-100 bg-white/95 p-2 text-xs font-medium text-slate-800">
                "{activeConv.captainDraft}"
              </p>
            </div>
          )}

          {/* ========================================================================= */}
          {/* COPILOT TEAMMATE APPROVAL & WORKFLOW CARD                                  */}
          {/* ========================================================================= */}
          {activeApproval && (
            <div className="mb-3 rounded-2xl border-2 border-amber-300 bg-linear-to-r from-amber-50 via-orange-50/60 to-amber-50 p-4 shadow-sm animate-in fade-in slide-in-from-top-2 duration-200 space-y-3">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-amber-200/80 pb-2.5">
                <div className="flex items-center gap-2">
                  <div className="w-7 h-7 rounded-lg bg-amber-500 text-white flex items-center justify-center font-bold text-xs shadow-xs">
                    🤖
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <h4 className="font-bold text-amber-950 text-xs">
                        Captain AI requests teammate approval before answering
                      </h4>
                      <span className="rounded-full bg-amber-200 text-amber-900 border border-amber-300 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider">
                        {activeApproval.requestType || activeApproval.request_type || 'APPROVAL'}
                      </span>
                    </div>
                    <p className="text-[11px] text-amber-800 font-medium">
                      {activeApproval.title}
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-2 text-xs">
                  <span className="text-[11px] font-semibold text-amber-700 bg-amber-100/90 border border-amber-300 px-2 py-0.5 rounded-md">
                    Policy Sign-Off Required
                  </span>
                </div>
              </div>

              {/* Action Context Breakdown */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px] bg-white/80 rounded-xl p-2.5 border border-amber-200/60">
                <div>
                  <span className="font-bold text-slate-500 uppercase text-[9.5px] block">Action Context:</span>
                  <p className="text-slate-800 font-medium">
                    {activeApproval.internalNote || activeApproval.internal_note}
                  </p>
                </div>
                <div>
                  <span className="font-bold text-slate-500 uppercase text-[9.5px] block">Details &amp; Policy:</span>
                  <div className="text-slate-700 space-y-0.5 font-mono text-[10.5px]">
                    {activeApproval.actionData?.amount && (
                      <span className="inline-block bg-amber-100 text-amber-900 font-bold px-1.5 py-0.2 rounded mr-1">
                        Amount: ${Number(activeApproval.actionData.amount).toFixed(2)}
                      </span>
                    )}
                    {activeApproval.actionData?.order_id && (
                      <span className="inline-block bg-slate-100 text-slate-800 px-1.5 py-0.2 rounded mr-1">
                        Order #{activeApproval.actionData.order_id}
                      </span>
                    )}
                    {activeApproval.actionData?.policy_rule && (
                      <span className="block text-[10px] text-amber-800 font-sans mt-0.5">
                        📜 {activeApproval.actionData.policy_rule}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              {/* Question to Teammate (if info_request) */}
              {(activeApproval.questionToTeammate || activeApproval.question_to_teammate) && (
                <div className="bg-amber-100/60 border border-amber-300 rounded-xl p-3 space-y-2">
                  <div className="flex items-center gap-1.5 text-xs font-bold text-amber-900">
                    <span>❓</span>
                    <span>Question from AI: {activeApproval.questionToTeammate || activeApproval.question_to_teammate}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      placeholder="Type information for AI (e.g. 4 units in stock in Bay C)..."
                      value={teammateInfoInput}
                      onChange={(e) => setTeammateInfoInput(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          handleProvideTeammateInfo(activeApproval.id, teammateInfoInput)
                        }
                      }}
                      className="flex-1 bg-white border border-amber-300 rounded-lg px-2.5 py-1.5 text-xs text-slate-800 focus:outline-hidden focus:border-amber-500"
                    />
                    <button
                      type="button"
                      disabled={isProcessingApproval || !teammateInfoInput.trim()}
                      onClick={() => handleProvideTeammateInfo(activeApproval.id, teammateInfoInput)}
                      className="bg-amber-600 hover:bg-amber-700 text-white font-bold text-xs px-3 py-1.5 rounded-lg shadow-xs transition-colors disabled:opacity-50 cursor-pointer shrink-0"
                    >
                      {isProcessingApproval ? 'Submitting…' : 'Submit & Finish Conversation'}
                    </button>
                  </div>
                </div>
              )}

              {/* Proposed Reply & Action Buttons */}
              {activeApproval.requestType !== 'info_request' && activeApproval.request_type !== 'info_request' && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-[11px] font-bold text-amber-950">
                    <span>Proposed Reply to Customer:</span>
                    {editingApprovalReplyId !== activeApproval.id && (
                      <button
                        type="button"
                        onClick={() => {
                          setEditingApprovalReplyId(activeApproval.id)
                          setEditedApprovalReply(activeApproval.proposedReply || activeApproval.proposed_reply || '')
                        }}
                        className="text-amber-800 hover:text-amber-950 font-semibold underline cursor-pointer"
                      >
                        ✏️ Edit Reply Before Approval
                      </button>
                    )}
                  </div>

                  {editingApprovalReplyId === activeApproval.id ? (
                    <div className="space-y-2">
                      <textarea
                        rows={3}
                        value={editedApprovalReply}
                        onChange={(e) => setEditedApprovalReply(e.target.value)}
                        className="w-full text-xs border border-amber-400 bg-white rounded-xl p-2.5 text-slate-800 focus:outline-hidden"
                      />
                      <div className="flex items-center justify-end gap-2">
                        <button
                          type="button"
                          onClick={() => setEditingApprovalReplyId(null)}
                          className="px-2.5 py-1 text-xs text-slate-600 hover:text-slate-800"
                        >
                          Cancel
                        </button>
                        <button
                          type="button"
                          disabled={isProcessingApproval}
                          onClick={() => handleApproveRequest(activeApproval, editedApprovalReply)}
                          className="px-3.5 py-1.5 text-xs bg-emerald-600 hover:bg-emerald-700 text-white font-bold rounded-lg shadow-xs transition-colors cursor-pointer"
                        >
                          {isProcessingApproval ? 'Sending…' : 'Approve Edited Reply & Let AI Send'}
                        </button>
                      </div>
                    </div>
                  ) : (
                    <div className="p-3 bg-white rounded-xl border border-amber-200/80 text-xs text-slate-800 leading-relaxed font-normal">
                      "{activeApproval.proposedReply || activeApproval.proposed_reply}"
                    </div>
                  )}

                  {/* Reject Feedback Input Box if rejecting */}
                  {rejectingApprovalId === activeApproval.id ? (
                    <div className="p-3 bg-rose-50 border border-rose-200 rounded-xl space-y-2">
                      <label className="block text-[11px] font-bold text-rose-900">
                        Reason for Rejection:
                      </label>
                      <input
                        type="text"
                        placeholder="e.g. Return window expired, request photos first..."
                        value={rejectionFeedback}
                        onChange={(e) => setRejectionFeedback(e.target.value)}
                        className="w-full text-xs border border-rose-300 bg-white rounded-lg px-2.5 py-1.5 text-slate-800 focus:outline-hidden"
                      />
                      <div className="flex justify-end gap-2">
                        <button
                          type="button"
                          onClick={() => setRejectingApprovalId(null)}
                          className="px-2.5 py-1 text-xs text-slate-600"
                        >
                          Cancel
                        </button>
                        <button
                          type="button"
                          disabled={isProcessingApproval}
                          onClick={() => handleRejectRequest(activeApproval.id, rejectionFeedback)}
                          className="px-3 py-1 text-xs bg-rose-600 hover:bg-rose-700 text-white font-bold rounded-lg shadow-xs transition-colors"
                        >
                          Confirm Rejection
                        </button>
                      </div>
                    </div>
                  ) : (
                    editingApprovalReplyId !== activeApproval.id && (
                      <div className="flex items-center justify-between pt-1">
                        <button
                          type="button"
                          onClick={() => setRejectingApprovalId(activeApproval.id)}
                          className="text-xs text-rose-600 hover:text-rose-800 font-semibold px-2 py-1 rounded hover:bg-rose-50 cursor-pointer transition-colors"
                        >
                          ✕ Reject Request
                        </button>

                        <button
                          type="button"
                          disabled={isProcessingApproval}
                          onClick={() => handleApproveRequest(activeApproval)}
                          className="flex items-center gap-1.5 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs rounded-xl shadow-md transition-all cursor-pointer transform hover:-translate-y-0.5"
                        >
                          <span>✅</span>
                          <span>{isProcessingApproval ? 'Processing…' : 'Approve & Let AI Send Reply'}</span>
                        </button>
                      </div>
                    )
                  )}
                </div>
              )}
            </div>
          )}

          <div className="relative">
            {/* Canned Responses Floating Autocomplete Menu */}
            {(isTypingSlash || showCannedPicker) && (
              <div className="absolute bottom-full mb-2 left-0 right-0 z-40 max-h-80 overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl animate-in fade-in slide-in-from-bottom-2 duration-150 flex flex-col">
                {/* Header & Category Filter Tabs */}
                <div className="bg-slate-50/90 border-b border-slate-200/80 px-3 py-2 flex flex-col gap-1.5 shrink-0">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1.5 font-bold text-xs text-slate-800">
                      <span className="flex items-center justify-center w-5 h-5 rounded-full bg-blue-100 text-blue-700 text-xs">
                        /
                      </span>
                      <span>Slash Commands &amp; Canned Replies</span>
                      <span className="text-[11px] font-normal text-slate-400">
                        ({filteredCanned.length} available)
                      </span>
                    </div>
                    <button
                      type="button"
                      onClick={() => setShowCannedModal(true)}
                      className="text-[11px] text-blue-600 hover:text-blue-800 font-semibold flex items-center gap-1 hover:underline cursor-pointer"
                    >
                      ⚙️ Manage Canned
                    </button>
                  </div>

                  <div className="flex items-center gap-1 overflow-x-auto pb-0.5 no-scrollbar">
                    {cannedCategories.map((cat) => (
                      <button
                        key={cat}
                        type="button"
                        onClick={() => {
                          setCannedCategory(cat)
                          setCannedSelectedIndex(0)
                        }}
                        className={`px-2 py-0.5 rounded-full text-[10.5px] font-semibold capitalize transition-all shrink-0 cursor-pointer ${
                          cannedCategory === cat
                            ? 'bg-blue-600 text-white shadow-xs'
                            : 'bg-white text-slate-600 hover:bg-slate-200/70 border border-slate-200'
                        }`}
                      >
                        {cat}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Autocomplete Suggestions List */}
                <div className="max-h-52 overflow-y-auto divide-y divide-slate-100/80 p-1">
                  {filteredCanned.length === 0 ? (
                    <div className="py-5 px-4 text-center">
                      <p className="text-xs text-slate-500 font-medium">
                        No canned replies match <span className="font-mono text-slate-800 font-bold">/{slashQuery}</span>
                        {cannedCategory !== 'all' ? ` in ${cannedCategory}` : ''}
                      </p>
                      <button
                        type="button"
                        onClick={() => {
                          setNewCannedCode(slashQuery)
                          setShowCannedModal(true)
                        }}
                        className="mt-2 inline-flex items-center gap-1 text-xs font-bold text-blue-600 hover:text-blue-800 hover:underline cursor-pointer"
                      >
                        + Create "/{slashQuery}" now
                      </button>
                    </div>
                  ) : (
                    filteredCanned.map((c, idx) => {
                      const isSelected = idx === cannedSelectedIndex
                      const resolvedPreview = resolveCannedVariables(c.content)
                      return (
                        <button
                          key={c.id || c.shortCode || c.short_code}
                          type="button"
                          onClick={() => insertCanned(c)}
                          onMouseEnter={() => setCannedSelectedIndex(idx)}
                          className={`flex w-full flex-col items-start gap-1 rounded-xl p-2.5 text-left transition-all cursor-pointer ${
                            isSelected
                              ? 'bg-blue-50/90 border-l-4 border-blue-600 shadow-xs'
                              : 'hover:bg-slate-50/80 border-l-4 border-transparent'
                          }`}
                        >
                          <div className="flex items-center justify-between w-full">
                            <div className="flex items-center gap-2">
                              <span className="font-mono text-xs font-black text-blue-700 bg-blue-100/80 px-2 py-0.5 rounded-md">
                                /{c.shortCode || c.short_code}
                              </span>
                              <span className="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded-full bg-slate-100 text-slate-600 border border-slate-200">
                                {c.category || 'general'}
                              </span>
                            </div>
                            {isSelected && (
                              <span className="text-[10px] font-semibold text-blue-600 animate-pulse">
                                ↵ or Tab to insert
                              </span>
                            )}
                          </div>
                          <p className="text-xs text-slate-700 line-clamp-2 w-full mt-0.5 font-normal">
                            {resolvedPreview}
                          </p>
                        </button>
                      )
                    })
                  )}
                </div>

                {/* Footer hints */}
                <div className="bg-slate-50 px-3 py-1.5 border-t border-slate-200/80 flex items-center justify-between text-[10.5px] text-slate-500 shrink-0 font-medium">
                  <span>
                    💡 Type <code className="bg-slate-200 px-1 py-0.5 rounded text-slate-800 font-mono text-[10px]">/</code> to trigger autocomplete
                  </span>
                  <div className="flex items-center gap-2 text-slate-400">
                    <span>↑↓ navigate</span>
                    <span>•</span>
                    <span>↵ / Tab insert</span>
                    <span>•</span>
                    <span>esc dismiss</span>
                  </div>
                </div>
              </div>
            )}

            <div
              className={`rounded-xl border shadow-xs transition-colors ${
                composerMode === 'note'
                  ? 'border-amber-300 bg-amber-50/40 focus-within:border-amber-500 focus-within:ring-1 focus-within:ring-amber-500'
                  : 'border-[#d0d5dd] bg-white focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500'
              }`}
            >
              {/* Top Composer Control Tabs */}
              <div className="flex items-center justify-between border-b border-[#f2f4f7] px-3 py-1.5">
                <div className="flex items-center gap-1 rounded bg-[#f2f4f7] p-0.5 text-xs font-medium">
                  <button
                    onClick={() => setComposerMode('reply')}
                    className={`rounded px-2.5 py-1 transition-colors ${
                      composerMode === 'reply' ? 'bg-white text-slate-900 shadow-xs font-semibold' : 'text-slate-600'
                    }`}
                  >
                    Reply
                  </button>
                  <button
                    onClick={() => setComposerMode('note')}
                    className={`rounded px-2.5 py-1 transition-colors ${
                      composerMode === 'note' ? 'bg-amber-200 text-amber-950 shadow-xs font-bold' : 'text-slate-600'
                    }`}
                  >
                    Private Note 🔒
                  </button>
                </div>

                <div className="flex items-center gap-2 text-slate-400">
                  <button
                    onClick={handleAIDraft}
                    title="Generate AI reply draft"
                    className="rounded p-1 text-purple-600 hover:bg-purple-50"
                  >
                    ✨
                  </button>
                  <button
                    type="button"
                    onClick={handleTriggerRefundApprovalDemo}
                    title="Ask teammate to approve refund reply (AI approval workflow)"
                    className="rounded px-2 py-0.5 text-[10.5px] font-bold text-amber-800 bg-amber-100 hover:bg-amber-200 border border-amber-300 flex items-center gap-1 transition-colors cursor-pointer"
                  >
                    <span>💳</span>
                    <span>Ask Refund Approval</span>
                  </button>
                  <button title="Expand editor" className="rounded p-1 hover:text-slate-600">
                    ⤢
                  </button>
                </div>
              </div>

              {/* Private Note Warning Strip */}
              {composerMode === 'note' && (
                <div className="flex items-center gap-1.5 bg-amber-100/70 border-b border-amber-200/80 px-3 py-1 text-[11px] font-medium text-amber-900">
                  <span>🔒</span>
                  <span>Visible only to internal teammates. Will not be sent to WhatsApp or customer.</span>
                </div>
              )}

              {/* Composer Textarea */}
              <textarea
                rows={3}
                value={replyText}
                onChange={(e) => handleComposerTextChange(e.target.value)}
                onKeyDown={(e) => {
                  const isPickerOpen = (isTypingSlash || showCannedPicker) && filteredCanned.length > 0
                  if (isPickerOpen) {
                    if (e.key === 'ArrowDown') {
                      e.preventDefault()
                      setCannedSelectedIndex((prev) => (prev + 1) % filteredCanned.length)
                      return
                    }
                    if (e.key === 'ArrowUp') {
                      e.preventDefault()
                      setCannedSelectedIndex((prev) => (prev - 1 + filteredCanned.length) % filteredCanned.length)
                      return
                    }
                    if (e.key === 'Enter' || e.key === 'Tab') {
                      if (!e.shiftKey && !e.ctrlKey && !e.metaKey) {
                        e.preventDefault()
                        if (filteredCanned[cannedSelectedIndex]) {
                          insertCanned(filteredCanned[cannedSelectedIndex])
                        }
                        return
                      }
                    }
                    if (e.key === 'Escape') {
                      e.preventDefault()
                      setShowCannedPicker(false)
                      return
                    }
                  }
                  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                    e.preventDefault()
                    handleSend()
                  }
                }}
                placeholder={
                  composerMode === 'note'
                    ? 'Write an internal team note... (Mention teammates or note customer details)'
                    : "Shift + enter for new line. Start with '/' to select a Canned Response."
                }
                className={`w-full resize-none px-3.5 py-2.5 text-xs text-slate-800 placeholder-slate-400 focus:outline-hidden ${
                  composerMode === 'note' ? 'bg-amber-50/20' : 'bg-transparent'
                }`}
              />

              {/* Bottom Toolbar & Send Button */}
              <div className="flex flex-col sm:flex-row sm:items-center justify-between border-t border-[#f2f4f7] px-3 py-2 gap-2">
                <div className="flex items-center gap-2 text-slate-400 text-sm">
                  <button title="Insert emoji" className="hover:text-slate-600">
                    😊
                  </button>
                  <button title="Attach file" className="hover:text-slate-600">
                    📎
                  </button>
                  <button title="Record voice note" className="hover:text-slate-600">
                    🎙️
                  </button>
                  <button
                    onClick={() => setShowCannedPicker((v) => !v)}
                    title="Canned responses (Saved replies)"
                    className={`rounded p-1 transition-colors ${
                      showCannedPicker ? 'bg-blue-100 text-blue-700' : 'hover:text-slate-600'
                    }`}
                  >
                    ⚡
                  </button>
                  <button
                    onClick={() => handleGenerateCaptainReply()}
                    disabled={isGeneratingReply}
                    title="Generate suggested reply with Captain AI"
                    className="inline-flex items-center gap-1 rounded bg-purple-50 border border-purple-200 px-2 py-0.5 text-[11px] font-semibold text-purple-700 hover:bg-purple-100 transition-colors disabled:opacity-50"
                  >
                    <span>{isGeneratingReply ? '⏳' : '✨'}</span>
                    <span>{isGeneratingReply ? 'Thinking...' : 'Captain AI'}</span>
                  </button>
                </div>

                {/* Tone Rewriter Pills when text is typed */}
                {replyText.trim() && (
                  <div className="flex items-center gap-1 text-[11px] overflow-x-auto py-0.5">
                    <span className="text-slate-400 text-[10px] font-bold uppercase tracking-wider">Rewrite:</span>
                    <button
                      onClick={() => handleRewriteTone('professional')}
                      disabled={isRewriting}
                      className="rounded bg-slate-100 px-1.5 py-0.5 text-slate-700 hover:bg-slate-200 font-medium transition-colors"
                      title="Make more professional"
                    >
                      👔 Formal
                    </button>
                    <button
                      onClick={() => handleRewriteTone('friendly')}
                      disabled={isRewriting}
                      className="rounded bg-slate-100 px-1.5 py-0.5 text-slate-700 hover:bg-slate-200 font-medium transition-colors"
                      title="Make friendlier"
                    >
                      😊 Warm
                    </button>
                    <button
                      onClick={() => handleRewriteTone('shorten')}
                      disabled={isRewriting}
                      className="rounded bg-slate-100 px-1.5 py-0.5 text-slate-700 hover:bg-slate-200 font-medium transition-colors"
                      title="Shorten and simplify"
                    >
                      ✂️ Short
                    </button>
                    <button
                      onClick={() => handleRewriteTone('fix_spelling')}
                      disabled={isRewriting}
                      className="rounded bg-slate-100 px-1.5 py-0.5 text-slate-700 hover:bg-slate-200 font-medium transition-colors"
                      title="Fix spelling and grammar"
                    >
                      📝 Grammar
                    </button>
                  </div>
                )}

                <button
                  onClick={handleSend}
                  disabled={!replyText.trim()}
                  className={`inline-flex items-center gap-1.5 rounded-lg px-3.5 py-1.5 text-xs font-semibold text-white shadow-xs disabled:opacity-40 transition-colors ${
                    composerMode === 'note'
                      ? 'bg-amber-600 hover:bg-amber-700'
                      : 'bg-[#1570ef] hover:bg-blue-700'
                  }`}
                >
                  <span>{composerMode === 'note' ? 'Add Private Note 🔒' : 'Send (⌘ + ↵)'}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </main>

      {/* ========================================================================= */}
      {/* COLUMN 4: CONTACT & CONVERSATION DETAILS SIDEBAR                         */}
      {/* ========================================================================= */}
      {showRightSidebar && (
        <aside className="flex w-[330px] shrink-0 flex-col border-l border-[#eaecf0] bg-white text-[13px] overflow-y-auto">
          {/* Header */}
          <div className="flex h-14 shrink-0 items-center justify-between border-b border-[#eaecf0] px-5">
            <h3 className="font-semibold text-slate-900 text-sm">Contact & Profile</h3>
            <button
              onClick={() => setShowRightSidebar(false)}
              className="text-slate-400 hover:text-slate-600 font-bold"
            >
              ✕
            </button>
          </div>

          {/* Contact Sub-Tabs */}
          <div className="flex items-center justify-around border-b border-[#eaecf0] px-3 pt-2 text-xs font-medium">
            <button
              onClick={() => setContactSidebarTab('overview')}
              className={`pb-2 border-b-2 transition-colors ${
                contactSidebarTab === 'overview'
                  ? 'border-blue-600 font-semibold text-blue-600'
                  : 'border-transparent text-slate-500 hover:text-slate-800'
              }`}
            >
              Overview
            </button>
            <button
              onClick={() => setContactSidebarTab('notes')}
              className={`pb-2 border-b-2 transition-colors flex items-center gap-1 ${
                contactSidebarTab === 'notes'
                  ? 'border-blue-600 font-semibold text-blue-600'
                  : 'border-transparent text-slate-500 hover:text-slate-800'
              }`}
            >
              <span>Notes</span>
              <span className="rounded-full bg-slate-100 px-1.5 text-[10px]">
                {(contactNotes[activeConv.id] || []).length}
              </span>
            </button>
            <button
              onClick={() => setContactSidebarTab('attributes')}
              className={`pb-2 border-b-2 transition-colors ${
                contactSidebarTab === 'attributes'
                  ? 'border-blue-600 font-semibold text-blue-600'
                  : 'border-transparent text-slate-500 hover:text-slate-800'
              }`}
            >
              Attributes
            </button>
            <button
              onClick={() => setContactSidebarTab('history')}
              className={`pb-2 border-b-2 transition-colors ${
                contactSidebarTab === 'history'
                  ? 'border-blue-600 font-semibold text-blue-600'
                  : 'border-transparent text-slate-500 hover:text-slate-800'
              }`}
            >
              History
            </button>
            <button
              onClick={() => setContactSidebarTab('billing')}
              className={`pb-2 border-b-2 transition-colors flex items-center gap-1 ${
                contactSidebarTab === 'billing'
                  ? 'border-indigo-600 font-semibold text-indigo-600'
                  : 'border-transparent text-slate-500 hover:text-slate-800'
              }`}
            >
              <span>💳 Billing</span>
              {stripeBilling?.subscription?.status === 'active' && (
                <span className="rounded-full bg-emerald-100 text-emerald-800 font-bold px-1 text-[9px]">
                  MRR
                </span>
              )}
            </button>
          </div>

          <div className="p-5 space-y-6">
            {/* SUBTAB 1: OVERVIEW */}
            {contactSidebarTab === 'overview' && (
              <>
                {/* Customer Avatar & Bio Section */}
                <div className="flex flex-col items-center text-center">
                  <img
                    src={activeConv.avatar}
                    alt={activeConv.name}
                    className="h-18 w-18 rounded-full object-cover ring-2 ring-slate-100 shadow-xs"
                  />
                  <div className="mt-3 flex items-center gap-1.5">
                    <h4 className="font-bold text-slate-900 text-[15px]">{activeConv.name}</h4>
                    <button title="Information" className="text-slate-400 hover:text-slate-600 text-xs">
                      ⓘ
                    </button>
                    <button title="Open CRM record" className="text-slate-400 hover:text-slate-600 text-xs">
                      ↗
                    </button>
                  </div>
                  <p className="mt-2 text-xs text-slate-600 leading-relaxed max-w-[260px]">
                    {activeConv.bio}
                  </p>
                </div>

                {/* Contact Details List */}
                <div className="space-y-2.5 text-xs text-slate-700">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 truncate">
                      <span className="text-slate-400">✉️</span>
                      <span className="truncate">{activeConv.email}</span>
                    </div>
                    <button
                      onClick={() => handleCopy(activeConv.email, 'email')}
                      title="Copy email"
                      className="text-slate-400 hover:text-slate-600 shrink-0"
                    >
                      {copiedField === 'email' ? '✓' : '📋'}
                    </button>
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 truncate">
                      <span className="text-slate-400">📞</span>
                      <span>{activeConv.phone}</span>
                    </div>
                    <button
                      onClick={() => handleCopy(activeConv.phone, 'phone')}
                      title="Copy phone"
                      className="text-slate-400 hover:text-slate-600 shrink-0"
                    >
                      {copiedField === 'phone' ? '✓' : '📋'}
                    </button>
                  </div>

                  <div className="flex items-center gap-2">
                    <span className="text-slate-400">🏢</span>
                    <span>{activeConv.company}</span>
                  </div>

                  <div className="flex items-center gap-2">
                    <span className="text-slate-400">📍</span>
                    <span>{activeConv.location}</span>
                  </div>
                </div>

                {/* Social Links Row */}
                <div className="flex items-center justify-center gap-3 text-slate-400 text-sm">
                  <button title="Twitter" className="hover:text-slate-600">
                    🐦
                  </button>
                  <button title="LinkedIn" className="hover:text-slate-600">
                    💼
                  </button>
                  <button title="GitHub" className="hover:text-slate-600">
                    🐙
                  </button>
                  <button title="Instagram" className="hover:text-slate-600">
                    📸
                  </button>
                </div>

                {/* Quick Actions Buttons Row */}
                <div className="flex items-center justify-center gap-2">
                  <button title="Start chat" className="flex h-8 w-8 items-center justify-center rounded-lg border border-[#eaecf0] hover:bg-slate-50 text-slate-600">
                    💬
                  </button>
                  <button title="Edit contact" className="flex h-8 w-8 items-center justify-center rounded-lg border border-[#eaecf0] hover:bg-slate-50 text-slate-600">
                    ✏️
                  </button>
                  <button title="Filter by contact" className="flex h-8 w-8 items-center justify-center rounded-lg border border-[#eaecf0] hover:bg-slate-50 text-slate-600">
                    ⚡
                  </button>
                  <button title="Delete" className="flex h-8 w-8 items-center justify-center rounded-lg border border-[#eaecf0] hover:bg-rose-50 text-rose-500">
                    🗑️
                  </button>
                </div>

                {/* Gap 3: Issue Trackers (Linear / GitHub / Jira) Card */}
                <div className="rounded-xl border border-purple-100 bg-gradient-to-br from-purple-50/60 via-white to-slate-50 p-3 shadow-2xs space-y-2.5">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1.5">
                      <span className="text-sm">🎯</span>
                      <span className="font-bold text-xs text-slate-900">Issue Trackers</span>
                      <div className="flex items-center gap-1 ml-1 text-[11px]">
                        <span title="Linear">🟣</span>
                        <span title="GitHub">🐙</span>
                        <span title="Jira">🔷</span>
                      </div>
                    </div>
                    <button
                      onClick={() => handleOpenCreateIssueModal('linear')}
                      className="text-[11px] font-bold text-purple-700 bg-purple-100/70 hover:bg-purple-200 px-2 py-0.5 rounded shadow-2xs cursor-pointer transition-colors"
                    >
                      + 1-Click Issue
                    </button>
                  </div>

                  {linkedIssues.length > 0 ? (
                    <div className="space-y-1.5 pt-1">
                      {linkedIssues.map((issue) => (
                        <div
                          key={issue.id}
                          className="flex items-center justify-between rounded-lg border border-slate-200 bg-white p-2 text-xs shadow-2xs"
                        >
                          <div className="flex items-center gap-2 truncate">
                            <span>
                              {issue.provider === 'linear' ? '🟣' : (issue.provider === 'github' ? '🐙' : '🔷')}
                            </span>
                            <a
                              href={issue.issue_url}
                              target="_blank"
                              rel="noreferrer"
                              className="font-bold font-mono text-purple-700 hover:underline shrink-0"
                            >
                              {issue.issue_key}
                            </a>
                            <span className="text-slate-600 truncate text-[11px]" title={issue.title}>
                              {issue.title}
                            </span>
                          </div>
                          <div className="flex items-center gap-1.5 shrink-0 ml-2">
                            <span className="rounded bg-slate-100 px-1.5 py-0.2 text-[9px] font-bold text-slate-700">
                              {issue.status}
                            </span>
                            <button
                              onClick={() => handleSyncIssueStatus(issue.id)}
                              title="Sync status"
                              className="text-slate-400 hover:text-slate-700 text-xs font-bold cursor-pointer"
                            >
                              ↻
                            </button>
                            <button
                              onClick={() => handleUnlinkIssue(issue.id)}
                              title="Unlink issue"
                              className="text-slate-400 hover:text-rose-600 text-xs font-bold cursor-pointer"
                            >
                              ×
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-[11px] text-slate-500">
                      No engineering bugs or feature requests linked. Push to Linear, GitHub, or Jira with 1 click.
                    </p>
                  )}
                </div>

                {/* Gap 3: Slack Conversation Sync Card */}
                <div className="rounded-xl border border-emerald-100 bg-gradient-to-br from-emerald-50/60 via-white to-slate-50 p-3 shadow-2xs space-y-2">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1.5">
                      <span className="text-sm">💬</span>
                      <span className="font-bold text-xs text-slate-900">Slack Team Sync</span>
                      {slackSync?.is_synced && (
                        <span className="rounded-full bg-emerald-100 text-emerald-800 font-bold px-1.5 py-0.2 text-[9px]">
                          Live Synced
                        </span>
                      )}
                    </div>
                    {slackSync?.thread_url ? (
                      <a
                        href={slackSync.thread_url}
                        target="_blank"
                        rel="noreferrer"
                        className="text-[11px] font-semibold text-emerald-700 hover:underline flex items-center gap-0.5"
                      >
                        <span>Open Thread</span>
                        <span>↗</span>
                      </a>
                    ) : (
                      <button
                        onClick={handleSyncToSlack}
                        disabled={slackSyncing}
                        className="text-[11px] font-bold text-emerald-700 bg-emerald-100/70 hover:bg-emerald-200 px-2 py-0.5 rounded shadow-2xs cursor-pointer transition-colors"
                      >
                        {slackSyncing ? 'Syncing...' : 'Sync to Slack'}
                      </button>
                    )}
                  </div>
                  <div className="flex items-center justify-between text-[11px] text-slate-500">
                    <span>Channel: {slackSync?.slack_channel_name || '#customer-escalations'}</span>
                    <button
                      onClick={handleSyncToSlack}
                      disabled={slackSyncing}
                      className="text-emerald-600 hover:underline font-semibold"
                    >
                      {slackSync ? 'Post Note' : 'Bridge Thread'}
                    </button>
                  </div>
                </div>

                {/* Stripe Billing Quick Overview Card */}
                <div className="rounded-xl border border-indigo-100 bg-gradient-to-br from-indigo-50/70 via-white to-slate-50 p-3 shadow-2xs">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-1.5">
                      <span className="text-sm">💳</span>
                      <span className="font-bold text-xs text-slate-900">Stripe Billing</span>
                      {stripeBilling?.subscription?.status === 'active' && (
                        <span className="rounded-full bg-emerald-100 text-emerald-700 font-bold px-1.5 py-0.5 text-[9px]">
                          Active
                        </span>
                      )}
                    </div>
                    <button
                      onClick={() => setContactSidebarTab('billing')}
                      className="text-[11px] font-semibold text-indigo-600 hover:text-indigo-800 hover:underline flex items-center gap-0.5"
                    >
                      Invoices & Actions →
                    </button>
                  </div>

                  {stripeLoading ? (
                    <div className="py-2 text-center text-xs text-slate-400 animate-pulse">
                      Loading customer billing...
                    </div>
                  ) : stripeBilling ? (
                    <div className="space-y-1.5 text-xs">
                      <div className="flex items-center justify-between">
                        <span className="text-slate-500 font-medium">Plan</span>
                        <span className="font-semibold text-slate-900">{stripeBilling.subscription?.plan_name || 'Enterprise Scale'}</span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-500 font-medium">MRR / Spend</span>
                        <div className="flex items-center gap-1">
                          <span className="font-bold text-emerald-600">{stripeBilling.mrr_formatted}</span>
                          <span className="text-[10px] text-slate-400">({stripeBilling.ltv_formatted} LTV)</span>
                        </div>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-500 font-medium">Payment Method</span>
                        <span className="font-medium text-slate-700 flex items-center gap-1">
                          <span className="uppercase text-[9px] bg-slate-100 px-1 rounded font-bold text-slate-600">{stripeBilling.payment_method?.brand || 'VISA'}</span>
                          •••• {stripeBilling.payment_method?.last4 || '4242'}
                        </span>
                      </div>
                      <div className="pt-2 border-t border-indigo-100/60 flex items-center justify-between text-[11px]">
                        <span className="text-slate-500">Renews</span>
                        <span className="text-slate-700 font-medium">
                          {stripeBilling.subscription?.current_period_end ? new Date(stripeBilling.subscription.current_period_end).toLocaleDateString() : 'Next Month'}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <div className="py-1 text-xs text-slate-400">
                      No Stripe subscription data available.
                    </div>
                  )}
                </div>

                {/* Contact Memory & Preferences ("Remembers") */}
                <div className="border-t border-[#eaecf0] pt-4">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-1.5">
                      <span className="font-semibold text-slate-900 text-xs">🧠 Memory & Preferences</span>
                      <span className="rounded bg-indigo-50 text-indigo-700 font-bold px-1.5 py-0.2 text-[10px]">
                        Remembers
                      </span>
                    </div>
                    <button
                      onClick={() => {
                        const fact = prompt(
                          'Enter preference or durable fact to remember (e.g. Their team prefers email over calls):'
                        )
                        if (fact) handleAddMemory(fact)
                      }}
                      className="text-blue-600 font-bold text-xs hover:underline"
                    >
                      + Add
                    </button>
                  </div>
                  <p className="text-[11px] text-slate-500 mb-2.5">
                    Durable facts learned by AI. Injected into every reply prompt so the agent respects them.
                  </p>

                  <div className="space-y-2">
                    {activeConv.memories && activeConv.memories.length > 0 ? (
                      activeConv.memories.map((m) => (
                        <div
                          key={m.id}
                          className="rounded-lg border border-indigo-100 bg-indigo-50/50 p-2.5 text-xs text-indigo-950 shadow-2xs"
                        >
                          <div className="flex items-center justify-between text-[10px] text-indigo-600 font-bold mb-1">
                            <span>Noted · {m.notedAt}</span>
                            <div className="flex items-center gap-1.5">
                              <span className="uppercase text-[9px] bg-white px-1.5 py-0.5 rounded border border-indigo-200">
                                {m.category.replace('_', ' ')}
                              </span>
                              <button
                                onClick={() => handleRemoveMemory(m.id)}
                                className="text-indigo-400 hover:text-rose-600 font-bold ml-1 text-xs"
                                title="Delete memory"
                              >
                                ×
                              </button>
                            </div>
                          </div>
                          <p className="font-medium text-slate-800 leading-snug">{m.fact}</p>
                        </div>
                      ))
                    ) : (
                      <div className="rounded-lg bg-slate-50 p-3 text-center text-xs text-slate-400 border border-dashed border-slate-200">
                        No memories noted yet. AI automatically listens to conversations and captures preferences.
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}

            {/* SUBTAB 2: INTERNAL NOTES */}
            {contactSidebarTab === 'notes' && (
              <div className="space-y-4">
                <div className="rounded-lg border border-slate-200 bg-white p-3 shadow-2xs">
                  <label className="block text-[11px] font-bold text-slate-700 mb-1">Add Contact Profile Note</label>
                  <textarea
                    rows={3}
                    value={newNoteText}
                    onChange={(e) => setNewNoteText(e.target.value)}
                    placeholder="Record durable internal context about this person..."
                    className="w-full resize-none text-xs text-slate-800 placeholder-slate-400 p-2 border border-slate-200 rounded-md focus:border-blue-500 focus:outline-hidden"
                  />
                  <div className="flex justify-end mt-2">
                    <button
                      onClick={handleAddContactNote}
                      disabled={!newNoteText.trim()}
                      className="rounded-md bg-blue-600 px-3 py-1 text-xs font-semibold text-white hover:bg-blue-700 disabled:opacity-40 shadow-xs"
                    >
                      + Save Note
                    </button>
                  </div>
                </div>

                <div className="space-y-2.5">
                  {(contactNotes[activeConv.id] || []).map((n) => (
                    <div key={n.id} className="rounded-lg border border-slate-200 bg-slate-50 p-3 text-xs shadow-2xs">
                      <div className="flex items-center justify-between text-[11px] font-semibold text-slate-700 mb-1">
                        <span>👤 {n.author}</span>
                        <span className="font-normal text-slate-400">{n.time}</span>
                      </div>
                      <p className="text-slate-800 leading-relaxed">{n.body}</p>
                    </div>
                  ))}
                  {!(contactNotes[activeConv.id] || []).length && (
                    <div className="p-6 text-center text-xs text-slate-400 border border-dashed rounded-lg bg-slate-50/50">
                      No internal notes recorded for this contact yet.
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* SUBTAB 3: CUSTOM ATTRIBUTES */}
            {contactSidebarTab === 'attributes' && (
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-xs text-slate-900">CRM Custom Fields</span>
                  <button
                    onClick={() => {
                      const key = prompt('Attribute name (e.g. loyalty_tier, billing_cycle):')
                      if (!key) return
                      const val = prompt(`Value for "${key}":`)
                      if (val === null) return
                      setContactAttributes((prev) => ({
                        ...prev,
                        [activeConv.id]: { ...(prev[activeConv.id] || {}), [key.toLowerCase()]: val },
                      }))
                    }}
                    className="text-xs font-bold text-blue-600 hover:underline"
                  >
                    + Add Field
                  </button>
                </div>

                <div className="divide-y divide-slate-100 rounded-lg border border-slate-200 bg-white shadow-2xs overflow-hidden">
                  {Object.entries(contactAttributes[activeConv.id] || {}).map(([k, v]) => (
                    <div key={k} className="flex items-center justify-between px-3 py-2 text-xs">
                      <span className="font-medium text-slate-500 capitalize">{k.replace('_', ' ')}</span>
                      <span className="font-semibold text-slate-900">{v}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* SUBTAB 4: OMNICHANNEL HISTORY */}
            {contactSidebarTab === 'history' && (
              <div className="space-y-3">
                <div className="font-bold text-xs text-slate-900">Omnichannel Touchpoints</div>
                <div className="relative pl-4 space-y-4 border-l-2 border-slate-200 text-xs">
                  <div className="relative">
                    <span className="absolute -left-[21px] top-0.5 flex h-2.5 w-2.5 rounded-full bg-emerald-500 ring-2 ring-white" />
                    <p className="font-semibold text-slate-900">WhatsApp Campaign Response</p>
                    <p className="text-slate-500 text-[11px]">Today · Replied to Spring Sale early access</p>
                  </div>
                  <div className="relative">
                    <span className="absolute -left-[21px] top-0.5 flex h-2.5 w-2.5 rounded-full bg-blue-500 ring-2 ring-white" />
                    <p className="font-semibold text-slate-900">WhatsApp Broadcast Delivered</p>
                    <p className="text-slate-500 text-[11px]">Today, 10:00 AM · Message read receipt confirmed</p>
                  </div>
                  <div className="relative">
                    <span className="absolute -left-[21px] top-0.5 flex h-2.5 w-2.5 rounded-full bg-purple-500 ring-2 ring-white" />
                    <p className="font-semibold text-slate-900">Website Live Chat Session</p>
                    <p className="text-slate-500 text-[11px]">Feb 26, 4:00 PM · Inquired about catalog items</p>
                  </div>
                </div>
              </div>
            )}

            {/* SUBTAB 5: STRIPE BILLING & PAYMENTS */}
            {contactSidebarTab === 'billing' && (
              <div className="space-y-4">
                {refundFeedback && (
                  <div
                    className={`rounded-lg p-2.5 text-xs font-medium flex items-center justify-between ${
                      refundFeedback.includes('Error')
                        ? 'bg-rose-50 text-rose-800 border border-rose-200'
                        : 'bg-emerald-50 text-emerald-800 border border-emerald-200'
                    }`}
                  >
                    <span>{refundFeedback}</span>
                    <button
                      onClick={() => setRefundFeedback(null)}
                      className="text-xs font-bold hover:opacity-75"
                    >
                      ✕
                    </button>
                  </div>
                )}

                {stripeLoading ? (
                  <div className="p-8 text-center text-xs text-slate-400 space-y-2">
                    <div className="inline-block h-6 w-6 animate-spin rounded-full border-2 border-indigo-600 border-t-transparent" />
                    <p>Loading Stripe customer billing...</p>
                  </div>
                ) : (
                  <>
                    {/* Subscription & Customer Profile Card */}
                    <div className="rounded-xl border border-indigo-100 bg-gradient-to-br from-indigo-50/60 via-white to-slate-50 p-4 shadow-2xs space-y-3">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-1.5">
                          <span className="text-base">💳</span>
                          <span className="font-bold text-xs text-slate-900">
                            {stripeBilling?.subscription?.plan_name || 'Enterprise Scale'}
                          </span>
                        </div>
                        <span
                          className={`rounded-full px-2 py-0.5 text-[10px] font-bold ${
                            stripeBilling?.subscription?.status === 'active'
                              ? 'bg-emerald-100 text-emerald-800 border border-emerald-200'
                              : 'bg-amber-100 text-amber-800 border border-amber-200'
                          }`}
                        >
                          {stripeBilling?.subscription?.status?.toUpperCase() || 'ACTIVE'}
                        </span>
                      </div>

                      <div className="grid grid-cols-2 gap-2 pt-1 text-xs">
                        <div className="rounded-lg bg-white/80 p-2.5 border border-slate-100">
                          <div className="text-[10px] text-slate-500 font-medium">Monthly MRR</div>
                          <div className="text-sm font-bold text-emerald-600 mt-0.5">
                            {stripeBilling?.mrr_formatted || '$299.00/mo'}
                          </div>
                        </div>
                        <div className="rounded-lg bg-white/80 p-2.5 border border-slate-100">
                          <div className="text-[10px] text-slate-500 font-medium">Lifetime Spend</div>
                          <div className="text-sm font-bold text-slate-800 mt-0.5">
                            {stripeBilling?.ltv_formatted || '$2,392.00'}
                          </div>
                        </div>
                      </div>

                      <div className="space-y-1.5 pt-2 border-t border-slate-100 text-xs">
                        <div className="flex items-center justify-between">
                          <span className="text-slate-500">Payment Card</span>
                          <span className="font-medium text-slate-700 flex items-center gap-1">
                            <span className="uppercase text-[9px] bg-slate-100 px-1 rounded font-bold text-slate-600">
                              {stripeBilling?.payment_method?.brand || 'VISA'}
                            </span>
                            •••• {stripeBilling?.payment_method?.last4 || '4242'}
                            <span className="text-[10px] text-slate-400">
                              (Exp {stripeBilling?.payment_method?.exp_month || 12}/
                              {stripeBilling?.payment_method?.exp_year || 2028})
                            </span>
                          </span>
                        </div>

                        <div className="flex items-center justify-between">
                          <span className="text-slate-500">Next Renewal</span>
                          <span className="font-medium text-slate-800">
                            {stripeBilling?.subscription?.current_period_end
                              ? new Date(stripeBilling.subscription.current_period_end).toLocaleDateString()
                              : 'In 24 days'}
                          </span>
                        </div>

                        <div className="flex items-center justify-between pt-1 text-[11px]">
                          <span className="text-slate-400">Customer ID</span>
                          <button
                            onClick={() => {
                              if (stripeBilling?.customer_id) {
                                navigator.clipboard.writeText(stripeBilling.customer_id)
                                alert(`Copied Stripe ID: ${stripeBilling.customer_id}`)
                              }
                            }}
                            className="font-mono text-indigo-600 hover:underline flex items-center gap-1"
                            title="Click to copy Stripe Customer ID"
                          >
                            <span>{stripeBilling?.customer_id || 'cus_sandbox_001'}</span>
                            <span className="text-[10px]">📋</span>
                          </button>
                        </div>
                      </div>

                      {/* Action Buttons */}
                      <div className="pt-2 flex items-center gap-2">
                        <button
                          onClick={handleCreatePortalLink}
                          className="flex-1 rounded-lg bg-indigo-50 border border-indigo-200 px-2.5 py-1.5 text-xs font-semibold text-indigo-700 hover:bg-indigo-100 transition-colors flex items-center justify-center gap-1 shadow-2xs cursor-pointer"
                        >
                          <span>🔗</span>
                          <span>{portalCopied ? 'Link Copied!' : 'Billing Portal'}</span>
                        </button>
                        <button
                          onClick={() => {
                            setSelectedInvoiceForRefund(stripeInvoices[0] || null)
                            setRefundAmountInput('')
                            setRefundModalOpen(true)
                          }}
                          className="flex-1 rounded-lg bg-rose-50 border border-rose-200 px-2.5 py-1.5 text-xs font-semibold text-rose-700 hover:bg-rose-100 transition-colors flex items-center justify-center gap-1 shadow-2xs cursor-pointer"
                        >
                          <span>⚡</span>
                          <span>Issue Refund</span>
                        </button>
                      </div>
                    </div>

                    {/* Invoices List */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="font-bold text-xs text-slate-900 flex items-center gap-1">
                          <span>🧾</span>
                          <span>Invoices & Receipts</span>
                          <span className="text-[10px] text-slate-400 font-normal">
                            ({stripeInvoices.length})
                          </span>
                        </span>
                      </div>

                      <div className="space-y-2">
                        {stripeInvoices.map((inv) => (
                          <div
                            key={inv.id}
                            className="rounded-lg border border-slate-200 bg-white p-3 text-xs shadow-2xs hover:border-slate-300 transition-colors"
                          >
                            <div className="flex items-center justify-between mb-1.5">
                              <span className="font-bold text-slate-900">{inv.number}</span>
                              <span className="rounded bg-emerald-50 text-emerald-700 border border-emerald-200 px-1.5 py-0.2 text-[10px] font-bold">
                                {inv.status.toUpperCase()}
                              </span>
                            </div>

                            <div className="flex items-center justify-between text-slate-500 text-[11px] mb-2">
                              <span>{new Date(inv.created_at).toLocaleDateString()}</span>
                              <span className="font-bold text-slate-900 text-xs">
                                {inv.amount_paid_formatted}
                              </span>
                            </div>

                            <div className="flex items-center justify-between pt-2 border-t border-slate-100">
                              <div className="flex items-center gap-2">
                                <a
                                  href={inv.hosted_invoice_url}
                                  target="_blank"
                                  rel="noreferrer"
                                  className="text-[11px] font-semibold text-indigo-600 hover:underline flex items-center gap-0.5"
                                >
                                  <span>Receipt</span>
                                  <span>↗</span>
                                </a>
                                <a
                                  href={inv.invoice_pdf_url}
                                  target="_blank"
                                  rel="noreferrer"
                                  className="text-[11px] font-semibold text-slate-500 hover:text-slate-800 flex items-center gap-0.5"
                                >
                                  <span>PDF</span>
                                  <span>⬇</span>
                                </a>
                              </div>

                              <button
                                onClick={() => {
                                  setSelectedInvoiceForRefund(inv)
                                  setRefundAmountInput('')
                                  setRefundModalOpen(true)
                                }}
                                className="text-[10px] font-bold text-rose-600 hover:text-rose-800 hover:underline cursor-pointer"
                              >
                                Refund
                              </button>
                            </div>
                          </div>
                        ))}

                        {stripeInvoices.length === 0 && (
                          <div className="rounded-lg border border-dashed border-slate-200 p-4 text-center text-xs text-slate-400">
                            No billing invoices found.
                          </div>
                        )}
                      </div>
                    </div>

                    {/* Billing Audit Trail */}
                    {stripeLogs.length > 0 && (
                      <div className="space-y-2 pt-2 border-t border-slate-100">
                        <span className="font-bold text-xs text-slate-900">
                          Billing Action Audit Trail
                        </span>
                        <div className="space-y-1.5">
                          {stripeLogs.slice(0, 5).map((log) => (
                            <div
                              key={log.id}
                              className="rounded-md border border-slate-100 bg-slate-50 p-2 text-[11px]"
                            >
                              <div className="flex items-center justify-between font-semibold text-slate-800">
                                <span className="capitalize">{log.action_type.replace('_', ' ')}</span>
                                <span className="text-emerald-600 font-bold">
                                  {log.amount_cents > 0 ? log.amount_formatted : 'Success'}
                                </span>
                              </div>
                              <div className="flex items-center justify-between text-slate-400 text-[10px] mt-0.5">
                                <span>By {log.actor_email}</span>
                                <span>{new Date(log.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </>
                )}
              </div>
            )}

            {/* Accordion: Conversation Actions */}
            <div className="border-t border-[#eaecf0] pt-4">
              <div
                onClick={() => setShowActionsAccordion((v) => !v)}
                className="flex items-center justify-between cursor-pointer"
              >
                <span className="font-semibold text-slate-900 text-xs">Conversation Actions</span>
                <span className="text-slate-400 font-bold">{showActionsAccordion ? '−' : '+'}</span>
              </div>

              {showActionsAccordion && (
                <div className="mt-3.5 space-y-3.5 text-xs">
                  {/* Captain AI Triage Quick Action */}
                  <div className="rounded-lg border border-purple-200 bg-purple-50/70 p-3 space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="font-bold text-xs text-purple-900 flex items-center gap-1.5">
                        <span>🎯</span> Captain AI Triage
                      </span>
                      <button
                        onClick={() => handleRunTriage(true)}
                        disabled={isTriaging}
                        className="text-[11px] font-bold text-purple-700 bg-white border border-purple-300 px-2 py-0.5 rounded shadow-2xs hover:bg-purple-100 disabled:opacity-50 transition-colors"
                      >
                        {isTriaging ? 'Analyzing...' : 'Run Triage'}
                      </button>
                    </div>
                    {triageResult ? (
                      <div className="space-y-1">
                        <p className="text-[11px] text-purple-950 font-medium">
                          {triageResult.reasoning}
                        </p>
                        <div className="flex flex-wrap gap-1 pt-1">
                          {triageResult.suggestedTags.map((tag) => (
                            <span key={tag} className="px-1.5 py-0.2 rounded bg-purple-200 text-purple-800 text-[10px] font-semibold">
                              #{tag}
                            </span>
                          ))}
                        </div>
                      </div>
                    ) : (
                      <p className="text-[11px] text-purple-700 leading-snug">
                        Inspects customer intent, automatically determines priority urgency, tags the thread, and routes to the right team.
                      </p>
                    )}
                  </div>

                  {/* Assigned Agent Selector */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="text-slate-500 font-medium">Assigned Agent</label>
                      <button
                        type="button"
                        onClick={() => handleAutoAssign('capacity_based')}
                        className="text-[10px] text-teal-600 hover:text-teal-800 font-semibold cursor-pointer hover:underline"
                        title="Auto-route to least-loaded agent"
                      >
                        ⚡ Auto-Assign
                      </button>
                    </div>
                    <select
                      value={activeConv.assignedAgent}
                      onChange={(e) => handleAgentChange(e.target.value)}
                      className="w-full rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 text-xs font-medium text-slate-800 shadow-xs focus:border-blue-500 focus:outline-hidden"
                    >
                      <option value="Captain (AI Agent)">✨ Captain (AI Agent)</option>
                      {routingAgents.length > 0 ? (
                        routingAgents.map((ag) => (
                          <option key={ag.id} value={ag.name}>
                            {ag.availabilityStatus === 'online' ? '🟢' : ag.availabilityStatus === 'busy' ? '🟡' : '⚪'}{' '}
                            {ag.name} ({ag.activeConversationsCount}/{ag.maxCapacity} active)
                          </option>
                        ))
                      ) : (
                        <>
                          <option value="Riley Brooks">🟢 Riley Brooks (1/5 active)</option>
                          <option value="Jamie Cole">🟢 Jamie Cole (0/5 active)</option>
                          <option value="Alex Rivera">🟡 Alex Rivera (3/3 busy)</option>
                          <option value="Morgan Vance">⚪ Morgan Vance (offline)</option>
                        </>
                      )}
                      <option value="">⏳ Unassigned</option>
                    </select>
                  </div>

                  {/* Assigned Team Selector */}
                  <div>
                    <label className="block text-slate-500 font-medium mb-1">Assigned Team</label>
                    <select
                      value={activeConv.team}
                      onChange={(e) => handleTeamChange(e.target.value)}
                      className="w-full rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 text-xs font-medium text-slate-800 shadow-xs focus:border-blue-500 focus:outline-hidden"
                    >
                      <option value="customer support">🤝 Customer Support</option>
                      <option value="sales & outreach">💼 Sales & Outreach</option>
                      <option value="technical support">🔧 Technical Support</option>
                      <option value="developer support">💻 Developer Support</option>
                      <option value="vip / escalation">👑 VIP / Escalation</option>
                    </select>
                  </div>

                  {/* Priority Urgency Selector */}
                  <div>
                    <label className="block text-slate-500 font-medium mb-1">Priority</label>
                    <select
                      value={activeConv.priority}
                      onChange={(e) => handlePriorityChange(e.target.value as any)}
                      className="w-full rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 text-xs font-semibold text-slate-800 shadow-xs focus:border-blue-500 focus:outline-hidden"
                    >
                      <option value="Urgent">🔴 Urgent</option>
                      <option value="High">🟠 High</option>
                      <option value="Medium">🟡 Medium</option>
                      <option value="Low">⚪ Low</option>
                    </select>
                  </div>

                  {/* Conversation Labels */}
                  <div>
                    <label className="block text-slate-500 font-medium mb-1.5">Conversation Labels</label>
                    <div className="flex flex-wrap items-center gap-1.5">
                      <button
                        onClick={handleAddLabel}
                        className="inline-flex items-center gap-1 rounded bg-[#f2f4f7] px-2 py-1 font-semibold text-blue-600 hover:bg-blue-50"
                      >
                        <span>+ Add Labels</span>
                      </button>
                      {activeConv.tags.map((t) => (
                        <span
                          key={t.name}
                          className="inline-flex items-center gap-1 rounded bg-amber-50 px-2 py-1 text-amber-800 font-medium border border-amber-200"
                        >
                          <span className="h-1.5 w-1.5 rounded-full bg-amber-500" />
                          <span>{t.name}</span>
                          <button
                            onClick={() => handleRemoveLabel(t.name)}
                            className="ml-0.5 text-amber-600 hover:text-amber-900 font-bold"
                          >
                            ×
                          </button>
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Accordion: Macros */}
            <div className="border-t border-[#eaecf0] pt-4">
              <div
                onClick={() => setShowMacrosAccordion((v) => !v)}
                className="flex items-center justify-between cursor-pointer group"
              >
                <div className="flex items-center gap-1.5">
                  <span className="font-semibold text-slate-900 text-xs">Macros</span>
                  <span className="text-[10px] bg-slate-100 text-slate-600 px-1.5 py-0.5 rounded-full font-medium">
                    {macros.length}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation()
                      setShowCreateMacroModal(true)
                    }}
                    className="text-[10px] text-brand-600 hover:text-brand-700 font-semibold hover:underline"
                  >
                    + Create
                  </button>
                  <span className="text-slate-400 font-bold group-hover:text-slate-600">
                    {showMacrosAccordion ? '−' : '+'}
                  </span>
                </div>
              </div>

              {showMacrosAccordion && (
                <div className="mt-2.5 space-y-2 text-xs">
                  {macros.map((macro) => (
                    <div
                      key={macro.id}
                      className="p-2.5 rounded-lg border border-slate-200/80 bg-white hover:border-brand-300 hover:shadow-xs transition-all space-y-1.5"
                    >
                      <div className="flex items-start justify-between gap-1.5">
                        <span className="font-semibold text-slate-900 leading-snug">
                          ⚡ {macro.name}
                        </span>
                      </div>
                      {macro.description && (
                        <p className="text-[11px] text-slate-500 line-clamp-2 leading-relaxed">
                          {macro.description}
                        </p>
                      )}
                      <div className="flex flex-wrap gap-1 pt-0.5">
                        {macro.actions.map((act, idx) => (
                          <span
                            key={idx}
                            className="text-[9px] font-medium px-1.5 py-0.5 rounded bg-slate-100 text-slate-600 uppercase tracking-wider"
                          >
                            {act.type.replace(/_/g, ' ')}
                          </span>
                        ))}
                      </div>
                      <div className="pt-1 flex justify-end">
                        <button
                          type="button"
                          disabled={macroLoading}
                          onClick={() => handleExecuteMacro(macro)}
                          className="px-2.5 py-1 text-[11px] font-semibold rounded bg-slate-900 hover:bg-slate-800 text-white shadow-xs transition-colors disabled:opacity-50 flex items-center gap-1 cursor-pointer"
                        >
                          {activeExecutingMacroId === macro.id ? (
                            <>
                              <svg className="animate-spin h-3 w-3 text-white" viewBox="0 0 24 24" fill="none">
                                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
                              </svg>
                              <span>Running...</span>
                            </>
                          ) : (
                            <>
                              <span>⚡ Run Macro</span>
                            </>
                          )}
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Accordion: SLA Targets & Deadlines */}
            <div className="border-t border-[#eaecf0] pt-4">
              <div
                onClick={() => setShowSLAAccordion((v) => !v)}
                className="flex items-center justify-between cursor-pointer group"
              >
                <div className="flex items-center gap-1.5">
                  <span className="font-semibold text-slate-900 text-xs">SLA &amp; Deadlines</span>
                  {activeConv.slaData?.isBreached ? (
                    <span className="text-[10px] bg-rose-100 text-rose-700 px-1.5 py-0.5 rounded font-bold">
                      Breached
                    </span>
                  ) : activeConv.slaData?.nearingBreach ? (
                    <span className="text-[10px] bg-amber-100 text-amber-800 px-1.5 py-0.5 rounded font-bold">
                      At Risk
                    </span>
                  ) : (
                    <span className="text-[10px] bg-emerald-100 text-emerald-800 px-1.5 py-0.5 rounded font-medium">
                      Healthy
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation()
                      openSLAMonitorModal()
                    }}
                    className="text-[10px] text-brand-600 hover:text-brand-700 font-semibold hover:underline"
                  >
                    Monitor
                  </button>
                  <span className="text-slate-400 font-bold group-hover:text-slate-600">
                    {showSLAAccordion ? '−' : '+'}
                  </span>
                </div>
              </div>

              {showSLAAccordion && (
                <div className="mt-2.5 space-y-2.5 text-xs">
                  {/* Active Policy Header */}
                  <div className="p-2 rounded bg-slate-50 border border-slate-200/80 flex items-center justify-between">
                    <div>
                      <p className="text-[10px] text-slate-400 uppercase font-semibold">Matched SLA Policy</p>
                      <p className="font-bold text-slate-900 text-xs truncate">
                        {activeConv.slaData?.policyName ||
                          (activeConv.priority === 'Urgent'
                            ? 'Critical VIP Escalation SLA'
                            : activeConv.priority === 'High'
                            ? 'High Priority SLA'
                            : 'Standard Customer Support SLA')}
                      </p>
                    </div>
                    <span className="text-xs">🎯</span>
                  </div>

                  {/* First Response Target Progress */}
                  <div className="space-y-1">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="text-slate-600 font-medium">First Response:</span>
                      <span
                        className={`font-semibold ${
                          activeConv.slaData?.firstResponseStatus === 'fulfilled'
                            ? 'text-emerald-700'
                            : activeConv.slaData?.firstResponseBreached
                            ? 'text-rose-700'
                            : 'text-slate-800'
                        }`}
                      >
                        {activeConv.slaData?.firstResponseStatus === 'fulfilled'
                          ? '✓ Met on time'
                          : activeConv.slaData?.firstResponseBreached
                          ? '⚠️ Breached'
                          : activeConv.sla || 'Active (within target)'}
                      </span>
                    </div>
                    <div className="w-full bg-slate-100 h-1.5 rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all ${
                          activeConv.slaData?.firstResponseStatus === 'fulfilled'
                            ? 'bg-emerald-500 w-full'
                            : activeConv.slaData?.firstResponseBreached
                            ? 'bg-rose-500 w-full'
                            : 'bg-blue-500 w-2/3'
                        }`}
                      />
                    </div>
                  </div>

                  {/* Resolution Target Progress */}
                  <div className="space-y-1">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="text-slate-600 font-medium">Ticket Resolution:</span>
                      <span
                        className={`font-semibold ${
                          activeConv.status === 'Resolved' || activeConv.slaData?.resolutionStatus === 'fulfilled'
                            ? 'text-emerald-700'
                            : activeConv.slaData?.resolutionBreached
                            ? 'text-rose-700'
                            : 'text-slate-800'
                        }`}
                      >
                        {activeConv.status === 'Resolved' || activeConv.slaData?.resolutionStatus === 'fulfilled'
                          ? '✓ Met'
                          : activeConv.slaData?.resolutionBreached
                          ? '⚠️ Breached'
                          : 'Target 24h'}
                      </span>
                    </div>
                    <div className="w-full bg-slate-100 h-1.5 rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all ${
                          activeConv.status === 'Resolved' || activeConv.slaData?.resolutionStatus === 'fulfilled'
                            ? 'bg-emerald-500 w-full'
                            : activeConv.slaData?.resolutionBreached
                            ? 'bg-rose-500 w-full'
                            : 'bg-indigo-500 w-1/3'
                        }`}
                      />
                    </div>
                  </div>

                  {/* Evaluate SLA Button */}
                  <div className="pt-1 flex items-center justify-between gap-2">
                    <button
                      type="button"
                      disabled={isEvaluatingSLA}
                      onClick={() => handleEvaluateSLA()}
                      className="flex-1 py-1 px-2 rounded border border-slate-200 bg-white hover:bg-slate-50 text-[11px] font-semibold text-slate-700 shadow-2xs flex items-center justify-center gap-1 cursor-pointer disabled:opacity-50"
                    >
                      {isEvaluatingSLA ? 'Evaluating...' : '⏱️ Check Deadlines'}
                    </button>
                    <button
                      type="button"
                      onClick={openSLAMonitorModal}
                      className="py-1 px-2 rounded border border-slate-200 bg-white hover:bg-slate-50 text-[11px] font-semibold text-slate-700 shadow-2xs flex items-center gap-1 cursor-pointer"
                      title="View aggregated SLA metrics & breached tickets"
                    >
                      📊
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        </aside>
      )}

      {/* ========================================================================= */}
      {/* CAPTAIN AI THREAD SUMMARY MODAL                                           */}
      {/* ========================================================================= */}
      {showSummaryModal && captainSummary && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-lg w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-purple-100 text-purple-700 flex items-center justify-center font-bold text-sm">
                  ✨
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Captain AI Thread Summary</h3>
                  <p className="text-xs text-slate-500">Executive takeaways for {activeConv.name}</p>
                </div>
              </div>
              <button
                onClick={() => setShowSummaryModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1"
              >
                ✕
              </button>
            </div>

            {/* Sentiment & Topics Row */}
            <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
              <div className="flex items-center gap-1.5">
                <span className="text-slate-500 font-medium">Sentiment:</span>
                <span
                  className={`px-2 py-0.5 rounded font-bold uppercase text-[10px] ${
                    captainSummary.sentiment === 'positive'
                      ? 'bg-emerald-100 text-emerald-800'
                      : captainSummary.sentiment === 'urgent'
                      ? 'bg-rose-100 text-rose-800'
                      : captainSummary.sentiment === 'negative'
                      ? 'bg-amber-100 text-amber-800'
                      : 'bg-slate-100 text-slate-700'
                  }`}
                >
                  {captainSummary.sentiment}
                </span>
              </div>
              <div className="flex flex-wrap items-center gap-1">
                {captainSummary.keyTopics.map((topic, i) => (
                  <span key={i} className="px-2 py-0.5 rounded bg-purple-50 text-purple-700 text-[10px] font-semibold border border-purple-200">
                    {topic}
                  </span>
                ))}
              </div>
            </div>

            {/* Bullet Points */}
            <div className="space-y-2 bg-slate-50 p-3.5 rounded-xl border border-slate-200 text-xs text-slate-800">
              <span className="font-semibold text-slate-900 text-xs block">Key Discussion Highlights:</span>
              <ul className="space-y-1.5 list-disc pl-4 text-slate-700">
                {captainSummary.bulletPoints.map((bullet, idx) => (
                  <li key={idx} className="leading-relaxed">
                    {bullet}
                  </li>
                ))}
              </ul>
            </div>

            {/* Action Buttons */}
            <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
              <button
                onClick={() => {
                  navigator.clipboard.writeText(captainSummary.summary)
                  alert('Thread summary copied to clipboard!')
                }}
                className="px-3 py-1.5 rounded-lg border border-slate-200 bg-white text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs"
              >
                Copy Summary
              </button>
              <button
                onClick={() => {
                  const noteObj = {
                    id: `cn-${Date.now()}`,
                    author: 'Captain AI (Summary)',
                    body: captainSummary.summary,
                    time: 'Just now',
                  }
                  setContactNotes((prev) => ({
                    ...prev,
                    [activeConv.id]: [noteObj, ...(prev[activeConv.id] || [])],
                  }))
                  api.addContactNote(activeConv.id, captainSummary.summary, 'Captain AI (Summary)').catch(() => {})
                  setShowSummaryModal(false)
                  alert('Summary saved to contact private notes!')
                }}
                className="px-3.5 py-1.5 rounded-lg bg-purple-700 text-xs font-bold text-white hover:bg-purple-800 shadow-xs"
              >
                Save as Private Note
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* CAPTAIN MONITORS & TOPIC INTELLIGENCE DRAWER                              */}
      {/* ========================================================================= */}
      {showMonitorsDrawer && (
        <div className="fixed inset-0 z-50 flex items-center justify-end bg-slate-900/40 backdrop-blur-xs">
          <div className="bg-white h-full w-full max-w-xl shadow-2xl p-6 overflow-y-auto space-y-6 flex flex-col animate-in slide-in-from-right duration-200">
            {/* Header */}
            <div className="flex items-start justify-between border-b border-slate-200 pb-4">
              <div>
                <div className="flex items-center gap-2">
                  <span className="w-8 h-8 rounded-lg bg-purple-100 text-purple-700 flex items-center justify-center font-bold text-sm">
                    ✨
                  </span>
                  <h2 className="text-lg font-bold text-slate-900">Captain Monitors &amp; Intelligence</h2>
                </div>
                <p className="text-xs text-slate-500 mt-1">
                  Automated topic spike detection, emerging customer conversation themes, and real-time sentiment telemetry.
                </p>
              </div>
              <button
                onClick={() => setShowMonitorsDrawer(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1"
              >
                ✕
              </button>
            </div>

            {/* Sentiment Telemetry Heatmap */}
            {monitorsData && (
              <div className="bg-slate-50 rounded-xl p-4 border border-slate-200 space-y-3">
                <div className="flex items-center justify-between text-xs font-semibold text-slate-900">
                  <span>Customer Sentiment Breakdown</span>
                  <span className="text-slate-500 text-[11px] font-normal">{monitorsData.totalMonitoredConversations} Analyzed</span>
                </div>
                <div className="h-3 w-full rounded-full bg-slate-200 overflow-hidden flex">
                  <div
                    className="bg-emerald-500 h-full"
                    style={{ width: `${monitorsData.sentimentTelemetry.positive || 60}%` }}
                    title={`Positive: ${monitorsData.sentimentTelemetry.positive}%`}
                  />
                  <div
                    className="bg-slate-400 h-full"
                    style={{ width: `${monitorsData.sentimentTelemetry.neutral || 25}%` }}
                    title={`Neutral: ${monitorsData.sentimentTelemetry.neutral}%`}
                  />
                  <div
                    className="bg-amber-500 h-full"
                    style={{ width: `${monitorsData.sentimentTelemetry.negative || 10}%` }}
                    title={`Negative: ${monitorsData.sentimentTelemetry.negative}%`}
                  />
                  <div
                    className="bg-rose-500 h-full"
                    style={{ width: `${monitorsData.sentimentTelemetry.urgent || 5}%` }}
                    title={`Urgent: ${monitorsData.sentimentTelemetry.urgent}%`}
                  />
                </div>
                <div className="flex justify-between text-[11px] text-slate-600 font-medium">
                  <span className="text-emerald-700">● Positive {monitorsData.sentimentTelemetry.positive}%</span>
                  <span className="text-slate-600">● Neutral {monitorsData.sentimentTelemetry.neutral}%</span>
                  <span className="text-amber-700">● Negative {monitorsData.sentimentTelemetry.negative}%</span>
                  <span className="text-rose-700">● Urgent {monitorsData.sentimentTelemetry.urgent}%</span>
                </div>
              </div>
            )}

            {/* Emerging Themes */}
            {monitorsData && (
              <div className="space-y-3">
                <h3 className="font-bold text-sm text-slate-900 flex items-center justify-between">
                  <span>Emerging Conversation Themes</span>
                  <span className="text-xs text-purple-600 font-semibold">Live Feed</span>
                </h3>
                <div className="space-y-2.5">
                  {monitorsData.emergingThemes.map((theme, idx) => (
                    <div key={idx} className="p-3.5 rounded-xl border border-slate-200 bg-white shadow-2xs space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="font-semibold text-slate-900 text-xs">{theme.theme}</span>
                        <span className="text-[11px] font-bold text-emerald-700 bg-emerald-50 px-2 py-0.5 rounded border border-emerald-200">
                          {theme.growth}
                        </span>
                      </div>
                      <p className="text-xs text-slate-600 italic">"{theme.sampleQuote}"</p>
                      <div className="flex items-center justify-between text-[11px] text-slate-500 pt-1 border-t border-slate-100">
                        <span>💡 {theme.recommendedAction}</span>
                        <span className="font-semibold text-slate-800">{theme.volume} chats</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Active Topic Monitors */}
            {monitorsData && (
              <div className="space-y-3">
                <h3 className="font-bold text-sm text-slate-900">Active Topic Monitors &amp; Spike Detectors</h3>
                <div className="space-y-2.5">
                  {monitorsData.activeMonitors.map((mon) => (
                    <div key={mon.id} className="p-3.5 rounded-xl border border-slate-200 bg-white shadow-2xs space-y-1.5">
                      <div className="flex items-center justify-between">
                        <span className="font-semibold text-slate-900 text-xs">{mon.name}</span>
                        <span
                          className={`text-[10px] font-bold uppercase px-2 py-0.5 rounded ${
                            mon.status === 'triggered'
                              ? 'bg-rose-100 text-rose-800'
                              : 'bg-emerald-100 text-emerald-800'
                          }`}
                        >
                          {mon.status}
                        </span>
                      </div>
                      <p className="text-xs text-slate-600 leading-snug">{mon.insight}</p>
                      <div className="flex flex-wrap gap-1 pt-1">
                        {mon.keywords.map((kw, i) => (
                          <span key={i} className="px-1.5 py-0.2 rounded bg-slate-100 text-slate-600 text-[10px]">
                            #{kw}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Create Custom Monitor Form */}
            <div className="border-t border-slate-200 pt-4 space-y-3">
              <h3 className="font-bold text-xs text-slate-900 uppercase tracking-wider text-slate-500">
                + Add Custom Topic Monitor
              </h3>
              <div className="space-y-2 text-xs">
                <div>
                  <label className="block text-slate-600 font-medium mb-1">Monitor Name</label>
                  <input
                    type="text"
                    placeholder="e.g. Refund Escalation Guard"
                    value={newMonitorName}
                    onChange={(e) => setNewMonitorName(e.target.value)}
                    className="w-full rounded-md border border-slate-300 p-2 text-xs focus:border-purple-500 focus:outline-hidden"
                  />
                </div>
                <div>
                  <label className="block text-slate-600 font-medium mb-1">Topic Category</label>
                  <select
                    value={newMonitorTopic}
                    onChange={(e) => setNewMonitorTopic(e.target.value)}
                    className="w-full rounded-md border border-slate-300 p-2 text-xs focus:border-purple-500 focus:outline-hidden"
                  >
                    <option value="support">Customer Support</option>
                    <option value="billing">Billing &amp; Refunds</option>
                    <option value="technical">Technical &amp; Bugs</option>
                    <option value="promotions">Promotions &amp; Sales</option>
                  </select>
                </div>
                <div>
                  <label className="block text-slate-600 font-medium mb-1">Keywords (comma-separated)</label>
                  <input
                    type="text"
                    placeholder="refund, dispute, chargeback, cancel"
                    value={newMonitorKeywords}
                    onChange={(e) => setNewMonitorKeywords(e.target.value)}
                    className="w-full rounded-md border border-slate-300 p-2 text-xs focus:border-purple-500 focus:outline-hidden"
                  />
                </div>
                <button
                  onClick={handleCreateMonitor}
                  disabled={!newMonitorName.trim() || !newMonitorKeywords.trim()}
                  className="w-full py-2 bg-purple-700 hover:bg-purple-800 text-white font-bold rounded-lg text-xs shadow-xs disabled:opacity-40 transition-colors"
                >
                  Create Monitor Rule
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* REAL-TIME COLLISION SEND INTERLOCK MODAL                                 */}
      {/* ========================================================================= */}
      {showCollisionInterlock && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-rose-200 shadow-2xl max-w-md w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-rose-100 text-rose-700 flex items-center justify-center font-bold text-lg">
                ⚠️
              </div>
              <div>
                <h3 className="font-bold text-slate-900 text-base">Overlapping Reply Warning</h3>
                <p className="text-xs text-rose-600 font-semibold">Active Teammate Composing Response</p>
              </div>
            </div>

            <p className="text-xs text-slate-600 leading-relaxed">
              {collisionInfo?.warningMessage || 'Another teammate is currently typing a reply to this customer.'}{' '}
              Sending your message now may result in contradictory or double customer replies.
            </p>

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
              <button
                onClick={() => setShowCollisionInterlock(false)}
                className="px-3.5 py-1.5 rounded-lg border border-slate-200 bg-white text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs"
              >
                Cancel &amp; Coordinate
              </button>
              <button
                onClick={executeSend}
                className="px-3.5 py-1.5 rounded-lg bg-rose-600 text-xs font-bold text-white hover:bg-rose-700 shadow-xs"
              >
                Send Anyway
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* CREATE CUSTOM MACRO MODAL                                                 */}
      {/* ========================================================================= */}
      {showCreateMacroModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-lg w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center font-bold text-sm">
                  ⚡
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Create One-Click Macro</h3>
                  <p className="text-xs text-slate-500">Automate multi-action sequences across conversations</p>
                </div>
              </div>
              <button
                onClick={() => setShowCreateMacroModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleCreateMacro} className="space-y-3.5">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Macro Name *</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. VIP Priority Escalation + Manager Note"
                  value={newMacroName}
                  onChange={(e) => setNewMacroName(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-brand-500 focus:outline-hidden"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Description</label>
                <input
                  type="text"
                  placeholder="Explains what this workflow executes"
                  value={newMacroDesc}
                  onChange={(e) => setNewMacroDesc(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-brand-500 focus:outline-hidden"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Set Priority</label>
                  <select
                    value={newMacroPriority}
                    onChange={(e) => setNewMacroPriority(e.target.value as any)}
                    className="w-full px-2.5 py-1.5 border border-slate-300 rounded-lg text-xs bg-white focus:outline-hidden"
                  >
                    <option value="low">Low</option>
                    <option value="medium">Medium</option>
                    <option value="high">High</option>
                    <option value="urgent">Urgent</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Add Tags</label>
                  <input
                    type="text"
                    placeholder="e.g. vip, follow-up"
                    value={newMacroTag}
                    onChange={(e) => setNewMacroTag(e.target.value)}
                    className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:outline-hidden"
                  />
                </div>
              </div>

              <div className="flex items-center gap-4 pt-1">
                <label className="flex items-center gap-1.5 text-xs text-slate-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={newMacroResolve}
                    onChange={(e) => setNewMacroResolve(e.target.checked)}
                    className="rounded text-brand-600 focus:ring-brand-500"
                  />
                  <span>Resolve conversation</span>
                </label>
                <label className="flex items-center gap-1.5 text-xs text-slate-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={newMacroCSAT}
                    onChange={(e) => setNewMacroCSAT(e.target.checked)}
                    className="rounded text-brand-600 focus:ring-brand-500"
                  />
                  <span>Send CSAT survey</span>
                </label>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Customer Outbound Message (optional)</label>
                <textarea
                  rows={2}
                  placeholder="Automated reply sent directly to the customer..."
                  value={newMacroMessage}
                  onChange={(e) => setNewMacroMessage(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:outline-hidden resize-none"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Internal Private Note (optional)</label>
                <textarea
                  rows={2}
                  placeholder="Staff-only audit note attached to thread..."
                  value={newMacroNote}
                  onChange={(e) => setNewMacroNote(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:outline-hidden resize-none"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
                <button
                  type="button"
                  onClick={() => setShowCreateMacroModal(false)}
                  className="px-3 py-1.5 text-xs text-slate-600 hover:bg-slate-100 rounded-lg font-medium transition-colors cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
                >
                  Save Macro
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* SLA POLICY & BREACH MONITOR MODAL                                         */}
      {/* ========================================================================= */}
      {showSLAMonitorModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-2xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-rose-100 text-rose-700 flex items-center justify-center font-bold text-sm">
                  ⏱️
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">SLA Compliance &amp; Breach Monitor</h3>
                  <p className="text-xs text-slate-500">Service level agreements, automated breach alerts, and real-time fulfillment rates</p>
                </div>
              </div>
              <button
                onClick={() => setShowSLAMonitorModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            {/* Top 4 KPI Metrics */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
              <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
                <span className="text-[10px] font-semibold text-slate-500 uppercase">Tracked Tickets</span>
                <p className="text-lg font-bold text-slate-900 mt-0.5">{slaMetrics?.totalTracked ?? 12}</p>
              </div>
              <div className="p-3 rounded-xl bg-emerald-50/70 border border-emerald-200/80">
                <span className="text-[10px] font-semibold text-emerald-700 uppercase">1st Resp Rate</span>
                <p className="text-lg font-bold text-emerald-900 mt-0.5">{slaMetrics?.firstResponseFulfillmentRate ?? 95.8}%</p>
              </div>
              <div className="p-3 rounded-xl bg-indigo-50/70 border border-indigo-200/80">
                <span className="text-[10px] font-semibold text-indigo-700 uppercase">Resolution Rate</span>
                <p className="text-lg font-bold text-indigo-900 mt-0.5">{slaMetrics?.resolutionFulfillmentRate ?? 92.4}%</p>
              </div>
              <div className="p-3 rounded-xl bg-rose-50/70 border border-rose-200/80">
                <span className="text-[10px] font-semibold text-rose-700 uppercase">Active Breaches</span>
                <p className="text-lg font-bold text-rose-900 mt-0.5">{slaMetrics?.activeBreachesCount ?? (slaBreaches.length || 0)}</p>
              </div>
            </div>

            {/* Configured SLA Policies */}
            <div>
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider mb-2 flex items-center justify-between">
                <span>Active SLA Policies</span>
                <span className="text-[11px] font-normal text-slate-500">{slaPolicies.length} configured</span>
              </h4>
              <div className="space-y-2">
                {slaPolicies.map((policy) => (
                  <div
                    key={policy.id}
                    className="p-3 rounded-lg border border-slate-200/80 bg-white hover:border-slate-300 transition-all flex items-center justify-between gap-3 text-xs"
                  >
                    <div>
                      <p className="font-semibold text-slate-900">{policy.name}</p>
                      <p className="text-[11px] text-slate-500 line-clamp-1">{policy.description}</p>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="px-2 py-0.5 rounded bg-blue-50 text-blue-700 text-[10px] font-bold">
                        1st: {policy.firstResponseTimeThreshold}m
                      </span>
                      <span className="px-2 py-0.5 rounded bg-purple-50 text-purple-700 text-[10px] font-bold">
                        Res: {policy.resolutionTimeThreshold >= 60 ? `${Math.round(policy.resolutionTimeThreshold / 60)}h` : `${policy.resolutionTimeThreshold}m`}
                      </span>
                      {policy.priorities.map((p) => (
                        <span key={p} className="px-1.5 py-0.5 rounded bg-slate-100 text-slate-700 text-[9px] font-medium uppercase">
                          {p}
                        </span>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Active Breaches & At-Risk Conversations */}
            <div>
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider mb-2 flex items-center justify-between">
                <span>Active Breaches &amp; At-Risk Conversations</span>
                <span className="text-[11px] font-normal text-slate-500">{slaBreaches.length} tickets</span>
              </h4>
              {slaBreaches.length === 0 ? (
                <div className="p-4 rounded-lg bg-emerald-50/50 border border-emerald-200 text-center text-xs text-emerald-800">
                  🎉 No active SLA breaches! All response and resolution targets are currently fulfilled.
                </div>
              ) : (
                <div className="border border-slate-200 rounded-lg overflow-hidden divide-y divide-slate-100 text-xs">
                  {slaBreaches.map((b) => (
                    <div
                      key={b.conversationId}
                      className="p-2.5 flex items-center justify-between hover:bg-slate-50 transition-colors"
                    >
                      <div className="flex items-center gap-2">
                        <span className="text-base">{b.isBreached ? '⚠️' : '⏰'}</span>
                        <div>
                          <p className="font-semibold text-slate-900">{b.contactName}</p>
                          <p className="text-[10px] text-slate-500">{b.channel} · Priority: {b.priority}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className={`px-2 py-0.5 rounded font-bold text-[10px] ${
                          b.isBreached ? 'bg-rose-100 text-rose-800' : 'bg-amber-100 text-amber-800'
                        }`}>
                          {b.isBreached ? 'Breached' : 'Nearing Breach'}
                        </span>
                        <button
                          type="button"
                          onClick={() => {
                            setSelectedId(b.conversationId)
                            setShowSLAMonitorModal(false)
                          }}
                          className="px-2 py-1 text-[11px] rounded bg-slate-900 text-white font-semibold hover:bg-slate-800 cursor-pointer"
                        >
                          Open Thread →
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="flex items-center justify-end pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setShowSLAMonitorModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* AGENT CAPACITY & ROUTING MANAGER MODAL                                     */}
      {/* ========================================================================= */}
      {showCapacityModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-3xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-teal-100 text-teal-700 flex items-center justify-center font-bold text-sm">
                  👥
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Team Routing &amp; Capacity Manager</h3>
                  <p className="text-xs text-slate-500">Live agent availability, concurrent capacity limits, and workload distribution</p>
                </div>
              </div>
              <button
                onClick={() => setShowCapacityModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            {/* Top 5 KPI Metrics */}
            <div className="grid grid-cols-2 sm:grid-cols-5 gap-3">
              <div className="p-3 rounded-xl bg-slate-50 border border-slate-200/80">
                <span className="text-[10px] font-semibold text-slate-500 uppercase">Total Agents</span>
                <p className="text-lg font-bold text-slate-900 mt-0.5">{capacityOverview?.totalAgents ?? routingAgents.length}</p>
              </div>
              <div className="p-3 rounded-xl bg-emerald-50/70 border border-emerald-200/80">
                <span className="text-[10px] font-semibold text-emerald-700 uppercase">Online Agents</span>
                <p className="text-lg font-bold text-emerald-900 mt-0.5">{capacityOverview?.onlineAgents ?? routingAgents.filter(a => a.availabilityStatus === 'online').length}</p>
              </div>
              <div className="p-3 rounded-xl bg-blue-50/70 border border-blue-200/80">
                <span className="text-[10px] font-semibold text-blue-700 uppercase">Total Slots</span>
                <p className="text-lg font-bold text-blue-900 mt-0.5">{capacityOverview?.totalMaxCapacity ?? 18}</p>
              </div>
              <div className="p-3 rounded-xl bg-purple-50/70 border border-purple-200/80">
                <span className="text-[10px] font-semibold text-purple-700 uppercase">Open Slots</span>
                <p className="text-lg font-bold text-purple-900 mt-0.5">{capacityOverview?.availableOnlineSlots ?? 12}</p>
              </div>
              <div className="p-3 rounded-xl bg-amber-50/70 border border-amber-200/80">
                <span className="text-[10px] font-semibold text-amber-700 uppercase">Load %</span>
                <p className="text-lg font-bold text-amber-900 mt-0.5">{capacityOverview?.utilizationRatePct ?? 25.0}%</p>
              </div>
            </div>

            {/* Batch Routing Action Bar */}
            <div className="p-3 rounded-xl bg-gradient-to-r from-teal-50 to-emerald-50 border border-teal-200 flex items-center justify-between gap-3">
              <div>
                <p className="font-bold text-xs text-teal-950">Auto-Route Unassigned Conversations</p>
                <p className="text-[11px] text-teal-700">Evenly distributes all unassigned open tickets across online agents</p>
              </div>
              <button
                type="button"
                onClick={handleBatchAutoAssign}
                className="px-3.5 py-1.5 rounded-lg bg-teal-800 hover:bg-teal-900 text-white font-semibold text-xs shadow-xs transition-colors cursor-pointer"
              >
                ⚡ Run Batch Auto-Assign
              </button>
            </div>

            {/* Agents Table */}
            <div>
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider mb-2 flex items-center justify-between">
                <span>Agent Availability &amp; Capacity Limits</span>
                <span className="text-[11px] font-normal text-slate-500">{routingAgents.length} agents</span>
              </h4>
              <div className="border border-slate-200 rounded-lg overflow-hidden divide-y divide-slate-100 text-xs">
                {routingAgents.map((ag) => (
                  <div key={ag.id} className="p-3 flex items-center justify-between hover:bg-slate-50 transition-colors">
                    <div className="flex items-center gap-3">
                      <div className="relative">
                        <div className="w-8 h-8 rounded-full bg-slate-200 text-slate-700 flex items-center justify-center font-bold text-xs uppercase">
                          {ag.name.slice(0, 2)}
                        </div>
                        <span
                          className={`absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full ring-2 ring-white ${
                            ag.availabilityStatus === 'online'
                              ? 'bg-emerald-500'
                              : ag.availabilityStatus === 'busy'
                              ? 'bg-amber-500'
                              : 'bg-slate-400'
                          }`}
                        />
                      </div>
                      <div>
                        <p className="font-semibold text-slate-900">{ag.name}</p>
                        <p className="text-[11px] text-slate-500">{ag.email}</p>
                      </div>
                    </div>

                    <div className="flex items-center gap-4">
                      {/* Status Selector */}
                      <div>
                        <label className="block text-[9px] text-slate-400 font-semibold uppercase mb-0.5">Status</label>
                        <select
                          value={ag.availabilityStatus}
                          onChange={async (e) => {
                            const newSt = e.target.value as any
                            await api.updateAgentStatus(ag.id, newSt)
                            const [updated, ov] = await Promise.all([
                              api.listRoutingAgents(),
                              api.getCapacityOverview(),
                            ])
                            setRoutingAgents(updated)
                            setCapacityOverview(ov)
                          }}
                          className="text-xs border border-slate-300 rounded px-2 py-1 bg-white focus:outline-hidden cursor-pointer"
                        >
                          <option value="online">🟢 Online</option>
                          <option value="busy">🟡 Busy</option>
                          <option value="offline">⚪ Offline</option>
                        </select>
                      </div>

                      {/* Capacity Limit input */}
                      <div>
                        <label className="block text-[9px] text-slate-400 font-semibold uppercase mb-0.5">Max Slots</label>
                        <input
                          type="number"
                          min={1}
                          max={50}
                          defaultValue={ag.maxCapacity}
                          onBlur={(e) => {
                            const val = parseInt(e.target.value, 10)
                            if (val > 0 && val !== ag.maxCapacity) {
                              handleUpdateCapacity(ag.id, val)
                            }
                          }}
                          className="w-16 text-xs border border-slate-300 rounded px-2 py-1 text-center bg-white focus:outline-hidden"
                        />
                      </div>

                      {/* Active Load & Bar */}
                      <div className="w-28 text-right">
                        <span className="text-[11px] font-semibold text-slate-800">
                          {ag.activeConversationsCount}/{ag.maxCapacity} active
                        </span>
                        <div className="w-full bg-slate-200 h-1.5 rounded-full mt-1 overflow-hidden">
                          <div
                            className={`h-full rounded-full ${
                              ag.utilizationRatePct >= 100
                                ? 'bg-rose-500'
                                : ag.utilizationRatePct >= 70
                                ? 'bg-amber-500'
                                : 'bg-emerald-500'
                            }`}
                            style={{ width: `${Math.min(100, ag.utilizationRatePct)}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-end pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setShowCapacityModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* CANNED RESPONSES / SLASH COMMANDS MANAGER MODAL                             */}
      {/* ========================================================================= */}
      {showCannedModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-3xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-blue-100 text-blue-700 flex items-center justify-center font-bold text-sm">
                  ⚡
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Canned Responses &amp; Slash Commands</h3>
                  <p className="text-xs text-slate-500">
                    Create reusable response templates accessible in any chat composer via <span className="font-mono font-bold text-blue-600">/shortcode</span>
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowCannedModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            {/* Create New Canned Snippet Form */}
            <div className="bg-blue-50/50 border border-blue-200/70 rounded-xl p-4 space-y-3">
              <h4 className="text-xs font-bold text-blue-900 uppercase tracking-wider flex items-center gap-1.5">
                <span>➕ Create New Canned Snippet</span>
              </h4>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label className="block text-[11px] font-semibold text-slate-700 mb-1">
                    Shortcode (Command) <span className="text-rose-500">*</span>
                  </label>
                  <div className="flex items-center">
                    <span className="bg-slate-100 border border-r-0 border-slate-300 rounded-l-lg px-2.5 py-1.5 text-xs font-mono font-bold text-slate-600">
                      /
                    </span>
                    <input
                      type="text"
                      placeholder="e.g. shipping_time"
                      value={newCannedCode}
                      onChange={(e) => setNewCannedCode(e.target.value.replace(/[^a-zA-Z0-9_-]/g, ''))}
                      className="w-full text-xs border border-slate-300 rounded-r-lg px-2.5 py-1.5 bg-white focus:outline-hidden focus:border-blue-500"
                    />
                  </div>
                </div>

                <div>
                  <label className="block text-[11px] font-semibold text-slate-700 mb-1">
                    Category
                  </label>
                  <select
                    value={newCannedCategory}
                    onChange={(e) => setNewCannedCategory(e.target.value)}
                    className="w-full text-xs border border-slate-300 rounded-lg px-2.5 py-1.5 bg-white focus:outline-hidden focus:border-blue-500"
                  >
                    <option value="greetings">Greetings</option>
                    <option value="sales">Sales</option>
                    <option value="support">Support</option>
                    <option value="billing">Billing</option>
                    <option value="general">General</option>
                  </select>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="text-[11px] font-semibold text-slate-700">
                    Template Content <span className="text-rose-500">*</span>
                  </label>
                  <div className="flex items-center gap-1 text-[10.5px]">
                    <span className="text-slate-400">Insert tag:</span>
                    {['customer.first_name', 'customer.name', 'agent.name', 'ticket.id', 'channel'].map((tag) => (
                      <button
                        key={tag}
                        type="button"
                        onClick={() => setNewCannedContent((prev) => prev + `{{${tag}}}`)}
                        className="bg-white border border-slate-200 hover:border-blue-400 text-blue-600 font-mono px-1.5 py-0.5 rounded cursor-pointer transition-colors"
                      >
                        +{tag}
                      </button>
                    ))}
                  </div>
                </div>
                <textarea
                  rows={2}
                  placeholder="Hi {{customer.first_name}}! Thanks for reaching out..."
                  value={newCannedContent}
                  onChange={(e) => setNewCannedContent(e.target.value)}
                  className="w-full text-xs border border-slate-300 rounded-lg px-3 py-2 bg-white focus:outline-hidden focus:border-blue-500 font-normal"
                />
              </div>

              <div className="flex justify-end">
                <button
                  type="button"
                  onClick={handleCreateCanned}
                  disabled={isSavingCanned || !newCannedCode.trim() || !newCannedContent.trim()}
                  className="bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold px-4 py-1.5 rounded-lg shadow-xs transition-colors disabled:opacity-50 cursor-pointer"
                >
                  {isSavingCanned ? 'Saving…' : 'Save Canned Snippet'}
                </button>
              </div>
            </div>

            {/* Existing Snippets List */}
            <div className="space-y-2">
              <h4 className="text-xs font-bold text-slate-900 uppercase tracking-wider flex items-center justify-between">
                <span>Existing Canned Replies</span>
                <span className="text-[11px] font-normal text-slate-500">{cannedResponses.length} snippets</span>
              </h4>

              <div className="border border-slate-200 rounded-xl overflow-hidden divide-y divide-slate-100 max-h-72 overflow-y-auto">
                {cannedResponses.map((c) => (
                  <div key={c.id || c.shortCode || c.short_code} className="p-3 hover:bg-slate-50/70 transition-colors flex items-start justify-between gap-3">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-black text-blue-700 bg-blue-100/80 px-2 py-0.5 rounded-md">
                          /{c.shortCode || c.short_code}
                        </span>
                        <span className="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded-full bg-slate-100 text-slate-600 border border-slate-200">
                          {c.category || 'general'}
                        </span>
                      </div>
                      <p className="text-xs text-slate-700 font-normal leading-relaxed">
                        {c.content}
                      </p>
                      <p className="text-[11px] text-slate-400 italic">
                        Preview: {resolveCannedVariables(c.content)}
                      </p>
                    </div>

                    <button
                      type="button"
                      onClick={() => handleDeleteCanned(c.id)}
                      title="Delete canned response"
                      className="text-slate-400 hover:text-rose-600 text-xs p-1 rounded hover:bg-rose-50 transition-colors cursor-pointer shrink-0"
                    >
                      🗑️
                    </button>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between pt-2 border-t border-slate-100 text-xs text-slate-500">
              <span>💡 All canned replies automatically resolve customer and agent names on insert.</span>
              <button
                type="button"
                onClick={() => setShowCannedModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* COPILOT TEAMMATE APPROVALS & WORKFLOWS CENTER MODAL                         */}
      {/* ========================================================================= */}
      {showApprovalsModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-4xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center font-bold text-sm">
                  🤝
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Copilot Teammate Approvals Center</h3>
                  <p className="text-xs text-slate-500">
                    Agent and your team work from the same inbox and record. When Agent needs approval or info, it asks a teammate and finishes the conversation itself.
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowApprovalsModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            {/* Filter Tabs */}
            <div className="flex items-center gap-2 border-b border-slate-100 pb-2">
              {['all', 'pending', 'approved', 'rejected'].map((st) => {
                const count = st === 'all'
                  ? copilotApprovals.length
                  : copilotApprovals.filter((a) => a.status === st).length
                return (
                  <button
                    key={st}
                    type="button"
                    onClick={() => setApprovalFilterStatus(st)}
                    className={`px-3 py-1.5 rounded-lg text-xs font-semibold capitalize transition-colors flex items-center gap-1.5 cursor-pointer ${
                      approvalFilterStatus === st
                        ? 'bg-amber-600 text-white shadow-xs'
                        : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                    }`}
                  >
                    <span>{st}</span>
                    <span className={`text-[10px] px-1.5 py-0.2 rounded-full ${
                      approvalFilterStatus === st ? 'bg-amber-700 text-white' : 'bg-slate-200 text-slate-700'
                    }`}>
                      {count}
                    </span>
                  </button>
                )
              })}

              <button
                type="button"
                onClick={handleTriggerRefundApprovalDemo}
                className="ml-auto text-xs font-bold text-amber-700 bg-amber-50 hover:bg-amber-100 border border-amber-300 px-3 py-1.5 rounded-lg flex items-center gap-1 cursor-pointer transition-colors"
              >
                <span>➕</span>
                <span>Simulate AI Refund Request</span>
              </button>
            </div>

            {/* Approvals List */}
            <div className="space-y-3">
              {copilotApprovals
                .filter((a) => approvalFilterStatus === 'all' || a.status === approvalFilterStatus)
                .map((app) => (
                  <div
                    key={app.id}
                    className={`rounded-xl border p-4 transition-all ${
                      app.status === 'pending'
                        ? 'border-amber-300 bg-amber-50/40 shadow-xs'
                        : app.status === 'approved' || app.status === 'amended'
                        ? 'border-emerald-200 bg-emerald-50/20'
                        : 'border-rose-200 bg-rose-50/20'
                    }`}
                  >
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-100 pb-2.5">
                      <div className="flex items-center gap-2">
                        <span className="text-base">
                          {app.status === 'pending' ? '⏳' : app.status === 'approved' || app.status === 'amended' ? '✅' : '❌'}
                        </span>
                        <div>
                          <h4 className="font-bold text-slate-900 text-xs">{app.title}</h4>
                          <span className="text-[11px] text-slate-500 font-mono">
                            Conversation ID: {app.conversationId || app.conversation_id}
                          </span>
                        </div>
                      </div>

                      <div className="flex items-center gap-2">
                        <span className="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-slate-100 text-slate-700 border border-slate-200">
                          {app.requestType || app.request_type}
                        </span>
                        <span
                          className={`text-[10px] font-black uppercase tracking-wider px-2 py-0.5 rounded-full ${
                            app.status === 'pending'
                              ? 'bg-amber-200 text-amber-900 border border-amber-300'
                              : app.status === 'approved' || app.status === 'amended'
                              ? 'bg-emerald-100 text-emerald-800 border border-emerald-300'
                              : 'bg-rose-100 text-rose-800 border border-rose-300'
                          }`}
                        >
                          {app.status}
                        </span>
                      </div>
                    </div>

                    <div className="mt-2.5 space-y-2 text-xs">
                      <p className="text-slate-600 bg-white/70 p-2 rounded-lg border border-slate-200/60 font-medium">
                        <strong className="text-slate-700">AI Context: </strong>
                        {app.internalNote || app.internal_note}
                      </p>

                      {app.actionData && Object.keys(app.actionData).length > 0 && (
                        <div className="flex flex-wrap items-center gap-2 text-[11px]">
                          {app.actionData.amount && (
                            <span className="bg-amber-100 text-amber-900 font-bold px-2 py-0.5 rounded-md">
                              Amount: ${Number(app.actionData.amount).toFixed(2)}
                            </span>
                          )}
                          {app.actionData.order_id && (
                            <span className="bg-slate-100 text-slate-800 font-mono px-2 py-0.5 rounded-md">
                              Order #{app.actionData.order_id}
                            </span>
                          )}
                          {app.actionData.policy_rule && (
                            <span className="text-slate-500 italic">
                              Policy: {app.actionData.policy_rule}
                            </span>
                          )}
                        </div>
                      )}

                      {/* Proposed reply display */}
                      <div className="bg-white rounded-lg p-2.5 border border-slate-200">
                        <span className="block text-[10px] font-bold uppercase text-slate-400 mb-1">
                          {app.status === 'pending' ? 'Proposed Customer Reply:' : 'Final Dispatched Reply:'}
                        </span>
                        <p className="text-slate-800 leading-relaxed font-normal">
                          "{app.amendedReply || app.amended_reply || app.proposedReply || app.proposed_reply}"
                        </p>
                      </div>

                      {/* Review details if resolved */}
                      {app.reviewerName && (
                        <div className="text-[11px] text-slate-500 pt-1">
                          Reviewed by <strong className="text-slate-700">{app.reviewerName}</strong>
                          {app.reviewerFeedback && ` • Feedback: "${app.reviewerFeedback}"`}
                        </div>
                      )}

                      {/* Actions for pending items */}
                      {app.status === 'pending' && (
                        <div className="flex items-center justify-end gap-2 pt-2 border-t border-amber-200/60">
                          <button
                            type="button"
                            onClick={() => handleRejectRequest(app.id, 'Declined by teammate')}
                            className="text-xs text-rose-600 hover:text-rose-800 font-semibold px-2.5 py-1 rounded hover:bg-rose-50"
                          >
                            Reject
                          </button>
                          <button
                            type="button"
                            onClick={() => handleApproveRequest(app)}
                            className="bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs px-3.5 py-1.5 rounded-lg shadow-xs cursor-pointer transition-colors"
                          >
                            Approve &amp; Let AI Dispatch Reply
                          </button>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
            </div>

            <div className="flex items-center justify-between pt-2 border-t border-slate-100 text-xs text-slate-500">
              <span>🤖 Once approved, the AI Agent dispatches the message directly to the customer and finalizes the ticket.</span>
              <button
                type="button"
                onClick={() => setShowApprovalsModal(false)}
                className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Gap 3: 1-Click Issue Creation Modal (Linear / GitHub / Jira) */}
      {issueModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl bg-white p-6 shadow-2xl border border-slate-200 space-y-4">
            <div className="flex items-center justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <span className="text-xl">🎯</span>
                <div>
                  <h3 className="font-bold text-sm text-slate-900">Create & Link Issue</h3>
                  <p className="text-[11px] text-slate-500">
                    Target Conversation: {activeConv?.name} ({activeConv?.channel})
                  </p>
                </div>
              </div>
              <button
                onClick={() => setIssueModalOpen(false)}
                className="text-slate-400 hover:text-slate-600 text-lg font-bold cursor-pointer"
              >
                ✕
              </button>
            </div>

            {issueNotice && (
              <div
                className={`rounded-lg p-2.5 text-xs font-medium ${
                  issueNotice.includes('Failed')
                    ? 'bg-rose-50 text-rose-800 border border-rose-200'
                    : 'bg-emerald-50 text-emerald-800 border border-emerald-200'
                }`}
              >
                {issueNotice}
              </div>
            )}

            {/* Provider Tabs */}
            <div className="flex border-b border-slate-200 gap-2">
              <button
                type="button"
                onClick={() => setIssueProvider('linear')}
                className={`py-1.5 px-3 border-b-2 font-semibold text-xs transition-colors flex items-center gap-1.5 cursor-pointer ${
                  issueProvider === 'linear'
                    ? 'border-purple-600 text-purple-700 font-bold'
                    : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
              >
                <span>🟣</span>
                <span>Linear</span>
              </button>
              <button
                type="button"
                onClick={() => setIssueProvider('github')}
                className={`py-1.5 px-3 border-b-2 font-semibold text-xs transition-colors flex items-center gap-1.5 cursor-pointer ${
                  issueProvider === 'github'
                    ? 'border-slate-800 text-slate-900 font-bold'
                    : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
              >
                <span>🐙</span>
                <span>GitHub Issues</span>
              </button>
              <button
                type="button"
                onClick={() => setIssueProvider('jira')}
                className={`py-1.5 px-3 border-b-2 font-semibold text-xs transition-colors flex items-center gap-1.5 cursor-pointer ${
                  issueProvider === 'jira'
                    ? 'border-blue-600 text-blue-700 font-bold'
                    : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
              >
                <span>🔷</span>
                <span>Jira Cloud</span>
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">Issue Title</label>
                <input
                  type="text"
                  value={issueTitle}
                  onChange={(e) => setIssueTitle(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden font-medium"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-[11px] font-bold text-slate-700 mb-1">Priority</label>
                  <select
                    value={issuePriority}
                    onChange={(e) => setIssuePriority(e.target.value)}
                    className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden"
                  >
                    <option value="urgent">🔴 Urgent</option>
                    <option value="high">🟠 High</option>
                    <option value="medium">🟡 Medium</option>
                    <option value="low">⚪ Low</option>
                  </select>
                </div>

                <div>
                  <label className="block text-[11px] font-bold text-slate-700 mb-1">Issue Type</label>
                  <select
                    value={issueType}
                    onChange={(e) => setIssueType(e.target.value)}
                    className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden"
                  >
                    <option value="bug">🐛 Bug</option>
                    <option value="feature">✨ Feature Request</option>
                    <option value="task">📋 Improvement / Task</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">
                  Description &amp; Customer Context
                </label>
                <textarea
                  rows={4}
                  value={issueDescription}
                  onChange={(e) => setIssueDescription(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden font-mono"
                />
              </div>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setIssueModalOpen(false)}
                className="px-3 py-1.5 text-xs text-slate-600 hover:text-slate-800 font-semibold cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleCreateLinkedIssue}
                disabled={issueSubmitting || !issueTitle.trim()}
                className="px-4 py-1.5 text-xs bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-bold shadow-xs transition-colors disabled:opacity-50 flex items-center gap-1.5 cursor-pointer"
              >
                {issueSubmitting ? 'Creating...' : `Create in ${issueProvider.toUpperCase()}`}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 1-Click Stripe Refund Modal */}
      {refundModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl border border-slate-200 space-y-4">
            <div className="flex items-center justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <span className="text-xl">⚡</span>
                <div>
                  <h3 className="font-bold text-sm text-slate-900">Issue Stripe Customer Refund</h3>
                  <p className="text-[11px] text-slate-500">
                    Target: {activeConv?.name || 'Customer'} ({activeConv?.email || 'email'})
                  </p>
                </div>
              </div>
              <button
                onClick={() => setRefundModalOpen(false)}
                className="text-slate-400 hover:text-slate-600 text-lg font-bold cursor-pointer"
              >
                ✕
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">
                  Select Target Invoice / Charge
                </label>
                <select
                  value={selectedInvoiceForRefund?.id || ''}
                  onChange={(e) => {
                    const inv = stripeInvoices.find((i) => i.id === e.target.value)
                    setSelectedInvoiceForRefund(inv || null)
                  }}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-hidden"
                >
                  {stripeInvoices.map((inv) => (
                    <option key={inv.id} value={inv.id}>
                      {inv.number} · {inv.amount_paid_formatted} (Paid on {new Date(inv.created_at).toLocaleDateString()})
                    </option>
                  ))}
                  {stripeInvoices.length === 0 && (
                    <option value="sim_default">Default Recent Charge ($299.00)</option>
                  )}
                </select>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="text-[11px] font-bold text-slate-700">Refund Amount ($)</label>
                  <button
                    type="button"
                    onClick={() => {
                      if (selectedInvoiceForRefund) {
                        setRefundAmountInput((selectedInvoiceForRefund.amount_paid_cents / 100.0).toFixed(2))
                      }
                    }}
                    className="text-[10px] font-semibold text-indigo-600 hover:underline cursor-pointer"
                  >
                    Full Amount ({selectedInvoiceForRefund?.amount_paid_formatted || '$299.00'})
                  </button>
                </div>
                <input
                  type="number"
                  step="0.01"
                  placeholder={
                    selectedInvoiceForRefund
                      ? (selectedInvoiceForRefund.amount_paid_cents / 100.0).toFixed(2)
                      : '299.00'
                  }
                  value={refundAmountInput}
                  onChange={(e) => setRefundAmountInput(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-hidden"
                />
                <p className="text-[10px] text-slate-400 mt-1">
                  Leave blank to refund the full paid amount.
                </p>
              </div>

              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">
                  Refund Reason
                </label>
                <select
                  value={refundReason}
                  onChange={(e) => setRefundReason(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-hidden"
                >
                  <option value="requested_by_customer">Customer Request / Satisfaction Guarantee</option>
                  <option value="duplicate">Accidental Duplicate Charge</option>
                  <option value="fraudulent">Suspected Fraudulent Activity</option>
                  <option value="goodwill_credit">Service Disruption / Goodwill Credit</option>
                </select>
              </div>

              <div className="rounded-lg bg-amber-50 p-2.5 border border-amber-200 text-[11px] text-amber-800 leading-snug">
                ⚠️ Refunds are recorded immediately in Stripe and an immutable audit log is generated.
              </div>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setRefundModalOpen(false)}
                className="px-3 py-1.5 text-xs text-slate-600 hover:text-slate-800 font-semibold cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleExecuteRefund}
                disabled={refundSubmitting}
                className="px-4 py-1.5 text-xs bg-rose-600 hover:bg-rose-700 text-white rounded-lg font-bold shadow-xs transition-colors disabled:opacity-50 flex items-center gap-1.5 cursor-pointer"
              >
                {refundSubmitting ? 'Processing...' : 'Confirm & Execute Refund'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
