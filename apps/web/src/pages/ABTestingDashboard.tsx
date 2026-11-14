import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  getABTestingSummary, listABExperiments, createABExperiment,
  updateABExperimentStatus, deleteABExperiment, getABVariants,
  generateABSummary, getABSettings, updateABSettings, getABResults
} from '@/lib/api'
import type {
  ABTestingSummary, ABExperiment, ABVariant, ABSettings,
  ABExperimentSummary, ABResult
} from '@/lib/api'
import { FlaskConical, Plus, Play, PauseCircle, Trash2, Eye, TrendingUp, AlertCircle, CheckCircle2 } from '@/components/icons'

const EXPERIMENT_TYPES = [
  'confidence_threshold', 'retrieval_strategy', 'response_format',
  'escalation_trigger', 'tone_style', 'model'
]

export function ABTestingDashboard() {
  const [summary, setSummary] = useState<ABTestingSummary | null>(null)
  const [experiments, setExperiments] = useState<ABExperiment[]>([])
  const [settings, setSettings] = useState<ABSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [selectedExperiment, setSelectedExperiment] = useState<ABExperiment | null>(null)
  const [selectedVariants, setSelectedVariants] = useState<ABVariant[]>([])
  const [selectedSummary, setSelectedSummary] = useState<ABExperimentSummary | null>(null)
  const [selectedResults, setSelectedResults] = useState<ABResult[]>([])
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [typeFilter, setTypeFilter] = useState<string>('')

  // Create experiment form
  const [newName, setNewName] = useState('')
  const [newDescription, setNewDescription] = useState('')
  const [newType, setNewType] = useState('confidence_threshold')
  const [newTrafficSplit, setNewTrafficSplit] = useState(0.5)
  const [newVariantA, setNewVariantA] = useState('Control')
  const [newVariantAConfig, setNewVariantAConfig] = useState('{"confidence_threshold": 0.7}')
  const [newVariantB, setNewVariantB] = useState('Variant A')
  const [newVariantBConfig, setNewVariantBConfig] = useState('{"confidence_threshold": 0.5}')

  // Settings form
  const [settingsForm, setSettingsForm] = useState<ABSettings>({
    id: '', org_id: 'org-default', enabled: true, auto_detect_winner: false,
    min_sample_size: 100, significance_threshold: 0.95, max_concurrent_experiments: 5
  })

  useEffect(() => {
    loadAll()
  }, [])

  async function loadAll() {
    setLoading(true)
    try {
      const [sum, exps, sts] = await Promise.all([
        getABTestingSummary(),
        listABExperiments(),
        getABSettings(),
      ])
      setSummary(sum)
      setExperiments(exps)
      setSettings(sts)
      setSettingsForm(sts)
    } catch (e) {
      console.error('Failed to load A/B testing data', e)
    }
    setLoading(false)
  }

  async function handleCreate() {
    try {
      let configA = {}, configB = {}
      try { configA = JSON.parse(newVariantAConfig) } catch { configA = {} }
      try { configB = JSON.parse(newVariantBConfig) } catch { configB = {} }

      await createABExperiment({
        name: newName,
        description: newDescription || undefined,
        experiment_type: newType,
        traffic_split: newTrafficSplit,
        variants: [
          { variant_name: newVariantA, config: configA },
          { variant_name: newVariantB, config: configB },
        ],
      })
      setShowCreateModal(false)
      setNewName('')
      setNewDescription('')
      setNewVariantA('Control')
      setNewVariantAConfig('{"confidence_threshold": 0.7}')
      setNewVariantB('Variant A')
      setNewVariantBConfig('{"confidence_threshold": 0.5}')
      loadAll()
    } catch (e) {
      console.error('Failed to create experiment', e)
    }
  }

  async function handleStatusChange(id: string, status: string) {
    try {
      await updateABExperimentStatus(id, status)
      loadAll()
    } catch (e) {
      console.error('Failed to update status', e)
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this experiment?')) return
    try {
      await deleteABExperiment(id)
      loadAll()
    } catch (e) {
      console.error('Failed to delete', e)
    }
  }

  async function handleView(id: string) {
    setSelectedExperiment(null)
    setSelectedVariants([])
    setSelectedSummary(null)
    setSelectedResults([])
    try {
      const [exp, variants, results] = await Promise.all([
        fetch(`/api/ab-testing/experiments/${id}`).then(r => r.json()),
        getABVariants(id),
        getABResults(id),
      ])
      setSelectedExperiment(exp)
      setSelectedVariants(variants)
      setSelectedResults(results)
    } catch (e) {
      console.error('Failed to load experiment details', e)
    }
  }

  async function handleGenerateSummary(id: string) {
    try {
      const result = await generateABSummary(id)
      setSelectedSummary(result.summary)
    } catch (e) {
      console.error('Failed to generate summary', e)
    }
  }

  async function handleSaveSettings() {
    try {
      const updated = await updateABSettings(settingsForm)
      setSettings(updated)
      setSettingsForm(updated)
    } catch (e) {
      console.error('Failed to save settings', e)
    }
  }

  function statusBadge(status: string) {
    const colors: Record<string, string> = {
      draft: 'bg-gray-100 text-gray-700',
      running: 'bg-green-100 text-green-700',
      paused: 'bg-yellow-100 text-yellow-700',
      completed: 'bg-blue-100 text-blue-700',
      failed: 'bg-red-100 text-red-700',
    }
    return <Badge className={colors[status] || 'bg-gray-100 text-gray-700'}>{status}</Badge>
  }

  function typeLabel(type: string) {
    return type.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
  }

  if (loading) return <div className="p-6">Loading...</div>

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">A/B Testing & Phased Deployments</h1>
          <p className="text-muted-foreground">Test different AI configurations on traffic subsets before full deployment</p>
        </div>
        <Button onClick={() => setShowCreateModal(true)}>
          <Plus className="w-4 h-4 mr-2" />
          New Experiment
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Total Experiments</p>
                <p className="text-3xl font-bold">{summary?.total_experiments ?? 0}</p>
              </div>
              <FlaskConical className="w-8 h-8 text-muted-foreground" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Running</p>
                <p className="text-3xl font-bold text-green-600">{summary?.running_experiments ?? 0}</p>
              </div>
              <Play className="w-8 h-8 text-green-500" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Total Assignments</p>
                <p className="text-3xl font-bold">{summary?.total_assignments ?? 0}</p>
              </div>
              <TrendingUp className="w-8 h-8 text-blue-500" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Total Results</p>
                <p className="text-3xl font-bold">{summary?.total_results ?? 0}</p>
              </div>
              <CheckCircle2 className="w-8 h-8 text-purple-500" />
            </div>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="experiments">
        <TabsList>
          <TabsTrigger value="experiments">Experiments</TabsTrigger>
          <TabsTrigger value="detail">Experiment Detail</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>

        <TabsContent value="experiments" className="space-y-4">
          {/* Filters */}
          <div className="flex gap-4">
            <select
              className="border rounded-md px-3 py-2 text-sm"
              value={statusFilter}
              onChange={e => setStatusFilter(e.target.value)}
            >
              <option value="">All Statuses</option>
              <option value="draft">Draft</option>
              <option value="running">Running</option>
              <option value="paused">Paused</option>
              <option value="completed">Completed</option>
              <option value="failed">Failed</option>
            </select>
            <select
              className="border rounded-md px-3 py-2 text-sm"
              value={typeFilter}
              onChange={e => setTypeFilter(e.target.value)}
            >
              <option value="">All Types</option>
              {EXPERIMENT_TYPES.map(t => (
                <option key={t} value={t}>{typeLabel(t)}</option>
              ))}
            </select>
            <Button variant="outline" onClick={loadAll}>Refresh</Button>
          </div>

          {/* Experiments List */}
          <Card>
            <CardContent className="pt-6">
              {experiments.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <FlaskConical className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No experiments yet. Create one to start testing AI configurations.</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {experiments.map(exp => (
                    <div key={exp.id} className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-1">
                          <h3 className="font-semibold">{exp.name}</h3>
                          {statusBadge(exp.status)}
                          <Badge variant="outline">{typeLabel(exp.experiment_type)}</Badge>
                        </div>
                        <p className="text-sm text-muted-foreground">
                          Traffic split: {Math.round(exp.traffic_split * 100)}% control
                          {exp.description && ` — ${exp.description}`}
                        </p>
                        <p className="text-xs text-muted-foreground mt-1">
                          Created: {new Date(exp.created_at).toLocaleString()}
                        </p>
                      </div>
                      <div className="flex items-center gap-2">
                        {exp.status === 'draft' && (
                          <Button size="sm" onClick={() => handleStatusChange(exp.id, 'running')}>
                            <Play className="w-4 h-4 mr-1" /> Start
                          </Button>
                        )}
                        {exp.status === 'running' && (
                          <Button size="sm" variant="outline" onClick={() => handleStatusChange(exp.id, 'paused')}>
                            <PauseCircle className="w-4 h-4 mr-1" /> Pause
                          </Button>
                        )}
                        {exp.status === 'paused' && (
                          <Button size="sm" onClick={() => handleStatusChange(exp.id, 'running')}>
                            <Play className="w-4 h-4 mr-1" /> Resume
                          </Button>
                        )}
                        <Button size="sm" variant="outline" onClick={() => handleView(exp.id)}>
                          <Eye className="w-4 h-4 mr-1" /> View
                        </Button>
                        <Button size="sm" variant="destructive" onClick={() => handleDelete(exp.id)}>
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Experiment types breakdown */}
          {summary?.experiments_by_type.length !== 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Experiments by Type</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {summary.experiments_by_type.map(bt => (
                    <div key={bt.experiment_type} className="flex items-center gap-3">
                      <span className="w-48 text-sm">{typeLabel(bt.experiment_type)}</span>
                      <div className="flex-1 bg-muted rounded-full h-2">
                        <div
                          className="bg-primary h-2 rounded-full"
                          style={{ width: `${Math.min(100, (bt.count / Math.max(1, summary.total_experiments)) * 100)}%` }}
                        />
                      </div>
                      <span className="text-sm font-medium w-8 text-right">{bt.count}</span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="detail" className="space-y-4">
          {!selectedExperiment ? (
            <Card>
              <CardContent className="pt-6 text-center py-12 text-muted-foreground">
                <Eye className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p>Select an experiment from the Experiments tab to view details.</p>
              </CardContent>
            </Card>
          ) : (
            <>
              {/* Experiment Info */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-3">
                    {selectedExperiment.name}
                    {statusBadge(selectedExperiment.status)}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-muted-foreground">Type:</span>{' '}
                      {typeLabel(selectedExperiment.experiment_type)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Traffic Split:</span>{' '}
                      {Math.round(selectedExperiment.traffic_split * 100)}% control
                    </div>
                    <div>
                      <span className="text-muted-foreground">Started:</span>{' '}
                      {selectedExperiment.start_at ? new Date(selectedExperiment.start_at).toLocaleString() : 'Not started'}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Ended:</span>{' '}
                      {selectedExperiment.end_at ? new Date(selectedExperiment.end_at).toLocaleString() : 'Still running'}
                    </div>
                    {selectedExperiment.description && (
                      <div className="col-span-2">
                        <span className="text-muted-foreground">Description:</span>{' '}
                        {selectedExperiment.description}
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>

              {/* Variants */}
              <Card>
                <CardHeader>
                  <CardTitle>Variants</CardTitle>
                </CardHeader>
                <CardContent>
                  {selectedVariants.length === 0 ? (
                    <p className="text-muted-foreground">No variants found.</p>
                  ) : (
                    <div className="space-y-3">
                      {selectedVariants.map(v => (
                        <div key={v.id} className="p-4 border rounded-lg">
                          <div className="flex items-center gap-3 mb-2">
                            <span className="font-semibold">{v.variant_name}</span>
                            {v.is_control && <Badge>Control</Badge>}
                          </div>
                          <pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-32">
                            {JSON.stringify(v.config, null, 2)}
                          </pre>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>

              {/* Summary & Results */}
              <div className="grid grid-cols-2 gap-4">
                <Card>
                  <CardHeader className="flex flex-row items-center justify-between">
                    <CardTitle>Summary</CardTitle>
                    <Button size="sm" onClick={() => handleGenerateSummary(selectedExperiment.id)}>
                      Generate
                    </Button>
                  </CardHeader>
                  <CardContent>
                    {!selectedSummary ? (
                      <p className="text-muted-foreground">Click "Generate" to analyze this experiment.</p>
                    ) : (
                      <div className="space-y-3">
                        <div className="grid grid-cols-2 gap-2 text-sm">
                          <div>Assignments: <strong>{selectedSummary.total_assignments}</strong></div>
                          <div>Results: <strong>{selectedSummary.total_results}</strong></div>
                          <div>Significance: <strong>{Math.round(selectedSummary.statistical_significance * 100)}%</strong></div>
                          <div>Winner: <strong>{selectedSummary.recommended_winner ? 'Detected' : 'None yet'}</strong></div>
                        </div>
                        {selectedSummary.variant_stats.map(vs => (
                          <div key={vs.variant_id} className="p-3 border rounded">
                            <div className="font-medium text-sm">{vs.variant_name}</div>
                            <div className="text-xs text-muted-foreground">
                              {vs.assignments} assignments, {vs.results} results
                            </div>
                            {Object.entries(vs.avg_metric_values).map(([key, val]) => (
                              <div key={key} className="text-xs">
                                {key}: {val.toFixed(3)}
                              </div>
                            ))}
                          </div>
                        ))}
                      </div>
                    )}
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Recent Results ({selectedResults.length})</CardTitle>
                  </CardHeader>
                  <CardContent>
                    {selectedResults.length === 0 ? (
                      <p className="text-muted-foreground">No results recorded yet.</p>
                    ) : (
                      <div className="space-y-2 max-h-64 overflow-auto">
                        {selectedResults.slice(0, 20).map(r => (
                          <div key={r.id} className="text-xs p-2 border rounded">
                            <span className="font-medium">{r.metric_name}</span>: {r.metric_value.toFixed(3)}
                            <span className="text-muted-foreground ml-2">
                              {new Date(r.created_at).toLocaleString()}
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </CardContent>
                </Card>
              </div>
            </>
          )}
        </TabsContent>

        <TabsContent value="settings" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>A/B Testing Settings</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between p-3 border rounded">
                <div>
                  <Label>A/B Testing Enabled</Label>
                  <p className="text-sm text-muted-foreground">Enable or disable A/B testing globally</p>
                </div>
                <input
                  type="checkbox"
                  checked={settingsForm.enabled}
                  onChange={e => setSettingsForm({ ...settingsForm, enabled: e.target.checked })}
                  className="w-5 h-5"
                />
              </div>
              <div className="flex items-center justify-between p-3 border rounded">
                <div>
                  <Label>Auto-Detect Winner</Label>
                  <p className="text-sm text-muted-foreground">Automatically detect winning variant when significance is reached</p>
                </div>
                <input
                  type="checkbox"
                  checked={settingsForm.auto_detect_winner}
                  onChange={e => setSettingsForm({ ...settingsForm, auto_detect_winner: e.target.checked })}
                  className="w-5 h-5"
                />
              </div>
              <div className="grid grid-cols-3 gap-4">
                <div>
                  <Label>Min Sample Size</Label>
                  <Input
                    type="number"
                    value={settingsForm.min_sample_size}
                    onChange={e => setSettingsForm({ ...settingsForm, min_sample_size: parseInt(e.target.value) || 100 })}
                  />
                </div>
                <div>
                  <Label>Significance Threshold</Label>
                  <Input
                    type="number"
                    step="0.01"
                    min="0"
                    max="1"
                    value={settingsForm.significance_threshold}
                    onChange={e => setSettingsForm({ ...settingsForm, significance_threshold: parseFloat(e.target.value) || 0.95 })}
                  />
                </div>
                <div>
                  <Label>Max Concurrent Experiments</Label>
                  <Input
                    type="number"
                    value={settingsForm.max_concurrent_experiments}
                    onChange={e => setSettingsForm({ ...settingsForm, max_concurrent_experiments: parseInt(e.target.value) || 5 })}
                  />
                </div>
              </div>
              <Button onClick={handleSaveSettings}>Save Settings</Button>
            </CardContent>
          </Card>

          {/* How it works */}
          <Card>
            <CardHeader>
              <CardTitle>How A/B Testing Works</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <div className="flex gap-3">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-bold text-primary shrink-0">1</div>
                <div>
                  <strong>Create an experiment</strong> — Define what you want to test (confidence threshold, retrieval strategy, etc.) and configure variants.
                </div>
              </div>
              <div className="flex gap-3">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-bold text-primary shrink-0">2</div>
                <div>
                  <strong>Start the experiment</strong> — Traffic is split between variants based on your configured ratio.
                </div>
              </div>
              <div className="flex gap-3">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-bold text-primary shrink-0">3</div>
                <div>
                  <strong>Collect results</strong> — Metrics like resolution rate, satisfaction, and response time are recorded for each variant.
                </div>
              </div>
              <div className="flex gap-3">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center font-bold text-primary shrink-0">4</div>
                <div>
                  <strong>Analyze & deploy</strong> — Generate a summary to see statistical significance and the recommended winner. Deploy the winning configuration.
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Create Experiment Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-2xl max-h-[90vh] overflow-auto">
            <CardHeader>
              <CardTitle>Create New Experiment</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label>Name</Label>
                <Input value={newName} onChange={e => setNewName(e.target.value)} placeholder="e.g., Lower Confidence Threshold Test" />
              </div>
              <div>
                <Label>Description</Label>
                <Textarea value={newDescription} onChange={e => setNewDescription(e.target.value)} placeholder="What are you testing and why?" />
              </div>
              <div>
                <Label>Experiment Type</Label>
                <select
                  className="w-full border rounded-md px-3 py-2"
                  value={newType}
                  onChange={e => setNewType(e.target.value)}
                >
                  {EXPERIMENT_TYPES.map(t => (
                    <option key={t} value={t}>{typeLabel(t)}</option>
                  ))}
                </select>
              </div>
              <div>
                <Label>Traffic Split (Control %): {Math.round(newTrafficSplit * 100)}%</Label>
                <Input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={newTrafficSplit}
                  onChange={e => setNewTrafficSplit(parseFloat(e.target.value))}
                />
              </div>

              <div className="border rounded-lg p-4 space-y-3">
                <h4 className="font-medium">Variant A (Control)</h4>
                <div>
                  <Label>Name</Label>
                  <Input value={newVariantA} onChange={e => setNewVariantA(e.target.value)} />
                </div>
                <div>
                  <Label>Config (JSON)</Label>
                  <Textarea
                    className="font-mono text-sm"
                    rows={3}
                    value={newVariantAConfig}
                    onChange={e => setNewVariantAConfig(e.target.value)}
                  />
                </div>
              </div>

              <div className="border rounded-lg p-4 space-y-3">
                <h4 className="font-medium">Variant B (Test)</h4>
                <div>
                  <Label>Name</Label>
                  <Input value={newVariantB} onChange={e => setNewVariantB(e.target.value)} />
                </div>
                <div>
                  <Label>Config (JSON)</Label>
                  <Textarea
                    className="font-mono text-sm"
                    rows={3}
                    value={newVariantBConfig}
                    onChange={e => setNewVariantBConfig(e.target.value)}
                  />
                </div>
              </div>

              <div className="flex gap-3 justify-end">
                <Button variant="outline" onClick={() => setShowCreateModal(false)}>Cancel</Button>
                <Button onClick={handleCreate} disabled={!newName.trim()}>Create Experiment</Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
