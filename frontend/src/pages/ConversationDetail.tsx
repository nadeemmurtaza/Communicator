import { useEffect, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { api, type ConversationDetail as ConversationDetailType } from '../lib/api'
import { stateBadgeClass, stateLabel, timeAgo } from '../lib/format'

export function ConversationDetail() {
  const { id } = useParams<{ id: string }>()
  const [conversation, setConversation] = useState<ConversationDetailType | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [generating, setGenerating] = useState(false)

  function load() {
    if (!id) return
    api
      .getConversation(id)
      .then(setConversation)
      .catch((e) => setError(e.message))
  }

  useEffect(load, [id])

  async function generateDraft() {
    if (!id) return
    setGenerating(true)
    setError(null)
    try {
      await api.generateDraft(id)
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setGenerating(false)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <Link to="/conversations" className="text-sm text-slate-500 hover:underline">
          ← Conversations
        </Link>
        <div className="mt-2 flex items-center gap-3">
          <h1 className="text-xl font-semibold text-slate-900">
            Conversation {id?.slice(0, 8)}
          </h1>
          {conversation && (
            <span className={`badge ${stateBadgeClass(conversation.state)}`}>
              {stateLabel(conversation.state)}
            </span>
          )}
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {conversation === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : (
        <>
          <ol className="space-y-3">
            {conversation.messages.map((m) => (
              <li
                key={m.id}
                className={`max-w-xl rounded-lg border px-4 py-3 text-sm ${
                  m.direction === 'inbound'
                    ? 'border-slate-200 bg-white'
                    : 'ml-auto border-slate-900 bg-slate-900 text-white'
                }`}
              >
                <p className="whitespace-pre-wrap">{m.body}</p>
                <p
                  className={`mt-1 text-xs ${
                    m.direction === 'inbound' ? 'text-slate-400' : 'text-slate-300'
                  }`}
                >
                  {m.channel} · {timeAgo(m.sentAt)}
                </p>
              </li>
            ))}
          </ol>

          <div className="flex justify-end">
            <button
              onClick={generateDraft}
              disabled={generating}
              className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-800 disabled:opacity-50"
            >
              {generating ? 'Generating…' : 'Generate a reply draft'}
            </button>
          </div>
        </>
      )}
    </div>
  )
}
