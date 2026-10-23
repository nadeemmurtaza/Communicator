import { Link } from 'react-router-dom'

interface CannedResponsesModalProps {
  cannedResponses: any
  handleCreateCanned: any
  handleDeleteCanned: any
  isSavingCanned: any
  newCannedCategory: any
  newCannedCode: any
  newCannedContent: any
  resolveCannedVariables: any
  setNewCannedCategory: any
  setNewCannedCode: any
  setNewCannedContent: any
  setShowCannedModal: any
}

export function CannedResponsesModal(props: CannedResponsesModalProps) {
  const {
    cannedResponses,
    handleCreateCanned,
    handleDeleteCanned,
    isSavingCanned,
    newCannedCategory,
    newCannedCode,
    newCannedContent,
    resolveCannedVariables,
    setNewCannedCategory,
    setNewCannedCode,
    setNewCannedContent,
    setShowCannedModal,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-ink)]/40 backdrop-blur-xs p-4">
          <div className="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] shadow-2xl max-w-3xl w-full p-6 space-y-5 animate-in fade-in zoom-in-95 max-h-[85vh] overflow-y-auto">
            <div className="flex items-start justify-between border-b border-[var(--color-border)] pb-3">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-blue-100 text-blue-700 flex items-center justify-center font-bold text-sm">
                  ⚡
                </div>
                <div>
                  <h3 className="font-bold text-[var(--color-ink)] text-base">Canned Responses &amp; Slash Commands</h3>
                  <p className="text-xs text-[var(--color-ink-muted)]">
                    Create reusable response templates accessible in any chat composer via <span className="font-mono font-bold text-blue-600">/shortcode</span>
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <Link to="/canned-responses" className="text-xs font-semibold text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] underline">
                  Open full page →
                </Link>
                <button
                  onClick={() => setShowCannedModal(false)}
                  className="text-[var(--color-ink-faint)] hover:text-[var(--color-ink-muted)] font-bold text-sm p-1 cursor-pointer"
                >
                  ✕
                </button>
              </div>
            </div>

            {/* Create New Canned Snippet Form */}
            <div className="bg-blue-50/50 border border-blue-200/70 rounded-xl p-4 space-y-3">
              <h4 className="text-xs font-bold text-blue-900 uppercase tracking-wider flex items-center gap-1.5">
                <span>➕ Create New Canned Snippet</span>
              </h4>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label className="block text-[11px] font-semibold text-[var(--color-ink)] mb-1">
                    Shortcode (Command) <span className="text-rose-500">*</span>
                  </label>
                  <div className="flex items-center">
                    <span className="bg-[var(--color-accent-soft)] border border-r-0 border-[var(--color-border)] rounded-l-lg px-2.5 py-1.5 text-xs font-mono font-bold text-[var(--color-ink-muted)]">
                      /
                    </span>
                    <input
                      type="text"
                      placeholder="e.g. shipping_time"
                      value={newCannedCode}
                      onChange={(e) => setNewCannedCode(e.target.value.replace(/[^a-zA-Z0-9_-]/g, ''))}
                      className="w-full text-xs border border-[var(--color-border)] rounded-r-lg px-2.5 py-1.5 bg-[var(--color-surface)] focus:outline-hidden focus:border-blue-500"
                    />
                  </div>
                </div>

                <div>
                  <label className="block text-[11px] font-semibold text-[var(--color-ink)] mb-1">
                    Category
                  </label>
                  <select
                    value={newCannedCategory}
                    onChange={(e) => setNewCannedCategory(e.target.value)}
                    className="w-full text-xs border border-[var(--color-border)] rounded-lg px-2.5 py-1.5 bg-[var(--color-surface)] focus:outline-hidden focus:border-blue-500"
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
                  <label className="text-[11px] font-semibold text-[var(--color-ink)]">
                    Template Content <span className="text-rose-500">*</span>
                  </label>
                  <div className="flex items-center gap-1 text-[10.5px]">
                    <span className="text-[var(--color-ink-faint)]">Insert tag:</span>
                    {['customer.first_name', 'customer.name', 'agent.name', 'ticket.id', 'channel'].map((tag) => (
                      <button
                        key={tag}
                        type="button"
                        onClick={() => setNewCannedContent((prev: any) => prev + `{{${tag}}}`)}
                        className="bg-[var(--color-surface)] border border-[var(--color-border)] hover:border-blue-400 text-blue-600 font-mono px-1.5 py-0.5 rounded cursor-pointer transition-colors"
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
                  className="w-full text-xs border border-[var(--color-border)] rounded-lg px-3 py-2 bg-[var(--color-surface)] focus:outline-hidden focus:border-blue-500 font-normal"
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
              <h4 className="text-xs font-bold text-[var(--color-ink)] uppercase tracking-wider flex items-center justify-between">
                <span>Existing Canned Replies</span>
                <span className="text-[11px] font-normal text-[var(--color-ink-muted)]">{cannedResponses.length} snippets</span>
              </h4>

              <div className="border border-[var(--color-border)] rounded-xl overflow-hidden divide-y divide-[var(--color-border)] max-h-72 overflow-y-auto">
                {cannedResponses.map((c: any) => (
                  <div key={c.id || c.shortCode || c.short_code} className="p-3 hover:bg-[var(--color-page)]/70 transition-colors flex items-start justify-between gap-3">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-black text-blue-700 bg-blue-100/80 px-2 py-0.5 rounded-md">
                          /{c.shortCode || c.short_code}
                        </span>
                        <span className="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded-full bg-[var(--color-accent-soft)] text-[var(--color-ink-muted)] border border-[var(--color-border)]">
                          {c.category || 'general'}
                        </span>
                      </div>
                      <p className="text-xs text-[var(--color-ink)] font-normal leading-relaxed">
                        {c.content}
                      </p>
                      <p className="text-[11px] text-[var(--color-ink-faint)] italic">
                        Preview: {resolveCannedVariables(c.content)}
                      </p>
                    </div>

                    <button
                      type="button"
                      onClick={() => handleDeleteCanned(c.id)}
                      title="Delete canned response"
                      className="text-[var(--color-ink-faint)] hover:text-rose-600 text-xs p-1 rounded hover:bg-rose-50 transition-colors cursor-pointer shrink-0"
                    >
                      🗑️
                    </button>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between pt-2 border-t border-[var(--color-border)] text-xs text-[var(--color-ink-muted)]">
              <span>💡 All canned replies automatically resolve customer and agent names on insert.</span>
              <button
                type="button"
                onClick={() => setShowCannedModal(false)}
                className="px-4 py-1.5 text-xs bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      
  )
}
