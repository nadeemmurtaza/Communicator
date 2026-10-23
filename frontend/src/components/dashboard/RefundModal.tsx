
interface RefundModalProps {
  activeConv: any
  handleExecuteRefund: any
  refundAmountInput: any
  refundReason: any
  refundSubmitting: any
  selectedInvoiceForRefund: any
  setRefundAmountInput: any
  setRefundModalOpen: any
  setRefundReason: any
  setSelectedInvoiceForRefund: any
  stripeInvoices: any
}

export function RefundModal(props: RefundModalProps) {
  const {
    activeConv,
    handleExecuteRefund,
    refundAmountInput,
    refundReason,
    refundSubmitting,
    selectedInvoiceForRefund,
    setRefundAmountInput,
    setRefundModalOpen,
    setRefundReason,
    setSelectedInvoiceForRefund,
    stripeInvoices,
  } = props

  return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-xs p-4">
          <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl border border-slate-200 space-y-4">
            <div className="flex items-center justify-between border-b border-slate-100 pb-3">
              <div className="flex items-center gap-2">
                <span className="text-xl">⚡</span>
                <div>
                  <h3 className="font-bold text-sm text-slate-900">Issue Stripe Customer Refund</h3>
                  <p className="text-[11px] text-slate-500">
                    Target: {activeConv?.name || 'Customer'} ({activeConv?.email || 'email'})
                  </p>
                </div>
              </div>
              <button
                onClick={() => setRefundModalOpen(false)}
                className="text-slate-400 hover:text-slate-600 text-lg font-bold cursor-pointer"
              >
                ✕
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">
                  Select Target Invoice / Charge
                </label>
                <select
                  value={selectedInvoiceForRefund?.id || ''}
                  onChange={(e) => {
                    const inv = stripeInvoices.find((i: any) => i.id === e.target.value)
                    setSelectedInvoiceForRefund(inv || null)
                  }}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-hidden"
                >
                  {stripeInvoices.map((inv: any) => (
                    <option key={inv.id} value={inv.id}>
                      {inv.number} · {inv.amount_paid_formatted} (Paid on {new Date(inv.created_at).toLocaleDateString()})
                    </option>
                  ))}
                  {stripeInvoices.length === 0 && (
                    <option value="sim_default">Default Recent Charge ($299.00)</option>
                  )}
                </select>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="text-[11px] font-bold text-slate-700">Refund Amount ($)</label>
                  <button
                    type="button"
                    onClick={() => {
                      if (selectedInvoiceForRefund) {
                        setRefundAmountInput((selectedInvoiceForRefund.amount_paid_cents / 100.0).toFixed(2))
                      }
                    }}
                    className="text-[10px] font-semibold text-indigo-600 hover:underline cursor-pointer"
                  >
                    Full Amount ({selectedInvoiceForRefund?.amount_paid_formatted || '$299.00'})
                  </button>
                </div>
                <input
                  type="number"
                  step="0.01"
                  placeholder={
                    selectedInvoiceForRefund
                      ? (selectedInvoiceForRefund.amount_paid_cents / 100.0).toFixed(2)
                      : '299.00'
                  }
                  value={refundAmountInput}
                  onChange={(e) => setRefundAmountInput(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-hidden"
                />
                <p className="text-[10px] text-slate-400 mt-1">
                  Leave blank to refund the full paid amount.
                </p>
              </div>

              <div>
                <label className="block text-[11px] font-bold text-slate-700 mb-1">
                  Refund Reason
                </label>
                <select
                  value={refundReason}
                  onChange={(e) => setRefundReason(e.target.value)}
                  className="w-full rounded-lg border border-slate-200 p-2 text-xs text-slate-800 focus:border-indigo-500 focus:outline-hidden"
                >
                  <option value="requested_by_customer">Customer Request / Satisfaction Guarantee</option>
                  <option value="duplicate">Accidental Duplicate Charge</option>
                  <option value="fraudulent">Suspected Fraudulent Activity</option>
                  <option value="goodwill_credit">Service Disruption / Goodwill Credit</option>
                </select>
              </div>

              <div className="rounded-lg bg-amber-50 p-2.5 border border-amber-200 text-[11px] text-amber-800 leading-snug">
                ⚠️ Refunds are recorded immediately in Stripe and an immutable audit log is generated.
              </div>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100">
              <button
                type="button"
                onClick={() => setRefundModalOpen(false)}
                className="px-3 py-1.5 text-xs text-slate-600 hover:text-slate-800 font-semibold cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleExecuteRefund}
                disabled={refundSubmitting}
                className="px-4 py-1.5 text-xs bg-rose-600 hover:bg-rose-700 text-white rounded-lg font-bold shadow-xs transition-colors disabled:opacity-50 flex items-center gap-1.5 cursor-pointer"
              >
                {refundSubmitting ? 'Processing...' : 'Confirm & Execute Refund'}
              </button>
            </div>
          </div>
        </div>
      
  )
}
