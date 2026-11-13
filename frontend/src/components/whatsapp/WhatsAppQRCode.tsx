import { useState, useMemo } from 'react'
import { WhatsAppIcon, Copy, Check, ExternalLink, Smartphone, Link as LinkIcon, Download } from '../../components/icons'

// ---------------------------------------------------------------------------
// QR Code generator using a public API (no dependencies)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------
interface Props {
  phoneNumber?: string
  displayName?: string
  connected: boolean
}

export function WhatsAppQRCode({ phoneNumber, displayName, connected }: Props) {
  const [message, setMessage] = useState('Welcome! How can we help you?')
  const [copied, setCopied] = useState(false)

  // Build the click-to-chat URL
  const chatUrl = useMemo(() => {
    if (!phoneNumber) return ''
    const clean = phoneNumber.replace(/\D/g, '')
    const encoded = encodeURIComponent(message)
    return `https://wa.me/${clean}?text=${encoded}`
  }, [phoneNumber, message])

  // QR code URL via quickchart.io (free, no API key needed)
  const qrImageUrl = useMemo(() => {
    if (!chatUrl) return ''
    return `https://api.qrserver.com/v1/create-qr-code/?size=256x256&data=${encodeURIComponent(chatUrl)}`
  }, [chatUrl])

  async function handleCopy() {
    if (!chatUrl) return
    try {
      await navigator.clipboard.writeText(chatUrl)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch { /* ignore */ }
  }

  function handleDownloadQR() {
    if (!qrImageUrl) return
    const a = document.createElement('a')
    a.href = qrImageUrl
    a.download = 'whatsapp-qr.png'
    a.target = '_blank'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h3 className="text-base font-bold text-[var(--color-ink)] flex items-center gap-2">
          <Smartphone className="h-5 w-5 text-emerald-600" />
          QR Code & Link Sharing
        </h3>
        <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
          Generate a QR code or shareable link that opens a WhatsApp conversation with your business number.
        </p>
      </div>

      {/* Connection status guard */}
      {!connected || !phoneNumber ? (
        <div className="rounded-[var(--radius-lg)] border-2 border-dashed border-[var(--color-border)] p-8 text-center">
          <WhatsAppIcon className="h-10 w-10 text-[var(--color-ink-faint)] mx-auto" />
          <p className="mt-3 text-sm font-semibold text-[var(--color-ink)]">Connect WhatsApp First</p>
          <p className="mt-1 text-xs text-[var(--color-ink-faint)]">
            Configure your WhatsApp credentials to generate a shareable link and QR code.
          </p>
        </div>
      ) : (
        <>
          {/* Message preview */}
          <div>
            <label className="block text-sm font-semibold text-[var(--color-ink)] mb-2">
              Pre-filled Message
            </label>
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              rows={2}
              maxLength={200}
              className="w-full rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-hidden resize-none"
              placeholder="Welcome message when customers click the link..."
            />
            <p className="mt-1 text-[11px] text-[var(--color-ink-faint)]">
              {message.length}/200 characters — shown when customers open the link
            </p>
          </div>

          {/* QR Code Display */}
          <div className="flex flex-col lg:flex-row gap-6">
            {/* QR Code */}
            <div className="flex-1">
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-white p-6 flex flex-col items-center">
                <p className="text-sm font-semibold text-[var(--color-ink)] mb-4">QR Code</p>
                {qrImageUrl ? (
                  <img
                    src={qrImageUrl}
                    alt="WhatsApp QR Code"
                    className="w-48 h-48 rounded-[var(--radius-md)] border border-[var(--color-border)]"
                  />
                ) : (
                  <div className="w-48 h-48 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-page)] flex items-center justify-center">
                    <Smartphone className="h-12 w-12 text-[var(--color-ink-faint)]" />
                  </div>
                )}
                <button
                  onClick={handleDownloadQR}
                  className="mt-4 rounded-[var(--radius-md)] bg-emerald-600 px-4 py-2 text-xs font-semibold text-white hover:bg-emerald-700 flex items-center gap-1.5 transition-colors"
                >
                  <Download className="h-3.5 w-3.5" />
                  Download QR Code
                </button>
              </div>
            </div>

            {/* Link */}
            <div className="flex-1 space-y-4">
              <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] p-5">
                <p className="text-sm font-semibold text-[var(--color-ink)] mb-2 flex items-center gap-2">
                  <LinkIcon className="h-4 w-4 text-emerald-600" />
                  Click-to-Chat Link
                </p>
                <div className="flex items-center gap-2">
                  <code className="flex-1 rounded-[var(--radius-md)] bg-[var(--color-page)] border border-[var(--color-border)] px-3 py-2 text-xs font-mono text-[var(--color-ink-muted)] break-all">
                    {chatUrl}
                  </code>
                  <button
                    onClick={handleCopy}
                    className="shrink-0 rounded-[var(--radius-md)] border border-[var(--color-border)] p-2 text-[var(--color-ink-muted)] hover:bg-[var(--color-accent-muted)] hover:text-[var(--color-ink)] transition-colors"
                    title="Copy to clipboard"
                  >
                    {copied ? <Check className="h-4 w-4 text-emerald-600" /> : <Copy className="h-4 w-4" />}
                  </button>
                </div>
                {copied && (
                  <p className="mt-2 text-xs text-emerald-600 font-semibold">Copied to clipboard!</p>
                )}
              </div>

              {/* Test link */}
              {chatUrl && (
                <a
                  href={chatUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center gap-2 rounded-[var(--radius-md)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm font-semibold text-emerald-700 hover:bg-emerald-100 transition-colors"
                >
                  <ExternalLink className="h-4 w-4" />
                  Test Link in New Tab
                </a>
              )}
            </div>
          </div>

          {/* How to use */}
          <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-accent-muted)] p-5">
            <p className="text-sm font-bold text-[var(--color-ink)] mb-3">How to Use</p>
            <ul className="space-y-2 text-xs text-[var(--color-ink-muted)]">
              <li className="flex items-start gap-2">
                <span className="mt-0.5 h-4 w-4 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center text-[10px] font-bold shrink-0">1</span>
                <span><strong>QR Code:</strong> Print it on business cards, posters, or packaging. Customers scan to start a chat.</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 h-4 w-4 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center text-[10px] font-bold shrink-0">2</span>
                <span><strong>Link:</strong> Share in emails, on your website, or in social media bios. Customers click to open WhatsApp.</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 h-4 w-4 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center text-[10px] font-bold shrink-0">3</span>
                <span><strong>Pre-filled Message:</strong> Customize the message to track where customers came from (e.g., "Hi from Instagram").</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 h-4 w-4 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center text-[10px] font-bold shrink-0">4</span>
                <span><strong>Meta Business Manager:</strong> For advanced QR code management, you can also use Meta's Business Manager directly.</span>
              </li>
            </ul>
          </div>

          {/* Meta Business Manager link */}
          <a
            href="https://business.facebook.com/"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-center gap-2 rounded-[var(--radius-md)] border border-[var(--color-border)] px-4 py-3 text-sm font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-muted)] transition-colors"
          >
            <ExternalLink className="h-4 w-4" />
            Open Meta Business Manager
          </a>
        </>
      )}
    </div>
  )
}
