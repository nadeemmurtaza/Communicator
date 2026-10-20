import { useEffect, useState } from 'react'
import { api, type Sequence, type VariantStats } from '../lib/api'

export function Sequences() {
  const [sequences, setSequences] = useState<Sequence[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [openId, setOpenId] = useState<string | null>(null)
  const [stats, setStats] = useState<VariantStats | null>(null)

  useEffect(() => {
    api
      .listSequences()
      .then(setSequences)
      .catch((e) => setError(e.message))
  }, [])

  async function toggle(id: string) {
    if (openId === id) {
      setOpenId(null)
      return
    }
    setOpenId(id)
    setStats(null)
    try {
      setStats(await api.sequenceVariantStats(id))
    } catch (e) {
      setError((e as Error).message)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-slate-900">Follow-up sequences</h1>
        <p className="mt-1 text-sm text-slate-500">A/B variant performance per sequence.</p>
      </div>

      {error && (
        <div className="rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </div>
      )}

      {sequences === null ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : sequences.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-white px-4 py-10 text-center text-sm text-slate-500">
          No sequences configured.
        </div>
      ) : (
        <ul className="space-y-2">
          {sequences.map((s) => (
            <li key={s.id} className="rounded-lg border border-slate-200 bg-white">
              <button
                onClick={() => toggle(s.id)}
                className="flex w-full items-center justify-between px-4 py-3 text-left"
              >
                <span className="text-sm font-medium text-slate-900">{s.name}</span>
                <span className={`badge ${s.isActive ? 'badge-success' : 'badge-ghost'}`}>
                  {s.isActive ? 'Active' : 'Inactive'}
                </span>
              </button>
              {openId === s.id && (
                <div className="border-t border-slate-100 px-4 py-3">
                  {stats === null ? (
                    <p className="text-sm text-slate-500">Loading stats…</p>
                  ) : stats.variants.length === 0 ? (
                    <p className="text-sm text-slate-500">No enrollments yet.</p>
                  ) : (
                    <table className="w-full text-sm">
                      <thead className="text-left text-xs uppercase tracking-wide text-slate-400">
                        <tr>
                          <th className="py-1">Variant</th>
                          <th className="py-1">Enrolled</th>
                          <th className="py-1">Replied</th>
                          <th className="py-1">Closed won</th>
                        </tr>
                      </thead>
                      <tbody>
                        {stats.variants.map((v) => (
                          <tr key={v.variant} className="border-t border-slate-100">
                            <td className="py-1.5 font-medium text-slate-800">{v.variant}</td>
                            <td className="py-1.5">{v.enrolled}</td>
                            <td className="py-1.5">{v.replied}</td>
                            <td className="py-1.5">{v.closedWon}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
