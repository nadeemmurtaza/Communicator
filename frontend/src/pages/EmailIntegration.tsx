import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import type {
  EmailChannelData,
  EmailUpdatePayload,
  SocialEvent,
} from '../lib/api'

export function EmailIntegration() {
  const [activeTab, setActiveTab] = useState<'smtp' | 'imap' | 'templates' | 'automations' | 'logs'>('smtp')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [syncing, setSyncing] = useState(false)
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null)

  // Channel Data & Telemetry
  const [_data, setData] = useState<EmailChannelData | null>(null)
  const [events, setEvents] = useState<SocialEvent[]>([])

  // Form Fields: SMTP
  const [name, setName] = useState('Support Email Inbox')
  const [emailFromAddress, setEmailFromAddress] = useState('support@bayworks.co')
  const [emailFromName, setEmailFromName] = useState('Bay Works Support')
  const [emailReplyTo, setEmailReplyTo] = useState('support@bayworks.co')
  const [smtpHost, setSmtpHost] = useState('smtp.bayworks.co')
  const [smtpPort, setSmtpPort] = useState(587)
  const [smtpUsername, setSmtpUsername] = useState('support@bayworks.co')
  const [smtpPassword, setSmtpPassword] = useState('')
  const [smtpEncryption, setSmtpEncryption] = useState('tls')

  // Form Fields: IMAP
  const [imapHost, setImapHost] = useState('imap.bayworks.co')
  const [imapPort, setImapPort] = useState(993)
  const [imapUsername, setImapUsername] = useState('support@bayworks.co')
  const [imapPassword, setImapPassword] = useState('')
  const [imapSsl, setImapSsl] = useState(true)
  const [imapFolder, setImapFolder] = useState('INBOX')

  // Form Fields: Templates & Automations
  const [emailSignature, setEmailSignature] = useState('--\nBest regards,\n{{agent.name}}\nBay Works Support Team')
  const [emailFooter, setEmailFooter] = useState('Bay Works Inc. · 100 Market St, San Francisco, CA')
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState('Thank you for contacting Bay Works Support. A support agent has been assigned to your ticket and will reply shortly.')
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState('We have received your email! Our team is currently out of the office; we will respond first thing upon returning.')
  const [csatEnabled, setCsatEnabled] = useState(true)

  // Test Outbound Email State
  const [testRecipient, setTestRecipient] = useState('victoria.vance@vancetech.io')
  const [testRecipientName, setTestRecipientName] = useState('Victoria Vance')
  const [testSubject, setTestSubject] = useState('Verification email from Bay Works Support')
  const [testMessage, setTestMessage] = useState('Hello! This is a test message to confirm outbound SMTP email delivery is operational.')
  const [sendingTest, setSendingTest] = useState(false)

  // Copy Feedback
  const [copiedKey, setCopiedKey] = useState<string | null>(null)

  const loadData = async () => {
    try {
      setLoading(true)
      const res = await api.getEmailChannel()
      setData(res)
      setName(res.inbox.name || 'Support Email Inbox')
      setEmailFromAddress(res.smtpConfig.fromAddress || 'support@bayworks.co')
      setEmailFromName(res.smtpConfig.fromName || 'Bay Works Support')
      setEmailReplyTo(res.smtpConfig.replyTo || 'support@bayworks.co')
      setSmtpHost(res.smtpConfig.host || 'smtp.bayworks.co')
      setSmtpPort(res.smtpConfig.port || 587)
      setSmtpUsername(res.smtpConfig.username || 'support@bayworks.co')
      setSmtpEncryption(res.smtpConfig.encryption || 'tls')
      setImapHost(res.imapConfig.host || 'imap.bayworks.co')
      setImapPort(res.imapConfig.port || 993)
      setImapUsername(res.imapConfig.username || 'support@bayworks.co')
      setImapSsl(res.imapConfig.ssl ?? true)
      setImapFolder(res.imapConfig.folder || 'INBOX')
      setEmailSignature(res.templateConfig.signature || '')
      setEmailFooter(res.templateConfig.footer || '')
      setGreetingEnabled((res.inbox as any).greetingEnabled ?? (res.inbox as any).greeting_enabled ?? true)
      setGreetingMessage((res.inbox as any).greetingMessage || (res.inbox as any).greeting_message || '')
      setOutOfOfficeEnabled((res.inbox as any).outOfOfficeEnabled ?? (res.inbox as any).out_of_office_enabled ?? false)
      setOutOfOfficeMessage((res.inbox as any).outOfOfficeMessage || (res.inbox as any).out_of_office_message || '')
      setCsatEnabled((res.inbox as any).csatEnabled ?? (res.inbox as any).csat_enabled ?? true)

      const eventsRes = await api.listEmailEvents().catch(() => [])
      setEvents(eventsRes)
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to load Email channel settings.' })
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [])

  const handleCopy = (text: string, key: string) => {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  const handleSave = async () => {
    try {
      setSaving(true)
      setFeedback(null)
      const payload: EmailUpdatePayload = {
        name,
        email_from_address: emailFromAddress,
        email_from_name: emailFromName,
        email_reply_to: emailReplyTo,
        smtp_host: smtpHost,
        smtp_port: Number(smtpPort),
        smtp_username: smtpUsername,
        smtp_password: smtpPassword,
        smtp_encryption: smtpEncryption,
        imap_host: imapHost,
        imap_port: Number(imapPort),
        imap_username: imapUsername,
        imap_password: imapPassword,
        imap_ssl: imapSsl,
        imap_folder: imapFolder,
        email_signature: emailSignature,
        email_footer: emailFooter,
        greeting_enabled: greetingEnabled,
        greeting_message: greetingMessage,
        out_of_office_enabled: outOfOfficeEnabled,
        out_of_office_message: outOfOfficeMessage,
        csat_enabled: csatEnabled,
      }
      const updated = await api.updateEmailChannel(payload)
      setData(updated)
      setFeedback({ type: 'success', message: 'Email channel configuration, SMTP, and IMAP settings saved successfully!' })
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to update Email channel.' })
    } finally {
      setSaving(false)
    }
  }

  const handleVerify = async () => {
    try {
      setVerifying(true)
      setFeedback(null)
      const res = await api.verifyEmailChannel()
      setFeedback({ type: 'success', message: res.message || 'SMTP & IMAP email server connections verified successfully!' })
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Email server verification failed.' })
    } finally {
      setVerifying(false)
    }
  }

  const handleSyncImap = async () => {
    try {
      setSyncing(true)
      setFeedback(null)
      const res = await api.syncEmailImap()
      setFeedback({ type: 'success', message: res.statusMessage || 'IMAP mailbox synced successfully.' })
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to sync IMAP mailbox.' })
    } finally {
      setSyncing(false)
    }
  }

  const handleSendTestEmail = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      setSendingTest(true)
      setFeedback(null)
      const res = await api.sendEmailTestMessage({
        recipient_email: testRecipient,
        recipient_name: testRecipientName,
        subject: testSubject,
        message: testMessage,
      })
      setFeedback({ type: 'success', message: `Test email dispatched to ${res.recipient} (Message ID: ${res.messageId}).` })
      const updatedEvents = await api.listEmailEvents()
      setEvents(updatedEvents)
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to send test email.' })
    } finally {
      setSendingTest(false)
    }
  }

  const webhookUrl = `${window.location.origin}/api/v1/webhooks/email/inbound`

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="flex items-center gap-2 text-[var(--color-ink-muted)]">
          <span className="loading loading-spinner loading-md"></span>
          <span>Loading Email channel settings...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Top Banner / Breadcrumb */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <Link to="/channels" className="text-xs font-medium text-[var(--color-ink-muted)] hover:text-slate-800">
              Channels Hub
            </Link>
            <span className="text-[var(--color-ink-faint)]">/</span>
            <span className="text-xs font-semibold text-[var(--color-ink)]">Email Channel (IMAP/SMTP)</span>
          </div>
          <div className="mt-1 flex items-center gap-2">
            <h1 className="text-xl font-bold text-[var(--color-ink)]">Support Emails via IMAP &amp; SMTP</h1>
            <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800 border border-emerald-200 flex items-center gap-1">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
              Active &amp; Connected
            </span>
          </div>
          <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
            Connect support email accounts using standard IMAP/SMTP or inbound webhooks with automatic threaded replies and attachments.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleSyncImap}
            disabled={syncing}
            className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-xs flex items-center gap-1.5"
          >
            <span>📥</span> {syncing ? 'Polling IMAP...' : 'Poll Mailbox Now'}
          </button>
          <button
            onClick={handleVerify}
            disabled={verifying}
            className="rounded-md border border-slate-300 bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-xs flex items-center gap-1.5"
          >
            <span>🔄</span> {verifying ? 'Verifying...' : 'Verify Connections'}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-md bg-emerald-600 px-4 py-1.5 text-xs font-semibold text-white hover:bg-emerald-700 shadow-xs flex items-center gap-1.5"
          >
            <span>💾</span> {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>

      {/* Feedback Alert */}
      {feedback && (
        <div
          className={`flex items-center justify-between rounded-md p-3 text-xs ${
            feedback.type === 'success'
              ? 'bg-emerald-50 text-emerald-800 border border-emerald-200'
              : 'bg-rose-50 text-rose-800 border border-rose-200'
          }`}
        >
          <div className="flex items-center gap-2">
            <span>{feedback.type === 'success' ? '✅' : '⚠️'}</span>
            <span>{feedback.message}</span>
          </div>
          <button onClick={() => setFeedback(null)} className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] font-bold">
            ×
          </button>
        </div>
      )}

      {/* Tabs Header */}
      <div className="border-b border-[var(--color-border)]">
        <nav className="flex space-x-6">
          <button
            onClick={() => setActiveTab('smtp')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'smtp'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            <span>📤</span> SMTP Outbound Delivery
          </button>
          <button
            onClick={() => setActiveTab('imap')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'imap'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            <span>📥</span> IMAP &amp; Inbound Webhooks
          </button>
          <button
            onClick={() => setActiveTab('templates')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'templates'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            <span>🎨</span> Templates &amp; Signatures
          </button>
          <button
            onClick={() => setActiveTab('automations')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'automations'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            <span>🤖</span> Auto-Reply &amp; Hours
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'logs'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
            }`}
          >
            <span>📋</span> Diagnostics &amp; Logs ({events.length})
          </button>
        </nav>
      </div>

      {/* TAB 1: SMTP Outbound Delivery */}
      {activeTab === 'smtp' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
              <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
                <span>📤</span> Outbound SMTP Mail Server Settings
              </h2>
              <p className="text-xs text-[var(--color-ink-muted)]">
                Configure your SMTP server to deliver agent responses, email notifications, and customer resolution confirmations.
              </p>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">Inbox Display Name</label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">From Name</label>
                  <input
                    type="text"
                    value={emailFromName}
                    onChange={(e) => setEmailFromName(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">From Email Address</label>
                  <input
                    type="email"
                    value={emailFromAddress}
                    onChange={(e) => setEmailFromAddress(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                    placeholder="support@bayworks.co"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">Reply-To Address</label>
                  <input
                    type="email"
                    value={emailReplyTo}
                    onChange={(e) => setEmailReplyTo(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2 border-t border-[var(--color-border)]">
                <div className="sm:col-span-2">
                  <label className="block text-xs font-medium text-[var(--color-ink)]">SMTP Host</label>
                  <input
                    type="text"
                    value={smtpHost}
                    onChange={(e) => setSmtpHost(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                    placeholder="smtp.sendgrid.net or smtp.gmail.com"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">SMTP Port</label>
                  <input
                    type="number"
                    value={smtpPort}
                    onChange={(e) => setSmtpPort(Number(e.target.value))}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">SMTP Username</label>
                  <input
                    type="text"
                    value={smtpUsername}
                    onChange={(e) => setSmtpUsername(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">SMTP Password</label>
                  <input
                    type="password"
                    value={smtpPassword}
                    onChange={(e) => setSmtpPassword(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                    placeholder="••••••••••••••••"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">Encryption</label>
                  <select
                    value={smtpEncryption}
                    onChange={(e) => setSmtpEncryption(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500"
                  >
                    <option value="tls">STARTTLS (Port 587)</option>
                    <option value="ssl">SSL / TLS (Port 465)</option>
                    <option value="none">None (Port 25)</option>
                  </select>
                </div>
              </div>
            </div>
          </div>

          <div className="space-y-6">
            <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
              <h3 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">DNS &amp; Deliverability Health</h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-[var(--color-ink-muted)]">SPF Record:</span>
                  <span className="rounded-full bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-800">
                    ✓ Valid (v=spf1)
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-[var(--color-ink-muted)]">DKIM Signature:</span>
                  <span className="rounded-full bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-800">
                    ✓ 2048-bit Active
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-[var(--color-ink-muted)]">DMARC Policy:</span>
                  <span className="rounded-full bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-800">
                    ✓ p=quarantine
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-[var(--color-ink-muted)]">Thread Tracking:</span>
                  <span className="font-semibold text-slate-800">Message-ID &amp; In-Reply-To</span>
                </div>
              </div>
            </div>

            <div className="rounded-lg border border-emerald-200 bg-emerald-50/50 p-5 shadow-xs space-y-2">
              <h4 className="text-xs font-bold text-emerald-900 flex items-center gap-1.5">
                <span>🛡️</span> Zero-Bounce Anti-Spam Compliance
              </h4>
              <p className="text-[11px] text-emerald-800 leading-relaxed">
                All outbound emails sent from Chatwoot include strict RFC2822 formatting, unsubscription headers, and automatic thread resolution tokens.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: IMAP & Inbound Webhooks */}
      {activeTab === 'imap' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* IMAP Mailbox Settings */}
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <span>📥</span> IMAP Mailbox Polling
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Fetch incoming customer support emails automatically from your IMAP server.
            </p>

            <div className="space-y-4 pt-1">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div className="sm:col-span-2">
                  <label className="block text-xs font-medium text-[var(--color-ink)]">IMAP Host</label>
                  <input
                    type="text"
                    value={imapHost}
                    onChange={(e) => setImapHost(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs"
                    placeholder="imap.bayworks.co"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">IMAP Port</label>
                  <input
                    type="number"
                    value={imapPort}
                    onChange={(e) => setImapPort(Number(e.target.value))}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">IMAP Username</label>
                  <input
                    type="text"
                    value={imapUsername}
                    onChange={(e) => setImapUsername(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">IMAP Password</label>
                  <input
                    type="password"
                    value={imapPassword}
                    onChange={(e) => setImapPassword(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs"
                    placeholder="••••••••••••••••"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)]">Mailbox Folder</label>
                  <input
                    type="text"
                    value={imapFolder}
                    onChange={(e) => setImapFolder(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs"
                  />
                </div>
                <div className="flex items-center pt-5">
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={imapSsl}
                      onChange={(e) => setImapSsl(e.target.checked)}
                      className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                    />
                    <span className="text-xs font-medium text-[var(--color-ink)]">Use SSL / TLS (Port 993)</span>
                  </label>
                </div>
              </div>
            </div>
          </div>

          {/* Inbound Email Webhook */}
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <span>🌐</span> Inbound Email Webhook
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Alternatively, stream incoming emails in real-time from Postmark, SendGrid Inbound Parse, Mailgun, or Cloudflare Email Routing:
            </p>

            <div className="space-y-3 pt-1">
              <div>
                <div className="flex items-center justify-between text-xs text-[var(--color-ink)] mb-1">
                  <span className="font-semibold">Inbound Webhook URL:</span>
                  <button
                    onClick={() => handleCopy(webhookUrl, 'inbound_email')}
                    className="text-blue-600 hover:text-blue-800 font-medium"
                  >
                    {copiedKey === 'inbound_email' ? '✓ Copied' : 'Copy'}
                  </button>
                </div>
                <input
                  readOnly
                  value={webhookUrl}
                  className="block w-full rounded-md bg-[var(--color-page)] border border-[var(--color-border)] px-3 py-1.5 text-xs font-mono text-[var(--color-ink)]"
                />
              </div>

              <div className="rounded-md bg-[var(--color-page)] p-3 border border-[var(--color-border)] text-xs text-[var(--color-ink-muted)] space-y-1.5">
                <div className="font-bold text-slate-800">Supported Inbound Formats:</div>
                <div>• <strong>Postmark Inbound</strong>: JSON payload with `FromFull`, `TextBody`, and attachments.</div>
                <div>• <strong>SendGrid Inbound Parse</strong>: Multipart form-data with automatic thread headers.</div>
                <div>• <strong>Universal JSON</strong>: Raw RFC2822 or structured &#123; from, to, subject, body &#125;.</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: Templates & Signatures */}
      {activeTab === 'templates' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <span>✍️</span> Dynamic Agent Signature
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Customize the signature appended to outgoing emails. Use variables like <code className="bg-slate-100 px-1 py-0.5 rounded text-[11px] font-mono">{'{{agent.name}}'}</code> and <code className="bg-slate-100 px-1 py-0.5 rounded text-[11px] font-mono">{'{{inbox.name}}'}</code>.
            </p>

            <div>
              <label className="block text-xs font-medium text-[var(--color-ink)]">Email Signature</label>
              <textarea
                rows={4}
                value={emailSignature}
                onChange={(e) => setEmailSignature(e.target.value)}
                className="mt-1 block w-full rounded-md border border-slate-300 p-2.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-[var(--color-ink)]">Legal Footer &amp; Company Address</label>
              <textarea
                rows={2}
                value={emailFooter}
                onChange={(e) => setEmailFooter(e.target.value)}
                className="mt-1 block w-full rounded-md border border-slate-300 p-2.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
              />
            </div>
          </div>

          {/* HTML Preview Box */}
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <h3 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">Live Email Preview</h3>
            <div className="rounded-md border border-[var(--color-border)] p-4 bg-[var(--color-page)]/50 space-y-3 text-xs text-slate-800">
              <div className="border-b border-[var(--color-border)] pb-2 text-[11px] text-[var(--color-ink-muted)] space-y-1">
                <div>From: <strong>{emailFromName} &lt;{emailFromAddress}&gt;</strong></div>
                <div>To: <strong>Customer &lt;customer@example.com&gt;</strong></div>
                <div>Subject: <strong>Re: Assistance with enterprise account [#CONV-8491]</strong></div>
              </div>
              <p className="leading-relaxed">
                Hi Alex, thank you for writing in! Our engineering team has deployed the requested rate limit configuration. Let us know if you need any additional assistance.
              </p>
              <div className="pt-2 border-t border-[var(--color-border)] text-[var(--color-ink-muted)] whitespace-pre-line font-mono text-[11px]">
                {emailSignature.replace('{{agent.name}}', 'Riley Brooks')}
              </div>
              <div className="text-[10px] text-[var(--color-ink-faint)] pt-2 border-t border-[var(--color-border)]">
                {emailFooter}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 4: Auto-Reply & Hours */}
      {activeTab === 'automations' && (
        <div className="space-y-6">
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <span>🤖</span> Automated Email Auto-Responders
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Keep customers informed automatically with welcome receipts, out-of-office notices, and CSAT satisfaction rating surveys.
            </p>

            <div className="space-y-4 pt-1">
              <div>
                <label className="flex items-center gap-2 mb-1.5">
                  <input
                    type="checkbox"
                    checked={greetingEnabled}
                    onChange={(e) => setGreetingEnabled(e.target.checked)}
                    className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                  />
                  <span className="text-xs font-semibold text-slate-800">Send instant ticket acknowledgement email on first inbound message</span>
                </label>
                <textarea
                  rows={2}
                  value={greetingMessage}
                  onChange={(e) => setGreetingMessage(e.target.value)}
                  disabled={!greetingEnabled}
                  className="block w-full rounded-md border border-slate-300 p-2 text-xs shadow-xs disabled:bg-slate-100"
                />
              </div>

              <div>
                <label className="flex items-center gap-2 mb-1.5">
                  <input
                    type="checkbox"
                    checked={outOfOfficeEnabled}
                    onChange={(e) => setOutOfOfficeEnabled(e.target.checked)}
                    className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                  />
                  <span className="text-xs font-semibold text-slate-800">Out-of-Office Auto-Reply (Outside working hours)</span>
                </label>
                <textarea
                  rows={2}
                  value={outOfOfficeMessage}
                  onChange={(e) => setOutOfOfficeMessage(e.target.value)}
                  disabled={!outOfOfficeEnabled}
                  className="block w-full rounded-md border border-slate-300 p-2 text-xs shadow-xs disabled:bg-slate-100"
                />
              </div>

              <div>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={csatEnabled}
                    onChange={(e) => setCsatEnabled(e.target.checked)}
                    className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                  />
                  <span className="text-xs font-semibold text-slate-800">Send CSAT customer satisfaction rating link upon ticket resolution</span>
                </label>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 5: Diagnostics & Mail Logs */}
      {activeTab === 'logs' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Outbound Test Email Sender */}
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
              <span>✉️</span> Send Diagnostic Outbound Email
            </h2>
            <p className="text-xs text-[var(--color-ink-muted)]">
              Dispatch a test message through your configured SMTP mail server to verify end-to-end deliverability.
            </p>

            <form onSubmit={handleSendTestEmail} className="space-y-3 pt-1">
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)]">Recipient Email</label>
                <input
                  type="email"
                  required
                  value={testRecipient}
                  onChange={(e) => setTestRecipient(e.target.value)}
                  className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)]">Recipient Name</label>
                <input
                  type="text"
                  value={testRecipientName}
                  onChange={(e) => setTestRecipientName(e.target.value)}
                  className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)]">Subject</label>
                <input
                  type="text"
                  required
                  value={testSubject}
                  onChange={(e) => setTestSubject(e.target.value)}
                  className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)]">Message Body</label>
                <textarea
                  rows={3}
                  required
                  value={testMessage}
                  onChange={(e) => setTestMessage(e.target.value)}
                  className="mt-1 block w-full rounded-md border border-slate-300 p-2 text-xs shadow-xs"
                />
              </div>

              <button
                type="submit"
                disabled={sendingTest}
                className="w-full rounded-md bg-emerald-600 px-4 py-2 text-xs font-bold text-white hover:bg-emerald-700 shadow-xs disabled:opacity-50 flex items-center justify-center gap-1.5"
              >
                <span>🚀</span> {sendingTest ? 'Sending Email...' : 'Send Test Outbound Email'}
              </button>
            </form>
          </div>

          {/* Email Transmission & Webhook Event Logs */}
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-xs space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider">Live Email Event Stream</h3>
              <button
                onClick={async () => {
                  const evs = await api.listEmailEvents()
                  setEvents(evs)
                }}
                className="text-xs text-blue-600 hover:text-blue-800 font-semibold"
              >
                Refresh
              </button>
            </div>

            <div className="rounded-md bg-[var(--color-accent)] text-slate-200 p-4 font-mono text-xs max-h-96 overflow-y-auto space-y-2">
              {events.length === 0 ? (
                <div className="text-[var(--color-ink-muted)]">No email events recorded yet. Send a test email or trigger an inbound webhook!</div>
              ) : (
                events.map((ev) => (
                  <div key={ev.id} className="text-slate-300 border-b border-slate-800 pb-1.5">
                    <span className="text-emerald-400">[{ev.sentAt}]</span>{' '}
                    <span className="text-blue-300">[{ev.direction.toUpperCase()}]</span>{' '}
                    <span className="text-amber-300">({ev.provider})</span>{' '}
                    <span className="text-purple-300">{ev.senderName}:</span>{' '}
                    <span>{ev.body}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
