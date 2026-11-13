import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  getBrandOverrideSummary,
  listBrandOverrideRules,
  createBrandOverrideRule,
  updateBrandOverrideRule,
  deleteBrandOverrideRule,
  getBrandOverrideSettings,
  updateBrandOverrideSettings,
  listBrandHierarchy,
  addBrandHierarchy,
  removeBrandHierarchy,
  type BrandOverrideSummary,
  type BrandOverrideRule,
  type BrandOverrideSettings,
  type BrandHierarchy,
} from '@/lib/api'
import { Layers, ArrowRightLeft, Plus, Trash2, ToggleLeft, ToggleRight } from '@/components/icons'

export function BrandOverridesDashboard() {
  const [summary, setSummary] = useState<BrandOverrideSummary | null>(null)
  const [rules, setRules] = useState<BrandOverrideRule[]>([])
  const [settings, setSettings] = useState<BrandOverrideSettings | null>(null)
  const [hierarchy, setHierarchy] = useState<BrandHierarchy[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreateRule, setShowCreateRule] = useState(false)
  const [showAddHierarchy, setShowAddHierarchy] = useState(false)
  const [activeFilter, setActiveFilter] = useState<boolean | undefined>(undefined)

  // New rule form
  const [newRuleName, setNewRuleName] = useState('')
  const [newRuleTriggerType, setNewRuleTriggerType] = useState('workflow')
  const [newRuleToBrandId, setNewRuleToBrandId] = useState('')
  const [newRuleFromBrandId, setNewRuleFromBrandId] = useState('')

  // New hierarchy form
  const [newParentBrandId, setNewParentBrandId] = useState('')
  const [newSubBrandId, setNewSubBrandId] = useState('')

  const loadData = async () => {
    try {
      const [sum, rulesData, settingsData, hierData] = await Promise.all([
        getBrandOverrideSummary(),
        listBrandOverrideRules(undefined, activeFilter),
        getBrandOverrideSettings(),
        listBrandHierarchy(),
      ])
      setSummary(sum)
      setRules(rulesData)
      setSettings(settingsData)
      setHierarchy(hierData)
    } catch (e) {
      console.error('Failed to load brand override data:', e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [activeFilter])

  const handleCreateRule = async () => {
    if (!newRuleName || !newRuleToBrandId) return
    try {
      await createBrandOverrideRule({
        organizationId: 'org-default',
        name: newRuleName,
        triggerType: newRuleTriggerType,
        fromBrandId: newRuleFromBrandId || undefined,
        toBrandId: newRuleToBrandId,
      })
      setNewRuleName('')
      setNewRuleToBrandId('')
      setNewRuleFromBrandId('')
      setNewRuleTriggerType('workflow')
      setShowCreateRule(false)
      loadData()
    } catch (e) {
      console.error('Failed to create rule:', e)
    }
  }

  const handleToggleRuleActive = async (rule: BrandOverrideRule) => {
    try {
      await updateBrandOverrideRule(rule.id, { is_active: !rule.isActive })
      loadData()
    } catch (e) {
      console.error('Failed to update rule:', e)
    }
  }

  const handleDeleteRule = async (ruleId: string) => {
    try {
      await deleteBrandOverrideRule(ruleId)
      loadData()
    } catch (e) {
      console.error('Failed to delete rule:', e)
    }
  }

  const handleAddHierarchy = async () => {
    if (!newParentBrandId || !newSubBrandId) return
    try {
      await addBrandHierarchy({
        parentBrandId: newParentBrandId,
        subBrandId: newSubBrandId,
      })
      setNewParentBrandId('')
      setNewSubBrandId('')
      setShowAddHierarchy(false)
      loadData()
    } catch (e) {
      console.error('Failed to add hierarchy:', e)
    }
  }

  const handleRemoveHierarchy = async (item: BrandHierarchy) => {
    try {
      await removeBrandHierarchy({
        parentBrandId: item.parentBrandId,
        subBrandId: item.subBrandId,
      })
      loadData()
    } catch (e) {
      console.error('Failed to remove hierarchy:', e)
    }
  }

  const handleUpdateSetting = async (key: string, value: boolean) => {
    try {
      const body: Record<string, boolean> = {}
      body[key] = value
      const updated = await updateBrandOverrideSettings(body)
      setSettings(updated)
    } catch (e) {
      console.error('Failed to update setting:', e)
    }
  }

  if (loading) return <div className="flex items-center justify-center h-64">Loading...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Layers className="h-6 w-6 text-muted-foreground" />
        <h1 className="text-2xl font-bold">Brand Overrides</h1>
      </div>

      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="rules">Rules</TabsTrigger>
          <TabsTrigger value="hierarchy">Hierarchy</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">Total Rules</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold">{summary?.totalRules ?? 0}</div>
                <p className="text-xs text-muted-foreground">{summary?.activeRules ?? 0} active</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">Switches (7d)</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold">{summary?.totalSwitches7d ?? 0}</div>
                <p className="text-xs text-muted-foreground">Last 7 days</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">Switches (30d)</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold">{summary?.totalSwitches30d ?? 0}</div>
                <p className="text-xs text-muted-foreground">Last 30 days</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">Hierarchy</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold">{summary?.hierarchyCount ?? 0}</div>
                <p className="text-xs text-muted-foreground">Brand relationships</p>
              </CardContent>
            </Card>
          </div>

          {/* Top Switched Brands */}
          {summary?.topSwitchedBrands && summary.topSwitchedBrands.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Top Switched-To Brands</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {summary.topSwitchedBrands.map((item, i) => (
                    <div key={i} className="flex items-center justify-between">
                      <span className="text-sm">{item.brandName || 'Unknown'}</span>
                      <div className="flex items-center gap-2">
                        <div className="w-48 h-2 bg-muted rounded-full overflow-hidden">
                          <div
                            className="h-full bg-primary rounded-full"
                            style={{
                              width: `${Math.min(100, (item.switchCount / (summary.topSwitchedBrands[0]?.switchCount || 1)) * 100)}%`,
                            }}
                          />
                        </div>
                        <span className="text-sm font-medium w-8 text-right">{item.switchCount}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Recent Switches */}
          {summary?.recentSwitches && summary.recentSwitches.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Recent Switches</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {summary.recentSwitches.slice(0, 5).map((sw) => (
                    <div key={sw.id} className="flex items-center gap-3 text-sm p-2 rounded-lg bg-muted/50">
                      <ArrowRightLeft className="h-4 w-4 text-muted-foreground" />
                      <span>{sw.fromBrandName || sw.fromBrandId}</span>
                      <span className="text-muted-foreground">→</span>
                      <span className="font-medium">{sw.toBrandName || sw.toBrandId}</span>
                      <Badge variant="outline" className="ml-auto">{sw.triggerType}</Badge>
                      <span className="text-xs text-muted-foreground">{new Date(sw.switchedAt).toLocaleString()}</span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* Rules Tab */}
        <TabsContent value="rules" className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setActiveFilter(undefined)}
                className={activeFilter === undefined ? 'bg-muted' : ''}
              >
                All
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setActiveFilter(true)}
                className={activeFilter === true ? 'bg-muted' : ''}
              >
                Active
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setActiveFilter(false)}
                className={activeFilter === false ? 'bg-muted' : ''}
              >
                Inactive
              </Button>
            </div>
            <Button size="sm" onClick={() => setShowCreateRule(true)}>
              <Plus className="h-4 w-4 mr-1" /> New Rule
            </Button>
          </div>

          {showCreateRule && (
            <Card>
              <CardHeader>
                <CardTitle>Create Override Rule</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>Rule Name</Label>
                    <Input
                      value={newRuleName}
                      onChange={(e) => setNewRuleName(e.target.value)}
                      placeholder="e.g., Switch to Sub-brand"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Trigger Type</Label>
                    <select
                      value={newRuleTriggerType}
                      onChange={(e) => setNewRuleTriggerType(e.target.value)}
                      className="w-full h-10 rounded-md border border-input bg-background px-3 text-sm"
                    >
                      <option value="workflow">Workflow</option>
                      <option value="keyword">Keyword</option>
                      <option value="segment">Segment</option>
                      <option value="manual">Manual</option>
                    </select>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>From Brand ID (optional)</Label>
                    <Input
                      value={newRuleFromBrandId}
                      onChange={(e) => setNewRuleFromBrandId(e.target.value)}
                      placeholder="e.g., brand-parent"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>To Brand ID</Label>
                    <Input
                      value={newRuleToBrandId}
                      onChange={(e) => setNewRuleToBrandId(e.target.value)}
                      placeholder="e.g., brand-sub"
                    />
                  </div>
                </div>
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setShowCreateRule(false)}>Cancel</Button>
                  <Button onClick={handleCreateRule}>Create Rule</Button>
                </div>
              </CardContent>
            </Card>
          )}

          <div className="space-y-3">
            {rules.map((rule) => (
              <Card key={rule.id}>
                <CardContent className="flex items-center gap-4 p-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium truncate">{rule.name}</span>
                      <Badge variant={rule.isActive ? 'default' : 'secondary'}>
                        {rule.isActive ? 'Active' : 'Inactive'}
                      </Badge>
                      <Badge variant="outline">{rule.triggerType}</Badge>
                    </div>
                    <div className="flex items-center gap-2 text-sm text-muted-foreground mt-1">
                      <span>{rule.fromBrandName || 'Any brand'}</span>
                      <ArrowRightLeft className="h-3 w-3" />
                      <span className="font-medium text-foreground">{rule.toBrandName || rule.toBrandId}</span>
                    </div>
                    <div className="flex items-center gap-3 text-xs text-muted-foreground mt-1">
                      {rule.autoSwitchKb && <Badge variant="outline" className="text-xs">KB</Badge>}
                      {rule.autoSwitchAiConfig && <Badge variant="outline" className="text-xs">AI</Badge>}
                      {rule.autoSwitchMessenger && <Badge variant="outline" className="text-xs">Messenger</Badge>}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleToggleRuleActive(rule)}
                      title={rule.isActive ? 'Disable' : 'Enable'}
                    >
                      {rule.isActive ? (
                        <ToggleRight className="h-5 w-5 text-green-600" />
                      ) : (
                        <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                      )}
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleDeleteRule(rule.id)}
                      title="Delete"
                    >
                      <Trash2 className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
            {rules.length === 0 && (
              <Card>
                <CardContent className="p-8 text-center text-muted-foreground">
                  No rules found. Create your first brand override rule.
                </CardContent>
              </Card>
            )}
          </div>
        </TabsContent>

        {/* Hierarchy Tab */}
        <TabsContent value="hierarchy" className="space-y-6">
          <div className="flex justify-end">
            <Button size="sm" onClick={() => setShowAddHierarchy(true)}>
              <Plus className="h-4 w-4 mr-1" /> Add Relationship
            </Button>
          </div>

          {showAddHierarchy && (
            <Card>
              <CardHeader>
                <CardTitle>Add Brand Relationship</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>Parent Brand ID</Label>
                    <Input
                      value={newParentBrandId}
                      onChange={(e) => setNewParentBrandId(e.target.value)}
                      placeholder="e.g., brand-parent"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Sub-brand ID</Label>
                    <Input
                      value={newSubBrandId}
                      onChange={(e) => setNewSubBrandId(e.target.value)}
                      placeholder="e.g., brand-sub"
                    />
                  </div>
                </div>
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setShowAddHierarchy(false)}>Cancel</Button>
                  <Button onClick={handleAddHierarchy}>Add</Button>
                </div>
              </CardContent>
            </Card>
          )}

          <div className="space-y-3">
            {hierarchy.map((item) => (
              <Card key={item.id}>
                <CardContent className="flex items-center gap-4 p-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{item.parentBrandName || item.parentBrandId}</span>
                      <ArrowRightLeft className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">{item.subBrandName || item.subBrandId}</span>
                      <Badge variant="outline">{item.relationshipType}</Badge>
                      <Badge variant="secondary">Priority: {item.priority}</Badge>
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleRemoveHierarchy(item)}
                    title="Remove"
                  >
                    <Trash2 className="h-4 w-4 text-red-500" />
                  </Button>
                </CardContent>
              </Card>
            ))}
            {hierarchy.length === 0 && (
              <Card>
                <CardContent className="p-8 text-center text-muted-foreground">
                  No brand relationships configured. Add a parent-sub-brand relationship.
                </CardContent>
              </Card>
            )}
          </div>
        </TabsContent>

        {/* Settings Tab */}
        <TabsContent value="settings" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Brand Override Settings</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {settings && (
                <>
                  <div className="flex items-center justify-between p-4 rounded-lg border">
                    <div>
                      <div className="font-medium">Enable Brand Overrides</div>
                      <div className="text-sm text-muted-foreground">Allow brand switching mid-conversation</div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleUpdateSetting('enabled', !settings.enabled)}
                    >
                      {settings.enabled ? (
                        <ToggleRight className="h-5 w-5 text-green-600" />
                      ) : (
                        <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                      )}
                    </Button>
                  </div>

                  <div className="flex items-center justify-between p-4 rounded-lg border">
                    <div>
                      <div className="font-medium">Allow Cross-Organization Brands</div>
                      <div className="text-sm text-muted-foreground">Permit switching to brands from other organizations</div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleUpdateSetting('allowCrossOrgBrands', !settings.allowCrossOrgBrands)}
                    >
                      {settings.allowCrossOrgBrands ? (
                        <ToggleRight className="h-5 w-5 text-green-600" />
                      ) : (
                        <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                      )}
                    </Button>
                  </div>

                  <div className="flex items-center justify-between p-4 rounded-lg border">
                    <div>
                      <div className="font-medium">Require Approval</div>
                      <div className="text-sm text-muted-foreground">Require human approval before brand switch</div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleUpdateSetting('requireApproval', !settings.requireApproval)}
                    >
                      {settings.requireApproval ? (
                        <ToggleRight className="h-5 w-5 text-green-600" />
                      ) : (
                        <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                      )}
                    </Button>
                  </div>

                  <div className="flex items-center justify-between p-4 rounded-lg border">
                    <div>
                      <div className="font-medium">Log All Switches</div>
                      <div className="text-sm text-muted-foreground">Record every brand switch for auditing</div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleUpdateSetting('logAllSwitches', !settings.logAllSwitches)}
                    >
                      {settings.logAllSwitches ? (
                        <ToggleRight className="h-5 w-5 text-green-600" />
                      ) : (
                        <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                      )}
                    </Button>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
