import {
  Send,
  Users,
  Plus,
  Trash2,
  Edit3,
  Eye,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Search,
  Image,
  Video,
  Link,
  Settings,
  Clock,
  Play,
  X,
  Check,
  ChevronDown,
  Info,
  Sparkles,
  Globe,
  Phone,
  Mail,
  MessageSquare,
  BarChart3,
  Activity,
  ListTodo,
  Calendar,
  FileText,
  Copy,
  Download,
  ExternalLink,
  History,
  Inbox,
  Key,
  KeyRound,
  Hash,
  Home,
  PauseCircle,
  Zap,
  BarChart2,
  BookOpen,
  Camera,
  CheckCheck,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  CircleDot,
  ClipboardList,
  Code,
  Edit2,
  FileText as FileTextIcon,
  Handshake,
  Layers,
  List as Split,
} from '../components/icons'
import { useState, useEffect } from 'react'
import {
  getBroadcastSummary,
  listBroadcasts,
  getBroadcast,
  createBroadcast,
  updateBroadcast,
  deleteBroadcast,
  updateBroadcastStatus,
  sendBroadcast,
  getFormatPreview,
  getAvailableChannels,
  uploadBroadcastMedia,
  listBroadcastMedia,
  deleteBroadcastMedia,
  listBroadcastChannels,
  listBroadcastRecipients,
  listBroadcastLogs,
  getBroadcastSettings,
  updateBroadcastSettings,
  previewHtmlForChannels,
} from '../lib/api'
import type {
  BroadcastItem,
  BroadcastSummary,
  BroadcastChannelCount,
  ChannelFormatItem,
  BroadcastMediaItem,
  BroadcastChannelItem,
  BroadcastRecipientItem,
  BroadcastLogItem,
  BroadcastSettingsItem,
  SendBroadcastResult,
  ChannelFormatPreview,
} from '../lib/api'

const CHANNEL_ICONS: Record<string, any> = {
  whatsapp: MessageSquare,
  email: Mail,
  sms: Phone,
  instagram: Camera,
  facebook: Globe,
  telegram: Zap,
  discord: MessageSquare,
  slack: Hash,
}

const CHANNEL_COLORS: Record<string, string> = {
  whatsapp: '#25D366',
  email: '#EA4335',
  sms: '#4285F4',
  instagram: '#E4405F',
  facebook: '#1877F2',
  telegram: '#0088CC',
  discord: '#5865F2',
  slack: '#611F69',
}

export function BroadcastDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'broadcasts' | 'create' | 'channels' | 'settings'>('overview')
  const [loading, setLoading] = useState(true)
  const [summary, setSummary] = useState<BroadcastSummary | null>(null)
  const [broadcasts, setBroadcasts] = useState<BroadcastItem[]>([])
  const [availableChannels, setAvailableChannels] = useState<ChannelFormatItem[]>([])
  const [settings, setSettings] = useState<BroadcastSettingsItem | null>(null)
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null)

  // Create broadcast form
  const [title, setTitle] = useState('')
  const [bodyText, setBodyText] = useState('')
  const [bodyHtml, setBodyHtml] = useState('')
  const [subjectLine, setSubjectLine] = useState('')
  const [previewText, setPreviewText] = useState('')
  const [mediaType, setMediaType] = useState('text')
  const [primaryMediaUrl, setPrimaryMediaUrl] = useState('')
  const [mediaCaption, setMediaCaption] = useState('')
  const [linkUrl, setLinkUrl] = useState('')
  const [linkTitle, setLinkTitle] = useState('')
  const [linkDescription, setLinkDescription] = useState('')
  const [linkImageUrl, setLinkImageUrl] = useState('')
  const [targetType, setTargetType] = useState('contacts')
  const [targetIds, setTargetIds] = useState('')
  const [selectedChannels, setSelectedChannels] = useState<string[]>([])
  const [scheduledAt, setScheduledAt] = useState('')
  const [sending, setSending] = useState(false)
  const [creating, setCreating] = useState(false)

  // Campaign type selection
  const [campaignType, setCampaignType] = useState<'omni' | 'separate' | null>(null)
  const [singleChannel, setSingleChannel] = useState('')
  
  // HTML-to-all-channels live preview
  const [htmlPreviewFormats, setHtmlPreviewFormats] = useState<ChannelFormatPreview[]>([])
  const [htmlPreviewLoading, setHtmlPreviewLoading] = useState(false)
  const [showHtmlPreview, setShowHtmlPreview] = useState(false)
  
  // Channel-by-Channel: per-channel message composer state
  const [channelMessages, setChannelMessages] = useState<Record<string, {
    bodyText: string
    bodyHtml: string
    subjectLine: string
    previewText: string
    mediaType: string
    primaryMediaUrl: string
    mediaCaption: string
    linkUrl: string
    linkTitle: string
    linkDescription: string
    linkImageUrl: string
    targetType: string
    targetIds: string
    scheduledAt: string
  }>>({})
  const [activeChannelTab, setActiveChannelTab] = useState<string>('')
  const [multiChannels, setMultiChannels] = useState<string[]>([])

  // Detail view
  const [viewingBroadcast, setViewingBroadcast] = useState<BroadcastItem | null>(null)
  const [broadcastChannels, setBroadcastChannels] = useState<BroadcastChannelItem[]>([])
  const [broadcastRecipients, setBroadcastRecipients] = useState<BroadcastRecipientItem[]>([])
  const [broadcastLogs, setBroadcastLogs] = useState<BroadcastLogItem[]>([])
  const [broadcastMedia, setBroadcastMedia] = useState<BroadcastMediaItem[]>([])
  const [formatPreview, setFormatPreview] = useState<ChannelFormatItem[]>([])

  // Filter
  const [filterStatus, setFilterStatus] = useState('')
  const [searchQuery, setSearchQuery] = useState('')

  // Media upload
  const [mediaFileName, setMediaFileName] = useState('')
  const [mediaFileUrl, setMediaFileUrl] = useState('')
  const [mediaMimeType, setMediaMimeType] = useState('image/jpeg')

  const clearFeedback = () => setFeedback(null)

  const loadSummary = async () => {
    try {
      const data = await getBroadcastSummary()
      setSummary(data)
    } catch (e: any) {
      console.error('Failed to load broadcast summary:', e)
    }
  }

  const loadBroadcasts = async () => {
    try {
      const data = await listBroadcasts(filterStatus || undefined, 50)
      setBroadcasts(data)
    } catch (e: any) {
      console.error('Failed to load broadcasts:', e)
    }
  }

  const loadChannels = async () => {
    try {
      const data = await getAvailableChannels()
      setAvailableChannels(data)
    } catch (e: any) {
      console.error('Failed to load channels:', e)
    }
  }

  const loadSettings = async () => {
    try {
      const data = await getBroadcastSettings()
      setSettings(data)
    } catch (e: any) {
      console.error('Failed to load settings:', e)
    }
  }

  const loadData = async () => {
    setLoading(true)
    await Promise.all([loadSummary(), loadBroadcasts(), loadChannels(), loadSettings()])
    setLoading(false)
  }

  useEffect(() => {
    loadData()
  }, [])

  useEffect(() => {
    loadBroadcasts()
  }, [filterStatus])

  // Channel toggle
  const toggleChannel = (channel: string) => {
    setSelectedChannels(prev =>
      prev.includes(channel) ? prev.filter(c => c !== channel) : [...prev, channel]
    )
  }

  // Initialize channel message state
  const getChannelMessage = (channel: string) => {
    if (!channelMessages[channel]) {
      const defaultMsg = {
        bodyText: '',
        bodyHtml: '',
        subjectLine: '',
        previewText: '',
        mediaType: 'text',
        primaryMediaUrl: '',
        mediaCaption: '',
        linkUrl: '',
        linkTitle: '',
        linkDescription: '',
        linkImageUrl: '',
        targetType: 'contacts',
        targetIds: '',
        scheduledAt: '',
      }
      setChannelMessages(prev => ({ ...prev, [channel]: defaultMsg }))
      return defaultMsg
    }
    return channelMessages[channel]
  }

  const updateChannelMessage = (channel: string, field: string, value: string) => {
    setChannelMessages(prev => ({
      ...prev,
      [channel]: { ...prev[channel] || getChannelMessage(channel), [field]: value }
    }))
  }

  // Toggle channel for multi-channel campaign
  const toggleMultiChannel = (channel: string) => {
    setMultiChannels(prev => {
      const next = prev.includes(channel)
        ? prev.filter(c => c !== channel)
        : [...prev, channel]
      // Initialize message state for newly added channel
      if (!prev.includes(channel)) {
        getChannelMessage(channel)
      }
      // Set active tab to the newly added channel
      if (!prev.includes(channel) && next.length > 0) {
        setActiveChannelTab(channel)
      }
      // If removing active tab, switch to first available
      if (prev.includes(channel) && activeChannelTab === channel) {
        setActiveChannelTab(next[0] || '')
      }
      return next
    })
  }

  // Create broadcast
  const handleCreate = async () => {
    setCreating(true)
    try {
      if (campaignType === 'omni') {
        // Omni-Channel: single message to all channels
        if (!title.trim() || !bodyText.trim()) {
          setFeedback({ type: 'error', message: 'Title and body text are required.' })
          setCreating(false)
          return
        }
        if (selectedChannels.length === 0) {
          setFeedback({ type: 'error', message: 'Select at least one channel for omni-channel broadcast.' })
          setCreating(false)
          return
        }
        const parsedTargetIds: string[] = []
        try {
          const parsed = JSON.parse(targetIds)
          if (Array.isArray(parsed)) parsedTargetIds.push(...parsed)
        } catch {
          parsedTargetIds.push(...targetIds.split(',').map(s => s.trim()).filter(Boolean))
        }

        const broadcast = await createBroadcast({
          title: title.trim(),
          bodyText: bodyText.trim(),
          bodyHtml: bodyHtml.trim() || undefined,
          subjectLine: subjectLine.trim() || undefined,
          previewText: previewText.trim() || undefined,
          mediaType,
          primaryMediaUrl: primaryMediaUrl.trim() || undefined,
          mediaCaption: mediaCaption.trim() || undefined,
          linkUrl: linkUrl.trim() || undefined,
          linkTitle: linkTitle.trim() || undefined,
          linkDescription: linkDescription.trim() || undefined,
          linkImageUrl: linkImageUrl.trim() || undefined,
          targetType,
          targetIds: parsedTargetIds,
          channels: selectedChannels,
          scheduledAt: scheduledAt || undefined,
        })

        setFeedback({ type: 'success', message: `Omni-Channel broadcast "${broadcast.title}" created for ${selectedChannels.length} channels!` })
      } else if (campaignType === 'separate') {
        // Channel-by-Channel: create separate broadcast per channel
        if (multiChannels.length === 0) {
          setFeedback({ type: 'error', message: 'Select at least one channel for channel-by-channel campaign.' })
          setCreating(false)
          return
        }
        if (!title.trim()) {
          setFeedback({ type: 'error', message: 'Campaign title is required.' })
          setCreating(false)
          return
        }

        let createdCount = 0
        let errors = 0
        for (const channel of multiChannels) {
          const msg = channelMessages[channel]
          if (!msg || !msg.bodyText.trim()) {
            errors++
            continue
          }
          const parsedTargetIds: string[] = []
          try {
            const parsed = JSON.parse(msg.targetIds)
            if (Array.isArray(parsed)) parsedTargetIds.push(...parsed)
          } catch {
            parsedTargetIds.push(...msg.targetIds.split(',').map(s => s.trim()).filter(Boolean))
          }

          try {
            await createBroadcast({
              title: `${title} (${channel})`,
              bodyText: msg.bodyText.trim(),
              bodyHtml: msg.bodyHtml.trim() || undefined,
              subjectLine: msg.subjectLine.trim() || undefined,
              previewText: msg.previewText.trim() || undefined,
              mediaType: msg.mediaType,
              primaryMediaUrl: msg.primaryMediaUrl.trim() || undefined,
              mediaCaption: msg.mediaCaption.trim() || undefined,
              linkUrl: msg.linkUrl.trim() || undefined,
              linkTitle: msg.linkTitle.trim() || undefined,
              linkDescription: msg.linkDescription.trim() || undefined,
              linkImageUrl: msg.linkImageUrl.trim() || undefined,
              targetType: msg.targetType,
              targetIds: parsedTargetIds,
              channels: [channel],
              scheduledAt: msg.scheduledAt || undefined,
            })
            createdCount++
          } catch (e: any) {
            console.error(`Failed to create broadcast for ${channel}:`, e)
            errors++
          }
        }

        if (createdCount > 0) {
          setFeedback({ type: 'success', message: `Created ${createdCount} channel-specific broadcasts${errors > 0 ? ` (${errors} skipped — missing body text)` : ''}!` })
        } else {
          setFeedback({ type: 'error', message: 'Failed to create any broadcasts. Each channel needs body text.' })
        }
      }

      resetForm()
      loadBroadcasts()
      loadSummary()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to create broadcast.' })
    } finally {
      setCreating(false)
    }
  }

  const resetForm = () => {
    setTitle('')
    setBodyText('')
    setBodyHtml('')
    setSubjectLine('')
    setPreviewText('')
    setMediaType('text')
    setPrimaryMediaUrl('')
    setMediaCaption('')
    setLinkUrl('')
    setLinkTitle('')
    setLinkDescription('')
    setLinkImageUrl('')
    setTargetType('contacts')
    setTargetIds('')
    setSelectedChannels([])
    setScheduledAt('')
    setCampaignType(null)
    setSingleChannel('')
    setMultiChannels([])
    setChannelMessages({})
    setActiveChannelTab('')
  }

  // Send broadcast
  const handleSend = async (id: string) => {
    setSending(true)
    try {
      const result: SendBroadcastResult = await sendBroadcast(id)
      setFeedback({ type: 'success', message: result.message })
      loadBroadcasts()
      loadSummary()
      if (viewingBroadcast) {
        setViewingBroadcast(null)
      }
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to send broadcast.' })
    } finally {
      setSending(false)
    }
  }

  // Delete broadcast
  const handleDelete = async (id: string) => {
    if (!confirm('Delete this broadcast?')) return
    try {
      await deleteBroadcast(id)
      setFeedback({ type: 'success', message: 'Broadcast deleted.' })
      loadBroadcasts()
      loadSummary()
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to delete.' })
    }
  }

  // View broadcast detail
  const handleView = async (broadcast: BroadcastItem) => {
    setViewingBroadcast(broadcast)
    try {
      const [channels, recipients, logs, media] = await Promise.all([
        listBroadcastChannels(broadcast.id),
        listBroadcastRecipients(broadcast.id, undefined, 50),
        listBroadcastLogs(broadcast.id, 50),
        listBroadcastMedia(broadcast.id),
      ])
      setBroadcastChannels(channels)
      setBroadcastRecipients(recipients)
      setBroadcastLogs(logs)
      setBroadcastMedia(media)
    } catch (e: any) {
      console.error('Failed to load broadcast detail:', e)
    }
  }

  // Format preview
  const handleFormatPreview = async (id: string) => {
    try {
      const preview = await getFormatPreview(id)
      setFormatPreview(preview.formats)
    } catch (e: any) {
      console.error('Failed to get format preview:', e)
    }
  }

  // Upload media
  const handleUploadMedia = async (broadcastId: string) => {
    if (!mediaFileUrl.trim()) {
      setFeedback({ type: 'error', message: 'Media URL is required.' })
      return
    }
    try {
      await uploadBroadcastMedia({
        broadcastId,
        mediaType,
        fileName: mediaFileName || 'media',
        fileSizeBytes: 0,
        mimeType: mediaMimeType,
        fileUrl: mediaFileUrl.trim(),
      })
      setFeedback({ type: 'success', message: 'Media uploaded.' })
      setMediaFileName('')
      setMediaFileUrl('')
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to upload media.' })
    }
  }

  // Update settings
  const handleUpdateSettings = async (updates: Partial<BroadcastSettingsItem>) => {
    try {
      const updated = await updateBroadcastSettings(updates)
      setSettings(updated)
      setFeedback({ type: 'success', message: 'Settings updated.' })
    } catch (e: any) {
      setFeedback({ type: 'error', message: e.message || 'Failed to update settings.' })
    }
  }

  const statusBadge = (status: string) => {
    const colors: Record<string, string> = {
      draft: '#6B7280',
      scheduled: '#8B5CF6',
      sending: '#F59E0B',
      sent: '#10B981',
      failed: '#EF4444',
      paused: '#F59E0B',
      cancelled: '#6B7280',
    }
    const color = colors[status] || '#6B7280'
    return (
      <span style={{
        background: color + '20',
        color,
        padding: '2px 10px',
        borderRadius: '9999px',
        fontSize: '12px',
        fontWeight: 600,
        textTransform: 'capitalize',
      }}>
        {status}
      </span>
    )
  }

  if (loading) {
    return <div className="p-8 text-center text-gray-400">Loading Broadcast System...</div>
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white flex items-center gap-3">
            <Send className="w-7 h-7 text-blue-400" />
            Omni-Channel Broadcast
          </h1>
          <p className="text-gray-400 mt-1">
            Design and send rich media broadcasts across all channels in one click
          </p>
        </div>
        <button
          onClick={() => { setActiveTab('create'); setViewingBroadcast(null); setFormatPreview([]) }}
          className="flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors"
        >
          <Plus className="w-4 h-4" />
          New Broadcast
        </button>
      </div>

      {/* Feedback */}
      {feedback && (
        <div className={`mb-4 p-4 rounded-lg flex items-center gap-3 ${
          feedback.type === 'success' ? 'bg-green-900/30 border border-green-800 text-green-300' : 'bg-red-900/30 border border-red-800 text-red-300'
        }`}>
          {feedback.type === 'success' ? <CheckCircle2 className="w-5 h-5" /> : <AlertCircle className="w-5 h-5" />}
          <span className="flex-1">{feedback.message}</span>
          <button onClick={clearFeedback}><X className="w-4 h-4" /></button>
        </div>
      )}

      {/* Detail View */}
      {viewingBroadcast ? (
        <div className="space-y-6">
          <button
            onClick={() => { setViewingBroadcast(null); setFormatPreview([]) }}
            className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
            Back to Broadcasts
          </button>

          {/* Broadcast Header */}
          <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
            <div className="flex items-start justify-between mb-4">
              <div>
                <h2 className="text-xl font-bold text-white">{viewingBroadcast.title}</h2>
                <div className="flex items-center gap-3 mt-2">
                  {statusBadge(viewingBroadcast.status)}
                  <span className="text-gray-400 text-sm">
                    {viewingBroadcast.totalRecipients} recipients · {viewingBroadcast.sentCount} sent
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {viewingBroadcast.status === 'draft' && (
                  <button
                    onClick={() => handleSend(viewingBroadcast.id)}
                    disabled={sending}
                    className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
                  >
                    <Send className="w-4 h-4" />
                    {sending ? 'Sending...' : 'Send Now'}
                  </button>
                )}
                <button
                  onClick={() => handleFormatPreview(viewingBroadcast.id)}
                  className="flex items-center gap-2 px-3 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
                >
                  <Eye className="w-4 h-4" />
                  Preview
                </button>
                <button
                  onClick={() => handleDelete(viewingBroadcast.id)}
                  className="flex items-center gap-2 px-3 py-2 bg-red-900/50 hover:bg-red-800 text-red-300 rounded-lg transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Content Preview */}
            <div className="bg-gray-900/50 rounded-lg p-4 mb-4">
              <p className="text-gray-300 whitespace-pre-wrap">{viewingBroadcast.bodyText}</p>
              {viewingBroadcast.primaryMediaUrl && (
                <div className="mt-3">
                  {viewingBroadcast.mediaType === 'image' && (
                    <img src={viewingBroadcast.primaryMediaUrl} alt="Media" className="max-h-64 rounded-lg" />
                  )}
                  {viewingBroadcast.mediaType === 'video' && (
                    <video src={viewingBroadcast.primaryMediaUrl} controls className="max-h-64 rounded-lg" />
                  )}
                </div>
              )}
              {viewingBroadcast.mediaCaption && (
                <p className="text-gray-400 text-sm mt-2 italic">{viewingBroadcast.mediaCaption}</p>
              )}
              {viewingBroadcast.linkUrl && (
                <div className="mt-3 p-3 bg-blue-900/20 border border-blue-800 rounded-lg">
                  <p className="text-blue-300 font-medium">{viewingBroadcast.linkTitle || viewingBroadcast.linkUrl}</p>
                  {viewingBroadcast.linkDescription && <p className="text-blue-400 text-sm">{viewingBroadcast.linkDescription}</p>}
                  <p className="text-blue-500 text-xs mt-1">{viewingBroadcast.linkUrl}</p>
                </div>
              )}
            </div>

            {/* Channel Badges */}
            <div className="flex flex-wrap gap-2">
              {viewingBroadcast.channels.map((ch: string) => {
                const Icon = CHANNEL_ICONS[ch] || Globe
                const color = CHANNEL_COLORS[ch] || '#6B7280'
                return (
                  <span key={ch} className="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium" style={{ background: color + '20', color }}>
                    <Icon className="w-3.5 h-3.5" />
                    {ch.charAt(0).toUpperCase() + ch.slice(1)}
                  </span>
                )
              })}
            </div>
          </div>

          {/* Format Preview */}
          {formatPreview.length > 0 && (
            <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
              <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                <Eye className="w-5 h-5 text-blue-400" />
                Per-Channel Format Preview
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {formatPreview.map(fmt => (
                  <div key={fmt.channelCode} className="bg-gray-900/50 rounded-lg p-4 border border-gray-700">
                    <div className="flex items-center gap-2 mb-3">
                      {(() => {
                        const Icon = CHANNEL_ICONS[fmt.channelCode] || Globe
                        const color = CHANNEL_COLORS[fmt.channelCode] || '#6B7280'
                        return <Icon className="w-4 h-4" style={{ color }} />
                      })()}
                      <span className="font-semibold text-white">{fmt.channelName}</span>
                      <span className="text-gray-500 text-xs ml-auto">Max {fmt.maxLength} chars</span>
                    </div>
                    <div className="bg-gray-800 rounded p-3 text-sm text-gray-300 whitespace-pre-wrap max-h-40 overflow-y-auto">
                      {fmt.formattedBody}
                    </div>
                    <div className="flex flex-wrap gap-1.5 mt-3">
                      {fmt.supportsHtml && <span className="text-xs px-2 py-0.5 bg-green-900/30 text-green-400 rounded">HTML</span>}
                      {fmt.supportsImages && <span className="text-xs px-2 py-0.5 bg-blue-900/30 text-blue-400 rounded">Images</span>}
                      {fmt.supportsVideo && <span className="text-xs px-2 py-0.5 bg-purple-900/30 text-purple-400 rounded">Video</span>}
                      {fmt.supportsLinks && <span className="text-xs px-2 py-0.5 bg-yellow-900/30 text-yellow-400 rounded">Links</span>}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Channel Status */}
          {broadcastChannels.length > 0 && (
            <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
              <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                <Globe className="w-5 h-5 text-blue-400" />
                Channel Delivery Status
              </h3>
              <div className="space-y-3">
                {broadcastChannels.map(ch => (
                  <div key={ch.id} className="flex items-center justify-between p-3 bg-gray-900/50 rounded-lg">
                    <div className="flex items-center gap-3">
                      {(() => {
                        const Icon = CHANNEL_ICONS[ch.channelCode] || Globe
                        const color = CHANNEL_COLORS[ch.channelCode] || '#6B7280'
                        return <Icon className="w-5 h-5" style={{ color }} />
                      })()}
                      <div>
                        <span className="text-white font-medium">{ch.channelName}</span>
                        <span className="text-gray-400 text-sm ml-3">
                          {ch.sentCount}/{ch.recipientCount} sent
                        </span>
                      </div>
                    </div>
                    {statusBadge(ch.status)}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recipients */}
          {broadcastRecipients.length > 0 && (
            <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
              <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                <Users className="w-5 h-5 text-blue-400" />
                Recipients ({broadcastRecipients.length})
              </h3>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-gray-400 border-b border-gray-700">
                      <th className="text-left py-2 px-3">Contact</th>
                      <th className="text-left py-2 px-3">Channel</th>
                      <th className="text-left py-2 px-3">Status</th>
                      <th className="text-left py-2 px-3">Sent At</th>
                    </tr>
                  </thead>
                  <tbody>
                    {broadcastRecipients.slice(0, 20).map(r => (
                      <tr key={r.id} className="border-b border-gray-800">
                        <td className="py-2 px-3 text-white">{r.contactId.slice(0, 8)}...</td>
                        <td className="py-2 px-3 text-gray-300">{r.channelCode}</td>
                        <td className="py-2 px-3">{statusBadge(r.status)}</td>
                        <td className="py-2 px-3 text-gray-400 text-xs">{r.sentAt ? new Date(r.sentAt).toLocaleString() : '-'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Logs */}
          {broadcastLogs.length > 0 && (
            <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
              <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                <History className="w-5 h-5 text-blue-400" />
                Activity Log
              </h3>
              <div className="space-y-2">
                {broadcastLogs.slice(0, 15).map(log => (
                  <div key={log.id} className="flex items-center gap-3 p-2 bg-gray-900/30 rounded">
                    <CircleDot className="w-3 h-3 text-gray-500" />
                    <span className="text-white text-sm font-medium">{log.eventType}</span>
                    {log.channelCode && <span className="text-gray-400 text-xs">{log.channelCode}</span>}
                    <span className="text-gray-500 text-xs ml-auto">{new Date(log.createdAt).toLocaleString()}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      ) : (
        /* Tabs */
        <>
          <div className="flex gap-1 mb-6 bg-gray-800/50 rounded-lg p-1">
            {([
              { key: 'overview', label: 'Overview', icon: BarChart3 },
              { key: 'broadcasts', label: 'Broadcasts', icon: ListTodo },
              { key: 'create', label: 'New Broadcast', icon: Plus },
              { key: 'channels', label: 'Channels', icon: Globe },
              { key: 'settings', label: 'Settings', icon: Settings },
            ] as const).map(tab => {
              const Icon = tab.icon
              return (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key)}
                  className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                    activeTab === tab.key
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-400 hover:text-white hover:bg-gray-700/50'
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
              {/* Summary Cards */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                {[
                  { label: 'Total Broadcasts', value: summary?.totalBroadcasts || 0, icon: Send, color: 'text-blue-400' },
                  { label: 'Drafts', value: summary?.draftBroadcasts || 0, icon: FileTextIcon, color: 'text-gray-400' },
                  { label: 'Sent', value: summary?.sentBroadcasts || 0, icon: CheckCircle2, color: 'text-green-400' },
                  { label: 'Failed', value: summary?.failedBroadcasts || 0, icon: XCircle, color: 'text-red-400' },
                ].map(card => (
                  <div key={card.label} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5">
                    <div className="flex items-center justify-between mb-3">
                      <span className="text-gray-400 text-sm">{card.label}</span>
                      <card.icon className={`w-5 h-5 ${card.color}`} />
                    </div>
                    <p className="text-3xl font-bold text-white">{card.value}</p>
                  </div>
                ))}
              </div>

              {/* Analytics Row */}
              <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                {[
                  { label: 'Recipients', value: summary?.totalRecipientsSent || 0 },
                  { label: 'Delivered', value: summary?.totalDelivered || 0 },
                  { label: 'Read', value: summary?.totalRead || 0 },
                  { label: 'Clicked', value: summary?.totalClicked || 0 },
                  { label: 'Replied', value: summary?.totalReplied || 0 },
                ].map(card => (
                  <div key={card.label} className="bg-gray-800/50 border border-gray-700 rounded-xl p-4 text-center">
                    <p className="text-2xl font-bold text-white">{card.value}</p>
                    <p className="text-gray-400 text-xs mt-1">{card.label}</p>
                  </div>
                ))}
              </div>

              {/* By Channel */}
              {summary?.byChannel && summary.byChannel.length > 0 && (
                <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                  <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                    <Globe className="w-5 h-5 text-blue-400" />
                    Broadcasts by Channel
                  </h3>
                  <div className="space-y-3">
                    {summary.byChannel.map(ch => {
                      const Icon = CHANNEL_ICONS[ch.channelCode] || Globe
                      const color = CHANNEL_COLORS[ch.channelCode] || '#6B7280'
                      return (
                        <div key={ch.channelCode} className="flex items-center gap-4">
                          <Icon className="w-5 h-5" style={{ color }} />
                          <span className="text-white w-24">{ch.channelName}</span>
                          <div className="flex-1 bg-gray-700 rounded-full h-3 overflow-hidden">
                            <div
                              className="h-full rounded-full transition-all"
                              style={{
                                width: `${Math.min(100, (ch.recipientCount / Math.max(1, summary.totalRecipientsSent)) * 100)}%`,
                                background: color,
                              }}
                            />
                          </div>
                          <span className="text-gray-300 text-sm w-20 text-right">
                            {ch.recipientCount} recipients
                          </span>
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

              {/* Recent Broadcasts */}
              {summary?.recentBroadcasts && summary.recentBroadcasts.length > 0 && (
                <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                  <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                    <Clock className="w-5 h-5 text-blue-400" />
                    Recent Broadcasts
                  </h3>
                  <div className="space-y-3">
                    {summary.recentBroadcasts.slice(0, 5).map(b => (
                      <div
                        key={b.id}
                        onClick={() => handleView(b)}
                        className="flex items-center justify-between p-3 bg-gray-900/50 rounded-lg cursor-pointer hover:bg-gray-900 transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          {statusBadge(b.status)}
                          <span className="text-white font-medium">{b.title}</span>
                          <span className="text-gray-400 text-sm">
                            {b.channels.length} channels · {b.totalRecipients} recipients
                          </span>
                        </div>
                        <span className="text-gray-500 text-xs">{new Date(b.createdAt).toLocaleDateString()}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* How It Works */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Info className="w-5 h-5 text-blue-400" />
                  How Omni-Channel Broadcast Works
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                  {[
                    { step: '1', title: 'Compose', desc: 'Write your message with text, images, videos, and link previews' },
                    { step: '2', title: 'Select Channels', desc: 'Choose which channels to send to (WhatsApp, Email, SMS, etc.)' },
                    { step: '3', title: 'Target Audience', desc: 'Select contacts or segments to receive the broadcast' },
                    { step: '4', title: 'One-Click Send', desc: 'System auto-formats per channel and sends to all at once' },
                  ].map(item => (
                    <div key={item.step} className="bg-gray-900/50 rounded-lg p-4">
                      <div className="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center text-white font-bold mb-3">
                        {item.step}
                      </div>
                      <h4 className="text-white font-semibold mb-1">{item.title}</h4>
                      <p className="text-gray-400 text-sm">{item.desc}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Broadcasts Tab */}
          {activeTab === 'broadcasts' && (
            <div className="space-y-4">
              <div className="flex items-center gap-3">
                <select
                  value={filterStatus}
                  onChange={e => setFilterStatus(e.target.value)}
                  className="bg-gray-800 border border-gray-700 text-white rounded-lg px-3 py-2 text-sm"
                >
                  <option value="">All Status</option>
                  <option value="draft">Draft</option>
                  <option value="scheduled">Scheduled</option>
                  <option value="sent">Sent</option>
                  <option value="failed">Failed</option>
                </select>
              </div>

              {broadcasts.length === 0 ? (
                <div className="text-center py-16 bg-gray-800/30 rounded-xl border border-gray-700">
                  <Send className="w-12 h-12 text-gray-600 mx-auto mb-4" />
                  <p className="text-gray-400 text-lg">No broadcasts yet</p>
                  <p className="text-gray-500 text-sm mt-1">Create your first omni-channel broadcast</p>
                  <button
                    onClick={() => setActiveTab('create')}
                    className="mt-4 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors"
                  >
                    Create Broadcast
                  </button>
                </div>
              ) : (
                <div className="space-y-3">
                  {broadcasts.map(b => (
                    <div key={b.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-3 mb-2">
                            {statusBadge(b.status)}
                            <h3 className="text-white font-semibold">{b.title}</h3>
                          </div>
                          <p className="text-gray-400 text-sm line-clamp-2 mb-3">{b.bodyText}</p>
                          <div className="flex flex-wrap items-center gap-3 text-xs text-gray-500">
                            <span className="flex items-center gap-1">
                              <Globe className="w-3.5 h-3.5" />
                              {b.channels.length} channels
                            </span>
                            <span className="flex items-center gap-1">
                              <Users className="w-3.5 h-3.5" />
                              {b.totalRecipients} recipients
                            </span>
                            <span className="flex items-center gap-1">
                              <Send className="w-3.5 h-3.5" />
                              {b.sentCount} sent
                            </span>
                            {b.primaryMediaUrl && (
                              <span className="flex items-center gap-1">
                                <Image className="w-3.5 h-3.5" />
                                Media
                              </span>
                            )}
                            <span className="ml-auto">{new Date(b.createdAt).toLocaleDateString()}</span>
                          </div>
                        </div>
                        <div className="flex items-center gap-2 ml-4">
                          <button
                            onClick={() => handleView(b)}
                            className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg transition-colors"
                          >
                            <Eye className="w-4 h-4" />
                          </button>
                          {b.status === 'draft' && (
                            <button
                              onClick={() => handleSend(b.id)}
                              disabled={sending}
                              className="p-2 text-green-400 hover:text-green-300 hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
                            >
                              <Send className="w-4 h-4" />
                            </button>
                          )}
                          <button
                            onClick={() => handleDelete(b.id)}
                            className="p-2 text-red-400 hover:text-red-300 hover:bg-gray-700 rounded-lg transition-colors"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Create Broadcast Tab */}
          {activeTab === 'create' && (
            <div className="space-y-6">
              {/* Campaign Type Selection */}
              {!campaignType ? (
                <div className="space-y-6">
                  <div className="text-center mb-8">
                    <h2 className="text-2xl font-bold text-white mb-2">Start a New Campaign</h2>
                    <p className="text-gray-400 text-lg">Choose how you want to send your broadcast</p>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl mx-auto">
                    {/* Omni-Channel Option */}
                    <button
                      onClick={() => {
                        setCampaignType('omni')
                        setSelectedChannels([])
                      }}
                      className="group relative bg-gradient-to-br from-blue-900/40 to-purple-900/40 border-2 border-blue-700/50 hover:border-blue-500 rounded-2xl p-8 text-left transition-all hover:scale-[1.02] hover:shadow-lg hover:shadow-blue-900/20"
                    >
                      <div className="flex items-center gap-4 mb-6">
                        <div className="w-16 h-16 rounded-xl bg-blue-600/20 flex items-center justify-center">
                          <Layers className="w-9 h-9 text-blue-400" />
                        </div>
                        <div>
                          <h3 className="text-xl font-bold text-white">Omni-Channel Campaign</h3>
                          <p className="text-blue-300 text-sm">One message, all channels</p>
                        </div>
                      </div>

                      <p className="text-gray-300 mb-6 leading-relaxed">
                        Write your message <strong className="text-white">once</strong> with text, images, videos, and link previews. The system automatically formats and sends it to <strong className="text-white">all selected channels</strong> in one click.
                      </p>

                      <div className="space-y-3">
                        {[{
                          icon: Send,
                          text: 'Send to multiple channels at once'
                        }, {
                          icon: Sparkles,
                          text: 'Auto-formats per channel capabilities'
                        }, {
                          icon: Users,
                          text: 'Same audience across all channels'
                        }].map((feat, i) => (
                          <div key={i} className="flex items-center gap-3 text-gray-300">
                            <div className="w-8 h-8 rounded-lg bg-blue-600/20 flex items-center justify-center flex-shrink-0">
                              <feat.icon className="w-4 h-4 text-blue-400" />
                            </div>
                            <span className="text-sm">{feat.text}</span>
                          </div>
                        ))}
                      </div>

                      <div className="mt-6 flex items-center gap-2 text-blue-400 font-medium group-hover:text-blue-300 transition-colors">
                        <span>Get Started</span>
                        <ChevronRight className="w-5 h-5" />
                      </div>
                    </button>

                    {/* Separate Channel Option */}
                    <button
                      onClick={() => setCampaignType('separate')}
                      className="group relative bg-gradient-to-br from-emerald-900/40 to-teal-900/40 border-2 border-emerald-700/50 hover:border-emerald-500 rounded-2xl p-8 text-left transition-all hover:scale-[1.02] hover:shadow-lg hover:shadow-emerald-900/20"
                    >
                      <div className="flex items-center gap-4 mb-6">
                        <div className="w-16 h-16 rounded-xl bg-emerald-600/20 flex items-center justify-center">
                          <Split className="w-9 h-9 text-emerald-400" />
                        </div>
                        <div>
                          <h3 className="text-xl font-bold text-white">Channel-by-Channel</h3>
                          <p className="text-emerald-300 text-sm">Separate campaigns per channel</p>
                        </div>
                      </div>

                      <p className="text-gray-300 mb-6 leading-relaxed">
                        Create <strong className="text-white">individual campaigns</strong> for each channel. Pick one channel, compose a tailored message, then create another for the next channel.
                      </p>

                      <div className="space-y-3">
                        {[{
                          icon: Edit3,
                          text: 'Custom message per channel'
                        }, {
                          icon: Users,
                          text: 'Different audiences per channel'
                        }, {
                          icon: Clock,
                          text: 'Separate scheduling per channel'
                        }].map((feat, i) => (
                          <div key={i} className="flex items-center gap-3 text-gray-300">
                            <div className="w-8 h-8 rounded-lg bg-emerald-600/20 flex items-center justify-center flex-shrink-0">
                              <feat.icon className="w-4 h-4 text-emerald-400" />
                            </div>
                            <span className="text-sm">{feat.text}</span>
                          </div>
                        ))}
                      </div>

                      <div className="mt-6 flex items-center gap-2 text-emerald-400 font-medium group-hover:text-emerald-300 transition-colors">
                        <span>Get Started</span>
                        <ChevronRight className="w-5 h-5" />
                      </div>
                    </button>
                  </div>

                  {/* How to choose guide */}
                  <div className="max-w-4xl mx-auto bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                    <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                      <Info className="w-5 h-5 text-blue-400" />
                      Which one should I choose?
                    </h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div className="bg-blue-900/20 border border-blue-800/50 rounded-lg p-4">
                        <h4 className="text-blue-300 font-semibold mb-2 flex items-center gap-2">
                          <Layers className="w-4 h-4" />
                          Choose Omni-Channel when:
                        </h4>
                        <ul className="text-gray-400 text-sm space-y-1.5">
                          <li>• You want to reach customers everywhere with the same message</li>
                          <li>• You're announcing something important (sale, update, alert)</li>
                          <li>• You want to save time composing one message</li>
                          <li>• The message is general enough for all channels</li>
                        </ul>
                      </div>
                      <div className="bg-emerald-900/20 border border-emerald-800/50 rounded-lg p-4">
                        <h4 className="text-emerald-300 font-semibold mb-2 flex items-center gap-2">
                          <Split className="w-4 h-4" />
                          Choose Channel-by-Channel when:
                        </h4>
                        <ul className="text-gray-400 text-sm space-y-1.5">
                          <li>• Each channel needs a different message tone or content</li>
                          <li>• You want to target different audiences per channel</li>
                          <li>• You need channel-specific scheduling</li>
                          <li>• You're running channel-specific promotions</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              ) : (
                /* Campaign Type Selected — Show Composer */
                <div className="space-y-6">
                  {/* Back to type selection */}
                  <div className="flex items-center justify-between">
                    <button
                      onClick={() => { setCampaignType(null); setSingleChannel(''); setSelectedChannels([]) }}
                      className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors"
                    >
                      <ChevronLeft className="w-4 h-4" />
                      Change Campaign Type
                    </button>
                    <div className="flex items-center gap-2">
                      {campaignType === 'omni' ? (
                        <span className="flex items-center gap-2 px-3 py-1.5 bg-blue-900/30 border border-blue-700/50 text-blue-300 rounded-lg text-sm font-medium">
                          <Layers className="w-4 h-4" />
                          Omni-Channel Campaign
                        </span>
                      ) : (
                        <span className="flex items-center gap-2 px-3 py-1.5 bg-emerald-900/30 border border-emerald-700/50 text-emerald-300 rounded-lg text-sm font-medium">
                          <Split className="w-4 h-4" />
                          Channel-by-Channel Campaign
                        </span>
                      )}
                    </div>
                  </div>
              {campaignType === 'omni' ? (
                /* ===== OMNI-CHANNEL COMPOSER ===== */
                <>
              {/* Message Composer */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Edit3 className="w-5 h-5 text-blue-400" />
                  Message Composer
                </h3>

                <div className="space-y-4">
                  <div>
                    <label className="block text-gray-400 text-sm mb-1">Title *</label>
                    <input
                      type="text"
                      value={title}
                      onChange={e => setTitle(e.target.value)}
                      placeholder="Internal title for this broadcast"
                      className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                    />
                  </div>

                  <div>
                    <label className="block text-gray-400 text-sm mb-1">Body Text *</label>
                    <textarea
                      value={bodyText}
                      onChange={e => setBodyText(e.target.value)}
                      placeholder="Write your broadcast message... (supports *bold*, _italic_ for WhatsApp)"
                      rows={5}
                      className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none resize-none"
                    />
                    <p className="text-gray-500 text-xs mt-1">{bodyText.length} characters</p>
                  </div>

                  <div>
                    <label className="block text-gray-400 text-sm mb-1">HTML Body (auto-formats for all channels)</label>
                    <textarea
                      value={bodyHtml}
                      onChange={e => setBodyHtml(e.target.value)}
                      placeholder="<h1>Summer Sale!</h1><p>Get <b>50% off</b> all items. <a href='https://example.com'>Shop now</a></p>"
                      rows={4}
                      className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none resize-none font-mono text-sm"
                    />
                    <div className="flex items-center justify-between mt-2">
                      <p className="text-gray-500 text-xs">Paste HTML here — it will be auto-formatted for each channel</p>
                      <button
                        type="button"
                        onClick={async () => {
                          if (!bodyHtml.trim() && !bodyText.trim()) return
                          setHtmlPreviewLoading(true)
                          setShowHtmlPreview(true)
                          try {
                            const channels = selectedChannels.length > 0 ? selectedChannels : ['whatsapp', 'email', 'sms', 'instagram', 'facebook', 'telegram', 'discord', 'slack']
                            const formats = await previewHtmlForChannels(bodyHtml.trim(), bodyText.trim(), channels)
                            setHtmlPreviewFormats(formats)
                          } catch (e: any) {
                            console.error('Preview failed:', e)
                          } finally {
                            setHtmlPreviewLoading(false)
                          }
                        }}
                        className="flex items-center gap-1.5 px-3 py-1.5 bg-purple-600 hover:bg-purple-500 text-white text-xs font-medium rounded-lg transition-colors"
                      >
                        <Eye className="w-3.5 h-3.5" />
                        Preview on all channels
                      </button>
                    </div>
                  </div>

                  {/* HTML-to-All-Channels Live Preview */}
                  {showHtmlPreview && (
                    <div className="mt-4 bg-gray-900/70 border border-purple-800/50 rounded-xl p-5">
                      <div className="flex items-center justify-between mb-4">
                        <h4 className="text-white font-semibold flex items-center gap-2">
                          <Sparkles className="w-4 h-4 text-purple-400" />
                          Channel-by-Channel Preview
                        </h4>
                        <button
                          onClick={() => setShowHtmlPreview(false)}
                          className="text-gray-400 hover:text-white"
                        >
                          <X className="w-4 h-4" />
                        </button>
                      </div>
                      {htmlPreviewLoading ? (
                        <p className="text-gray-400 text-sm">Generating previews...</p>
                      ) : (
                        <div className="space-y-4 max-h-96 overflow-y-auto pr-2">
                          {htmlPreviewFormats.map(fmt => {
                            const Icon = CHANNEL_ICONS[fmt.channelCode] || Globe
                            const color = CHANNEL_COLORS[fmt.channelCode] || '#6B7280'
                            const charPercent = Math.min(100, (fmt.formattedBody.length / fmt.maxLength) * 100)
                            const charColor = charPercent > 90 ? 'text-red-400' : charPercent > 70 ? 'text-yellow-400' : 'text-green-400'
                            return (
                              <div key={fmt.channelCode} className="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                                <div className="flex items-center gap-2 mb-2">
                                  <Icon className="w-4 h-4" style={{ color }} />
                                  <span className="text-white font-medium text-sm">{fmt.channelName}</span>
                                  <span className={`text-xs ${charColor}`}>{fmt.formattedBody.length}/{fmt.maxLength} chars</span>
                                  {fmt.supportsHtml && <span className="text-xs bg-blue-900/50 text-blue-300 px-1.5 py-0.5 rounded">HTML</span>}
                                  {fmt.supportsImages && <span className="text-xs bg-green-900/50 text-green-300 px-1.5 py-0.5 rounded">Images</span>}
                                  {fmt.supportsVideo && <span className="text-xs bg-purple-900/50 text-purple-300 px-1.5 py-0.5 rounded">Video</span>}
                                </div>
                                <div className="bg-gray-900 rounded-lg p-3 text-gray-200 text-sm whitespace-pre-wrap font-mono max-h-32 overflow-y-auto">
                                  {fmt.formattedBody || <span className="text-gray-600 italic">No content</span>}
                                </div>
                                {fmt.formattedSubject && (
                                  <div className="mt-2 text-xs text-gray-400">
                                    <span className="font-medium">Subject:</span> {fmt.formattedSubject}
                                  </div>
                                )}
                              </div>
                            )
                          })}
                        </div>
                      )}
                    </div>
                  )}

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-gray-400 text-sm mb-1">Email Subject</label>
                      <input
                        type="text"
                        value={subjectLine}
                        onChange={e => setSubjectLine(e.target.value)}
                        placeholder="Email subject line"
                        className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                      />
                    </div>
                    <div>
                      <label className="block text-gray-400 text-sm mb-1">Preview Text</label>
                      <input
                        type="text"
                        value={previewText}
                        onChange={e => setPreviewText(e.target.value)}
                        placeholder="Short preview for link previews"
                        className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                      />
                    </div>
                  </div>
                </div>
              </div>

              {/* Media Section */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Image className="w-5 h-5 text-blue-400" />
                  Media & Links
                </h3>

                <div className="space-y-4">
                  <div>
                    <label className="block text-gray-400 text-sm mb-1">Media Type</label>
                    <div className="flex gap-2">
                      {[
                        { value: 'text', label: 'Text Only', icon: FileTextIcon },
                        { value: 'image', label: 'Image', icon: Image },
                        { value: 'video', label: 'Video', icon: Video },
                        { value: 'link', label: 'Link', icon: Link },
                        { value: 'mixed', label: 'Mixed', icon: Sparkles },
                      ].map(opt => {
                        const Icon = opt.icon
                        return (
                          <button
                            key={opt.value}
                            onClick={() => setMediaType(opt.value)}
                            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                              mediaType === opt.value
                                ? 'bg-blue-600 text-white'
                                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                            }`}
                          >
                            <Icon className="w-4 h-4" />
                            {opt.label}
                          </button>
                        )
                      })}
                    </div>
                  </div>

                  {mediaType !== 'text' && (
                    <>
                      <div>
                        <label className="block text-gray-400 text-sm mb-1">Media URL</label>
                        <input
                          type="text"
                          value={primaryMediaUrl}
                          onChange={e => setPrimaryMediaUrl(e.target.value)}
                          placeholder="https://example.com/image.jpg"
                          className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                        />
                      </div>
                      <div>
                        <label className="block text-gray-400 text-sm mb-1">Media Caption</label>
                        <input
                          type="text"
                          value={mediaCaption}
                          onChange={e => setMediaCaption(e.target.value)}
                          placeholder="Caption for the media"
                          className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                        />
                      </div>
                      {primaryMediaUrl && mediaType === 'image' && (
                        <div className="bg-gray-900/50 rounded-lg p-3">
                          <img src={primaryMediaUrl} alt="Preview" className="max-h-48 rounded-lg mx-auto" onError={e => { (e.target as HTMLImageElement).style.display = 'none' }} />
                        </div>
                      )}
                    </>
                  )}

                  {(mediaType === 'link' || mediaType === 'mixed') && (
                    <div className="space-y-3 p-4 bg-gray-900/30 rounded-lg border border-gray-700">
                      <h4 className="text-white font-medium flex items-center gap-2">
                        <Link className="w-4 h-4 text-blue-400" />
                        Link Preview
                      </h4>
                      <div>
                        <label className="block text-gray-400 text-sm mb-1">URL</label>
                        <input
                          type="text"
                          value={linkUrl}
                          onChange={e => setLinkUrl(e.target.value)}
                          placeholder="https://example.com/page"
                          className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                        />
                      </div>
                      <div className="grid grid-cols-2 gap-3">
                        <div>
                          <label className="block text-gray-400 text-sm mb-1">Title</label>
                          <input
                            type="text"
                            value={linkTitle}
                            onChange={e => setLinkTitle(e.target.value)}
                            placeholder="Link title"
                            className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                          />
                        </div>
                        <div>
                          <label className="block text-gray-400 text-sm mb-1">Image URL</label>
                          <input
                            type="text"
                            value={linkImageUrl}
                            onChange={e => setLinkImageUrl(e.target.value)}
                            placeholder="https://example.com/thumb.jpg"
                            className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                          />
                        </div>
                      </div>
                      <div>
                        <label className="block text-gray-400 text-sm mb-1">Description</label>
                        <input
                          type="text"
                          value={linkDescription}
                          onChange={e => setLinkDescription(e.target.value)}
                          placeholder="Link description"
                          className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                        />
                      </div>
                    </div>
                  )}
                </div>
              </div>

              {/* Channel Selection */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Globe className="w-5 h-5 text-blue-400" />
                  Select Channels
                </h3>

                <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                  {availableChannels.map(ch => {
                    const Icon = CHANNEL_ICONS[ch.channelCode] || Globe
                    const color = CHANNEL_COLORS[ch.channelCode] || '#6B7280'
                    const isSelected = selectedChannels.includes(ch.channelCode)
                    return (
                      <button
                        key={ch.channelCode}
                        onClick={() => toggleChannel(ch.channelCode)}
                        className={`flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all ${
                          isSelected
                            ? 'border-blue-500 bg-blue-900/20'
                            : 'border-gray-700 bg-gray-900/30 hover:border-gray-600'
                        }`}
                      >
                        <Icon className="w-8 h-8" style={{ color: isSelected ? '#60A5FA' : color }} />
                        <span className="text-white font-medium text-sm">{ch.channelName}</span>
                        <span className="text-gray-500 text-xs">Max {ch.maxLength} chars</span>
                        {isSelected && <Check className="w-4 h-4 text-blue-400" />}
                      </button>
                    )
                  })}
                </div>
              </div>

              {/* Target Audience */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Users className="w-5 h-5 text-blue-400" />
                  Target Audience
                </h3>

                <div className="space-y-4">
                  <div>
                    <label className="block text-gray-400 text-sm mb-1">Target Type</label>
                    <select
                      value={targetType}
                      onChange={e => setTargetType(e.target.value)}
                      className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                    >
                      <option value="contacts">Individual Contacts</option>
                      <option value="segments">Segments</option>
                      <option value="all_contacts">All Contacts</option>
                    </select>
                  </div>

                  {targetType !== 'all_contacts' && (
                    <div>
                      <label className="block text-gray-400 text-sm mb-1">
                        {targetType === 'contacts' ? 'Contact IDs (comma-separated)' : 'Segment IDs (comma-separated)'}
                      </label>
                      <textarea
                        value={targetIds}
                        onChange={e => setTargetIds(e.target.value)}
                        placeholder={targetType === 'contacts' ? 'contact-1, contact-2, contact-3' : 'segment-1, segment-2'}
                        rows={3}
                        className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none resize-none"
                      />
                    </div>
                  )}
                </div>
              </div>

              {/* Schedule */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Clock className="w-5 h-5 text-blue-400" />
                  Schedule
                </h3>

                <div>
                  <label className="block text-gray-400 text-sm mb-1">Send At (optional — leave empty for immediate send)</label>
                  <input
                    type="datetime-local"
                    value={scheduledAt}
                    onChange={e => setScheduledAt(e.target.value)}
                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                  />
                </div>
              </div>

              {/* Create Button */}
              <button
                onClick={handleCreate}
                disabled={creating}
                className="w-full flex items-center justify-center gap-3 px-6 py-4 bg-blue-600 hover:bg-blue-500 text-white rounded-xl font-bold text-lg transition-colors disabled:opacity-50"
              >
                <Sparkles className="w-6 h-6" />
                {creating ? 'Creating...' : 'Create Omni-Channel Campaign'}
              </button>
                </>
              ) : (
                /* ===== CHANNEL-BY-CHANNEL COMPOSER ===== */
                <>
              {/* Campaign Title */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Edit3 className="w-5 h-5 text-emerald-400" />
                  Campaign Title
                </h3>
                <div>
                  <label className="block text-gray-400 text-sm mb-1">Campaign Title *</label>
                  <input
                    type="text"
                    value={title}
                    onChange={e => setTitle(e.target.value)}
                    placeholder="e.g., Summer Sale 2026 — each channel gets its own message"
                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                  />
                  <p className="text-gray-500 text-xs mt-1">This title will be prefixed to each channel's broadcast (e.g., "Summer Sale 2026 (whatsapp)")</p>
                </div>
              </div>

              {/* Select Channels for Channel-by-Channel */}
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Globe className="w-5 h-5 text-emerald-400" />
                  Select Channels to Include
                </h3>
                <p className="text-gray-400 text-sm mb-4">Choose which channels to include in this campaign. Each selected channel will get its own tailored message.</p>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                  {availableChannels.map(ch => {
                    const Icon = CHANNEL_ICONS[ch.channelCode] || Globe
                    const color = CHANNEL_COLORS[ch.channelCode] || '#6B7280'
                    const isSelected = multiChannels.includes(ch.channelCode)
                    return (
                      <button
                        key={ch.channelCode}
                        onClick={() => toggleMultiChannel(ch.channelCode)}
                        className={`flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all ${
                          isSelected
                            ? 'border-emerald-500 bg-emerald-900/20'
                            : 'border-gray-700 bg-gray-900/30 hover:border-gray-600'
                        }`}
                      >
                        <Icon className="w-8 h-8" style={{ color: isSelected ? '#34D399' : color }} />
                        <span className="text-white font-medium text-sm">{ch.channelName}</span>
                        <span className="text-gray-500 text-xs">Max {ch.maxLength} chars</span>
                        {isSelected && <Check className="w-4 h-4 text-emerald-400" />}
                      </button>
                    )
                  })}
                </div>
                {multiChannels.length > 0 && (
                  <div className="mt-4 p-3 bg-emerald-900/20 border border-emerald-800/50 rounded-lg">
                    <p className="text-emerald-300 text-sm font-medium">{multiChannels.length} channel{multiChannels.length > 1 ? 's' : ''} selected</p>
                    <p className="text-gray-400 text-xs mt-1">{multiChannels.map(c => c.charAt(0).toUpperCase() + c.slice(1)).join(', ')}</p>
                  </div>
                )}
              </div>

              {/* Per-Channel Message Composer */}
              {multiChannels.length > 0 && (
                <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                  <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                    <Edit3 className="w-5 h-5 text-emerald-400" />
                    Per-Channel Messages
                  </h3>
                  <p className="text-gray-400 text-sm mb-4">Compose a tailored message for each channel. Click a channel tab to edit its message.</p>

                  {/* Channel Tabs */}
                  <div className="flex flex-wrap gap-2 mb-6">
                    {multiChannels.map(ch => {
                      const Icon = CHANNEL_ICONS[ch.channelCode] || Globe
                      const color = CHANNEL_COLORS[ch.channelCode] || '#6B7280'
                      const isActive = activeChannelTab === ch
                      const msg = channelMessages[ch]
                      const hasBody = msg?.bodyText.trim()
                      return (
                        <button
                          key={ch}
                          onClick={() => {
                            setActiveChannelTab(ch)
                            if (!channelMessages[ch]) getChannelMessage(ch)
                          }}
                          className={`flex items-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-all ${
                            isActive
                              ? 'bg-emerald-600 text-white shadow-lg shadow-emerald-900/30'
                              : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
                          }`}
                        >
                          <Icon className="w-4 h-4" style={{ color: isActive ? '#fff' : color }} />
                          <span>{ch.charAt(0).toUpperCase() + ch.slice(1)}</span>
                          {hasBody && <Check className="w-3.5 h-3.5 text-green-300" />}
                          {!hasBody && <CircleDot className="w-3.5 h-3.5 text-yellow-500" />}
                        </button>
                      )
                    })}
                  </div>

                  {/* Active Channel Composer */}
                  {activeChannelTab && (() => {
                    const msg = getChannelMessage(activeChannelTab)
                    const Icon = CHANNEL_ICONS[activeChannelTab] || Globe
                    const color = CHANNEL_COLORS[activeChannelTab] || '#6B7280'
                    const chInfo = availableChannels.find(c => c.channelCode === activeChannelTab)
                    return (
                      <div className="bg-gray-900/50 rounded-xl p-6 border border-gray-700">
                        <div className="flex items-center gap-3 mb-6 pb-4 border-b border-gray-700">
                          <Icon className="w-6 h-6" style={{ color }} />
                          <div>
                            <h4 className="text-white font-semibold">
                              Compose for {activeChannelTab.charAt(0).toUpperCase() + activeChannelTab.slice(1)}
                            </h4>
                            {chInfo && <p className="text-gray-500 text-xs">Max {chInfo.maxLength} chars · {chInfo.supportsHtml ? 'HTML' : 'Plain text'}{chInfo.supportsImages ? ' · Images' : ''}{chInfo.supportsVideo ? ' · Video' : ''}</p>}
                          </div>
                        </div>

                        <div className="space-y-4">
                          <div>
                            <label className="block text-gray-400 text-sm mb-1">Body Text *</label>
                            <textarea
                              value={msg.bodyText}
                              onChange={e => updateChannelMessage(activeChannelTab, 'bodyText', e.target.value)}
                              placeholder={`Write your ${activeChannelTab} message...`}
                              rows={5}
                              className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none resize-none"
                            />
                            <p className="text-gray-500 text-xs mt-1">{msg.bodyText.length} / {chInfo?.maxLength || 4096} characters</p>
                          </div>

                          {activeChannelTab === 'email' && (
                            <>
                              <div className="grid grid-cols-2 gap-4">
                                <div>
                                  <label className="block text-gray-400 text-sm mb-1">Email Subject</label>
                                  <input
                                    type="text"
                                    value={msg.subjectLine}
                                    onChange={e => updateChannelMessage(activeChannelTab, 'subjectLine', e.target.value)}
                                    placeholder="Email subject line"
                                    className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                                  />
                                </div>
                                <div>
                                  <label className="block text-gray-400 text-sm mb-1">Preview Text</label>
                                  <input
                                    type="text"
                                    value={msg.previewText}
                                    onChange={e => updateChannelMessage(activeChannelTab, 'previewText', e.target.value)}
                                    placeholder="Preview text"
                                    className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                                  />
                                </div>
                              </div>
                              <div>
                                <label className="block text-gray-400 text-sm mb-1">HTML Body</label>
                                <textarea
                                  value={msg.bodyHtml}
                                  onChange={e => updateChannelMessage(activeChannelTab, 'bodyHtml', e.target.value)}
                                  placeholder="<p>HTML version...</p>"
                                  rows={3}
                                  className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none resize-none font-mono text-sm"
                                />
                              </div>
                            </>
                          )}

                          {/* Media */}
                          <div>
                            <label className="block text-gray-400 text-sm mb-1">Media Type</label>
                            <div className="flex gap-2 flex-wrap">
                              {[
                                { value: 'text', label: 'Text Only', icon: FileTextIcon },
                                { value: 'image', label: 'Image', icon: Image },
                                { value: 'video', label: 'Video', icon: Video },
                                { value: 'link', label: 'Link', icon: Link },
                              ].map(opt => {
                                const OIcon = opt.icon
                                return (
                                  <button
                                    key={opt.value}
                                    onClick={() => updateChannelMessage(activeChannelTab, 'mediaType', opt.value)}
                                    className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                                      msg.mediaType === opt.value
                                        ? 'bg-emerald-600 text-white'
                                        : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                                    }`}
                                  >
                                    <OIcon className="w-3.5 h-3.5" />
                                    {opt.label}
                                  </button>
                                )
                              })}
                            </div>
                          </div>

                          {msg.mediaType !== 'text' && (
                            <>
                              <div>
                                <label className="block text-gray-400 text-sm mb-1">Media URL</label>
                                <input
                                  type="text"
                                  value={msg.primaryMediaUrl}
                                  onChange={e => updateChannelMessage(activeChannelTab, 'primaryMediaUrl', e.target.value)}
                                  placeholder="https://example.com/media.jpg"
                                  className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                                />
                              </div>
                              <div>
                                <label className="block text-gray-400 text-sm mb-1">Media Caption</label>
                                <input
                                  type="text"
                                  value={msg.mediaCaption}
                                  onChange={e => updateChannelMessage(activeChannelTab, 'mediaCaption', e.target.value)}
                                  placeholder="Caption"
                                  className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                                />
                              </div>
                              {msg.primaryMediaUrl && msg.mediaType === 'image' && (
                                <div className="bg-gray-800 rounded-lg p-3">
                                  <img src={msg.primaryMediaUrl} alt="Preview" className="max-h-40 rounded-lg mx-auto" onError={e => { (e.target as HTMLImageElement).style.display = 'none' }} />
                                </div>
                              )}
                            </>
                          )}

                          {msg.mediaType === 'link' && (
                            <div className="space-y-3 p-4 bg-gray-800 rounded-lg border border-gray-700">
                              <h4 className="text-white font-medium flex items-center gap-2 text-sm">
                                <Link className="w-4 h-4 text-emerald-400" />
                                Link Preview
                              </h4>
                              <div>
                                <label className="block text-gray-400 text-xs mb-1">URL</label>
                                <input
                                  type="text"
                                  value={msg.linkUrl}
                                  onChange={e => updateChannelMessage(activeChannelTab, 'linkUrl', e.target.value)}
                                  placeholder="https://example.com/page"
                                  className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-3 py-2 focus:border-emerald-500 focus:outline-none text-sm"
                                />
                              </div>
                              <div className="grid grid-cols-2 gap-3">
                                <div>
                                  <label className="block text-gray-400 text-xs mb-1">Title</label>
                                  <input
                                    type="text"
                                    value={msg.linkTitle}
                                    onChange={e => updateChannelMessage(activeChannelTab, 'linkTitle', e.target.value)}
                                    placeholder="Link title"
                                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-3 py-2 focus:border-emerald-500 focus:outline-none text-sm"
                                  />
                                </div>
                                <div>
                                  <label className="block text-gray-400 text-xs mb-1">Image URL</label>
                                  <input
                                    type="text"
                                    value={msg.linkImageUrl}
                                    onChange={e => updateChannelMessage(activeChannelTab, 'linkImageUrl', e.target.value)}
                                    placeholder="https://example.com/thumb.jpg"
                                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-3 py-2 focus:border-emerald-500 focus:outline-none text-sm"
                                  />
                                </div>
                              </div>
                              <div>
                                <label className="block text-gray-400 text-xs mb-1">Description</label>
                                <input
                                  type="text"
                                  value={msg.linkDescription}
                                  onChange={e => updateChannelMessage(activeChannelTab, 'linkDescription', e.target.value)}
                                  placeholder="Link description"
                                  className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-3 py-2 focus:border-emerald-500 focus:outline-none text-sm"
                                />
                              </div>
                            </div>
                          )}

                          {/* Target Audience per channel */}
                          <div className="pt-4 border-t border-gray-700">
                            <h4 className="text-white font-medium mb-3 flex items-center gap-2">
                              <Users className="w-4 h-4 text-emerald-400" />
                              Target Audience for {activeChannelTab}
                            </h4>
                            <div className="space-y-3">
                              <div>
                                <label className="block text-gray-400 text-sm mb-1">Target Type</label>
                                <select
                                  value={msg.targetType}
                                  onChange={e => updateChannelMessage(activeChannelTab, 'targetType', e.target.value)}
                                  className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                                >
                                  <option value="contacts">Individual Contacts</option>
                                  <option value="segments">Segments</option>
                                  <option value="all_contacts">All Contacts</option>
                                </select>
                              </div>
                              {msg.targetType !== 'all_contacts' && (
                                <div>
                                  <label className="block text-gray-400 text-sm mb-1">
                                    {msg.targetType === 'contacts' ? 'Contact IDs (comma-separated)' : 'Segment IDs (comma-separated)'}
                                  </label>
                                  <textarea
                                    value={msg.targetIds}
                                    onChange={e => updateChannelMessage(activeChannelTab, 'targetIds', e.target.value)}
                                    placeholder={msg.targetType === 'contacts' ? 'contact-1, contact-2' : 'segment-1, segment-2'}
                                    rows={2}
                                    className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none resize-none text-sm"
                                  />
                                </div>
                              )}
                            </div>
                          </div>

                          {/* Schedule per channel */}
                          <div className="pt-4 border-t border-gray-700">
                            <h4 className="text-white font-medium mb-3 flex items-center gap-2">
                              <Clock className="w-4 h-4 text-emerald-400" />
                              Schedule for {activeChannelTab}
                            </h4>
                            <div>
                              <label className="block text-gray-400 text-sm mb-1">Send At (optional)</label>
                              <input
                                type="datetime-local"
                                value={msg.scheduledAt}
                                onChange={e => updateChannelMessage(activeChannelTab, 'scheduledAt', e.target.value)}
                                className="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-emerald-500 focus:outline-none"
                              />
                            </div>
                          </div>
                        </div>
                      </div>
                    )
                  })()}
                </div>
              )}

              {/* Create Button for Channel-by-Channel */}
              {multiChannels.length > 0 && (
                <button
                  onClick={handleCreate}
                  disabled={creating}
                  className="w-full flex items-center justify-center gap-3 px-6 py-4 bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl font-bold text-lg transition-colors disabled:opacity-50"
                >
                  <Sparkles className="w-6 h-6" />
                  {creating ? `Creating ${multiChannels.length} Broadcasts...` : `Create ${multiChannels.length} Channel Campaign${multiChannels.length > 1 ? 's' : ''}`}
                </button>
              )}
                </>
              )}
            </div>
              )}
            </div>
          )}

          {/* Channels Tab */}
          {activeTab === 'channels' && (
            <div className="space-y-6">
              <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Globe className="w-5 h-5 text-blue-400" />
                  Available Channels & Formatting
                </h3>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {availableChannels.map(ch => {
                    const Icon = CHANNEL_ICONS[ch.channelCode] || Globe
                    const color = CHANNEL_COLORS[ch.channelCode] || '#6B7280'
                    return (
                      <div key={ch.channelCode} className="bg-gray-900/50 rounded-xl p-5 border border-gray-700">
                        <div className="flex items-center gap-3 mb-3">
                          <Icon className="w-6 h-6" style={{ color }} />
                          <h4 className="text-white font-semibold">{ch.channelName}</h4>
                          <span className="text-gray-500 text-xs ml-auto">Max {ch.maxLength} chars</span>
                        </div>
                        <p className="text-gray-400 text-sm mb-3">{ch.formattedBody}</p>
                        <div className="flex flex-wrap gap-1.5">
                          {ch.supportsHtml && <span className="text-xs px-2 py-0.5 bg-green-900/30 text-green-400 rounded">HTML</span>}
                          {ch.supportsImages && <span className="text-xs px-2 py-0.5 bg-blue-900/30 text-blue-400 rounded">Images</span>}
                          {ch.supportsVideo && <span className="text-xs px-2 py-0.5 bg-purple-900/30 text-purple-400 rounded">Video</span>}
                          {ch.supportsLinks && <span className="text-xs px-2 py-0.5 bg-yellow-900/30 text-yellow-400 rounded">Links</span>}
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            </div>
          )}

          {/* Settings Tab */}
          {activeTab === 'settings' && settings && (
            <div className="bg-gray-800/50 border border-gray-700 rounded-xl p-6">
              <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                <Settings className="w-5 h-5 text-blue-400" />
                Broadcast Settings
              </h3>

              <div className="space-y-4 max-w-lg">
                {[
                  { key: 'enabled', label: 'Enable Broadcasts', type: 'toggle' as const },
                  { key: 'allowScheduling', label: 'Allow Scheduling', type: 'toggle' as const },
                  { key: 'requireApproval', label: 'Require Approval Before Send', type: 'toggle' as const },
                  { key: 'autoFormatChannels', label: 'Auto-Format for Each Channel', type: 'toggle' as const },
                  { key: 'trackOpens', label: 'Track Opens', type: 'toggle' as const },
                  { key: 'trackClicks', label: 'Track Clicks', type: 'toggle' as const },
                ].map(toggle => (
                  <div key={toggle.key} className="flex items-center justify-between p-3 bg-gray-900/30 rounded-lg">
                    <span className="text-gray-300">{toggle.label}</span>
                    <button
                      onClick={() => handleUpdateSettings({ [toggle.key]: !(settings as any)[toggle.key] })}
                      className={`w-12 h-6 rounded-full transition-colors relative ${
                        (settings as any)[toggle.key] ? 'bg-blue-600' : 'bg-gray-600'
                      }`}
                    >
                      <div className={`w-5 h-5 bg-white rounded-full absolute top-0.5 transition-all ${
                        (settings as any)[toggle.key] ? 'left-6' : 'left-0.5'
                      }`} />
                    </button>
                  </div>
                ))}

                <div>
                  <label className="block text-gray-400 text-sm mb-1">Max Recipients per Broadcast</label>
                  <input
                    type="number"
                    value={settings.maxRecipientsPerBroadcast}
                    onChange={e => handleUpdateSettings({ maxRecipientsPerBroadcast: parseInt(e.target.value) || 10000 })}
                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                  />
                </div>

                <div>
                  <label className="block text-gray-400 text-sm mb-1">Default Send Delay (seconds)</label>
                  <input
                    type="number"
                    value={settings.defaultSendDelaySeconds}
                    onChange={e => handleUpdateSettings({ defaultSendDelaySeconds: parseInt(e.target.value) || 0 })}
                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                  />
                </div>

                <div>
                  <label className="block text-gray-400 text-sm mb-1">Rate Limit (per minute)</label>
                  <input
                    type="number"
                    value={settings.rateLimitPerMinute}
                    onChange={e => handleUpdateSettings({ rateLimitPerMinute: parseInt(e.target.value) || 60 })}
                    className="w-full bg-gray-900 border border-gray-700 text-white rounded-lg px-4 py-2.5 focus:border-blue-500 focus:outline-none"
                  />
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  )
}
