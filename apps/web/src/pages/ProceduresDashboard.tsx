import { useState, useEffect } from 'react'
import {
  getProcedureSummary,
  listProcedures,
  createProcedure,
  updateProcedure,
  deleteProcedure,
  listActions,
  createAction,
  updateAction,
  deleteAction,
  type Procedure,
  type ProcedureAction,
  type ProcedureSummary,
} from '../lib/api'

export function ProceduresDashboard() {
  const [tab, setTab] = useState<'overview' | 'procedures' | 'actions'>('overview')
  const [summary, setSummary] = useState<ProcedureSummary | null>(null)
  const [procedures, setProcedures] = useState<Procedure[]>([])
  const [actions, setActions] = useState<ProcedureAction[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreateProcedure, setShowCreateProcedure] = useState(false)
  const [showCreateAction, setShowCreateAction] = useState(false)
  const [editingProcedure, setEditingProcedure] = useState<Procedure | null>(null)
  const [editingAction, setEditingAction] = useState<ProcedureAction | null>(null)

  useEffect(() => {
    loadData()
  }, [tab])

  async function loadData() {
    setLoading(true)
    try {
      const [summaryRes, procsRes, actionsRes] = await Promise.all([
        getProcedureSummary().catch(() => null),
        listProcedures().catch(() => []),
        listActions().catch(() => []),
      ])
      setSummary(summaryRes)
      setProcedures(procsRes)
      setActions(actionsRes)
    } finally {
      setLoading(false)
    }
  }

  async function handleCreateProcedure(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    const body: Record<string, unknown> = {
      name: formData.get('name'),
      description: formData.get('description'),
      category: formData.get('category'),
      trigger_keywords: JSON.parse(formData.get('trigger_keywords') as string || '[]'),
      steps: JSON.parse(formData.get('steps') as string || '[]'),
      max_iterations: parseInt(formData.get('max_iterations') as string) || 10,
      timeout_seconds: parseInt(formData.get('timeout_seconds') as string) || 300,
    }
    await createProcedure(body)
    setShowCreateProcedure(false)
    loadData()
  }

  async function handleUpdateProcedure(id: string, updates: Record<string, unknown>) {
    await updateProcedure(id, updates)
    setEditingProcedure(null)
    loadData()
  }

  async function handleDeleteProcedure(id: string) {
    if (confirm('Delete this procedure?')) {
      await deleteProcedure(id)
      loadData()
    }
  }

  async function handleCreateAction(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const form = e.currentTarget
    const formData = new FormData(form)
    const body: Record<string, unknown> = {
      name: formData.get('name'),
      description: formData.get('description'),
      action_type: formData.get('action_type'),
      endpoint: formData.get('endpoint'),
      method: formData.get('method'),
      headers: JSON.parse(formData.get('headers') as string || '{}'),
      timeout_seconds: parseInt(formData.get('timeout_seconds') as string) || 30,
    }
    await createAction(body)
    setShowCreateAction(false)
    loadData()
  }

  async function handleToggleAction(id: string, enabled: boolean) {
    await updateAction(id, { enabled })
    loadData()
  }

  async function handleDeleteAction(id: string) {
    if (confirm('Delete this action?')) {
      await deleteAction(id)
      loadData()
    }
  }

  const categoryColors: Record<string, string> = {
    refund: 'bg-red-100 text-red-800',
    troubleshooting: 'bg-blue-100 text-blue-800',
    order_change: 'bg-green-100 text-green-800',
    account_update: 'bg-purple-100 text-purple-800',
    escalation: 'bg-orange-100 text-orange-800',
    billing: 'bg-yellow-100 text-yellow-800',
    shipping: 'bg-indigo-100 text-indigo-800',
    general: 'bg-gray-100 text-gray-800',
  }

  const statusColors: Record<string, string> = {
    active: 'bg-green-100 text-green-800',
    draft: 'bg-gray-100 text-gray-800',
    paused: 'bg-yellow-100 text-yellow-800',
    archived: 'bg-red-100 text-red-800',
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading procedures...</div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">AI Procedures & Actions</h1>
          <p className="text-gray-500 mt-1">
            Multi-step AI workflows and external API connectors
          </p>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {(['overview', 'procedures', 'actions'] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`py-2 px-1 border-b-2 font-medium text-sm capitalize ${
                tab === t
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              {t}
            </button>
          ))}
        </nav>
      </div>

      {/* Overview Tab */}
      {tab === 'overview' && (
        <div className="space-y-6">
          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Total Procedures</div>
              <div className="text-3xl font-bold text-gray-900 mt-1">
                {summary?.totalProcedures ?? 0}
              </div>
            </div>
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Active Procedures</div>
              <div className="text-3xl font-bold text-green-600 mt-1">
                {summary?.activeProcedures.length ?? 0}
              </div>
            </div>
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Total Executions</div>
              <div className="text-3xl font-bold text-gray-900 mt-1">
                {summary?.totalExecutions ?? 0}
              </div>
            </div>
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <div className="text-sm text-gray-500">Total Actions</div>
              <div className="text-3xl font-bold text-gray-900 mt-1">
                {summary?.totalActions ?? 0}
              </div>
            </div>
          </div>

          {/* Procedures by Category */}
          <div className="bg-white p-6 rounded-lg border border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Procedures by Category</h3>
            <div className="space-y-3">
              {Object.entries(summary?.proceduresByCategory ?? {}).map(([cat, count]) => (
                <div key={cat} className="flex items-center justify-between">
                  <span className={`px-2 py-1 rounded-full text-xs font-medium capitalize ${categoryColors[cat] || 'bg-gray-100 text-gray-800'}`}>
                    {cat}
                  </span>
                  <span className="text-sm text-gray-500">{count as number}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Recent Executions */}
          <div className="bg-white p-6 rounded-lg border border-gray-200">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Executions</h3>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead>
                  <tr>
                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">ID</th>
                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Step</th>
                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">State</th>
                    <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Started</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200">
                  {(summary?.recentExecutions ?? []).map((exec) => (
                    <tr key={exec.id}>
                      <td className="px-4 py-2 text-sm text-gray-900">{exec.id.slice(0, 8)}</td>
                      <td className="px-4 py-2 text-sm text-gray-500">{exec.currentStep}</td>
                      <td className="px-4 py-2">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                          exec.state === 'completed' ? 'bg-green-100 text-green-800' :
                          exec.state === 'running' ? 'bg-blue-100 text-blue-800' :
                          exec.state === 'failed' ? 'bg-red-100 text-red-800' :
                          'bg-gray-100 text-gray-800'
                        }`}>
                          {exec.state}
                        </span>
                      </td>
                      <td className="px-4 py-2 text-sm text-gray-500">
                        {new Date(exec.startedAt).toLocaleString()}
                      </td>
                    </tr>
                  ))}
                  {(!summary?.recentExecutions || summary.recentExecutions.length === 0) && (
                    <tr>
                      <td colSpan={4} className="px-4 py-8 text-center text-gray-500">
                        No executions yet
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Procedures Tab */}
      {tab === 'procedures' && (
        <div className="space-y-4">
          <div className="flex justify-between">
            <h2 className="text-lg font-semibold text-gray-900">Procedures</h2>
            <button
              onClick={() => setShowCreateProcedure(true)}
              className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 text-sm"
            >
              New Procedure
            </button>
          </div>

          {showCreateProcedure && (
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <h3 className="text-lg font-semibold mb-4">Create Procedure</h3>
              <form onSubmit={handleCreateProcedure} className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">Name</label>
                  <input name="name" required className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2" />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Description</label>
                  <textarea name="description" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2" />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Category</label>
                  <select name="category" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2">
                    <option value="general">General</option>
                    <option value="refund">Refund</option>
                    <option value="troubleshooting">Troubleshooting</option>
                    <option value="order_change">Order Change</option>
                    <option value="account_update">Account Update</option>
                    <option value="escalation">Escalation</option>
                    <option value="billing">Billing</option>
                    <option value="shipping">Shipping</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Trigger Keywords (JSON array)</label>
                  <input name="trigger_keywords" defaultValue="[]" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2" />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Steps (JSON array)</label>
                  <textarea name="steps" defaultValue="[]" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2 font-mono text-sm" rows={6} />
                </div>
                <div className="flex gap-2">
                  <button type="submit" className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700">
                    Create
                  </button>
                  <button type="button" onClick={() => setShowCreateProcedure(false)} className="px-4 py-2 bg-gray-200 rounded-lg hover:bg-gray-300">
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          )}

          <div className="space-y-3">
            {procedures.map((proc) => (
              <div key={proc.id} className="bg-white p-6 rounded-lg border border-gray-200">
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="text-lg font-semibold text-gray-900">{proc.name}</h3>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${categoryColors[proc.category] || 'bg-gray-100 text-gray-800'}`}>
                        {proc.category}
                      </span>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusColors[proc.status] || 'bg-gray-100 text-gray-800'}`}>
                        {proc.status}
                      </span>
                    </div>
                    {proc.description && (
                      <p className="text-gray-500 mt-1">{proc.description}</p>
                    )}
                    <div className="mt-2 flex items-center gap-4 text-sm text-gray-500">
                      <span>Steps: {(proc.steps as unknown[]).length}</span>
                      <span>Keywords: {(proc.triggerKeywords as unknown[]).length}</span>
                      <span>Timeout: {proc.timeoutSeconds}s</span>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleDeleteProcedure(proc.id)}
                      className="px-3 py-1 text-sm text-red-600 hover:bg-red-50 rounded"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
            {procedures.length === 0 && (
              <div className="text-center py-12 text-gray-500">
                No procedures yet. Create one to get started.
              </div>
            )}
          </div>
        </div>
      )}

      {/* Actions Tab */}
      {tab === 'actions' && (
        <div className="space-y-4">
          <div className="flex justify-between">
            <h2 className="text-lg font-semibold text-gray-900">Actions</h2>
            <button
              onClick={() => setShowCreateAction(true)}
              className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 text-sm"
            >
              New Action
            </button>
          </div>

          {showCreateAction && (
            <div className="bg-white p-6 rounded-lg border border-gray-200">
              <h3 className="text-lg font-semibold mb-4">Create Action</h3>
              <form onSubmit={handleCreateAction} className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">Name</label>
                  <input name="name" required className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2" />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Description</label>
                  <textarea name="description" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2" />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Action Type</label>
                  <select name="action_type" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2">
                    <option value="http_request">HTTP Request</option>
                    <option value="webhook">Webhook</option>
                    <option value="stripe">Stripe</option>
                    <option value="internal_api">Internal API</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Endpoint</label>
                  <input name="endpoint" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2" />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Method</label>
                  <select name="method" className="mt-1 w-full rounded-md border border-gray-300 px-3 py-2">
                    <option value="GET">GET</option>
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                    <option value="DELETE">DELETE</option>
                  </select>
                </div>
                <div className="flex gap-2">
                  <button type="submit" className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700">
                    Create
                  </button>
                  <button type="button" onClick={() => setShowCreateAction(false)} className="px-4 py-2 bg-gray-200 rounded-lg hover:bg-gray-300">
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          )}

          <div className="space-y-3">
            {actions.map((action) => (
              <div key={action.id} className="bg-white p-6 rounded-lg border border-gray-200">
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="text-lg font-semibold text-gray-900">{action.name}</h3>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                        action.enabled ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      }`}>
                        {action.enabled ? 'Enabled' : 'Disabled'}
                      </span>
                      <span className="px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 capitalize">
                        {action.actionType.replace('_', ' ')}
                      </span>
                    </div>
                    {action.description && (
                      <p className="text-gray-500 mt-1">{action.description}</p>
                    )}
                    <div className="mt-2 flex items-center gap-4 text-sm text-gray-500">
                      {action.endpoint && <span>Endpoint: {action.endpoint}</span>}
                      <span>Method: {action.method}</span>
                      <span>Timeout: {action.timeoutSeconds}s</span>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleToggleAction(action.id, !action.enabled)}
                      className="px-3 py-1 text-sm text-indigo-600 hover:bg-indigo-50 rounded"
                    >
                      {action.enabled ? 'Disable' : 'Enable'}
                    </button>
                    <button
                      onClick={() => handleDeleteAction(action.id)}
                      className="px-3 py-1 text-sm text-red-600 hover:bg-red-50 rounded"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
            {actions.length === 0 && (
              <div className="text-center py-12 text-gray-500">
                No actions yet. Create one to get started.
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
