# Hospitality

A hotel's front office for [Ferrum](https://github.com/CBFerrum/Ferrum): properties, room types and
rooms, rate plans by season, stays from reservation to check-out, folios, and the night audit that
closes each business date. Invoices go through ERP.

| | |
|---|---|
| 🏨 App | `hospitality`, native, in the server |
| 🧩 Package | `hospitality` (needs ERP's selling and accounting) |
| 🖥️ Interface | its own, at `/hotel/`: the front desk and the records |
| 🔐 Roles | Hotel Manager, Front Desk, Housekeeping |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **No room is sold twice, and no room type past its rooms.** Each night of a reservation is checked
  against the rooms of its type that are not out of order, and against its room when it has one. A
  lock on the room type means two reservations for its last room cannot both succeed.
- **A night's price is its season's.** A rate plan prices room types by dated seasons; a night no
  season covers is the room type's base rate.
- **Check-in only into a room that is ready**, clean or inspected and empty, and only from the arrival
  day on the hotel's business date. The stay's folio opens.
- **The night audit closes the business date.** Each stay in the house is charged its night, once,
  as a room charge the front desk cannot edit (it adds an adjustment instead). Reservations whose
  arrival day passed become no-shows. The night's rooms, occupancy, room revenue, ADR and RevPAR are
  kept, and the next day opens.
- **Check-out invoices the folio** as a draft Sales Invoice to the stay's customer, or to the
  property's walk-in customer. Room nights are the property's room item. The folio closes, and the
  room is marked for cleaning.
- **Personal data** requests find a guest by email and erase their name and contact from stays and
  folios. The nights and charges stay, as the books need them.

## How it works

| Record | What it is |
|---|---|
| Property | A hotel: its company, its room item, its walk-in customer, its business date |
| Room Type, Room | What is sold, and what there is; a room's housekeeping status and whether a guest is in it |
| Rate Plan | Seasons of nightly rates by room type |
| Stay | A guest's nights, room type and room, through Reserved, Checked In, Checked Out, Cancelled or No Show |
| Folio | A stay's charges and payments while the guest is in the house |
| Night Audit | What one business date closed with |

Actions: **Check In** (`room`), **Check Out** and **Cancel the Reservation** (`reason`) on a stay;
**Set the Business Date** (`date`, once) and **Run the Night Audit** on a property.

Adapted from ERPNext 12's Hotels module (GPL-3.0): room types, rooms, per-night pricing by season,
and availability by night. See [docs/PROVENANCE.md](docs/PROVENANCE.md).

## Develop

```bash
# Against the sibling platform checkout, with a git-ignored .cargo/config.toml patch, or Ferrum's main.
cargo test                       # needs FERRUM_HOSPITALITY_TEST_DATABASE_URL's database
cd ui && npm run build && npm run check
```

`tests/hospitality_it.rs` walks a stay end to end: reservations refused when full or held, check-in,
a minibar charge and a deposit, two night audits, a no-show, check-out with its invoice, a
cancellation that frees the night, and who may do what.

## Not yet

- Channel managers, online booking, groups and allotments, deposits taken at booking, and payment
  taken at check-out (the folio records payments; the invoice is ERP's to settle).
- Housekeeping tasks and rotas, and maintenance of rooms.
- Point-of-sale charges posted to a folio from Restaurants or POS.

## Changelog

| Version | Highlights |
|---|---|
| **0.1.2** | Rebuilt with Ferrum-Frontend daeadda: actions say what they did. |
| **0.1.1** | Rebuilt with Ferrum-Frontend 80ff179: labels name their fields, and linked titles show their titles. |
| **0.1.0** | Properties, room types and rooms, rate plans by season, stays from reservation to check-out, folios and the night audit (FERRUM-INDUSTRY-001). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
