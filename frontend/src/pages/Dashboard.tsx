import { useState } from 'react'
import { Link } from 'react-router-dom'

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
  channelType: 'facebook' | 'instagram' | 'whatsapp' | 'website' | 'email'
  assignedAgent: string
  agentAvatar: string
  timeAgo: string
  preview: string
  unreadCount?: number
  sla?: string
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

export function Dashboard() {
  const [conversations, setConversations] = useState<ConversationItem[]>(INITIAL_CONVERSATIONS)
  const [selectedId, setSelectedId] = useState<string>('conv-maya')
  const [activeTab, setActiveTab] = useState<'Mine' | 'Unassigned' | 'All' | 'Autonomous'>('All')
  const [composerMode, setComposerMode] = useState<'reply' | 'note'>('reply')
  const [replyText, setReplyText] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [copiedField, setCopiedField] = useState<string | null>(null)
  const [showActionsAccordion, setShowActionsAccordion] = useState(true)
  const [showMacrosAccordion, setShowMacrosAccordion] = useState(false)
  const [showNavFolders, setShowNavFolders] = useState(true)
  const [showNavTeams, setShowNavTeams] = useState(true)
  const [showNavChannels, setShowNavChannels] = useState(true)
  const [showRightSidebar, setShowRightSidebar] = useState(true)

  const activeConv = conversations.find((c) => c.id === selectedId) || conversations[0]

  function handleSend() {
    if (!replyText.trim()) return

    const newMsg: MessageItem = {
      id: `m-${Date.now()}`,
      sender: composerMode === 'note' ? 'note' : 'agent',
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
            preview: `↩ ${replyText.trim().slice(0, 32)}...`,
            messages: [...c.messages, newMsg],
          }
        }
        return c
      })
    )

    setReplyText('')
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

  function handleAIDraft() {
    setReplyText("Hi Lorrie, I've verified your session on our backend. Everything is synced and working properly now. Let me know if you need any other assistance!")
  }

  const filteredConversations = conversations.filter((c) => {
    if (activeTab === 'Mine' && c.assignedAgent !== 'Riley Brooks') return false
    if (activeTab === 'Unassigned' && c.assignedAgent !== '') return false
    if (activeTab === 'Autonomous' && c.resolutionType !== 'autonomous_no_handoff') return false
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
              <div className="space-y-0.5 pt-1">
                {[
                  { name: 'Bay Works E...', icon: '✉️' },
                  { name: 'Bay Works Fa...', icon: '💬' },
                  { name: 'Bay Works Wh...', icon: '📱' },
                ].map((c) => (
                  <button
                    key={c.name}
                    className="flex w-full items-center gap-2 rounded-md px-2.5 py-1 text-left text-xs text-slate-600 hover:bg-[#f2f4f7]"
                  >
                    <span>{c.icon}</span>
                    <span>{c.name}</span>
                  </button>
                ))}
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
        </div>

        {/* Bottom Current Agent Account Card */}
        <div className="border-t border-[#eaecf0] p-3 bg-white">
          <div className="flex items-center gap-2.5">
            <img
              src="https://images.unsplash.com/photo-1580489944761-15a19d654956?w=150&auto=format&fit=crop&q=80"
              alt="Riley Brooks"
              className="h-8 w-8 rounded-full object-cover ring-1 ring-slate-200"
            />
            <div className="min-w-0 flex-1">
              <p className="truncate text-xs font-semibold text-slate-900 leading-tight">Riley Brooks</p>
              <p className="truncate text-[11px] text-slate-500 leading-tight">riley.brooks@baywork...</p>
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
                    <span className="inline-flex items-center gap-1 rounded bg-rose-50 px-1.5 py-0.5 font-medium text-rose-600">
                      <span>🔥</span>
                      <span>{item.sla}</span>
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
              <div className="flex items-center gap-1.5">
                <h2 className="font-semibold text-slate-900 text-sm">{activeConv.name}</h2>
              </div>
              <p className="text-xs text-slate-500">
                #1 · {activeConv.channel}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {activeConv.sla && (
              <span className="inline-flex items-center gap-1 rounded-full bg-rose-50 px-2.5 py-1 text-xs font-medium text-rose-600">
                <span>🔥</span>
                <span>{activeConv.sla}</span>
              </span>
            )}

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

        {/* Sub-header Tabs (Messages / Customer Dashboard) */}
        <div className="flex items-center justify-between border-b border-[#eaecf0] px-6 pt-2">
          <div className="flex items-center gap-6 text-xs font-medium">
            <button className="border-b-2 border-blue-600 pb-2 text-blue-600 font-semibold">
              Messages
            </button>
            <button className="pb-2 text-slate-500 hover:text-slate-800">
              Customer Dashboard
            </button>
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
                    Grounded in Knowledge Base
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

          <div className="rounded-xl border border-[#d0d5dd] bg-white shadow-xs focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500">
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
                    composerMode === 'note' ? 'bg-amber-100 text-amber-900 shadow-xs font-semibold' : 'text-slate-600'
                  }`}
                >
                  Private Note
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
                <button title="Expand editor" className="rounded p-1 hover:text-slate-600">
                  ⤢
                </button>
              </div>
            </div>

            {/* Composer Textarea */}
            <textarea
              rows={3}
              value={replyText}
              onChange={(e) => setReplyText(e.target.value)}
              onKeyDown={(e) => {
                if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                  e.preventDefault()
                  handleSend()
                }
              }}
              placeholder="Shift + enter for new line. Start with '/' to select a Canned Response."
              className="w-full resize-none px-3.5 py-2.5 text-xs text-slate-800 placeholder-slate-400 focus:outline-hidden"
            />

            {/* Bottom Toolbar & Send Button */}
            <div className="flex items-center justify-between border-t border-[#f2f4f7] px-3 py-2">
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
                <button title="Canned responses" className="hover:text-slate-600">
                  ⚡
                </button>
              </div>

              <button
                onClick={handleSend}
                disabled={!replyText.trim()}
                className="inline-flex items-center gap-1.5 rounded-lg bg-[#1570ef] px-3.5 py-1.5 text-xs font-semibold text-white shadow-xs hover:bg-blue-700 disabled:opacity-40"
              >
                <span>Send (⌘ + ↵)</span>
              </button>
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
            <h3 className="font-semibold text-slate-900 text-sm">Contact</h3>
            <button
              onClick={() => setShowRightSidebar(false)}
              className="text-slate-400 hover:text-slate-600 font-bold"
            >
              ✕
            </button>
          </div>

          <div className="p-5 space-y-6">
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
                  {/* Assigned Agent Dropdown */}
                  <div>
                    <label className="block text-slate-500 font-medium mb-1">Assigned Agent</label>
                    <div className="flex items-center justify-between rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 shadow-xs">
                      <div className="flex items-center gap-2 truncate">
                        <img
                          src={activeConv.agentAvatar}
                          alt={activeConv.assignedAgent}
                          className="h-5 w-5 rounded-full object-cover"
                        />
                        <span className="font-medium text-slate-800">{activeConv.assignedAgent}</span>
                      </div>
                      <span className="text-slate-400 text-xs">⌄</span>
                    </div>
                  </div>

                  {/* Assigned Team Dropdown */}
                  <div>
                    <label className="block text-slate-500 font-medium mb-1">Assigned Team</label>
                    <div className="flex items-center justify-between rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 shadow-xs">
                      <div className="flex items-center gap-2 truncate">
                        <span className="flex h-4 w-4 items-center justify-center rounded-full bg-emerald-100 text-[10px] font-bold text-emerald-800">
                          T
                        </span>
                        <span className="font-medium text-slate-800">🛠️ {activeConv.team}</span>
                      </div>
                      <span className="text-slate-400 text-xs">⌄</span>
                    </div>
                  </div>

                  {/* Priority Dropdown */}
                  <div>
                    <label className="block text-slate-500 font-medium mb-1">Priority</label>
                    <div className="flex items-center justify-between rounded-md border border-[#d0d5dd] bg-white px-2.5 py-1.5 shadow-xs">
                      <div className="flex items-center gap-2">
                        <span className="text-amber-500 text-xs">📶</span>
                        <span className="font-medium text-slate-800">{activeConv.priority}</span>
                      </div>
                      <span className="text-slate-400 text-xs">⌄</span>
                    </div>
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
                className="flex items-center justify-between cursor-pointer"
              >
                <span className="font-semibold text-slate-900 text-xs">Macros</span>
                <span className="text-slate-400 font-bold">{showMacrosAccordion ? '−' : '+'}</span>
              </div>
              {showMacrosAccordion && (
                <div className="mt-2 space-y-1 text-xs">
                  <button
                    onClick={() => setReplyText('I have reset your password and sent a temporary link to your registered email.')}
                    className="w-full text-left rounded p-1.5 hover:bg-slate-50 text-slate-700"
                  >
                    ⚡ Send Password Reset Link
                  </button>
                  <button
                    onClick={() => setReplyText('Could you please try opening an Incognito window and let me know if it resolves?')}
                    className="w-full text-left rounded p-1.5 hover:bg-slate-50 text-slate-700"
                  >
                    ⚡ Request Incognito Verification
                  </button>
                  <button
                    onClick={() => setReplyText('I am escalating this ticket to our senior Tier 2 engineering team for immediate review.')}
                    className="w-full text-left rounded p-1.5 hover:bg-slate-50 text-slate-700"
                  >
                    ⚡ Escalate to Tier 2 Engineering
                  </button>
                </div>
              )}
            </div>
          </div>
        </aside>
      )}
    </div>
  )
}
