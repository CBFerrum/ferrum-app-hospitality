# Hospitality

A hotel's front office for [Ferrum](https://github.com/CBFerrum/Ferrum): properties, room types and
rooms, rate plans by season with their deposits, a public booking page, stays from reservation to
check-out, folios with outlets' charges, housekeeping, and the night audit that closes each business
date. Invoices and payments go through ERP.

| | |
|---|---|
| 🏨 App | `hospitality`, native, in the server |
| 🧩 Package | `hospitality` (needs ERP's selling and accounting) |
| 🖥️ Interface | its own, at `/hotel/`: the front desk, housekeeping and the records; the public booking page at `/hotel/book/{workspace}` |
| 🔐 Roles | Hotel Manager, Front Desk, Housekeeping, Housekeeping Supervisor, Outlet Cashier |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **No room is sold twice, and no room type past its rooms.** Each night of a reservation is checked
  against the rooms of its type that are not out of order, and against its room when it has one. A
  lock on the room type means two reservations for its last room cannot both succeed.
- **A night's price is its season's.** A rate plan prices room types by dated seasons; a night no
  season covers is the room type's base rate.
- **Check-in only into a room that is ready**, clean or inspected and empty (inspected, where the
  property inspects its rooms), and only from the arrival day on the hotel's business date. The
  stay's folio opens, with the deposits taken before as its first payments.
- **Deposits by the rate plan.** A rate plan asks a share of the stay's total when it is booked, and
  says how many days before arrival a cancellation still gets it back. Take a Deposit puts the money
  in the books at once (a submitted payment entry, the payer's advance until the folio's invoice).
  Cancelled in time, the deposit is to be given back: a draft journal entry for accounts to submit
  when the money goes out. Cancelled late, or a no-show at the night audit, the hotel keeps it: a
  draft invoice for it as a cancellation fee, for accounts to apply the deposit to.
- **Housekeeping.** Check-out (and a guest moving rooms) puts the room it frees down for cleaning; the
  night audit, each room a guest stays on in for its service. Cleaned, the room is Clean; a
  departure inspected by a supervisor who did not clean it is Inspected. The Housekeeping page lists
  what is still to do.
- **Charges from outlets on a guest's room.** An outlet's cashier (the restaurant, the spa) puts what
  a guest had on their room: checked against the guest in it (their name, an open folio), posted to
  the folio as a service charge with the outlet's item, and invoiced with the room. The cashier sees
  neither the stay nor the folio, and a posted charge does not change.
- **A booking page.** A property that books online has a public page, /hotel/book/{workspace}: guests
  pick their dates and party, see the room types free every night at the online rate plan's prices
  with the deposit asked, and book one with their name and email. They get their reservation's own
  link (emailed where the server sends mail), where they see it and may cancel it by the same
  policy. Only the link's hash is kept. Bookings and cancellations are limited per address.
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
| Housekeeping Task | A room to clean on a business date, cleaned and then inspected |
| Room Charge | What a guest had at an outlet, put on their room's folio |
| Night Audit | What one business date closed with |

Actions: **Check In** (`room`), **Check Out**, **Cancel the Reservation** (`reason`) and **Take a
Deposit** (`amount`, `mode`, `reference`) on a stay; **Set the Business Date** (`date`, once) and
**Run the Night Audit** on a property; **Cleaned** and **Inspected** on a housekeeping task.

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

- Channel managers and online travel agencies, groups and allotments, and card payments taken online
  (the booking page asks no payment; deposits are taken by the front desk).
- Housekeeping rotas and assignment rules, and maintenance of rooms (out of order is set by hand).
- Paying deposits back and applying advances in ERP (drafts are made for accounts).

## Changelog

| Version | Highlights |
|---|---|
| **0.3.0** | Deposits by the rate plan's policy, kept for late cancellations and no-shows; housekeeping tasks cleaned and inspected; outlets' charges on a guest's room; a public booking page with the guest's own link (FEO-045). |
| **0.2.11** | Its names in Arabic and Hindi. |
| **0.2.10** | The assistant knows its records. |
| **0.2.9** | Rebuilt with Ferrum-Frontend 5b22e7b: its own words in Arabic and Hindi. |
| **0.2.8** | Rebuilt with Ferrum-Frontend 6537d65: once-shown answers kept; English isolated right to left; its own words translatable by a workspace. |
| **0.2.7** | Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own interface words; pictures in attachment fields; a duplicate value said so. |
| **0.2.6** | Rebuilt with Ferrum-Frontend 63280d3: right to left, pages mirror; the records pages' words in Arabic and Hindi. |
| **0.2.5** | Rebuilt with Ferrum-Frontend a64416a: a box whose default is 1 shows ticked. |
| **0.2.4** | Rebuilt with Ferrum-Frontend 922fcfb: action dialogs say every refusal and ask once. |
| **0.2.3** | Rebuilt with Ferrum-Frontend a76295e: times on a person's own clock when they chose one. |
| **0.2.2** | Rebuilt with Ferrum-Frontend 24c3951: times on the workspace's clock; drafts headed as drafts. |
| **0.2.1** | Rebuilt with Ferrum-Frontend bb60115: empty numbers read as the server reads them; settings open as their record. |
| **0.2.0** | Payments taken at the desk reach ERP at check-out; the business date, check-in's room and a cancellation's reason are asked for on their pages. |
| **0.1.4** | Rebuilt with Ferrum-Frontend 00dea07: a workflow state's kept fields are read-only, its needed ones required. |
| **0.1.3** | Personal-data requests find stay records by the person's phone number as well as their email, and list the folio, stay records holding their name apart, for an … |
| **0.1.2** | Rebuilt with Ferrum-Frontend daeadda: actions say what they did. |
| **0.1.1** | Rebuilt with Ferrum-Frontend 80ff179: labels name their fields, and linked titles show their titles. |
| **0.1.0** | Properties, room types and rooms, rate plans by season, stays from reservation to check-out, folios and the night audit (FERRUM-INDUSTRY-001). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
