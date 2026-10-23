import { useEffect, useState } from 'react'
import {
  api,
  type WebhookDeliveryLog,
  type WebhookEndpoint,
  type WebhookEventCatalogItem,
} from '../lib/api'

export function Webhooks() {
  const [activeTab, setActiveTab] = useState<'endpoints' | 'deliveries' | 'catalog'>('endpoints')
  const [endpoints, setEndpoints] = useState<WebhookEndpoint[]>([])
  const [deliveries, setDeliveries] = useState<WebhookDeliveryLog[]>([])
  const [catalog, setCatalog] = useState<WebhookEventCatalogItem[]>([])
  const [selectedEndpointId, setSelectedEndpointId] = useState<string | null>(null)
  const [inspectedLog, setInspectedLog] = useState<WebhookDeliveryLog | null>(null)
  const [showAddModal, setShowAddModal] = useState(false)
  const [editingEndpoint, setEditingEndpoint] = useState<WebhookEndpoint | null>(null)
  const [loading, setLoading] = useState(false)
  const [testingId, setTestingId] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  function loadEndpoints() {
    setLoading(true)
    api
      .listWebhookEndpoints()
      .then(setEndpoints)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false))
  }

  function loadDeliveries(epId?: string) {
    api
      .getWebhookDeliveries(epId || undefined, 50)
      .then(setDeliveries)
      .catch((e) => setError(e.message))
  }

  useEffect(() => {
    loadEndpoints()
    api.getWebhookCatalog().then(setCatalog).catch(console.error)
  }, [])

  useEffect(() => {
    if (activeTab === 'deliveries') {
      loadDeliveries(selectedEndpointId || undefined)
    }
  }, [activeTab, selectedEndpointId])

  async function handleTestPing(epId: string) {
    setTestingId(epId)
    try {
      const res = await api.testWebhookPing(epId)
      if (res.success) {
        setNotice(`Test ping delivered successfully! Status: ${res.status_code} (${res.duration_ms}ms)`)
      } else {
        setError(`Test ping failed: HTTP ${res.status_code} - ${res.error || 'Connection error'}`)
      }
      loadEndpoints()
      if (activeTab === 'deliveries') loadDeliveries(selectedEndpointId || undefined)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setTestingId(null)
    }
  }

  async function handleResend(deliveryId: string) {
    try {
      const res = await api.resendWebhookDelivery(deliveryId)
      if (res.success) {
        setNotice(`Payload re-dispatched successfully! Status: ${res.status_code} (${res.duration_ms}ms)`)
        loadDeliveries(selectedEndpointId || undefined)
        loadEndpoints()
      } else {
        setError('Failed to resend payload.')
      }
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this webhook endpoint?')) return
    try {
      await api.deleteWebhookEndpoint(id)
      setNotice('Webhook endpoint removed.')
      loadEndpoints()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  // Summary statistics
  const totalDelivered = endpoints.reduce((acc, ep) => acc + (ep.total_delivered || 0), 0)
  const totalFailed = endpoints.reduce((acc, ep) => acc + (ep.total_failed || 0), 0)
  const successRate =
    totalDelivered + totalFailed > 0
      ? ((totalDelivered / (totalDelivered + totalFailed)) * 100).toFixed(1)
      : '100.0'

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-[var(--color-ink)]">Outgoing Webhooks Engine</h1>
          <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
            Send real-time event notifications with HMAC-SHA256 signatures, automatic retries, and delivery telemetry.
          </p>
        </div>

        <button
          onClick={() => {
            setEditingEndpoint(null)
            setShowAddModal(true)
          }}
          className="inline-flex items-center gap-1.5 self-start rounded-md bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] px-3.5 py-1.5 text-xs font-medium text-white shadow-xs transition-colors"
        >
          <span>+</span>
          <span>Add Webhook Endpoint</span>
        </button>
      </div>

      {notice && <Banner tone="success">{notice}</Banner>}
      {error && <Banner tone="error">{error}</Banner>}

      {/* KPI Stats Bar */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-2xs">
          <span className="text-xs text-[var(--color-ink-muted)] font-medium">Active Endpoints</span>
          <div className="mt-1 text-xl font-bold text-[var(--color-ink)]">
            {endpoints.filter((e) => e.is_active).length} <span className="text-xs text-[var(--color-ink-faint)] font-normal">/ {endpoints.length}</span>
          </div>
        </div>
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-2xs">
          <span className="text-xs text-[var(--color-ink-muted)] font-medium">Deliveries Dispatched</span>
          <div className="mt-1 text-xl font-bold text-[var(--color-ink)]">{totalDelivered.toLocaleString()}</div>
        </div>
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-2xs">
          <span className="text-xs text-[var(--color-ink-muted)] font-medium">Delivery Success Rate</span>
          <div className="mt-1 text-xl font-bold text-emerald-600">{successRate}%</div>
        </div>
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-3.5 shadow-2xs">
          <span className="text-xs text-[var(--color-ink-muted)] font-medium">Signature Standard</span>
          <div className="mt-1 text-sm font-semibold text-[var(--color-accent)] bg-[var(--color-accent-soft)] px-2 py-0.5 rounded inline-block mt-1">
            HMAC-SHA256
          </div>
        </div>
      </div>

      {/* Primary Tab Navigation */}
      <div className="flex border-b border-[var(--color-border)] gap-4 overflow-x-auto">
        <button
          onClick={() => setActiveTab('endpoints')}
          className={`py-2 px-1 border-b-2 font-medium text-sm transition-colors ${
            activeTab === 'endpoints'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          📡 Webhook Endpoints ({endpoints.length})
        </button>
        <button
          onClick={() => setActiveTab('deliveries')}
          className={`py-2 px-1 border-b-2 font-medium text-sm transition-colors ${
            activeTab === 'deliveries'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          📜 Delivery History & Logs
        </button>
        <button
          onClick={() => setActiveTab('catalog')}
          className={`py-2 px-1 border-b-2 font-medium text-sm transition-colors ${
            activeTab === 'catalog'
              ? 'border-indigo-600 text-[var(--color-accent)] font-semibold'
              : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
          }`}
        >
          📖 Event Catalog & Verification Guide
        </button>
      </div>

      {/* Tab 1: Webhook Endpoints */}
      {activeTab === 'endpoints' && (
        <div className="space-y-4">
          {loading ? (
            <p className="text-sm text-[var(--color-ink-muted)] py-8 text-center">Loading webhook endpoints…</p>
          ) : endpoints.length === 0 ? (
            <div className="rounded-lg border border-dashed border-slate-300 p-8 text-center bg-[var(--color-surface)]">
              <p className="text-sm text-[var(--color-ink-muted)]">No outgoing webhooks configured yet.</p>
              <button
                onClick={() => setShowAddModal(true)}
                className="mt-3 text-xs font-medium text-[var(--color-accent)] hover:underline"
              >
                + Create your first webhook
              </button>
            </div>
          ) : (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              {endpoints.map((ep) => (
                <div
                  key={ep.id}
                  className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-2xs flex flex-col justify-between hover:shadow-xs transition-shadow"
                >
                  <div>
                    <div className="flex items-start justify-between gap-3 mb-2">
                      <div className="min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-semibold text-[var(--color-ink)] text-sm truncate">{ep.name}</span>
                          <span
                            className={`rounded-full px-2 py-0.5 text-3xs font-medium ${
                              ep.is_active
                                ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                                : 'bg-slate-100 text-[var(--color-ink-muted)] border border-[var(--color-border)]'
                            }`}
                          >
                            {ep.is_active ? 'Active' : 'Paused'}
                          </span>
                        </div>
                        <div className="mt-1 font-mono text-2xs text-[var(--color-ink-muted)] truncate bg-[var(--color-page)] px-2 py-1 rounded border border-[var(--color-border)]">
                          {ep.url}
                        </div>
                      </div>

                      {/* Status indicator */}
                      <span
                        className={`shrink-0 rounded-full px-2 py-0.5 text-2xs font-semibold ${
                          ep.last_status_code === 200 || ep.last_status_code === 201
                            ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                            : ep.last_status_code && ep.last_status_code >= 400
                              ? 'bg-red-50 text-red-700 border border-red-200'
                              : 'bg-slate-100 text-[var(--color-ink-muted)] border border-[var(--color-border)]'
                        }`}
                      >
                        {ep.last_status_code ? `HTTP ${ep.last_status_code}` : 'Untested'}
                      </span>
                    </div>

                    {/* Subscribed Events Tags */}
                    <div className="my-3">
                      <span className="text-3xs font-semibold uppercase tracking-wider text-[var(--color-ink-faint)] block mb-1">
                        Subscribed Events ({ep.events.length})
                      </span>
                      <div className="flex flex-wrap gap-1">
                        {ep.events.map((ev) => (
                          <span
                            key={ev}
                            className="rounded bg-slate-100 px-1.5 py-0.5 text-3xs font-mono text-[var(--color-ink)]"
                          >
                            {ev}
                          </span>
                        ))}
                      </div>
                    </div>

                    {/* Secret & Telemetry */}
                    <div className="grid grid-cols-2 gap-2 text-2xs text-[var(--color-ink-muted)] pt-2 border-t border-[var(--color-border)]">
                      <div>
                        <span className="text-[var(--color-ink-faint)]">Secret: </span>
                        <code className="text-[var(--color-ink)] font-mono">{ep.secret_masked}</code>
                      </div>
                      <div className="text-right">
                        <span>Delivered: <strong className="text-slate-800">{ep.total_delivered}</strong></span>
                        {ep.total_failed > 0 && (
                          <span className="ml-2 text-red-600">Failed: {ep.total_failed}</span>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Action Bar */}
                  <div className="mt-4 pt-3 border-t border-[var(--color-border)] flex items-center justify-between text-xs">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleTestPing(ep.id)}
                        disabled={testingId === ep.id}
                        className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1 font-medium text-[var(--color-ink)] hover:bg-[var(--color-page)] disabled:opacity-50"
                      >
                        {testingId === ep.id ? 'Pinging…' : '⚡ Test Ping'}
                      </button>
                      <button
                        onClick={() => {
                          setSelectedEndpointId(ep.id)
                          setActiveTab('deliveries')
                        }}
                        className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1 font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-page)]"
                      >
                        📜 Logs
                      </button>
                      <button
                        onClick={() => {
                          setEditingEndpoint(ep)
                          setShowAddModal(true)
                        }}
                        className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]"
                      >
                        Edit
                      </button>
                    </div>

                    <button
                      onClick={() => handleDelete(ep.id)}
                      className="text-[var(--color-ink-faint)] hover:text-red-600 transition-colors p-1"
                      title="Delete Webhook"
                    >
                      ✕
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Tab 2: Delivery History & Logs */}
      {activeTab === 'deliveries' && (
        <div className="space-y-4">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 bg-[var(--color-surface)] p-3.5 rounded-lg border border-[var(--color-border)]">
            <div className="flex items-center gap-2">
              <span className="text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider">Endpoint:</span>
              <select
                value={selectedEndpointId || ''}
                onChange={(e) => setSelectedEndpointId(e.target.value || null)}
                className="rounded-md border border-slate-300 px-3 py-1 text-xs text-slate-800"
              >
                <option value="">All Endpoints ({deliveries.length} logs)</option>
                {endpoints.map((ep) => (
                  <option key={ep.id} value={ep.id}>
                    {ep.name}
                  </option>
                ))}
              </select>
            </div>

            <button
              onClick={() => loadDeliveries(selectedEndpointId || undefined)}
              className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-2.5 py-1 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-page)]"
            >
              🔄 Refresh Logs
            </button>
          </div>

          {deliveries.length === 0 ? (
            <div className="rounded-lg border border-dashed border-slate-300 p-8 text-center bg-[var(--color-surface)]">
              <p className="text-sm text-[var(--color-ink-muted)]">No delivery logs recorded yet.</p>
            </div>
          ) : (
            <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden shadow-2xs">
              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs">
                  <thead className="bg-[var(--color-page)] border-b border-[var(--color-border)] text-[var(--color-ink-muted)] uppercase text-3xs font-semibold tracking-wider">
                    <tr>
                      <th className="px-4 py-2.5">Status</th>
                      <th className="px-4 py-2.5">Event</th>
                      <th className="px-4 py-2.5">Duration</th>
                      <th className="px-4 py-2.5">Attempt</th>
                      <th className="px-4 py-2.5">Dispatched At</th>
                      <th className="px-4 py-2.5 text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {deliveries.map((d) => (
                      <tr key={d.id} className="hover:bg-[var(--color-page)]/70 transition-colors">
                        <td className="px-4 py-2.5">
                          <span
                            className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-3xs font-bold ${
                              d.status_code >= 200 && d.status_code < 300
                                ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
                                : 'bg-red-50 text-red-700 border border-red-200'
                            }`}
                          >
                            {d.status_code ? `HTTP ${d.status_code}` : 'TIMEOUT'}
                          </span>
                        </td>
                        <td className="px-4 py-2.5 font-mono text-slate-800 font-medium">
                          {d.event}
                        </td>
                        <td className="px-4 py-2.5 text-[var(--color-ink-muted)]">
                          {d.duration_ms}ms
                        </td>
                        <td className="px-4 py-2.5 text-[var(--color-ink-muted)]">
                          #{d.attempt}
                        </td>
                        <td className="px-4 py-2.5 text-[var(--color-ink-muted)] whitespace-nowrap">
                          {new Date(d.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                        </td>
                        <td className="px-4 py-2.5 text-right space-x-2">
                          <button
                            onClick={() => setInspectedLog(d)}
                            className="text-xs font-medium text-[var(--color-accent)] hover:text-indigo-800"
                          >
                            Inspect Payload
                          </button>
                          <button
                            onClick={() => handleResend(d.id)}
                            className="text-xs font-medium text-[var(--color-ink-muted)] hover:text-slate-800"
                          >
                            🔄 Resend
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Tab 3: Catalog & HMAC Guide */}
      {activeTab === 'catalog' && (
        <div className="space-y-6">
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-2xs">
            <h2 className="text-base font-semibold text-[var(--color-ink)] mb-2">Supported Event Types Catalog</h2>
            <p className="text-xs text-[var(--color-ink-muted)] mb-4">
              Your endpoints can subscribe to any subset of these events, or use <code className="text-[var(--color-accent)] font-bold">*</code> to receive all events.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {catalog.map((item) => (
                <div key={item.event} className="rounded-md border border-[var(--color-border)] bg-[var(--color-page)] p-3 text-xs">
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-mono font-semibold text-slate-800">{item.event}</span>
                    <span className="rounded bg-[var(--color-accent-soft)] text-[var(--color-accent)] px-1.5 py-0.5 text-3xs font-medium border border-indigo-100">
                      {item.category}
                    </span>
                  </div>
                  <p className="text-[var(--color-ink-muted)] text-2xs mt-1">{item.description}</p>
                </div>
              ))}
            </div>
          </div>

          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-2xs">
            <h2 className="text-base font-semibold text-[var(--color-ink)] mb-2">Payload Verification Guide (HMAC-SHA256)</h2>
            <p className="text-xs text-[var(--color-ink-muted)] mb-3">
              Every webhook request includes an <code className="text-slate-800 font-mono">X-Communicator-Signature</code> header.
              Calculate the HMAC-SHA256 digest of the raw request body using your endpoint secret to verify authenticity.
            </p>

            <div className="rounded-md bg-[var(--color-accent)] p-4 text-slate-100 font-mono text-2xs overflow-x-auto space-y-2">
              <div className="text-[var(--color-ink-faint)]"># Python Example (FastAPI / Flask)</div>
              <div>import hmac, hashlib</div>
              <div>expected_sig = &quot;sha256=&quot; + hmac.new(WEBHOOK_SECRET.encode(), raw_body, hashlib.sha256).hexdigest()</div>
              <div>is_valid = hmac.compare_digest(expected_sig, request.headers.get(&quot;X-Communicator-Signature&quot;, &quot;&quot;))</div>
            </div>
          </div>
        </div>
      )}

      {/* Add / Edit Webhook Modal */}
      {showAddModal && (
        <AddEditWebhookModal
          catalog={catalog}
          endpoint={editingEndpoint}
          onClose={() => {
            setShowAddModal(false)
            setEditingEndpoint(null)
          }}
          onSaved={() => {
            setShowAddModal(false)
            setEditingEndpoint(null)
            setNotice(editingEndpoint ? 'Webhook updated.' : 'New webhook endpoint registered!')
            loadEndpoints()
          }}
          onError={setError}
        />
      )}

      {/* Inspect Payload Drawer / Modal */}
      {inspectedLog && (
        <InspectPayloadModal log={inspectedLog} onClose={() => setInspectedLog(null)} />
      )}
    </div>
  )
}

function AddEditWebhookModal({
  catalog,
  endpoint,
  onClose,
  onSaved,
  onError,
}: {
  catalog: WebhookEventCatalogItem[]
  endpoint: WebhookEndpoint | null
  onClose: () => void
  onSaved: () => void
  onError: (msg: string) => void
}) {
  const [name, setName] = useState(endpoint?.name || '')
  const [url, setUrl] = useState(endpoint?.url || '')
  const [selectedEvents, setSelectedEvents] = useState<string[]>(endpoint?.events || ['conversation.created', 'ticket.created'])
  const [customSecret, setCustomSecret] = useState('')
  const [headerKey, setHeaderKey] = useState('')
  const [headerVal, setHeaderVal] = useState('')
  const [headers, setHeaders] = useState<Record<string, string>>(endpoint?.headers || {})
  const [isActive, setIsActive] = useState(endpoint ? endpoint.is_active : true)
  const [retryCount, setRetryCount] = useState(endpoint ? endpoint.retry_count : 3)
  const [timeoutSeconds, setTimeoutSeconds] = useState(endpoint ? endpoint.timeout_seconds : 10)
  const [saving, setSaving] = useState(false)

  function toggleEvent(ev: string) {
    if (selectedEvents.includes(ev)) {
      setSelectedEvents(selectedEvents.filter((e) => e !== ev))
    } else {
      setSelectedEvents([...selectedEvents, ev])
    }
  }

  function handleAddHeader() {
    if (!headerKey.trim()) return
    setHeaders({ ...headers, [headerKey.trim()]: headerVal.trim() })
    setHeaderKey('')
    setHeaderVal('')
  }

  function handleRemoveHeader(k: string) {
    const next = { ...headers }
    delete next[k]
    setHeaders(next)
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    try {
      if (endpoint) {
        await api.updateWebhookEndpoint(endpoint.id, {
          name,
          url,
          events: selectedEvents,
          headers,
          is_active: isActive,
          retry_count: Number(retryCount),
          timeout_seconds: Number(timeoutSeconds),
        })
      } else {
        await api.createWebhookEndpoint({
          name,
          url,
          events: selectedEvents,
          secret: customSecret || undefined,
          headers,
          is_active: isActive,
          retry_count: Number(retryCount),
          timeout_seconds: Number(timeoutSeconds),
        })
      }
      onSaved()
    } catch (err) {
      onError((err as Error).message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 p-4 backdrop-blur-xs">
      <div className="w-full max-w-lg rounded-xl bg-[var(--color-surface)] shadow-xl border border-[var(--color-border)] overflow-hidden max-h-[90vh] flex flex-col text-xs">
        <div className="border-b border-[var(--color-border)] px-5 py-3.5 flex items-center justify-between bg-[var(--color-page)]">
          <h2 className="text-sm font-semibold text-[var(--color-ink)]">
            {endpoint ? 'Edit Webhook Endpoint' : '+ Add Webhook Endpoint'}
          </h2>
          <button onClick={onClose} className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)]">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 space-y-4 overflow-y-auto flex-1">
          <div>
            <label className="block font-medium text-[var(--color-ink)] mb-1">Subscriber Name</label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Data Warehouse Sync"
              className="w-full rounded border border-slate-300 px-3 py-1.5 text-xs"
            />
          </div>

          <div>
            <label className="block font-medium text-[var(--color-ink)] mb-1">HTTPS Callback URL</label>
            <input
              type="url"
              required
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://api.company.com/webhooks/incoming"
              className="w-full rounded border border-slate-300 px-3 py-1.5 text-xs"
            />
          </div>

          {/* Subscribed Events Selection */}
          <div>
            <div className="flex items-center justify-between mb-1.5">
              <label className="font-medium text-[var(--color-ink)]">Subscribed Events ({selectedEvents.length})</label>
              <button
                type="button"
                onClick={() => {
                  if (selectedEvents.length === catalog.length) {
                    setSelectedEvents([])
                  } else {
                    setSelectedEvents(catalog.map((c) => c.event))
                  }
                }}
                className="text-[var(--color-accent)] hover:underline text-2xs font-medium"
              >
                {selectedEvents.length === catalog.length ? 'Deselect All' : 'Select All'}
              </button>
            </div>

            <div className="grid grid-cols-2 gap-2 border border-[var(--color-border)] rounded-md p-3 max-h-36 overflow-y-auto bg-[var(--color-page)]/50">
              {catalog.map((item) => (
                <label key={item.event} className="flex items-start gap-1.5 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={selectedEvents.includes(item.event)}
                    onChange={() => toggleEvent(item.event)}
                    className="mt-0.5 h-3.5 w-3.5 rounded border-slate-300 text-[var(--color-accent)]"
                  />
                  <div className="min-w-0">
                    <span className="font-mono text-3xs text-slate-800 block truncate">{item.event}</span>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* Custom Headers */}
          <div>
            <label className="block font-medium text-[var(--color-ink)] mb-1">Custom HTTP Headers</label>
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                placeholder="Header Name (e.g. Authorization)"
                value={headerKey}
                onChange={(e) => setHeaderKey(e.target.value)}
                className="flex-1 rounded border border-slate-300 px-2 py-1 text-2xs"
              />
              <input
                type="text"
                placeholder="Value (e.g. Bearer token)"
                value={headerVal}
                onChange={(e) => setHeaderVal(e.target.value)}
                className="flex-1 rounded border border-slate-300 px-2 py-1 text-2xs"
              />
              <button
                type="button"
                onClick={handleAddHeader}
                className="rounded bg-slate-100 hover:bg-slate-200 px-2.5 py-1 text-2xs font-medium text-[var(--color-ink)]"
              >
                + Add
              </button>
            </div>

            {Object.entries(headers).length > 0 && (
              <div className="space-y-1">
                {Object.entries(headers).map(([k, v]) => (
                  <div key={k} className="flex items-center justify-between bg-[var(--color-page)] px-2 py-1 rounded text-2xs border border-[var(--color-border)]">
                    <span className="font-mono text-[var(--color-ink)]">{k}: {v}</span>
                    <button
                      type="button"
                      onClick={() => handleRemoveHeader(k)}
                      className="text-[var(--color-ink-faint)] hover:text-red-600"
                    >
                      ✕
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {!endpoint && (
            <div>
              <label className="block font-medium text-[var(--color-ink)] mb-1">
                HMAC-SHA256 Signing Secret (Optional)
              </label>
              <input
                type="password"
                value={customSecret}
                onChange={(e) => setCustomSecret(e.target.value)}
                placeholder="Leave blank to auto-generate high-entropy secret"
                className="w-full rounded border border-slate-300 px-3 py-1.5 text-xs"
              />
            </div>
          )}

          <div className="grid grid-cols-2 gap-3 pt-1">
            <div>
              <label className="block text-2xs text-[var(--color-ink-muted)] mb-1">Retry Attempts</label>
              <input
                type="number"
                min={1}
                max={5}
                value={retryCount}
                onChange={(e) => setRetryCount(Number(e.target.value))}
                className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
              />
            </div>
            <div>
              <label className="block text-2xs text-[var(--color-ink-muted)] mb-1">Timeout (seconds)</label>
              <input
                type="number"
                min={3}
                max={30}
                value={timeoutSeconds}
                onChange={(e) => setTimeoutSeconds(Number(e.target.value))}
                className="w-full rounded border border-slate-300 px-2 py-1 text-xs"
              />
            </div>
          </div>

          <label className="flex items-center gap-2 text-xs text-[var(--color-ink)] pt-2">
            <input
              type="checkbox"
              checked={isActive}
              onChange={(e) => setIsActive(e.target.checked)}
              className="h-4 w-4 rounded border-slate-300 text-[var(--color-accent)]"
            />
            <span>Active (Receive real-time event dispatches)</span>
          </label>

          <div className="flex justify-end gap-2 pt-4 border-t border-[var(--color-border)]">
            <button
              type="button"
              onClick={onClose}
              className="rounded border border-slate-300 px-3 py-1.5 text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="rounded bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] px-4 py-1.5 text-white font-medium disabled:opacity-50"
            >
              {saving ? 'Saving…' : endpoint ? 'Update Endpoint' : 'Register Webhook'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

function InspectPayloadModal({
  log,
  onClose,
}: {
  log: WebhookDeliveryLog
  onClose: () => void
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-accent)]/40 p-4 backdrop-blur-xs">
      <div className="w-full max-w-xl rounded-xl bg-[var(--color-surface)] shadow-xl border border-[var(--color-border)] overflow-hidden max-h-[90vh] flex flex-col text-xs">
        <div className="border-b border-[var(--color-border)] px-5 py-3.5 flex items-center justify-between bg-[var(--color-page)]">
          <div>
            <h3 className="text-sm font-semibold text-[var(--color-ink)]">
              Payload Delivery Details · {log.event}
            </h3>
            <span className="text-2xs text-[var(--color-ink-muted)]">ID: {log.id}</span>
          </div>
          <button onClick={onClose} className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)]">✕</button>
        </div>

        <div className="p-5 space-y-4 overflow-y-auto flex-1">
          <div className="grid grid-cols-3 gap-2 bg-[var(--color-page)] p-3 rounded-md border border-[var(--color-border)]">
            <div>
              <span className="text-3xs text-[var(--color-ink-faint)] uppercase font-semibold block">Status</span>
              <span className={`font-bold ${log.status_code >= 200 && log.status_code < 300 ? 'text-emerald-600' : 'text-red-600'}`}>
                HTTP {log.status_code}
              </span>
            </div>
            <div>
              <span className="text-3xs text-[var(--color-ink-faint)] uppercase font-semibold block">Duration</span>
              <span className="font-semibold text-slate-800">{log.duration_ms}ms</span>
            </div>
            <div>
              <span className="text-3xs text-[var(--color-ink-faint)] uppercase font-semibold block">Attempt</span>
              <span className="font-semibold text-slate-800">#{log.attempt}</span>
            </div>
          </div>

          <div>
            <h4 className="font-semibold text-slate-800 mb-1">Dispatched JSON Payload</h4>
            <pre className="rounded bg-[var(--color-accent)] p-3 text-emerald-400 font-mono text-2xs overflow-x-auto max-h-52">
              {JSON.stringify(log.payload, null, 2)}
            </pre>
          </div>

          <div>
            <h4 className="font-semibold text-slate-800 mb-1">Response Body</h4>
            <pre className="rounded bg-[var(--color-page)] border border-[var(--color-border)] p-3 text-[var(--color-ink)] font-mono text-2xs overflow-x-auto max-h-36">
              {log.response_body || log.error_message || '(Empty response body)'}
            </pre>
          </div>
        </div>

        <div className="border-t border-[var(--color-border)] px-5 py-3 flex justify-end bg-[var(--color-page)]">
          <button
            onClick={onClose}
            className="rounded bg-[var(--color-accent)] px-4 py-1.5 text-white font-medium"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}

function Banner({ tone, children }: { tone: 'error' | 'success'; children: React.ReactNode }) {
  const cls =
    tone === 'error'
      ? 'border-red-200 bg-red-50 text-red-700'
      : 'border-emerald-200 bg-emerald-50 text-emerald-700'
  return <div className={`rounded-md border px-4 py-2.5 text-xs ${cls}`}>{children}</div>
}
