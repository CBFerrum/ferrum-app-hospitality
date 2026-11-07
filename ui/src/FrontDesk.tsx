// The front desk on the property's business date: who arrives, who is in the house, who leaves, and
// the rooms as housekeeping left them. Checking in, checking out and the night audit happen here;
// everything else opens in the records frame.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router";
import { api, docPath, refusal, t } from "@ferrum/client";
import { Badge, Button, confirmDialog, EmptyState, ErrorState, PageLoading, toast, useShellTitle } from "@ferrum/kit";

interface Property {
  id: string;
  property_name: string;
  business_date: string | null;
}

interface Stay {
  id: string;
  number: string;
  guest_name: string;
  arrival: string;
  departure: string;
  room?: string | null;
  room_type: string;
  status: string;
}

interface Room {
  id: string;
  room_number: string;
  housekeeping: string;
  occupied: boolean;
}

const rows = <T,>(entity: string, filters: unknown[], extra: Record<string, string | number> = {}) =>
  api.get<{ rows: T[] }>(docPath(entity), { filters: JSON.stringify(filters), limit: 500, ...extra }).then((r) => r.rows);

const act = (entity: string, id: string, action: string, params: unknown = {}) => api.post<Record<string, unknown>>(docPath(entity, id, "actions", action), params);

const TONE: Record<string, "green" | "amber" | "red" | "neutral"> = { Clean: "green", Inspected: "green", Dirty: "amber", "Out of Order": "red" };

function List({ title, stays, action, onAct }: { title: string; stays: Stay[]; action?: { label: string; name: string }; onAct: (s: Stay, name: string) => void }) {
  return (
    <section className="card card-pad stack" aria-label={title}>
      <h3>
        {title} <span className="faint">({stays.length})</span>
      </h3>
      {stays.length === 0 ? (
        <p className="muted">{t("Nobody.")}</p>
      ) : (
        <ul className="stack">
          {stays.map((s) => (
            <li key={s.id} className="row" style={{ justifyContent: "space-between", gap: 8 }}>
              <span>
                <Link to={`/e/Stay/${s.id}`}>{s.guest_name}</Link> <span className="faint">{s.number}</span>
                <div className="faint">
                  {s.arrival} to {s.departure}
                </div>
              </span>
              {action && (
                <Button size="sm" onClick={() => onAct(s, action.name)}>
                  {action.label}
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function Desk({ property }: { property: Property }) {
  const qc = useQueryClient();
  const bd = property.business_date;
  const stays = useQuery({
    queryKey: ["hotel", "stays", property.id, bd],
    queryFn: () => rows<Stay>("Stay", [["property", "eq", property.id], ["status", "in", ["Reserved", "Checked In"]]], { sort: "arrival asc" }),
    enabled: !!bd,
  });
  const rooms = useQuery({ queryKey: ["hotel", "rooms", property.id], queryFn: () => rows<Room>("Room", [["property", "eq", property.id]], { sort: "room_number asc" }) });
  const done = () => void qc.invalidateQueries({ queryKey: ["hotel"] });
  const move = useMutation({
    mutationFn: ({ stay, name }: { stay: Stay; name: string }) => act("Stay", stay.id, name),
    onSuccess: (_, v) => {
      toast(v.name === "check_in" ? `${v.stay.guest_name} is in` : `${v.stay.guest_name} has left`);
      done();
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  const audit = useMutation({
    mutationFn: () => act("HotelProperty", property.id, "night_audit"),
    onSuccess: (r) => {
      toast(`The night of ${String(r.night)} is closed: ${String(r.charges_posted)} rooms charged, ${String(r.no_shows)} no-shows`);
      done();
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  const [opening, setOpening] = useState(false);
  if (!bd)
    return (
      <EmptyState title={t("The hotel has no business date yet")}>
        <p className="muted">{t("Set the first one on the property, with Set the Business Date.")}</p>
        <Link to={`/e/HotelProperty/${property.id}`}>{t("Open {property_name}", { property_name: property.property_name })}</Link>
      </EmptyState>
    );
  if (stays.isPending || rooms.isPending) return <PageLoading />;
  if (stays.isError) return <ErrorState error={stays.error} onRetry={() => void stays.refetch()} />;
  if (rooms.isError) return <ErrorState error={rooms.error} onRetry={() => void rooms.refetch()} />;
  const arriving = stays.data.filter((s) => s.status === "Reserved" && s.arrival <= bd);
  const inHouse = stays.data.filter((s) => s.status === "Checked In");
  const leaving = inHouse.filter((s) => s.departure <= bd);
  const onAct = (stay: Stay, name: string) => move.mutate({ stay, name });
  return (
    <>
      <header className="row" style={{ justifyContent: "space-between" }}>
        <div>
          <h2>{property.property_name}</h2>
          <div className="faint">{t("Business date {bd}", { bd })}</div>
        </div>
        <Button
          loading={audit.isPending || opening}
          onClick={async () => {
            setOpening(true);
            const ok = await confirmDialog({
              title: `Close the night of ${bd}?`,
              message: "Room charges are posted to every stay in the house, reservations that did not arrive become no-shows, and the next day opens.",
              confirmLabel: "Run the night audit",
            });
            setOpening(false);
            if (ok) audit.mutate();
          }}
        >
          {t("Run the night audit")}
        </Button>
      </header>
      <div className="hotel-columns">
        <List title={t("Arriving")} stays={arriving} action={{ label: "Check in", name: "check_in" }} onAct={onAct} />
        <List title={t("In the house")} stays={inHouse} onAct={onAct} />
        <List title={t("Leaving")} stays={leaving} action={{ label: "Check out", name: "check_out" }} onAct={onAct} />
      </div>
      <section className="card card-pad stack" aria-label={t("Rooms")}>
        <h3>{t("Rooms")}</h3>
        <div className="hotel-rooms">
          {rooms.data.map((r) => (
            <Link key={r.id} className="hotel-room" to={`/e/Room/${r.id}`}>
              <strong>{r.room_number}</strong>
              <div>
                <Badge tone={TONE[r.housekeeping] ?? "neutral"}>{r.housekeeping}</Badge> {r.occupied ? <Badge tone="blue">{t("Occupied")}</Badge> : null}
              </div>
            </Link>
          ))}
        </div>
      </section>
    </>
  );
}

export function FrontDesk() {
  useShellTitle("Front desk");
  const q = useQuery({ queryKey: ["hotel", "properties"], queryFn: () => rows<Property>("HotelProperty", [], { sort: "property_name asc" }) });
  const [chosen, setChosen] = useState<string | null>(null);
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  if (q.data.length === 0)
    return (
      <EmptyState title={t("No property yet")}>
        <Link to="/e/HotelProperty/new">{t("Add your hotel")}</Link>
      </EmptyState>
    );
  const property = q.data.find((p) => p.id === chosen) ?? q.data[0];
  return (
    <div className="hotel-page">
      {q.data.length > 1 && (
        <label className="row" style={{ gap: 8 }}>
          {t("Property")}
          <select className="select" value={property.id} onChange={(e) => setChosen(e.target.value)}>
            {q.data.map((p) => (
              <option key={p.id} value={p.id}>
                {p.property_name}
              </option>
            ))}
          </select>
        </label>
      )}
      <Desk property={property} />
    </div>
  );
}
