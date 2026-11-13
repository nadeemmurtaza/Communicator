import { useState, useEffect } from 'react'
import {
  getSchedulerSummary, listScheduledTasks, createScheduledTask, updateScheduledTask,
  deleteScheduledTask, triggerScheduledTask, processTaskQueue, listTaskRuns,
  listTaskQueue, getSchedulerSettings, updateSchedulerSettings, cleanupOldLogs,
  getAvailableTaskTypes,
} from '../lib/api'
import type { ScheduledTask, TaskRun, TaskQueueEntry, SchedulerSettings, SchedulerSummary } from '../lib/api'
import { Clock, Play, Trash2, Plus, Settings, Activity, AlertCircle, CheckCircle, XCircle, RefreshCw } from '../components/icons'

export default function ScheduledTasksDashboard() {
  const [summary, setSummary] = useState<SchedulerSummary | null>(null)
  const [tasks, setTasks] = useState<ScheduledTask[]>([])
  const [runs, setRuns] = useState<TaskRun[]>([])
  const [queue, setQueue] = useState<TaskQueueEntry[]>([])
  const [settings, setSettings] = useState<SchedulerSettings | null>(null)
  const [taskTypes, setTaskTypes] = useState<string[]>([])
  const [activeTab, setActiveTab] = useState<'overview' | 'tasks' | 'runs' | 'settings'>('overview')
  const [loading, setLoading] = useState(true)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newTask, setNewTask] = useState({ name: '', description: '', task_type: 'metric_collection', cron_schedule: '0 */6 * * *', config: '{}' })
  const [filterType, setFilterType] = useState('')
  const [filterStatus, setFilterStatus] = useState('')

  const load = async () => {
    setLoading(true)
    try {
      const [s, t, r, q, st, tt] = await Promise.all([
        getSchedulerSummary(),
        listScheduledTasks(false, filterType || undefined),
        listTaskRuns(undefined, filterStatus || undefined, 50),
        listTaskQueue(),
        getSchedulerSettings(),
        getAvailableTaskTypes(),
      ])
      setSummary(s)
      setTasks(t)
      setRuns(r)
      setQueue(q)
      setSettings(st)
      setTaskTypes(tt)
    } catch (e) {
      console.error('Failed to load scheduled tasks:', e)
    }
    setLoading(false)
  }

  useEffect(() => { load() }, [])

  const handleCreate = async () => {
    try {
      const config = newTask.config ? JSON.parse(newTask.config) : {}
      await createScheduledTask({ ...newTask, config })
      setShowCreateModal(false)
      setNewTask({ name: '', description: '', task_type: 'metric_collection', cron_schedule: '0 */6 * * *', config: '{}' })
      load()
    } catch (e) {
      console.error('Failed to create task:', e)
    }
  }

  const handleToggleActive = async (task: ScheduledTask) => {
    try {
      await updateScheduledTask(task.id, { is_active: !task.is_active })
      load()
    } catch (e) {
      console.error('Failed to update task:', e)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this scheduled task?')) return
    try {
      await deleteScheduledTask(id)
      load()
    } catch (e) {
      console.error('Failed to delete task:', e)
    }
  }

  const handleTrigger = async (id: string) => {
    try {
      await triggerScheduledTask(id)
      load()
    } catch (e) {
      console.error('Failed to trigger task:', e)
    }
  }

  const handleProcessQueue = async () => {
    try {
      await processTaskQueue()
      load()
    } catch (e) {
      console.error('Failed to process queue:', e)
    }
  }

  const handleCleanup = async () => {
    try {
      await cleanupOldLogs(30)
      load()
    } catch (e) {
      console.error('Failed to cleanup:', e)
    }
  }

  const handleSettingsUpdate = async () => {
    if (!settings) return
    try {
      await updateSchedulerSettings(settings)
      load()
    } catch (e) {
      console.error('Failed to update settings:', e)
    }
  }

  const statusBadge = (status: string) => {
    const colors: Record<string, string> = {
      success: 'bg-green-100 text-green-800',
      failed: 'bg-red-100 text-red-800',
      running: 'bg-blue-100 text-blue-800',
      pending: 'bg-yellow-100 text-yellow-800',
      completed: 'bg-green-100 text-green-800',
      skipped: 'bg-gray-100 text-gray-800',
    }
    return (
      <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[status] || 'bg-gray-100 text-gray-800'}`}>
        {status}
      </span>
    )
  }

  if (loading) return <div className="p-6">Loading...</div>

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Scheduled Tasks & Cron Jobs</h1>
        <p className="text-gray-500 mt-1">Automate recurring operations across the platform</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-100 rounded-lg p-1">
        {(['overview', 'tasks', 'runs', 'settings'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 rounded-md text-sm font-medium capitalize transition-colors ${
              activeTab === tab ? 'bg-white text-gray-900 shadow-sm' : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-white p-4 rounded-lg border">
              <div className="text-sm text-gray-500">Total Tasks</div>
              <div className="text-2xl font-bold">{summary?.total_tasks || 0}</div>
              <div className="text-xs text-green-600">{summary?.active_tasks || 0} active</div>
            </div>
            <div className="bg-white p-4 rounded-lg border">
              <div className="text-sm text-gray-500">Pending Queue</div>
              <div className="text-2xl font-bold">{summary?.pending_queue || 0}</div>
              <div className="text-xs text-gray-500">waiting to run</div>
            </div>
            <div className="bg-white p-4 rounded-lg">
              <div className="text-sm text-gray-500">Total Runs</div>
              <div className="text-2xl font-bold">{summary?.total_runs || 0}</div>
              <div className="text-xs text-green-600">{summary?.successful_runs || 0} successful</div>
            </div>
            <div className="bg-white p-4 rounded-lg border">
              <div className="text-sm text-gray-500">Failed Runs</div>
              <div className="text-2xl font-bold text-red-600">{summary?.failed_runs || 0}</div>
              <div className="text-xs text-gray-500">need attention</div>
            </div>
          </div>

          <div className="flex gap-2">
            <button onClick={handleProcessQueue} className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700">
              <RefreshCw className="w-4 h-4 inline mr-1" /> Process Queue
            </button>
            <button onClick={handleCleanup} className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-300">
              <Trash2 className="w-4 h-4 inline mr-1" /> Cleanup Old Logs
            </button>
          </div>

          {/* Task Type Breakdown */}
          <div className="bg-white rounded-lg border p-4">
            <h3 className="font-semibold mb-3">Tasks by Type</h3>
            <div className="space-y-2">
              {summary?.task_type_breakdown.map((tt, i) => (
                <div key={i} className="flex items-center gap-3">
                  <div className="w-40 text-sm text-gray-600 truncate">{tt.task_type}</div>
                  <div className="flex-1 bg-gray-100 rounded-full h-4 overflow-hidden">
                    <div
                      className="h-full bg-blue-500 rounded-full"
                      style={{ width: `${Math.min((tt.count / (summary.total_tasks || 1)) * 100, 100)}%` }}
                    />
                  </div>
                  <div className="text-sm font-medium w-8 text-right">{tt.count}</div>
                </div>
              ))}
            </div>
          </div>

          {/* Recent Runs */}
          <div className="bg-white rounded-lg border p-4">
            <h3 className="font-semibold mb-3">Recent Runs</h3>
            <div className="space-y-2">
              {summary?.recent_runs.slice(0, 5).map(run => (
                <div key={run.id} className="flex items-center gap-3 py-2 border-b last:border-0">
                  {statusBadge(run.status)}
                  <div className="text-sm text-gray-600 flex-1 truncate">{run.task_id.substring(0, 8)}...</div>
                  <div className="text-xs text-gray-400">{new Date(run.started_at).toLocaleString()}</div>
                  {run.duration_ms && <div className="text-xs text-gray-500">{run.duration_ms}ms</div>}
                  <div className="text-xs text-gray-500">{run.records_processed} records</div>
                </div>
              ))}
              {(!summary?.recent_runs || summary.recent_runs.length === 0) && (
                <div className="text-sm text-gray-400">No runs yet</div>
              )}
            </div>
          </div>

          {/* Queue */}
          {queue.length > 0 && (
            <div className="bg-white rounded-lg border p-4">
              <h3 className="font-semibold mb-3">Task Queue ({queue.length})</h3>
              <div className="space-y-2">
                {queue.slice(0, 5).map(entry => (
                  <div key={entry.id} className="flex items-center gap-3 py-2 border-b last:border-0">
                    {statusBadge(entry.status)}
                    <div className="text-sm text-gray-600 flex-1">{entry.triggered_by}</div>
                    <div className="text-xs text-gray-400">{new Date(entry.triggered_at).toLocaleString()}</div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Tasks Tab */}
      {activeTab === 'tasks' && (
        <div className="space-y-4">
          <div className="flex gap-2 items-center">
            <select
              value={filterType}
              onChange={e => setFilterType(e.target.value)}
              className="px-3 py-2 border rounded-lg text-sm"
            >
              <option value="">All Types</option>
              {taskTypes.map(tt => (
                <option key={tt} value={tt}>{tt}</option>
              ))}
            </select>
            <button
              onClick={() => setShowCreateModal(true)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700"
            >
              <Plus className="w-4 h-4 inline mr-1" /> New Task
            </button>
          </div>

          <div className="bg-white rounded-lg border overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b">
                <tr>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Name</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Type</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Schedule</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Status</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Actions</th>
                </tr>
              </thead>
              <tbody>
                {tasks.map(task => (
                  <tr key={task.id} className="border-b last:border-0 hover:bg-gray-50">
                    <td className="px-4 py-3">
                      <div className="font-medium">{task.name}</div>
                      {task.description && <div className="text-xs text-gray-400 truncate max-w-xs">{task.description}</div>}
                    </td>
                    <td className="px-4 py-3">
                      <span className="px-2 py-0.5 bg-blue-100 text-blue-800 rounded-full text-xs">{task.task_type}</span>
                    </td>
                    <td className="px-4 py-3 font-mono text-xs text-gray-500">{task.cron_schedule}</td>
                    <td className="px-4 py-3">
                      <button
                        onClick={() => handleToggleActive(task)}
                        className={`px-2 py-0.5 rounded-full text-xs font-medium cursor-pointer ${
                          task.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-500'
                        }`}
                      >
                        {task.is_active ? 'Active' : 'Inactive'}
                      </button>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex gap-1">
                        <button onClick={() => handleTrigger(task.id)} className="p-1 text-blue-600 hover:bg-blue-50 rounded" title="Trigger now">
                          <Play className="w-4 h-4" />
                        </button>
                        <button onClick={() => handleDelete(task.id)} className="p-1 text-red-600 hover:bg-red-50 rounded" title="Delete">
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Create Modal */}
          {showCreateModal && (
            <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
              <div className="bg-white rounded-lg p-6 w-full max-w-md">
                <h3 className="text-lg font-semibold mb-4">New Scheduled Task</h3>
                <div className="space-y-3">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
                    <input
                      type="text"
                      value={newTask.name}
                      onChange={e => setNewTask({ ...newTask, name: e.target.value })}
                      className="w-full px-3 py-2 border rounded-lg text-sm"
                      placeholder="My Task"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                    <input
                      type="text"
                      value={newTask.description}
                      onChange={e => setNewTask({ ...newTask, description: e.target.value })}
                      className="w-full px-3 py-2 border rounded-lg text-sm"
                      placeholder="What this task does"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Task Type</label>
                    <select
                      value={newTask.task_type}
                      onChange={e => setNewTask({ ...newTask, task_type: e.target.value })}
                      className="w-full px-3 py-2 border rounded-lg text-sm"
                    >
                      {taskTypes.map(tt => (
                        <option key={tt} value={tt}>{tt}</option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Cron Schedule</label>
                    <input
                      type="text"
                      value={newTask.cron_schedule}
                      onChange={e => setNewTask({ ...newTask, cron_schedule: e.target.value })}
                      className="w-full px-3 py-2 border rounded-lg text-sm font-mono"
                      placeholder="0 */6 * * *"
                    />
                    <div className="text-xs text-gray-400 mt-1">e.g. "0 */6 * * *" = every 6 hours</div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Config (JSON)</label>
                    <textarea
                      value={newTask.config}
                      onChange={e => setNewTask({ ...newTask, config: e.target.value })}
                      className="w-full px-3 py-2 border rounded-lg text-sm font-mono"
                      rows={3}
                      placeholder='{"key": "value"}'
                    />
                  </div>
                </div>
                <div className="flex gap-2 mt-4 justify-end">
                  <button onClick={() => setShowCreateModal(false)} className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg text-sm">
                    Cancel
                  </button>
                  <button onClick={handleCreate} className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700">
                    Create
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Runs Tab */}
      {activeTab === 'runs' && (
        <div className="space-y-4">
          <div className="flex gap-2 items-center">
            <select
              value={filterStatus}
              onChange={e => setFilterStatus(e.target.value)}
              className="px-3 py-2 border rounded-lg text-sm"
            >
              <option value="">All Statuses</option>
              <option value="success">Success</option>
              <option value="failed">Failed</option>
              <option value="running">Running</option>
              <option value="skipped">Skipped</option>
            </select>
            <button onClick={load} className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-300">
              <RefreshCw className="w-4 h-4 inline mr-1" /> Refresh
            </button>
          </div>

          <div className="bg-white rounded-lg border overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b">
                <tr>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Status</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Task ID</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Started</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Duration</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Records</th>
                  <th className="text-left px-4 py-3 font-medium text-gray-600">Error</th>
                </tr>
              </thead>
              <tbody>
                {runs.map(run => (
                  <tr key={run.id} className="border-b last:border-0 hover:bg-gray-50">
                    <td className="px-4 py-3">{statusBadge(run.status)}</td>
                    <td className="px-4 py-3 font-mono text-xs text-gray-500">{run.task_id.substring(0, 12)}...</td>
                    <td className="px-4 py-3 text-xs text-gray-500">{new Date(run.started_at).toLocaleString()}</td>
                    <td className="px-4 py-3 text-xs text-gray-500">{run.duration_ms ? `${run.duration_ms}ms` : '-'}</td>
                    <td className="px-4 py-3 text-xs text-gray-500">{run.records_processed}</td>
                    <td className="px-4 py-3 text-xs text-red-500 truncate max-w-xs">
                      {run.error_message || '-'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {runs.length === 0 && (
              <div className="p-6 text-center text-gray-400">No runs found</div>
            )}
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && settings && (
        <div className="max-w-2xl bg-white rounded-lg border p-6 space-y-4">
          <h3 className="text-lg font-semibold">Scheduler Settings</h3>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Enable Scheduler</div>
              <div className="text-sm text-gray-500">Master toggle for all scheduled tasks</div>
            </div>
            <button
              onClick={() => setSettings({ ...settings, enabled: !settings.enabled })}
              className={`w-12 h-6 rounded-full transition-colors ${settings.enabled ? 'bg-blue-600' : 'bg-gray-300'} relative`}
            >
              <div className={`w-5 h-5 bg-white rounded-full absolute top-0.5 transition-transform ${settings.enabled ? 'translate-x-6' : 'translate-x-0.5'}`} />
            </button>
          </div>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Max Concurrent Tasks</div>
              <div className="text-sm text-gray-500">Maximum tasks running simultaneously</div>
            </div>
            <input
              type="number"
              value={settings.max_concurrent_tasks}
              onChange={e => setSettings({ ...settings, max_concurrent_tasks: parseInt(e.target.value) || 1 })}
              className="w-20 px-3 py-1 border rounded text-sm text-center"
            />
          </div>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Task Timeout (seconds)</div>
              <div className="text-sm text-gray-500">Max execution time per task</div>
            </div>
            <input
              type="number"
              value={settings.task_timeout_seconds}
              onChange={e => setSettings({ ...settings, task_timeout_seconds: parseInt(e.target.value) || 300 })}
              className="w-20 px-3 py-1 border rounded text-sm text-center"
            />
          </div>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Retry Failed Tasks</div>
              <div className="text-sm text-gray-500">Automatically retry on failure</div>
            </div>
            <button
              onClick={() => setSettings({ ...settings, retry_failed: !settings.retry_failed })}
              className={`w-12 h-6 rounded-full transition-colors ${settings.retry_failed ? 'bg-blue-600' : 'bg-gray-300'} relative`}
            >
              <div className={`w-5 h-5 bg-white rounded-full absolute top-0.5 transition-transform ${settings.retry_failed ? 'translate-x-6' : 'translate-x-0.5'}`} />
            </button>
          </div>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Retry Count</div>
              <div className="text-sm text-gray-500">Number of retry attempts</div>
            </div>
            <input
              type="number"
              value={settings.retry_count}
              onChange={e => setSettings({ ...settings, retry_count: parseInt(e.target.value) || 1 })}
              className="w-20 px-3 py-1 border rounded text-sm text-center"
            />
          </div>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Retry Delay (seconds)</div>
              <div className="text-sm text-gray-500">Wait between retries</div>
            </div>
            <input
              type="number"
              value={settings.retry_delay_seconds}
              onChange={e => setSettings({ ...settings, retry_delay_seconds: parseInt(e.target.value) || 60 })}
              className="w-20 px-3 py-1 border rounded text-sm text-center"
            />
          </div>

          <div className="flex items-center justify-between py-2 border-b">
            <div>
              <div className="font-medium">Log Retention (days)</div>
              <div className="text-sm text-gray-500">How long to keep execution logs</div>
            </div>
            <input
              type="number"
              value={settings.log_retention_days}
              onChange={e => setSettings({ ...settings, log_retention_days: parseInt(e.target.value) || 30 })}
              className="w-20 px-3 py-1 border rounded text-sm text-center"
            />
          </div>

          <div className="py-2">
            <div className="font-medium mb-1">Webhook URL</div>
            <div className="text-sm text-gray-500 mb-2">Notification webhook for task failures</div>
            <input
              type="text"
              value={settings.webhook_url || ''}
              onChange={e => setSettings({ ...settings, webhook_url: e.target.value || null })}
              className="w-full px-3 py-2 border rounded-lg text-sm"
              placeholder="https://hooks.slack.com/..."
            />
          </div>

          <div className="flex justify-end pt-4">
            <button onClick={handleSettingsUpdate} className="px-6 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700">
              Save Settings
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
