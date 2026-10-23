
interface IssueModalProps {
  activeConv: any
  handleCreateLinkedIssue: any
  issueDescription: any
  issueNotice: any
  issuePriority: any
  issueProvider: any
  issueSubmitting: any
  issueTitle: any
  issueType: any
  setIssueDescription: any
  setIssueModalOpen: any
  setIssuePriority: any
  setIssueProvider: any
  setIssueTitle: any
  setIssueType: any
}

export function IssueModal(props: IssueModalProps) {
  const {
    activeConv,
    handleCreateLinkedIssue,
    issueDescription,
    issueNotice,
    issuePriority,
    issueProvider,
    issueSubmitting,
    issueTitle,
    issueType,
    setIssueDescription,
    setIssueModalOpen,
    setIssuePriority,
    setIssueProvider,
    setIssueTitle,
    setIssueType,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg rounded-2xl bg-white p-6 shadow-2xl border border-slate-200 space-y-4">
            <div className="flex items-center justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <span className="text-xl">🎯</span>
                <div>
                  <h3 className="font-bold text-sm text-slate-900">Create & Link Issue</h3>
                  <p className="text-[11px] text-slate-500">
                    Target Conversation: {activeConv?.name} ({activeConv?.channel})
                  </p>
                </div>
              </div>
              <button
                onClick={() => setIssueModalOpen(false)}
                className="text-slate-400 hover:text-slate-600 text-lg font-bold cursor-pointer"
              >
                ✕
              </button>
            </div>

            {issueNotice && (
              <div
                className={`rounded-lg p-2.5 text-xs font-medium ${
                  issueNotice.includes('Failed')
                    ? 'bg-rose-50 text-rose-800 border border-rose-200'
                    : 'bg-emerald-50 text-emerald-800 border border-emerald-200'
                }`}
              >
                {issueNotice}
              </div>
            )}

            {/* Provider Tabs */}
            <div className="flex border-b border-slate-200 gap-2">
              <button
                type="button"
                onClick={() => setIssueProvider('linear')}
                className={`py-1.5 px-3 border-b-2 font-semibold text-xs transition-colors flex items-center gap-1.5 cursor-pointer ${
                  issueProvider === 'linear'
                    ? 'border-purple-600 text-purple-700 font-bold'
                    : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
              >
                <span>🟣</span>
                <span>Linear</span>
              </button>
              <button
                type="button"
                onClick={() => setIssueProvider('github')}
                className={`py-1.5 px-3 border-b-2 font-semibold text-xs transition-colors flex items-center gap-1.5 cursor-pointer ${
                  issueProvider === 'github'
                    ? 'border-slate-800 text-slate-900 font-bold'
                    : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
              >
                <span>🐙</span>
                <span>GitHub Issues</span>
              </button>
              <button
                type="button"
                onClick={() => setIssueProvider('jira')}
                className={`py-1.5 px-3 border-b-2 font-semibold text-xs transition-colors flex items-center gap-1.5 cursor-pointer ${
                  issueProvider === 'jira'
                    ? 'border-blue-600 text-blue-700 font-bold'
                    : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
              >
                <span>🔷</span>
                <span>Jira Cloud</span>
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">Issue Title</label>
                <input
                  type="text"
                  value={issueTitle}
                  onChange={(e) => setIssueTitle(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden font-medium"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-[11px] font-bold text-slate-700 mb-1">Priority</label>
                  <select
                    value={issuePriority}
                    onChange={(e) => setIssuePriority(e.target.value)}
                    className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden"
                  >
                    <option value="urgent">🔴 Urgent</option>
                    <option value="high">🟠 High</option>
                    <option value="medium">🟡 Medium</option>
                    <option value="low">⚪ Low</option>
                  </select>
                </div>

                <div>
                  <label className="block text-[11px] font-bold text-slate-700 mb-1">Issue Type</label>
                  <select
                    value={issueType}
                    onChange={(e) => setIssueType(e.target.value)}
                    className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden"
                  >
                    <option value="bug">🐛 Bug</option>
                    <option value="feature">✨ Feature Request</option>
                    <option value="task">📋 Improvement / Task</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">
                  Description &amp; Customer Context
                </label>
                <textarea
                  rows={4}
                  value={issueDescription}
                  onChange={(e) => setIssueDescription(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-purple-500 focus:outline-hidden font-mono"
                />
              </div>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setIssueModalOpen(false)}
                className="px-3 py-1.5 text-xs text-slate-600 hover:text-slate-800 font-semibold cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleCreateLinkedIssue}
                disabled={issueSubmitting || !issueTitle.trim()}
                className="px-4 py-1.5 text-xs bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-bold shadow-xs transition-colors disabled:opacity-50 flex items-center gap-1.5 cursor-pointer"
              >
                {issueSubmitting ? 'Creating...' : `Create in ${issueProvider.toUpperCase()}`}
              </button>
            </div>
          </div>
        </div>
      
  )
}
