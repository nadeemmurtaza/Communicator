import { useState, useEffect } from 'react'
import {
  getInAppSummary, createInAppMessage, listInAppMessages, updateInAppMessage,
  deleteInAppMessage, createProactiveTrigger, listProactiveTriggers,
  updateProactiveTrigger, deleteProactiveTrigger, recordBehavioralEvent,
  listBehavioralEvents, evaluateTriggers, getInAppSettings, updateInAppSettings
} from '../lib/api'
import type {
  InAppSummary, InAppMessage, ProactiveTrigger, BehavioralEvent,
  InAppSettings, TriggerEvaluationResult
} from '../lib/api'
import { MessageSquare, Plus, Trash2, Play, Pause, Zap, Settings, Eye, Search, X } from '../components/icons'

export default function InAppMessagingDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'messages' | 'triggers' | 'events' | 'settings'>('overview')
  const [summary, setSummary] = useState<InAppSummary | null>(null)
  const [messages, setMessages] = useState<InAppMessage[]>([])
  const [triggers, setTriggers] = useState<ProactiveTrigger[]>([])
  const [events, setEvents] = useState<BehavioralEvent[]>([])
  const [settings, setSettings] = useState<InAppSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [showMessageModal, setShowMessageModal] = useState(false)
  const [showTriggerModal, setShowTriggerModal] = useState(false)
  const [showEvaluateModal, setShowEvaluateModal] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState('')
  const [newMessage, setNewMessage] = useState({
    title: '', body: '', messageType: 'popup', triggerType: 'event',
    triggerEvent: '', triggerPage: '', frequency: 'once_per_session'
  })
  const [newTrigger, setNewTrigger] = useState({
    name: '', triggerType: 'page_view', condition: '{}', action: 'show_message',
    messageId: '', delaySeconds: 0
  })
  const [evaluateForm, setEvaluateForm] = useState({ eventName: '', pageUrl: '' })
  const [evaluationResults, setEvaluationResults] = useState<TriggerEvaluationResult[]>([])
  const [settingsForm, setSettingsForm] = useState({
    enabled: true, maxConcurrentMessages: 1, trackBehavioralEvents: true,
    eventRetentionDays: 30
  })

  const loadSummary = () => getInAppSummary().then(setSummary).catch(() => {})
  const loadMessages = () => listInAppMessages(statusFilter || undefined).then(setMessages).catch(() => {})
  const loadTriggers = () => listProactiveTriggers().then(setTriggers).catch(() => {})
  const loadEvents = () => listBehavioralEvents().then(setEvents).catch(() => {})
  const loadSettings = () => getInAppSettings().then(s => { setSettings(s); setSettingsForm({ enabled: s.enabled, maxConcurrentMessages: s.maxConcurrentMessages, trackBehavioralEvents: s.trackBehavioralEvents, eventRetentionDays: s.eventRetentionDays }) }).catch(() => {})

  useEffect(() => {
    setLoading(true)
    Promise.all([loadSummary(), loadMessages(), loadTriggers(), loadEvents(), loadSettings()]).finally(() => setLoading(false))
  }, [activeTab, statusFilter])

  const handleCreateMessage = async () => {
    await createInAppMessage({ ...newMessage, organizationId: 'org-default' })
    setShowMessageModal(false)
    setNewMessage({ title: '', body: '', messageType: 'popup', triggerType: 'event', triggerEvent: '', triggerPage: '', frequency: 'once_per_session' })
    loadMessages()
    loadSummary()
  }

  const handleCreateTrigger = async () => {
    await createProactiveTrigger({ ...newTrigger, organizationId: 'org-default' })
    setShowTriggerModal(false)
    setNewTrigger({ name: '', triggerType: 'page_view', condition: '{}', action: 'show_message', messageId: '', delaySeconds: 0 })
    loadTriggers()
    loadSummary()
  }

  const handleDeleteMessage = async (id: string) => {
    await deleteInAppMessage(id)
    loadMessages()
    loadSummary()
  }

  const handleDeleteTrigger = async (id: string) => {
    await deleteProactiveTrigger(id)
    loadTriggers()
    loadSummary()
  }

  const handleToggleMessageStatus = async (msg: InAppMessage) => {
    const newStatus = msg.status === 'active' ? 'draft' : 'active'
    await updateInAppMessage(msg.id, { status: newStatus })
    loadMessages()
    loadSummary()
  }

  const handleToggleTriggerEnabled = async (trig: ProactiveTrigger) => {
    await updateProactiveTrigger(trig.id, { enabled: !trig.enabled })
    loadTriggers()
    loadSummary()
  }

  const handleEvaluate = async () => {
    const results = await evaluateTriggers({
      organizationId: 'org-default',
      eventName: evaluateForm.eventName,
      pageUrl: evaluateForm.pageUrl || undefined
    })
    setEvaluationResults(results)
  }

  const handleSaveSettings = async () => {
    await updateInAppSettings(settingsForm)
    loadSettings()
    loadSummary()
  }

  const handleRecordTestEvent = async () => {
    await recordBehavioralEvent({
      organizationId: 'org-default',
      eventName: 'test_event',
      pageUrl: '/test'
    })
    loadEvents()
    loadSummary()
  }

  const filteredMessages = messages.filter(m =>
    m.title.toLowerCase().includes(searchQuery.toLowerCase())
  )

  if (loading) return <div className="p-6 text-center text-slate-400">Loading...</div>

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">In-App Messaging</h1>
          <p className="text-sm text-slate-400 mt-1">Proactive triggers & behavioral event-driven messaging</p>
        </div>
        <div className="flex items-center gap-2">
          <span className={`inline-block w-2 h-2 rounded-full ${summary?.enabled ? 'bg-emerald-400' : 'bg-red-400'}`} />
          <span className="text-sm text-slate-400">{summary?.enabled ? 'Enabled' : 'Disabled'}</span>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-slate-800/50 p-1 rounded-lg w-fit">
        {(['overview', 'messages', 'triggers', 'events', 'settings'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 rounded-md text-sm font-medium capitalize transition-colors ${
              activeTab === tab ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400">Active Messages</div>
              <div className="text-3xl font-bold text-indigo-400 mt-1">{summary?.activeMessages}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400">Draft Messages</div>
              <div className="text-3xl font-bold text-amber-400 mt-1">{summary?.draftMessages}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400">Enabled Triggers</div>
              <div className="text-3xl font-bold text-emerald-400 mt-1">{summary?.enabledTriggers}</div>
            </div>
            <div className="bg-slate-800/50 rounded-lg p-4">
              <div className="text-sm text-slate-400">Events Today</div>
              <div className="text-3xl font-bold text-cyan-400 mt-1">{summary?.totalEventsToday}</div>
            </div>
          </div>

          {/* By Message Type */}
          {summary?.byMessageType.length && (
            <div className="bg-slate-800/50 rounded-lg p-4">
              <h3 className="text-sm font-semibold text-slate-300 mb-3">Messages by Type</h3>
              <div className="space-y-2">
                {summary.byMessageType.map(mt => (
                  <div key={mt.messageType} className="flex items-center gap-3">
                    <span className="text-sm text-slate-400 w-24 capitalize">{mt.messageType}</span>
                    <div className="flex-1 bg-slate-700 rounded-full h-2">
                      <div
                        className="bg-indigo-500 h-2 rounded-full"
                        style={{ width: `${(mt.count / summary.totalMessages) * 100}%` }}
                      />
                    </div>
                    <span className="text-sm text-slate-300 w-8 text-right">{mt.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* By Trigger Type */}
          {summary?.byTriggerType.length && (
            <div className="bg-slate-800/50 rounded-lg p-4">
              <h3 className="text-sm font-semibold text-slate-300 mb-3">Triggers by Type</h3>
              <div className="space-y-2">
                {summary.byTriggerType.map(tt => (
                  <div key={tt.triggerType} className="flex items-center gap-3">
                    <span className="text-sm text-slate-400 w-24 capitalize">{tt.triggerType}</span>
                    <div className="flex-1 bg-slate-700 rounded-full h-2">
                      <div
                        className="bg-emerald-500 h-2 rounded-full"
                        style={{ width: `${(tt.count / summary.totalTriggers) * 100}%` }}
                      />
                    </div>
                    <span className="text-sm text-slate-300 w-8 text-right">{tt.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Events */}
          {summary?.recentEvents.length && (
            <div className="bg-slate-800/50 rounded-lg p-4">
              <h3 className="text-sm font-semibold text-slate-300 mb-3">Recent Message Events</h3>
              <div className="space-y-2">
                {summary.recentEvents.slice(0, 5).map(evt => (
                  <div key={evt.id} className="flex items-center justify-between text-sm">
                    <span className="text-slate-400 capitalize">{evt.eventType}</span>
                    <span className="text-slate-500">{new Date(evt.createdAt).toLocaleString()}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Messages Tab */}
      {activeTab === 'messages' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <input
                type="text"
                placeholder="Search messages..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                className="bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500"
              />
              <select
                value={statusFilter}
                onChange={e => setStatusFilter(e.target.value)}
                className="bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-200"
              >
                <option value="">All Statuses</option>
                <option value="active">Active</option>
                <option value="draft">Draft</option>
                <option value="archived">Archived</option>
              </select>
            </div>
            <button
              onClick={() => setShowMessageModal(true)}
              className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-lg text-sm font-medium"
            >
              <Plus className="w-4 h-4" /> New Message
            </button>
          </div>

          <div className="space-y-3">
            {filteredMessages.map(msg => (
              <div key={msg.id} className="bg-slate-800/50 rounded-lg p-4 space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <MessageSquare className="w-5 h-5 text-indigo-400" />
                    <div>
                      <h3 className="text-sm font-semibold text-slate-200">{msg.title}</h3>
                      <p className="text-xs text-slate-400 truncate max-w-md">{msg.body}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                      msg.status === 'active' ? 'bg-emerald-500/20 text-emerald-400' :
                      msg.status === 'draft' ? 'bg-amber-500/20 text-amber-400' :
                      'bg-slate-500/20 text-slate-400'
                    }`}>
                      {msg.status}
                    </span>
                    <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-500/20 text-indigo-400 capitalize">
                      {msg.messageType}
                    </span>
                    <button
                      onClick={() => handleToggleMessageStatus(msg)}
                      className="p-1 hover:bg-slate-700 rounded"
                      title={msg.status === 'active' ? 'Deactivate' : 'Activate'}
                    >
                      {msg.status === 'active' ? <Pause className="w-4 h-4 text-amber-400" /> : <Play className="w-4 h-4 text-emerald-400" />}
                    </button>
                    <button
                      onClick={() => handleDeleteMessage(msg.id)}
                      className="p-1 hover:bg-slate-700 rounded"
                      title="Delete"
                    >
                      <Trash2 className="w-4 h-4 text-red-400" />
                    </button>
                  </div>
                </div>
                <div className="flex items-center gap-4 text-xs text-slate-500">
                  <span className="flex items-center gap-1"><Eye className="w-3 h-3" /> {msg.views} views</span>
                  <span>Trigger: {msg.triggerType}</span>
                  <span>Frequency: {msg.frequency}</span>
                  <span>{new Date(msg.createdAt).toLocaleDateString()}</span>
                </div>
              </div>
            ))}
            {filteredMessages.length === 0 && (
              <div className="text-center text-slate-500 py-8">No messages found</div>
            )}
          </div>
        </div>
      )}

      {/* Triggers Tab */}
      {activeTab === 'triggers' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-slate-200">Proactive Triggers</h2>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowEvaluateModal(true)}
                className="flex items-center gap-2 bg-amber-600 hover:bg-amber-700 text-white px-4 py-2 rounded-lg text-sm font-medium"
              >
                <Zap className="w-4 h-4" /> Test Trigger
              </button>
              <button
                onClick={() => setShowTriggerModal(true)}
                className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-lg text-sm font-medium"
              >
                <Plus className="w-4 h-4" /> New Trigger
              </button>
            </div>
          </div>

          <div className="space-y-3">
            {triggers.map(trig => (
              <div key={trig.id} className="bg-slate-800/50 rounded-lg p-4 space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <Zap className={`w-5 h-5 ${trig.enabled ? 'text-emerald-400' : 'text-slate-500'}`} />
                    <div>
                      <h3 className="text-sm font-semibold text-slate-200">{trig.name}</h3>
                      <p className="text-xs text-slate-400">Type: {trig.triggerType} | Action: {trig.action}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                      trig.enabled ? 'bg-emerald-500/20 text-emerald-400' : 'bg-slate-500/20 text-slate-400'
                    }`}>
                      {trig.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                    <span className="text-xs text-slate-500">{trig.triggersToday}/{trig.maxTriggersPerDay} today</span>
                    <button
                      onClick={() => handleToggleTriggerEnabled(trig)}
                      className="p-1 hover:bg-slate-700 rounded"
                    >
                      {trig.enabled ? <Pause className="w-4 h-4 text-amber-400" /> : <Play className="w-4 h-4 text-emerald-400" />}
                    </button>
                    <button
                      onClick={() => handleDeleteTrigger(trig.id)}
                      className="p-1 hover:bg-slate-700 rounded"
                    >
                      <Trash2 className="w-4 h-4 text-red-400" />
                    </button>
                  </div>
                </div>
                <div className="text-xs text-slate-500">
                  Delay: {trig.delaySeconds}s | Message: {trig.messageId || 'None'}
                </div>
              </div>
            ))}
            {triggers.length === 0 && (
              <div className="text-center text-slate-500 py-8">No triggers configured</div>
            )}
          </div>
        </div>
      )}

      {/* Events Tab */}
      {activeTab === 'events' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-slate-200">Behavioral Events</h2>
            <button
              onClick={handleRecordTestEvent}
              className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-lg text-sm font-medium"
            >
              <Plus className="w-4 h-4" /> Record Test Event
            </button>
          </div>

          <div className="bg-slate-800/50 rounded-lg overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="text-left p-3 text-slate-400 font-medium">Event</th>
                  <th className="text-left p-3 text-slate-400 font-medium">Page</th>
                  <th className="text-left p-3 text-slate-400 font-medium">Contact</th>
                  <th className="text-left p-3 text-slate-400 font-medium">Time</th>
                </tr>
              </thead>
              <tbody>
                {events.map(evt => (
                  <tr key={evt.id} className="border-b border-slate-700/50 hover:bg-slate-700/30">
                    <td className="p-3 text-slate-200">{evt.eventName}</td>
                    <td className="p-3 text-slate-400">{evt.pageUrl || '-'}</td>
                    <td className="p-3 text-slate-400">{evt.contactId || '-'}</td>
                    <td className="p-3 text-slate-500">{new Date(evt.createdAt).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {events.length === 0 && (
              <div className="text-center text-slate-500 py-8">No behavioral events recorded</div>
            )}
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && settings && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-slate-200">Settings</h2>
          <div className="bg-slate-800/50 rounded-lg p-6 space-y-4 max-w-lg">
            <div className="flex items-center justify-between">
              <label className="text-sm text-slate-300">Enable In-App Messaging</label>
              <button
                onClick={() => { setSettingsForm(f => ({ ...f, enabled: !f.enabled })); }}
                className={`w-11 h-6 rounded-full transition-colors ${settingsForm.enabled ? 'bg-indigo-600' : 'bg-slate-600'} relative`}
              >
                <div className={`absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform ${settingsForm.enabled ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div>
              <label className="text-sm text-slate-300 block mb-1">Max Concurrent Messages</label>
              <input
                type="number"
                value={settingsForm.maxConcurrentMessages}
                onChange={e => setSettingsForm(f => ({ ...f, maxConcurrentMessages: parseInt(e.target.value) || 1 }))}
                className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
              />
            </div>
            <div className="flex items-center justify-between">
              <label className="text-sm text-slate-300">Track Behavioral Events</label>
              <button
                onClick={() => { setSettingsForm(f => ({ ...f, trackBehavioralEvents: !f.trackBehavioralEvents })); }}
                className={`w-11 h-6 rounded-full transition-colors ${settingsForm.trackBehavioralEvents ? 'bg-indigo-600' : 'bg-slate-600'} relative`}
              >
                <div className={`absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform ${settingsForm.trackBehavioralEvents ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div>
              <label className="text-sm text-slate-300 block mb-1">Event Retention (days)</label>
              <input
                type="number"
                value={settingsForm.eventRetentionDays}
                onChange={e => setSettingsForm(f => ({ ...f, eventRetentionDays: parseInt(e.target.value) || 30 }))}
                className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
              />
            </div>
            <button
              onClick={handleSaveSettings}
              className="w-full bg-indigo-600 hover:bg-indigo-700 text-white py-2 rounded-lg text-sm font-medium"
            >
              Save Settings
            </button>
          </div>
        </div>
      )}

      {/* Create Message Modal */}
      {showMessageModal && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
          <div className="bg-slate-800 rounded-xl p-6 w-full max-w-lg space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-semibold text-slate-100">New In-App Message</h3>
              <button onClick={() => setShowMessageModal(false)}><X className="w-5 h-5 text-slate-400" /></button>
            </div>
            <input
              placeholder="Title"
              value={newMessage.title}
              onChange={e => setNewMessage(m => ({ ...m, title: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <textarea
              placeholder="Body"
              rows={3}
              value={newMessage.body}
              onChange={e => setNewMessage(m => ({ ...m, body: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <div className="grid grid-cols-2 gap-3">
              <select
                value={newMessage.messageType}
                onChange={e => setNewMessage(m => ({ ...m, messageType: e.target.value }))}
                className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
              >
                <option value="popup">Popup</option>
                <option value="banner">Banner</option>
                <option value="carousel">Carousel</option>
                <option value="spotlight">Spotlight</option>
                <option value="modal">Modal</option>
              </select>
              <select
                value={newMessage.triggerType}
                onChange={e => setNewMessage(m => ({ ...m, triggerType: e.target.value }))}
                className="bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
              >
                <option value="event">Event</option>
                <option value="page_view">Page View</option>
                <option value="time_delay">Time Delay</option>
                <option value="segment">Segment</option>
              </select>
            </div>
            <input
              placeholder="Trigger Event (e.g. button_click)"
              value={newMessage.triggerEvent}
              onChange={e => setNewMessage(m => ({ ...m, triggerEvent: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <input
              placeholder="Trigger Page (e.g. /pricing)"
              value={newMessage.triggerPage}
              onChange={e => setNewMessage(m => ({ ...m, triggerPage: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <select
              value={newMessage.frequency}
              onChange={e => setNewMessage(m => ({ ...m, frequency: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            >
              <option value="once_per_session">Once per session</option>
              <option value="once_per_user">Once per user</option>
              <option value="every_visit">Every visit</option>
              <option value="once_per_day">Once per day</option>
            </select>
            <button
              onClick={handleCreateMessage}
              disabled={!newMessage.title || !newMessage.body}
              className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white py-2 rounded-lg text-sm font-medium"
            >
              Create Message
            </button>
          </div>
        </div>
      )}

      {/* Create Trigger Modal */}
      {showTriggerModal && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
          <div className="bg-slate-800 rounded-xl p-6 w-full max-w-lg space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-semibold text-slate-100">New Proactive Trigger</h3>
              <button onClick={() => setShowTriggerModal(false)}><X className="w-5 h-5 text-slate-400" /></button>
            </div>
            <input
              placeholder="Trigger name"
              value={newTrigger.name}
              onChange={e => setNewTrigger(t => ({ ...t, name: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <select
              value={newTrigger.triggerType}
              onChange={e => setNewTrigger(t => ({ ...t, triggerType: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            >
              <option value="page_view">Page View</option>
              <option value="event">Custom Event</option>
              <option value="exit_intent">Exit Intent</option>
              <option value="inactivity">Inactivity</option>
              <option value="time_on_page">Time on Page</option>
            </select>
            <textarea
              placeholder='Condition JSON (e.g. {"pages": ["/pricing"]})'
              rows={2}
              value={newTrigger.condition}
              onChange={e => setNewTrigger(t => ({ ...t, condition: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200 font-mono"
            />
            <input
              placeholder="Linked message ID (optional)"
              value={newTrigger.messageId}
              onChange={e => setNewTrigger(t => ({ ...t, messageId: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <div>
              <label className="text-xs text-slate-400 block mb-1">Delay (seconds)</label>
              <input
                type="number"
                value={newTrigger.delaySeconds}
                onChange={e => setNewTrigger(t => ({ ...t, delaySeconds: parseInt(e.target.value) || 0 }))}
                className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
              />
            </div>
            <button
              onClick={handleCreateTrigger}
              disabled={!newTrigger.name}
              className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white py-2 rounded-lg text-sm font-medium"
            >
              Create Trigger
            </button>
          </div>
        </div>
      )}

      {/* Evaluate Trigger Modal */}
      {showEvaluateModal && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
          <div className="bg-slate-800 rounded-xl p-6 w-full max-w-lg space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-semibold text-slate-100">Test Trigger Evaluation</h3>
              <button onClick={() => setShowEvaluateModal(false)}><X className="w-5 h-5 text-slate-400" /></button>
            </div>
            <input
              placeholder="Event name (e.g. page_view)"
              value={evaluateForm.eventName}
              onChange={e => setEvaluateForm(f => ({ ...f, eventName: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <input
              placeholder="Page URL (e.g. /pricing)"
              value={evaluateForm.pageUrl}
              onChange={e => setEvaluateForm(f => ({ ...f, pageUrl: e.target.value }))}
              className="w-full bg-slate-700 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200"
            />
            <button
              onClick={handleEvaluate}
              disabled={!evaluateForm.eventName}
              className="w-full bg-amber-600 hover:bg-amber-700 disabled:opacity-50 text-white py-2 rounded-lg text-sm font-medium"
            >
              Evaluate Triggers
            </button>
            {evaluationResults.length > 0 && (
              <div className="space-y-2 max-h-48 overflow-y-auto">
                {evaluationResults.map((r, i) => (
                  <div key={i} className={`p-3 rounded-lg text-sm ${r.matched ? 'bg-emerald-500/20 text-emerald-300' : 'bg-slate-700 text-slate-400'}`}>
                    <div className="font-medium">{r.triggerName || 'Unknown'}</div>
                    <div className="text-xs mt-1">{r.reason}</div>
                    {r.messageId && <div className="text-xs mt-1">Message: {r.messageId}</div>}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
