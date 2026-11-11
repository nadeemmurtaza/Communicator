import { useState, useEffect } from 'react'
import {
  getTranslationLanguages, getTranslationSettings, updateTranslationSettings,
  detectLanguage, translateMessage, prepareOutboundTranslation
} from '../lib/api'
import type { TranslationResult } from '../lib/api'
import {
  Globe, Languages, Settings, Search, Play, CheckCircle,
  AlertCircle, MessageSquare, ArrowRightLeft, RefreshCw
} from '../components/icons'

interface Language {
  code: string
  name: string
  native: string
  dir: string
  curated: boolean
}

interface TranslationSettings {
  enabled: boolean
  autoTranslate: boolean
  defaultTargetLanguage: string
  translationApiUrl: string
  translationApiKey: string
  detectLanguage: boolean
  showOriginal: boolean
  fallbackLanguage: string
}

export default function TranslationDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'languages' | 'detect' | 'translate' | 'settings'>('overview')
  const [languages, setLanguages] = useState<Language[]>([])
  const [settings, setSettings] = useState<TranslationSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  // Language detection
  const [detectText, setDetectText] = useState('')
  const [detectResult, setDetectResult] = useState<{ language: string; name: string; confidence: number; script: string } | null>(null)
  const [detecting, setDetecting] = useState(false)

  // Translation
  const [translateText, setTranslateText] = useState('')
  const [targetLang, setTargetLang] = useState('en')
  const [translateResult, setTranslateResult] = useState<TranslationResult | null>(null)
  const [translating, setTranslating] = useState(false)

  // Stats
  const [stats, setStats] = useState({
    totalLanguages: 0,
    curatedLanguages: 0,
    enabled: false,
    autoTranslate: false
  })

  useEffect(() => {
    loadData()
  }, [])

  async function loadData() {
    setLoading(true)
    try {
      const [langs, settingsData] = await Promise.all([
        getTranslationLanguages(),
        getTranslationSettings()
      ])
      setLanguages(langs.languages || [])
      setSettings(settingsData)
      setStats({
        totalLanguages: langs.total || 0,
        curatedLanguages: (langs.languages || []).filter((l: Language) => l.curated).length,
        enabled: settingsData?.enabled || false,
        autoTranslate: settingsData?.autoTranslate || false
      })
      if (settingsData?.defaultTargetLanguage) {
        setTargetLang(settingsData.defaultTargetLanguage)
      }
    } catch (e) {
      console.error('Failed to load translation data:', e)
    }
    setLoading(false)
  }

  async function handleDetect() {
    if (!detectText.trim()) return
    setDetecting(true)
    try {
      const result = await detectLanguage(detectText)
      setDetectResult(result)
    } catch (e) {
      console.error('Failed to detect language:', e)
    }
    setDetecting(false)
  }

  async function handleTranslate() {
    if (!translateText.trim()) return
    setTranslating(true)
    try {
      const result = await prepareOutboundTranslation('test', translateText, targetLang)
      setTranslateResult({
        original_text: translateText,
        translated_text: result.translated_text,
        source_language: result.source_language || 'unknown',
        target_language: result.target_language || targetLang,
        confidence: result.confidence || 0
      })
    } catch (e) {
      console.error('Failed to translate:', e)
    }
    setTranslating(false)
  }

  async function handleSaveSettings() {
    if (!settings) return
    setSaving(true)
    try {
      await updateTranslationSettings(settings)
      await loadData()
    } catch (e) {
      console.error('Failed to save settings:', e)
    }
    setSaving(false)
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[var(--color-accent)] mx-auto mb-4" />
          <p className="text-sm text-[var(--color-text-muted)]">Loading translation settings...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-text)]">Translation</h1>
          <p className="text-sm text-[var(--color-text-muted)] mt-1">
            Manage multilingual support, language detection, and auto-translation settings
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={loadData}
            className="inline-flex items-center gap-2 rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-bg-hover)]"
          >
            <RefreshCw className="h-4 w-4" />
            Refresh
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-blue-100 p-2 dark:bg-blue-900/30">
              <Globe className="h-5 w-5 text-blue-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{stats.totalLanguages}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Total Languages</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-green-100 p-2 dark:bg-green-900/30">
              <CheckCircle className="h-5 w-5 text-green-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{stats.curatedLanguages}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Curated Languages</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className={`rounded-lg p-2 ${stats.enabled ? 'bg-green-100 dark:bg-green-900/30' : 'bg-red-100 dark:bg-red-900/30'}`}>
              {stats.enabled ? <CheckCircle className="h-5 w-5 text-green-600" /> : <AlertCircle className="h-5 w-5 text-red-600" />}
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{stats.enabled ? 'On' : 'Off'}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Translation</p>
            </div>
          </div>
        </div>
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4">
          <div className="flex items-center gap-3">
            <div className={`rounded-lg p-2 ${stats.autoTranslate ? 'bg-purple-100 dark:bg-purple-900/30' : 'bg-gray-100 dark:bg-gray-800'}`}>
              <ArrowRightLeft className={`h-5 w-5 ${stats.autoTranslate ? 'text-purple-600' : 'text-gray-600'}`} />
            </div>
            <div>
              <p className="text-2xl font-bold text-[var(--color-text)]">{stats.autoTranslate ? 'On' : 'Off'}</p>
              <p className="text-xs text-[var(--color-text-muted)]">Auto-Translate</p>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-[var(--color-border)]">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'overview' as const, label: 'Overview', icon: Globe },
            { id: 'languages' as const, label: 'Languages', icon: Languages },
            { id: 'detect' as const, label: 'Detect Language', icon: Search },
            { id: 'translate' as const, label: 'Translate', icon: MessageSquare },
            { id: 'settings' as const, label: 'Settings', icon: Settings },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`inline-flex items-center gap-2 border-b-2 px-1 pb-4 text-sm font-medium ${
                activeTab === tab.id
                  ? 'border-[var(--color-accent)] text-[var(--color-accent)]'
                  : 'border-transparent text-[var(--color-text-muted)] hover:border-[var(--color-border)] hover:text-[var(--color-text)]'
              }`}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Translation Overview</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h3 className="text-sm font-medium text-[var(--color-text-muted)] mb-3">Supported Languages</h3>
                <div className="space-y-2">
                  {languages.slice(0, 10).map((lang) => (
                    <div key={lang.code} className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-3 py-2">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-[var(--color-text)]">{lang.name}</span>
                        <span className="text-xs text-[var(--color-text-muted)]">{lang.native}</span>
                        {lang.curated && (
                          <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400">
                            Curated
                          </span>
                        )}
                      </div>
                      <span className="text-xs text-[var(--color-text-muted)]">{lang.dir}</span>
                    </div>
                  ))}
                  {languages.length > 10 && (
                    <p className="text-xs text-[var(--color-text-muted)]">
                      +{languages.length - 10} more languages...
                    </p>
                  )}
                </div>
              </div>
              <div>
                <h3 className="text-sm font-medium text-[var(--color-text-muted)] mb-3">Current Settings</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-3 py-2">
                    <span className="text-sm text-[var(--color-text)]">Translation Enabled</span>
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${settings?.enabled ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'}`}>
                      {settings?.enabled ? 'Yes' : 'No'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-3 py-2">
                    <span className="text-sm text-[var(--color-text)]">Auto-Translate</span>
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${settings?.autoTranslate ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400'}`}>
                      {settings?.autoTranslate ? 'Yes' : 'No'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-3 py-2">
                    <span className="text-sm text-[var(--color-text)]">Default Target</span>
                    <span className="text-sm font-medium text-[var(--color-text)]">
                      {languages.find((l) => l.code === settings?.defaultTargetLanguage)?.name || 'English'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] px-3 py-2">
                    <span className="text-sm text-[var(--color-text)]">Language Detection</span>
                    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${settings?.detectLanguage ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400'}`}>
                      {settings?.detectLanguage ? 'Yes' : 'No'}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* How it works */}
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">How Translation Works</h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Search className="h-5 w-5 text-blue-600" />
                  <h3 className="font-medium text-[var(--color-text)]">1. Detect</h3>
                </div>
                <p className="text-sm text-[var(--color-text-muted)]">
                  When a customer sends a message, the system automatically detects the language using script-level markers and keyword analysis.
                </p>
              </div>
              <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                <div className="flex items-center gap-2 mb-2">
                  <ArrowRightLeft className="h-5 w-5 text-purple-600" />
                  <h3 className="font-medium text-[var(--color-text)]">2. Translate</h3>
                </div>
                <p className="text-sm text-[var(--color-text-muted)]">
                  Inbound messages are translated to your default language. Outbound messages are auto-translated to the customer's language.
                </p>
              </div>
              <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                <div className="flex items-center gap-2 mb-2">
                  <MessageSquare className="h-5 w-5 text-green-600" />
                  <h3 className="font-medium text-[var(--color-text)]">3. Respond</h3>
                </div>
                <p className="text-sm text-[var(--color-text-muted)]">
                  AI generates responses in the customer's language, ensuring seamless multilingual conversations without manual intervention.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Languages Tab */}
      {activeTab === 'languages' && (
        <div className="space-y-4">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-[var(--color-border)]">
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Code</th>
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Name</th>
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Native Name</th>
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Direction</th>
                    <th className="text-left text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider px-4 py-3">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-border)]">
                  {languages.map((lang) => (
                    <tr key={lang.code} className="hover:bg-[var(--color-bg-hover)]">
                      <td className="px-4 py-3 text-sm font-mono text-[var(--color-text)]">{lang.code}</td>
                      <td className="px-4 py-3 text-sm text-[var(--color-text)]">{lang.name}</td>
                      <td className="px-4 py-3 text-sm text-[var(--color-text)]">{lang.native}</td>
                      <td className="px-4 py-3 text-sm text-[var(--color-text-muted)]">{lang.dir}</td>
                      <td className="px-4 py-3">
                        {lang.curated ? (
                          <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400">
                            Curated
                          </span>
                        ) : (
                          <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-700 dark:bg-gray-800 dark:text-gray-400">
                            Detect Only
                          </span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Detect Language Tab */}
      {activeTab === 'detect' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Language Detection</h2>
            <p className="text-sm text-[var(--color-text-muted)] mb-4">
              Enter text to detect its language. The system uses script-level markers and keyword analysis for accurate detection.
            </p>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                  Text to Analyze
                </label>
                <textarea
                  value={detectText}
                  onChange={(e) => setDetectText(e.target.value)}
                  rows={4}
                  className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent)] focus:outline-none"
                  placeholder="Enter text in any language..."
                />
              </div>
              <button
                onClick={handleDetect}
                disabled={detecting || !detectText.trim()}
                className="inline-flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90 disabled:opacity-50"
              >
                {detecting ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent" />
                    Detecting...
                  </>
                ) : (
                  <>
                    <Search className="h-4 w-4" />
                    Detect Language
                  </>
                )}
              </button>
              {detectResult && (
                <div className="rounded-lg bg-green-50 border border-green-200 p-4 dark:bg-green-900/20 dark:border-green-800">
                  <div className="flex items-center gap-2 mb-2">
                    <CheckCircle className="h-5 w-5 text-green-600" />
                    <h3 className="font-medium text-green-800 dark:text-green-400">Detection Result</h3>
                  </div>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-[var(--color-text-muted)]">Language:</span>
                      <span className="ml-2 font-medium text-[var(--color-text)]">{detectResult.name}</span>
                    </div>
                    <div>
                      <span className="text-[var(--color-text-muted)]">Code:</span>
                      <span className="ml-2 font-mono text-[var(--color-text)]">{detectResult.language}</span>
                    </div>
                    <div>
                      <span className="text-[var(--color-text-muted)]">Confidence:</span>
                      <span className="ml-2 font-medium text-[var(--color-text)]">{Math.round(detectResult.confidence * 100)}%</span>
                    </div>
                    <div>
                      <span className="text-[var(--color-text-muted)]">Script:</span>
                      <span className="ml-2 font-medium text-[var(--color-text)]">{detectResult.script}</span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Translate Tab */}
      {activeTab === 'translate' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Test Translation</h2>
            <p className="text-sm text-[var(--color-text-muted)] mb-4">
              Test the translation engine by translating text to any supported language.
            </p>
            <div className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                    Source Text
                  </label>
                  <textarea
                    value={translateText}
                    onChange={(e) => setTranslateText(e.target.value)}
                    rows={4}
                    className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent)] focus:outline-none"
                    placeholder="Enter text to translate..."
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                    Target Language
                  </label>
                  <select
                    value={targetLang}
                    onChange={(e) => setTargetLang(e.target.value)}
                    className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] focus:border-[var(--color-accent)] focus:outline-none"
                  >
                    {languages.map((lang) => (
                      <option key={lang.code} value={lang.code}>
                        {lang.name} ({lang.native})
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <button
                onClick={handleTranslate}
                disabled={translating || !translateText.trim()}
                className="inline-flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90 disabled:opacity-50"
              >
                {translating ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent" />
                    Translating...
                  </>
                ) : (
                  <>
                    <ArrowRightLeft className="h-4 w-4" />
                    Translate
                  </>
                )}
              </button>
              {translateResult && (
                <div className="rounded-lg bg-blue-50 border border-blue-200 p-4 dark:bg-blue-900/20 dark:border-blue-800">
                  <div className="flex items-center gap-2 mb-3">
                    <CheckCircle className="h-5 w-5 text-blue-600" />
                    <h3 className="font-medium text-blue-800 dark:text-blue-400">Translation Result</h3>
                  </div>
                  <div className="space-y-3">
                    <div>
                      <span className="text-xs font-medium text-[var(--color-text-muted)]">Original:</span>
                      <p className="text-sm text-[var(--color-text)] mt-1">{translateResult.original_text}</p>
                    </div>
                    <div className="flex items-center gap-2">
                      <ArrowRightLeft className="h-4 w-4 text-blue-600" />
                    </div>
                    <div>
                      <span className="text-xs font-medium text-[var(--color-text-muted)]">
                        Translated ({translateResult.target_language}):
                      </span>
                      <p className="text-sm text-[var(--color-text)] mt-1">{translateResult.translated_text}</p>
                    </div>
                    <div className="flex items-center gap-4 text-xs text-[var(--color-text-muted)]">
                      <span>Confidence: {Math.round((translateResult.confidence || 0) * 100)}%</span>
                      <span>From: {translateResult.source_language}</span>
                      <span>To: {translateResult.target_language}</span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && (
        <div className="space-y-6">
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-6">
            <h2 className="text-lg font-semibold text-[var(--color-text)] mb-4">Translation Settings</h2>
            {settings && (
              <div className="space-y-4">
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <div>
                    <h3 className="font-medium text-[var(--color-text)]">Enable Translation</h3>
                    <p className="text-sm text-[var(--color-text-muted)]">Enable or disable translation across the platform</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, enabled: !settings.enabled })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.enabled ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-600'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.enabled ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <div>
                    <h3 className="font-medium text-[var(--color-text)]">Auto-Translate</h3>
                    <p className="text-sm text-[var(--color-text-muted)]">Automatically translate inbound messages to default language</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, autoTranslate: !settings.autoTranslate })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.autoTranslate ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-600'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.autoTranslate ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <div>
                    <h3 className="font-medium text-[var(--color-text)]">Language Detection</h3>
                    <p className="text-sm text-[var(--color-text-muted)]">Automatically detect the language of inbound messages</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, detectLanguage: !settings.detectLanguage })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.detectLanguage ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-600'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.detectLanguage ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="flex items-center justify-between rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <div>
                    <h3 className="font-medium text-[var(--color-text)]">Show Original</h3>
                    <p className="text-sm text-[var(--color-text-muted)]">Display original text alongside translation</p>
                  </div>
                  <button
                    onClick={() => setSettings({ ...settings, showOriginal: !settings.showOriginal })}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.showOriginal ? 'bg-[var(--color-accent)]' : 'bg-gray-300 dark:bg-gray-600'}`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.showOriginal ? 'translate-x-6' : 'translate-x-1'}`} />
                  </button>
                </div>
                <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                        Default Target Language
                      </label>
                      <select
                        value={settings.defaultTargetLanguage}
                        onChange={(e) => setSettings({ ...settings, defaultTargetLanguage: e.target.value })}
                        className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] focus:border-[var(--color-accent)] focus:outline-none"
                      >
                        {languages.map((lang) => (
                          <option key={lang.code} value={lang.code}>
                            {lang.name} ({lang.native})
                          </option>
                        ))}
                      </select>
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                        Fallback Language
                      </label>
                      <select
                        value={settings.fallbackLanguage}
                        onChange={(e) => setSettings({ ...settings, fallbackLanguage: e.target.value })}
                        className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] focus:border-[var(--color-accent)] focus:outline-none"
                      >
                        {languages.map((lang) => (
                          <option key={lang.code} value={lang.code}>
                            {lang.name} ({lang.native})
                          </option>
                        ))}
                      </select>
                    </div>
                  </div>
                </div>
                <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                    Translation API URL
                  </label>
                  <input
                    type="text"
                    value={settings.translationApiUrl}
                    onChange={(e) => setSettings({ ...settings, translationApiUrl: e.target.value })}
                    className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent)] focus:outline-none"
                    placeholder="https://api.example.com/translate"
                  />
                  <p className="text-xs text-[var(--color-text-muted)] mt-1">
                    Leave empty to use built-in phrase-based translation
                  </p>
                </div>
                <div className="rounded-lg bg-[var(--color-bg-hover)] p-4">
                  <label className="block text-sm font-medium text-[var(--color-text)] mb-2">
                    Translation API Key
                  </label>
                  <input
                    type="password"
                    value={settings.translationApiKey}
                    onChange={(e) => setSettings({ ...settings, translationApiKey: e.target.value })}
                    className="w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-accent)] focus:outline-none"
                    placeholder="Enter API key..."
                  />
                </div>
                <button
                  onClick={handleSaveSettings}
                  disabled={saving}
                  className="inline-flex items-center gap-2 rounded-lg bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-accent)]/90 disabled:opacity-50"
                >
                  {saving ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent" />
                      Saving...
                    </>
                  ) : (
                    <>
                      <Save className="h-4 w-4" />
                      Save Settings
                    </>
                  )}
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

function Save({ className }: { className?: string }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  )
}
