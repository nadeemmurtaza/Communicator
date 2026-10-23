/**
 * The product's three-tier navigation, in one place -- primary sections
 * (icon rail), their secondary items (labeled sidebar), and the labels
 * Breadcrumbs reads to build the trail. Adding a page is adding one row
 * here, not touching layout markup.
 */
import type { ComponentType, SVGProps } from 'react'
import {
  LayoutDashboard,
  Inbox,
  Users,
  Radio,
  Sparkles,
  BarChart3,
  Settings,
} from 'lucide-react'

export interface NavChild {
  to: string
  label: string
  end?: boolean
  channel?: string
}

export interface NavSection {
  id: string
  label: string
  to: string
  end?: boolean
  icon: ComponentType<SVGProps<SVGSVGElement>>
  children?: NavChild[]
}

export const NAV_SECTIONS: NavSection[] = [
  { id: 'dashboard', label: 'Dashboard', to: '/', end: true, icon: LayoutDashboard },
  {
    id: 'inbox',
    label: 'Inbox',
    to: '/conversations',
    icon: Inbox,
    children: [
      { to: '/conversations', label: 'Conversations' },
      { to: '/drafts', label: 'Draft review' },
      { to: '/tickets', label: 'Tickets' },
      { to: '/sla', label: 'SLA Monitoring' },
      { to: '/approvals', label: 'Approvals Queue' },
    ],
  },
  {
    id: 'audience',
    label: 'Audience',
    to: '/contacts',
    icon: Users,
    children: [
      { to: '/contacts', label: 'Contacts' },
      { to: '/broadcasts', label: 'Broadcasts' },
      { to: '/sequences', label: 'Sequences' },
      { to: '/automation', label: 'Automation' },
    ],
  },
  {
    id: 'channels',
    label: 'Channels',
    to: '/channels',
    icon: Radio,
    children: [
      { to: '/channels', label: 'All channels', end: true },
      { to: '/channels/whatsapp', label: 'WhatsApp', channel: 'whatsapp' },
      { to: '/channels/instagram', label: 'Instagram', channel: 'instagram' },
      { to: '/channels/facebook', label: 'Facebook', channel: 'facebook' },
      { to: '/channels/sms', label: 'SMS', channel: 'sms' },
      { to: '/channels/email', label: 'Email', channel: 'email' },
      { to: '/channels/voice', label: 'Voice', channel: 'voice' },
      { to: '/channels/telegram', label: 'Telegram', channel: 'telegram' },
      { to: '/channels/line', label: 'LINE', channel: 'line' },
      { to: '/channels/tiktok', label: 'TikTok', channel: 'tiktok' },
      { to: '/channels/live-chat', label: 'Live Chat', channel: 'live-chat' },
    ],
  },
  {
    id: 'intelligence',
    label: 'Intelligence',
    to: '/knowledge-base',
    icon: Sparkles,
    children: [
      { to: '/knowledge-base', label: 'Knowledge base' },
      { to: '/services', label: 'Service catalog' },
      { to: '/simulations', label: 'AI Simulations' },
      { to: '/self-improving', label: 'Self-Improving AI' },
    ],
  },
  { id: 'reports', label: 'Reports', to: '/reports', icon: BarChart3 },
  {
    id: 'admin',
    label: 'Admin',
    to: '/team',
    icon: Settings,
    children: [
      { to: '/team', label: 'Team' },
      { to: '/connections', label: 'Connections' },
      { to: '/webhooks', label: 'Webhooks' },
      { to: '/audit-logs', label: 'Audit logs' },
      { to: '/custom-attributes', label: 'CRM schema' },
      { to: '/routing-capacity', label: 'Routing & Capacity' },
      { to: '/macros', label: 'Macros' },
      { to: '/canned-responses', label: 'Canned Responses' },
    ],
  },
]

export function findActiveSection(pathname: string): NavSection | undefined {
  // Longest matching `to` wins, so /channels/instagram doesn't match /channels'
  // parent section over its own more specific one.
  const all = [...NAV_SECTIONS].sort((a, b) => b.to.length - a.to.length)
  return all.find((s) => (s.end ? pathname === s.to : pathname.startsWith(s.to)))
    ?? NAV_SECTIONS.flatMap((s) => (s.children ?? []).map((c) => ({ ...c, parent: s })))
      .sort((a, b) => b.to.length - a.to.length)
      .find((c) => (c.end ? pathname === c.to : pathname.startsWith(c.to)))?.parent
}

export function findActiveChild(section: NavSection | undefined, pathname: string): NavChild | undefined {
  if (!section?.children) return undefined
  const sorted = [...section.children].sort((a, b) => b.to.length - a.to.length)
  return sorted.find((c) => (c.end ? pathname === c.to : pathname.startsWith(c.to)))
}
