// A guest's own reservation, from the link they were given: what they booked, what its deposit is
// and became, and, while it is reserved, cancelling it (by the rate plan's policy, as at the desk).

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useParams } from "react-router";
import { refusal, t, isolate } from "@ferrum/client";
import { Alert, Button, confirmDialog, Field, PageHead, ReaderSkeleton, StatusBadge } from "@ferrum/kit";
import { Closed, Frame } from "./Frame";
import { cancel, mine, type Reservation } from "./site";

/** Stay.status as stored, for `statusTone`: only a reservation still to come is green. */
export const RESERVATION_TONES = { Reserved: "green", Cancelled: "red" } as const;

export function ManageRoute() {
  const { tenant = "", token = "" } = useParams();
  const qc = useQueryClient();
  const q = useQuery({ queryKey: ["hotel-mine", tenant, token], queryFn: ({ signal }) => mine(tenant, token, signal) });
  const [reason, setReason] = useState("");
  const [said, setSaid] = useState<string | null>(null);
  const undo = useMutation({
    mutationFn: () => cancel(tenant, token, reason),
    onSuccess: (r) => {
      setSaid(typeof r.message === "string" ? r.message : null);
      qc.setQueryData(["hotel-mine", tenant, token], { reservation: r.reservation });
    },
  });
  if (q.isPending)
    return (
      <Frame>
        <ReaderSkeleton />
      </Frame>
    );
  if (q.isError)
    return (
      <Frame>
        <Closed error={q.error} what="link" onRetry={() => void q.refetch()} />
      </Frame>
    );
  const r: Reservation = q.data.reservation;
  return (
    <Frame>
      <PageHead title={isolate(r.number)} actions={<span className="hb-head-badge"><StatusBadge value={r.status} tones={RESERVATION_TONES} /></span>} />
      <section className="card card-pad stack" aria-label={t("Your reservation")}>
        <p>
          {t("{room_type} at {property}, {arrival} to {departure}.", { room_type: r.room_type ?? "", property: r.property, arrival: r.arrival, departure: r.departure })}
        </p>
        {(r.check_in_time || r.check_out_time) && (
          <p className="faint">{t("Check-in from {in}; check-out by {out}.", { in: r.check_in_time ?? "", out: r.check_out_time ?? "" })}</p>
        )}
        <p>{t("Total {total}; deposit {deposit}, of which {paid} is paid.", { total: r.total, deposit: r.deposit_due ?? "0", paid: r.deposit_paid ?? "0" })}</p>
        {r.deposit_outcome === "Refund Due" && <Alert tone="info">{t("Your deposit is being given back.")}</Alert>}
        {r.deposit_outcome === "Kept" && <Alert tone="warning">{t("Your deposit is kept, as the hotel's terms say.")}</Alert>}
        {said && <Alert tone="success">{isolate(said)}</Alert>}
        {r.cancellable && (
          <div className="stack">
            <p className="muted">
              {r.refund_days === null
                ? t("Cancelled before arrival, its deposit is given back.")
                : t("Cancelled up to {days} days before arrival, its deposit is given back; later, the hotel keeps it.", { days: r.refund_days })}
            </p>
            <Field label={t("Why you are cancelling (optional)")}>
              <input className="input" maxLength={500} value={reason} onChange={(e) => setReason(e.target.value)} />
            </Field>
            {undo.isError && <Alert tone="error">{refusal(undo.error)}</Alert>}
            <div>
              <Button
                variant="danger"
                loading={undo.isPending}
                onClick={async () => {
                  const ok = await confirmDialog({ title: t("Cancel this reservation?"), message: t("Its nights are given up."), confirmLabel: t("Cancel it") });
                  if (ok) undo.mutate();
                }}
              >
                {t("Cancel the reservation")}
              </Button>
            </div>
          </div>
        )}
      </section>
    </Frame>
  );
}
