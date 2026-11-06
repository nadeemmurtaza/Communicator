import { useEffect, useState } from 'react';
import {
  Calendar, Send, BarChart3, Users, Settings, Plus, Play, Pause,
  X, MessageSquare, FolderOpen, Globe, Zap, Eye, RefreshCw, Trash2,
  Copy, CheckCircle, AlertCircle, Clock, TrendingUp, AlertTriangle,
  ChevronRight, Search, Filter, Edit3, Save
} from 'lucide-react';
import {
  listCampaigns, createCampaign, startCampaign, pauseCampaign,
  cancelCampaign, deleteCampaign, addCampaignContacts, retryCampaign,
  getCampaignContacts, getCampaignAnalytics,
  listFlows, createFlow, updateFlow, deleteFlow, duplicateFlow,
  listIndustries, createIndustry,
  listTemplateGroups, createTemplateGroup,
  listFlowSteps, createFlowStep, deleteFlowStep,
  getMessageLimits, syncMessageLimits,
  syncTemplates, listTemplates,
  aiSuggestCampaign, aiGenerateCampaign,
  listContacts
} from '../lib/api';

export default function WhatsAppCampaigns() {
  const [activeTab, setActiveTab] = useState('campaigns');
  const [brandId, setBrandId] = useState(localStorage.getItem('selectedBrand') || '');

  useEffect(() => {
    setBrandId(localStorage.getItem('selectedBrand') || '');
  }, []);

  const tabs = [
    { id: 'campaigns', label: 'Campaigns', icon: Calendar },
    { id: 'templates', label: 'Templates & Flows', icon: MessageSquare },
    { id: 'flow-editor', label: 'Flow Editor', icon: Edit3 },
    { id: 'industries', label: 'Industries', icon: Globe },
    { id: 'limits', label: 'Message Limits', icon: BarChart3 },
    { id: 'contacts', label: 'Contact Selection', icon: Users },
    { id: 'analytics', label: 'Analytics', icon: TrendingUp },
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">WhatsApp Campaigns</h1>
            <p className="text-sm text-gray-500 mt-1">
              Manage campaigns, templates, flows, and message limits
            </p>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => syncTemplates(brandId).catch(() => {})}
              className="flex items-center gap-2 px-3 py-2 text-sm bg-blue-50 text-blue-700 rounded-lg hover:bg-blue-100"
            >
              <RefreshCw className="w-4 h-4" />
              Sync Templates
            </button>
            <button
              onClick={() => syncMessageLimits(brandId).catch(() => {})}
              className="flex items-center gap-2 px-3 py-2 text-sm bg-green-50 text-green-700 rounded-lg hover:bg-green-100"
            >
              <Zap className="w-4 h-4" />
              Sync Limits
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white border-b border-gray-200 px-6">
        <div className="flex gap-1 overflow-x-auto">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
                  activeTab === tab.id
                    ? 'border-blue-600 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Content */}
      <div className="p-6">
        {activeTab === 'campaigns' && <CampaignsTab brandId={brandId} />}
        {activeTab === 'templates' && <TemplatesTab brandId={brandId} />}
        {activeTab === 'flow-editor' && <FlowEditorTab brandId={brandId} />}
        {activeTab === 'industries' && <IndustriesTab brandId={brandId} />}
        {activeTab === 'limits' && <LimitsTab brandId={brandId} />}
        {activeTab === 'contacts' && <ContactsTab brandId={brandId} />}
        {activeTab === 'analytics' && <AnalyticsTab brandId={brandId} />}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Campaigns Tab
// ---------------------------------------------------------------------------

function CampaignsTab({ brandId }) {
  const [campaigns, setCampaigns] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [showAI, setShowAI] = useState(false);
  const [aiSuggestions, setAiSuggestions] = useState([]);
  const [aiContent, setAiContent] = useState(null);
  const [newCampaign, setNewCampaign] = useState({
    name: '', campaign_type: 'whatsapp', template_flow_id: '', scheduled_at: ''
  });

  const loadCampaigns = () => {
    listCampaigns(brandId).then(setCampaigns).finally(() => setLoading(false));
  };

  useEffect(() => { loadCampaigns(); }, [brandId]);

  const handleCreate = () => {
    createCampaign(brandId, newCampaign).then(() => {
      setShowCreate(false);
      setNewCampaign({ name: '', campaign_type: 'whatsapp', template_flow_id: '', scheduled_at: '' });
      loadCampaigns();
    });
  };

  const handleAISuggest = () => {
    aiSuggestCampaign(brandId, {}).then(data => {
      setAiSuggestions(data.suggestions || []);
      setShowAI(true);
    });
  };

  const handleAIGenerate = (suggestion) => {
    aiGenerateCampaign(brandId, {
      campaign_type: suggestion?.category || 'promotional',
      audience: 'existing customers',
      goal: suggestion?.description || 'increase engagement'
    }).then(data => {
      setAiContent(data);
    });
  };

  const statusBadge = (status) => {
    const colors = {
      draft: 'bg-gray-100 text-gray-700',
      running: 'bg-blue-100 text-blue-700',
      paused: 'bg-yellow-100 text-yellow-700',
      completed: 'bg-green-100 text-green-700',
      cancelled: 'bg-red-100 text-red-700',
    };
    return (
      <span className={`px-2 py-1 text-xs font-medium rounded-full ${colors[status] || colors.draft}`}>
        {status}
      </span>
    );
  };

  if (loading) return <div className="flex justify-center py-12"><div className="animate-spin w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full" /></div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">Campaigns</h2>
        <div className="flex gap-2">
          <button onClick={handleAISuggest} className="flex items-center gap-2 px-3 py-2 text-sm bg-purple-50 text-purple-700 rounded-lg hover:bg-purple-100">
            <Zap className="w-4 h-4" /> AI Suggest
          </button>
          <button onClick={() => setShowCreate(true)} className="flex items-center gap-2 px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">
            <Plus className="w-4 h-4" /> New Campaign
          </button>
        </div>
      </div>

      {campaigns.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-xl border border-gray-200">
          <Calendar className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">No campaigns yet. Create one to get started.</p>
        </div>
      ) : (
        <div className="grid gap-4">
          {campaigns.map((c) => (
            <div key={c.id} className="bg-white rounded-xl border border-gray-200 p-5">
              <div className="flex items-start justify-between">
                <div>
                  <div className="flex items-center gap-3">
                    <h3 className="font-semibold text-gray-900">{c.name}</h3>
                    {statusBadge(c.status)}
                  </div>
                  <p className="text-sm text-gray-500 mt-1">
                    {c.campaign_type} · {c.total_contacts} contacts · Sent: {c.sent_count} / Failed: {c.failed_count}
                  </p>
                  <div className="flex items-center gap-4 mt-3 text-xs text-gray-400">
                    <span className="flex items-center gap-1"><Clock className="w-3 h-3" /> Created {c.created_at?.slice(0, 10)}</span>
                    {c.started_at && <span className="flex items-center gap-1"><Play className="w-3 h-3" /> Started {c.started_at?.slice(0, 10)}</span>}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {c.status === 'draft' && (
                    <button onClick={() => startCampaign(c.id).then(loadCampaigns)} className="p-2 text-green-600 hover:bg-green-50 rounded-lg" title="Start">
                      <Play className="w-4 h-4" />
                    </button>
                  )}
                  {c.status === 'running' && (
                    <button onClick={() => pauseCampaign(c.id).then(loadCampaigns)} className="p-2 text-yellow-600 hover:bg-yellow-50 rounded-lg" title="Pause">
                      <Pause className="w-4 h-4" />
                    </button>
                  )}
                  {(c.status === 'running' || c.status === 'paused') && (
                    <button onClick={() => cancelCampaign(c.id).then(loadCampaigns)} className="p-2 text-red-600 hover:bg-red-50 rounded-lg" title="Cancel">
                      <X className="w-4 h-4" />
                    </button>
                  )}
                  {c.failed_count > 0 && (
                    <button onClick={() => retryCampaign(c.id).then(loadCampaigns)} className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg" title="Retry Failed">
                      <RefreshCw className="w-4 h-4" />
                    </button>
                  )}
                  <button onClick={() => deleteCampaign(c.id).then(loadCampaigns)} className="p-2 text-gray-400 hover:bg-gray-50 rounded-lg" title="Delete">
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
              {c.total_contacts > 0 && (
                <div className="mt-3">
                  <div className="w-full bg-gray-100 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full transition-all"
                      style={{ width: `${(c.sent_count / c.total_contacts) * 100}%` }}
                    />
                  </div>
                  <p className="text-xs text-gray-400 mt-1">{Math.round((c.sent_count / c.total_contacts) * 100)}% sent</p>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Create Modal */}
      {showCreate && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">New Campaign</h3>
            <input
              className="w-full px-3 py-2 border border-gray-300 rounded-lg mb-3"
              placeholder="Campaign name"
              value={newCampaign.name}
              onChange={e => setNewCampaign({ ...newCampaign, name: e.target.value })}
            />
            <select
              className="w-full px-3 py-2 border border-gray-300 rounded-lg mb-3"
              value={newCampaign.campaign_type}
              onChange={e => setNewCampaign({ ...newCampaign, campaign_type: e.target.value })}
            >
              <option value="whatsapp">WhatsApp</option>
              <option value="promotional">Promotional</option>
              <option value="utility">Utility</option>
            </select>
            <input
              type="datetime-local"
              className="w-full px-3 py-2 border border-gray-300 rounded-lg mb-4"
              value={newCampaign.scheduled_at}
              onChange={e => setNewCampaign({ ...newCampaign, scheduled_at: e.target.value })}
            />
            <div className="flex justify-end gap-2">
              <button onClick={() => setShowCreate(false)} className="px-4 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded-lg">Cancel</button>
              <button onClick={handleCreate} className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">Create</button>
            </div>
          </div>
        </div>
      )}

      {/* AI Suggestions Modal */}
      {showAI && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-lg max-h-[80vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">AI Campaign Suggestions</h3>
              <button onClick={() => setShowAI(false)}><X className="w-5 h-5" /></button>
            </div>
            {aiContent && (
              <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-4">
                <h4 className="font-medium text-green-800 mb-2">Generated Content</h4>
                <p className="text-sm text-green-700">{aiContent.generatedContent?.body}</p>
              </div>
            )}
            {aiSuggestions.map((s, i) => (
              <div key={i} className="border border-gray-200 rounded-lg p-4 mb-3">
                <h4 className="font-medium">{s.title}</h4>
                <p className="text-sm text-gray-500 mt-1">{s.description}</p>
                <div className="flex items-center gap-3 mt-2 text-xs text-gray-400">
                  <span>{s.category}</span>
                  <span>~{s.estimated_contacts} contacts</span>
                </div>
                <button
                  onClick={() => handleAIGenerate(s)}
                  className="mt-3 px-3 py-1.5 text-sm bg-purple-600 text-white rounded-lg hover:bg-purple-700"
                >
                  Generate Content
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Templates & Flows Tab
// ---------------------------------------------------------------------------

function TemplatesTab({ brandId }) {
  const [flows, setFlows] = useState([]);
  const [templates, setTemplates] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newFlow, setNewFlow] = useState({ name: '', body: '', category: 'general', industry_id: '', group_id: '' });

  const loadAll = () => {
    Promise.all([listFlows(brandId), listTemplates(brandId)]).then(([f, t]) => {
      setFlows(f); setTemplates(t);
    }).finally(() => setLoading(false));
  };

  useEffect(() => { loadAll(); }, [brandId]);

  const handleCreate = () => {
    createFlow(brandId, newFlow).then(() => {
      setShowCreate(false);
      setNewFlow({ name: '', body: '', category: 'general', industry_id: '', group_id: '' });
      loadAll();
    });
  };

  if (loading) return <div className="flex justify-center py-12"><div className="animate-spin w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full" /></div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">Templates & Flows</h2>
        <button onClick={() => setShowCreate(true)} className="flex items-center gap-2 px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">
          <Plus className="w-4 h-4" /> New Flow
        </button>
      </div>

      <div className="mb-6">
        <h3 className="text-sm font-medium text-gray-500 mb-3">Meta Templates ({templates.length})</h3>
        {templates.length === 0 ? (
          <p className="text-sm text-gray-400">No templates synced yet. Click "Sync Templates" above.</p>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {templates.map((t) => (
              <div key={t.id} className="bg-white border border-gray-200 rounded-lg p-4">
                <div className="flex items-center justify-between">
                  <h4 className="font-medium text-sm">{t.name}</h4>
                  <span className={`px-2 py-0.5 text-xs rounded-full ${t.status === 'APPROVED' ? 'bg-green-100 text-green-700' : 'bg-yellow-100 text-yellow-700'}`}>
                    {t.status}
                  </span>
                </div>
                <p className="text-xs text-gray-500 mt-1">{t.category} · {t.language}</p>
                <p className="text-xs text-gray-400 mt-2 line-clamp-2">{t.body}</p>
              </div>
            ))}
          </div>
        )}
      </div>

      <div>
        <h3 className="text-sm font-medium text-gray-500 mb-3">Custom Flows ({flows.length})</h3>
        {flows.length === 0 ? (
          <p className="text-sm text-gray-400">No custom flows yet.</p>
        ) : (
          <div className="grid gap-3">
            {flows.map((f) => (
              <div key={f.id} className="bg-white border border-gray-200 rounded-lg p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium">{f.name}</h4>
                    <p className="text-xs text-gray-500 mt-1">{f.category} · {f.flow_type} · {f.language}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-0.5 text-xs rounded-full ${f.is_active ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'}`}>
                      {f.status}
                    </span>
                    <button onClick={() => duplicateFlow(f.id, `${f.name} (copy)`).then(loadAll)} className="p-1.5 text-gray-400 hover:bg-gray-50 rounded" title="Duplicate">
                      <Copy className="w-4 h-4" />
                    </button>
                    <button onClick={() => deleteFlow(f.id).then(loadAll)} className="p-1.5 text-gray-400 hover:bg-gray-50 rounded" title="Delete">
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <p className="text-sm text-gray-600 mt-2 line-clamp-2">{f.body}</p>
              </div>
            ))}
          </div>
        )}
      </div>

      {showCreate && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">New Flow</h3>
            <input className="w-full px-3 py-2 border border-gray-300 rounded-lg mb-3" placeholder="Flow name" value={newFlow.name} onChange={e => setNewFlow({ ...newFlow, name: e.target.value })} />
            <textarea className="w-full px-3 py-2 border border-gray-300 rounded-lg mb-3" placeholder="Message body" rows={4} value={newFlow.body} onChange={e => setNewFlow({ ...newFlow, body: e.target.value })} />
            <select className="w-full px-3 py-2 border border-gray-300 rounded-lg mb-4" value={newFlow.category} onChange={e => setNewFlow({ ...newFlow, category: e.target.value })}>
              <option value="general">General</option>
              <option value="promotional">Promotional</option>
              <option value="utility">Utility</option>
              <option value="authentication">Authentication</option>
            </select>
            <div className="flex justify-end gap-2">
              <button onClick={() => setShowCreate(false)} className="px-4 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded-lg">Cancel</button>
              <button onClick={handleCreate} className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">Create</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Flow Editor Tab
// ---------------------------------------------------------------------------

function FlowEditorTab({ brandId }) {
  const [flows, setFlows] = useState([]);
  const [selectedFlow, setSelectedFlow] = useState(null);
  const [steps, setSteps] = useState([]);
  const [newStep, setNewStep] = useState({ step_number: 1, action_type: 'text', condition: '' });

  useEffect(() => { listFlows(brandId).then(setFlows); }, [brandId]);

  const loadSteps = (flowId) => {
    listFlowSteps(flowId).then(setSteps);
  };

  const handleAddStep = () => {
    createFlowStep(selectedFlow.id, newStep).then(() => {
      setNewStep({ step_number: steps.length + 2, action_type: 'text', condition: '' });
      loadSteps(selectedFlow.id);
    });
  };

  return (
    <div>
      <h2 className="text-lg font-semibold text-gray-900 mb-6">Flow Editor</h2>
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1">
          <h3 className="text-sm font-medium text-gray-500 mb-3">Select Flow</h3>
          {flows.map((f) => (
            <button
              key={f.id}
              onClick={() => { setSelectedFlow(f); loadSteps(f.id); }}
              className={`w-full text-left p-3 rounded-lg mb-2 border ${selectedFlow?.id === f.id ? 'border-blue-600 bg-blue-50' : 'border-gray-200 hover:border-gray-300'}`}
            >
              <p className="font-medium text-sm">{f.name}</p>
              <p className="text-xs text-gray-400">{f.category}</p>
            </button>
          ))}
        </div>
        <div className="lg:col-span-2">
          {selectedFlow ? (
            <div>
              <div className="bg-white border border-gray-200 rounded-xl p-5 mb-4">
                <h3 className="font-semibold">{selectedFlow.name}</h3>
                <p className="text-sm text-gray-500 mt-1">{selectedFlow.body}</p>
              </div>
              <h4 className="text-sm font-medium text-gray-500 mb-3">Steps ({steps.length})</h4>
              {steps.map((s) => (
                <div key={s.id} className="bg-white border border-gray-200 rounded-lg p-4 mb-3 flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="w-8 h-8 bg-blue-100 text-blue-700 rounded-full flex items-center justify-center text-sm font-medium">{s.step_number}</span>
                    <div>
                      <p className="text-sm font-medium">{s.action_type}</p>
                      {s.condition && <p className="text-xs text-gray-400">If: {s.condition}</p>}
                    </div>
                  </div>
                  <button onClick={() => deleteFlowStep(s.id).then(() => loadSteps(selectedFlow.id))} className="p-1.5 text-gray-400 hover:bg-gray-50 rounded">
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              ))}
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 mt-4">
                <h4 className="text-sm font-medium mb-3">Add Step</h4>
                <div className="grid grid-cols-2 gap-3">
                  <input className="px-3 py-2 border border-gray-300 rounded-lg text-sm" placeholder="Step number" type="number" value={newStep.step_number} onChange={e => setNewStep({ ...newStep, step_number: parseInt(e.target.value) })} />
                  <select className="px-3 py-2 border border-gray-300 rounded-lg text-sm" value={newStep.action_type} onChange={e => setNewStep({ ...newStep, action_type: e.target.value })}>
                    <option value="text">Text</option>
                    <option value="buttons">Buttons</option>
                    <option value="list">List</option>
                    <option value="media">Media</option>
                    <option value="template">Template</option>
                    <option value="handoff">Handoff</option>
                  </select>
                  <input className="col-span-2 px-3 py-2 border border-gray-300 rounded-lg text-sm" placeholder="Condition (optional)" value={newStep.condition} onChange={e => setNewStep({ ...newStep, condition: e.target.value })} />
                </div>
                <button onClick={handleAddStep} className="mt-3 px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">Add Step</button>
              </div>
            </div>
          ) : (
            <div className="text-center py-12 bg-white rounded-xl border border-gray-200">
              <Edit3 className="w-12 h-12 text-gray-300 mx-auto mb-4" />
              <p className="text-gray-500">Select a flow to edit</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Industries Tab
// ---------------------------------------------------------------------------

function IndustriesTab({ brandId }) {
  const [industries, setIndustries] = useState([]);
  const [groups, setGroups] = useState([]);
  const [newIndustry, setNewIndustry] = useState({ name: '', description: '' });

  useEffect(() => {
    listIndustries().then(setIndustries);
    listTemplateGroups(brandId).then(setGroups);
  }, [brandId]);

  return (
    <div>
      <h2 className="text-lg font-semibold text-gray-900 mb-6">Industries & Groups</h2>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div>
          <h3 className="text-sm font-medium text-gray-500 mb-3">Industries</h3>
          <div className="flex gap-2 mb-3">
            <input className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm" placeholder="Industry name" value={newIndustry.name} onChange={e => setNewIndustry({ ...newIndustry, name: e.target.value })} />
            <button onClick={() => { createIndustry(newIndustry.name, newIndustry.description).then(() => { setNewIndustry({ name: '', description: '' }); listIndustries().then(setIndustries); }); }} className="px-3 py-2 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700">Add</button>
          </div>
          {industries.map((ind) => (
            <div key={ind.id} className="bg-white border border-gray-200 rounded-lg p-3 mb-2 flex items-center justify-between">
              <div>
                <p className="font-medium text-sm">{ind.name}</p>
                <p className="text-xs text-gray-400">{ind.description}</p>
              </div>
              {ind.icon && <span className="text-lg">{ind.icon}</span>}
            </div>
          ))}
        </div>
        <div>
          <h3 className="text-sm font-medium text-gray-500 mb-3">Template Groups</h3>
          {groups.length === 0 ? (
            <p className="text-sm text-gray-400">No template groups yet.</p>
          ) : (
            groups.map((g) => (
              <div key={g.id} className="bg-white border border-gray-200 rounded-lg p-3 mb-2">
                <p className="font-medium text-sm">{g.name}</p>
                <p className="text-xs text-gray-400">{g.description}</p>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Message Limits Tab
// ---------------------------------------------------------------------------

function LimitsTab({ brandId }) {
  const [limits, setLimits] = useState(null);
  const [loading, setLoading] = useState(true);

  const loadLimits = () => {
    getMessageLimits(brandId).then(setLimits).finally(() => setLoading(false));
  };

  useEffect(() => { loadLimits(); }, [brandId]);

  if (loading) return <div className="flex justify-center py-12"><div className="animate-spin w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full" /></div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">Message Limits</h2>
        <button onClick={() => syncMessageLimits(brandId).then(loadLimits)} className="flex items-center gap-2 px-3 py-2 text-sm bg-green-50 text-green-700 rounded-lg hover:bg-green-100">
          <RefreshCw className="w-4 h-4" /> Sync Limits
        </button>
      </div>

      {limits ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Quality Rating</p>
            <p className="text-2xl font-bold mt-1">{limits.qualityRating || 'Unknown'}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Messaging Tier</p>
            <p className="text-2xl font-bold mt-1">{limits.messagingTier || 'Unknown'}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Total Limit</p>
            <p className="text-2xl font-bold mt-1">{limits.totalLimit || 0}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Remaining</p>
            <p className={`text-2xl font-bold mt-1 ${limits.remaining > 0 ? 'text-green-600' : 'text-red-600'}`}>{limits.remaining || 0}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Utility Used</p>
            <p className="text-xl font-semibold mt-1">{limits.utilityUsed || 0}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Marketing Used</p>
            <p className="text-xl font-semibold mt-1">{limits.marketingUsed || 0}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">Auth Used</p>
            <p className="text-xl font-semibold mt-1">{limits.authenticationUsed || 0}</p>
          </div>
          <div className="bg-white border border-gray-200 rounded-xl p-5">
            <p className="text-sm text-gray-500">CS Used</p>
            <p className="text-xl font-semibold mt-1">{limits.customerServiceUsed || 0}</p>
          </div>
        </div>
      ) : (
        <div className="text-center py-12 bg-white rounded-xl border border-gray-200">
          <BarChart3 className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">No limit data. Sync from Meta to see your limits.</p>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Contact Selection Tab
// ---------------------------------------------------------------------------

function ContactsTab({ brandId }) {
  const [contacts, setContacts] = useState([]);
  const [selected, setSelected] = useState(new Set());
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

  useEffect(() => {
    listContacts(brandId, 200).then(data => {
      setContacts(data.contacts || []);
      setLoading(false);
    });
  }, [brandId]);

  const toggleContact = (id) => {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const filtered = contacts.filter(c =>
    !search || c.name?.toLowerCase().includes(search.toLowerCase()) || c.value?.toLowerCase().includes(search.toLowerCase())
  );

  if (loading) return <div className="flex justify-center py-12"><div className="animate-spin w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full" /></div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">Contact Selection</h2>
        <p className="text-sm text-gray-500">{selected.size} selected</p>
      </div>
      <div className="flex gap-2 mb-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input className="w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg text-sm" placeholder="Search contacts..." value={search} onChange={e => setSearch(e.target.value)} />
        </div>
        {selected.size > 0 && (
          <button className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700">
            Add {selected.size} to Campaign
          </button>
        )}
      </div>
      <div className="bg-white border border-gray-200 rounded-xl overflow-hidden">
        {filtered.map((c) => (
          <div key={c.id} className={`flex items-center gap-3 p-3 border-b border-gray-100 cursor-pointer hover:bg-gray-50 ${selected.has(c.id) ? 'bg-blue-50' : ''}`} onClick={() => toggleContact(c.id)}>
            <div className={`w-5 h-5 rounded border-2 flex items-center justify-center ${selected.has(c.id) ? 'bg-blue-600 border-blue-600' : 'border-gray-300'}`}>
              {selected.has(c.id) && <CheckCircle className="w-3 h-3 text-white" />}
            </div>
            <div className="w-8 h-8 bg-gray-200 rounded-full flex items-center justify-center text-sm font-medium text-gray-600">
              {c.name?.[0]?.toUpperCase() || '?'}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium truncate">{c.name || 'Unknown'}</p>
              <p className="text-xs text-gray-400 truncate">{c.value}</p>
            </div>
            {c.channel && <span className="text-xs px-2 py-0.5 bg-gray-100 text-gray-600 rounded-full">{c.channel}</span>}
          </div>
        ))}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Analytics Tab
// ---------------------------------------------------------------------------

function AnalyticsTab({ brandId }) {
  const [campaigns, setCampaigns] = useState([]);
  const [selectedCampaign, setSelectedCampaign] = useState(null);
  const [analytics, setAnalytics] = useState(null);

  useEffect(() => { listCampaigns(brandId).then(setCampaigns); }, [brandId]);

  const loadAnalytics = (id) => {
    setSelectedCampaign(id);
    getCampaignAnalytics(id).then(setAnalytics);
  };

  return (
    <div>
      <h2 className="text-lg font-semibold text-gray-900 mb-6">Campaign Analytics</h2>
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1">
          <h3 className="text-sm font-medium text-gray-500 mb-3">Select Campaign</h3>
          {campaigns.filter(c => c.status === 'completed').map((c) => (
            <button key={c.id} onClick={() => loadAnalytics(c.id)} className={`w-full text-left p-3 rounded-lg mb-2 border ${selectedCampaign === c.id ? 'border-blue-600 bg-blue-50' : 'border-gray-200 hover:border-gray-300'}`}>
              <p className="font-medium text-sm">{c.name}</p>
              <p className="text-xs text-gray-400">{c.total_contacts} contacts</p>
            </button>
          ))}
          {campaigns.filter(c => c.status === 'completed').length === 0 && (
            <p className="text-sm text-gray-400">No completed campaigns yet.</p>
          )}
        </div>
        <div className="lg:col-span-2">
          {analytics ? (
            <div>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
                <div className="bg-white border border-gray-200 rounded-xl p-4">
                  <p className="text-xs text-gray-500">Total</p>
                  <p className="text-xl font-bold mt-1">{analytics.totalContacts}</p>
                </div>
                <div className="bg-white border border-gray-200 rounded-xl p-4">
                  <p className="text-xs text-gray-500">Sent</p>
                  <p className="text-xl font-bold mt-1 text-green-600">{analytics.sentCount}</p>
                </div>
                <div className="bg-white border border-gray-200 rounded-xl p-4">
                  <p className="text-xs text-gray-500">Delivered</p>
                  <p className="text-xl font-bold mt-1 text-blue-600">{analytics.deliveredCount}</p>
                </div>
                <div className="bg-white border border-gray-200 rounded-xl p-4">
                  <p className="text-xs text-gray-500">Failed</p>
                  <p className="text-xl font-bold mt-1 text-red-600">{analytics.failedCount}</p>
                </div>
              </div>
              <div className="bg-white border border-gray-200 rounded-xl p-5">
                <h4 className="font-medium mb-4">Rates</h4>
                <div className="space-y-3">
                  {[
                    { label: 'Send Rate', value: analytics.sendRate, color: 'bg-green-500' },
                    { label: 'Delivery Rate', value: analytics.deliveryRate, color: 'bg-blue-500' },
                    { label: 'Read Rate', value: analytics.readRate, color: 'bg-purple-500' },
                    { label: 'Failure Rate', value: analytics.failureRate, color: 'bg-red-500' },
                  ].map((r) => (
                    <div key={r.label}>
                      <div className="flex items-center justify-between text-sm mb-1">
                        <span className="text-gray-600">{r.label}</span>
                        <span className="font-medium">{r.value?.toFixed(1)}%</span>
                      </div>
                      <div className="w-full bg-gray-100 rounded-full h-2">
                        <div className={`${r.color} h-2 rounded-full`} style={{ width: `${Math.min(r.value || 0, 100)}%` }} />
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center py-12 bg-white rounded-xl border border-gray-200">
              <BarChart3 className="w-12 h-12 text-gray-300 mx-auto mb-4" />
              <p className="text-gray-500">Select a completed campaign to view analytics</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
