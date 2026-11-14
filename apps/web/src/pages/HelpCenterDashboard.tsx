import { useEffect, useState } from 'react'
import {
  getHelpCenterSummary,
  listHelpCenterCategories,
  listHelpCenterArticles,
  createHelpCenterArticle,
  updateHelpCenterArticle,
  deleteHelpCenterArticle,
  searchHelpCenterArticles,
  recordArticleFeedback,
  getHelpCenterSettings,
  updateHelpCenterSettings,
  createHelpCenterCategory,
  type HelpCenterSummary as HelpCenterSummaryType,
  type HelpCenterArticle,
  type HelpCenterCategory,
  type HelpCenterSettings,
} from '../lib/api'
import {
  BookOpen,
  FileText,
  Search,
  Eye,
  ThumbsUp,
  ThumbsDown,
  Plus,
  X,
  Check,
  Loader2,
  Settings,
  BarChart3,
  Edit3,
  Trash2,
  Pin,
  Tag,
} from '../components/icons'

export function HelpCenterDashboard() {
  const [summary, setSummary] = useState<HelpCenterSummaryType | null>(null)
  const [categories, setCategories] = useState<HelpCenterCategory[]>([])
  const [articles, setArticles] = useState<HelpCenterArticle[]>([])
  const [settings, setSettings] = useState<HelpCenterSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'overview' | 'articles' | 'categories' | 'settings'>('overview')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [categoryFilter, setCategoryFilter] = useState<string>('')
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<HelpCenterArticle[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const [actionLoading, setActionLoading] = useState<string | null>(null)
  const [showArticleModal, setShowArticleModal] = useState(false)
  const [showCategoryModal, setShowCategoryModal] = useState(false)
  const [editingArticle, setEditingArticle] = useState<HelpCenterArticle | null>(null)
  const [articleForm, setArticleForm] = useState({
    title: '',
    slug: '',
    content: '',
    categoryId: '',
    organizationId: 'org-default',
  })
  const [categoryForm, setCategoryForm] = useState({
    name: '',
    slug: '',
    description: '',
    icon: '',
    displayOrder: 0,
  })

  useEffect(() => {
    loadData()
  }, [activeTab, statusFilter, categoryFilter])

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [summaryRes, categoriesRes, articlesRes, settingsRes] = await Promise.all([
        getHelpCenterSummary(),
        listHelpCenterCategories(),
        listHelpCenterArticles(50, 0, categoryFilter || undefined, statusFilter || undefined),
        getHelpCenterSettings(),
      ])
      setSummary(summaryRes)
      setCategories(categoriesRes.categories ?? [])
      setArticles(articlesRes.articles ?? [])
      setSettings(settingsRes)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load Help Center data')
    } finally {
      setLoading(false)
    }
  }

  const handleSearch = async (q: string) => {
    setSearchQuery(q)
    if (!q.trim()) {
      setSearchResults([])
      return
    }
    setIsSearching(true)
    try {
      const result = await searchHelpCenterArticles(q)
      setSearchResults(result.articles ?? [])
    } catch {
      setSearchResults([])
    } finally {
      setIsSearching(false)
    }
  }

  const handleCreateArticle = async () => {
    if (!articleForm.title || !articleForm.slug || !articleForm.content || !articleForm.categoryId) {
      alert('Title, slug, content, and category are required')
      return
    }
    setActionLoading('create-article')
    try {
      if (editingArticle) {
        await updateHelpCenterArticle(editingArticle.id, articleForm)
      } else {
        await createHelpCenterArticle(articleForm)
      }
      setShowArticleModal(false)
      resetArticleForm()
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save article')
    } finally {
      setActionLoading(null)
    }
  }

  const handleEditArticle = (article: HelpCenterArticle) => {
    setEditingArticle(article)
    setArticleForm({
      title: article.title,
      slug: article.slug,
      content: article.content,
      categoryId: article.categoryId,
      organizationId: article.organizationId,
    })
    setShowArticleModal(true)
  }

  const handleDeleteArticle = async (id: string) => {
    if (!confirm('Are you sure you want to delete this article?')) return
    setActionLoading(id)
    try {
      await deleteHelpCenterArticle(id)
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete article')
    } finally {
      setActionLoading(null)
    }
  }

  const handlePublishArticle = async (id: string, status: string) => {
    setActionLoading(id)
    try {
      await updateHelpCenterArticle(id, { status })
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update article status')
    } finally {
      setActionLoading(null)
    }
  }

  const handlePinArticle = async (id: string, isPinned: boolean) => {
    setActionLoading(id)
    try {
      await updateHelpCenterArticle(id, { is_pinned: !isPinned })
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update pin')
    } finally {
      setActionLoading(null)
    }
  }

  const handleArticleFeedback = async (articleId: string, helpful: boolean) => {
    setActionLoading(articleId)
    try {
      await recordArticleFeedback(articleId, helpful)
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to record feedback')
    } finally {
      setActionLoading(null)
    }
  }

  const handleCreateCategory = async () => {
    if (!categoryForm.name || !categoryForm.slug) {
      alert('Name and slug are required')
      return
    }
    setActionLoading('create-category')
    try {
      await createHelpCenterCategory({
        organizationId: 'org-default',
        name: categoryForm.name,
        slug: categoryForm.slug,
        description: categoryForm.description || undefined,
        icon: categoryForm.icon || undefined,
        displayOrder: categoryForm.displayOrder,
      })
      setShowCategoryModal(false)
      setCategoryForm({ name: '', slug: '', description: '', icon: '', displayOrder: 0 })
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create category')
    } finally {
      setActionLoading(null)
    }
  }

  const handleUpdateSettings = async () => {
    if (!settings) return
    setActionLoading('settings')
    try {
      const updated = await updateHelpCenterSettings(settings)
      setSettings(updated)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update settings')
    } finally {
      setActionLoading(null)
    }
  }

  const resetArticleForm = () => {
    setEditingArticle(null)
    setArticleForm({ title: '', slug: '', content: '', categoryId: '', organizationId: 'org-default' })
  }

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <Loader2 className="animate-spin" style={{ width: 32, height: 32, color: '#6366f1' }} />
      </div>
    )
  }

  return (
    <div style={{ padding: 24 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 24 }}>
        <div>
          <h1 style={{ margin: 0, fontSize: 24, fontWeight: 700 }}>Help Center</h1>
          <p style={{ margin: '4px 0 0', color: '#64748b' }}>
            Manage your public-facing self-service knowledge base
          </p>
        </div>
      </div>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 24, borderBottom: '1px solid #e2e8f0', paddingBottom: 12 }}>
        {(['overview', 'articles', 'categories', 'settings'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: '8px 16px',
              border: 'none',
              borderRadius: 8,
              cursor: 'pointer',
              fontWeight: 600,
              fontSize: 14,
              backgroundColor: activeTab === tab ? '#6366f1' : 'transparent',
              color: activeTab === tab ? '#fff' : '#64748b',
              textTransform: 'capitalize',
            }}
          >
            {tab}
          </button>
        ))}
      </div>

      {error && (
        <div style={{ padding: '12px 16px', backgroundColor: '#fef2f2', border: '1px solid #fecaca', borderRadius: 8, marginBottom: 16, color: '#dc2626' }}>
          {error}
        </div>
      )}

      {/* Overview Tab */}
      {activeTab === 'overview' && summary && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 16, marginBottom: 24 }}>
            <StatCard icon={<BookOpen />} label="Total Articles" value={summary.totalArticles} />
            <StatCard icon={<FileText />} label="Published" value={summary.publishedArticles} />
            <StatCard icon={<Edit3 />} label="Drafts" value={summary.draftArticles} />
            <StatCard icon={<Tag />} label="Categories" value={summary.totalCategories} />
            <StatCard icon={<Eye />} label="Views (7d)" value={summary.totalViews7d} />
            <StatCard icon={<Search />} label="Searches (7d)" value={summary.totalSearches7d} />
            <StatCard icon={<ThumbsUp />} label="Helpful Rate" value={`${summary.avgHelpfulRate.toFixed(1)}%`} />
          </div>

          {/* Search */}
          <div style={{ marginBottom: 24 }}>
            <h3 style={{ margin: '0 0 12px' }}>Quick Search</h3>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                type="text"
                placeholder="Search articles..."
                value={searchQuery}
                onChange={(e) => handleSearch(e.target.value)}
                style={{
                  flex: 1,
                  padding: '10px 14px',
                  border: '1px solid #e2e8f0',
                  borderRadius: 8,
                  fontSize: 14,
                }}
              />
              {isSearching && <Loader2 className="animate-spin" style={{ width: 20, height: 20, color: '#6366f1' }} />}
            </div>
            {searchResults.length > 0 && (
              <div style={{ marginTop: 12 }}>
                {searchResults.map((a) => (
                  <div key={a.id} style={{ padding: '10px 14px', borderBottom: '1px solid #f1f5f9' }}>
                    <div style={{ fontWeight: 600 }}>{a.title}</div>
                    <div style={{ fontSize: 12, color: '#64748b' }}>
                      {a.categoryName} · {a.viewCount} views
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Top Articles */}
          {summary.topArticles.length > 0 && (
            <div>
              <h3 style={{ margin: '0 0 12px' }}>Top Articles</h3>
              {summary.topArticles.map((a) => (
                <div key={a.id} style={{ padding: '12px 14px', borderBottom: '1px solid #f1f5f9', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <div style={{ fontWeight: 600 }}>{a.title}</div>
                    <div style={{ fontSize: 12, color: '#64748b' }}>
                      {a.categoryName} · {a.viewCount} views · {a.helpfulCount}👍 {a.notHelpfulCount}👎
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Popular Searches */}
          {summary.popularSearches.length > 0 && (
            <div style={{ marginTop: 24 }}>
              <h3 style={{ margin: '0 0 12px' }}>Popular Searches</h3>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
                {summary.popularSearches.map((s, i) => (
                  <span key={i} style={{ padding: '6px 12px', backgroundColor: '#f1f5f9', borderRadius: 20, fontSize: 13 }}>
                    {s.query} ({s.count})
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Articles Tab */}
      {activeTab === 'articles' && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
            <div style={{ display: 'flex', gap: 8 }}>
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                style={{ padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 13 }}
              >
                <option value="">All Statuses</option>
                <option value="published">Published</option>
                <option value="draft">Draft</option>
                <option value="archived">Archived</option>
              </select>
              <select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
                style={{ padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 13 }}
              >
                <option value="">All Categories</option>
                {categories.map((c) => (
                  <option key={c.id} value={c.id}>{c.name}</option>
                ))}
              </select>
            </div>
            <button
              onClick={() => { resetArticleForm(); setShowArticleModal(true) }}
              style={{ padding: '8px 16px', backgroundColor: '#6366f1', color: '#fff', border: 'none', borderRadius: 8, cursor: 'pointer', fontWeight: 600, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6 }}
            >
              <Plus style={{ width: 16, height: 16 }} /> New Article
            </button>
          </div>

          {articles.map((article) => (
            <div key={article.id} style={{ padding: '14px 16px', borderBottom: '1px solid #f1f5f9', display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <div style={{ flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  {article.isPinned && <Pin style={{ width: 14, height: 14, color: '#f59e0b' }} />}
                  <span style={{ fontWeight: 600 }}>{article.title}</span>
                  <span style={{ padding: '2px 8px', borderRadius: 12, fontSize: 11, fontWeight: 600, backgroundColor: article.status === 'published' ? '#dcfce7' : article.status === 'draft' ? '#fef3c7' : '#f1f5f9', color: article.status === 'published' ? '#16a34a' : article.status === 'draft' ? '#d97706' : '#64748b' }}>
                    {article.status}
                  </span>
                </div>
                <div style={{ fontSize: 12, color: '#64748b', marginTop: 4 }}>
                  {article.categoryName} · {article.viewCount} views · {article.helpfulCount}👍 {article.notHelpfulCount}👎 · {article.slug}
                </div>
              </div>
              <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                {article.status === 'draft' && (
                  <button
                    onClick={() => handlePublishArticle(article.id, 'published')}
                    disabled={actionLoading === article.id}
                    title="Publish"
                    style={{ padding: '4px 8px', border: '1px solid #16a34a', borderRadius: 6, background: 'none', cursor: 'pointer', color: '#16a34a', fontSize: 12 }}
                  >
                    {actionLoading === article.id ? <Loader2 className="animate-spin" style={{ width: 12, height: 12 }} /> : 'Publish'}
                  </button>
                )}
                {article.status === 'published' && (
                  <button
                    onClick={() => handlePublishArticle(article.id, 'draft')}
                    disabled={actionLoading === article.id}
                    title="Unpublish"
                    style={{ padding: '4px 8px', border: '1px solid #d97706', borderRadius: 6, background: 'none', cursor: 'pointer', color: '#d97706', fontSize: 12 }}
                  >
                    Unpublish
                  </button>
                )}
                <button
                  onClick={() => handlePinArticle(article.id, article.isPinned)}
                  title={article.isPinned ? 'Unpin' : 'Pin'}
                  style={{ padding: '4px 8px', border: '1px solid #e2e8f0', borderRadius: 6, background: 'none', cursor: 'pointer', color: article.isPinned ? '#f59e0b' : '#64748b', fontSize: 12 }}
                >
                  <Pin style={{ width: 12, height: 12 }} />
                </button>
                <button
                  onClick={() => handleEditArticle(article)}
                  title="Edit"
                  style={{ padding: '4px 8px', border: '1px solid #e2e8f0', borderRadius: 6, background: 'none', cursor: 'pointer', color: '#6366f1', fontSize: 12 }}
                >
                  <Edit3 style={{ width: 12, height: 12 }} />
                </button>
                <button
                  onClick={() => handleDeleteArticle(article.id)}
                  disabled={actionLoading === article.id}
                  title="Delete"
                  style={{ padding: '4px 8px', border: '1px solid #e2e8f0', borderRadius: 6, background: 'none', cursor: 'pointer', color: '#ef4444', fontSize: 12 }}
                >
                  {actionLoading === article.id ? <Loader2 className="animate-spin" style={{ width: 12, height: 12 }} /> : <Trash2 style={{ width: 12, height: 12 }} />}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Categories Tab */}
      {activeTab === 'categories' && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: 16 }}>
            <button
              onClick={() => setShowCategoryModal(true)}
              style={{ padding: '8px 16px', backgroundColor: '#6366f1', color: '#fff', border: 'none', borderRadius: 8, cursor: 'pointer', fontWeight: 600, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6 }}
            >
              <Plus style={{ width: 16, height: 16 }} /> New Category
            </button>
          </div>

          {categories.map((cat) => (
            <div key={cat.id} style={{ padding: '14px 16px', borderBottom: '1px solid #f1f5f9', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  {cat.icon && <span style={{ fontSize: 20 }}>{cat.icon}</span>}
                  <span style={{ fontWeight: 600 }}>{cat.name}</span>
                  <span style={{ fontSize: 12, color: '#64748b' }}>{cat.slug}</span>
                </div>
                {cat.description && <div style={{ fontSize: 13, color: '#64748b', marginTop: 4 }}>{cat.description}</div>}
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <span style={{ fontSize: 13, color: '#64748b' }}>{cat.articleCount} articles</span>
                <span style={{ padding: '2px 8px', borderRadius: 12, fontSize: 11, backgroundColor: cat.isVisible ? '#dcfce7' : '#f1f5f9', color: cat.isVisible ? '#16a34a' : '#64748b' }}>
                  {cat.isVisible ? 'Visible' : 'Hidden'}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && settings && (
        <div style={{ maxWidth: 600 }}>
          <h3 style={{ margin: '0 0 16px' }}>Help Center Settings</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField label="Name">
              <input
                type="text"
                value={settings.name}
                onChange={(e) => setSettings({ ...settings, name: e.target.value })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Tagline">
              <input
                type="text"
                value={settings.tagline ?? ''}
                onChange={(e) => setSettings({ ...settings, tagline: e.target.value })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Primary Color">
              <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                <input
                  type="color"
                  value={settings.primaryColor}
                  onChange={(e) => setSettings({ ...settings, primaryColor: e.target.value })}
                  style={{ width: 40, height: 32, border: 'none', cursor: 'pointer' }}
                />
                <input
                  type="text"
                  value={settings.primaryColor}
                  onChange={(e) => setSettings({ ...settings, primaryColor: e.target.value })}
                  style={{ flex: 1, padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
                />
              </div>
            </FormField>
            <div style={{ display: 'flex', gap: 16 }}>
              <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={settings.enableSearch}
                  onChange={(e) => setSettings({ ...settings, enableSearch: e.target.checked })}
                />
                Enable Search
              </label>
              <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={settings.enableArticleFeedback}
                  onChange={(e) => setSettings({ ...settings, enableArticleFeedback: e.target.checked })}
                />
                Article Feedback
              </label>
            </div>
            <FormField label="Footer Text">
              <textarea
                value={settings.footerText ?? ''}
                onChange={(e) => setSettings({ ...settings, footerText: e.target.value })}
                rows={3}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14, resize: 'vertical' }}
              />
            </FormField>
            <button
              onClick={handleUpdateSettings}
              disabled={actionLoading === 'settings'}
              style={{ padding: '10px 20px', backgroundColor: '#6366f1', color: '#fff', border: 'none', borderRadius: 8, cursor: 'pointer', fontWeight: 600, fontSize: 14, alignSelf: 'flex-start', display: 'flex', alignItems: 'center', gap: 8 }}
            >
              {actionLoading === 'settings' && <Loader2 className="animate-spin" style={{ width: 16, height: 16 }} />}
              Save Settings
            </button>
          </div>
        </div>
      )}

      {/* Article Modal */}
      {showArticleModal && (
        <Modal onClose={() => { setShowArticleModal(false); resetArticleForm() }}>
          <h3 style={{ margin: '0 0 16px' }}>{editingArticle ? 'Edit Article' : 'New Article'}</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <FormField label="Title">
              <input
                type="text"
                value={articleForm.title}
                onChange={(e) => setArticleForm({ ...articleForm, title: e.target.value })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Slug">
              <input
                type="text"
                value={articleForm.slug}
                onChange={(e) => setArticleForm({ ...articleForm, slug: e.target.value })}
                placeholder="my-article-slug"
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Category">
              <select
                value={articleForm.categoryId}
                onChange={(e) => setArticleForm({ ...articleForm, categoryId: e.target.value })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              >
                <option value="">Select a category</option>
                {categories.map((c) => (
                  <option key={c.id} value={c.id}>{c.name}</option>
                ))}
              </select>
            </FormField>
            <FormField label="Content (HTML/Markdown)">
              <textarea
                value={articleForm.content}
                onChange={(e) => setArticleForm({ ...articleForm, content: e.target.value })}
                rows={10}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14, resize: 'vertical', fontFamily: 'monospace' }}
              />
            </FormField>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button
                onClick={() => { setShowArticleModal(false); resetArticleForm() }}
                style={{ padding: '8px 16px', border: '1px solid #e2e8f0', borderRadius: 8, background: 'none', cursor: 'pointer', fontSize: 13 }}
              >
                Cancel
              </button>
              <button
                onClick={handleCreateArticle}
                disabled={actionLoading === 'create-article'}
                style={{ padding: '8px 16px', backgroundColor: '#6366f1', color: '#fff', border: 'none', borderRadius: 8, cursor: 'pointer', fontWeight: 600, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6 }}
              >
                {actionLoading === 'create-article' && <Loader2 className="animate-spin" style={{ width: 14, height: 14 }} />}
                {editingArticle ? 'Update' : 'Create'}
              </button>
            </div>
          </div>
        </Modal>
      )}

      {/* Category Modal */}
      {showCategoryModal && (
        <Modal onClose={() => setShowCategoryModal(false)}>
          <h3 style={{ margin: '0 0 16px' }}>New Category</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <FormField label="Name">
              <input
                type="text"
                value={categoryForm.name}
                onChange={(e) => setCategoryForm({ ...categoryForm, name: e.target.value })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Slug">
              <input
                type="text"
                value={categoryForm.slug}
                onChange={(e) => setCategoryForm({ ...categoryForm, slug: e.target.value })}
                placeholder="category-slug"
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Description">
              <textarea
                value={categoryForm.description}
                onChange={(e) => setCategoryForm({ ...categoryForm, description: e.target.value })}
                rows={2}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14, resize: 'vertical' }}
              />
            </FormField>
            <FormField label="Icon (emoji)">
              <input
                type="text"
                value={categoryForm.icon}
                onChange={(e) => setCategoryForm({ ...categoryForm, icon: e.target.value })}
                placeholder="📚"
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <FormField label="Display Order">
              <input
                type="number"
                value={categoryForm.displayOrder}
                onChange={(e) => setCategoryForm({ ...categoryForm, displayOrder: parseInt(e.target.value) || 0 })}
                style={{ width: '100%', padding: '8px 12px', border: '1px solid #e2e8f0', borderRadius: 8, fontSize: 14 }}
              />
            </FormField>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button
                onClick={() => setShowCategoryModal(false)}
                style={{ padding: '8px 16px', border: '1px solid #e2e8f0', borderRadius: 8, background: 'none', cursor: 'pointer', fontSize: 13 }}
              >
                Cancel
              </button>
              <button
                onClick={handleCreateCategory}
                disabled={actionLoading === 'create-category'}
                style={{ padding: '8px 16px', backgroundColor: '#6366f1', color: '#fff', border: 'none', borderRadius: 8, cursor: 'pointer', fontWeight: 600, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6 }}
              >
                {actionLoading === 'create-category' && <Loader2 className="animate-spin" style={{ width: 14, height: 14 }} />}
                Create
              </button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  )
}

function StatCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string | number }) {
  return (
    <div style={{ padding: 20, backgroundColor: '#f8fafc', borderRadius: 12, border: '1px solid #e2e8f0' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, color: '#6366f1' }}>
        {icon}
        <span style={{ fontSize: 13, color: '#64748b' }}>{label}</span>
      </div>
      <div style={{ fontSize: 28, fontWeight: 700 }}>{value}</div>
    </div>
  )
}

function FormField({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label style={{ display: 'block', fontSize: 13, fontWeight: 600, marginBottom: 4, color: '#374151' }}>{label}</label>
      {children}
    </div>
  )
}

function Modal({ onClose, children }: { onClose: () => void; children: React.ReactNode }) {
  return (
    <div style={{ position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.5)', display: 'flex', justifyContent: 'center', alignItems: 'center', zIndex: 1000 }} onClick={onClose}>
      <div style={{ backgroundColor: '#fff', borderRadius: 12, padding: 24, maxWidth: 600, width: '90%', maxHeight: '80vh', overflow: 'auto' }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
          <h3 style={{ margin: 0 }}>Modal</h3>
          <button onClick={onClose} style={{ border: 'none', background: 'none', cursor: 'pointer', padding: 4 }}>
            <X style={{ width: 20, height: 20 }} />
          </button>
        </div>
        {children}
      </div>
    </div>
  )
}
