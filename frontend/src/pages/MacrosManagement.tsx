import { useEffect, useState } from 'react'
import {
  Check,
  CheckCircle2,
  Edit2,
  Play,
  Plus,
  Sparkles,
  Tag,
  Trash2,
  X,
} from 'lucide-react'
import { api } from '../lib/api'
import type { MacroAction, MacroItem, MacroExecutionResult } from '../lib/api'

export function MacrosManagement() {
  const [macros, setMacros] = useState<MacroItem[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreate, setShowCreate] = useState(false)

  // Create form state
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [resolve, setResolve] = useState(false)
  const [sendCsat, setSendCsat] = useState(false)
  const [tags, setTags] = useState('')
  const [message, setMessage] = useState('')
  const [note, setNote] = useState('')
  const [saving, setSaving] = useState(false)

  // Edit modal state
  const [editingMacro, setEditingMacro] = useState<MacroItem | null>(null)
  const [editName, setEditName] = useState('')
  const [editDescription, setEditDescription] = useState('')
  const [editResolve, setEditResolve] = useState(false)
  const [editSendCsat, setEditSendCsat] = useState(false)
  const [editTags, setEditTags] = useState('')
  const [editMessage, setEditMessage] = useState('')
  const [editNote, setEditNote] = useState('')
  const [updating, setUpdating] = useState(false)

  // Test / Execute Macro state
  const [executingMacro, setExecutingMacro] = useState<MacroItem | null>(null)
  const [testConvId, setTestConvId] = useState('')
  const [agentName, setAgentName] = useState('Riley Brooks')
  const [running, setRunning] = useState(false)
  const [executionResult, setExecutionResult] = useState<MacroExecutionResult | null>(null)

  function refresh() {
    api
      .listMacros()
      .then(setMacros)
      .finally(() => setLoading(false))
  }

  useEffect(refresh, [])

  function buildActions(
    res: boolean,
    csat: boolean,
    tagStr: string,
    msg: string,
    noteStr: string
  ): MacroAction[] {
    const actions: MacroAction[] = []
    if (res) actions.push({ type: 'resolve_conversation', resolution_type: 'human_assisted' })
    if (csat) actions.push({ type: 'send_csat_survey' })
    if (tagStr.trim()) {
      actions.push({
        type: 'add_tags',
        tags: tagStr
          .split(',')
          .map((t) => t.trim().toLowerCase())
          .filter(Boolean),
      })
    }
    if (msg.trim()) actions.push({ type: 'send_message', message: msg.trim() })
    if (noteStr.trim()) actions.push({ type: 'add_private_note', note: noteStr.trim() })
    return actions
  }

  const actionsPreview = buildActions(resolve, sendCsat, tags, message, note)
  const editActionsPreview = buildActions(
    editResolve,
    editSendCsat,
    editTags,
    editMessage,
    editNote
  )

  async function handleCreate() {
    if (!name.trim() || actionsPreview.length === 0) return
    setSaving(true)
    try {
      await api.createMacro({
        name: name.trim(),
        description: description.trim(),
        actions: actionsPreview,
      })
      setName('')
      setDescription('')
      setResolve(false)
      setSendCsat(false)
      setTags('')
      setMessage('')
      setNote('')
      setShowCreate(false)
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to create macro')
    } finally {
      setSaving(false)
    }
  }

  function startEdit(macro: MacroItem) {
    setEditingMacro(macro)
    setEditName(macro.name || '')
    setEditDescription(macro.description || '')

    const acts = Array.isArray(macro.actions) ? macro.actions : []
    setEditResolve(acts.some((a) => a.type === 'resolve_conversation'))
    setEditSendCsat(acts.some((a) => a.type === 'send_csat_survey'))

    const tagAction = acts.find((a) => a.type === 'add_tags')
    setEditTags(tagAction && Array.isArray(tagAction.tags) ? tagAction.tags.join(', ') : '')

    const msgAction = acts.find((a) => a.type === 'send_message')
    setEditMessage(msgAction?.message || '')

    const noteAction = acts.find((a) => a.type === 'add_private_note')
    setEditNote(noteAction?.note || '')
  }

  async function handleUpdate() {
    if (!editingMacro || !editName.trim() || editActionsPreview.length === 0) return
    setUpdating(true)
    try {
      await api.updateMacro(editingMacro.id, {
        name: editName.trim(),
        description: editDescription.trim(),
        actions: editActionsPreview,
      })
      setEditingMacro(null)
      refresh()
    } catch (e: any) {
      alert(e.message || 'Failed to update macro')
    } finally {
      setUpdating(false)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this macro?')) return
    await api.deleteMacro(id)
    refresh()
  }

  function openExecuteModal(macro: MacroItem) {
    setExecutingMacro(macro)
    setExecutionResult(null)
  }

  async function handleExecute() {
    if (!executingMacro || !testConvId.trim()) {
      alert('Please enter a conversation ID to execute this macro against.')
      return
    }
    setRunning(true)
    try {
      const res = await api.executeMacro(testConvId.trim(), executingMacro.id, agentName.trim())
      setExecutionResult(res)
    } catch (e: any) {
      alert(e.message || 'Failed to execute macro')
    } finally {
      setRunning(false)
    }
  }

  if (loading && macros.length === 0) {
    return <p className="text-sm text-[var(--color-ink-muted)]">Loading macros...</p>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-[var(--color-accent)]" />
            Macros &amp; One-Click Automations
          </h1>
          <p className="text-sm text-[var(--color-ink-muted)] mt-0.5">
            One-click bundles of actions (reply, tag, resolve, team assignment) triggered from conversations.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setShowCreate((v) => !v)}
          className="flex items-center gap-1.5 px-4 py-2 rounded-[var(--radius-sm)] bg-[var(--color-accent)] text-[var(--color-on-accent)] font-semibold text-sm hover:bg-[var(--color-accent-hover)] transition-colors"
        >
          {showCreate ? (
            <>
              <X className="w-4 h-4" /> Cancel
            </>
          ) : (
            <>
              <Plus className="w-4 h-4" /> New Macro
            </>
          )}
        </button>
      </div>

      {showCreate && (
        <div className="glass-card p-5 space-y-3">
          <h2 className="text-sm font-semibold text-[var(--color-ink)] flex items-center gap-2">
            <Plus className="w-4 h-4 text-[var(--color-accent)]" />
            Build New Action Macro
          </h2>
          <input
            type="text"
            placeholder="Macro name (e.g. VIP Resolution & CSAT Survey)"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />
          <textarea
            rows={2}
            placeholder="Description explaining when agents should trigger this macro..."
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />

          <p className="text-xs font-semibold text-[var(--color-ink-muted)] pt-1">
            Actions to execute simultaneously (select at least one):
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <label className="flex items-center gap-2 text-sm text-[var(--color-ink)] cursor-pointer">
              <input type="checkbox" checked={resolve} onChange={(e) => setResolve(e.target.checked)} />
              Resolve conversation (closed_won)
            </label>
            <label className="flex items-center gap-2 text-sm text-[var(--color-ink)] cursor-pointer">
              <input type="checkbox" checked={sendCsat} onChange={(e) => setSendCsat(e.target.checked)} />
              Dispatch CSAT survey rating request
            </label>
          </div>
          <input
            type="text"
            placeholder="Add tags (comma-separated, e.g. vip, discount-applied)"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />
          <textarea
            rows={2}
            placeholder="Send outbound reply message to customer..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />
          <textarea
            rows={2}
            placeholder="Add internal private note..."
            value={note}
            onChange={(e) => setNote(e.target.value)}
            className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
          />

          <div className="flex items-center justify-between pt-1">
            <p className="text-xs text-[var(--color-ink-muted)]">
              {actionsPreview.length === 0 ? 'Select at least one action.' : `${actionsPreview.length} action(s) configured`}
            </p>
            <button
              type="button"
              onClick={handleCreate}
              disabled={saving || !name.trim() || actionsPreview.length === 0}
              className="flex items-center gap-1.5 px-4 py-1.5 text-sm bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded-[var(--radius-sm)] font-semibold disabled:opacity-50 hover:bg-[var(--color-accent-hover)] transition-colors"
            >
              <Plus className="w-4 h-4" />
              {saving ? 'Saving...' : 'Save Macro'}
            </button>
          </div>
        </div>
      )}

      {/* Macro list */}
      <div className="glass-card p-5">
        <h2 className="text-sm font-semibold text-[var(--color-ink)] mb-3">
          {macros.length} macro{macros.length === 1 ? '' : 's'} configured
        </h2>
        <div className="border border-[var(--color-border)] rounded-[var(--radius-sm)] overflow-hidden divide-y divide-[var(--color-border)] text-sm">
          {macros.length === 0 ? (
            <p className="p-6 text-center text-sm text-[var(--color-ink-muted)]">No macros found.</p>
          ) : (
            macros.map((m) => {
              const acts = Array.isArray(m.actions) ? m.actions : []
              return (
                <div key={m.id} className="p-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-3 hover:bg-black/5 transition-colors">
                  <div className="space-y-1 flex-1 min-w-0">
                    <p className="font-semibold text-[var(--color-ink)] truncate">{m.name}</p>
                    <p className="text-xs text-[var(--color-ink-muted)] line-clamp-2">{m.description || 'No description'}</p>
                    <div className="flex flex-wrap items-center gap-1.5 pt-1">
                      {acts.map((act, i) => (
                        <span
                          key={i}
                          className="text-[11px] px-2 py-0.5 rounded bg-black/5 font-mono text-[var(--color-ink-muted)] flex items-center gap-1"
                        >
                          <Tag className="w-3 h-3 text-[var(--color-accent)]" />
                          {act.type.replace(/_/g, ' ')}
                        </span>
                      ))}
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0 self-end sm:self-center">
                    <button
                      type="button"
                      onClick={() => openExecuteModal(m)}
                      className="flex items-center gap-1 text-xs text-[var(--color-accent)] hover:bg-[var(--color-accent-soft)] px-2.5 py-1 rounded transition-colors"
                      title="Test / Execute Macro"
                    >
                      <Play className="w-3.5 h-3.5" />
                      Run
                    </button>
                    <button
                      type="button"
                      onClick={() => startEdit(m)}
                      className="flex items-center gap-1 text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] px-2.5 py-1 rounded hover:bg-black/5 transition-colors"
                      title="Edit Macro"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                      Edit
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDelete(m.id)}
                      className="flex items-center gap-1 text-xs text-[var(--color-ink-muted)] hover:text-red-600 px-2 py-1 rounded hover:bg-red-50 transition-colors"
                      title="Delete Macro"
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
      {editingMacro && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-lg w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <h3 className="font-semibold text-base text-[var(--color-ink)] flex items-center gap-2">
                <Edit2 className="w-4 h-4 text-[var(--color-accent)]" />
                Edit Macro: {editingMacro.name}
              </h3>
              <button
                type="button"
                onClick={() => setEditingMacro(null)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <input
                type="text"
                placeholder="Macro name"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
              />
              <textarea
                rows={2}
                placeholder="Description"
                value={editDescription}
                onChange={(e) => setEditDescription(e.target.value)}
                className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
              />

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
                <label className="flex items-center gap-2 text-sm text-[var(--color-ink)] cursor-pointer">
                  <input type="checkbox" checked={editResolve} onChange={(e) => setEditResolve(e.target.checked)} />
                  Resolve conversation
                </label>
                <label className="flex items-center gap-2 text-sm text-[var(--color-ink)] cursor-pointer">
                  <input type="checkbox" checked={editSendCsat} onChange={(e) => setEditSendCsat(e.target.checked)} />
                  Send CSAT survey
                </label>
              </div>
              <input
                type="text"
                placeholder="Add tags (comma-separated)"
                value={editTags}
                onChange={(e) => setEditTags(e.target.value)}
                className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
              />
              <textarea
                rows={2}
                placeholder="Send message to customer"
                value={editMessage}
                onChange={(e) => setEditMessage(e.target.value)}
                className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
              />
              <textarea
                rows={2}
                placeholder="Add private note"
                value={editNote}
                onChange={(e) => setEditNote(e.target.value)}
                className="w-full text-sm border border-[var(--color-border)] rounded-[var(--radius-sm)] px-3 py-2 bg-[var(--color-surface)]"
              />
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setEditingMacro(null)}
                className="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-black/5"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleUpdate}
                disabled={updating || !editName.trim() || editActionsPreview.length === 0}
                className="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
              >
                <Check className="w-3.5 h-3.5" />
                {updating ? 'Updating...' : 'Save Changes'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Execute / Test Macro Modal */}
      {executingMacro && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="glass-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[var(--radius-md)] max-w-md w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between pb-2 border-b border-[var(--color-border)]">
              <h3 className="font-semibold text-base text-[var(--color-ink)] flex items-center gap-2">
                <Play className="w-4 h-4 text-[var(--color-accent)]" />
                Execute Macro: {executingMacro.name}
              </h3>
              <button
                type="button"
                onClick={() => setExecutingMacro(null)}
                className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                  Target Conversation ID
                </label>
                <input
                  type="text"
                  placeholder="Enter Conversation UUID..."
                  value={testConvId}
                  onChange={(e) => setTestConvId(e.target.value)}
                  className="w-full text-xs font-mono border border-[var(--color-border)] rounded px-2.5 py-1.5 bg-[var(--color-surface)]"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-[var(--color-ink-muted)] block mb-1">
                  Executing Agent Name
                </label>
                <input
                  type="text"
                  value={agentName}
                  onChange={(e) => setAgentName(e.target.value)}
                  className="w-full text-xs border border-[var(--color-border)] rounded px-2.5 py-1.5 bg-[var(--color-surface)]"
                />
              </div>

              {executionResult && (
                <div className="p-3 bg-[var(--color-accent-soft)]/50 border border-[var(--color-accent)]/20 rounded space-y-2 text-xs">
                  <div className="flex items-center gap-1.5 text-emerald-700 font-semibold">
                    <CheckCircle2 className="w-4 h-4" />
                    Macro Executed Successfully!
                  </div>
                  <div className="space-y-1 text-[var(--color-ink)]">
                    <p>Actions executed: <span className="font-semibold">{executionResult.executedActionsCount}</span></p>
                    <p>New State: <span className="font-semibold">{executionResult.conversationState}</span></p>
                    <p>Priority: <span className="font-semibold">{executionResult.conversationPriority}</span></p>
                    <p>Tags: <span className="font-semibold">{executionResult.conversationTags.join(', ') || 'none'}</span></p>
                  </div>
                </div>
              )}
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
              <button
                type="button"
                onClick={() => setExecutingMacro(null)}
                className="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-black/5"
              >
                Close
              </button>
              <button
                type="button"
                onClick={handleExecute}
                disabled={running || !testConvId.trim()}
                className="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-[var(--color-accent)] text-[var(--color-on-accent)] rounded font-semibold disabled:opacity-50"
              >
                <Play className="w-3.5 h-3.5" />
                {running ? 'Executing...' : 'Run Macro'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
