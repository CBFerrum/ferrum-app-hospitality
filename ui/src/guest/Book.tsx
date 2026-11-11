// The public booking page, /hotel/book/{workspace}: a guest picks a hotel, their dates and party,
// sees the room types free every night at the online rate with the deposit asked, and books one
// with their name and email. They are given their reservation's own link. Every rule is the
// server's; a refusal says why.

import { useMutation, useQuery } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";
import { Link, useParams } from "react-router";
import { refusal, t, isolate } from "@ferrum/client";
import { Alert, Button, Field, FormSkeleton } from "@ferrum/kit";
import { Closed, Frame } from "./Frame";
import { availability, book, type Booked, dayAfter, manageLink, type Offer, properties, type Search } from "./site";

export function BookPage() {
  const { tenant = "" } = useParams();
  const hotels = useQuery({ queryKey: ["hotel-book", tenant], queryFn: ({ signal }) => properties(tenant, signal) });
  if (hotels.isPending)
    return (
      <Frame>
        <FormSkeleton />
      </Frame>
    );
  if (hotels.isError)
    return (
      <Frame>
        <Closed error={hotels.error} what="page" onRetry={() => void hotels.refetch()} />
      </Frame>
    );
  return (
    <Frame>
      <Booking tenant={tenant} hotels={hotels.data.properties} />
    </Frame>
  );
}

function Booking({ tenant, hotels }: { tenant: string; hotels: { code: string; name: string }[] }) {
  const today = new Date();
  const [q, setQ] = useState<Search>({ property: hotels[0]?.code ?? "", arrival: dayAfter(today, 1), departure: dayAfter(today, 2), adults: 2, children: 0 });
  const [asked, setAsked] = useState<Search | null>(null);
  const [chosen, setChosen] = useState<Offer | null>(null);
  const [done, setDone] = useState<Booked | null>(null);
  const offers = useQuery({
    queryKey: ["hotel-book", tenant, "offers", asked],
    queryFn: ({ signal }) => availability(tenant, asked!, signal),
    enabled: asked !== null,
  });
  if (done) return <Done tenant={tenant} booked={done} />;
  const search = (e: FormEvent) => {
    e.preventDefault();
    setChosen(null);
    setAsked({ ...q });
  };
  return (
    <div className="stack">
      <h1>{t("Book a room")}</h1>
      <form className="card card-pad stack" onSubmit={search} aria-label={t("Dates and guests")}>
        {hotels.length > 1 && (
          <Field label={t("Hotel")}>
            <select className="select" value={q.property} onChange={(e) => setQ({ ...q, property: e.target.value })}>
              {hotels.map((h) => (
                <option key={h.code} value={h.code}>
                  {h.name}
                </option>
              ))}
            </select>
          </Field>
        )}
        <div className="hb-row">
          <Field label={t("Arrival")} required>
            <input className="input" type="date" required value={q.arrival} onChange={(e) => setQ({ ...q, arrival: e.target.value })} />
          </Field>
          <Field label={t("Departure")} required>
            <input className="input" type="date" required value={q.departure} onChange={(e) => setQ({ ...q, departure: e.target.value })} />
          </Field>
          <Field label={t("Adults")}>
            <input className="input" type="number" min={1} max={20} value={q.adults} onChange={(e) => setQ({ ...q, adults: Number(e.target.value) || 1 })} />
          </Field>
          <Field label={t("Children")}>
            <input className="input" type="number" min={0} max={20} value={q.children} onChange={(e) => setQ({ ...q, children: Number(e.target.value) || 0 })} />
          </Field>
        </div>
        <div>
          <Button type="submit" variant="primary" loading={offers.isFetching}>
            {t("See the rooms free")}
          </Button>
        </div>
      </form>
      {offers.isError && <Alert tone="error">{refusal(offers.error)}</Alert>}
      {offers.data && (
        <section className="stack" aria-label={t("Rooms free")} aria-live="polite">
          {offers.data.offers.length === 0 ? (
            <p className="muted">{t("No room is free for every night of these dates. Try other dates.")}</p>
          ) : (
            <>
              <p className="muted">
                {offers.data.refund_days === null
                  ? t("A cancellation gets its deposit back any time before arrival.")
                  : t("A cancellation up to {days} days before arrival gets its deposit back; later, or not arriving, the hotel keeps it.", { days: offers.data.refund_days })}
              </p>
              {offers.data.offers.map((o) => (
                <article key={o.room_type} className="card card-pad stack" aria-label={o.name}>
                  <div className="row" style={{ justifyContent: "space-between" }}>
                    <h2>{isolate(o.name)}</h2>
                    <strong>{isolate(o.total)}</strong>
                  </div>
                  <div className="faint">
                    {t("Sleeps {sleeps} · {nights} nights · deposit {deposit}", { sleeps: o.sleeps, nights: offers.data.nights, deposit: o.deposit })}
                  </div>
                  {o.amenities.length > 0 && <div className="faint">{isolate(o.amenities.join(", "))}</div>}
                  {chosen?.room_type === o.room_type ? (
                    <Details tenant={tenant} search={asked!} offer={o} onBooked={setDone} />
                  ) : (
                    <div>
                      <Button onClick={() => setChosen(o)}>{t("Book this room")}</Button>
                    </div>
                  )}
                </article>
              ))}
            </>
          )}
        </section>
      )}
    </div>
  );
}

function Details({ tenant, search, offer, onBooked }: { tenant: string; search: Search; offer: Offer; onBooked: (b: Booked) => void }) {
  const [v, setV] = useState({ name: "", email: "", phone: "", notes: "", website: "" });
  const go = useMutation({
    mutationFn: () => book(tenant, { ...search, room_type: offer.room_type, ...v }),
    onSuccess: onBooked,
  });
  return (
    <form
      className="stack"
      onSubmit={(e) => {
        e.preventDefault();
        go.mutate();
      }}
      aria-label={t("Your details")}
    >
      <Field label={t("Name")} required>
        <input className="input" autoComplete="name" required maxLength={200} value={v.name} onChange={(e) => setV({ ...v, name: e.target.value })} />
      </Field>
      <Field label={t("Email")} required hint={t("Where your reservation is sent")}>
        <input className="input" type="email" autoComplete="email" required maxLength={254} value={v.email} onChange={(e) => setV({ ...v, email: e.target.value })} />
      </Field>
      <Field label={t("Phone")}>
        <input className="input" type="tel" autoComplete="tel" maxLength={40} value={v.phone} onChange={(e) => setV({ ...v, phone: e.target.value })} />
      </Field>
      <Field label={t("Anything the hotel should know")}>
        <textarea className="input" maxLength={2000} rows={3} value={v.notes} onChange={(e) => setV({ ...v, notes: e.target.value })} />
      </Field>
      {/* Hidden from people: whatever fills it is a program. */}
      <input className="hb-trap" tabIndex={-1} autoComplete="off" aria-hidden="true" value={v.website} onChange={(e) => setV({ ...v, website: e.target.value })} name="website" />
      {go.isError && <Alert tone="error">{refusal(go.error)}</Alert>}
      <div>
        <Button type="submit" variant="primary" loading={go.isPending}>
          {t("Book it")}
        </Button>
      </div>
    </form>
  );
}

function Done({ tenant, booked }: { tenant: string; booked: Booked }) {
  if (!("reservation" in booked)) return <Alert tone="success">{t("Thank you: your request is in.")}</Alert>;
  const r = booked.reservation;
  return (
    <section className="card card-pad stack" aria-label={t("Booked")}>
      <h1>{t("Booked: {number}", { number: r.number })}</h1>
      <p>
        {t("{room_type} at {property}, {arrival} to {departure}.", { room_type: r.room_type ?? "", property: r.property, arrival: r.arrival, departure: r.departure })}
      </p>
      <p>{t("Total {total}; deposit {deposit}, taken by the hotel.", { total: r.total, deposit: r.deposit_due ?? "0" })}</p>
      <p className="muted">{t("Keep your reservation's link: it is how you see or cancel it.")}</p>
      <Link to={manageLink(tenant, booked.token)}>{t("Your reservation")}</Link>
    </section>
  );
}
