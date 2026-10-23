
interface CreateMacroModalProps {
  conversations: any
  handleCreateMacro: any
  newMacroCSAT: any
  newMacroDesc: any
  newMacroMessage: any
  newMacroName: any
  newMacroNote: any
  newMacroPriority: any
  newMacroResolve: any
  newMacroTag: any
  setNewMacroCSAT: any
  setNewMacroDesc: any
  setNewMacroMessage: any
  setNewMacroName: any
  setNewMacroNote: any
  setNewMacroPriority: any
  setNewMacroResolve: any
  setNewMacroTag: any
  setShowCreateMacroModal: any
}

export function CreateMacroModal(props: CreateMacroModalProps) {
  const {
    handleCreateMacro,
    newMacroCSAT,
    newMacroDesc,
    newMacroMessage,
    newMacroName,
    newMacroNote,
    newMacroPriority,
    newMacroResolve,
    newMacroTag,
    setNewMacroCSAT,
    setNewMacroDesc,
    setNewMacroMessage,
    setNewMacroName,
    setNewMacroNote,
    setNewMacroPriority,
    setNewMacroResolve,
    setNewMacroTag,
    setShowCreateMacroModal,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-lg w-full p-6 space-y-4 animate-in fade-in zoom-in-95">
            <div className="flex items-start justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center font-bold text-sm">
                  ⚡
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 text-base">Create One-Click Macro</h3>
                  <p className="text-xs text-slate-500">Automate multi-action sequences across conversations</p>
                </div>
              </div>
              <button
                onClick={() => setShowCreateMacroModal(false)}
                className="text-slate-400 hover:text-slate-600 font-bold text-sm p-1 cursor-pointer"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleCreateMacro} className="space-y-3.5">
              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Macro Name *</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. VIP Priority Escalation + Manager Note"
                  value={newMacroName}
                  onChange={(e) => setNewMacroName(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-brand-500 focus:outline-hidden"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Description</label>
                <input
                  type="text"
                  placeholder="Explains what this workflow executes"
                  value={newMacroDesc}
                  onChange={(e) => setNewMacroDesc(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:ring-2 focus:ring-brand-500 focus:outline-hidden"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Set Priority</label>
                  <select
                    value={newMacroPriority}
                    onChange={(e) => setNewMacroPriority(e.target.value as any)}
                    className="w-full px-2.5 py-1.5 border border-slate-300 rounded-lg text-xs bg-white focus:outline-hidden"
                  >
                    <option value="low">Low</option>
                    <option value="medium">Medium</option>
                    <option value="high">High</option>
                    <option value="urgent">Urgent</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-semibold text-slate-700 mb-1">Add Tags</label>
                  <input
                    type="text"
                    placeholder="e.g. vip, follow-up"
                    value={newMacroTag}
                    onChange={(e) => setNewMacroTag(e.target.value)}
                    className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:outline-hidden"
                  />
                </div>
              </div>

              <div className="flex items-center gap-4 pt-1">
                <label className="flex items-center gap-1.5 text-xs text-slate-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={newMacroResolve}
                    onChange={(e) => setNewMacroResolve(e.target.checked)}
                    className="rounded text-brand-600 focus:ring-brand-500"
                  />
                  <span>Resolve conversation</span>
                </label>
                <label className="flex items-center gap-1.5 text-xs text-slate-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={newMacroCSAT}
                    onChange={(e) => setNewMacroCSAT(e.target.checked)}
                    className="rounded text-brand-600 focus:ring-brand-500"
                  />
                  <span>Send CSAT survey</span>
                </label>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Customer Outbound Message (optional)</label>
                <textarea
                  rows={2}
                  placeholder="Automated reply sent directly to the customer..."
                  value={newMacroMessage}
                  onChange={(e) => setNewMacroMessage(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:outline-hidden resize-none"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-700 mb-1">Internal Private Note (optional)</label>
                <textarea
                  rows={2}
                  placeholder="Staff-only audit note attached to thread..."
                  value={newMacroNote}
                  onChange={(e) => setNewMacroNote(e.target.value)}
                  className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs focus:outline-hidden resize-none"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
                <button
                  type="button"
                  onClick={() => setShowCreateMacroModal(false)}
                  className="px-3 py-1.5 text-xs text-slate-600 hover:bg-slate-100 rounded-lg font-medium transition-colors cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 text-xs bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold shadow-xs transition-colors cursor-pointer"
                >
                  Save Macro
                </button>
              </div>
            </form>
          </div>
        </div>
      
  )
}
