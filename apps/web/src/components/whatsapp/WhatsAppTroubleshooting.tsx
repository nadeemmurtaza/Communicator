import { useState } from 'react'
import { AlertTriangle, CheckCircle2, XCircle, Info, ChevronDown, ChevronUp, RefreshCw, ShieldAlert, MessageSquare, Clock, Settings, FileText, ExternalLink } from '../../components/icons'

// ---------------------------------------------------------------------------
// Error code database — comprehensive WhatsApp Cloud API error reference
// ---------------------------------------------------------------------------
interface ErrorCode {
  code: string
  title: string
  severity: 'critical' | 'warning' | 'info'
  category: string
  description: string
  cause: string
  resolution: string[]
  prevention: string
}

const ERROR_CODES: ErrorCode[] = [
  // --- Connection Errors ---
  {
    code: '33',
    title: 'Access Token Invalid',
    severity: 'critical',
    category: 'Connection',
    description: 'The access token provided is invalid, expired, or revoked.',
    cause: 'The temporary access token has expired (valid for 24 hours) or the permanent token was revoked.',
    resolution: [
      'Generate a new temporary access token from Meta Business Manager',
      'Exchange the temporary token for a permanent token via the token exchange API',
      'Update the access token in WhatsAppIntegration > Meta Credentials tab',
      'Verify the token using the "Verify Connection" button',
    ],
    prevention: 'Use permanent (long-lived) tokens that expire in 60 days and set up automatic token refresh.',
  },
  {
    code: '34',
    title: 'Access Token Expired',
    severity: 'critical',
    category: 'Connection',
    description: 'The access token has expired and is no longer valid.',
    cause: 'Temporary tokens expire after 24 hours; long-lived tokens expire after 60 days.',
    resolution: [
      'Generate a new long-lived access token from Meta Business Manager',
      'Update the token in the Meta Credentials tab',
      'Re-verify the connection after updating',
    ],
    prevention: 'Set up a reminder to refresh tokens before they expire. Consider implementing automatic token refresh.',
  },
  {
    code: '39',
    title: 'Insufficient Privileges',
    severity: 'critical',
    category: 'Connection',
    description: 'The access token does not have the required permissions.',
    cause: 'The token was generated without the `whatsapp_cloud_api` permission scope.',
    resolution: [
      'Re-generate the access token with the `whatsapp_cloud_api` permission',
      'Ensure the Facebook app has the WhatsApp product added',
      'Verify the user has admin access to the WhatsApp Business Account',
    ],
    prevention: 'Always request the full `whatsapp_cloud_api` permission scope when generating tokens.',
  },
  {
    code: '100',
    title: 'Invalid Phone Number ID',
    severity: 'critical',
    category: 'Connection',
    description: 'The phone number ID is invalid or does not belong to the WhatsApp Business Account.',
    cause: 'The phone number ID was copied incorrectly or belongs to a different WABA.',
    resolution: [
      'Copy the correct phone number ID from Meta Business Manager > WhatsApp > Settings',
      'Ensure the phone number is linked to the same WABA ID configured in credentials',
      'Update the phone number ID in the Meta Credentials tab',
    ],
    prevention: 'Double-check the phone number ID against the Meta Business Manager dashboard.',
  },
  {
    code: '104',
    title: 'WABA ID Mismatch',
    severity: 'critical',
    category: 'Connection',
    description: 'The WhatsApp Business Account ID does not match the phone number or access token.',
    cause: 'The WABA ID, phone number ID, and access token must all belong to the same Meta account.',
    resolution: [
      'Verify all three credentials (WABA ID, phone number ID, access token) are from the same Meta account',
      'Check Meta Business Manager for the correct WABA ID',
      'Update all credentials consistently in the Meta Credentials tab',
    ],
    prevention: 'Keep a record of all credential IDs and verify they match before configuration.',
  },

  // --- Message Sending Errors ---
  {
    code: '1001',
    title: 'Message Too Long',
    severity: 'warning',
    category: 'Messaging',
    description: 'The message body exceeds the maximum length of 1024 characters.',
    cause: 'WhatsApp Cloud API limits message bodies to 1024 characters.',
    resolution: [
      'Shorten the message body to 1024 characters or less',
      'Split the message into multiple messages if needed',
      'Use a template for longer, structured messages',
    ],
    prevention: 'Add character count validation in the message composer before sending.',
  },
  {
    code: '1003',
    title: 'Invalid Recipient',
    severity: 'critical',
    category: 'Messaging',
    description: 'The recipient phone number is invalid or not registered on WhatsApp.',
    cause: 'The phone number format is incorrect or the number is not a WhatsApp user.',
    resolution: [
      'Ensure the phone number includes the country code (e.g., +14155551234)',
      'Remove any formatting characters (spaces, dashes, parentheses)',
      'Verify the recipient has a WhatsApp account',
    ],
    prevention: 'Validate phone number format before sending. Use E.164 format consistently.',
  },
  {
    code: '1004',
    title: 'Template Not Found',
    severity: 'critical',
    category: 'Templates',
    description: 'The specified template name or ID does not exist or is not approved.',
    cause: 'The template was deleted, rejected, or the name/ID is incorrect.',
    resolution: [
      'Check the template status in the Templates tab — it must be APPROVED',
      'Verify the template name matches exactly (case-sensitive)',
      'Sync templates from Meta to refresh the local cache',
      'Create a new template if the original was deleted',
    ],
    prevention: 'Regularly sync templates from Meta. Monitor template approval status.',
  },
  {
    code: '1010',
    title: 'Template Variables Missing',
    severity: 'warning',
    category: 'Templates',
    description: 'The template requires variables but none were provided.',
    cause: 'The template contains {{1}}, {{2}} placeholders but no variable values were sent.',
    resolution: [
      'Check the template variables in the Templates tab',
      'Provide values for all required variables when sending',
      'Ensure the number of variables matches the template definition',
    ],
    prevention: 'Always check template variable requirements before sending.',
  },
  {
    code: '130019',
    title: '24-Hour Window Closed',
    severity: 'warning',
    category: 'Messaging',
    description: 'Cannot send free-text message — the 24-hour customer service window has closed.',
    cause: 'The customer has not replied for more than 24 hours. WhatsApp requires using an approved template to re-open the conversation.',
    resolution: [
      'Use the "Choose Template" button to send an approved template message',
      'Wait for the customer to reply, which re-opens the 24-hour window',
      'Check the 24-hour window indicator in the conversation for status',
    ],
    prevention: 'Monitor the 24-hour window indicator. Send templates proactively before the window closes.',
  },
  {
    code: '130020',
    title: 'Marketing Template Blocked',
    severity: 'critical',
    category: 'Messaging',
    description: 'Marketing templates cannot be sent to US phone numbers.',
    cause: 'Meta blocks marketing templates to US (+1) phone numbers due to regulatory requirements.',
    resolution: [
      'Use utility or authentication templates instead for US numbers',
      'For marketing to US numbers, use a different channel (email, SMS)',
      'Contact Meta support for special permissions if needed',
    ],
    prevention: 'Check recipient country code before sending marketing templates. Avoid marketing templates for US numbers.',
  },
  {
    code: '130021',
    title: 'Messaging Limit Exceeded',
    severity: 'critical',
    category: 'Messaging',
    description: 'The current messaging tier limit has been exceeded.',
    cause: 'The WhatsApp Business Account has reached the conversation limit for the current tier (100, 1K, 10K, or 100K).',
    resolution: [
      'Check the current messaging tier in the Quality Dashboard',
      'Wait for the limit to reset (monthly cycle)',
      'Improve quality rating to increase the messaging tier',
      'Contact Meta to request a tier increase',
    ],
    prevention: 'Monitor messaging limits in the Quality Dashboard. Maintain high quality rating to increase limits.',
  },
  {
    code: '130022',
    title: 'Template Rejected',
    severity: 'warning',
    category: 'Templates',
    description: 'The template was rejected by Meta and cannot be used.',
    cause: 'Meta rejected the template for violating WhatsApp policies (spam, misleading content, etc.).',
    resolution: [
      'Check the rejection reason in Meta Business Manager',
      'Modify the template to comply with WhatsApp policies',
      'Re-submit the template for approval',
      'Use an existing approved template instead',
    ],
    prevention: 'Review WhatsApp template policies before creating templates. Avoid spam-like content.',
  },
  {
    code: '130023',
    title: 'Phone Number Blocked',
    severity: 'critical',
    category: 'Connection',
    description: 'The phone number has been temporarily or permanently blocked by Meta.',
    cause: 'The phone number violated WhatsApp policies (spam reports, blocked by users, etc.).',
    resolution: [
      'Check the quality rating — a Red rating indicates blocking risk',
      'Review block reasons in the Quality Dashboard',
      'Appeal the block via Meta Business Manager if appropriate',
      'Use a different phone number if permanently blocked',
    ],
    prevention: 'Maintain high quality rating. Monitor spam flags and block reasons. Follow WhatsApp policies.',
  },

  // --- Webhook Errors ---
  {
    code: '20001',
    title: 'Webhook Verification Failed',
    severity: 'critical',
    category: 'Webhooks',
    description: 'Meta cannot verify the webhook endpoint.',
    cause: 'The webhook URL is inaccessible, the verify token does not match, or the endpoint is not responding correctly.',
    resolution: [
      'Verify the webhook URL is publicly accessible (not localhost)',
      'Check the verify token matches in both Meta and the Webhooks tab',
      'Ensure the backend server is running and responding to GET requests',
      'Check server logs for webhook verification errors',
    ],
    prevention: 'Use a stable public URL. Test webhook accessibility before configuration.',
  },
  {
    code: '20002',
    title: 'Webhook Delivery Failed',
    severity: 'warning',
    category: 'Webhooks',
    description: 'Meta sent an event but the webhook endpoint returned an error.',
    cause: 'The server returned a non-200 response, timed out, or is unreachable.',
    resolution: [
      'Check server logs for webhook processing errors',
      'Ensure the server responds with 200 OK within 200ms',
      'Verify the webhook signature validation is working',
      'Check the Live Events stream in Diagnostics for errors',
    ],
    prevention: 'Monitor webhook health. Ensure server responds quickly to webhook events.',
  },
  {
    code: '20003',
    title: 'Webhook Signature Invalid',
    severity: 'critical',
    category: 'Webhooks',
    description: 'The webhook signature does not match the expected signature.',
    cause: 'The app secret (META_APP_SECRET) does not match the one configured in Meta.',
    resolution: [
      'Verify the app secret in Meta Business Manager matches the META_APP_SECRET environment variable',
      'Check for typos in the app secret',
      'Re-copy the app secret from Meta if it was regenerated',
    ],
    prevention: 'Store the app secret securely. Verify it matches after any Meta account changes.',
  },

  // --- Media Errors ---
  {
    code: '30001',
    title: 'Invalid Media URL',
    severity: 'warning',
    category: 'Media',
    description: 'The media URL is invalid or inaccessible.',
    cause: 'The media file URL is not publicly accessible or the file does not exist.',
    resolution: [
      'Ensure the media URL is publicly accessible (HTTPS)',
      'Verify the file exists at the specified URL',
      'Use a CDN or cloud storage service for media files',
    ],
    prevention: 'Host media files on a reliable, publicly accessible server.',
  },
  {
    code: '30002',
    title: 'Media Type Not Supported',
    severity: 'warning',
    category: 'Media',
    description: 'The media file type is not supported by WhatsApp.',
    cause: 'WhatsApp supports specific file types: images (jpg, png, gif), videos (mp4), documents (pdf, doc, etc.), audio (mp3, ogg).',
    resolution: [
      'Convert the file to a supported format',
      'Check the file extension matches the content type',
      'Use the correct media type in the API request',
    ],
    prevention: 'Validate file types before uploading. Use supported formats.',
  },
  {
    code: '30003',
    title: 'Media Size Exceeded',
    severity: 'warning',
    category: 'Media',
    description: 'The media file exceeds WhatsApp size limits.',
    cause: 'WhatsApp limits: images 16MB, videos 16MB, documents 100MB, audio 16MB.',
    resolution: [
      'Compress the media file to reduce size',
      'Use a shorter video or lower resolution',
      'Split large documents into smaller files',
    ],
    prevention: 'Check file sizes before sending. Compress media proactively.',
  },

  // --- Quality & Policy Errors ---
  {
    code: '40001',
    title: 'Quality Rating Dropped',
    severity: 'warning',
    category: 'Quality',
    description: 'The quality rating has dropped from Green to Yellow or Red.',
    cause: 'Users are blocking messages, reporting spam, or the account has high bounce rates.',
    resolution: [
      'Check block reasons in the Quality Dashboard',
      'Review recent messages for policy violations',
      'Improve message quality and relevance',
      'Reduce the frequency of unsolicited messages',
    ],
    prevention: 'Monitor quality metrics regularly. Maintain user opt-in lists. Follow WhatsApp policies.',
  },
  {
    code: '40002',
    title: 'Spam Flag Detected',
    severity: 'critical',
    category: 'Quality',
    description: 'Meta detected spam-like behavior from the account.',
    cause: 'High volume of similar messages, user reports, or blocked messages.',
    resolution: [
      'Stop sending messages immediately',
      'Review the Quality Dashboard for spam flag details',
      'Modify messaging strategy to avoid spam patterns',
      'Wait for the spam flag to clear (usually 24-48 hours)',
    ],
    prevention: 'Vary message content. Send only to opted-in contacts. Monitor spam flags.',
  },
  {
    code: '40003',
    title: 'Business Number Verified Name Rejected',
    severity: 'warning',
    category: 'Quality',
    description: 'The verified business name was rejected by Meta.',
    cause: 'The name violates Meta naming policies (misleading, trademark infringement, etc.).',
    resolution: [
      'Check the rejection reason in Meta Business Manager',
      'Modify the business name to comply with policies',
      'Re-submit the name for verification',
    ],
    prevention: 'Review Meta naming policies before submitting a business name.',
  },

  // --- Campaign Errors ---
  {
    code: '50001',
    title: 'Campaign Send Failed',
    severity: 'critical',
    category: 'Campaigns',
    description: 'The campaign failed to send messages to one or more contacts.',
    cause: 'Invalid phone numbers, template issues, or API rate limits.',
    resolution: [
      'Check campaign analytics for failure details',
      'Validate contact phone numbers before adding to campaigns',
      'Use the retry button to resend failed messages',
      'Check the campaign retry queue for failed messages',
    ],
    prevention: 'Validate contacts before campaign creation. Monitor campaign analytics.',
  },
  {
    code: '50002',
    title: 'Campaign Rate Limited',
    severity: 'warning',
    category: 'Campaigns',
    description: 'The campaign is being throttled due to API rate limits.',
    cause: 'WhatsApp Cloud API has rate limits on message sending frequency.',
    resolution: [
      'Wait for the rate limit to reset',
      'Reduce the campaign send rate',
      'Schedule campaigns during off-peak hours',
    ],
    prevention: 'Space out campaign sends. Use scheduled sending for large campaigns.',
  },
]

// ---------------------------------------------------------------------------
// Category filter options
// ---------------------------------------------------------------------------
const CATEGORIES = ['All', 'Connection', 'Messaging', 'Templates', 'Webhooks', 'Media', 'Quality', 'Campaigns']

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------
interface Props {
  connected: boolean
}

export function WhatsAppTroubleshooting({ connected }: Props) {
  const [search, setSearch] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('All')
  const [expandedCode, setExpandedCode] = useState<string | null>(null)

  // Filter error codes
  const filtered = ERROR_CODES.filter((e) => {
    const matchesSearch = !search.trim() ||
      e.code.includes(search) ||
      e.title.toLowerCase().includes(search.toLowerCase()) ||
      e.description.toLowerCase().includes(search.toLowerCase())
    const matchesCategory = selectedCategory === 'All' || e.category === selectedCategory
    return matchesSearch && matchesCategory
  })

  function severityColor(severity: string) {
    switch (severity) {
      case 'critical': return 'text-red-600 bg-red-50 border-red-200'
      case 'warning': return 'text-amber-600 bg-amber-50 border-amber-200'
      case 'info': return 'text-blue-600 bg-blue-50 border-blue-200'
      default: return 'text-[var(--color-ink)] bg-[var(--color-page)] border-[var(--color-border)]'
    }
  }

  function severityIcon(severity: string) {
    switch (severity) {
      case 'critical': return <XCircle className="h-4 w-4" />
      case 'warning': return <AlertTriangle className="h-4 w-4" />
      case 'info': return <Info className="h-4 w-4" />
    }
  }

  function categoryIcon(category: string) {
    switch (category) {
      case 'Connection': return <Settings className="h-4 w-4" />
      case 'Messaging': return <MessageSquare className="h-4 w-4" />
      case 'Templates': return <FileText className="h-4 w-4" />
      case 'Webhooks': return <RefreshCw className="h-4 w-4" />
      case 'Media': return <ShieldAlert className="h-4 w-4" />
      case 'Quality': return <AlertTriangle className="h-4 w-4" />
      case 'Campaigns': return <Clock className="h-4 w-4" />
      default: return <Info className="h-4 w-4" />
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h3 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
          <ShieldAlert className="h-5 w-5 text-red-600" />
          Message Failure Troubleshooting
        </h3>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
          Reference guide for WhatsApp Cloud API error codes, causes, and resolution steps.
        </p>
      </div>

      {/* Quick stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div className="rounded-[var(--radius-lg)] border border-red-200 bg-red-50 p-4">
          <p className="text-2xl font-bold text-red-700">
            {ERROR_CODES.filter((e) => e.severity === 'critical').length}
          </p>
          <p className="text-xs font-semibold text-red-600">Critical Errors</p>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-amber-200 bg-amber-50 p-4">
          <p className="text-2xl font-bold text-amber-700">
            {ERROR_CODES.filter((e) => e.severity === 'warning').length}
          </p>
          <p className="text-xs font-semibold text-amber-600">Warnings</p>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-blue-200 bg-blue-50 p-4">
          <p className="text-2xl font-bold text-blue-700">
            {ERROR_CODES.filter((e) => e.severity === 'info').length}
          </p>
          <p className="text-xs font-semibold text-blue-600">Info</p>
        </div>
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-page)] p-4">
          <p className="text-2xl font-bold text-[var(--color-ink)]">{ERROR_CODES.length}</p>
          <p className="text-xs font-semibold text-[var(--color-ink-muted)]">Total Errors</p>
        </div>
      </div>

      {/* Search & Filter */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by error code, title, or description..."
            className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2.5 text-sm focus:border-[var(--color-accent)] focus:outline-hidden"
          />
        </div>
        <div className="flex gap-2 flex-wrap">
          {CATEGORIES.map((cat) => (
            <button
              key={cat}
              onClick={() => setSelectedCategory(cat)}
              className={`rounded-[var(--radius-md)] px-3 py-2 text-xs font-semibold transition-colors ${
                selectedCategory === cat
                  ? 'bg-[var(--color-accent)] text-white'
                  : 'border border-[var(--color-border)] text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)]'
              }`}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      {/* Error code list */}
      {filtered.length === 0 ? (
        <div className="rounded-[var(--radius-lg)] border-2 border-dashed border-[var(--color-border)] p-8 text-center">
          <CheckCircle2 className="h-10 w-10 text-emerald-500 mx-auto" />
          <p className="mt-3 text-sm font-semibold text-[var(--color-ink)]">No errors found</p>
          <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
            Try adjusting your search or filter criteria.
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((error) => (
            <div
              key={error.code}
              className="rounded-[var(--radius-lg)] border border-[var(--color-border)] overflow-hidden transition-colors"
            >
              {/* Error header — always visible */}
              <button
                onClick={() => setExpandedCode(expandedCode === error.code ? null : error.code)}
                className="w-full flex items-center gap-3 p-4 text-left hover:bg-[var(--color-accent-muted)] transition-colors"
              >
                {/* Severity badge */}
                <span className={`shrink-0 rounded-full p-1.5 ${severityColor(error.severity)} border`}>
                  {severityIcon(error.severity)}
                </span>

                {/* Error code */}
                <code className="shrink-0 rounded-[var(--radius-sm)] bg-[var(--color-page)] border border-[var(--color-border)] px-2 py-0.5 text-xs font-mono font-bold text-[var(--color-ink)]">
                  {error.code}
                </code>

                {/* Title */}
                <span className="flex-1 text-sm font-semibold text-[var(--color-ink)]">
                  {error.title}
                </span>

                {/* Category */}
                <span className="shrink-0 flex items-center gap-1 rounded-full bg-[var(--color-page)] border border-[var(--color-border)] px-2.5 py-1 text-[11px] font-semibold text-[var(--color-ink-muted)]">
                  {categoryIcon(error.category)}
                  {error.category}
                </span>

                {/* Expand/collapse icon */}
                {expandedCode === error.code ? (
                  <ChevronUp className="h-4 w-4 text-[var(--color-ink-faint)] shrink-0" />
                ) : (
                  <ChevronDown className="h-4 w-4 text-[var(--color-ink-faint)] shrink-0" />
                )}
              </button>

              {/* Expanded details */}
              {expandedCode === error.code && (
                <div className="border-t border-[var(--color-border)] bg-[var(--color-page)]/50 p-4 space-y-4">
                  {/* Description */}
                  <div>
                    <p className="text-xs font-bold text-[var(--color-ink)] mb-1">Description</p>
                    <p className="text-sm text-[var(--color-ink-muted)]">{error.description}</p>
                  </div>

                  {/* Cause */}
                  <div>
                    <p className="text-xs font-bold text-[var(--color-ink)] mb-1">Cause</p>
                    <p className="text-sm text-[var(--color-ink-muted)]">{error.cause}</p>
                  </div>

                  {/* Resolution steps */}
                  <div>
                    <p className="text-xs font-bold text-[var(--color-ink)] mb-2">Resolution Steps</p>
                    <ol className="space-y-1.5">
                      {error.resolution.map((step, i) => (
                        <li key={i} className="flex items-start gap-2 text-sm text-[var(--color-ink-muted)]">
                          <span className="mt-0.5 shrink-0 h-5 w-5 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center text-[10px] font-bold">
                            {i + 1}
                          </span>
                          <span>{step}</span>
                        </li>
                      ))}
                    </ol>
                  </div>

                  {/* Prevention */}
                  <div className="rounded-[var(--radius-md)] border border-emerald-200 bg-emerald-50 p-3">
                    <p className="text-xs font-bold text-emerald-800 mb-1">Prevention</p>
                    <p className="text-xs text-emerald-700">{error.prevention}</p>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* General troubleshooting tips */}
      <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-5 space-y-3">
        <p className="text-sm font-bold text-[var(--color-ink)]">General Troubleshooting Tips</p>
        <ul className="space-y-2 text-xs text-[var(--color-ink-muted)]">
          <li className="flex items-start gap-2">
            <Info className="h-3.5 w-3.5 text-[var(--color-accent)] mt-0.5 shrink-0" />
            <span><strong>Check Connection First:</strong> Most errors stem from invalid or expired credentials. Verify your connection in the Meta Credentials tab.</span>
          </li>
          <li className="flex items-start gap-2">
            <Info className="h-3.5 w-3.5 text-[var(--color-accent)] mt-0.5 shrink-0" />
            <span><strong>Monitor Quality:</strong> A low quality rating can cause message delivery failures. Check the Quality Dashboard regularly.</span>
          </li>
          <li className="flex items-start gap-2">
            <Info className="h-3.5 w-3.5 text-[var(--color-accent)] mt-0.5 shrink-0" />
            <span><strong>Review Webhook Events:</strong> The Live Events stream in Diagnostics shows real-time webhook events and errors.</span>
          </li>
          <li className="flex items-start gap-2">
            <Info className="h-3.5 w-3.5 text-[var(--color-accent)] mt-0.5 shrink-0" />
            <span><strong>Test Before Sending:</strong> Use the Live Test & Diagnostics tab to test messages before sending to customers.</span>
          </li>
          <li className="flex items-start gap-2">
            <Info className="h-3.5 w-3.5 text-[var(--color-accent)] mt-0.5 shrink-0" />
            <span><strong>Meta Documentation:</strong> For detailed API error codes, refer to Meta's official WhatsApp Cloud API documentation.</span>
          </li>
        </ul>
      </div>

      {/* Meta documentation link */}
      <a
        href="https://developers.facebook.com/docs/whatsapp/cloud-api/support/error-codes"
        target="_blank"
        rel="noopener noreferrer"
        className="flex items-center justify-center gap-2 rounded-[var(--radius-md)] border border-[var(--color-border)] px-4 py-3 text-sm font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] transition-colors"
      >
        <ExternalLink className="h-4 w-4" />
        View Meta's Official Error Codes
      </a>
    </div>
  )
}
