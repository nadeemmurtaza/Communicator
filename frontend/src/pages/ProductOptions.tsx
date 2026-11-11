import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Plus, X, Check, Trash2, Edit2, Save, Package } from '../components/icons'
import {
  api,
  listProductOptions,
  createProductOption,
  updateProductOption,
  deleteProductOption,
  createProductOptionValue,
  deleteProductOptionValue,
  type CommerceProduct,
} from '../lib/api'

interface OptionValue {
  id: string
  name: string
  priceAdjustment: number
  sku?: string
  isActive: boolean
}

interface ProductOption {
  id: string
  name: string
  description?: string
  priceAdjustment: number
  required: boolean
  maxSelections: number
  isActive: boolean
  values: OptionValue[]
}

export function ProductOptions() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const productId = id!

  const [product, setProduct] = useState<CommerceProduct | null>(null)
  const [options, setOptions] = useState<ProductOption[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // New option form
  const [showForm, setShowForm] = useState(false)
  const [optName, setOptName] = useState('')
  const [optDescription, setOptDescription] = useState('')
  const [optPrice, setOptPrice] = useState('0')
  const [optRequired, setOptRequired] = useState(false)
  const [optMaxSelections, setOptMaxSelections] = useState('1')
  const [saving, setSaving] = useState(false)

  // Edit option form
  const [editingOption, setEditingOption] = useState<ProductOption | null>(null)
  const [editName, setEditName] = useState('')
  const [editDescription, setEditDescription] = useState('')
  const [editPrice, setEditPrice] = useState('0')
  const [editRequired, setEditRequired] = useState(false)
  const [editMaxSelections, setEditMaxSelections] = useState('1')

  // New value form
  const [addingValuesFor, setAddingValuesFor] = useState<string | null>(null)
  const [valName, setValName] = useState('')
  const [valPrice, setValPrice] = useState('0')
  const [valSku, setValSku] = useState('')

  useEffect(() => {
    loadOptions()
  }, [productId])

  async function loadOptions() {
    try {
      const [opts, prod] = await Promise.all([
        listProductOptions(productId).catch(() => []),
        api.getProduct(productId).catch(() => null),
      ])
      setOptions(opts)
      setProduct(prod)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setLoading(false)
    }
  }

  async function createOption(e: React.FormEvent) {
    e.preventDefault()
    setSaving(true)
    setError(null)
    try {
      await createProductOption({
        productId,
        name: optName,
        description: optDescription || undefined,
        priceAdjustment: parseFloat(optPrice) || 0,
        required: optRequired,
        maxSelections: parseInt(optMaxSelections) || 1,
      })
      setOptName('')
      setOptDescription('')
      setOptPrice('0')
      setOptRequired(false)
      setOptMaxSelections('1')
      setShowForm(false)
      setSuccess(`Option "${optName}" created.`)
      loadOptions()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function updateOption(optionId: string) {
    setSaving(true)
    setError(null)
    try {
      await updateProductOption(optionId, {
        name: editName,
        description: editDescription || undefined,
        priceAdjustment: parseFloat(editPrice) || 0,
        required: editRequired,
        maxSelections: parseInt(editMaxSelections) || 1,
      })
      setEditingOption(null)
      setSuccess(`Option "${editName}" updated.`)
      loadOptions()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  async function deleteOption(optionId: string) {
    if (!confirm('Delete this option and all its values?')) return
    try {
      await deleteProductOption(optionId)
      setSuccess('Option deleted.')
      loadOptions()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function addValue(optionId: string) {
    if (!valName.trim()) return
    try {
      await createProductOptionValue(optionId, {
        name: valName,
        priceAdjustment: parseFloat(valPrice) || 0,
        sku: valSku || undefined,
      })
      setValName('')
      setValPrice('0')
      setValSku('')
      setSuccess(`Value added.`)
      loadOptions()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  async function deleteValue(valueId: string) {
    if (!confirm('Delete this value?')) return
    try {
      await deleteProductOptionValue(valueId)
      setSuccess('Value deleted.')
      loadOptions()
    } catch (e) {
      setError((e as Error).message)
    }
  }

  function startEdit(option: ProductOption) {
    setEditingOption(option)
    setEditName(option.name)
    setEditDescription(option.description || '')
    setEditPrice(option.priceAdjustment.toString())
    setEditRequired(option.required)
    setEditMaxSelections(option.maxSelections.toString())
  }

  if (loading) return <div className="p-8 text-center text-[var(--color-ink-muted)]">Loading options...</div>

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate(`/commerce/catalog/${productId}`)}
            className="p-2 rounded-[var(--radius-sm)] hover:bg-[var(--color-surface-hover)]">
            <ArrowLeft className="h-5 w-5 text-[var(--color-ink-muted)]" />
          </button>
          <div>
            <h1 className="text-xl font-semibold text-[var(--color-ink)]">Product Options & Add-ons</h1>
            {product && (
              <div className="flex items-center gap-2 mt-1">
                <Package className="h-4 w-4 text-[var(--color-accent)]" />
                <span className="text-sm font-medium text-[var(--color-ink)]">{product.name}</span>
                <span className="text-sm text-[var(--color-ink-muted)]">
                  — ${product.price?.toFixed(2) ?? '0.00'}
                </span>
                <span className="text-xs text-[var(--color-ink-muted)]">
                  ({options.length} option{options.length !== 1 ? 's' : ''})
                </span>
              </div>
            )}
            {!product && (
              <p className="mt-1 text-sm text-[var(--color-ink-muted)]">
                {options.length} option{options.length !== 1 ? 's' : ''} — extras customers can add to this product
              </p>
            )}
          </div>
        </div>
        <button onClick={() => setShowForm((v) => !v)}
          className="shrink-0 flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
          {showForm ? <><X className="h-4 w-4" /> Cancel</> : <><Plus className="h-4 w-4" /> Add option</>}
        </button>
      </div>

      {/* Alerts */}
      {error && (
        <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          <span>{error}</span>
          <button onClick={() => setError(null)}><X className="h-4 w-4" /></button>
        </div>
      )}
      {success && (
        <div className="flex items-center justify-between rounded-[var(--radius-sm)] border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          <span className="flex items-center gap-1.5"><Check className="h-4 w-4" /> {success}</span>
          <button onClick={() => setSuccess(null)}><X className="h-4 w-4" /></button>
        </div>
      )}

      {/* Info card */}
      <div className="rounded-[var(--radius-lg)] border border-blue-200 bg-blue-50 p-4 text-sm text-blue-800">
        <p className="font-medium mb-1">How options work</p>
        <p className="text-blue-700">
          Options let customers add extras to a product — like &quot;Lemon butter +$2.00&quot; or &quot;Extra cheese +$1.50&quot;.
          Each option can have multiple values (e.g. &quot;Small&quot;, &quot;Medium&quot;, &quot;Large&quot;) with different price adjustments.
        </p>
      </div>

      {/* Channel Preview */}
      {product && options.length > 0 && (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
          <div className="px-5 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface-hover)]">
            <h2 className="text-sm font-bold text-[var(--color-ink)]">📱 Channel Preview — How customers see this</h2>
            <p className="text-xs text-[var(--color-ink-muted)] mt-0.5">Options render as native buttons in each chat channel</p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-0">
            {/* WhatsApp Preview */}
            <div className="p-4 border-r border-b border-[var(--color-border)]">
              <div className="flex items-center gap-2 mb-3">
                <span className="text-lg">💬</span>
                <span className="text-sm font-semibold text-[var(--color-ink)]">WhatsApp</span>
              </div>
              <div className="bg-emerald-50 rounded-lg p-3 text-xs text-emerald-900 space-y-1.5 font-mono">
                <div><strong>🛍️ {product.name}</strong></div>
                <div>${product.price?.toFixed(2) ?? '0.00'}</div>
                <div className="border-t border-emerald-200 pt-1.5 mt-1.5">
                  <strong>🎛️ Customize your order:</strong>
                  {options.map(opt => (
                    <div key={opt.id} className="ml-2 mt-1">
                      <strong>{opt.name}{opt.required ? ' (required)' : ''}:</strong>
                      {opt.values.map(val => (
                        <div key={val.id} className="ml-2">• {val.name}{val.priceAdjustment ? ` +$${val.priceAdjustment.toFixed(2)}` : ''}</div>
                      ))}
                    </div>
                  ))}
                </div>
                <div className="text-emerald-600 mt-1">👆 Reply with the options you want</div>
              </div>
            </div>

            {/* Instagram Preview */}
            <div className="p-4 border-b border-[var(--color-border)]">
              <div className="flex items-center gap-2 mb-3">
                <span className="text-lg">📸</span>
                <span className="text-sm font-semibold text-[var(--color-ink)]">Instagram DM</span>
              </div>
              <div className="bg-gradient-to-br from-purple-50 to-pink-50 rounded-lg p-3 text-xs text-purple-900 space-y-1.5 font-mono">
                <div>✨ {product.name} — ${product.price?.toFixed(2) ?? '0.00'}</div>
                {options.length > 0 && (
                  <>
                    <div className="border-t border-purple-200 pt-1.5 mt-1.5"><strong>✨ Customize:</strong></div>
                    {options.map(opt => (
                      <div key={opt.id} className="ml-2">
                        🔹 {opt.name}{opt.required ? ' *' : ''}
                        {opt.values.map(val => (
                          <div key={val.id} className="ml-2">• {val.name}{val.priceAdjustment ? ` (+$${val.priceAdjustment.toFixed(2)})` : ''}</div>
                        ))}
                      </div>
                    ))}
                    <div className="text-purple-600 mt-1">💬 Reply with your choices to order!</div>
                  </>
                )}
              </div>
            </div>

            {/* Telegram Preview */}
            <div className="p-4 border-r border-[var(--color-border)]">
              <div className="flex items-center gap-2 mb-3">
                <span className="text-lg">✈️</span>
                <span className="text-sm font-semibold text-[var(--color-ink)]">Telegram</span>
              </div>
              <div className="bg-blue-50 rounded-lg p-3 text-xs text-blue-900 space-y-1.5 font-mono">
                <div className="border border-blue-200 rounded p-2">
                  {options.map(opt => (
                    <div key={opt.id} className="mb-1">
                      <div className="bg-blue-100 rounded px-2 py-1 inline-block">🔹 {opt.name}</div>
                      {opt.values.map(val => (
                        <div key={val.id} className="ml-4 bg-blue-100 rounded px-2 py-1 inline-block ml-1">{val.name}{val.priceAdjustment ? ` +$${val.priceAdjustment.toFixed(2)}` : ''}</div>
                      ))}
                    </div>
                  ))}
                  <div className="flex gap-2 mt-2">
                    <span className="bg-blue-200 rounded px-3 py-1">✅ Add to cart</span>
                    <span className="bg-red-100 rounded px-3 py-1">❌ Cancel</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Email Preview */}
            <div className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <span className="text-lg">📧</span>
                <span className="text-sm font-semibold text-[var(--color-ink)]">Email</span>
              </div>
              <div className="bg-gray-50 rounded-lg p-3 text-xs text-gray-800 space-y-1.5 font-mono border border-gray-200">
                <div><strong>{product.name}</strong> — ${product.price?.toFixed(2) ?? '0.00'}</div>
                {options.length > 0 && (
                  <div className="bg-white rounded p-2 border border-gray-200 mt-1">
                    <strong>⚙️ Customize:</strong>
                    {options.map(opt => (
                      <div key={opt.id} className="mt-1">
                        <strong>{opt.name}</strong>{opt.required ? ' <em style="color:red">(required)</em>' : ''}:
                        {opt.values.map(val => (
                          <span key={val.id} className="ml-1">☐ {val.name}{val.priceAdjustment ? ` (+$${val.priceAdjustment.toFixed(2)})` : ''}</span>
                        ))}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create Option Form */}
      {showForm && (
        <form onSubmit={createOption} className="space-y-4 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Add new option</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Name</label>
              <input required value={optName} onChange={(e) => setOptName(e.target.value)}
                placeholder="e.g. Extra toppings, Size upgrade..."
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Default price adjustment</label>
              <input type="number" step="0.01" value={optPrice} onChange={(e) => setOptPrice(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Description</label>
              <input value={optDescription} onChange={(e) => setOptDescription(e.target.value)}
                placeholder="Optional description shown to customers"
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Max selections</label>
              <input type="number" min="1" value={optMaxSelections} onChange={(e) => setOptMaxSelections(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div className="flex items-center gap-2 pt-6">
              <input type="checkbox" id="optRequired" checked={optRequired} onChange={(e) => setOptRequired(e.target.checked)}
                className="rounded border-[var(--color-border)]" />
              <label htmlFor="optRequired" className="text-sm text-[var(--color-ink)]">Required (customer must select)</label>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <button type="submit" disabled={saving}
              className="rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50">
              {saving ? 'Creating...' : 'Create option'}
            </button>
            <button type="button" onClick={() => setShowForm(false)}
              className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">
              Cancel
            </button>
          </div>
        </form>
      )}

      {/* Edit Option Form */}
      {editingOption && (
        <div className="space-y-4 rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-5 shadow-sm">
          <h2 className="text-sm font-bold text-[var(--color-ink)]">Edit option</h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Name</label>
              <input required value={editName} onChange={(e) => setEditName(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Default price adjustment</label>
              <input type="number" step="0.01" value={editPrice} onChange={(e) => setEditPrice(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Description</label>
              <input value={editDescription} onChange={(e) => setEditDescription(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-1">Max selections</label>
              <input type="number" min="1" value={editMaxSelections} onChange={(e) => setEditMaxSelections(e.target.value)}
                className="w-full rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-2 text-sm bg-[var(--color-surface)]" />
            </div>
            <div className="flex items-center gap-2 pt-6">
              <input type="checkbox" id="editRequired" checked={editRequired} onChange={(e) => setEditRequired(e.target.checked)}
                className="rounded border-[var(--color-border)]" />
              <label htmlFor="editRequired" className="text-sm text-[var(--color-ink)]">Required</label>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <button type="button" onClick={() => updateOption(editingOption.id)} disabled={saving}
              className="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)] disabled:opacity-50">
              <Save className="h-4 w-4" /> {saving ? 'Saving...' : 'Save changes'}
            </button>
            <button type="button" onClick={() => setEditingOption(null)}
              className="rounded-[var(--radius-sm)] border border-[var(--color-border)] px-4 py-2 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-hover)]">
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Options List */}
      {options.length === 0 ? (
        <div className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] p-8 text-center shadow-sm">
          <p className="text-sm text-[var(--color-ink-muted)] mb-3">No options yet. Add extras customers can choose when ordering this product.</p>
          <button onClick={() => setShowForm(true)}
            className="inline-flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
            <Plus className="h-4 w-4" /> Add first option
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          {options.map((option) => (
            <div key={option.id} className="rounded-[var(--radius-lg)] border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm overflow-hidden">
              {/* Option header */}
              <div className="flex items-start justify-between gap-4 px-5 py-4 border-b border-[var(--color-border)]">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium text-[var(--color-ink)]">{option.name}</h3>
                    {option.required && (
                      <span className="rounded-full bg-red-100 px-2 py-0.5 text-xs font-medium text-red-700">Required</span>
                    )}
                    {!option.isActive && (
                      <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-700">Inactive</span>
                    )}
                    <span className="rounded-full bg-[var(--color-accent-muted)] px-2 py-0.5 text-xs font-medium text-[var(--color-ink)]">
                      Max {option.maxSelections}
                    </span>
                  </div>
                  {option.description && (
                    <p className="mt-1 text-sm text-[var(--color-ink-muted)]">{option.description}</p>
                  )}
                  <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
                    Default: +${option.priceAdjustment.toFixed(2)} · {option.values.length} value{option.values.length !== 1 ? 's' : ''}
                  </p>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  <button onClick={() => startEdit(option)}
                    className="p-1.5 rounded-[var(--radius-sm)] text-[var(--color-accent)] hover:bg-[var(--color-surface-hover)]">
                    <Edit2 className="h-4 w-4" />
                  </button>
                  <button onClick={() => setAddingValuesFor(addingValuesFor === option.id ? null : option.id)}
                    className="p-1.5 rounded-[var(--radius-sm)] text-[var(--color-accent)] hover:bg-[var(--color-surface-hover)]"
                    title="Add value">
                    <Plus className="h-4 w-4" />
                  </button>
                  <button onClick={() => deleteOption(option.id)}
                    className="p-1.5 rounded-[var(--radius-sm)] text-red-600 hover:bg-red-50">
                    <Trash2 className="h-4 w-4" />
                  </button>
                </div>
              </div>

              {/* Values list */}
              {option.values.length > 0 && (
                <div className="px-5 py-3 bg-[var(--color-surface-hover)]">
                  <p className="text-xs font-semibold text-[var(--color-ink-muted)] uppercase tracking-wider mb-2">Values</p>
                  <div className="flex flex-wrap gap-2">
                    {option.values.map((val) => (
                      <div key={val.id} className="group flex items-center gap-1.5 rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-sm">
                        <span className="text-[var(--color-ink)]">{val.name}</span>
                        {val.priceAdjustment !== 0 && (
                          <span className={`text-xs font-medium ${val.priceAdjustment > 0 ? 'text-emerald-600' : 'text-red-600'}`}>
                            {val.priceAdjustment > 0 ? '+' : ''}${val.priceAdjustment.toFixed(2)}
                          </span>
                        )}
                        <button onClick={() => deleteValue(val.id)}
                          className="ml-1 p-0.5 rounded-full text-red-400 opacity-0 group-hover:opacity-100 hover:text-red-600 hover:bg-red-50">
                          <X className="h-3 w-3" />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Add value form */}
              {addingValuesFor === option.id && (
                <div className="px-5 py-3 border-t border-[var(--color-border)] flex items-center gap-3">
                  <input value={valName} onChange={(e) => setValName(e.target.value)}
                    placeholder="Value name (e.g. Lemon butter)"
                    className="flex-1 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm bg-[var(--color-surface)]"
                    onKeyDown={(e) => { if (e.key === 'Enter') addValue(option.id) }} />
                  <input type="number" step="0.01" value={valPrice} onChange={(e) => setValPrice(e.target.value)}
                    placeholder="Price +"
                    className="w-24 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm bg-[var(--color-surface)]" />
                  <input value={valSku} onChange={(e) => setValSku(e.target.value)}
                    placeholder="SKU (optional)"
                    className="w-32 rounded-[var(--radius-sm)] border border-[var(--color-border)] px-3 py-1.5 text-sm bg-[var(--color-surface)]" />
                  <button onClick={() => addValue(option.id)}
                    className="flex items-center gap-1 rounded-[var(--radius-sm)] bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white hover:bg-[var(--color-accent-hover)]">
                    <Plus className="h-3 w-3" /> Add
                  </button>
                  <button onClick={() => { setAddingValuesFor(null); setValName(''); setValPrice('0'); setValSku('') }}
                    className="p-1.5 rounded-[var(--radius-sm)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-hover)]">
                    <X className="h-4 w-4" />
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
