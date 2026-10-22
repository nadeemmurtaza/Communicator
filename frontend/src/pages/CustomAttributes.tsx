import { useEffect, useState } from 'react'
import { api, type CustomAttributeDefinitionItem } from '../lib/api'

export function CustomAttributes() {
  const [activeModel, setActiveModel] = useState<'contact_attribute' | 'conversation_attribute'>('contact_attribute')
  const [attributes, setAttributes] = useState<CustomAttributeDefinitionItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')

  // Modal State
  const [showModal, setShowModal] = useState(false)
  const [editingAttr, setEditingAttr] = useState<CustomAttributeDefinitionItem | null>(null)
  const [name, setName] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [description, setDescription] = useState('')
  const [attrType, setAttrType] = useState<'text' | 'number' | 'currency' | 'boolean' | 'list' | 'date' | 'link'>('text')
  const [optionsText, setOptionsText] = useState('')
  const [defaultValue, setDefaultValue] = useState('')
  const [isRequired, setIsRequired] = useState(false)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    loadAttributes()
  }, [])

  async function loadAttributes() {
    try {
      setLoading(true)
      setError(null)
      const data = await api.getCustomAttributes()
      setAttributes(data)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  function openCreateModal() {
    setEditingAttr(null)
    setName('')
    setDisplayName('')
    setDescription('')
    setAttrType('text')
    setOptionsText('')
    setDefaultValue('')
    setIsRequired(false)
    setShowModal(true)
  }

  function openEditModal(attr: CustomAttributeDefinitionItem) {
    setEditingAttr(attr)
    setName(attr.name)
    setDisplayName(attr.display_name)
    setDescription(attr.description || '')
    setAttrType(attr.attribute_type)
    setOptionsText((attr.options || []).join(', '))
    setDefaultValue(attr.default_value || '')
    setIsRequired(attr.is_required)
    setShowModal(true)
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    try {
      const options =
        attrType === 'list'
          ? optionsText
              .split(',')
              .map((o) => o.trim())
              .filter(Boolean)
          : []

      if (editingAttr) {
        await api.updateCustomAttribute(editingAttr.id, {
          display_name: displayName.trim(),
          description: description.trim(),
          options: options.length > 0 ? options : undefined,
          default_value: defaultValue.trim() || undefined,
          is_required: isRequired,
        })
        setSuccess(`Attribute "${displayName}" updated successfully!`)
      } else {
        await api.createCustomAttribute({
          name: name.trim() || displayName.toLowerCase().replace(/[^a-z0-9_]/g, '_'),
          display_name: displayName.trim(),
          description: description.trim(),
          attribute_model: activeModel,
          attribute_type: attrType,
          options,
          default_value: defaultValue.trim() || undefined,
          is_required: isRequired,
        })
        setSuccess(`Custom attribute "${displayName}" created successfully!`)
      }

      setShowModal(false)
      setTimeout(() => setSuccess(null), 3500)
      loadAttributes()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function handleDelete(attr: CustomAttributeDefinitionItem) {
    if (!window.confirm(`Are you sure you want to delete custom attribute "${attr.display_name}"?`)) {
      return
    }
    try {
      await api.deleteCustomAttribute(attr.id)
      setSuccess(`Attribute "${attr.display_name}" deleted.`)
      setTimeout(() => setSuccess(null), 3000)
      setAttributes((prev) => prev.filter((a) => a.id !== attr.id))
    } catch (e) {
      setError((e as Error).message)
    }
  }

  const filtered = attributes
    .filter((a) => a.attribute_model === activeModel)
    .filter(
      (a) =>
        a.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        a.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        a.description.toLowerCase().includes(searchQuery.toLowerCase())
    )

  const getTypeBadge = (type: string) => {
    switch (type) {
      case 'text':
        return <span className="bg-slate-100 text-slate-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-slate-200">📝 TEXT</span>
      case 'number':
        return <span className="bg-emerald-50 text-emerald-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-emerald-200">🔢 NUMBER</span>
      case 'currency':
        return <span className="bg-green-50 text-green-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-green-200">💲 CURRENCY</span>
      case 'boolean':
        return <span className="bg-purple-50 text-purple-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-purple-200">🔘 BOOLEAN</span>
      case 'list':
        return <span className="bg-blue-50 text-blue-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-blue-200">📋 LIST / DROPDOWN</span>
      case 'date':
        return <span className="bg-amber-50 text-amber-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-amber-200">📅 DATE</span>
      case 'link':
        return <span className="bg-cyan-50 text-cyan-700 px-2 py-0.5 rounded font-mono text-[10px] font-bold border border-cyan-200">🔗 URL</span>
      default:
        return <span className="bg-slate-100 text-slate-600 px-2 py-0.5 rounded font-mono text-[10px]">{type.toUpperCase()}</span>
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-slate-200 pb-5">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-slate-900">
              Custom Attributes & CRM Schema Builder
            </h1>
            <span className="inline-flex items-center gap-1 rounded-full bg-blue-50 px-2.5 py-0.5 text-xs font-semibold text-blue-700 border border-blue-200">
              CRM Data Model
            </span>
          </div>
          <p className="text-xs text-slate-500 mt-1">
            Define structured enterprise fields for Contacts and Conversations to power automation rules, segmentation, and live inbox sidebars.
          </p>
        </div>

        <button
          onClick={openCreateModal}
          className="inline-flex items-center gap-1.5 rounded-lg bg-blue-600 px-4 py-2 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 cursor-pointer transition-colors"
        >
          <span>+ Add Custom Attribute</span>
        </button>
      </div>

      {success && (
        <div className="rounded-lg border border-emerald-200 bg-emerald-50 p-3.5 text-xs text-emerald-800 flex items-center justify-between">
          <span>✅ {success}</span>
          <button onClick={() => setSuccess(null)} className="text-emerald-600 hover:text-emerald-900">✕</button>
        </div>
      )}

      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-3.5 text-xs text-red-800 flex items-center justify-between">
          <span>⚠️ {error}</span>
          <button onClick={() => setError(null)} className="text-red-600 hover:text-red-900">✕</button>
        </div>
      )}

      {/* Tabs & Search */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div className="flex items-center gap-1 rounded-lg bg-slate-200/80 p-1 text-xs font-semibold">
          <button
            onClick={() => setActiveModel('contact_attribute')}
            className={`rounded-md px-4 py-1.5 transition-all cursor-pointer ${
              activeModel === 'contact_attribute'
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-slate-600 hover:text-slate-900'
            }`}
          >
            👤 Contact Attributes ({attributes.filter((a) => a.attribute_model === 'contact_attribute').length})
          </button>
          <button
            onClick={() => setActiveModel('conversation_attribute')}
            className={`rounded-md px-4 py-1.5 transition-all cursor-pointer ${
              activeModel === 'conversation_attribute'
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-slate-600 hover:text-slate-900'
            }`}
          >
            💬 Conversation Attributes ({attributes.filter((a) => a.attribute_model === 'conversation_attribute').length})
          </button>
        </div>

        <div className="relative">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search attributes by name or slug..."
            className="w-full sm:w-64 rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs text-slate-800 focus:border-blue-500 focus:outline-none"
          />
        </div>
      </div>

      {/* Attributes Table */}
      <div className="rounded-2xl border border-slate-200 bg-white shadow-sm overflow-hidden">
        {loading ? (
          <div className="p-10 text-center text-xs text-slate-400">Loading CRM schema...</div>
        ) : filtered.length === 0 ? (
          <div className="p-10 text-center space-y-2">
            <div className="text-2xl">📋</div>
            <p className="text-xs font-semibold text-slate-700">No custom attributes found</p>
            <p className="text-[11px] text-slate-400">Click "+ Add Custom Attribute" to define your first field.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse text-xs">
              <thead>
                <tr className="border-b border-slate-200 bg-slate-50/70 text-slate-500 font-semibold text-[11px]">
                  <th className="py-3 px-4">Display Name & Key</th>
                  <th className="py-3 px-4">Data Type</th>
                  <th className="py-3 px-4">Description</th>
                  <th className="py-3 px-4">Default Value / Options</th>
                  <th className="py-3 px-4 text-center">Required</th>
                  <th className="py-3 px-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {filtered.map((attr) => (
                  <tr key={attr.id} className="hover:bg-slate-50/80 transition-colors">
                    <td className="py-3 px-4">
                      <div className="font-bold text-slate-900">{attr.display_name}</div>
                      <div className="font-mono text-[10px] text-slate-400">{attr.name}</div>
                    </td>
                    <td className="py-3 px-4">
                      {getTypeBadge(attr.attribute_type)}
                    </td>
                    <td className="py-3 px-4 text-slate-600 max-w-xs truncate">
                      {attr.description || '—'}
                    </td>
                    <td className="py-3 px-4">
                      {attr.attribute_type === 'list' && attr.options?.length > 0 ? (
                        <div className="flex flex-wrap gap-1 max-w-xs">
                          {attr.options.slice(0, 3).map((opt) => (
                            <span key={opt} className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] text-slate-600">
                              {opt}
                            </span>
                          ))}
                          {attr.options.length > 3 && (
                            <span className="text-[10px] text-slate-400">+{attr.options.length - 3} more</span>
                          )}
                        </div>
                      ) : (
                        <span className="text-slate-500 font-mono text-[11px]">{attr.default_value ?? '—'}</span>
                      )}
                    </td>
                    <td className="py-3 px-4 text-center">
                      {attr.is_required ? (
                        <span className="rounded bg-red-50 text-red-600 px-1.5 py-0.5 text-[10px] font-bold border border-red-200">
                          Required
                        </span>
                      ) : (
                        <span className="text-slate-400 text-[10px]">Optional</span>
                      )}
                    </td>
                    <td className="py-3 px-4 text-right space-x-2">
                      <button
                        onClick={() => openEditModal(attr)}
                        className="text-xs font-semibold text-blue-600 hover:text-blue-800 cursor-pointer"
                      >
                        Edit
                      </button>
                      <button
                        onClick={() => handleDelete(attr)}
                        className="text-xs font-semibold text-red-600 hover:text-red-800 cursor-pointer"
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* CREATE / EDIT MODAL */}
      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4 backdrop-blur-xs">
          <div className="w-full max-w-lg rounded-2xl bg-white shadow-2xl border border-slate-200 overflow-hidden flex flex-col max-h-[90vh]">
            <div className="flex items-center justify-between p-4 border-b border-slate-200 bg-slate-50">
              <div>
                <h3 className="font-bold text-sm text-slate-900">
                  {editingAttr ? `Edit Attribute: ${editingAttr.display_name}` : 'Create Custom Attribute'}
                </h3>
                <p className="text-[11px] text-slate-500">
                  Target: {activeModel === 'contact_attribute' ? 'Contact Profile' : 'Conversation Thread'}
                </p>
              </div>
              <button
                onClick={() => setShowModal(false)}
                className="h-7 w-7 rounded-full bg-slate-200 text-slate-600 hover:bg-slate-300 flex items-center justify-center text-xs font-bold cursor-pointer"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleSubmit} className="p-5 space-y-4 overflow-y-auto flex-1 text-xs">
              <div>
                <label className="block font-semibold text-slate-700 mb-1">Display Label *</label>
                <input
                  required
                  value={displayName}
                  onChange={(e) => {
                    setDisplayName(e.target.value)
                    if (!editingAttr && !name) {
                      setName(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, '_'))
                    }
                  }}
                  placeholder="e.g. Annual Contract Value (ARR)"
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="block font-semibold text-slate-700 mb-1">Slug Identifier (API Key) *</label>
                <input
                  required
                  disabled={!!editingAttr}
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. annual_contract_value"
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 font-mono text-[11px] focus:border-blue-500 focus:outline-none disabled:bg-slate-100"
                />
              </div>

              <div>
                <label className="block font-semibold text-slate-700 mb-1">Attribute Data Type *</label>
                <select
                  disabled={!!editingAttr}
                  value={attrType}
                  onChange={(e) => setAttrType(e.target.value as any)}
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none disabled:bg-slate-100"
                >
                  <option value="text">📝 Text (Single-line string)</option>
                  <option value="number">🔢 Number (Integer or decimal)</option>
                  <option value="currency">💲 Currency (Formatted monetary amount)</option>
                  <option value="boolean">🔘 Boolean (True / False toggle)</option>
                  <option value="list">📋 List / Dropdown (Predefined choices)</option>
                  <option value="date">📅 Date (YYYY-MM-DD)</option>
                  <option value="link">🔗 Link (Web URL)</option>
                </select>
              </div>

              {attrType === 'list' && (
                <div>
                  <label className="block font-semibold text-slate-700 mb-1">
                    Dropdown Options (comma-separated) *
                  </label>
                  <input
                    required
                    value={optionsText}
                    onChange={(e) => setOptionsText(e.target.value)}
                    placeholder="e.g. Free, Starter, Growth, Enterprise"
                    className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none"
                  />
                  <p className="text-[10px] text-slate-400 mt-1">Separate options with commas.</p>
                </div>
              )}

              <div>
                <label className="block font-semibold text-slate-700 mb-1">Description / Tooltip</label>
                <input
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Brief note for agents on how to use this field"
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="block font-semibold text-slate-700 mb-1">Default Value</label>
                <input
                  value={defaultValue}
                  onChange={(e) => setDefaultValue(e.target.value)}
                  placeholder="Optional fallback value"
                  className="w-full rounded-lg border border-slate-300 px-3 py-2 focus:border-blue-500 focus:outline-none font-mono text-[11px]"
                />
              </div>

              <div className="pt-2">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={isRequired}
                    onChange={(e) => setIsRequired(e.target.checked)}
                    className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="font-semibold text-slate-800">Required field</span>
                </label>
              </div>

              <div className="pt-4 border-t border-slate-100 flex items-center justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowModal(false)}
                  className="rounded-lg border border-slate-300 bg-white px-4 py-2 text-xs font-semibold text-slate-700 hover:bg-slate-50 cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={saving || !displayName.trim()}
                  className="rounded-lg bg-blue-600 px-5 py-2 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 disabled:opacity-50 cursor-pointer"
                >
                  {saving ? 'Saving...' : editingAttr ? 'Update Attribute' : 'Create Attribute'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
