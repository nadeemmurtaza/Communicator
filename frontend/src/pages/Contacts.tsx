import { useEffect, useRef, useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import {
  Globe,
  MessageCircle,
  Mail,
  Smartphone,
  Music,
  Send,
  Camera,
  Users,
  MessageSquare,
  Phone,
  RefreshCw,
  Download,
  Upload,
  Plus,
  Search,
  AlertCircle,
  CheckCircle2,
  X,
  Tag as TagIcon,
  Pencil,
  Check,
} from '../components/icons'
import {
  api,
  type ContactRecord,
  type ContactSegment,
  type ImportPreviewResult,
} from '../lib/api'

const CHANNELS = [
  { id: 'all', label: 'All Channels', icon: Globe },
  { id: 'whatsapp', label: 'WhatsApp', icon: MessageCircle },
  { id: 'email', label: 'Email', icon: Mail },
  { id: 'sms', label: 'SMS', icon: Smartphone },
  { id: 'tiktok', label: 'TikTok', icon: Music },
  { id: 'telegram', label: 'Telegram', icon: Send },
  { id: 'instagram', label: 'Instagram', icon: Camera },
  { id: 'facebook', label: 'Facebook', icon: Users },
  { id: 'line', label: 'LINE', icon: MessageSquare },
  { id: 'voice', label: 'Voice', icon: Phone },
]

export function Contacts() {
  const navigate = useNavigate()
  const [contacts, setContacts] = useState<ContactRecord[] | null>(null)
  const [segments, setSegments] = useState<ContactSegment[]>([])
  const [tags, setTags] = useState<string[]>([])
  const [selectedChannel, setSelectedChannel] = useState<string>('all')
  const [selectedSegmentId, setSelectedSegmentId] = useState<string>('all')
  const [selectedTag, setSelectedTag] = useState<string>('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [syncingMeta, setSyncingMeta] = useState(false)

  // Modals state
  const [showImportModal, setShowImportModal] = useState(false)
  const [showSegmentModal, setShowSegmentModal] = useState(false)
  const [editingSegment, setEditingSegment] = useState<ContactSegment | null>(null)

  // Import State
  const [importFile, setImportFile] = useState<File | null>(null)
  const [defaultChannel, setDefaultChannel] = useState<string>('')
  const [defaultTags, setDefaultTags] = useState<string>('')
  const [importPreview, setImportPreview] = useState<ImportPreviewResult | null>(null)
  const [previewing, setPreviewing] = useState(false)
  const [importing, setImporting] = useState(false)
  const fileRef = useRef<HTMLInputElement>(null)

  // Segment Form State
  const [segmentName, setSegmentName] = useState('')
  const [segmentDesc, setSegmentDesc] = useState('')
  const [segmentChannels, setSegmentChannels] = useState<string[]>([])
  const [segmentTags, setSegmentTags] = useState<string[]>([])
  const [segmentTagMode, setSegmentTagMode] = useState<'any' | 'all'>('any')
  const [savingSegment, setSavingSegment] = useState(false)

  // Tag Management on individual contact
  const [taggingContactId, setTaggingContactId] = useState<string | null>(null)
  const [newTagInput, setNewTagInput] = useState('')

  useEffect(() => {
    loadSegments()
    loadTags()
  }, [])

  useEffect(() => {
    loadContacts()
  }, [selectedChannel, selectedSegmentId, selectedTag, searchQuery])

  async function loadSegments() {
    try {
      const segs = await api.listSegments()
      setSegments(segs)
    } catch {
      // Ignore initial segment load error
    }
  }

  async function loadTags() {
    try {
      const t = await api.listContactTags()
      setTags(t.map((x) => x.name))
    } catch {
      // Ignore
    }
  }

  async function loadContacts() {
    try {
      setError(null)
      const data = await api.listContacts({
        channel: selectedChannel === 'all' ? undefined : selectedChannel,
        segment_id: selectedSegmentId === 'all' ? undefined : selectedSegmentId,
        tag: selectedTag === 'all' ? undefined : selectedTag,
        search: searchQuery.trim() || undefined,
      })
      setContacts(data)
    } catch (err: any) {
      setError(err.message || 'Failed to load contacts')
    }
  }

  async function handleSyncMeta() {
    try {
      setSyncingMeta(true)
      setError(null)
      const res = await api.syncMetaContacts()
      setNotice(res.message || 'Successfully synchronized contacts from Meta Cloud API.')
      await loadContacts()
      await loadSegments()
      await loadTags()
    } catch (err: any) {
      setError(err.message || 'Failed to synchronize contacts from Meta')
    } finally {
      setSyncingMeta(false)
    }
  }

  // Handle CSV File Selection & Preview
  async function handleFileSelect(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    setImportFile(file)
    setPreviewing(true)
    setError(null)

    try {
      const preview = await api.previewContactImport(file, defaultChannel || undefined)
      setImportPreview(preview)
    } catch (err: any) {
      setError(err.message || 'Failed to preview CSV file')
    } finally {
      setPreviewing(false)
    }
  }

  async function handleExecuteImport() {
    if (!importFile) return
    setImporting(true)
    setError(null)

    try {
      const res = await api.importContacts(
        importFile,
        defaultChannel || undefined,
        defaultTags.trim() || undefined
      )
      const chList = Object.entries(res.channels || {})
        .map(([k, v]) => `${k.toUpperCase()}: ${v}`)
        .join(', ')
      setNotice(
        `Successfully imported ${res.created} new contacts (${res.updated} updated, ${res.skipped} skipped)${
          chList ? ` · [${chList}]` : ''
        }.`
      )
      loadContacts()
      loadSegments()
      loadTags()
      setImportFile(null)
      setImportPreview(null)
      setShowImportModal(false)
    } catch (err: any) {
      setError(err.message || 'CSV Import failed')
    } finally {
      setImporting(false)
    }
  }

  // Handle Segment Save
  async function handleSaveSegment() {
    if (!segmentName.trim()) {
      setError('Segment name cannot be empty')
      return
    }

    setSavingSegment(true)
    setError(null)

    const payload = {
      name: segmentName.trim(),
      description: segmentDesc.trim(),
      filters: {
        channels: segmentChannels.length > 0 ? segmentChannels : undefined,
        tags: segmentTags.length > 0 ? segmentTags : undefined,
        tag_match_mode: segmentTagMode,
      },
    }

    try {
      if (editingSegment) {
        await api.updateSegment(editingSegment.id, payload)
        setNotice(`Segment '${segmentName}' updated successfully.`)
      } else {
        await api.createSegment(payload)
        setNotice(`Segment '${segmentName}' created successfully.`)
      }
      setShowSegmentModal(false)
      setEditingSegment(null)
      resetSegmentForm()
      await loadSegments()
    } catch (err: any) {
      setError(err.message || 'Failed to save segment')
    } finally {
      setSavingSegment(false)
    }
  }

  async function handleDeleteSegment(seg: ContactSegment) {
    if (!confirm(`Are you sure you want to delete segment '${seg.name}'?`)) return
    try {
      await api.deleteSegment(seg.id)
      setNotice(`Segment '${seg.name}' deleted.`)
      if (selectedSegmentId === seg.id) {
        setSelectedSegmentId('all')
      }
      loadSegments()
    } catch (err: any) {
      setError(err.message || 'Failed to delete segment')
    }
  }

  function openEditSegment(seg: ContactSegment) {
    setEditingSegment(seg)
    setSegmentName(seg.name)
    setSegmentDesc(seg.description)
    setSegmentChannels(seg.filters?.channels || [])
    setSegmentTags(seg.filters?.tags || [])
    setSegmentTagMode(seg.filters?.tag_match_mode || 'any')
    setShowSegmentModal(true)
  }

  function resetSegmentForm() {
    setSegmentName('')
    setSegmentDesc('')
    setSegmentChannels([])
    setSegmentTags([])
    setSegmentTagMode('any')
  }

  async function handleAddTag(contactId: string) {
    if (!newTagInput.trim()) return
    try {
      await api.addContactTag(contactId, newTagInput.trim())
      setNewTagInput('')
      setTaggingContactId(null)
      loadContacts()
      loadTags()
    } catch (err: any) {
      setError(err.message || 'Failed to add tag')
    }
  }

  async function handleRemoveTag(contactId: string, tag: string) {
    try {
      await api.removeContactTag(contactId, tag)
      loadContacts()
      loadTags()
    } catch (err: any) {
      setError(err.message || 'Failed to remove tag')
    }
  }

  async function handleToggleConsent(contactId: string) {
    try {
      const res = await api.toggleContactConsent(contactId)
      setNotice(`Consent updated to ${res.optInStatus.toUpperCase()}`)
      loadContacts()
    } catch (err: any) {
      setError(err.message || 'Failed to toggle consent')
    }
  }

  async function handleOpenChat(contactId: string) {
    try {
      const conv = await api.getOrCreateContactConversation(contactId)
      navigate(`/conversations/${conv.id}`)
    } catch (err: any) {
      setError(err.message || 'Failed to open conversation')
    }
  }

  function getChannelIcon(channel: string) {
    const found = CHANNELS.find((c) => c.id === channel)
    const IconComp = found?.icon || MessageSquare
    return <IconComp className="h-3.5 w-3.5 shrink-0" />
  }

  function getChannelBadge(channel: string) {
    switch (channel) {
      case 'whatsapp':
        return 'bg-emerald-50 text-emerald-800 border-emerald-200'
      case 'email':
        return 'bg-indigo-50 text-indigo-800 border-indigo-200'
      case 'sms':
        return 'bg-blue-50 text-blue-800 border-blue-200'
      case 'tiktok':
        return 'bg-slate-900 text-white border-slate-700'
      case 'telegram':
        return 'bg-sky-50 text-sky-800 border-sky-200'
      case 'instagram':
        return 'bg-pink-50 text-pink-800 border-pink-200'
      case 'facebook':
        return 'bg-blue-50 text-blue-900 border-blue-200'
      case 'line':
        return 'bg-green-50 text-green-800 border-green-200'
      default:
        return 'bg-slate-50 text-slate-700 border-slate-200'
    }
  }

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-[var(--color-ink)]">Contacts &amp; Audience Segments</h1>
            <span className="rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-semibold text-[var(--color-ink)]">
              {contacts ? `${contacts.length} Reachable` : 'Loading...'}
            </span>
          </div>
          <p className="mt-1 text-xs text-[var(--color-ink-muted)]">
            Omnichannel customer directory with dynamic Meta Cloud sync, multi-channel CSV import, and targeting segments.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={handleSyncMeta}
            disabled={syncingMeta}
            className="rounded-md border border-slate-300 bg-white px-3.5 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50 shadow-xs flex items-center gap-1.5 transition-colors"
          >
            <RefreshCw className={`h-3.5 w-3.5 text-emerald-600 ${syncingMeta ? 'animate-spin' : ''}`} />
            {syncingMeta ? 'Syncing with Meta...' : 'Sync with Meta'}
          </button>
          <button
            type="button"
            onClick={() =>
              api.downloadContactsExport({
                channel: selectedChannel === 'all' ? undefined : selectedChannel,
                segment_id: selectedSegmentId === 'all' ? undefined : selectedSegmentId,
                tag: selectedTag === 'all' ? undefined : selectedTag,
                search: searchQuery.trim() || undefined,
              })
            }
            className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-xs flex items-center gap-1.5"
          >
            <Download className="h-3.5 w-3.5 text-slate-500" />
            Export CSV
          </button>
          <button
            type="button"
            onClick={() => {
              setEditingSegment(null)
              resetSegmentForm()
              setShowSegmentModal(true)
            }}
            className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)] shadow-xs flex items-center gap-1.5"
          >
            <TagIcon className="h-3.5 w-3.5 text-slate-500" />
            + New Segment
          </button>
          <button
            type="button"
            onClick={() => {
              setImportFile(null)
              setImportPreview(null)
              setShowImportModal(true)
            }}
            className="rounded-md bg-slate-900 px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs flex items-center gap-1.5"
          >
            <Upload className="h-3.5 w-3.5 text-white" />
            Import CSV
          </button>
        </div>
      </div>

      {/* Notifications */}
      {error && (
        <div className="rounded-lg bg-rose-50 border border-rose-200 p-3.5 text-xs text-rose-800 flex items-center justify-between shadow-xs">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4 text-rose-600 shrink-0" />
            <span>{error}</span>
          </div>
          <button onClick={() => setError(null)} className="text-rose-600 hover:text-rose-900">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      {notice && (
        <div className="rounded-lg bg-emerald-50 border border-emerald-200 p-3.5 text-xs text-emerald-800 flex items-center justify-between shadow-xs">
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-4 w-4 text-emerald-600 shrink-0" />
            <span>{notice}</span>
          </div>
          <button onClick={() => setNotice(null)} className="text-emerald-600 hover:text-emerald-900">
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      {/* Saved Dynamic Segments Carousel / Cards */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs font-bold uppercase tracking-wider text-[var(--color-ink)]">
            Saved Dynamic Audience Segments
          </span>
          <span className="text-[11px] text-[var(--color-ink-faint)]">
            Click segment to filter audience
          </span>
        </div>
        <div className="flex gap-2 overflow-x-auto pb-1 scrollbar-none">
          <button
            onClick={() => setSelectedSegmentId('all')}
            className={`shrink-0 rounded-lg border px-3 py-2 text-xs font-medium text-left transition-all ${
              selectedSegmentId === 'all'
                ? 'bg-slate-900 text-white border-slate-900 shadow-xs'
                : 'bg-[var(--color-surface)] text-[var(--color-ink)] border-[var(--color-border)] hover:border-slate-300'
            }`}
          >
            <div className="font-semibold">All Contacts</div>
            <div className="text-[10px] opacity-75">{contacts?.length ?? 0} contacts</div>
          </button>

          {segments.map((seg) => (
            <div
              key={seg.id}
              className={`shrink-0 rounded-lg border px-3 py-2 text-xs text-left transition-all flex items-center gap-3 ${
                selectedSegmentId === seg.id
                  ? 'bg-indigo-50/60 border-indigo-300 ring-1 ring-indigo-200'
                  : 'bg-[var(--color-surface)] border-[var(--color-border)] hover:border-slate-300'
              }`}
            >
              <button
                onClick={() => setSelectedSegmentId(seg.id)}
                className="text-left"
              >
                <div className="font-semibold text-[var(--color-ink)]">{seg.name}</div>
                <div className="text-[10px] text-[var(--color-ink-muted)]">
                  {seg.memberCount} members · {seg.filters?.channels?.join(', ') || 'all channels'}
                </div>
              </button>
              <div className="flex items-center gap-1.5 border-l border-[var(--color-border)] pl-2">
                <button
                  onClick={() => openEditSegment(seg)}
                  title="Edit Segment"
                  className="text-slate-400 hover:text-slate-700 p-0.5"
                >
                  <Pencil className="h-3 w-3" />
                </button>
                <button
                  onClick={() => handleDeleteSegment(seg)}
                  title="Delete Segment"
                  className="text-slate-400 hover:text-rose-600 p-0.5"
                >
                  <X className="h-3 w-3" />
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Filter Toolbar */}
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-xs space-y-3">
        {/* Search & Channel Filters */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div className="relative flex-1">
            <span className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Search className="h-3.5 w-3.5 text-slate-400" />
            </span>
            <input
              type="text"
              placeholder="Search contacts by name, email, phone number, or handle..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-8 pr-3 py-1.5 rounded-md border border-slate-300 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
            />
          </div>

          {/* Channel Selector Pills */}
          <div className="flex flex-wrap items-center gap-1.5 overflow-x-auto">
            {CHANNELS.map((ch) => {
              const IconComp = ch.icon
              return (
                <button
                  key={ch.id}
                  onClick={() => setSelectedChannel(ch.id)}
                  className={`rounded-full px-2.5 py-1 text-xs font-medium flex items-center gap-1.5 transition-all ${
                    selectedChannel === ch.id
                      ? 'bg-slate-900 text-white shadow-xs'
                      : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
                  }`}
                >
                  <IconComp className="h-3 w-3" />
                  <span>{ch.label}</span>
                </button>
              )
            })}
          </div>
        </div>

        {/* Tag Filters */}
        {tags.length > 0 && (
          <div className="flex items-center gap-2 pt-2 border-t border-[var(--color-border)] overflow-x-auto">
            <span className="text-[11px] font-semibold text-[var(--color-ink-faint)] uppercase tracking-wider shrink-0">
              Tags:
            </span>
            <button
              onClick={() => setSelectedTag('all')}
              className={`rounded-full px-2 py-0.5 text-[11px] font-medium shrink-0 ${
                selectedTag === 'all'
                  ? 'bg-slate-700 text-white'
                  : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
              }`}
            >
              All Tags
            </button>
            {tags.map((t) => (
              <button
                key={t}
                onClick={() => setSelectedTag(t)}
                className={`rounded-full px-2 py-0.5 text-[11px] font-medium shrink-0 ${
                  selectedTag === t
                    ? 'bg-slate-700 text-white'
                    : 'bg-slate-100 text-[var(--color-ink-muted)] hover:bg-slate-200'
                }`}
              >
                #{t}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Contacts Table */}
      {contacts === null ? (
        <div className="flex h-48 items-center justify-center">
          <div className="flex items-center gap-2 text-[var(--color-ink-muted)] text-xs">
            <div className="h-4 w-4 animate-spin rounded-full border-2 border-slate-300 border-t-slate-600" />
            <span>Loading contacts...</span>
          </div>
        </div>
      ) : contacts.length === 0 ? (
        <div className="rounded-xl border border-dashed border-slate-300 bg-[var(--color-surface)] p-10 text-center space-y-3">
          <Users className="h-10 w-10 text-slate-300 mx-auto" />
          <h3 className="text-sm font-bold text-[var(--color-ink)]">No matching contacts found</h3>
          <p className="text-xs text-[var(--color-ink-muted)] max-w-sm mx-auto">
            Try adjusting your search criteria, channel filters, or synchronize contacts directly from Meta Cloud.
          </p>
          <div className="flex items-center justify-center gap-2">
            <button
              onClick={handleSyncMeta}
              disabled={syncingMeta}
              className="rounded-md border border-slate-300 bg-white px-3 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 flex items-center gap-1.5"
            >
              <RefreshCw className={`h-3.5 w-3.5 text-emerald-600 ${syncingMeta ? 'animate-spin' : ''}`} />
              Sync with Meta
            </button>
            <button
              onClick={() => {
                setSelectedChannel('all')
                setSelectedSegmentId('all')
                setSelectedTag('all')
                setSearchQuery('')
              }}
              className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
            >
              Clear Filters
            </button>
          </div>
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xs">
          <table className="w-full text-xs">
            <thead className="border-b border-[var(--color-border)] bg-[var(--color-page)] text-left font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
              <tr>
                <th className="px-4 py-3">Phone Number</th>
                <th className="px-4 py-3">Customer Name</th>
                <th className="px-4 py-3">Channel</th>
                <th className="px-4 py-3">Consent</th>
                <th className="px-4 py-3">Language</th>
                <th className="px-4 py-3">Audience Tags</th>
                <th className="px-4 py-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {contacts.map((c) => (
                <tr
                  key={c.id}
                  onClick={(e) => {
                    if ((e.target as HTMLElement).closest('button, input, a')) return
                    navigate(`/contacts/${c.id}`)
                  }}
                  className="hover:bg-slate-50/70 transition-colors cursor-pointer"
                >
                  {/* Phone Number */}
                  <td className="px-4 py-3 font-mono text-xs font-semibold text-slate-800">
                    <div className="flex items-center gap-1.5">
                      <Phone className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                      <span>{c.value || '—'}</span>
                    </div>
                  </td>

                  {/* Customer Name */}
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2.5">
                      <div className="h-7 w-7 rounded-full bg-slate-100 border border-slate-200 flex items-center justify-center font-bold text-slate-800 text-[11px] shrink-0">
                        {c.personName ? c.personName.slice(0, 2).toUpperCase() : 'CU'}
                      </div>
                      <div>
                        <Link
                          to={`/contacts/${c.id}`}
                          className="font-semibold text-slate-900 hover:text-emerald-700 hover:underline"
                        >
                          {c.personName || 'Unnamed Contact'}
                        </Link>
                        <div className="text-[10px] text-slate-400 font-mono">ID: {c.id.slice(0, 8)}</div>
                      </div>
                    </div>
                  </td>

                  {/* Channel */}
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[11px] font-semibold ${getChannelBadge(
                        c.channel
                      )}`}
                    >
                      {getChannelIcon(c.channel)}
                      <span className="capitalize">{c.channel}</span>
                    </span>
                  </td>

                  {/* Consent */}
                  <td className="px-4 py-3">
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation()
                        handleToggleConsent(c.id)
                      }}
                      className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold transition-colors cursor-pointer ${
                        c.optInStatus?.toUpperCase() === 'OPTED_IN'
                          ? 'border-emerald-200 bg-emerald-50 text-emerald-800 hover:bg-emerald-100'
                          : 'border-amber-200 bg-amber-50 text-amber-800 hover:bg-amber-100'
                      }`}
                      title="Click to toggle consent"
                    >
                      <CheckCircle2 className="h-2.5 w-2.5" />
                      <span>{c.optInStatus?.toUpperCase() === 'OPTED_IN' ? 'Opted-In' : 'Opted-Out'}</span>
                    </button>
                  </td>

                  {/* Language */}
                  <td className="px-4 py-3">
                    <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-mono text-slate-600 uppercase">
                      {c.language || 'en'}
                    </span>
                  </td>

                  {/* Tags */}
                  <td className="px-4 py-3">
                    <div className="flex flex-wrap items-center gap-1">
                      {c.tags.map((t) => (
                        <span
                          key={t}
                          className="inline-flex items-center gap-1 rounded-md bg-slate-100 border border-slate-200 px-1.5 py-0.5 text-[10px] text-slate-700"
                        >
                          <span>#{t}</span>
                          <button
                            type="button"
                            onClick={(e) => {
                              e.stopPropagation()
                              handleRemoveTag(c.id, t)
                            }}
                            className="text-slate-400 hover:text-rose-600"
                          >
                            <X className="h-3 w-3" />
                          </button>
                        </span>
                      ))}

                      {taggingContactId === c.id ? (
                        <div className="inline-flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                          <input
                            type="text"
                            placeholder="Tag name"
                            value={newTagInput}
                            onChange={(e) => setNewTagInput(e.target.value)}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter') handleAddTag(c.id)
                              if (e.key === 'Escape') setTaggingContactId(null)
                            }}
                            autoFocus
                            className="rounded border border-slate-300 px-1.5 py-0.5 text-[10px] w-20 focus:outline-none focus:ring-1 focus:ring-slate-900"
                          />
                          <button
                            type="button"
                            onClick={() => handleAddTag(c.id)}
                            className="text-emerald-700 font-bold text-xs"
                          >
                            <Check className="h-3.5 w-3.5" />
                          </button>
                          <button
                            type="button"
                            onClick={() => setTaggingContactId(null)}
                            className="text-slate-400 hover:text-slate-600 text-xs"
                          >
                            <X className="h-3.5 w-3.5" />
                          </button>
                        </div>
                      ) : (
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation()
                            setTaggingContactId(c.id)
                            setNewTagInput('')
                          }}
                          className="rounded border border-dashed border-slate-300 px-1.5 py-0.5 text-[10px] text-slate-500 hover:border-slate-400 hover:text-slate-800 flex items-center gap-0.5"
                        >
                          <Plus className="h-2.5 w-2.5" />
                          <span>Tag</span>
                        </button>
                      )}
                    </div>
                  </td>

                  {/* Actions / Added */}
                  <td className="px-4 py-3 text-right">
                    <div className="flex items-center justify-end gap-2">
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleOpenChat(c.id)
                        }}
                        title="Open Conversation"
                        className="inline-flex items-center gap-1 rounded-md border border-slate-200 bg-white px-2 py-1 text-[11px] font-semibold text-slate-700 hover:bg-slate-50 hover:text-emerald-700 transition-colors shadow-2xs"
                      >
                        <MessageSquare className="h-3 w-3 text-emerald-600" />
                        <span>Chat</span>
                      </button>
                      <span className="text-[11px] text-slate-400">
                        {c.createdAt ? new Date(c.createdAt).toLocaleDateString() : 'Active'}
                      </span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* ========================================================================= */}
      {/* MODAL 1: Multi-Channel CSV Import & Preview Modal */}
      {/* ========================================================================= */}
      {showImportModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4">
          <div className="w-full max-w-2xl rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl overflow-hidden flex flex-col max-h-[90vh]">
            <div className="border-b border-[var(--color-border)] p-4 flex items-center justify-between">
              <div>
                <h3 className="text-sm font-bold text-[var(--color-ink)]">Multi-Channel CSV Audience Import</h3>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Import contacts across WhatsApp, Email, SMS, TikTok, Telegram, and Social channels.
                </p>
              </div>
              <button
                onClick={() => {
                  setShowImportModal(false)
                  setImportFile(null)
                  setImportPreview(null)
                }}
                className="text-slate-400 hover:text-slate-700"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="p-5 space-y-4 overflow-y-auto flex-1">
              {/* Step 1: File Dropzone & Default Attributes */}
              <div className="space-y-3">
                <label className="block text-xs font-semibold text-[var(--color-ink)]">Select CSV File</label>
                <div
                  onClick={() => fileRef.current?.click()}
                  className="rounded-lg border-2 border-dashed border-slate-300 p-6 text-center hover:border-slate-400 cursor-pointer bg-slate-50/50 transition-colors"
                >
                  <input
                    ref={fileRef}
                    type="file"
                    accept=".csv"
                    onChange={handleFileSelect}
                    className="hidden"
                  />
                  <Upload className="h-8 w-8 text-slate-400 mx-auto mb-1" />
                  {importFile ? (
                    <div>
                      <div className="text-xs font-bold text-[var(--color-ink)]">{importFile.name}</div>
                      <div className="text-[11px] text-[var(--color-ink-muted)]">
                        {(importFile.size / 1024).toFixed(1)} KB · Click to change file
                      </div>
                    </div>
                  ) : (
                    <div>
                      <div className="text-xs font-semibold text-[var(--color-ink)]">Click or drag CSV file here</div>
                      <div className="text-[11px] text-[var(--color-ink-faint)] mt-0.5">
                        Supported columns: <code>name</code>, <code>channel</code>, <code>value</code>, <code>email</code>, <code>phone</code>, <code>tags</code>
                      </div>
                    </div>
                  )}
                </div>
              </div>

              {/* Default channel & tags overrides */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">
                    Fallback Channel (if column missing)
                  </label>
                  <select
                    value={defaultChannel}
                    onChange={(e) => setDefaultChannel(e.target.value)}
                    className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs bg-[var(--color-surface)]"
                  >
                    <option value="">Auto-Detect from Email/Phone</option>
                    <option value="whatsapp">WhatsApp</option>
                    <option value="email">Email</option>
                    <option value="sms">SMS</option>
                    <option value="tiktok">TikTok</option>
                    <option value="telegram">Telegram</option>
                    <option value="instagram">Instagram</option>
                    <option value="facebook">Facebook</option>
                    <option value="line">LINE</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">
                    Apply Tags to All Imported Contacts
                  </label>
                  <input
                    type="text"
                    placeholder="e.g. spring_campaign, leads_2026"
                    value={defaultTags}
                    onChange={(e) => setDefaultTags(e.target.value)}
                    className="w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs"
                  />
                </div>
              </div>

              {/* Step 2: Live Preview Table */}
              {previewing && (
                <div className="text-center py-4 text-xs text-[var(--color-ink-muted)] flex items-center justify-center gap-2">
                  <div className="h-4 w-4 animate-spin rounded-full border-2 border-slate-300 border-t-slate-600" />
                  <span>Inspecting CSV structure...</span>
                </div>
              )}

              {importPreview && (
                <div className="space-y-2 pt-2 border-t border-[var(--color-border)]">
                  <div className="flex items-center justify-between text-xs font-semibold text-slate-800">
                    <span>Parsed Preview ({importPreview.totalRows} Total Rows)</span>
                    <span className="text-[11px] text-[var(--color-ink-muted)]">
                      Channels: {importPreview.detectedChannels.join(', ') || 'Auto'}
                    </span>
                  </div>

                  <div className="overflow-hidden rounded-lg border border-[var(--color-border)] text-xs">
                    <table className="w-full">
                      <thead className="bg-[var(--color-page)] text-left text-[11px] font-semibold text-[var(--color-ink-muted)] border-b border-[var(--color-border)]">
                        <tr>
                          <th className="px-3 py-1.5">Name</th>
                          <th className="px-3 py-1.5">Channel</th>
                          <th className="px-3 py-1.5">Address / Number</th>
                          <th className="px-3 py-1.5">Tags</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-slate-100 font-mono text-[11px]">
                        {importPreview.sampleRows.map((r, i) => (
                          <tr key={i} className={r.valid ? '' : 'bg-rose-50/50'}>
                            <td className="px-3 py-1.5 font-sans font-medium text-slate-800">
                              {r.name || '—'}
                            </td>
                            <td className="px-3 py-1.5 uppercase font-semibold text-[var(--color-ink)]">
                              {r.channel}
                            </td>
                            <td className="px-3 py-1.5 text-[var(--color-ink-muted)]">{r.value || '[Missing]'}</td>
                            <td className="px-3 py-1.5 text-[var(--color-ink-muted)]">
                              {r.tags.length > 0 ? r.tags.join(', ') : '—'}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>

            <div className="border-t border-[var(--color-border)] p-4 bg-[var(--color-page)] flex items-center justify-between">
              <span className="text-xs text-[var(--color-ink-muted)]">
                Contacts will be automatically opted-in and mapped to unique communication channels.
              </span>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowImportModal(false)}
                  className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
                >
                  Cancel
                </button>
                <button
                  onClick={handleExecuteImport}
                  disabled={!importFile || importing}
                  className="rounded-md bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs disabled:opacity-50"
                >
                  {importing ? 'Importing Contacts...' : 'Confirm & Import Audience'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================================= */}
      {/* MODAL 2: Dynamic Segment Builder Modal */}
      {/* ========================================================================= */}
      {showSegmentModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4">
          <div className="w-full max-w-lg rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-xl overflow-hidden">
            <div className="border-b border-[var(--color-border)] p-4 flex items-center justify-between">
              <div>
                <h3 className="text-sm font-bold text-[var(--color-ink)]">
                  {editingSegment ? 'Edit Dynamic Audience Segment' : 'Create Dynamic Saved Segment'}
                </h3>
                <p className="text-xs text-[var(--color-ink-muted)]">
                  Target customers dynamically using multi-channel filters, tags, and search keywords.
                </p>
              </div>
              <button
                onClick={() => setShowSegmentModal(false)}
                className="text-slate-400 hover:text-slate-700"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="p-5 space-y-4">
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">Segment Name</label>
                <input
                  type="text"
                  placeholder="e.g. VIP Enterprise Customers"
                  value={segmentName}
                  onChange={(e) => setSegmentName(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">Description</label>
                <textarea
                  rows={2}
                  placeholder="Describe the purpose of this target group..."
                  value={segmentDesc}
                  onChange={(e) => setSegmentDesc(e.target.value)}
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
                />
              </div>

              {/* Channels filter */}
              <div>
                <label className="block text-xs font-medium text-[var(--color-ink)] mb-1">
                  Channel Targeting (Leave empty for all channels)
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {CHANNELS.filter((c) => c.id !== 'all').map((ch) => {
                    const IconComp = ch.icon
                    return (
                      <label
                        key={ch.id}
                        className={`flex items-center gap-1.5 rounded-md border p-2 text-xs cursor-pointer transition-all ${
                          segmentChannels.includes(ch.id)
                            ? 'border-slate-900 bg-slate-900 text-white font-semibold'
                            : 'border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:bg-[var(--color-page)]'
                        }`}
                      >
                        <input
                          type="checkbox"
                          checked={segmentChannels.includes(ch.id)}
                          onChange={(e) => {
                            if (e.target.checked) {
                              setSegmentChannels([...segmentChannels, ch.id])
                            } else {
                              setSegmentChannels(segmentChannels.filter((c) => c !== ch.id))
                            }
                          }}
                          className="sr-only"
                        />
                        <IconComp className="h-3.5 w-3.5 shrink-0" />
                        <span className="truncate">{ch.label}</span>
                      </label>
                    )
                  })}
                </div>
              </div>

              {/* Tags Filter */}
              {tags.length > 0 && (
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <label className="text-xs font-medium text-[var(--color-ink)]">Filter by Tags</label>
                    <div className="flex items-center gap-2 text-[11px] text-[var(--color-ink-muted)]">
                      <span>Match:</span>
                      <label className="inline-flex items-center gap-1 cursor-pointer">
                        <input
                          type="radio"
                          name="tagMode"
                          value="any"
                          checked={segmentTagMode === 'any'}
                          onChange={() => setSegmentTagMode('any')}
                        />
                        <span>Any tag</span>
                      </label>
                      <label className="inline-flex items-center gap-1 cursor-pointer">
                        <input
                          type="radio"
                          name="tagMode"
                          value="all"
                          checked={segmentTagMode === 'all'}
                          onChange={() => setSegmentTagMode('all')}
                        />
                        <span>All tags</span>
                      </label>
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-1.5 max-h-28 overflow-y-auto p-2 rounded-md border border-[var(--color-border)] bg-[var(--color-page)]">
                    {tags.map((t) => (
                      <button
                        key={t}
                        type="button"
                        onClick={() => {
                          if (segmentTags.includes(t)) {
                            setSegmentTags(segmentTags.filter((x) => x !== t))
                          } else {
                            setSegmentTags([...segmentTags, t])
                          }
                        }}
                        className={`rounded-full px-2 py-0.5 text-[11px] font-medium transition-all ${
                          segmentTags.includes(t)
                            ? 'bg-slate-900 text-white'
                            : 'bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-ink)] hover:bg-slate-100'
                        }`}
                      >
                        #{t}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="border-t border-[var(--color-border)] p-4 bg-[var(--color-page)] flex items-center justify-end gap-2">
              <button
                onClick={() => setShowSegmentModal(false)}
                className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] hover:bg-[var(--color-page)]"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveSegment}
                disabled={savingSegment}
                className="rounded-md bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white hover:bg-slate-800 shadow-xs"
              >
                {savingSegment ? 'Saving...' : editingSegment ? 'Update Segment' : 'Create Segment'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
