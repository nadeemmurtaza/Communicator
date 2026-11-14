import {
  Mail,
  Send,
  Settings,
  FileText,
  Activity,
  Plus,
  Trash2,
  Edit3,
  Eye,
  CheckCircle2,
  XCircle,
  AlertCircle,
  RefreshCw,
  Search,
} from '../components/icons'
import { useState, useEffect } from 'react'
import {
  getSmtpSummary,
  listSmtpSettings,
  createSmtpSettings,
  updateSmtpSettings,
  deleteSmtpSettings,
  testSmtpConnection,
  listEmailTemplates,
  createEmailTemplate,
  updateEmailTemplate,
  deleteEmailTemplate,
  sendTemplatedEmail,
  getEmailSendLogs,
} from '../lib/api'
import type {
  SmtpSummary,
  SmtpSettingsItem,
  EmailTemplateItem,
  EmailSendLogItem,
} from '../lib/api'

export function EmailTemplatesDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'smtp' | 'templates' | 'logs' | 'test'>('overview')
  const [loading, setLoading] = useState(true)
  const [summary, setSummary] = useState<SmtpSummary | null>(null)
  const [smtpConfigs, setSmtpConfigs] = useState<SmtpSettingsItem[]>([])
  const [templates, setTemplates] = useState<EmailTemplateItem[]>([])
  const [logs, setLogs] = useState<EmailSendLogItem[]>([])
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null)

  // SMTP form
  const [showSmtpForm, setShowSmtpForm] = useState(false)
  const [editingSmtp, setEditingSmtp] = useState<SmtpSettingsItem | null>(null)
  const [smtpHost, setSmtpHost] = useState('')
  const [smtpPort, setSmtpPort] = useState(587)
  const [smtpSecurity, setSmtpSecurity] = useState('starttls')
  const [smtpAuthMethod, setSmtpAuthMethod] = useState('login')
  const [smtpUsername, setSmtpUsername] = useState('')
  const [smtpPassword, setSmtpPassword] = useState('')
  const [smtpFromAddress, setSmtpFromAddress] = useState('')
  const [smtpFromName, setSmtpFromName] = useState('')
  const [smtpReplyTo, setSmtpReplyTo] = useState('')

  // Template form
  const [showTemplateForm, setShowTemplateForm] = useState(false)
  const [editingTemplate, setEditingTemplate] = useState<EmailTemplateItem | null>(null)
  const [tplName, setTplName] = useState('')
  const [tplSubject, setTplSubject] = useState('')
  const [tplBodyHtml, setTplBodyHtml] = useState('')
  const [tplBodyText, setTplBodyText] = useState('')
  const [tplCategory, setTplCategory] = useState('general')
  const [tplType, setTplType] = useState('transactional')
  const [tplLanguage, setTplLanguage] = useState('en')

  // Test send form
  const [testTo, setTestTo] = useState('')
  const [testTemplate, setTestTemplate] = useState('')
  const [testVariables, setTestVariables] = useState('')
  const [sendingTest, setSendingTest] = useState(false)

  // Template filter
  const [filterCategory, setFilterCategory] = useState('')
  const [filterType, setFilterType] = useState('')
  const [searchQuery, setSearchQuery] = useState('')

  // View template modal
  const [viewingTemplate, setViewingTemplate] = useState<EmailTemplateItem | null>(null)

  const clearFeedback = () => setFeedback(null)

  const loadSummary = async () => {
    try {
      const data = await getSmtpSummary()
      setSummary(data)
    } catch (e: any) {
      console.error('Failed to load SMTP summary:', e)
    }
  }

  const loadSmtpConfigs = async () => {
    try {
      const res: any = await listSmtpSettings()
      setSmtpConfigs(res.smtp_settings || [])
    } catch (e: any) {
      console.error('Failed to load SMTP configs:', e)
    }
  }

  const loadTemplates = async () => {
    try {
      const data = await listEmailTemplates(filterCategory || undefined, filterType || undefined)
      setTemplates(data)
    } catch (e: any) {
      console.error('Failed to load templates:', e)
    }
  }

  const loadLogs = async () => {
    try {
      const data = await getEmailSendLogs(50)
      setLogs(data)
    } catch (e: any) {
      console.error('Failed to load send logs:', e)
    }
  }

  const loadData = async () => {
    setLoading(true)
    await Promise.all([loadSummary(), loadSmtpConfigs(), loadTemplates(), loadLogs()])
    setLoading(false)
  }

  useEffect(() => {
    loadData()
  }, [])

  useEffect(() => {
    loadTemplates()
  }, [filterCategory, filterType])

  // SMTP handlers
  const resetSmtpForm = () => {
    setEditingSmtp(null)
    setSmtpHost('')
    setSmtpPort(587)
    setSmtpSecurity('starttls')
    setSmtpAuthMethod('login')
    setSmtpUsername('')
    setSmtpPassword('')
    setSmtpFromAddress('')
    setSmtpFromName('')
    setSmtpReplyTo('')
    setShowSmtpForm(false)
  }

  const handleEditSmtp = (config: SmtpSettingsItem) => {
    setEditingSmtp(config)
    setSmtpHost(config.host)
    setSmtpPort(config.port)
    setSmtpSecurity(config.security)
    setSmtpAuthMethod(config.authMethod)
    setSmtpUsername(config.username)
    setSmtpFromAddress(config.fromAddress)
    setSmtpFromName(config.fromName)
    setSmtpReplyTo(config.replyToAddress || '')
    setShowSmtpForm(true)
  }

  const handleSaveSmtp = async () => {
    if (!smtpHost || !smtpFromAddress || !smtpUsername || !smtpPassword) {
      setFeedback({ type: 'error', message: 'Host, from address, username, and password are required.' })
      return
    }
    try {
      if (editingSmtp) {
        await updateSmtpSettings(editingSmtp.id, {
          host: smtpHost,
          port: smtpPort,
          security: smtpSecurity,
          authMethod: smtpAuthMethod,
          username: smtpUsername,
          password: smtpPassword,
          fromAddress: smtpFromAddress,
          fromName: smtpFromName,
          replyToAddress: smtpReplyTo || null,
        })
        setFeedback({ type: 'success', message: 'SMTP config updated successfully.' })
      } else {
        await createSmtpSettings({
          host: smtpHost,
          port: smtpPort,
          security: smtpSecurity,
          authMethod: smtpAuthMethod,
          username: smtpUsername,
          password: smtpPassword,
          fromAddress: smtpFromAddress,
          fromName: smtpFromName,
          replyToAddress: smtpReplyTo || undefined,
        })
        setFeedback({ type: 'success', message: 'SMTP config created successfully.' })
      }
      resetSmtpForm()
      loadSmtpConfigs()
      loadSummary()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to save SMTP config.' })
    }
  }

  const handleDeleteSmtp = async (id: string) => {
    if (!confirm('Delete this SMTP config?')) return
    try {
      await deleteSmtpSettings(id)
      setFeedback({ type: 'success', message: 'SMTP config deleted.' })
      loadSmtpConfigs()
      loadSummary()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to delete.' })
    }
  }

  const handleTestSmtp = async (id: string) => {
    const to = prompt('Enter test email address:')
    if (!to) return
    try {
      const result: any = await testSmtpConnection(id, to)
      if (result.success) {
        setFeedback({ type: 'success', message: result.message || 'Test email sent successfully!' })
      } else {
        setFeedback({ type: 'error', message: result.error || 'Test failed.' })
      }
      loadSmtpConfigs()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Test failed.' })
    }
  }

  // Template handlers
  const resetTemplateForm = () => {
    setEditingTemplate(null)
    setTplName('')
    setTplSubject('')
    setTplBodyHtml('')
    setTplBodyText('')
    setTplCategory('general')
    setTplType('transactional')
    setTplLanguage('en')
    setShowTemplateForm(false)
  }

  const handleEditTemplate = (tpl: EmailTemplateItem) => {
    setEditingTemplate(tpl)
    setTplName(tpl.name)
    setTplSubject(tpl.subject)
    setTplBodyHtml(tpl.bodyHtml)
    setTplBodyText(tpl.bodyText)
    setTplCategory(tpl.category)
    setTplType(tpl.templateType)
    setTplLanguage(tpl.language)
    setShowTemplateForm(true)
  }

  const handleSaveTemplate = async () => {
    if (!tplName || !tplSubject || !tplBodyHtml) {
      setFeedback({ type: 'error', message: 'Name, subject, and body are required.' })
      return
    }
    try {
      if (editingTemplate) {
        await updateEmailTemplate(editingTemplate.id, {
          name: tplName,
          subject: tplSubject,
          bodyHtml: tplBodyHtml,
          bodyText: tplBodyText,
          category: tplCategory,
          templateType: tplType,
          language: tplLanguage,
        })
        setFeedback({ type: 'success', message: 'Template updated successfully.' })
      } else {
        await createEmailTemplate({
          name: tplName,
          subject: tplSubject,
          bodyHtml: tplBodyHtml,
          bodyText: tplBodyText,
          category: tplCategory,
          templateType: tplType,
          language: tplLanguage,
        })
        setFeedback({ type: 'success', message: 'Template created successfully.' })
      }
      resetTemplateForm()
      loadTemplates()
      loadSummary()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to save template.' })
    }
  }

  const handleDeleteTemplate = async (id: string) => {
    if (!confirm('Delete this template? Default templates cannot be deleted.')) return
    try {
      await deleteEmailTemplate(id)
      setFeedback({ type: 'success', message: 'Template deleted.' })
      loadTemplates()
      loadSummary()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to delete.' })
    }
  }

  // Test send handler
  const handleTestSend = async () => {
    if (!testTo || !testTemplate) {
      setFeedback({ type: 'error', message: 'Recipient and template are required.' })
      return
    }
    setSendingTest(true)
    try {
      let variables: Record<string, string> = {}
      if (testVariables.trim()) {
        try {
          variables = JSON.parse(testVariables)
        } catch {
          setFeedback({ type: 'error', message: 'Variables must be valid JSON.' })
          setSendingTest(false)
          return
        }
      }
      const result: any = await sendTemplatedEmail({
        toAddress: testTo,
        templateName: testTemplate,
        brandId: null,
        variables,
      })
      if (result.success) {
        setFeedback({ type: 'success', message: `Email sent! Log ID: ${result.logId}` })
        setTestTo('')
        setTestTemplate('')
        setTestVariables('')
        loadLogs()
        loadSummary()
      } else {
        setFeedback({ type: 'error', message: result.error || 'Send failed.' })
      }
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Send failed.' })
    }
    setSendingTest(false)
  }

  const filteredTemplates = templates.filter((t) => {
    if (searchQuery && !t.name.toLowerCase().includes(searchQuery.toLowerCase()) &&
        !t.subject.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false
    }
    return true
  })

  const tabs = [
    { id: 'overview' as const, label: 'Overview', icon: Activity },
    { id: 'smtp' as const, label: 'SMTP Config', icon: Settings },
    { id: 'templates' as const, label: 'Templates', icon: FileText },
    { id: 'logs' as const, label: 'Send Logs', icon: Mail },
    { id: 'test' as const, label: 'Test Send', icon: Send },
  ]

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="w-6 h-6 animate-spin text-[var(--color-ink-muted)]" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Feedback */}
      {feedback && (
        <div className={`p-4 rounded-lg border ${feedback.type === 'success' ? 'bg-emerald-50 border-emerald-200 text-emerald-800' : 'bg-red-50 border-red-200 text-red-800'}`}>
          <div className="flex items-center justify-between">
            <span className="text-sm">{feedback.message}</span>
            <button onClick={clearFeedback} className="ml-4 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
              <XCircle className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 border-b border-[var(--color-border)]">
        {tabs.map((tab) => {
          const Icon = tab.icon
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-4 py-2 text-sm border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-[var(--color-accent)] text-[var(--color-accent)] font-medium'
                  : 'border-transparent text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'
              }`}
            >
              <Icon className="w-4 h-4" />
              {tab.label}
            </button>
          )
        })}
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
              <div className="text-2xs text-[var(--color-ink-muted)] mb-1">SMTP Configs</div>
              <div className="text-2xl font-bold text-[var(--color-ink)]">{summary?.enabledSmtpConfigs || 0}<span className="text-sm text-[var(--color-ink-muted)] font-normal">/{summary?.totalSmtpConfigs || 0}</span></div>
            </div>
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
              <div className="text-2xs text-[var(--color-ink-muted)] mb-1">Templates</div>
              <div className="text-2xl font-bold text-[var(--color-ink)]">{summary?.enabledTemplates || 0}<span className="text-sm text-[var(--color-ink-muted)] font-normal">/{summary?.totalTemplates || 0}</span></div>
            </div>
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
              <div className="text-2xs text-[var(--color-ink-muted)] mb-1">Emails Sent</div>
              <div className="text-2xl font-bold text-emerald-600">{summary?.sentCount || 0}</div>
            </div>
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
              <div className="text-2xs text-[var(--color-ink-muted)] mb-1">Failed</div>
              <div className="text-2xl font-bold text-red-600">{summary?.failedCount || 0}</div>
            </div>
          </div>

          {/* Categories */}
          {summary?.categories && summary.categories.length > 0 && (
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
              <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-3">Templates by Category</h3>
              <div className="space-y-2">
                {summary.categories.map((cat) => (
                  <div key={cat.category} className="flex items-center gap-3">
                    <span className="text-xs text-[var(--color-ink-muted)] w-24 capitalize">{cat.category}</span>
                    <div className="flex-1 bg-[var(--color-page-secondary)] rounded-full h-2">
                      <div
                        className="bg-[var(--color-accent)] h-2 rounded-full"
                        style={{ width: `${Math.min((cat.count / Math.max(...summary.categories.map(c => c.count))) * 100, 100)}%` }}
                      />
                    </div>
                    <span className="text-xs text-[var(--color-ink-muted)] w-6 text-right">{cat.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Logs */}
          {summary?.recent_logs && summary.recent_logs.length > 0 && (
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
              <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-3">Recent Send Activity</h3>
              <div className="space-y-2">
                {summary.recent_logs.slice(0, 5).map((log) => (
                  <div key={log.id} className="flex items-center gap-3 text-xs py-1">
                    <span className={`w-2 h-2 rounded-full ${log.status === 'sent' ? 'bg-emerald-500' : log.status === 'failed' ? 'bg-red-500' : 'bg-amber-500'}`} />
                    <span className="text-[var(--color-ink-muted)] w-28 truncate">{log.toAddress}</span>
                    <span className="text-[var(--color-ink)] flex-1 truncate">{log.subject}</span>
                    <span className={`px-2 py-0.5 rounded text-[10px] font-medium ${
                      log.status === 'sent' ? 'bg-emerald-100 text-emerald-700' :
                      log.status === 'failed' ? 'bg-red-100 text-red-700' :
                      'bg-amber-100 text-amber-700'
                    }`}>{log.status}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* How it works */}
          <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
            <h3 className="text-sm font-semibold text-[var(--color-ink)] mb-3">How Email Templates Work</h3>
            <div className="grid md:grid-cols-3 gap-4 text-xs text-[var(--color-ink-muted)]">
              <div>
                <div className="font-medium text-[var(--color-ink)] mb-1">1. Configure SMTP</div>
                <p>Set up your SMTP server connection with host, port, credentials, and sender address. Test the connection before use.</p>
              </div>
              <div>
                <div className="font-medium text-[var(--color-ink)] mb-1">2. Create Templates</div>
                <p>Design email templates with variable placeholders like <code className="bg-[var(--color-page-secondary)] px-1 rounded">{'{{name}}'}</code> for personalization.</p>
              </div>
              <div>
                <div className="font-medium text-[var(--color-ink)] mb-1">3. Send & Monitor</div>
                <p>Send templated emails via API with variable substitution. All sends are logged for audit and analytics.</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* SMTP Config Tab */}
      {activeTab === 'smtp' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-[var(--color-ink)]">SMTP Configurations</h3>
            <button
              onClick={() => setShowSmtpForm(true)}
              className="flex items-center gap-2 px-3 py-1.5 bg-[var(--color-accent)] text-white rounded text-xs hover:opacity-90"
            >
              <Plus className="w-3 h-3" /> Add Config
            </button>
          </div>

          {/* SMTP Form Modal */}
          {showSmtpForm && (
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4 space-y-3">
              <h4 className="text-sm font-semibold text-[var(--color-ink)]">{editingSmtp ? 'Edit' : 'New'} SMTP Config</h4>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Host *</label>
                  <input type="text" value={smtpHost} onChange={(e) => setSmtpHost(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" placeholder="smtp.example.com" />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Port</label>
                  <input type="number" value={smtpPort} onChange={(e) => setSmtpPort(Number(e.target.value))} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Security</label>
                  <select value={smtpSecurity} onChange={(e) => setSmtpSecurity(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs">
                    <option value="starttls">STARTTLS</option>
                    <option value="tls">TLS</option>
                    <option value="none">None</option>
                  </select>
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Auth Method</label>
                  <select value={smtpAuthMethod} onChange={(e) => setSmtpAuthMethod(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs">
                    <option value="login">LOGIN</option>
                    <option value="plain">PLAIN</option>
                    <option value="none">None</option>
                  </select>
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Username *</label>
                  <input type="text" value={smtpUsername} onChange={(e) => setSmtpUsername(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Password *</label>
                  <input type="password" value={smtpPassword} onChange={(e) => setSmtpPassword(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">From Address *</label>
                  <input type="email" value={smtpFromAddress} onChange={(e) => setSmtpFromAddress(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">From Name</label>
                  <input type="text" value={smtpFromName} onChange={(e) => setSmtpFromName(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
                <div className="col-span-2">
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Reply-To Address</label>
                  <input type="email" value={smtpReplyTo} onChange={(e) => setSmtpReplyTo(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
              </div>
              <div className="flex gap-2 justify-end">
                <button onClick={resetSmtpForm} className="px-3 py-1.5 border border-[var(--color-border)] rounded text-xs text-[var(--color-ink-muted)]">Cancel</button>
                <button onClick={handleSaveSmtp} className="px-3 py-1.5 bg-[var(--color-accent)] text-white rounded text-xs">Save</button>
              </div>
            </div>
          )}

          {/* SMTP Config List */}
          {smtpConfigs.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-ink-muted)] text-sm">
              No SMTP configurations. Add one to get started.
            </div>
          ) : (
            <div className="space-y-2">
              {smtpConfigs.map((config) => (
                <div key={config.id} className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-3 flex items-center gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-[var(--color-ink)]">{config.fromAddress}</span>
                      {config.enabled ? (
                        <span className="px-1.5 py-0.5 bg-emerald-100 text-emerald-700 rounded text-[10px]">enabled</span>
                      ) : (
                        <span className="px-1.5 py-0.5 bg-gray-100 text-gray-600 rounded text-[10px]">disabled</span>
                      )}
                      {config.testStatus === 'success' && (
                        <span className="px-1.5 py-0.5 bg-blue-100 text-blue-700 rounded text-[10px]">tested</span>
                      )}
                    </div>
                    <div className="text-xs text-[var(--color-ink-muted)] mt-0.5">
                      {config.host}:{config.port} · {config.security} · {config.fromName || 'No name'}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <button onClick={() => handleTestSmtp(config.id)} className="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-accent)]" title="Test connection">
                      <Send className="w-3.5 h-3.5" />
                    </button>
                    <button onClick={() => handleEditSmtp(config)} className="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-accent)]" title="Edit">
                      <Edit3 className="w-3.5 h-3.5" />
                    </button>
                    <button onClick={() => handleDeleteSmtp(config.id)} className="p-1.5 text-[var(--color-ink-muted)] hover:text-red-600" title="Delete">
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Templates Tab */}
      {activeTab === 'templates' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="relative">
                <Search className="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-[var(--color-ink-muted)]" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search templates..."
                  className="pl-8 pr-3 py-1.5 rounded border border-[var(--color-border)] text-xs w-48"
                />
              </div>
              <select value={filterCategory} onChange={(e) => setFilterCategory(e.target.value)} className="px-2 py-1.5 rounded border border-[var(--color-border)] text-xs">
                <option value="">All Categories</option>
                <option value="otp">OTP</option>
                <option value="welcome">Welcome</option>
                <option value="password_reset">Password Reset</option>
                <option value="order">Order</option>
                <option value="general">General</option>
              </select>
              <select value={filterType} onChange={(e) => setFilterType(e.target.value)} className="px-2 py-1.5 rounded border border-[var(--color-border)] text-xs">
                <option value="">All Types</option>
                <option value="transactional">Transactional</option>
                <option value="marketing">Marketing</option>
                <option value="notification">Notification</option>
              </select>
            </div>
            <button
              onClick={() => setShowTemplateForm(true)}
              className="flex items-center gap-2 px-3 py-1.5 bg-[var(--color-accent)] text-white rounded text-xs hover:opacity-90"
            >
              <Plus className="w-3 h-3" /> New Template
            </button>
          </div>

          {/* Template Form Modal */}
          {showTemplateForm && (
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4 space-y-3">
              <h4 className="text-sm font-semibold text-[var(--color-ink)]">{editingTemplate ? 'Edit' : 'New'} Template</h4>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Name *</label>
                  <input type="text" value={tplName} onChange={(e) => setTplName(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" placeholder="My Template" />
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Category</label>
                  <select value={tplCategory} onChange={(e) => setTplCategory(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs">
                    <option value="general">General</option>
                    <option value="otp">OTP</option>
                    <option value="welcome">Welcome</option>
                    <option value="password_reset">Password Reset</option>
                    <option value="order">Order</option>
                  </select>
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Type</label>
                  <select value={tplType} onChange={(e) => setTplType(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs">
                    <option value="transactional">Transactional</option>
                    <option value="marketing">Marketing</option>
                    <option value="notification">Notification</option>
                  </select>
                </div>
                <div>
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Language</label>
                  <input type="text" value={tplLanguage} onChange={(e) => setTplLanguage(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" />
                </div>
                <div className="col-span-2">
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Subject *</label>
                  <input type="text" value={tplSubject} onChange={(e) => setTplSubject(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" placeholder="Use {{{variable}}} for placeholders" />
                </div>
                <div className="col-span-2">
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">HTML Body *</label>
                  <textarea value={tplBodyHtml} onChange={(e) => setTplBodyHtml(e.target.value)} rows={6} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs font-mono" placeholder="HTML with {{{variable}}} placeholders" />
                </div>
                <div className="col-span-2">
                  <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Plain Text Body (optional)</label>
                  <textarea value={tplBodyText} onChange={(e) => setTplBodyText(e.target.value)} rows={3} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" placeholder="Plain text fallback" />
                </div>
              </div>
              <div className="flex gap-2 justify-end">
                <button onClick={resetTemplateForm} className="px-3 py-1.5 border border-[var(--color-border)] rounded text-xs text-[var(--color-ink-muted)]">Cancel</button>
                <button onClick={handleSaveTemplate} className="px-3 py-1.5 bg-[var(--color-accent)] text-white rounded text-xs">Save</button>
              </div>
            </div>
          )}

          {/* Template List */}
          {filteredTemplates.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-ink-muted)] text-sm">
              No templates found.
            </div>
          ) : (
            <div className="space-y-2">
              {filteredTemplates.map((tpl) => (
                <div key={tpl.id} className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-3">
                  <div className="flex items-start gap-3">
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-[var(--color-ink)]">{tpl.name}</span>
                        {tpl.isDefault && <span className="px-1.5 py-0.5 bg-purple-100 text-purple-700 rounded text-[10px]">default</span>}
                        {tpl.enabled ? (
                          <span className="px-1.5 py-0.5 bg-emerald-100 text-emerald-700 rounded text-[10px]">enabled</span>
                        ) : (
                          <span className="px-1.5 py-0.5 bg-gray-100 text-gray-600 rounded text-[10px]">disabled</span>
                        )}
                        <span className="px-1.5 py-0.5 bg-[var(--color-page-secondary)] text-[var(--color-ink-muted)] rounded text-[10px] capitalize">{tpl.category}</span>
                      </div>
                      <div className="text-xs text-[var(--color-ink-muted)] mt-0.5">{tpl.subject}</div>
                      {tpl.variables && tpl.variables.length > 0 && (
                        <div className="flex gap-1 mt-1 flex-wrap">
                          {tpl.variables.map((v: string, i: number) => (
                            <span key={i} className="px-1.5 py-0.5 bg-blue-50 text-blue-600 rounded text-[10px] font-mono">{`{{${v}}}`}</span>
                          ))}
                        </div>
                      )}
                    </div>
                    <div className="flex items-center gap-1">
                      <button onClick={() => setViewingTemplate(tpl)} className="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-accent)]" title="View">
                        <Eye className="w-3.5 h-3.5" />
                      </button>
                      <button onClick={() => handleEditTemplate(tpl)} className="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-accent)]" title="Edit">
                        <Edit3 className="w-3.5 h-3.5" />
                      </button>
                      {!tpl.isDefault && (
                        <button onClick={() => handleDeleteTemplate(tpl.id)} className="p-1.5 text-[var(--color-ink-muted)] hover:text-red-600" title="Delete">
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* View Template Modal */}
          {viewingTemplate && (
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4 space-y-3">
              <div className="flex items-center justify-between">
                <h4 className="text-sm font-semibold text-[var(--color-ink)]">{viewingTemplate.name}</h4>
                <button onClick={() => setViewingTemplate(null)} className="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
                  <XCircle className="w-4 h-4" />
                </button>
              </div>
              <div className="text-xs text-[var(--color-ink-muted)]">
                <div><strong>Subject:</strong> {viewingTemplate.subject}</div>
                <div><strong>Category:</strong> {viewingTemplate.category} · <strong>Type:</strong> {viewingTemplate.templateType}</div>
              </div>
              <div>
                <div className="text-2xs text-[var(--color-ink-muted)] mb-1">HTML Preview</div>
                <div className="border border-[var(--color-border)] rounded p-3 max-h-64 overflow-auto bg-white">
                  <div dangerouslySetInnerHTML={{ __html: viewingTemplate.bodyHtml }} />
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Send Logs Tab */}
      {activeTab === 'logs' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-[var(--color-ink)]">Email Send Logs</h3>
            <button onClick={loadLogs} className="flex items-center gap-2 px-3 py-1.5 border border-[var(--color-border)] rounded text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
              <RefreshCw className="w-3 h-3" /> Refresh
            </button>
          </div>

          {logs.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-ink-muted)] text-sm">
              No send logs yet.
            </div>
          ) : (
            <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg overflow-hidden">
              <table className="w-full text-xs">
                <thead>
                  <tr className="border-b border-[var(--color-border)]">
                    <th className="text-left px-3 py-2 text-[var(--color-ink-muted)] font-medium">To</th>
                    <th className="text-left px-3 py-2 text-[var(--color-ink-muted)] font-medium">Subject</th>
                    <th className="text-left px-3 py-2 text-[var(--color-ink-muted)] font-medium">Template</th>
                    <th className="text-left px-3 py-2 text-[var(--color-ink-muted)] font-medium">Status</th>
                    <th className="text-left px-3 py-2 text-[var(--color-ink-muted)] font-medium">Sent At</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map((log) => (
                    <tr key={log.id} className="border-b border-[var(--color-border)] last:border-0">
                      <td className="px-3 py-2 text-[var(--color-ink)]">{log.toAddress}</td>
                      <td className="px-3 py-2 text-[var(--color-ink)] max-w-xs truncate">{log.subject}</td>
                      <td className="px-3 py-2 text-[var(--color-ink-muted)]">{log.templateName || '-'}</td>
                      <td className="px-3 py-2">
                        <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${
                          log.status === 'sent' ? 'bg-emerald-100 text-emerald-700' :
                          log.status === 'failed' ? 'bg-red-100 text-red-700' :
                          log.status === 'bounced' ? 'bg-amber-100 text-amber-700' :
                          'bg-gray-100 text-gray-600'
                        }`}>{log.status}</span>
                      </td>
                      <td className="px-3 py-2 text-[var(--color-ink-muted)]">{log.sentAt ? new Date(log.sentAt).toLocaleString() : '-'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Test Send Tab */}
      {activeTab === 'test' && (
        <div className="space-y-4">
          <h3 className="text-sm font-semibold text-[var(--color-ink)]">Test Send Email</h3>
          <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4 space-y-3 max-w-lg">
            <div>
              <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Recipient Email *</label>
              <input type="email" value={testTo} onChange={(e) => setTestTo(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs" placeholder="test@example.com" />
            </div>
            <div>
              <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Template *</label>
              <select value={testTemplate} onChange={(e) => setTestTemplate(e.target.value)} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs">
                <option value="">Select a template...</option>
                {templates.filter(t => t.enabled).map((tpl) => (
                  <option key={tpl.id} value={tpl.name}>{tpl.name} ({tpl.category})</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-2xs text-[var(--color-ink-muted)] mb-0.5">Variables (JSON, optional)</label>
              <textarea value={testVariables} onChange={(e) => setTestVariables(e.target.value)} rows={3} className="w-full rounded border border-[var(--color-border)] px-2 py-1 text-xs font-mono" placeholder='{"name": "John", "code": "123456"}' />
            </div>
            <button
              onClick={handleTestSend}
              disabled={sendingTest}
              className="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] text-white rounded text-xs hover:opacity-90 disabled:opacity-50"
            >
              {sendingTest ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Send className="w-3 h-3" />}
              {sendingTest ? 'Sending...' : 'Send Test Email'}
            </button>
          </div>

          <div className="bg-[var(--color-page)] border border-[var(--color-border)] rounded-lg p-4">
            <h4 className="text-sm font-semibold text-[var(--color-ink)] mb-2">Available Templates</h4>
            <div className="space-y-1">
              {templates.filter(t => t.enabled).map((tpl) => (
                <div key={tpl.id} className="flex items-center gap-2 text-xs">
                  <CheckCircle2 className="w-3 h-3 text-emerald-500" />
                  <span className="text-[var(--color-ink)]">{tpl.name}</span>
                  <span className="text-[var(--color-ink-muted)]">({tpl.category})</span>
                  {tpl.variables && tpl.variables.length > 0 && (
                    <span className="text-[var(--color-ink-muted)]">— vars: {tpl.variables.join(', ')}</span>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
