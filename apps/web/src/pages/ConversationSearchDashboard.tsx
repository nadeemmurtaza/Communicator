import { useState, useEffect, useCallback } from 'react'
import {
  getSearchSummary,
  searchConversations,
  listSavedSearches,
  createSavedSearch,
  deleteSavedSearch,
  pinSavedSearch,
  useSavedSearch,
  getSearchHistory,
  clearSearchHistory,
  getSearchSettings,
  updateSearchSettings,
  type SearchSummary,
  type SearchResponse,
  type SearchResult,
  type SavedSearch,
  type SearchHistoryEntry,
  type SearchSettings,
} from '../lib/api'
import { Search, Save, Trash2, Pin, Clock, BarChart3, Settings, X, Sparkles } from '../components/icons'

function ConversationSearchDashboard() {
  const [tab, setTab] = useState<'search' | 'saved' | 'history' | 'settings'>('search')
  const [summary, setSummary] = useState<SearchSummary | null>(null)
  const [loading, setLoading] = useState(false)
  const [searchResults, setSearchResults] = useState<SearchResponse | null>(null)
  const [query, setQuery] = useState('')
  const [searchType, setSearchType] = useState('all')
  const [savedSearches, setSavedSearches] = useState<SavedSearch[]>([])
  const [history, setHistory] = useState<SearchHistoryEntry[]>([])
  const [settings, setSettings] = useState<SearchSettings | null>(null)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newName, setNewName] = useState('')
  const [newQuery, setNewQuery] = useState('')

  const loadSummary = useCallback(async () => {
    try {
      const data = await getSearchSummary()
      setSummary(data)
    } catch {}
  }, [])

  const loadSavedSearches = useCallback(async () => {
    try {
      const data = await listSavedSearches()
      setSavedSearches(data)
    } catch {}
  }, [])

  const loadHistory = useCallback(async () => {
    try {
      const data = await getSearchHistory(50)
      setHistory(data)
    } catch {}
  }, [])

  const loadSettings = useCallback(async () => {
    try {
      const data = await getSearchSettings()
      setSettings(data)
    } catch {}
  }, [])

  useEffect(() => {
    loadSummary()
    loadSavedSearches()
    loadHistory()
    loadSettings()
  }, [loadSummary, loadSavedSearches, loadHistory, loadSettings])

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!query.trim()) return
    setLoading(true)
    try {
      const data = await searchConversations(query, searchType, 50)
      setSearchResults(data)
    } catch {
      setSearchResults(null)
    } finally {
      setLoading(false)
    }
  }

  const handleSaveSearch = async () => {
    if (!newName.trim() || !newQuery.trim()) return
    try {
      await createSavedSearch({ name: newName, query: newQuery, search_type: searchType })
      setShowCreateModal(false)
      setNewName('')
      setNewQuery('')
      loadSavedSearches()
      loadSummary()
    } catch {}
  }

  const handleDeleteSaved = async (id: string) => {
    try {
      await deleteSavedSearch(id)
      loadSavedSearches()
      loadSummary()
    } catch {}
  }

  const handlePinSaved = async (id: string, pinned: boolean) => {
    try {
      await pinSavedSearch(id, !pinned)
      loadSavedSearches()
    } catch {}
  }

  const handleUseSaved = async (id: string) => {
    try {
      const data = await useSavedSearch(id)
      setSearchResults(data)
      setTab('search')
    } catch {}
  }

  const handleClearHistory = async () => {
    try {
      await clearSearchHistory()
      loadHistory()
      loadSummary()
    } catch {}
  }

  const handleUpdateSetting = async (key: string, value: any) => {
    if (!settings) return
    try {
      const updated = await updateSearchSettings({ ...settings, [key]: value })
      setSettings(updated)
    } catch {}
  }

  const renderSearchResult = (result: SearchResult, type: string) => {
    const typeColors: Record<string, string> = {
      conversation: 'bg-blue-100 text-blue-700',
      message: 'bg-green-100 text-green-700',
      contact: 'bg-purple-100 text-purple-700',
    }
    return (
      <div key={result.id} className="p-4 border border-slate-200 rounded-lg hover:bg-slate-50">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className={`text-xs px-2 py-0.5 rounded-full ${typeColors[type] || 'bg-slate-100 text-slate-600'}`}>
                {type}
              </span>
              <h4 className="font-medium text-slate-900 truncate">{result.title}</h4>
            </div>
            <p className="text-sm text-slate-600 line-clamp-2">{result.snippet}</p>
            <div className="flex items-center gap-3 mt-2 text-xs text-slate-400">
              {result.matchedFields.map(f => (
                <span key={f} className="bg-slate-100 px-1.5 py-0.5 rounded">{f}</span>
              ))}
              {result.metadata.created_at && (
                <span>{new Date(result.metadata.created_at).toLocaleDateString()}</span>
              )}
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-slate-900">Conversation Search</h1>
        <p className="text-slate-500 mt-1">Search across conversations, messages, and contacts</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-slate-100 p-1 rounded-lg w-fit">
        {([
          { key: 'search', label: 'Search', icon: Search },
          { key: 'saved', label: 'Saved', icon: Save },
          { key: 'history', label: 'History', icon: Clock },
          { key: 'settings', label: 'Settings', icon: Settings },
        ] as const).map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => setTab(key)}
            className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              tab === key ? 'bg-white text-slate-900 shadow-sm' : 'text-slate-500 hover:text-slate-700'
            }`}
          >
            <Icon className="w-4 h-4" />
            {label}
          </button>
        ))}
      </div>

      {/* Search Tab */}
      {tab === 'search' && (
        <div className="space-y-6">
          {/* Search Bar */}
          <form onSubmit={handleSearch} className="flex gap-3">
            <input
              type="text"
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder="Search conversations, messages, contacts..."
              className="flex-1 px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <select
              value={searchType}
              onChange={e => setSearchType(e.target.value)}
              className="px-3 py-3 border border-slate-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All</option>
              <option value="conversation">Conversations</option>
              <option value="message">Messages</option>
              <option value="contact">Contacts</option>
            </select>
            <button
              type="submit"
              disabled={loading || !query.trim()}
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 flex items-center gap-2"
            >
              <Search className="w-4 h-4" />
              Search
            </button>
          </form>

          {/* Summary Cards */}
          {summary && (
            <div className="grid grid-cols-4 gap-4">
              <div className="bg-white p-4 rounded-lg border border-slate-200">
                <div className="text-sm text-slate-500">Searches Today</div>
                <div className="text-2xl font-bold text-slate-900">{summary.totalSearchesToday}</div>
              </div>
              <div className="bg-white p-4 rounded-lg border border-slate-200">
                <div className="text-sm text-slate-500">Searches (7d)</div>
                <div className="text-2xl font-bold text-slate-900">{summary.totalSearches7d}</div>
              </div>
              <div className="bg-white p-4 rounded-lg border border-slate-200">
                <div className="text-sm text-slate-500">Avg Results</div>
                <div className="text-2xl font-bold text-slate-900">{summary.avgResultsPerSearch.toFixed(1)}</div>
              </div>
              <div className="bg-white p-4 rounded-lg border border-slate-200">
                <div className="text-sm text-slate-500">Zero Results %</div>
                <div className="text-2xl font-bold text-slate-900">{summary.zeroResultRate.toFixed(1)}%</div>
              </div>
            </div>
          )}

          {/* Results */}
          {searchResults && (
            <div>
              <div className="flex items-center justify-between mb-3">
                <h3 className="font-medium text-slate-700">
                  {searchResults.totalResults} results for "{searchResults.query}" ({searchResults.durationMs}ms)
                </h3>
              </div>
              <div className="space-y-3">
                {searchResults.conversations.length > 0 && (
                  <div>
                    <h4 className="text-sm font-medium text-blue-600 mb-2">Conversations ({searchResults.conversations.length})</h4>
                    <div className="space-y-2">
                      {searchResults.conversations.map(r => renderSearchResult(r, 'conversation'))}
                    </div>
                  </div>
                )}
                {searchResults.messages.length > 0 && (
                  <div>
                    <h4 className="text-sm font-medium text-green-600 mb-2">Messages ({searchResults.messages.length})</h4>
                    <div className="space-y-2">
                      {searchResults.messages.map(r => renderSearchResult(r, 'message'))}
                    </div>
                  </div>
                )}
                {searchResults.contacts.length > 0 && (
                  <div>
                    <h4 className="text-sm font-medium text-purple-600 mb-2">Contacts ({searchResults.contacts.length})</h4>
                    <div className="space-y-2">
                      {searchResults.contacts.map(r => renderSearchResult(r, 'contact'))}
                    </div>
                  </div>
                )}
                {searchResults.totalResults === 0 && (
                  <div className="text-center py-12 text-slate-400">
                    <Search className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p>No results found for "{searchResults.query}"</p>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Top Queries */}
          {summary && summary.topQueries.length > 0 && (
            <div className="bg-white p-4 rounded-lg border border-slate-200">
              <h3 className="font-medium text-slate-700 mb-3">Top Queries (7 days)</h3>
              <div className="space-y-2">
                {summary.topQueries.map((q, i) => (
                  <div key={i} className="flex items-center justify-between">
                    <span className="text-sm text-slate-600">{q.query}</span>
                    <span className="text-sm font-medium text-slate-900">{q.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Saved Searches Tab */}
      {tab === 'saved' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="font-medium text-slate-700">Saved Searches ({savedSearches.length})</h3>
            <button
              onClick={() => setShowCreateModal(true)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm flex items-center gap-2"
            >
              <Save className="w-4 h-4" />
              Save Current Search
            </button>
          </div>

          {savedSearches.length === 0 ? (
            <div className="text-center py-12 text-slate-400">
              <Save className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>No saved searches yet</p>
            </div>
          ) : (
            <div className="space-y-2">
              {savedSearches.map(s => (
                <div key={s.id} className="flex items-center justify-between p-4 bg-white border border-slate-200 rounded-lg">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <h4 className="font-medium text-slate-900">{s.name}</h4>
                      {s.isPinned && <Pin className="w-3.5 h-3.5 text-amber-500" />}
                    </div>
                    <p className="text-sm text-slate-500 truncate">{s.query}</p>
                    <div className="flex items-center gap-3 mt-1 text-xs text-slate-400">
                      <span>{s.searchType}</span>
                      <span>Used {s.usageCount} times</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 ml-3">
                    <button
                      onClick={() => handleUseSaved(s.id)}
                      className="p-2 text-slate-400 hover:text-blue-600"
                      title="Run search"
                    >
                      <Search className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handlePinSaved(s.id, s.isPinned)}
                      className={`p-2 ${s.isPinned ? 'text-amber-500' : 'text-slate-400'} hover:text-amber-500`}
                      title={s.isPinned ? 'Unpin' : 'Pin'}
                    >
                      <Pin className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handleDeleteSaved(s.id)}
                      className="p-2 text-slate-400 hover:text-red-600"
                      title="Delete"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Create Modal */}
          {showCreateModal && (
            <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
              <div className="bg-white rounded-xl p-6 w-full max-w-md">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="font-semibold text-slate-900">Save Search</h3>
                  <button onClick={() => setShowCreateModal(false)} className="text-slate-400 hover:text-slate-600">
                    <X className="w-5 h-5" />
                  </button>
                </div>
                <div className="space-y-3">
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">Name</label>
                    <input
                      type="text"
                      value={newName}
                      onChange={e => setNewName(e.target.value)}
                      placeholder="e.g. Refund inquiries"
                      className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">Query</label>
                    <input
                      type="text"
                      value={newQuery}
                      onChange={e => setNewQuery(e.target.value)}
                      placeholder="e.g. refund"
                      className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <div className="flex gap-3 justify-end">
                    <button
                      onClick={() => setShowCreateModal(false)}
                      className="px-4 py-2 text-slate-600 hover:text-slate-800"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleSaveSearch}
                      disabled={!newName.trim() || !newQuery.trim()}
                      className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                    >
                      Save
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* History Tab */}
      {tab === 'history' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="font-medium text-slate-700">Search History</h3>
            <button
              onClick={handleClearHistory}
              className="px-3 py-1.5 text-sm text-red-600 hover:text-red-700 border border-red-200 rounded-lg"
            >
              Clear History
            </button>
          </div>

          {history.length === 0 ? (
            <div className="text-center py-12 text-slate-400">
              <Clock className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>No search history yet</p>
            </div>
          ) : (
            <div className="space-y-2">
              {history.map(h => (
                <div key={h.id} className="flex items-center justify-between p-3 bg-white border border-slate-200 rounded-lg">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-slate-900">{h.query}</span>
                      <span className="text-xs px-1.5 py-0.5 bg-slate-100 text-slate-500 rounded">{h.searchType}</span>
                    </div>
                    <div className="flex items-center gap-3 mt-1 text-xs text-slate-400">
                      <span>{h.resultCount} results</span>
                      <span>{h.durationMs}ms</span>
                      <span>{new Date(h.performedAt).toLocaleString()}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {tab === 'settings' && settings && (
        <div className="space-y-4">
          <h3 className="font-medium text-slate-700">Search Settings</h3>
          <div className="bg-white border border-slate-200 rounded-lg divide-y">
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Max Results</div>
                <div className="text-sm text-slate-500">Maximum results per search</div>
              </div>
              <input
                type="number"
                value={settings.maxResults}
                onChange={e => handleUpdateSetting('maxResults', parseInt(e.target.value))}
                className="w-20 px-3 py-1.5 border border-slate-200 rounded-lg text-right"
              />
            </div>
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Search Message Content</div>
                <div className="text-sm text-slate-500">Include message body in search</div>
              </div>
              <button
                onClick={() => handleUpdateSetting('searchMessageContent', !settings.searchMessageContent)}
                className={`w-11 h-6 rounded-full transition-colors ${settings.searchMessageContent ? 'bg-blue-600' : 'bg-slate-200'}`}
              >
                <div className={`w-5 h-5 bg-white rounded-full shadow-sm transform transition-transform ${settings.searchMessageContent ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Search Contact Details</div>
                <div className="text-sm text-slate-500">Include contact info in search</div>
              </div>
              <button
                onClick={() => handleUpdateSetting('searchContactDetails', !settings.searchContactDetails)}
                className={`w-11 h-6 rounded-full transition-colors ${settings.searchContactDetails ? 'bg-blue-600' : 'bg-slate-200'}`}
              >
                <div className={`w-5 h-5 bg-white rounded-full shadow-sm transform transition-transform ${settings.searchContactDetails ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Track Search History</div>
                <div className="text-sm text-slate-500">Record searches for analytics</div>
              </div>
              <button
                onClick={() => handleUpdateSetting('trackSearchHistory', !settings.trackSearchHistory)}
                className={`w-11 h-6 rounded-full transition-colors ${settings.trackSearchHistory ? 'bg-blue-600' : 'bg-slate-200'}`}
              >
                <div className={`w-5 h-5 bg-white rounded-full shadow-sm transform transition-transform ${settings.trackSearchHistory ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Auto Suggest</div>
                <div className="text-sm text-slate-500">Show search suggestions</div>
              </div>
              <button
                onClick={() => handleUpdateSetting('autoSuggest', !settings.autoSuggest)}
                className={`w-11 h-6 rounded-full transition-colors ${settings.autoSuggest ? 'bg-blue-600' : 'bg-slate-200'}`}
              >
                <div className={`w-5 h-5 bg-white rounded-full shadow-sm transform transition-transform ${settings.autoSuggest ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Minimum Query Length</div>
                <div className="text-sm text-slate-500">Minimum characters to trigger search</div>
              </div>
              <input
                type="number"
                value={settings.minQueryLength}
                onChange={e => handleUpdateSetting('minQueryLength', parseInt(e.target.value))}
                className="w-20 px-3 py-1.5 border border-slate-200 rounded-lg text-right"
              />
            </div>
            <div className="flex items-center justify-between p-4">
              <div>
                <div className="font-medium text-slate-900">Highlight Matches</div>
                <div className="text-sm text-slate-500">Highlight matching text in results</div>
              </div>
              <button
                onClick={() => handleUpdateSetting('highlightMatches', !settings.highlightMatches)}
                className={`w-11 h-6 rounded-full transition-colors ${settings.highlightMatches ? 'bg-blue-600' : 'bg-slate-200'}`}
              >
                <div className={`w-5 h-5 bg-white rounded-full shadow-sm transform transition-transform ${settings.highlightMatches ? 'translate-x-5' : 'translate-x-0.5'}`} />
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default ConversationSearchDashboard
