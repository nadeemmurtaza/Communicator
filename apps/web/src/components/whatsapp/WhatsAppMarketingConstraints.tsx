import { useState } from 'react'
import { AlertTriangle, CheckCircle2, XCircle, Info, ShieldAlert, MessageSquare, Globe, Users, Clock, Ban, FileText } from '../../components/icons'

// ---------------------------------------------------------------------------
// Marketing Template Constraints — Intercom parity
// Intercom shows these constraints in the WhatsApp channel settings to warn
// about Meta's marketing template restrictions (US blocking, opt-in, caps).
// ---------------------------------------------------------------------------

interface Constraint {
  id: string
  title: string
  severity: 'critical' | 'warning' | 'info'
  icon: 'ban' | 'shield' | 'globe' | 'users' | 'clock' | 'file'
  description: string
  details: string[]
  mitigation: string
}

const CONSTRAINTS: Constraint[] = [
  {
    id: 'us-blocking',
    title: 'US Phone Numbers Blocked for Marketing',
    severity: 'critical',
    icon: 'ban',
    description: 'Meta blocks marketing templates from being sent to US (+1) phone numbers.',
    details: [
      'Marketing category templates cannot be delivered to recipients with US phone numbers',
      'This is a Meta-enforced restriction due to US regulatory requirements (TCPA, CAN-SPAM)',
      'Attempting to send will result in error code 130020: Marketing Template Blocked',
      'The restriction applies to all US area codes (+1 prefix)',
    ],
    mitigation: 'Use Utility or Authentication templates for US numbers. For marketing to US audiences, use email or SMS channels instead.',
  },
  {
    id: 'opt-in-required',
    title: 'Explicit Opt-In Required',
    severity: 'critical',
    icon: 'users',
    description: 'WhatsApp requires explicit opt-in consent before sending any business-initiated messages.',
    details: [
      'Customers must explicitly opt in to receive messages from your business',
      'Opt-in can be via checkbox, button click, or written confirmation',
      'You must be able to prove opt-in consent if Meta requests evidence',
      'Marketing templates require an opt-in disclaimer in the template text',
      'Missing opt-in can result in phone number blocking or permanent ban',
    ],
    mitigation: 'Implement opt-in checkboxes on forms, landing pages, and checkout flows. Store opt-in timestamps and channels for audit purposes.',
  },
  {
    id: 'frequency-caps',
    title: 'Marketing Message Frequency Caps',
    severity: 'warning',
    icon: 'clock',
    description: 'Meta enforces frequency limits on marketing messages to prevent spam.',
    details: [
      'A customer can receive at most 4 marketing messages per 24-hour period from a single business',
      'This limit is per customer, per business (WABA), not per phone number',
      'Exceeding the cap results in error code 130020 and potential quality rating impact',
      'The 24-hour window resets based on the customer\'s local timezone',
      'Utility and authentication messages are NOT subject to this cap',
    ],
    mitigation: 'Track sent marketing messages per customer. Implement a cooldown period between marketing sends. Use utility templates for transactional communications.',
  },
  {
    id: 'template-disclaimer',
    title: 'Marketing Template Disclaimer',
    severity: 'warning',
    icon: 'file',
    description: 'Marketing templates must include an opt-out disclaimer in the template body.',
    details: [
      'Meta requires marketing templates to include language like "Reply STOP to unsubscribe"',
      'Templates without proper disclaimer may be rejected during approval',
      'The disclaimer should be at the end of the template body text',
      'This helps customers understand they can opt out of marketing messages',
    ],
    mitigation: 'Always include "Reply STOP to unsubscribe" or similar language in marketing template body text before submitting for approval.',
  },
  {
    id: 'country-restrictions',
    title: 'Country-Specific Restrictions',
    severity: 'warning',
    icon: 'globe',
    description: 'Some countries have additional restrictions on marketing messages via WhatsApp.',
    details: [
      'US (+1): Marketing templates blocked entirely',
      'EU countries: GDPR compliance required — explicit consent and data processing notice',
      'India (+91): DLT (Distributed Ledger Technology) registration required for business messages',
      'Brazil (+55): ANPD compliance for data protection',
      'Other countries may have specific telecom regulations',
    ],
    mitigation: 'Research local regulations before sending marketing messages. Use utility templates where marketing is restricted. Consult legal counsel for compliance.',
  },
  {
    id: 'quality-impact',
    title: 'Quality Rating Impact',
    severity: 'warning',
    icon: 'shield',
    description: 'Marketing messages can significantly impact your WhatsApp quality rating.',
    details: [
      'High block rates on marketing messages lower your quality rating',
      'A declining quality rating reduces your messaging tier (100K → 10K → 1K → 100)',
      'Red quality rating can result in temporary or permanent phone number blocking',
      'Marketing messages have higher block rates than utility or authentication messages',
      'Monitor block reasons in the Quality Dashboard tab regularly',
    ],
    mitigation: 'Segment your audience and only send marketing to engaged contacts. Monitor block rates and pause campaigns with high block rates. Maintain high quality rating by following best practices.',
  },
  {
    id: 'best-practices',
    title: 'Marketing Best Practices',
    severity: 'info',
    icon: 'info',
    description: 'Follow these best practices to maximize marketing template delivery and maintain quality.',
    details: [
      'Personalize templates using variables ({{1}}, {{2}}) to increase engagement',
      'Send at optimal times based on customer timezone and past engagement',
      'Segment audiences by engagement level — prioritize active customers',
      'A/B test template content to improve open and reply rates',
      'Monitor campaign analytics for delivery, read, and reply rates',
      'Honor opt-out requests immediately — reply STOP should unsubscribe',
      'Keep templates concise and value-driven to reduce block rates',
    ],
    mitigation: 'Use the Cost Tracking and Quality Dashboard tabs to monitor performance. Review analytics regularly and adjust strategy based on data.',
  },
]

function SeverityBadge({ severity }: { severity: 'critical' | 'warning' | 'info' }) {
  const config = {
    critical: 'bg-red-100 text-red-800',
    warning: 'bg-amber-100 text-amber-800',
    info: 'bg-blue-100 text-blue-800',
  }
  const iconConfig = {
    critical: <XCircle className="h-3.5 w-3.5" />,
    warning: <AlertTriangle className="h-3.5 w-3.5" />,
    info: <Info className="h-3.5 w-3.5" />,
  }
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold uppercase ${config[severity]}`}>
      {iconConfig[severity]}
      {severity}
    </span>
  )
}

function IconDisplay({ icon }: { icon: string }) {
  const iconMap: Record<string, JSX.Element> = {
    ban: <Ban className="h-5 w-5 text-red-600" />,
    shield: <ShieldAlert className="h-5 w-5 text-amber-600" />,
    globe: <Globe className="h-5 w-5 text-blue-600" />,
    users: <Users className="h-5 w-5 text-purple-600" />,
    clock: <Clock className="h-5 w-5 text-amber-600" />,
    file: <FileText className="h-5 w-5 text-indigo-600" />,
    info: <Info className="h-5 w-5 text-blue-600" />,
  }
  return <span className="shrink-0">{iconMap[icon] || <Info className="h-5 w-5 text-blue-600" />}</span>
}

export function WhatsAppMarketingConstraints() {
  const [expandedId, setExpandedId] = useState<string | null>(null)

  const criticalCount = CONSTRAINTS.filter((c) => c.severity === 'critical').length
  const warningCount = CONSTRAINTS.filter((c) => c.severity === 'warning').length
  const infoCount = CONSTRAINTS.filter((c) => c.severity === 'info').length

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h3 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
          <ShieldAlert className="h-5 w-5 text-red-600" />
          Marketing Template Constraints
        </h3>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
          Meta enforces strict rules on marketing templates. Violating these constraints can result in message delivery failures, quality rating degradation, or phone number blocking.
        </p>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-3 gap-3">
        <div className="rounded-[var(--radius-lg)] border border-red-200 bg-red-50/50 p-4">
          <div className="flex items-center gap-2">
            <XCircle className="h-5 w-5 text-red-600" />
            <span className="text-sm font-bold text-red-800">{criticalCount} Critical</span>
          </div>
          <p className="mt-1 text-xs text-red-600">Must comply or messages will fail</p>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-amber-200 bg-amber-50/50 p-4">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-amber-600" />
            <span className="text-sm font-bold text-amber-800">{warningCount} Warnings</span>
          </div>
          <p className="mt-1 text-xs text-amber-600">Risk of quality impact if ignored</p>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-blue-200 bg-blue-50/50 p-4">
          <div className="flex items-center gap-2">
            <Info className="h-5 w-5 text-blue-600" />
            <span className="text-sm font-bold text-blue-800">{infoCount} Info</span>
          </div>
          <p className="mt-1 text-xs text-blue-600">Best practices for success</p>
        </div>
      </div>

      {/* Critical Warning Banner */}
      <div className="rounded-[var(--radius-lg)] border-2 border-red-300 bg-red-50 p-5">
        <div className="flex items-start gap-3">
          <Ban className="h-6 w-6 text-red-600 shrink-0 mt-0.5" />
          <div>
            <h4 className="text-sm font-bold text-red-800">US Marketing Templates Are Blocked</h4>
            <p className="mt-1 text-xs text-red-700">
              Meta <strong>completely blocks</strong> marketing category templates from being sent to US (+1) phone numbers. This is not a suggestion — it is enforced at the API level.
              Attempting to send will result in error code <code className="bg-red-100 px-1 py-0.5 rounded font-mono">130020</code>.
              Use <strong>Utility</strong> or <strong>Authentication</strong> templates for US recipients, or switch to email/SMS for marketing to US audiences.
            </p>
          </div>
        </div>
      </div>

      {/* Constraint Cards */}
      <div className="space-y-3">
        {CONSTRAINTS.map((constraint) => {
          const isExpanded = expandedId === constraint.id
          return (
            <div
              key={constraint.id}
              className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white shadow-none overflow-hidden"
            >
              {/* Card Header — always visible */}
              <button
                onClick={() => setExpandedId(isExpanded ? null : constraint.id)}
                className="w-full flex items-center justify-between p-4 hover:bg-[var(--color-accent-muted)]/30 transition-colors text-left"
              >
                <div className="flex items-center gap-3">
                  <IconDisplay icon={constraint.icon} />
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-bold text-[var(--color-ink)]">{constraint.title}</span>
                    <SeverityBadge severity={constraint.severity} />
                  </div>
                </div>
                <span className="text-[var(--color-ink-faint)] text-xs">
                  {isExpanded ? '▲' : '▼'}
                </span>
              </button>

              {/* Expanded Details */}
              {isExpanded && (
                <div className="border-t border-[var(--color-border)] p-4 space-y-4 bg-[var(--color-page)]">
                  <div>
                    <h5 className="text-xs font-bold text-[var(--color-ink)] mb-2">Details</h5>
                    <p className="text-xs text-[var(--color-ink-muted)] mb-3">{constraint.description}</p>
                    <ul className="space-y-1.5">
                      {constraint.details.map((detail, i) => (
                        <li key={i} className="flex items-start gap-2 text-xs text-[var(--color-ink-muted)]">
                          <span className="mt-1.5 h-1.5 w-1.5 rounded-full bg-[var(--color-accent)] shrink-0" />
                          {detail}
                        </li>
                      ))}
                    </ul>
                  </div>

                  <div className="rounded-[var(--radius-md)] border border-emerald-200 bg-emerald-50/50 p-3">
                    <h5 className="text-xs font-bold text-emerald-800 mb-1 flex items-center gap-1.5">
                      <CheckCircle2 className="h-3.5 w-3.5" />
                      Mitigation
                    </h5>
                    <p className="text-xs text-emerald-700">{constraint.mitigation}</p>
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>

      {/* Quick Reference */}
      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)]/40 p-5">
        <h4 className="text-sm font-bold text-[var(--color-ink)] mb-3 flex items-center gap-2">
          <MessageSquare className="h-4 w-4 text-[var(--color-accent)]" />
          Quick Reference: Template Categories
        </h4>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <div className="rounded-[var(--radius-md)] border border-red-200 bg-white p-3">
            <h5 className="text-xs font-bold text-red-700 mb-1">Marketing</h5>
            <p className="text-[11px] text-[var(--color-ink-muted)]">
              Promotional content, offers, announcements. <strong>Blocked for US numbers.</strong> Subject to frequency caps (4/24h). Requires opt-in disclaimer.
            </p>
          </div>
          <div className="rounded-[var(--radius-md)] border border-blue-200 bg-white p-3">
            <h5 className="text-xs font-bold text-blue-700 mb-1">Utility</h5>
            <p className="text-[11px] text-[var(--color-ink-muted)]">
              Transactional updates, order confirmations, shipping notifications. <strong>No US restriction.</strong> No frequency caps. Higher delivery priority.
            </p>
          </div>
          <div className="rounded-[var(--radius-md)] border border-purple-200 bg-white p-3">
            <h5 className="text-xs font-bold text-purple-700 mb-1">Authentication</h5>
            <p className="text-[11px] text-[var(--color-ink-muted)]">
              OTP codes, login verification, security alerts. <strong>No US restriction.</strong> No frequency caps. Highest delivery priority.
            </p>
          </div>
        </div>
      </div>

      {/* Meta Policy Link */}
      <a
        href="https://developers.facebook.com/docs/whatsapp/cloud-api/guidelines/template-usage"
        target="_blank"
        rel="noopener noreferrer"
        className="flex items-center justify-center gap-2 rounded-[var(--radius-md)] border border-[var(--color-border)] px-4 py-3 text-sm font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] transition-colors"
      >
        <Globe className="h-4 w-4" />
        Read Meta's Official Template Usage Policy
      </a>
    </div>
  )
}
