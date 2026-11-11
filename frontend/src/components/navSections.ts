/**
 * The product's three-tier navigation, in one place -- primary sections
 * (icon rail), their secondary items (labeled sidebar), and the labels
 * Breadcrumbs reads to build the trail. Adding a page is adding one row
 * here, not touching layout markup.
 *
 * Navigation structure (7 top-level sections):
 *  1. Dashboard    — overview and quick actions
 *  2. Workspace    — organization + brand hierarchy management
 *  3. Inbox        — conversations, drafts, tickets, SLA, approvals
 *  4. Audience     — contacts, clients, broadcasts, sequences, automation
 *  5. Channels     — all channel integrations
 *  6. Intelligence — knowledge base, services, AI, simulations
 *  7. Reports      — analytics and reports
 *  8. Settings     — team, connections, webhooks, audit, macros
 */
import type { ComponentType, SVGProps } from 'react'
import {
  LayoutDashboard,
  Rocket,
  Inbox,
  Users,
  Radio,
  Sparkles,
  BarChart3,
  Settings,
  Building2,
  ShoppingCart,
} from './icons'

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
    id: 'launchpad',
    label: 'Launchpad',
    to: '/launchpad',
    end: true,
    icon: Rocket,
  },
  {
    id: 'inbox',
    label: 'Inbox',
    to: '/conversations',
    icon: Inbox,
    children: [
      { to: '/conversations', label: 'Conversations' },
      { to: '/drafts', label: 'Draft Review' },
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
      { to: '/clients', label: 'Client Profiles' },
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
      { to: '/channels', label: 'All Channels', end: true },
      { to: '/channels/whatsapp', label: 'WhatsApp', channel: 'whatsapp' },
      { to: '/channels/whatsapp/campaigns', label: 'WhatsApp Campaigns', channel: 'whatsapp' },
      { to: '/channels/instagram', label: 'Instagram', channel: 'instagram' },
      { to: '/channels/facebook', label: 'Facebook', channel: 'facebook' },
      { to: '/channels/sms', label: 'SMS', channel: 'sms' },
      { to: '/channels/email', label: 'Email', channel: 'email' },
      { to: '/channels/voice', label: 'Voice', channel: 'voice' },
      { to: '/channels/telegram', label: 'Telegram', channel: 'telegram' },
      { to: '/channels/line', label: 'LINE', channel: 'line' },
      { to: '/channels/tiktok', label: 'TikTok', channel: 'tiktok' },
      { to: '/channels/discord', label: 'Discord', channel: 'discord' },
      { to: '/channels/slack', label: 'Slack', channel: 'slack' },
      { to: '/channels/live-chat', label: 'Live Chat', channel: 'live-chat' },
    ],
  },
  {
    id: 'intelligence',
    label: 'Intelligence',
    to: '/knowledge-base',
    icon: Sparkles,
    children: [
      { to: '/knowledge-base', label: 'Knowledge Base' },
      { to: '/services', label: 'Service Catalog' },
      { to: '/ai-agent', label: 'AI Agent' },
      { to: '/simulations', label: 'AI Simulations' },
      { to: '/self-improving', label: 'Self-Improving AI' },
      { to: '/topic-categorization', label: 'Topic Categorization' },
      { to: '/cx-qa', label: 'CX Score & QA' },
      { to: '/monitoring', label: 'Monitoring & Alerts' },
      { to: '/suggestions', label: 'AI Suggestions' },
      { to: '/csat', label: 'CSAT Surveys' },
      { to: '/operator', label: 'Fin Operator' },
      { to: '/procedures', label: 'Procedures & Actions' },
      { to: '/unified-inbox', label: 'Unified Inbox' },
      { to: '/workflows', label: 'Workflow Builder' },
       { to: '/fin-vision', label: 'Fin Vision' },
       { to: '/audience-kb', label: 'Audience KB Scoping' },
       { to: '/brand-overrides', label: 'Brand Overrides' },
       { to: '/mcp', label: 'MCP Protocol' },
       { to: '/autonomous-resolution', label: 'Autonomous Resolution' },
       { to: '/help-center', label: 'Help Center' },
      { to: '/crm-sync', label: 'CRM Sync' },
      { to: '/kb-import', label: 'KB Import' },
      { to: '/agent-performance', label: 'Agent Performance' },
      { to: '/notifications', label: 'Notifications' },
      { to: '/inapp-messaging', label: 'In-App Messaging' },
      { to: '/tone-of-voice', label: 'Tone of Voice' },
      { to: '/translation', label: 'Translation' },
      { to: '/embed-widget', label: 'Embed Widget' },
      { to: '/unified-analytics', label: 'Unified Analytics' },
      { to: '/reply-integration', label: 'Reply Integration' },
      { to: '/ab-testing', label: 'A/B Testing' },
      { to: '/fulfillment', label: 'Fulfillment' },
      { to: '/scheduled-tasks', label: 'Scheduled Tasks' },
      { to: '/compliance', label: 'Compliance & Privacy' },
      { to: '/conversation-search', label: 'Conversation Search' },
      { to: '/system-health', label: 'System Health & Onboarding' },
            { to: '/email-templates', label: 'Email Templates & SMTP' },
            { to: '/broadcast', label: 'Omni-Channel Broadcast' },
    ],
  },
  {
    id: 'workspace',
    label: 'Workspace',
    to: '/workspace/organization',
    icon: Building2,
    children: [
      { to: '/workspace/organization', label: 'Organization Profile' },
      { to: '/workspace/organization/keys', label: 'Organization Keys' },
      { to: '/workspace/brands', label: 'Brands' },
      { to: '/workspace/brand/profile', label: 'Brand Profile' },
      { to: '/workspace/brand/keys', label: 'Brand Keys (BYOK)' },
      { to: '/workspace/brand/ai-config', label: 'Brand AI Config' },
      { to: '/workspace/brand/knowledge', label: 'Brand Knowledge Base' },
    ],
  },
  {
    id: 'commerce',
    label: 'Commerce',
    to: '/commerce',
    icon: ShoppingCart,
    children: [
      { to: '/commerce', label: 'Dashboard', end: true },
      { to: '/commerce/catalog', label: 'Catalog' },
      { to: '/commerce/carts', label: 'Carts' },
      { to: '/commerce/orders', label: 'Orders' },
      { to: '/commerce/analytics', label: 'Analytics' },
      { to: '/commerce/coupons', label: 'Coupons' },
      { to: '/commerce/shipping', label: 'Shipping' },
      { to: '/commerce/inventory', label: 'Inventory' },
      { to: '/commerce/storefront', label: 'Storefront' },
      { to: '/commerce/sync', label: 'Sync Settings' },
    ],
  },
  {
    id: 'reports',
    label: 'Reports',
    to: '/reports',
    icon: BarChart3,
    children: [
      { to: '/reports', label: 'Overview' },
      { to: '/reports/analytics', label: 'Analytics' },
    ],
  },
  {
    id: 'settings',
    label: 'Settings',
    to: '/settings/team',
    icon: Settings,
    children: [
      { to: '/settings/team', label: 'Team' },
      { to: '/settings/connections', label: 'Connections' },
      { to: '/settings/webhooks', label: 'Webhooks' },
      { to: '/settings/audit-logs', label: 'Audit Logs' },
      { to: '/settings/custom-attributes', label: 'CRM Schema' },
      { to: '/settings/routing-capacity', label: 'Routing & Capacity' },
      { to: '/settings/macros', label: 'Macros' },
      { to: '/settings/canned-responses', label: 'Canned Responses' },
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
