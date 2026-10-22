import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import type {
  AgentReportItem,
  ConversationReportData,
  CsatReportData,
  LiveReportData,
  TeamReportItem,
} from '../lib/api'

type TabType = 'live' | 'conversations' | 'agents' | 'teams' | 'csat'
type PeriodType = '7d' | '30d' | '90d'

export function Reports() {
  const [activeTab, setActiveTab] = useState<TabType>('live')
  const [period, setPeriod] = useState<PeriodType>('7d')
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)

  const [liveData, setLiveData] = useState<LiveReportData | null>(null)
  const [convData, setConvData] = useState<ConversationReportData | null>(null)
  const [agentsData, setAgentsData] = useState<AgentReportItem[]>([])
  const [teamsData, setTeamsData] = useState<TeamReportItem[]>([])
  const [csatData, setCsatData] = useState<CsatReportData | null>(null)

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      if (activeTab === 'live') {
        const res = await api.getLiveReport()
        setLiveData(res)
      } else if (activeTab === 'conversations') {
        const res = await api.getConversationReports(period)
        setConvData(res)
      } else if (activeTab === 'agents') {
        const res = await api.getAgentReports(period)
        setAgentsData(res)
      } else if (activeTab === 'teams') {
        const res = await api.getTeamReports(period)
        setTeamsData(res)
      } else if (activeTab === 'csat') {
        const res = await api.getCsatReport(period)
        setCsatData(res)
      }
    } catch (err: any) {
      console.error('Failed to load reporting data:', err)
      setError(err?.message || 'Failed to load report data from server')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [activeTab, period])

  return (
    <div className="p-6 max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-b border-slate-200 pb-5">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold tracking-tight text-slate-900">Reporting &amp; Analytics</h1>
            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-emerald-100 text-emerald-800">
              Live Operations
            </span>
          </div>
          <p className="text-sm text-slate-500 mt-1">
            Real-time contact center telemetry, agent SLA leaderboards, channel distributions, and CSAT customer surveys.
          </p>
        </div>

        <div className="flex items-center gap-3">
          {activeTab !== 'live' && (
            <div className="inline-flex rounded-lg border border-slate-200 bg-white p-1 text-xs font-medium text-slate-600 shadow-sm">
              {(['7d', '30d', '90d'] as PeriodType[]).map((p) => (
                <button
                  key={p}
                  onClick={() => setPeriod(p)}
                  className={`px-3 py-1.5 rounded-md transition-colors ${
                    period === p
                      ? 'bg-slate-900 text-white font-semibold'
                      : 'hover:text-slate-900 hover:bg-slate-50'
                  }`}
                >
                  {p === '7d' ? 'Last 7 Days' : p === '30d' ? 'Last 30 Days' : 'Last 90 Days'}
                </button>
              ))}
            </div>
          )}

          <button
            onClick={loadData}
            disabled={loading}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-slate-200 bg-white text-xs font-medium text-slate-700 shadow-sm hover:bg-slate-50 disabled:opacity-50"
          >
            <svg
              className={`w-3.5 h-3.5 text-slate-500 ${loading ? 'animate-spin' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            Refresh
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-slate-200 gap-6 text-sm font-medium">
        <button
          onClick={() => setActiveTab('live')}
          className={`pb-3 -mb-px flex items-center gap-2 border-b-2 transition-colors ${
            activeTab === 'live'
              ? 'border-indigo-600 text-indigo-600 font-semibold'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          Live Operations
        </button>

        <button
          onClick={() => setActiveTab('conversations')}
          className={`pb-3 -mb-px border-b-2 transition-colors ${
            activeTab === 'conversations'
              ? 'border-indigo-600 text-indigo-600 font-semibold'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          Conversation Volume &amp; Trends
        </button>

        <button
          onClick={() => setActiveTab('agents')}
          className={`pb-3 -mb-px border-b-2 transition-colors ${
            activeTab === 'agents'
              ? 'border-indigo-600 text-indigo-600 font-semibold'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          Agent Leaderboard
        </button>

        <button
          onClick={() => setActiveTab('teams')}
          className={`pb-3 -mb-px border-b-2 transition-colors ${
            activeTab === 'teams'
              ? 'border-indigo-600 text-indigo-600 font-semibold'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          Team Performance
        </button>

        <button
          onClick={() => setActiveTab('csat')}
          className={`pb-3 -mb-px border-b-2 transition-colors ${
            activeTab === 'csat'
              ? 'border-indigo-600 text-indigo-600 font-semibold'
              : 'border-transparent text-slate-500 hover:text-slate-800'
          }`}
        >
          CSAT Satisfaction
        </button>
      </div>

      {/* Error state */}
      {error && (
        <div className="p-4 rounded-lg bg-rose-50 border border-rose-200 text-rose-700 text-sm">
          {error}
        </div>
      )}

      {/* TAB 1: LIVE OPERATIONS VIEW */}
      {activeTab === 'live' && liveData && (
        <div className="space-y-6">
          {/* Key Stat Cards */}
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Open Conversations</span>
              <div className="mt-2 text-2xl font-bold text-slate-900">{liveData.liveMetrics.openConversations}</div>
              <span className="text-xs text-slate-400 mt-1 inline-block">Across all active channels</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-amber-600 uppercase tracking-wider">Unassigned Queue</span>
              <div className="mt-2 text-2xl font-bold text-amber-700">{liveData.liveMetrics.unassignedConversations}</div>
              <span className="text-xs text-amber-600/80 mt-1 inline-block">Awaiting agent claim</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-indigo-600 uppercase tracking-wider">Pending Reply</span>
              <div className="mt-2 text-2xl font-bold text-indigo-700">{liveData.liveMetrics.pendingReplies}</div>
              <span className="text-xs text-indigo-600/80 mt-1 inline-block">Customer sent last message</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-emerald-600 uppercase tracking-wider">Resolved Today</span>
              <div className="mt-2 text-2xl font-bold text-emerald-700">{liveData.liveMetrics.resolvedToday}</div>
              <span className="text-xs text-emerald-600/80 mt-1 inline-block">Closed or converted</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Avg Wait in Queue</span>
              <div className="mt-2 text-2xl font-bold text-slate-900">{liveData.liveMetrics.avgWaitTimeFormatted}</div>
              <span className="text-xs text-slate-400 mt-1 inline-block">Target SLA &lt; 5m</span>
            </div>
          </div>

          {/* Operational Split: Channels & Agent Workloads */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Live Channel Breakdown */}
            <div className="bg-white rounded-xl border border-slate-200 p-5 shadow-sm space-y-4">
              <h2 className="text-base font-semibold text-slate-900">Live Traffic by Channel</h2>
              <div className="space-y-3">
                {Object.entries(liveData.channelBreakdown).map(([ch, count]) => {
                  const total = liveData.liveMetrics.openConversations || 1
                  const pct = Math.min(100, Math.round((count / total) * 100))
                  return (
                    <div key={ch} className="space-y-1">
                      <div className="flex justify-between text-xs font-medium">
                        <span className="capitalize text-slate-700">{ch}</span>
                        <span className="text-slate-500">{count} chats ({pct}%)</span>
                      </div>
                      <div className="w-full h-2 rounded-full bg-slate-100 overflow-hidden">
                        <div
                          className="h-full bg-indigo-500 rounded-full transition-all"
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>

            {/* Agent Workload & Capacity */}
            <div className="lg:col-span-2 bg-white rounded-xl border border-slate-200 p-5 shadow-sm space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-base font-semibold text-slate-900">Agent Concurrency &amp; Capacity</h2>
                  <p className="text-xs text-slate-500 mt-0.5">Real-time load balancing and agent active seat utilization</p>
                </div>
                <span className="text-xs px-2.5 py-1 bg-slate-100 rounded-md font-medium text-slate-700">
                  {liveData.agentsOnline.length} Teammates Active
                </span>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {liveData.agentsOnline.map((agent) => {
                  const capPct = Math.min(100, Math.round((agent.activeChats / agent.maxCapacity) * 100))
                  const isBusy = agent.activeChats >= agent.maxCapacity
                  return (
                    <div key={agent.id} className="p-4 rounded-lg border border-slate-200 bg-slate-50/50 space-y-3">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2.5">
                          <div className="relative">
                            <div className="w-8 h-8 rounded-full bg-indigo-100 text-indigo-700 font-semibold flex items-center justify-center text-xs">
                              {agent.name.split(' ').map((n) => n[0]).join('')}
                            </div>
                            <span
                              className={`absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full ring-2 ring-white ${
                                isBusy ? 'bg-amber-500' : 'bg-emerald-500'
                              }`}
                            />
                          </div>
                          <div>
                            <div className="text-sm font-semibold text-slate-900">{agent.name}</div>
                            <div className="text-xs text-slate-500">{agent.email}</div>
                          </div>
                        </div>
                        <span
                          className={`text-xs px-2 py-0.5 rounded font-medium ${
                            isBusy
                              ? 'bg-amber-100 text-amber-800'
                              : 'bg-emerald-100 text-emerald-800'
                          }`}
                        >
                          {isBusy ? 'At Capacity' : 'Available'}
                        </span>
                      </div>

                      <div className="space-y-1">
                        <div className="flex justify-between text-xs text-slate-600">
                          <span>Active Conversations</span>
                          <span className="font-semibold text-slate-900">
                            {agent.activeChats} / {agent.maxCapacity} chats
                          </span>
                        </div>
                        <div className="w-full h-1.5 rounded-full bg-slate-200 overflow-hidden">
                          <div
                            className={`h-full rounded-full ${
                              capPct >= 80 ? 'bg-amber-500' : 'bg-indigo-600'
                            }`}
                            style={{ width: `${capPct}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: CONVERSATION VOLUME & TRENDS */}
      {activeTab === 'conversations' && convData && (
        <div className="space-y-6">
          {/* Summary KPI Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4">
            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Total Volume</span>
              <div className="mt-2 text-2xl font-bold text-slate-900">{convData.summary.totalConversations}</div>
              <span className="text-xs text-slate-400 mt-1 inline-block">{period} span</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Total Messages</span>
              <div className="mt-2 text-2xl font-bold text-slate-900">{convData.summary.totalMessages}</div>
              <span className="text-xs text-slate-400 mt-1 inline-block">
                In: {convData.summary.inboundMessages} | Out: {convData.summary.outboundMessages}
              </span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-emerald-600 uppercase tracking-wider">Resolution Rate</span>
              <div className="mt-2 text-2xl font-bold text-emerald-700">{convData.summary.resolutionRatePercent}%</div>
              <span className="text-xs text-emerald-600/80 mt-1 inline-block">Target &gt; 85%</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-indigo-600 uppercase tracking-wider">Avg First Response</span>
              <div className="mt-2 text-2xl font-bold text-indigo-700">{convData.summary.avgFirstResponseMinutes}m</div>
              <span className="text-xs text-indigo-600/80 mt-1 inline-block">First reply speed</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Avg Resolution Time</span>
              <div className="mt-2 text-2xl font-bold text-slate-900">{convData.summary.avgResolutionHours}h</div>
              <span className="text-xs text-slate-400 mt-1 inline-block">Full resolution</span>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
              <span className="text-xs font-medium text-purple-600 uppercase tracking-wider">Inbound Ratio</span>
              <div className="mt-2 text-2xl font-bold text-purple-700">
                {Math.round((convData.summary.inboundMessages / (convData.summary.totalMessages || 1)) * 100)}%
              </div>
              <span className="text-xs text-purple-600/80 mt-1 inline-block">Organic incoming</span>
            </div>
          </div>

          {/* Time Series Volume Chart */}
          <div className="bg-white rounded-xl border border-slate-200 p-6 shadow-sm space-y-4">
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
              <div>
                <h2 className="text-base font-semibold text-slate-900">Conversation &amp; Resolution Trajectory</h2>
                <p className="text-xs text-slate-500">Daily inbound conversation creation vs ticket resolutions</p>
              </div>
              <div className="flex items-center gap-4 text-xs">
                <span className="flex items-center gap-1.5 text-slate-600 font-medium">
                  <span className="w-3 h-3 rounded-sm bg-indigo-500 inline-block" /> Conversations
                </span>
                <span className="flex items-center gap-1.5 text-slate-600 font-medium">
                  <span className="w-3 h-3 rounded-sm bg-emerald-500 inline-block" /> Resolutions
                </span>
              </div>
            </div>

            {/* SVG Visual Bar/Trend Chart */}
            <div className="pt-4">
              <div className="h-48 flex items-end gap-2 border-b border-slate-200 pb-2">
                {convData.timeSeries.map((point, idx) => {
                  const maxVal = Math.max(...convData.timeSeries.map((p) => p.conversations), 20)
                  const convHeight = Math.max(8, Math.round((point.conversations / maxVal) * 160))
                  const resHeight = Math.max(6, Math.round((point.resolutions / maxVal) * 160))
                  return (
                    <div key={idx} className="flex-1 flex flex-col items-center gap-1 h-full justify-end group">
                      <div className="w-full flex items-end justify-center gap-1">
                        <div
                          className="w-1/2 max-w-[14px] bg-indigo-500 rounded-t hover:bg-indigo-600 transition-all cursor-pointer relative"
                          style={{ height: `${convHeight}px` }}
                          title={`${point.date}: ${point.conversations} convs`}
                        />
                        <div
                          className="w-1/2 max-w-[14px] bg-emerald-500 rounded-t hover:bg-emerald-600 transition-all cursor-pointer"
                          style={{ height: `${resHeight}px` }}
                          title={`${point.date}: ${point.resolutions} resolved`}
                        />
                      </div>
                      <span className="text-[10px] text-slate-400 truncate w-full text-center group-hover:text-slate-800">
                        {point.date}
                      </span>
                    </div>
                  )
                })}
              </div>
            </div>
          </div>

          {/* Channel Distribution Table */}
          <div className="bg-white rounded-xl border border-slate-200 overflow-hidden shadow-sm">
            <div className="p-4 border-b border-slate-200 bg-slate-50/50">
              <h2 className="text-sm font-semibold text-slate-900">Traffic Distribution by Channel</h2>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-sm text-slate-600">
                <thead className="bg-slate-50 text-xs uppercase font-semibold text-slate-500 border-b border-slate-200">
                  <tr>
                    <th className="px-5 py-3">Channel Name</th>
                    <th className="px-5 py-3">Conversations</th>
                    <th className="px-5 py-3">Traffic Share</th>
                    <th className="px-5 py-3">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {Object.entries(convData.channelDistribution).map(([ch, count]) => {
                    const share = Math.round((count / (convData.summary.totalConversations || 1)) * 100)
                    return (
                      <tr key={ch} className="hover:bg-slate-50/60">
                        <td className="px-5 py-3.5 font-medium text-slate-900 capitalize flex items-center gap-2">
                          <span className="w-2 h-2 rounded-full bg-indigo-500" />
                          {ch}
                        </td>
                        <td className="px-5 py-3.5 text-slate-900 font-semibold">{count}</td>
                        <td className="px-5 py-3.5">
                          <div className="flex items-center gap-2">
                            <span className="text-xs font-medium w-8">{share}%</span>
                            <div className="w-24 h-1.5 rounded-full bg-slate-100 overflow-hidden">
                              <div className="h-full bg-indigo-600 rounded-full" style={{ width: `${share}%` }} />
                            </div>
                          </div>
                        </td>
                        <td className="px-5 py-3.5">
                          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-emerald-100 text-emerald-800">
                            Active
                          </span>
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: AGENT LEADERBOARD */}
      {activeTab === 'agents' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border border-slate-200 overflow-hidden shadow-sm">
            <div className="p-5 border-b border-slate-200 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
              <div>
                <h2 className="text-base font-semibold text-slate-900">Teammate Performance Leaderboard</h2>
                <p className="text-xs text-slate-500 mt-0.5">Ranked by resolution speed, customer satisfaction score, and workload handling</p>
              </div>
              <span className="text-xs text-slate-500 font-medium">Period: {period}</span>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full text-left text-sm text-slate-600">
                <thead className="bg-slate-50 text-xs uppercase font-semibold text-slate-500 border-b border-slate-200">
                  <tr>
                    <th className="px-5 py-3">Agent</th>
                    <th className="px-5 py-3">Assigned</th>
                    <th className="px-5 py-3">Resolved</th>
                    <th className="px-5 py-3">Resolution Rate</th>
                    <th className="px-5 py-3">Avg First Response</th>
                    <th className="px-5 py-3">Avg Resolution</th>
                    <th className="px-5 py-3">CSAT Score</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {agentsData.map((agent, i) => (
                    <tr key={agent.id} className="hover:bg-slate-50/70">
                      <td className="px-5 py-3.5">
                        <div className="flex items-center gap-3">
                          <img
                            src={agent.avatar}
                            alt={agent.name}
                            className="w-9 h-9 rounded-full object-cover border border-slate-200 shadow-xs"
                          />
                          <div>
                            <div className="font-semibold text-slate-900 flex items-center gap-2">
                              {agent.name}
                              {i === 0 && (
                                <span className="text-[10px] bg-amber-100 text-amber-800 px-1.5 py-0.2 rounded font-bold">
                                  Top
                                </span>
                              )}
                            </div>
                            <div className="text-xs text-slate-400">{agent.email}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-3.5 font-medium text-slate-900">{agent.conversationsAssigned}</td>
                      <td className="px-5 py-3.5 font-medium text-emerald-700">{agent.conversationsResolved}</td>
                      <td className="px-5 py-3.5">
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200">
                          {agent.resolutionRate}
                        </span>
                      </td>
                      <td className="px-5 py-3.5 font-medium text-indigo-700">{agent.avgFirstResponseMinutes}m</td>
                      <td className="px-5 py-3.5 text-slate-700">{agent.avgResolutionHours}h</td>
                      <td className="px-5 py-3.5">
                        <div className="flex items-center gap-1.5">
                          <span className="text-amber-500 font-bold">★</span>
                          <span className="font-bold text-slate-900">{agent.csatRating}</span>
                          <span className="text-xs text-slate-400">({agent.csatResponsesCount} ratings)</span>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* TAB 4: TEAM PERFORMANCE */}
      {activeTab === 'teams' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {teamsData.map((team) => (
              <div key={team.id} className="bg-white rounded-xl border border-slate-200 p-5 shadow-sm space-y-4">
                <div className="flex items-start justify-between">
                  <div>
                    <h3 className="font-bold text-slate-900 text-base">{team.name}</h3>
                    <p className="text-xs text-slate-500 mt-0.5 line-clamp-2">{team.description}</p>
                  </div>
                  <span className="text-xs px-2.5 py-1 bg-emerald-50 border border-emerald-200 rounded-md font-semibold text-emerald-800">
                    SLA: {team.slaAdherencePercent}
                  </span>
                </div>

                <div className="grid grid-cols-2 gap-3 pt-2 border-t border-slate-100">
                  <div className="bg-slate-50 rounded-lg p-2.5">
                    <span className="text-[11px] text-slate-500 font-medium uppercase">Total Handled</span>
                    <div className="text-lg font-bold text-slate-900 mt-1">{team.conversationsCount} chats</div>
                  </div>
                  <div className="bg-slate-50 rounded-lg p-2.5">
                    <span className="text-[11px] text-slate-500 font-medium uppercase">Unassigned Queue</span>
                    <div className="text-lg font-bold text-amber-600 mt-1">{team.unassignedCount} waiting</div>
                  </div>
                  <div className="bg-slate-50 rounded-lg p-2.5">
                    <span className="text-[11px] text-slate-500 font-medium uppercase">Resolution Rate</span>
                    <div className="text-lg font-bold text-emerald-700 mt-1">{team.resolutionRate}</div>
                  </div>
                  <div className="bg-slate-50 rounded-lg p-2.5">
                    <span className="text-[11px] text-slate-500 font-medium uppercase">Avg First Reply</span>
                    <div className="text-lg font-bold text-indigo-700 mt-1">{team.avgResponseMinutes}</div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* TAB 5: CSAT SATISFACTION */}
      {activeTab === 'csat' && csatData && (
        <div className="space-y-6">
          {/* Top CSAT Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-white rounded-xl border border-slate-200 p-5 shadow-sm flex items-center gap-4">
              <div className="w-14 h-14 rounded-2xl bg-amber-50 border border-amber-100 flex items-center justify-center text-amber-500 text-3xl font-extrabold">
                ★
              </div>
              <div>
                <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Average Rating</span>
                <div className="text-3xl font-extrabold text-slate-900 mt-0.5">
                  {csatData.metrics.averageScore} <span className="text-base font-normal text-slate-400">/ 5.0</span>
                </div>
                <span className="text-xs text-slate-400">{csatData.metrics.totalResponses} verified customer survey ratings</span>
              </div>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-5 shadow-sm flex items-center gap-4">
              <div className="w-14 h-14 rounded-2xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600 text-2xl font-bold">
                %
              </div>
              <div>
                <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Customer Satisfaction</span>
                <div className="text-3xl font-extrabold text-emerald-700 mt-0.5">
                  {csatData.metrics.satisfactionPercentage}%
                </div>
                <span className="text-xs text-emerald-600/80">Percentage of 4★ and 5★ reviews</span>
              </div>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 p-5 shadow-sm flex items-center gap-4">
              <div className="w-14 h-14 rounded-2xl bg-indigo-50 border border-indigo-100 flex items-center justify-center text-indigo-600 text-2xl font-bold">
                ✓
              </div>
              <div>
                <span className="text-xs font-medium text-slate-500 uppercase tracking-wider">Total Responses</span>
                <div className="text-3xl font-extrabold text-slate-900 mt-0.5">
                  {csatData.metrics.totalResponses}
                </div>
                <span className="text-xs text-indigo-600/80">Automated CSAT post-resolution trigger</span>
              </div>
            </div>
          </div>

          {/* Rating Breakdown & Customer Feedback Quotes */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Star Distribution */}
            <div className="bg-white rounded-xl border border-slate-200 p-5 shadow-sm space-y-4">
              <h2 className="text-base font-semibold text-slate-900">Score Distribution</h2>
              <div className="space-y-2.5">
                {[5, 4, 3, 2, 1].map((stars) => {
                  const count = csatData.metrics.distribution[String(stars)] || 0
                  const total = csatData.metrics.totalResponses || 1
                  const pct = Math.round((count / total) * 100)
                  return (
                    <div key={stars} className="flex items-center gap-3 text-xs">
                      <span className="w-12 font-semibold text-slate-700 flex items-center gap-1">
                        {stars} <span className="text-amber-500">★</span>
                      </span>
                      <div className="flex-1 h-2 rounded-full bg-slate-100 overflow-hidden">
                        <div
                          className={`h-full rounded-full ${
                            stars >= 4 ? 'bg-emerald-500' : stars === 3 ? 'bg-amber-400' : 'bg-rose-400'
                          }`}
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                      <span className="w-10 text-right text-slate-500 font-medium">{pct}%</span>
                      <span className="w-8 text-right text-slate-400 text-[11px]">({count})</span>
                    </div>
                  )
                })}
              </div>
            </div>

            {/* Testimonials */}
            <div className="lg:col-span-2 bg-white rounded-xl border border-slate-200 p-5 shadow-sm space-y-4">
              <h2 className="text-base font-semibold text-slate-900">Recent Customer Feedback</h2>
              <div className="space-y-3">
                {csatData.recentFeedback.map((fb, idx) => (
                  <div key={idx} className="p-4 rounded-lg border border-slate-200 bg-slate-50/50 space-y-2">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-slate-900 text-sm">{fb.contactName}</span>
                        <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-slate-200 text-slate-700 uppercase">
                          {fb.channel}
                        </span>
                        <span className="text-xs text-slate-400">• Agent: {fb.assignedAgent}</span>
                      </div>
                      <div className="flex items-center gap-1 text-amber-500 font-bold text-xs">
                        {'★'.repeat(fb.score)}
                        <span className="text-slate-400 ml-1 text-[11px] font-normal">{fb.date}</span>
                      </div>
                    </div>
                    <p className="text-sm text-slate-700 italic">"{fb.comment}"</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
