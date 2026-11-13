import { useState, useEffect } from 'react'
import {
  getVisionSummary, uploadVisionImage, analyzeVisionImage,
  listVisionImages, getVisionImage, deleteVisionImage,
  getVisionSettings, updateVisionSettings
} from '../lib/api'
import type { VisionSummary as VisionSummaryType, VisionImage, VisionSettings, VisionAnalysis } from '../lib/api'
import { Eye, Upload, Image as ImageIcon, Settings, Trash2, Play, FileText, Search, AlertCircle, CheckCircle, XCircle } from '../components/icons'

export default function FinVisionDashboard() {
  const [activeTab, setActiveTab] = useState<'overview' | 'images' | 'analysis' | 'settings'>('overview')
  const [summary, setSummary] = useState<VisionSummaryType | null>(null)
  const [images, setImages] = useState<VisionImage[]>([])
  const [settings, setSettings] = useState<VisionSettings | null>(null)
  const [selectedImage, setSelectedImage] = useState<VisionImage | null>(null)
  const [selectedAnalyses, setSelectedAnalyses] = useState<VisionAnalysis[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [statusFilter, setStatusFilter] = useState<string>('')

  // Upload state
  const [uploading, setUploading] = useState(false)
  const [analyzing, setAnalyzing] = useState<string | null>(null)
  const [analysisType, setAnalysisType] = useState('general')

  // Settings state
  const [settingsForm, setSettingsForm] = useState({
    enabled: true,
    autoAnalyze: true,
    defaultAnalysisType: 'general',
    claudeModel: 'claude-sonnet-4-20250514',
    maxFileSizeMb: 10,
    ocrEnabled: true,
    objectDetectionEnabled: true,
  })

  useEffect(() => {
    loadSummary()
    loadImages()
    loadSettings()
  }, [statusFilter])

  const loadSummary = async () => {
    try {
      const data = await getVisionSummary()
      setSummary(data)
    } catch (e: any) {
      setError(e.message || 'Failed to load summary')
    }
  }

  const loadImages = async () => {
    try {
      const data = await listVisionImages(statusFilter || undefined, 50)
      setImages(data)
    } catch (e: any) {
      setError(e.message || 'Failed to load images')
    }
  }

  const loadSettings = async () => {
    try {
      const data = await getVisionSettings()
      setSettings(data)
      setSettingsForm({
        enabled: data.enabled,
        autoAnalyze: data.autoAnalyze,
        defaultAnalysisType: data.defaultAnalysisType,
        claudeModel: data.claudeModel,
        maxFileSizeMb: data.maxFileSizeMb,
        ocrEnabled: data.ocrEnabled,
        objectDetectionEnabled: data.objectDetectionEnabled,
      })
    } catch (e: any) {
      setError(e.message || 'Failed to load settings')
    }
  }

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    setUploading(true)
    setError(null)
    try {
      const base64 = await fileToBase64(file)
      const result = await uploadVisionImage({
        imageData: base64,
        mimeType: file.type,
        fileSize: file.size,
      })
      if (result.error) {
        setError(result.error as string)
      } else {
        await loadImages()
        await loadSummary()
      }
    } catch (err: any) {
      setError(err.message || 'Upload failed')
    } finally {
      setUploading(false)
      e.target.value = ''
    }
  }

  const fileToBase64 = (file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const result = reader.result as string
        // Remove the data:image/...;base64, prefix
        const base64 = result.split(',')[1]
        resolve(base64)
      }
      reader.onerror = reject
      reader.readAsDataURL(file)
    })
  }

  const handleAnalyze = async (imageId: string) => {
    setAnalyzing(imageId)
    setError(null)
    try {
      await analyzeVisionImage(imageId, { analysisType })
      await loadImages()
      await loadSummary()
    } catch (err: any) {
      setError(err.message || 'Analysis failed')
    } finally {
      setAnalyzing(null)
    }
  }

  const handleViewImage = async (image: VisionImage) => {
    setLoading(true)
    try {
      const data = await getVisionImage(image.id)
      setSelectedImage(data.image)
      setSelectedAnalyses(data.analyses)
      setActiveTab('analysis')
    } catch (err: any) {
      setError(err.message || 'Failed to load image details')
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (imageId: string) => {
    if (!confirm('Delete this image and its analyses?')) return
    try {
      await deleteVisionImage(imageId)
      await loadImages()
      await loadSummary()
    } catch (err: any) {
      setError(err.message || 'Delete failed')
    }
  }

  const handleSaveSettings = async () => {
    setLoading(true)
    try {
      const data = await updateVisionSettings(settingsForm)
      setSettings(data)
      await loadSummary()
    } catch (err: any) {
      setError(err.message || 'Failed to save settings')
    } finally {
      setLoading(false)
    }
  }

  const statusBadge = (status: string) => {
    const colors: Record<string, string> = {
      uploaded: 'bg-gray-100 text-gray-700',
      analyzing: 'bg-blue-100 text-blue-700',
      completed: 'bg-green-100 text-green-700',
      completed_with_errors: 'bg-yellow-100 text-yellow-700',
      failed: 'bg-red-100 text-red-700',
    }
    const color = colors[status] || 'bg-gray-100 text-gray-700'
    return <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${color}`}>{status}</span>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Fin Vision</h1>
          <p className="text-gray-500 mt-1">AI-powered image understanding for customer support</p>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-500 mt-0.5 flex-shrink-0" />
          <div>
            <p className="text-red-800 text-sm">{error}</p>
            <button onClick={() => setError(null)} className="text-red-600 text-xs mt-1 underline">Dismiss</button>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'overview' as const, label: 'Overview', icon: Eye },
            { id: 'images' as const, label: 'Images', icon: ImageIcon },
            { id: 'analysis' as const, label: 'Analysis Detail', icon: Search },
            { id: 'settings' as const, label: 'Settings', icon: Settings },
          ].map((tab) => {
            const Icon = tab.icon
            const active = activeTab === tab.id
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 py-3 px-1 border-b-2 text-sm font-medium transition-colors ${
                  active
                    ? 'border-purple-500 text-purple-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            )
          })}
        </nav>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Total Images</p>
                  <p className="text-3xl font-bold text-gray-900 mt-1">{summary?.totalImages ?? 0}</p>
                </div>
                <ImageIcon className="w-10 h-10 text-purple-400" />
              </div>
            </div>
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Total Analyses</p>
                  <p className="text-3xl font-bold text-gray-900 mt-1">{summary?.totalAnalyses ?? 0}</p>
                </div>
                <Eye className="w-10 h-10 text-blue-400" />
              </div>
            </div>
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Avg Confidence</p>
                  <p className="text-3xl font-bold text-gray-900 mt-1">
                    {summary?.avgConfidence ? Math.round(summary.avgConfidence * 100) : 0}%
                  </p>
                </div>
                <CheckCircle className="w-10 h-10 text-green-400" />
              </div>
            </div>
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Avg Processing</p>
                  <p className="text-3xl font-bold text-gray-900 mt-1">
                    {summary?.avgProcessingTimeMs ? `${Math.round(summary.avgProcessingTimeMs / 1000)}s` : '0s'}
                  </p>
                </div>
                <Play className="w-10 h-10 text-orange-400" />
              </div>
            </div>
          </div>

          {/* Analysis Type Breakdown */}
          {summary?.analysisTypeBreakdown.length && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Analysis Type Breakdown</h3>
              <div className="space-y-3">
                {summary.analysisTypeBreakdown.map((item) => (
                  <div key={item.analysisType} className="flex items-center gap-4">
                    <span className="text-sm text-gray-600 w-28 capitalize">{item.analysisType}</span>
                    <div className="flex-1 bg-gray-100 rounded-full h-3">
                      <div
                        className="bg-purple-500 h-3 rounded-full"
                        style={{
                          width: `${summary.totalAnalyses ? (item.count / summary.totalAnalyses) * 100 : 0}%`,
                        }}
                      />
                    </div>
                    <span className="text-sm font-medium text-gray-900 w-8 text-right">{item.count}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Recent Analyses */}
          {summary?.recentAnalyses.length && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Analyses</h3>
              <div className="space-y-4">
                {summary.recentAnalyses.slice(0, 5).map((analysis) => (
                  <div key={analysis.id} className="border border-gray-100 rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-gray-900 capitalize">{analysis.analysisType}</span>
                      <div className="flex items-center gap-2">
                        <span className="text-xs text-gray-500">{analysis.processingTimeMs}ms</span>
                        {analysis.errorMessage ? (
                          <XCircle className="w-4 h-4 text-red-400" />
                        ) : (
                          <CheckCircle className="w-4 h-4 text-green-400" />
                        )}
                      </div>
                    </div>
                    {analysis.summary && (
                      <p className="text-sm text-gray-600 line-clamp-2">{analysis.summary}</p>
                    )}
                    {analysis.textExtracted && (
                      <p className="text-xs text-gray-400 mt-1">
                        Text: {analysis.textExtracted.substring(0, 100)}...
                      </p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Status */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-3">Vision Status</h3>
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2">
                {summary?.enabled ? (
                  <CheckCircle className="w-5 h-5 text-green-500" />
                ) : (
                  <XCircle className="w-5 h-5 text-red-500" />
                )}
                <span className="text-sm text-gray-700">
                  {summary?.enabled ? 'Vision is enabled' : 'Vision is disabled'}
                </span>
              </div>
              <div className="flex items-center gap-2">
                {summary?.autoAnalyze ? (
                  <CheckCircle className="w-5 h-5 text-green-500" />
                ) : (
                  <XCircle className="w-5 h-5 text-gray-400" />
                )}
                <span className="text-sm text-gray-700">
                  {summary?.autoAnalyze ? 'Auto-analyze on' : 'Auto-analyze off'}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Images Tab */}
      {activeTab === 'images' && (
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <label className="flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 cursor-pointer transition-colors">
                <Upload className="w-4 h-4" />
                <span>Upload Image</span>
                <input
                  type="file"
                  accept="image/jpeg,image/png,image/webp,image/gif"
                  onChange={handleFileUpload}
                  className="hidden"
                  disabled={uploading}
                />
              </label>
              {uploading && <span className="text-sm text-gray-500">Uploading...</span>}
            </div>
            <div className="flex items-center gap-2">
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="border border-gray-300 rounded-lg px-3 py-2 text-sm"
              >
                <option value="">All statuses</option>
                <option value="uploaded">Uploaded</option>
                <option value="analyzing">Analyzing</option>
                <option value="completed">Completed</option>
                <option value="failed">Failed</option>
              </select>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
            <table className="w-full">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Image</th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Type</th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Size</th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Status</th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Channel</th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Uploaded</th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {images.map((img) => (
                  <tr key={img.id} className="hover:bg-gray-50">
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 bg-gray-100 rounded-lg flex items-center justify-center">
                          <ImageIcon className="w-5 h-5 text-gray-400" />
                        </div>
                        <span className="text-sm text-gray-900 font-mono">{img.id.substring(0, 12)}...</span>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-600">{img.mimeType}</td>
                    <td className="px-4 py-3 text-sm text-gray-600">
                      {img.fileSize > 1024 * 1024
                        ? `${(img.fileSize / (1024 * 1024)).toFixed(1)}MB`
                        : `${Math.round(img.fileSize / 1024)}KB`}
                    </td>
                    <td className="px-4 py-3">{statusBadge(img.status)}</td>
                    <td className="px-4 py-3 text-sm text-gray-600">{img.channel || '-'}</td>
                    <td className="px-4 py-3 text-sm text-gray-500">
                      {new Date(img.createdAt).toLocaleDateString()}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => handleViewImage(img)}
                          className="text-purple-600 hover:text-purple-800 text-sm"
                        >
                          View
                        </button>
                        {img.status === 'uploaded' && (
                          <button
                            onClick={() => handleAnalyze(img.id)}
                            disabled={analyzing === img.id}
                            className="text-blue-600 hover:text-blue-800 text-sm disabled:opacity-50"
                          >
                            {analyzing === img.id ? 'Analyzing...' : 'Analyze'}
                          </button>
                        )}
                        <button
                          onClick={() => handleDelete(img.id)}
                          className="text-red-600 hover:text-red-800 text-sm"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
                {images.length === 0 && (
                  <tr>
                    <td colSpan={7} className="px-4 py-12 text-center text-gray-500">
                      No images uploaded yet. Upload an image to get started.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Analysis Detail Tab */}
      {activeTab === 'analysis' && (
        <div className="space-y-6">
          {!selectedImage ? (
            <div className="bg-white rounded-xl border border-gray-200 p-12 text-center">
              <Eye className="w-12 h-12 text-gray-300 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900">No image selected</h3>
              <p className="text-gray-500 mt-1">Select an image from the Images tab to view its analysis details.</p>
              <button
                onClick={() => setActiveTab('images')}
                className="mt-4 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
              >
                Go to Images
              </button>
            </div>
          ) : (
            <>
              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-semibold text-gray-900">Image Details</h3>
                  <button
                    onClick={() => setActiveTab('images')}
                    className="text-sm text-purple-600 hover:text-purple-800"
                  >
                    Back to Images
                  </button>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div>
                    <p className="text-xs text-gray-500">ID</p>
                    <p className="text-sm font-mono text-gray-900">{selectedImage.id.substring(0, 20)}...</p>
                  </div>
                  <div>
                    <p className="text-xs text-gray-500">MIME Type</p>
                    <p className="text-sm text-gray-900">{selectedImage.mimeType}</p>
                  </div>
                  <div>
                    <p className="text-xs text-gray-500">Size</p>
                    <p className="text-sm text-gray-900">
                      {selectedImage.fileSize > 1024 * 1024
                        ? `${(selectedImage.fileSize / (1024 * 1024)).toFixed(1)}MB`
                        : `${Math.round(selectedImage.fileSize / 1024)}KB`}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs text-gray-500">Status</p>
                    <div>{statusBadge(selectedImage.status)}</div>
                  </div>
                </div>
              </div>

              {/* Analysis type selector for re-analysis */}
              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Re-analyze with different type</h3>
                <div className="flex items-center gap-4">
                  <select
                    value={analysisType}
                    onChange={(e) => setAnalysisType(e.target.value)}
                    className="border border-gray-300 rounded-lg px-3 py-2 text-sm"
                  >
                    <option value="general">General</option>
                    <option value="screenshot">Screenshot</option>
                    <option value="document">Document</option>
                    <option value="error_code">Error Code</option>
                    <option value="product">Product</option>
                  </select>
                  <button
                    onClick={() => handleAnalyze(selectedImage.id)}
                    disabled={analyzing === selectedImage.id}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
                  >
                    {analyzing === selectedImage.id ? 'Analyzing...' : 'Analyze'}
                  </button>
                </div>
              </div>

              {/* Analyses */}
              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">
                  Analyses ({selectedAnalyses.length})
                </h3>
                {selectedAnalyses.length === 0 ? (
                  <p className="text-gray-500">No analyses yet for this image.</p>
                ) : (
                  <div className="space-y-4">
                    {selectedAnalyses.map((analysis) => (
                      <div key={analysis.id} className="border border-gray-100 rounded-lg p-4">
                        <div className="flex items-center justify-between mb-3">
                          <div className="flex items-center gap-3">
                            <span className="text-sm font-medium text-gray-900 capitalize">{analysis.analysisType}</span>
                            {statusBadge(analysis.errorMessage ? 'failed' : 'completed')}
                          </div>
                          <div className="flex items-center gap-3 text-xs text-gray-500">
                            <span>{analysis.processingTimeMs}ms</span>
                            <span>Confidence: {Math.round(analysis.confidence * 100)}%</span>
                            <span>{analysis.claudeModel}</span>
                          </div>
                        </div>

                        {analysis.summary && (
                          <div className="mb-3">
                            <p className="text-xs text-gray-500 mb-1">Summary</p>
                            <p className="text-sm text-gray-700 bg-gray-50 rounded-lg p-3">{analysis.summary}</p>
                          </div>
                        )}

                        {analysis.textExtracted && (
                          <div className="mb-3">
                            <p className="text-xs text-gray-500 mb-1">Extracted Text</p>
                            <pre className="text-sm text-gray-700 bg-gray-50 rounded-lg p-3 whitespace-pre-wrap font-sans">
                              {analysis.textExtracted}
                            </pre>
                          </div>
                        )}

                        {analysis.objectsDetected && analysis.objectsDetected !== '[]' && (
                          <div className="mb-3">
                            <p className="text-xs text-gray-500 mb-1">Objects Detected</p>
                            <div className="flex flex-wrap gap-2">
                              {(() => {
                                try {
                                  const objects = JSON.parse(analysis.objectsDetected)
                                  return objects.map((obj: string, i: number) => (
                                    <span key={i} className="px-2 py-1 bg-purple-100 text-purple-700 rounded-full text-xs">
                                      {obj}
                                    </span>
                                  ))
                                } catch {
                                  return null
                                }
                              })()}
                            </div>
                          </div>
                        )}

                        {analysis.errorMessage && (
                          <div className="flex items-start gap-2 bg-red-50 border border-red-200 rounded-lg p-3">
                            <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 flex-shrink-0" />
                            <p className="text-sm text-red-700">{analysis.errorMessage}</p>
                          </div>
                        )}

                        <p className="text-xs text-gray-400 mt-2">
                          {new Date(analysis.createdAt).toLocaleString()}
                        </p>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </>
          )}
        </div>
      )}

      {/* Settings Tab */}
      {activeTab === 'settings' && (
        <div className="space-y-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-6">Vision Configuration</h3>

            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-900">Enable Fin Vision</p>
                  <p className="text-xs text-gray-500">Turn image analysis on or off</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settingsForm.enabled}
                    onChange={(e) => setSettingsForm({ ...settingsForm, enabled: e.target.checked })}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-purple-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-600" />
                </label>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-900">Auto-analyze on upload</p>
                  <p className="text-xs text-gray-500">Automatically analyze images when uploaded</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settingsForm.autoAnalyze}
                    onChange={(e) => setSettingsForm({ ...settingsForm, autoAnalyze: e.target.checked })}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-purple-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-600" />
                </label>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-900">OCR (Text Extraction)</p>
                  <p className="text-xs text-gray-500">Extract text from images</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settingsForm.ocrEnabled}
                    onChange={(e) => setSettingsForm({ ...settingsForm, ocrEnabled: e.target.checked })}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-purple-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-600" />
                </label>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-900">Object Detection</p>
                  <p className="text-xs text-gray-500">Detect objects and elements in images</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settingsForm.objectDetectionEnabled}
                    onChange={(e) => setSettingsForm({ ...settingsForm, objectDetectionEnabled: e.target.checked })}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-purple-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-600" />
                </label>
              </div>

              <div>
                <p className="text-sm font-medium text-gray-900 mb-1">Default Analysis Type</p>
                <select
                  value={settingsForm.defaultAnalysisType}
                  onChange={(e) => setSettingsForm({ ...settingsForm, defaultAnalysisType: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm"
                >
                  <option value="general">General</option>
                  <option value="screenshot">Screenshot</option>
                  <option value="document">Document</option>
                  <option value="error_code">Error Code</option>
                  <option value="product">Product</option>
                </select>
              </div>

              <div>
                <p className="text-sm font-medium text-gray-900 mb-1">Claude Model</p>
                <input
                  type="text"
                  value={settingsForm.claudeModel}
                  onChange={(e) => setSettingsForm({ ...settingsForm, claudeModel: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm"
                  placeholder="claude-sonnet-4-20250514"
                />
              </div>

              <div>
                <p className="text-sm font-medium text-gray-900 mb-1">Max File Size (MB)</p>
                <input
                  type="number"
                  value={settingsForm.maxFileSizeMb}
                  onChange={(e) => setSettingsForm({ ...settingsForm, maxFileSizeMb: parseFloat(e.target.value) || 10 })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm"
                  min={1}
                  max={100}
                  step={1}
                />
              </div>

              <div className="pt-4 border-t border-gray-200">
                <button
                  onClick={handleSaveSettings}
                  disabled={loading}
                  className="px-6 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors disabled:opacity-50"
                >
                  {loading ? 'Saving...' : 'Save Settings'}
                </button>
              </div>
            </div>
          </div>

          {/* How it works */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">How Fin Vision Works</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="text-center">
                <div className="w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <Upload className="w-6 h-6 text-purple-600" />
                </div>
                <h4 className="font-medium text-gray-900 mb-1">1. Upload</h4>
                <p className="text-sm text-gray-500">Customers send images via any channel — screenshots, documents, product photos.</p>
              </div>
              <div className="text-center">
                <div className="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <Eye className="w-6 h-6 text-blue-600" />
                </div>
                <h4 className="font-medium text-gray-900 mb-1">2. Analyze</h4>
                <p className="text-sm text-gray-500">Claude's vision AI analyzes the image, extracts text, detects objects, and identifies issues.</p>
              </div>
              <div className="text-center">
                <div className="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <FileText className="w-6 h-6 text-green-600" />
                </div>
                <h4 className="font-medium text-gray-900 mb-1">3. Context</h4>
                <p className="text-sm text-gray-500">Analysis results are attached to the conversation, giving the AI visual context for responses.</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
