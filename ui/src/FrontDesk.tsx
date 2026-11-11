// The front desk on the property's business date: who arrives, who is in the house, who leaves, and
// the rooms as housekeeping left them. Checking in, checking out and the night audit happen here;
// everything else opens in the records frame.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { api, docPath, formatValue, refusal, t, isolate } from "@ferrum/client";
import { Button, ButtonLink, confirmDialog, HomeSection, HomeSkeleton, LoadFailed, PageHead, Queue, QueueGrid, QueueItem, StatePanel, StatusBadge, Tile, TileGrid, toast, type Tone } from "@ferrum/kit";

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

/** Room.housekeeping as stored (packages/hospitality/entities), for `statusTone`: four states, four tones, and the word
 *  is always shown beside it. */
export const ROOM_TONES: Readonly<Record<string, Tone>> = { Clean: "blue", Inspected: "green", Dirty: "amber", "Out of Order": "red" };

/** A day as the person reads it ("11 Nov 2026"). */
const dayText = (v: string) => formatValue({ type: "date" }, v);

/** A stay's day in a row, without its year ("14 Nov"): the desk works within days of the business date, and a row is narrow. */
export const shortDay = (v: string): string => new Intl.DateTimeFormat(typeof document === "undefined" ? "en" : document.documentElement.lang || undefined, { day: "numeric", month: "short" }).format(new Date(`${v}T00:00:00`));

/** The number of the room a stay holds (its `room` is the room's id); empty until one is given. */
export const roomNumber = (stay: Pick<Stay, "room">, rooms: Pick<Room, "id" | "room_number">[]): string => rooms.find((r) => r.id === stay.room)?.room_number ?? "";

function List({ title, stays, rooms, action, busy, onAct }: { title: string; stays: Stay[]; rooms: Room[]; action?: { label: string; name: string }; busy: string | null; onAct: (s: Stay, name: string) => void }) {
  return (
    <Queue title={title} count={stays.length} empty={t("Nobody.")}>
      {stays.map((s) => (
        <QueueItem
          key={s.id}
          to={`/e/Stay/${s.id}`}
          lead={isolate(roomNumber(s, rooms)) || "—"}
          title={isolate(s.guest_name)}
          meta={`${isolate(s.number)} · ${t("{arrival} to {departure}", { arrival: shortDay(s.arrival), departure: shortDay(s.departure) })}`}
          action={
            action ? (
              <Button size="sm" loading={busy === s.id} onClick={() => onAct(s, action.name)}>
                {isolate(action.label)}
              </Button>
            ) : undefined
          }
        />
      ))}
    </Queue>
  );
}

function Desk({ property, picker }: { property: Property; picker: ReactNode }) {
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
      toast(v.name === "check_in" ? t("{guest_name} is in", { guest_name: v.stay.guest_name }) : t("{guest_name} has left", { guest_name: v.stay.guest_name }));
      done();
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  const audit = useMutation({
    mutationFn: () => act("HotelProperty", property.id, "night_audit"),
    onSuccess: (r) => {
      toast(t("The night of {value} is closed: {value2} rooms charged, {value3} no-shows", { value: String(r.night), value2: String(r.charges_posted), value3: String(r.no_shows) }));
      done();
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  const [opening, setOpening] = useState(false);
  const nameLine = isolate(property.property_name);
  if (!bd)
    return (
      <>
        <PageHead title={t("Front desk")} description={nameLine} actions={picker} />
        <StatePanel
          kind="empty"
          title={t("The hotel has no business date yet")}
          action={
            <ButtonLink to={`/e/HotelProperty/${property.id}`} variant="primary">
              {t("Open {property_name}", { property_name: property.property_name })}
            </ButtonLink>
          }
        >
          {t("Set the first one on the property, with Set the Business Date.")}
        </StatePanel>
      </>
    );
  if (stays.isPending || rooms.isPending)
    return (
      <>
        <PageHead title={t("Front desk")} description={nameLine} actions={picker} />
        <HomeSkeleton />
      </>
    );
  if (stays.isError || rooms.isError)
    return (
      <>
        <PageHead title={t("Front desk")} description={nameLine} actions={picker} />
        <LoadFailed error={stays.error ?? rooms.error} onRetry={() => void (stays.isError ? stays : rooms).refetch()} />
      </>
    );
  const arriving = stays.data.filter((s) => s.status === "Reserved" && s.arrival <= bd);
  const inHouse = stays.data.filter((s) => s.status === "Checked In");
  const leaving = inHouse.filter((s) => s.departure <= bd);
  const onAct = (stay: Stay, name: string) => move.mutate({ stay, name });
  const busy = move.isPending ? move.variables.stay.id : null;
  return (
    <>
      <PageHead
        title={t("Front desk")}
        description={`${nameLine} · ${t("Business date {bd}", { bd: dayText(bd) })}`}
        actions={
          <>
            {picker}
            <ButtonLink to="/housekeeping">{t("Housekeeping")}</ButtonLink>
            <Button
              loading={audit.isPending || opening}
              onClick={async () => {
                setOpening(true);
                const ok = await confirmDialog({
                  title: t("Close the night of {bd}?", { bd: dayText(bd) }),
                  message: t("Room charges are posted to every stay in the house, reservations that did not arrive become no-shows, and the next day opens."),
                  confirmLabel: t("Run the night audit"),
                });
                setOpening(false);
                if (ok) audit.mutate();
              }}
            >
              {t("Run the night audit")}
            </Button>
          </>
        }
      />
      <QueueGrid>
        <List title={t("Arriving")} stays={arriving} rooms={rooms.data} action={{ label: t("Check in"), name: "check_in" }} busy={busy} onAct={onAct} />
        <List title={t("In the house")} stays={inHouse} rooms={rooms.data} busy={busy} onAct={onAct} />
        <List title={t("Leaving")} stays={leaving} rooms={rooms.data} action={{ label: t("Check out"), name: "check_out" }} busy={busy} onAct={onAct} />
      </QueueGrid>
      <HomeSection title={t("Rooms")} count={rooms.data.length}>
      <TileGrid label={t("Rooms")}>
        {rooms.data.map((r) => (
          <Tile key={r.id} to={`/e/Room/${r.id}`} title={isolate(r.room_number)} meta={r.occupied ? <span>{t("Occupied")}</span> : undefined} status={<StatusBadge value={r.housekeeping} tones={ROOM_TONES} />} />
        ))}
      </TileGrid>
      </HomeSection>
    </>
  );
}

export function FrontDesk() {
  const q = useQuery({ queryKey: ["hotel", "properties"], queryFn: () => rows<Property>("HotelProperty", [], { sort: "property_name asc" }) });
  const [chosen, setChosen] = useState<string | null>(null);
  if (q.isPending)
    return (
      <>
        <PageHead title={t("Front desk")} />
        <HomeSkeleton />
      </>
    );
  if (q.isError)
    return (
      <>
        <PageHead title={t("Front desk")} />
        <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
      </>
    );
  if (q.data.length === 0)
    return (
      <>
        <PageHead title={t("Front desk")} />
        <StatePanel
          kind="empty"
          title={t("No property yet")}
          action={
            <ButtonLink to="/e/HotelProperty/new" variant="primary" icon="plus">
              {t("Add your hotel")}
            </ButtonLink>
          }
        />
      </>
    );
  const property = q.data.find((p) => p.id === chosen) ?? q.data[0];
  const picker =
    q.data.length > 1 ? (
      <label className="hotel-pick">
        <span className="muted">{t("Property")}</span>
        <select className="select" value={property.id} onChange={(e) => setChosen(e.target.value)}>
          {q.data.map((p) => (
            <option key={p.id} value={p.id}>
              {isolate(p.property_name)}
            </option>
          ))}
        </select>
      </label>
    ) : null;
  return <Desk property={property} picker={picker} />;
}
