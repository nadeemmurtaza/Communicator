import { useEffect, useState } from 'react'
import {
  Code,
  Edit2,
  MessageSquare,
  Play,
  Plus,
  Search,
  Trash2,
  X,
  Check,
  Sparkles,
} from '../components/icons'
import { api } from '../lib/api'
import type { CannedResponse } from '../lib/api'

export function CannedResponsesManagement() {
  const [responses, setResponses] = useState<CannedResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [categoryFilter, setCategoryFilter] = useState('all')

  // Create form state
  const [code, setCode] = useState('')
  const [category, setCategory] = useState('support')
  const [content, setContent] = useState('')
  const [saving, setSaving] = useState(false)

  // Edit modal state
  const [editingItem, setEditingItem] = useState<CannedResponse | null>(null)
  const [editCode, setEditCode] = useState('')
  const [editCategory, setEditCategory] = useState('support')
  const [editContent, setEditContent] = useState('')
  const [updating, setUpdating] = useState(false)

  // Test expansion drawer/modal state
  const [testingItem, setTestingItem] = useState<CannedResponse | null>(null)
  const [testConvId, setTestConvId] = useState('')
  const [testCustomerName, setTestCustomerName] = useState('Sarah Connor')
  const [testAgentName, setTestAgentName] = useState('Alex Rivers')
  const [testChannel, setTestChannel] = useState('WhatsApp')
  const [expanding, setExpanding] = useState(false)
  const [expansionResult, setExpansionResult] = useState<{
    shortCode: string
    rawContent: string
    renderedContent: string
    variables: Record<string, string>
  } | null>(null)

  function refresh() {
    api
      .listCannedResponses({
        q: searchQuery.trim() || undefined,
        category: categoryFilter !== 'all' ? categoryFilter : undefined,
      })
      .then(setResponses)
      .finally(() => setLoading(false))
  }

  useEffect(() => {
    refresh()
  }, [searchQuery, categoryFilter])

  async function handleCreate() {
    if (!code.trim() || !content.trim()) return
    setSaving(true)
    try {
      await api.createCannedResponse({
        short_code: code.trim().replace(/^\//, ''),
        content: content.trim(),
        category: category.trim().toLowerCase() || 'general',
      })
      setCode('')
      setContent('')
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to create canned response')
    } finally {
      setSaving(false)
    }
  }

  function startEdit(item: CannedResponse) {
    setEditingItem(item)
    setEditCode((item.shortCode || item.short_code || '').replace(/^\//, ''))
    setEditCategory(item.category || 'general')
    setEditContent(item.content || '')
  }

  async function handleUpdate() {
    if (!editingItem || !editCode.trim() || !editContent.trim()) return
    setUpdating(true)
    try {
      await api.updateCannedResponse(editingItem.id, {
        short_code: editCode.trim().replace(/^\//, ''),
        content: editContent.trim(),
        category: editCategory.trim().toLowerCase() || 'general',
      })
      setEditingItem(null)
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to update canned response')
    } finally {
      setUpdating(false)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this canned response?')) return
    await api.deleteCannedResponse(id)
    refresh()
  }

  function startTestExpansion(item: CannedResponse) {
    setTestingItem(item)
    setExpansionResult(null)
    executeTestExpansion(item.shortCode || item.short_code || '')
  }

  async function executeTestExpansion(shortCodeOverride?: string) {
    const sc = shortCodeOverride || (testingItem ? testingItem.shortCode || testingItem.short_code : '')
    if (!sc) return
    setExpanding(true)
    try {
      const customVars: Record<string, string> = {
        'customer.name': testCustomerName,
        'customer.first_name': testCustomerName.split(' ')[0] || testCustomerName,
        'agent.name': testAgentName,
        'channel': testChannel,
      }

      const res = await api.expandCannedResponse({
        short_code: sc.replace(/^\//, ''),
        conversation_id: testConvId.trim() || undefined,
        custom_vars: customVars,
      })
      setExpansionResult(res)
    } catch (e: any) {
      alert(e.message || 'Failed to expand template')
    } finally {
      setExpanding(false)
    }
  }

  if (loading && responses.length === 0) {
    return <p className="text-sm text-[var(--color-ink-muted)]">Loading canned responses...</p>
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div>
          <h1 className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
            <MessageSquare className="w-5 h-5 text-[var(--color-accent)]" />
            Canned Responses &amp; Slash Commands
          </h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
            Reusable response templates, accessible in any chat composer via{' '}
            <span className="font-mono font-semibold text-[var(--color-accent)]">/shortcode</span>.
          </p>
        </div>
      </div>

      {/* Creation form */}
      <div className="glass-card p-5 space-y-3">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] flex items-center gap-2">
          <Plus className="w-4 h-4 text-[var(--color-accent)]" />
          Create New Response Template
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div className="flex items-center">
            <span className="bg-[var(--color-accent-soft)] border border-r-0 border-[var(--color-border)] rounded-l-[var(--radius-sm)] px-2.5 py-1.5 text-xs font-mono font-bold text-[var(--color-accent)]">
              /
            </span>
            <input
              type="text"
              placeholder="e.g. shipping_time"
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/[^a-zA-Z0-9_-]/g, ''))}
              className="w-full text-sm border border-[var(--color-border)] rounded-r-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
            />
          </div>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
          >
            <option value="greetings">Greetings</option>
            <option value="sales">Sales</option>
            <option value="support">Support</option>
            <option value="billing">Billing</option>
            <option value="general">General</option>
          </select>
        </div>
        <textarea
          rows={3}
          placeholder="Hi {{customer.first_name}}! Thanks for reaching out to Bay Works. How can I assist you today?"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
        />
        <div className="flex items-center justify-between">
          <p className="text-xs text-[var(--color-ink-muted)]">
            Supported variables: <code className="bg-black/5 px-1 py-0.5 rounded text-[11px] font-mono">{'{{customer.name}}'}</code>,{' '}
            <code className="bg-black/5 px-1 py-0.5 rounded text-[11px] font-mono">{'{{customer.first_name}}'}</code>,{' '}
            <code className="bg-black/5 px-1 py-0.5 rounded text-[11px] font-mono">{'{{agent.name}}'}</code>,{' '}
            <code className="bg-black/5 px-1 py-0.5 rounded text-[11px] font-mono">{'{{ticket.id}}'}</code>
          </p>
          <button
            type="button"
            onClick={handleCreate}
            disabled={saving || !code.trim() || !content.trim()}
            className="flex items-center gap-1.5 px-4 py-1.5 text-sm bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded-[var(--radius-sm)] font-semibold disabled:opacity-50 hover:bg-[var(--color-accent-hover)] transition-colors"
          >
            <Plus className="w-4 h-4" />
            {saving ? 'Saving...' : 'Save Canned Snippet'}
          </button>
        </div>
      </div>

      {/* Snippet list with filters */}
      <div className="glass-card p-5 space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <h2 className="text-sm font-semibold text-[var(--color-ink)]">
            {responses.length} snippet{responses.length === 1 ? '' : 's'} configured
          </h2>
          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="w-3.5 h-3.5 text-[var(--color-ink-muted)] absolute left-2.5 top-2.5" />
              <input
                type="text"
                placeholder="Search shortcode or text..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="text-xs pl-8 pr-3 py-1.5 border border-[var(--color-border)] rounded-[var(--radius-sm)] bg-[var(--color-surface)] w-48 sm:w-56"
              />
            </div>
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value)}
              className="text-xs border border-[var(--color-border)] rounded-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
            >
              <option value="all">All Categories</option>
              <option value="greetings">Greetings</option>
              <option value="sales">Sales</option>
              <option value="support">Support</option>
              <option value="billing">Billing</option>
              <option value="general">General</option>
            </select>
          </div>
        </div>

        <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)]">
          {responses.length === 0 ? (
            <p className="p-6 text-center text-sm text-[var(--color-ink-muted)]">No canned responses found.</p>
          ) : (
            responses.map((c) => {
              const displayCode = (c.shortCode || c.short_code || '').replace(/^\//, '')
              return (
                <div
                  key={c.id || displayCode}
                  className="p-3.5 hover:bg-black/5 transition-colors flex items-start justify-between gap-4"
                >
                  <div className="space-y-1.5 flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-xs font-bold text-[var(--color-accent)] bg-[var(--color-accent-soft)] px-2 py-0.5 rounded">
                        /{displayCode}
                      </span>
                      <span className="text-xs uppercase tracking-wide px-2 py-0.5 rounded-full bg-black/5 text-[var(--color-ink-muted)] font-medium">
                        {c.category || 'general'}
                      </span>
                    </div>
                    <p className="text-sm text-[var(--color-ink)] break-words whitespace-pre-wrap">{c.content}</p>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    <button
                      type="button"
                      onClick={() => startTestExpansion(c)}
                      className="flex items-center gap-1 text-xs text-[var(--color-accent)] hover:bg-[var(--color-accent-soft)] px-2.5 py-1 rounded transition-colors"
                      title="Test variable expansion"
                    >
                      <Play className="w-3.5 h-3.5" />
                      Test
                    </button>
                    <button
                      type="button"
                      onClick={() => startEdit(c)}
                      className="flex items-center gap-1 text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] px-2.5 py-1 rounded hover:bg-black/5 transition-colors"
                      title="Edit response"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                      Edit
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDelete(c.id)}
                      className="flex items-center gap-1 text-xs text-[var(--color-ink-muted)] hover:text-red-600 px-2 py-1 rounded hover:bg-red-50 transition-colors"
                      title="Delete response"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              )
            })
          )}
        </div>
      </div>

      {/* Edit Modal */}
      {editingItem && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-lg w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <h3 className="font-semibold text-base text-[var(--color-ink)] flex items-center gap-2">
                <Edit2 className="w-4 h-4 text-[var(--color-accent)]" />
                Edit Canned Response
              </h3>
              <button
                type="button"
                onClick={() => setEditingItem(null)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)] mb-1 block">Short Code</label>
                  <div className="flex items-center">
                    <span className="bg-[var(--color-accent-soft)] border border-r-0 border-[var(--color-border)] rounded-l-[var(--radius-sm)] px-2.5 py-1.5 text-xs font-mono font-bold text-[var(--color-accent)]">
                      /
                    </span>
                    <input
                      type="text"
                      value={editCode}
                      onChange={(e) => setEditCode(e.target.value.replace(/[^a-zA-Z0-9_-]/g, ''))}
                      className="w-full text-sm border border-[var(--color-border)] rounded-r-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
                    />
                  </div>
                </div>

                <div>
                  <label className="text-xs font-semibold text-[var(--color-ink-muted)] mb-1 block">Category</label>
                  <select
                    value={editCategory}
                    onChange={(e) => setEditCategory(e.target.value)}
                    className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-2.5 py-1.5 bg-[var(--color-surface)]"
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
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] mb-1 block">Template Content</label>
                <textarea
                  rows={4}
                  value={editContent}
                  onChange={(e) => setEditContent(e.target.value)}
                  className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
                />
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setEditingItem(null)}
                className="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-black/5"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleUpdate}
                disabled={updating || !editCode.trim() || !editContent.trim()}
                className="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
              >
                <Check className="w-3.5 h-3.5" />
                {updating ? 'Updating...' : 'Save Changes'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Test Expansion Modal */}
      {testingItem && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-xl w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <div className="flex items-center gap-2">
                <Sparkles className="w-5 h-5 text-[var(--color-accent)]" />
                <h3 className="font-semibold text-base text-[var(--color-ink)]">
                  Live Variable Expansion Test: <span className="font-mono text-sm text-[var(--color-accent)]">/{testingItem.shortCode || testingItem.short_code}</span>
                </h3>
              </div>
              <button
                type="button"
                onClick={() => setTestingItem(null)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <div>
                  <label className="text-[11px] font-semibold text-[var(--color-ink-muted)] block">Customer Name</label>
                  <input
                    type="text"
                    value={testCustomerName}
                    onChange={(e) => setTestCustomerName(e.target.value)}
                    className="w-full text-xs border border-[var(--color-border)] rounded px-2 py-1 bg-[var(--color-surface)]"
                  />
                </div>
                <div>
                  <label className="text-[11px] font-semibold text-[var(--color-ink-muted)] block">Agent Name</label>
                  <input
                    type="text"
                    value={testAgentName}
                    onChange={(e) => setTestAgentName(e.target.value)}
                    className="w-full text-xs border border-[var(--color-border)] rounded px-2 py-1 bg-[var(--color-surface)]"
                  />
                </div>
                <div>
                  <label className="text-[11px] font-semibold text-[var(--color-ink-muted)] block">Channel</label>
                  <input
                    type="text"
                    value={testChannel}
                    onChange={(e) => setTestChannel(e.target.value)}
                    className="w-full text-xs border border-[var(--color-border)] rounded px-2 py-1 bg-[var(--color-surface)]"
                  />
                </div>
              </div>

              <div>
                <label className="text-[11px] font-semibold text-[var(--color-ink-muted)] block">Optional Conversation ID</label>
                <input
                  type="text"
                  placeholder="Paste UUID to test real database lookup..."
                  value={testConvId}
                  onChange={(e) => setTestConvId(e.target.value)}
                  className="w-full text-xs font-mono border border-[var(--color-border)] rounded px-2 py-1 bg-[var(--color-surface)]"
                />
              </div>

              <div className="flex justify-end">
                <button
                  type="button"
                  onClick={() => executeTestExpansion()}
                  disabled={expanding}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
                >
                  <Play className="w-3.5 h-3.5" />
                  {expanding ? 'Expanding...' : 'Re-run Expansion'}
                </button>
              </div>

              {expansionResult && (
                <div className="space-y-3 pt-2 border-t border-[var(--color-border)]">
                  <div>
                    <span className="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] block mb-1">
                      Raw Template
                    </span>
                    <pre className="text-xs p-2.5 bg-black/5 rounded text-[var(--color-ink)] font-mono whitespace-pre-wrap">
                      {expansionResult.rawContent}
                    </pre>
                  </div>

                  <div>
                    <span className="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-accent)] block mb-1 flex items-center gap-1">
                      <Code className="w-3.5 h-3.5" />
                      Rendered Output (Ready to Send)
                    </span>
                    <div className="text-sm p-3 bg-[var(--color-accent-soft)]/40 border border-[var(--color-accent)]/20 rounded text-[var(--color-ink)] whitespace-pre-wrap">
                      {expansionResult.renderedContent}
                    </div>
                  </div>

                  <div>
                    <span className="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] block mb-1">
                      Resolved Variable Mappings
                    </span>
                    <div className="grid grid-cols-2 gap-1.5 max-h-32 overflow-y-auto text-xs p-2 bg-black/5 rounded">
                      {Object.entries(expansionResult.variables).map(([k, v]) => (
                        <div key={k} className="flex items-center gap-1 text-[11px] truncate">
                          <span className="font-mono text-[var(--color-ink-muted)] truncate">{k}:</span>
                          <span className="font-semibold text-[var(--color-ink)] truncate">{v}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>

            <div className="flex justify-end pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setTestingItem(null)}
                className="px-3 py-1 text-xs rounded border border-[var(--color-border)] hover:bg-black/5"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
