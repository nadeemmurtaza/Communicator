import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../lib/api'
import type {
  VoiceChannelData,
  VoiceUpdatePayload,
  VoiceCallLogItem,
  SocialEvent,
} from '../lib/api'

export function VoiceIntegration() {
  const [activeTab, setActiveTab] = useState<'credentials' | 'dialpad' | 'ivr' | 'recordings' | 'logs'>('credentials')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null)

  // Voice Data State
  const [_data, setData] = useState<VoiceChannelData | null>(null)
  const [callLogs, setCallLogs] = useState<VoiceCallLogItem[]>([])
  const [events, setEvents] = useState<SocialEvent[]>([])

  // Form Fields
  const [name, setName] = useState('Voice & WebRTC Calling')
  const [accountSid, setAccountSid] = useState('')
  const [authToken, setAuthToken] = useState('')
  const [voiceNumber, setVoiceNumber] = useState('+14155550199')
  const [callerId, setCallerId] = useState('+14155550199')
  const [twimlAppSid, setTwimlAppSid] = useState('AP_mock_twiml_app_sid')
  const [sipDomain, setSipDomain] = useState('bayworks.sip.twilio.com')
  const [fallbackNumber, setFallbackNumber] = useState('+14155550100')
  const [recordingEnabled, setRecordingEnabled] = useState(true)
  const [recordingChannels, setRecordingChannels] = useState('dual')
  const [transcriptionEnabled, setTranscriptionEnabled] = useState(true)
  const [ivrEnabled, setIvrEnabled] = useState(true)
  const [ivrPrompt, setIvrPrompt] = useState('Thank you for calling Bay Works. Press 1 for Sales, 2 for Technical Support, or 3 to leave a Voicemail.')
  const [ivrMenu, setIvrMenu] = useState<Array<{ digit: string; label: string; action: string; target: string }>>([
    { digit: '1', label: 'Sales', action: 'queue', target: 'sales-queue' },
    { digit: '2', label: 'Technical Support', action: 'queue', target: 'support-queue' },
    { digit: '3', label: 'Voicemail', action: 'voicemail', target: 'voicemail-box' },
  ])
  const [webrtcEnabled, setWebrtcEnabled] = useState(true)
  const [greetingEnabled, setGreetingEnabled] = useState(true)
  const [greetingMessage, setGreetingMessage] = useState('Thank you for calling Bay Works. Connecting you to an agent.')
  const [outOfOfficeEnabled, setOutOfOfficeEnabled] = useState(false)
  const [outOfOfficeMessage, setOutOfOfficeMessage] = useState('Our phone lines are currently closed. Please leave a voicemail after the tone.')
  const [csatEnabled, setCsatEnabled] = useState(true)

  // In-Inbox Softphone / Dialpad State
  const [dialNumber, setDialNumber] = useState('')
  const [activeCall, setActiveCall] = useState<{
    callSid: string
    number: string
    status: 'calling' | 'connected' | 'on_hold'
    duration: number
    isMuted: boolean
  } | null>(null)
  const [timerInterval, setTimerInterval] = useState<any>(null)

  // Copy Feedback
  const [copiedKey, setCopiedKey] = useState<string | null>(null)

  const loadData = async () => {
    try {
      setLoading(true)
      const res = await api.getVoiceChannel()
      setData(res)
      setName(res.inbox.name || 'Voice & WebRTC Calling')
      setAccountSid(res.inbox.credentials?.voiceAccountSid || res.inbox.credentials?.accountSid || '')
      setVoiceNumber((res.inbox as any).phoneNumber || (res.inbox as any).phone_number || res.inbox.credentials?.voiceNumber || '+14155550199')
      setCallerId(res.sipConfig?.callerId || '+14155550199')
      setTwimlAppSid(res.sipConfig?.twimlAppSid || 'AP_mock_twiml_app_sid')
      setSipDomain(res.sipConfig?.sipDomain || 'bayworks.sip.twilio.com')
      setFallbackNumber(res.sipConfig?.fallbackNumber || '+14155550100')
      setRecordingEnabled(res.recordingConfig?.recordingEnabled ?? true)
      setRecordingChannels(res.recordingConfig?.recordingChannels || 'dual')
      setTranscriptionEnabled(res.recordingConfig?.transcriptionEnabled ?? true)
      setIvrEnabled(res.ivrConfig?.enabled ?? true)
      setIvrPrompt(res.ivrConfig?.prompt || '')
      if (res.ivrConfig?.menu?.length) {
        setIvrMenu(res.ivrConfig.menu)
      }
      setWebrtcEnabled(res.webrtcConfig?.enabled ?? true)
      setGreetingEnabled((res.inbox as any).greetingEnabled ?? (res.inbox as any).greeting_enabled ?? true)
      setGreetingMessage((res.inbox as any).greetingMessage || (res.inbox as any).greeting_message || '')
      setOutOfOfficeEnabled((res.inbox as any).outOfOfficeEnabled ?? (res.inbox as any).out_of_office_enabled ?? false)
      setOutOfOfficeMessage((res.inbox as any).outOfOfficeMessage || (res.inbox as any).out_of_office_message || '')
      setCsatEnabled((res.inbox as any).csatEnabled ?? (res.inbox as any).csat_enabled ?? true)

      const [logsRes, eventsRes] = await Promise.all([
        api.getVoiceCallLogs().catch(() => []),
        api.listVoiceEvents().catch(() => []),
      ])
      setCallLogs(logsRes)
      setEvents(eventsRes)
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to load Voice channel settings.' })
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [])

  // Active call timer
  useEffect(() => {
    if (activeCall && activeCall.status === 'connected') {
      const interval = setInterval(() => {
        setActiveCall(prev => prev ? { ...prev, duration: prev.duration + 1 } : null)
      }, 1000)
      setTimerInterval(interval)
      return () => clearInterval(interval)
    } else {
      if (timerInterval) clearInterval(timerInterval)
    }
  }, [activeCall?.status])

  const handleCopy = (text: string, key: string) => {
    navigator.clipboard.writeText(text)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  const handleSave = async () => {
    try {
      setSaving(true)
      setFeedback(null)
      const payload: VoiceUpdatePayload = {
        name,
        account_sid: accountSid,
        auth_token: authToken,
        voice_number: voiceNumber,
        caller_id: callerId,
        twiml_app_sid: twimlAppSid,
        sip_domain: sipDomain,
        recording_enabled: recordingEnabled,
        recording_channels: recordingChannels,
        transcription_enabled: transcriptionEnabled,
        ivr_enabled: ivrEnabled,
        ivr_prompt: ivrPrompt,
        ivr_menu: ivrMenu,
        webrtc_enabled: webrtcEnabled,
        fallback_number: fallbackNumber,
        greeting_enabled: greetingEnabled,
        greeting_message: greetingMessage,
        out_of_office_enabled: outOfOfficeEnabled,
        out_of_office_message: outOfOfficeMessage,
        csat_enabled: csatEnabled,
      }
      const updated = await api.updateVoiceChannel(payload)
      setData(updated)
      setFeedback({ type: 'success', message: 'Voice settings and IVR configuration saved successfully!' })
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to update Voice channel.' })
    } finally {
      setSaving(false)
    }
  }

  const handleVerify = async () => {
    try {
      setVerifying(true)
      setFeedback(null)
      const res = await api.verifyVoiceChannel()
      setFeedback({ type: 'success', message: res.message || 'Twilio Voice credentials & SIP trunk verified successfully!' })
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Twilio Voice verification failed.' })
    } finally {
      setVerifying(false)
    }
  }

  const handleDialKeyPress = (key: string) => {
    setDialNumber(prev => prev + key)
  }

  const handleStartCall = async (phoneToCall?: string) => {
    const target = phoneToCall || dialNumber
    if (!target) return
    try {
      const res = await api.initiateVoiceCall({
        recipient_phone: target,
        agent_name: 'Current Agent',
        message: 'Connecting call from Bay Works Voice dialer.',
      })
      setActiveCall({
        callSid: res.callSid,
        number: target,
        status: 'calling',
        duration: 0,
        isMuted: false,
      })
      setTimeout(() => {
        setActiveCall(prev => prev ? { ...prev, status: 'connected' } : null)
      }, 1500)
      setFeedback({ type: 'success', message: `Call initiated to ${target}.` })
      const logs = await api.getVoiceCallLogs()
      setCallLogs(logs)
    } catch (err: any) {
      setFeedback({ type: 'error', message: err.message || 'Failed to place call.' })
    }
  }

  const handleHangup = async () => {
    if (!activeCall) return
    try {
      await api.hangupVoiceCall(activeCall.callSid)
      setActiveCall(null)
      setFeedback({ type: 'success', message: 'Call terminated.' })
      const logs = await api.getVoiceCallLogs()
      setCallLogs(logs)
    } catch (err: any) {
      setActiveCall(null)
    }
  }

  const handleAddIvrOption = () => {
    const nextDigit = String(ivrMenu.length + 1)
    setIvrMenu([...ivrMenu, { digit: nextDigit, label: 'New Option', action: 'queue', target: 'support-queue' }])
  }

  const handleRemoveIvrOption = (index: number) => {
    setIvrMenu(ivrMenu.filter((_, i) => i !== index))
  }

  const formatSeconds = (sec: number) => {
    const mins = Math.floor(sec / 60)
    const rem = sec % 60
    return `${mins.toString().padStart(2, '0')}:${rem.toString().padStart(2, '0')}`
  }

  const webhookBaseUrl = window.location.origin

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="flex items-center gap-2 text-slate-500">
          <span className="loading loading-spinner loading-md"></span>
          <span>Loading Voice &amp; WebRTC settings...</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Top Banner / Breadcrumb */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <Link to="/channels" className="text-xs font-medium text-slate-500 hover:text-slate-800">
              Channels Hub
            </Link>
            <span className="text-slate-400">/</span>
            <span className="text-xs font-semibold text-slate-700">Voice &amp; WebRTC Calling</span>
          </div>
          <div className="mt-1 flex items-center gap-2">
            <h1 className="text-xl font-bold text-slate-900">Phone Calls Inside Inbox</h1>
            <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800 border border-emerald-200 flex items-center gap-1">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
              Live &amp; Active
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-500">
            Handle phone calls directly inside your agent inbox via Twilio Voice, SIP Trunking, and browser WebRTC softphone.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleVerify}
            disabled={verifying}
            className="rounded-md border border-slate-300 bg-white px-3 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 shadow-xs flex items-center gap-1.5"
          >
            <span>🔄</span> {verifying ? 'Pinging Trunk...' : 'Verify SIP Trunk'}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-md bg-emerald-600 px-4 py-1.5 text-xs font-semibold text-white hover:bg-emerald-700 shadow-xs flex items-center gap-1.5"
          >
            <span>💾</span> {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>

      {/* Feedback Alert */}
      {feedback && (
        <div
          className={`flex items-center justify-between rounded-md p-3 text-xs ${
            feedback.type === 'success'
              ? 'bg-emerald-50 text-emerald-800 border border-emerald-200'
              : 'bg-rose-50 text-rose-800 border border-rose-200'
          }`}
        >
          <div className="flex items-center gap-2">
            <span>{feedback.type === 'success' ? '✅' : '⚠️'}</span>
            <span>{feedback.message}</span>
          </div>
          <button onClick={() => setFeedback(null)} className="text-slate-400 hover:text-slate-600 font-bold">
            ×
          </button>
        </div>
      )}

      {/* In-Call Active Floating Controller Bar */}
      {activeCall && (
        <div className="rounded-xl bg-slate-900 text-white p-4 shadow-xl border border-slate-700 flex flex-col sm:flex-row items-center justify-between gap-4 animate-in slide-in-from-top-2">
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center text-xl animate-pulse">
              📞
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-sm text-white">{activeCall.number}</span>
                <span className="rounded-full bg-emerald-500/20 px-2 py-0.5 text-[10px] font-semibold text-emerald-300 uppercase tracking-wider">
                  {activeCall.status === 'calling' ? 'Ringing...' : 'Connected · WebRTC'}
                </span>
              </div>
              <div className="text-xs text-slate-400 flex items-center gap-2 mt-0.5">
                <span>Duration: <strong className="text-white font-mono">{formatSeconds(activeCall.duration)}</strong></span>
                <span>·</span>
                <span>Dual-Channel Audio</span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={() => setActiveCall({ ...activeCall, isMuted: !activeCall.isMuted })}
              className={`rounded-md px-3 py-1.5 text-xs font-semibold flex items-center gap-1.5 transition-colors ${
                activeCall.isMuted ? 'bg-amber-500 text-slate-900' : 'bg-slate-800 text-slate-200 hover:bg-slate-700'
              }`}
            >
              <span>{activeCall.isMuted ? '🔇' : '🎙️'}</span>
              <span>{activeCall.isMuted ? 'Muted' : 'Mute'}</span>
            </button>
            <button
              onClick={() => setActiveCall({ ...activeCall, status: activeCall.status === 'on_hold' ? 'connected' : 'on_hold' })}
              className={`rounded-md px-3 py-1.5 text-xs font-semibold flex items-center gap-1.5 transition-colors ${
                activeCall.status === 'on_hold' ? 'bg-blue-500 text-white' : 'bg-slate-800 text-slate-200 hover:bg-slate-700'
              }`}
            >
              <span>⏸️</span>
              <span>{activeCall.status === 'on_hold' ? 'Resume' : 'Hold'}</span>
            </button>
            <button
              onClick={handleHangup}
              className="rounded-md bg-rose-600 px-4 py-1.5 text-xs font-bold text-white hover:bg-rose-700 shadow-md flex items-center gap-1.5"
            >
              <span>🔴</span> End Call
            </button>
          </div>
        </div>
      )}

      {/* Tabs Header */}
      <div className="border-b border-slate-200">
        <nav className="flex space-x-6">
          <button
            onClick={() => setActiveTab('credentials')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'credentials'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            <span>🔐</span> Credentials &amp; SIP Trunk
          </button>
          <button
            onClick={() => setActiveTab('dialpad')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'dialpad'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            <span>📟</span> In-Inbox Softphone &amp; Dialer
          </button>
          <button
            onClick={() => setActiveTab('ivr')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'ivr'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            <span>🎛️</span> IVR &amp; Routing
          </button>
          <button
            onClick={() => setActiveTab('recordings')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'recordings'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            <span>🎙️</span> Recordings &amp; AI Transcripts
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`pb-3 text-xs font-semibold border-b-2 flex items-center gap-1.5 ${
              activeTab === 'logs'
                ? 'border-emerald-600 text-emerald-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            <span>📋</span> Call History &amp; Diagnostics ({callLogs.length})
          </button>
        </nav>
      </div>

      {/* TAB 1: Credentials & SIP Trunk */}
      {activeTab === 'credentials' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
              <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                <span>📞</span> Twilio Voice &amp; SIP Trunk Configuration
              </h2>
              <p className="text-xs text-slate-500">
                Connect your Twilio Voice account or SIP trunking provider to send and receive phone calls inside the agent workspace.
              </p>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2">
                <div>
                  <label className="block text-xs font-medium text-slate-700">Inbox Display Name</label>
                  <input
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-slate-700">Voice Phone Number</label>
                  <input
                    type="text"
                    value={voiceNumber}
                    onChange={(e) => setVoiceNumber(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                    placeholder="+14155550199"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-medium text-slate-700">Twilio Account SID</label>
                  <input
                    type="text"
                    value={accountSid}
                    onChange={(e) => setAccountSid(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                    placeholder="AC_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-slate-700">Auth Token</label>
                  <input
                    type="password"
                    value={authToken}
                    onChange={(e) => setAuthToken(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                    placeholder="••••••••••••••••••••••••••••••••"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-medium text-slate-700">Outbound Caller ID</label>
                  <input
                    type="text"
                    value={callerId}
                    onChange={(e) => setCallerId(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-slate-700">TwiML Application SID</label>
                  <input
                    type="text"
                    value={twimlAppSid}
                    onChange={(e) => setTwimlAppSid(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs font-mono shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-medium text-slate-700">SIP Domain Trunk</label>
                  <input
                    type="text"
                    value={sipDomain}
                    onChange={(e) => setSipDomain(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-slate-700">After-Hours Fallback Forwarding Phone</label>
                  <input
                    type="text"
                    value={fallbackNumber}
                    onChange={(e) => setFallbackNumber(e.target.value)}
                    className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
                  />
                </div>
              </div>
            </div>

            {/* Webhook Endpoints Box */}
            <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
              <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                <span>🌐</span> Twilio Voice Webhook Configuration
              </h2>
              <p className="text-xs text-slate-500">
                Configure these endpoints in your Twilio Console Phone Number configuration under Voice &amp; Fax:
              </p>

              <div className="space-y-3 pt-1">
                <div>
                  <div className="flex items-center justify-between text-xs text-slate-700 mb-1">
                    <span className="font-semibold">A Call Comes In (Inbound TwiML Webhook):</span>
                    <button
                      onClick={() => handleCopy(`${webhookBaseUrl}/api/v1/webhooks/voice/inbound`, 'inbound_url')}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      {copiedKey === 'inbound_url' ? '✓ Copied' : 'Copy'}
                    </button>
                  </div>
                  <input
                    readOnly
                    value={`${webhookBaseUrl}/api/v1/webhooks/voice/inbound`}
                    className="block w-full rounded-md bg-slate-50 border border-slate-200 px-3 py-1.5 text-xs font-mono text-slate-700"
                  />
                </div>

                <div>
                  <div className="flex items-center justify-between text-xs text-slate-700 mb-1">
                    <span className="font-semibold">Call Status Callback URL:</span>
                    <button
                      onClick={() => handleCopy(`${webhookBaseUrl}/api/v1/webhooks/voice/status`, 'status_url')}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      {copiedKey === 'status_url' ? '✓ Copied' : 'Copy'}
                    </button>
                  </div>
                  <input
                    readOnly
                    value={`${webhookBaseUrl}/api/v1/webhooks/voice/status`}
                    className="block w-full rounded-md bg-slate-50 border border-slate-200 px-3 py-1.5 text-xs font-mono text-slate-700"
                  />
                </div>

                <div>
                  <div className="flex items-center justify-between text-xs text-slate-700 mb-1">
                    <span className="font-semibold">Call Recording &amp; Audio URL:</span>
                    <button
                      onClick={() => handleCopy(`${webhookBaseUrl}/api/v1/webhooks/voice/recording`, 'rec_url')}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      {copiedKey === 'rec_url' ? '✓ Copied' : 'Copy'}
                    </button>
                  </div>
                  <input
                    readOnly
                    value={`${webhookBaseUrl}/api/v1/webhooks/voice/recording`}
                    className="block w-full rounded-md bg-slate-50 border border-slate-200 px-3 py-1.5 text-xs font-mono text-slate-700"
                  />
                </div>
              </div>
            </div>
          </div>

          {/* Right Rail: Health & WebRTC Status */}
          <div className="space-y-6">
            <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
              <h3 className="text-xs font-bold text-slate-900 uppercase tracking-wider">Trunk Telemetry</h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-500">Channel Status:</span>
                  <span className="rounded-full bg-emerald-100 px-2 py-0.5 font-semibold text-emerald-800">
                    Active &amp; Operational
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-500">WebRTC Client:</span>
                  <span className="font-semibold text-slate-900">Twilio WebRTC / Opus</span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-500">Concurrent Channels:</span>
                  <span className="font-semibold text-slate-900">25 Trunks</span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-500">Audio Codec:</span>
                  <span className="font-mono text-slate-700">Opus 48kHz / PCMU</span>
                </div>
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-500">ICE / STUN Server:</span>
                  <span className="font-mono text-[11px] text-slate-700">global.stun.twilio.com</span>
                </div>
              </div>
            </div>

            <div className="rounded-lg border border-blue-200 bg-blue-50/50 p-5 shadow-xs space-y-2">
              <h4 className="text-xs font-bold text-blue-900 flex items-center gap-1.5">
                <span>💡</span> Omnichannel Voice Benefits
              </h4>
              <p className="text-[11px] text-blue-800 leading-relaxed">
                When phone calls arrive, the agent inbox creates or links the contact record, records the audio session, and streams an AI transcript with automatic customer sentiment tagging.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: In-Inbox Softphone & Dialer */}
      {activeTab === 'dialpad' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-1">
            <div className="rounded-2xl border border-slate-300 bg-white p-6 shadow-md max-w-sm mx-auto space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-slate-700 uppercase tracking-wider">Browser Softphone</span>
                <span className="h-2.5 w-2.5 rounded-full bg-emerald-500 ring-2 ring-emerald-200" title="Microphone Active"></span>
              </div>

              {/* Number Display Input */}
              <div className="relative">
                <input
                  type="text"
                  value={dialNumber}
                  onChange={(e) => setDialNumber(e.target.value)}
                  placeholder="+1 (555) 000-0000"
                  className="w-full rounded-xl bg-slate-50 border border-slate-300 px-4 py-3 text-lg font-mono font-bold text-slate-900 text-center tracking-wider focus:outline-hidden focus:ring-2 focus:ring-emerald-500"
                />
                {dialNumber && (
                  <button
                    onClick={() => setDialNumber(dialNumber.slice(0, -1))}
                    className="absolute right-3 top-3 text-slate-400 hover:text-slate-600 font-bold"
                  >
                    ⌫
                  </button>
                )}
              </div>

              {/* Keypad Grid */}
              <div className="grid grid-cols-3 gap-2.5 pt-1">
                {[
                  { key: '1', sub: ' ' },
                  { key: '2', sub: 'ABC' },
                  { key: '3', sub: 'DEF' },
                  { key: '4', sub: 'GHI' },
                  { key: '5', sub: 'JKL' },
                  { key: '6', sub: 'MNO' },
                  { key: '7', sub: 'PQRS' },
                  { key: '8', sub: 'TUV' },
                  { key: '9', sub: 'WXYZ' },
                  { key: '*', sub: ' ' },
                  { key: '0', sub: '+' },
                  { key: '#', sub: ' ' },
                ].map((item) => (
                  <button
                    key={item.key}
                    type="button"
                    onClick={() => handleDialKeyPress(item.key)}
                    className="flex flex-col items-center justify-center h-14 rounded-xl border border-slate-200 bg-slate-50 hover:bg-slate-100 active:bg-slate-200 transition-colors shadow-2xs"
                  >
                    <span className="text-lg font-bold text-slate-800">{item.key}</span>
                    <span className="text-[9px] font-semibold text-slate-400 tracking-wider">{item.sub}</span>
                  </button>
                ))}
              </div>

              {/* Action Buttons */}
              <div className="pt-2">
                <button
                  type="button"
                  onClick={() => handleStartCall()}
                  disabled={!dialNumber || !!activeCall}
                  className="w-full rounded-xl bg-emerald-600 hover:bg-emerald-700 active:bg-emerald-800 disabled:opacity-50 py-3 text-sm font-bold text-white shadow-md flex items-center justify-center gap-2 transition-all"
                >
                  <span className="text-base">📞</span> Call Customer
                </button>
              </div>
            </div>
          </div>

          <div className="lg:col-span-2 space-y-6">
            <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
              <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                <span>⚡</span> Quick Click-To-Call Hotlist
              </h2>
              <p className="text-xs text-slate-500">
                Click any contact to instantly initiate an outbound voice call from your agent workspace.
              </p>

              <div className="divide-y divide-slate-100">
                {[
                  { name: 'Maya Chen', company: 'Luxury Living Decor', phone: '+14155552480', type: 'VIP Customer' },
                  { name: 'Lorrie Trosdall', company: 'Northwind Traders', phone: '+14155550100', type: 'Enterprise Client' },
                  { name: 'Morgan Hayes', company: 'Folkard Studio', phone: '+14155550144', type: 'Billing Inquiry' },
                ].map((c) => (
                  <div key={c.phone} className="py-3 flex items-center justify-between">
                    <div>
                      <div className="font-semibold text-xs text-slate-900 flex items-center gap-1.5">
                        <span>{c.name}</span>
                        <span className="rounded-full bg-slate-100 px-2 py-0.5 text-[10px] text-slate-600">{c.type}</span>
                      </div>
                      <div className="text-[11px] text-slate-500">{c.company} · {c.phone}</div>
                    </div>
                    <button
                      onClick={() => handleStartCall(c.phone)}
                      disabled={!!activeCall}
                      className="rounded-md border border-emerald-300 bg-emerald-50 px-3 py-1.5 text-xs font-semibold text-emerald-800 hover:bg-emerald-100 flex items-center gap-1"
                    >
                      <span>📞</span> Dial
                    </button>
                  </div>
                ))}
              </div>
            </div>

            <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-3">
              <h3 className="text-xs font-bold text-slate-900 uppercase tracking-wider">Browser WebRTC Diagnostics</h3>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div className="rounded-md bg-slate-50 p-3 border border-slate-200">
                  <div className="text-[11px] text-slate-500">Microphone Input</div>
                  <div className="text-xs font-bold text-emerald-700 mt-0.5 flex items-center gap-1">
                    <span>✓</span> Permitted &amp; Ready
                  </div>
                </div>
                <div className="rounded-md bg-slate-50 p-3 border border-slate-200">
                  <div className="text-[11px] text-slate-500">WebRTC Capability Token</div>
                  <div className="text-xs font-bold text-emerald-700 mt-0.5 flex items-center gap-1">
                    <span>✓</span> Generated &amp; Signed
                  </div>
                </div>
                <div className="rounded-md bg-slate-50 p-3 border border-slate-200">
                  <div className="text-[11px] text-slate-500">Network Latency (STUN)</div>
                  <div className="text-xs font-bold text-slate-800 mt-0.5">28 ms</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: IVR & Call Routing */}
      {activeTab === 'ivr' && (
        <div className="space-y-6">
          <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                  <span>🎛️</span> Interactive Voice Response (IVR) Auto-Attendant
                </h2>
                <p className="text-xs text-slate-500">
                  Greet callers with customized voice prompts and route calls to specific team queues or voicemail based on DTMF keypad digits.
                </p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={ivrEnabled}
                  onChange={(e) => setIvrEnabled(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-slate-200 peer-focus:outline-hidden rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-emerald-600"></div>
                <span className="ml-2 text-xs font-semibold text-slate-700">IVR Enabled</span>
              </label>
            </div>

            <div>
              <label className="block text-xs font-medium text-slate-700">Spoken Speech Greeting Prompt (TTS)</label>
              <textarea
                rows={3}
                value={ivrPrompt}
                onChange={(e) => setIvrPrompt(e.target.value)}
                className="mt-1 block w-full rounded-md border border-slate-300 p-2.5 text-xs shadow-xs focus:border-emerald-500 focus:outline-hidden"
              />
              <p className="text-[11px] text-slate-400 mt-1">Rendered with Amazon Polly Joanna (Neural Text-to-Speech Engine).</p>
            </div>

            <div className="pt-2">
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-xs font-bold text-slate-800 uppercase tracking-wider">Keypad Digit Mappings</h3>
                <button
                  type="button"
                  onClick={handleAddIvrOption}
                  className="rounded-md border border-slate-300 bg-white px-2.5 py-1 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                >
                  + Add Keypress Option
                </button>
              </div>

              <div className="rounded-md border border-slate-200 overflow-hidden">
                <table className="min-w-full divide-y divide-slate-200 text-xs">
                  <thead className="bg-slate-50">
                    <tr>
                      <th className="px-4 py-2 text-left font-semibold text-slate-700">Digit</th>
                      <th className="px-4 py-2 text-left font-semibold text-slate-700">Option Label</th>
                      <th className="px-4 py-2 text-left font-semibold text-slate-700">Action Type</th>
                      <th className="px-4 py-2 text-left font-semibold text-slate-700">Target Queue / Destination</th>
                      <th className="px-4 py-2 text-right font-semibold text-slate-700">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100 bg-white">
                    {ivrMenu.map((opt, idx) => (
                      <tr key={idx}>
                        <td className="px-4 py-2.5 font-bold font-mono text-slate-900 w-16">
                          <input
                            type="text"
                            value={opt.digit}
                            onChange={(e) => {
                              const updated = [...ivrMenu]
                              updated[idx].digit = e.target.value
                              setIvrMenu(updated)
                            }}
                            className="w-10 rounded-md border border-slate-300 px-2 py-1 text-center font-bold text-xs"
                          />
                        </td>
                        <td className="px-4 py-2.5 font-medium text-slate-800">
                          <input
                            type="text"
                            value={opt.label}
                            onChange={(e) => {
                              const updated = [...ivrMenu]
                              updated[idx].label = e.target.value
                              setIvrMenu(updated)
                            }}
                            className="w-full rounded-md border border-slate-300 px-2 py-1 text-xs"
                          />
                        </td>
                        <td className="px-4 py-2.5 text-slate-600">
                          <select
                            value={opt.action}
                            onChange={(e) => {
                              const updated = [...ivrMenu]
                              updated[idx].action = e.target.value
                              setIvrMenu(updated)
                            }}
                            className="rounded-md border border-slate-300 px-2 py-1 text-xs"
                          >
                            <option value="queue">Route to Team Queue</option>
                            <option value="agent">Direct to Agent</option>
                            <option value="voicemail">Voicemail Drop</option>
                            <option value="forward">Forward Phone Number</option>
                          </select>
                        </td>
                        <td className="px-4 py-2.5 text-slate-600">
                          <input
                            type="text"
                            value={opt.target}
                            onChange={(e) => {
                              const updated = [...ivrMenu]
                              updated[idx].target = e.target.value
                              setIvrMenu(updated)
                            }}
                            className="w-full rounded-md border border-slate-300 px-2 py-1 text-xs font-mono"
                          />
                        </td>
                        <td className="px-4 py-2.5 text-right">
                          <button
                            type="button"
                            onClick={() => handleRemoveIvrOption(idx)}
                            className="text-rose-600 hover:text-rose-800 font-semibold"
                          >
                            Delete
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 4: Recordings & AI Transcriptions */}
      {activeTab === 'recordings' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
              <span>🎙️</span> Dual-Channel Call Recording
            </h2>
            <p className="text-xs text-slate-500">
              Record conversations in high fidelity dual-track audio to isolate the agent voice and customer voice for compliance and review.
            </p>

            <div className="space-y-3 pt-2">
              <label className="flex items-center gap-2.5">
                <input
                  type="checkbox"
                  checked={recordingEnabled}
                  onChange={(e) => setRecordingEnabled(e.target.checked)}
                  className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                />
                <span className="text-xs font-semibold text-slate-800">Record all inbound and outbound calls</span>
              </label>

              <div>
                <label className="block text-xs font-medium text-slate-700">Audio Recording Channels</label>
                <select
                  value={recordingChannels}
                  onChange={(e) => setRecordingChannels(e.target.value)}
                  className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs"
                >
                  <option value="dual">Dual-Channel (Agent &amp; Customer separated)</option>
                  <option value="mono">Mono (Single mixed channel)</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700">Storage Retention Period</label>
                <select defaultValue="365" className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs">
                  <option value="90">90 Days</option>
                  <option value="180">180 Days</option>
                  <option value="365">1 Year (Compliant standard)</option>
                  <option value="indefinite">Indefinite Retention</option>
                </select>
              </div>
            </div>
          </div>

          <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
            <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
              <span>📝</span> Automated AI Speech-To-Text Transcriptions
            </h2>
            <p className="text-xs text-slate-500">
              Convert every recorded call into a timestamped, searchable text transcript and extract key action items automatically.
            </p>

            <div className="space-y-3 pt-2">
              <label className="flex items-center gap-2.5">
                <input
                  type="checkbox"
                  checked={transcriptionEnabled}
                  onChange={(e) => setTranscriptionEnabled(e.target.checked)}
                  className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                />
                <span className="text-xs font-semibold text-slate-800">Enable automatic call transcription</span>
              </label>

              <div>
                <label className="block text-xs font-medium text-slate-700">Speech Recognition Engine</label>
                <select defaultValue="whisper" className="mt-1 block w-full rounded-md border border-slate-300 px-3 py-1.5 text-xs shadow-xs">
                  <option value="whisper">OpenAI Whisper Large-v3 (Multi-lingual)</option>
                  <option value="deepgram">Deepgram Nova-2 (Ultra-fast real-time)</option>
                  <option value="google">Google Cloud Speech-to-Text</option>
                </select>
              </div>

              <label className="flex items-center gap-2.5">
                <input
                  type="checkbox"
                  defaultChecked
                  className="rounded-sm border-slate-300 text-emerald-600 focus:ring-emerald-500"
                />
                <span className="text-xs font-semibold text-slate-800">Generate Captain AI Summary &amp; Sentiment on call end</span>
              </label>
            </div>
          </div>
        </div>
      )}

      {/* TAB 5: Call History & Diagnostics */}
      {activeTab === 'logs' && (
        <div className="space-y-6">
          <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                <span>📋</span> Recent Voice Call Logs
              </h2>
              <button
                onClick={async () => {
                  const logs = await api.getVoiceCallLogs()
                  setCallLogs(logs)
                }}
                className="text-xs text-blue-600 hover:text-blue-800 font-semibold"
              >
                Refresh Logs
              </button>
            </div>

            {callLogs.length === 0 ? (
              <div className="py-8 text-center text-xs text-slate-500">
                No voice calls recorded yet. Use the dialpad or incoming call webhook to test!
              </div>
            ) : (
              <div className="divide-y divide-slate-100">
                {callLogs.map((log) => (
                  <div key={log.id} className="py-3 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider ${
                          log.direction === 'inbound' ? 'bg-blue-100 text-blue-800' : 'bg-purple-100 text-purple-800'
                        }`}>
                          {log.direction}
                        </span>
                        <span className="font-semibold text-xs text-slate-900 font-mono">{log.from} → {log.to}</span>
                        <span className="text-slate-400 text-xs">·</span>
                        <span className="text-xs text-slate-600">Duration: <strong>{log.duration}s</strong></span>
                        <span className={`px-2 py-0.2 rounded-full text-[10px] font-semibold ${
                          log.status === 'completed' ? 'bg-emerald-100 text-emerald-800' : 'bg-slate-100 text-slate-700'
                        }`}>
                          {log.status}
                        </span>
                      </div>
                      <p className="text-xs text-slate-600">{log.summary}</p>
                      {log.transcriptionText && (
                        <div className="bg-slate-50 border border-slate-200 rounded-md p-2 text-xs text-slate-700 italic">
                          "{log.transcriptionText}"
                        </div>
                      )}
                    </div>

                    {log.recordingUrl && (
                      <div className="shrink-0 flex items-center gap-2">
                        <audio controls src={log.recordingUrl} className="h-8 max-w-[200px]" />
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Webhook Events Stream */}
          <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-xs space-y-4">
            <h3 className="text-xs font-bold text-slate-900 uppercase tracking-wider">Live Webhook Events Stream</h3>
            <div className="rounded-md bg-slate-900 text-slate-200 p-4 font-mono text-xs max-h-60 overflow-y-auto space-y-1.5">
              {events.length === 0 ? (
                <div className="text-slate-500">No events received yet. Listening on /api/v1/webhooks/voice/*</div>
              ) : (
                events.map((ev) => (
                  <div key={ev.id} className="text-slate-300">
                    <span className="text-emerald-400">[{ev.sentAt}]</span>{' '}
                    <span className="text-blue-300">[{ev.direction.toUpperCase()}]</span>{' '}
                    <span className="text-amber-300">({ev.provider})</span>:{' '}
                    <span>{ev.body}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
