import { useState, useEffect } from 'react'
import {
  getWorkflowSummary,
  listWorkflows,
  createWorkflow,
  updateWorkflow,
  deleteWorkflow,
  triggerWorkflowExecution,
  listWorkflowExecutions,
  getAvailableTriggers,
  type WorkflowBuilderWorkflow,
  type WorkflowExecution,
  type WorkflowSummary as WorkflowSummaryType,
} from '../lib/api'
import { GitBranch, Play, Plus, Trash2, Eye, AlertCircle, CheckCircle, Clock } from '../components/icons'

export default function WorkflowBuilder() {
  const [activeTab, setActiveTab] = useState<'overview' | 'workflows' | 'executions'>('overview')
  const [summary, setSummary] = useState<WorkflowSummaryType | null>(null)
  const [workflows, setWorkflows] = useState<WorkflowBuilderWorkflow[]>([])
  const [executions, setExecutions] = useState<WorkflowExecution[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [selectedWorkflow, setSelectedWorkflow] = useState<WorkflowBuilderWorkflow | null>(null)
  const [filterType, setFilterType] = useState('')
  const [filterStatus, setFilterStatus] = useState('')

  // Create workflow form
  const [newName, setNewName] = useState('')
  const [newType, setNewType] = useState('automation')
  const [newTrigger, setNewTrigger] = useState('event')
  const [newDesc, setNewDesc] = useState('')

  useEffect(() => {
    loadData()
  }, [activeTab, filterType, filterStatus])

  async function loadData() {
    setLoading(true)
    try {
      const [summaryRes, workflowsRes, executionsRes] = await Promise.all([
        getWorkflowSummary(),
        listWorkflows(50, 0, filterType || undefined, filterStatus || undefined),
        listWorkflowExecutions(20, 0),
      ])
      setSummary(summaryRes)
      setWorkflows(workflowsRes.workflows || [])
      setExecutions(executionsRes.executions || [])
    } catch (e) {
      console.error('Failed to load workflow data:', e)
    }
    setLoading(false)
  }

  async function handleCreate() {
    if (!newName.trim()) return
    try {
      await createWorkflow({
        organizationId: 'org-default',
        name: newName.trim(),
        workflowType: newType,
        triggerType: newTrigger,
        description: newDesc.trim() || undefined,
      })
      setShowCreateModal(false)
      setNewName('')
      setNewDesc('')
      loadData()
    } catch (e) {
      console.error('Failed to create workflow:', e)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this workflow?')) return
    try {
      await deleteWorkflow(id)
      loadData()
    } catch (e) {
      console.error('Failed to delete:', e)
    }
  }

  async function handleToggleStatus(wf: WorkflowBuilderWorkflow) {
    try {
      const newStatus = wf.status === 'active' ? 'paused' : 'active'
      await updateWorkflow(wf.id, { status: newStatus })
      loadData()
    } catch (e) {
      console.error('Failed to update status:', e)
    }
  }

  async function handleExecute(wf: WorkflowBuilderWorkflow) {
    try {
      const result = await triggerWorkflowExecution(wf.id, {
        triggerSource: 'manual',
      })
      alert(`Execution ${result.status}: ${result.stepsCompleted}/${result.stepsTotal} steps`)
      loadData()
    } catch (e) {
      console.error('Failed to execute:', e)
    }
  }

  function statusBadge(status: string) {
    const colors: Record<string, string> = {
      active: 'bg-green-100 text-green-800',
      draft: 'bg-gray-100 text-gray-800',
      paused: 'bg-yellow-100 text-yellow-800',
      archived: 'bg-red-100 text-red-800',
    }
    return (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[status] || colors.draft}`}>
        {status}
      </span>
    )
  }

  function typeBadge(type: string) {
    const colors: Record<string, string> = {
      automation: 'bg-blue-100 text-blue-800',
      assignment: 'bg-purple-100 text-purple-800',
      notification: 'bg-orange-100 text-orange-800',
      escalation: 'bg-red-100 text-red-800',
      custom: 'bg-gray-100 text-gray-800',
    }
    return (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[type] || colors.custom}`}>
        {type}
      </span>
    )
  }

  if (loading) {
    return <div className="p-8 text-center text-gray-500">Loading workflows...</div>
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Workflow Builder</h1>
          <p className="text-sm text-gray-500 mt-1">
            Create and manage multi-step automation workflows with conditional branching
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus size={16} />
          New Workflow
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-gray-200">
        {(['overview', 'workflows', 'executions'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors capitalize ${
              activeTab === tab
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Total Workflows</div>
              <div className="text-2xl font-bold text-gray-900 mt-1">{summary?.totalWorkflows || 0}</div>
            </div>
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Active</div>
              <div className="text-2xl font-bold text-green-600 mt-1">{summary?.activeWorkflows || 0}</div>
            </div>
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Drafts</div>
              <div className="text-2xl font-bold text-gray-600 mt-1">{summary?.draftWorkflows || 0}</div>
            </div>
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Executions (24h)</div>
              <div className="text-2xl font-bold text-blue-600 mt-1">{summary?.totalExecutions24h || 0}</div>
            </div>
          </div>

          {/* By Type */}
          {summary?.workflowsByType && Object.keys(summary.workflowsByType).length > 0 && (
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <h3 className="text-sm font-semibold text-gray-700 mb-3">Workflows by Type</h3>
              <div className="space-y-2">
                {Object.entries(summary.workflowsByType).map(([type, count]) => (
                  <div key={type} className="flex items-center gap-3">
                    <span className="text-sm text-gray-600 w-28 capitalize">{type}</span>
                    <div className="flex-1 bg-gray-100 rounded-full h-2">
                      <div
                        className="bg-blue-500 h-2 rounded-full"
                        style={{ width: `${Math.min((count as number) / Math.max(summary.totalWorkflows, 1) * 100, 100)}%` }}
                      />
                    </div>
                    <span className="text-sm font-medium text-gray-700 w-8 text-right">{count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Executions */}
          <div className="bg-white p-4 rounded-lg border border-gray-200">
            <h3 className="text-sm font-semibold text-gray-700 mb-3">Recent Executions</h3>
            {summary?.recentExecutions && summary.recentExecutions.length > 0 ? (
              <div className="space-y-2">
                {summary.recentExecutions.slice(0, 5).map((exec) => (
                  <div key={exec.id} className="flex items-center gap-3 p-2 bg-gray-50 rounded">
                    {exec.status === 'completed' ? (
                      <CheckCircle size={16} className="text-green-500" />
                    ) : exec.status === 'failed' ? (
                      <AlertCircle size={16} className="text-red-500" />
                    ) : (
                      <Clock size={16} className="text-yellow-500" />
                    )}
                    <span className="text-sm text-gray-700 flex-1 truncate">{exec.workflowId}</span>
                    <span className="text-xs text-gray-500">{exec.startedAt?.slice(0, 16)}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-gray-400">No executions yet</p>
            )}
          </div>
        </div>
      )}

      {/* Workflows Tab */}
      {activeTab === 'workflows' && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex gap-3">
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value)}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm"
            >
              <option value="">All Types</option>
              <option value="automation">Automation</option>
              <option value="assignment">Assignment</option>
              <option value="notification">Notification</option>
              <option value="escalation">Escalation</option>
              <option value="custom">Custom</option>
            </select>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value)}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm"
            >
              <option value="">All Statuses</option>
              <option value="active">Active</option>
              <option value="draft">Draft</option>
              <option value="paused">Paused</option>
              <option value="archived">Archived</option>
            </select>
          </div>

          {/* Workflow List */}
          {workflows.length === 0 ? (
            <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
              <GitBranch size={48} className="mx-auto text-gray-300 mb-3" />
              <p className="text-gray-500">No workflows yet</p>
              <p className="text-sm text-gray-400 mt-1">Create your first workflow to get started</p>
            </div>
          ) : (
            <div className="space-y-3">
              {workflows.map((wf) => (
                <div key={wf.id} className="bg-white p-4 rounded-lg border border-gray-200 hover:border-blue-300 transition-colors">
                  <div className="flex items-start gap-3">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <h3 className="font-medium text-gray-900">{wf.name}</h3>
                        {statusBadge(wf.status)}
                        {typeBadge(wf.workflowType)}
                      </div>
                      {wf.description && (
                        <p className="text-sm text-gray-500 mt-1 truncate">{wf.description}</p>
                      )}
                      <div className="flex items-center gap-4 mt-2 text-xs text-gray-400">
                        <span>Trigger: {wf.triggerType}</span>
                        <span>Executions: {wf.executionCount}</span>
                        {wf.lastExecutedAt && <span>Last: {wf.lastExecutedAt.slice(0, 16)}</span>}
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleToggleStatus(wf)}
                        className="p-1.5 text-gray-400 hover:text-blue-600 transition-colors"
                        title={wf.status === 'active' ? 'Pause' : 'Activate'}
                      >
                        <Play size={16} />
                      </button>
                      <button
                        onClick={() => handleExecute(wf)}
                        className="p-1.5 text-gray-400 hover:text-green-600 transition-colors"
                        title="Execute"
                      >
                        <Play size={16} />
                      </button>
                      <button
                        onClick={() => setSelectedWorkflow(wf)}
                        className="p-1.5 text-gray-400 hover:text-blue-600 transition-colors"
                        title="View details"
                      >
                        <Eye size={16} />
                      </button>
                      <button
                        onClick={() => handleDelete(wf.id)}
                        className="p-1.5 text-gray-400 hover:text-red-600 transition-colors"
                        title="Delete"
                      >
                        <Trash2 size={16} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Executions Tab */}
      {activeTab === 'executions' && (
        <div className="space-y-3">
          {executions.length === 0 ? (
            <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
              <Clock size={48} className="mx-auto text-gray-300 mb-3" />
              <p className="text-gray-500">No executions yet</p>
              <p className="text-sm text-gray-400 mt-1">Executions will appear here when workflows run</p>
            </div>
          ) : (
            executions.map((exec) => (
              <div key={exec.id} className="bg-white p-4 rounded-lg border border-gray-200">
                <div className="flex items-center gap-3">
                  {exec.status === 'completed' ? (
                    <CheckCircle size={18} className="text-green-500" />
                  ) : exec.status === 'failed' ? (
                    <AlertCircle size={18} className="text-red-500" />
                  ) : (
                    <Clock size={18} className="text-yellow-500" />
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-gray-900 text-sm">{exec.workflowId}</span>
                      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                        exec.status === 'completed' ? 'bg-green-100 text-green-800' :
                        exec.status === 'failed' ? 'bg-red-100 text-red-800' :
                        'bg-yellow-100 text-yellow-800'
                      }`}>
                        {exec.status}
                      </span>
                    </div>
                    <div className="text-xs text-gray-400 mt-1">
                      {exec.triggerSource} • {exec.startedAt?.slice(0, 16)}
                      {exec.errorMessage && ` • Error: ${exec.errorMessage}`}
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Create Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowCreateModal(false)}>
          <div className="bg-white rounded-lg p-6 w-full max-w-md" onClick={(e) => e.stopPropagation()}>
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Create Workflow</h2>
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-gray-600 mb-1">Name</label>
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                  placeholder="e.g. Auto-assign new conversations"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Type</label>
                <select
                  value={newType}
                  onChange={(e) => setNewType(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="automation">Automation</option>
                  <option value="assignment">Assignment</option>
                  <option value="notification">Notification</option>
                  <option value="escalation">Escalation</option>
                  <option value="custom">Custom</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Trigger Type</label>
                <select
                  value={newTrigger}
                  onChange={(e) => setNewTrigger(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="event">Event</option>
                  <option value="schedule">Schedule</option>
                  <option value="manual">Manual</option>
                  <option value="webhook">Webhook</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-gray-600 mb-1">Description</label>
                <textarea
                  value={newDesc}
                  onChange={(e) => setNewDesc(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                  rows={3}
                  placeholder="What does this workflow do?"
                />
              </div>
            </div>
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800"
              >
                Cancel
              </button>
              <button
                onClick={handleCreate}
                disabled={!newName.trim()}
                className="px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Detail Modal */}
      {selectedWorkflow && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setSelectedWorkflow(null)}>
          <div className="bg-white rounded-lg p-6 w-full max-w-lg max-h-[80vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-start justify-between mb-4">
              <div>
                <h2 className="text-lg font-semibold text-gray-900">{selectedWorkflow.name}</h2>
                <div className="flex gap-2 mt-1">
                  {statusBadge(selectedWorkflow.status)}
                  {typeBadge(selectedWorkflow.workflowType)}
                </div>
              </div>
              <button onClick={() => setSelectedWorkflow(null)} className="text-gray-400 hover:text-gray-600">✕</button>
            </div>
            {selectedWorkflow.description && (
              <p className="text-sm text-gray-600 mb-4">{selectedWorkflow.description}</p>
            )}
            <div className="space-y-2 text-sm">
              <div className="flex justify-between"><span className="text-gray-500">Trigger</span><span className="text-gray-900">{selectedWorkflow.triggerType}</span></div>
              <div className="flex justify-between"><span className="text-gray-500">Executions</span><span className="text-gray-900">{selectedWorkflow.executionCount}</span></div>
              {selectedWorkflow.lastExecutedAt && (
                <div className="flex justify-between"><span className="text-gray-500">Last Executed</span><span className="text-gray-900">{selectedWorkflow.lastExecutedAt.slice(0, 16)}</span></div>
              )}
              <div className="flex justify-between"><span className="text-gray-500">Created</span><span className="text-gray-900">{selectedWorkflow.createdAt.slice(0, 16)}</span></div>
            </div>
            <div className="mt-4 p-3 bg-gray-50 rounded-lg">
              <p className="text-xs text-gray-500">
                Steps, conditions, and triggers are configured via the API. Use the workflow editor to add steps and conditions.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
