<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/banner-dark.svg">
  <img alt="Ferrum Hospitality: a hotel's front office on your own books" src="docs/assets/banner-light.svg" width="100%">
</picture>

<br>

[![Version](https://img.shields.io/badge/version-0.3.2-9a3412?style=flat-square)](CHANGELOG.md)
[![Licence: AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Ferrum platform](https://img.shields.io/badge/runs%20on-Ferrum-2f5bea?style=flat-square)](https://github.com/CBFerrum/Ferrum)
![Rust](https://img.shields.io/badge/Rust-server-b7410e?style=flat-square&logo=rust)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-data-336791?style=flat-square&logo=postgresql&logoColor=white)
![React](https://img.shields.io/badge/React-TypeScript-087ea4?style=flat-square&logo=react&logoColor=white)
![Tests](https://img.shields.io/badge/tests-2%20Rust%20%C2%B7%207%20UI-16a34a?style=flat-square)

### A hotel's front office, from the first reservation to the night audit, on the same books as the rest of the business.

Ferrum Hospitality is for small and mid-size hotels, guesthouses and serviced apartments that want a property management system they host themselves. It covers rooms and rates, a public booking page, check-in and check-out, folios, housekeeping and the night audit. It is an alternative to Mews, Cloudbeds, Apaleo or QloApps for a property that wants invoices and payments in its own ERP.

[💡 Why](#-why-hospitality) · [📸 Screenshots](#-screenshots) · [✨ Features](#-features) · [📊 Compare](#-how-it-compares) · [📍 Roadmap](#-roadmap) · [🚀 Install](#-install) · [🔧 Develop](#-develop)

</div>

## 💡 Why Hospitality

- 🔒 **No room is sold twice.** Every night of a reservation is counted against the rooms of its type under a lock, so two bookings for the last room cannot both succeed.
- 🧾 **One ledger with the hotel.** Check-out makes a draft sales invoice in ERP, and deposits are real payment entries. The restaurant, the spa and accounts use the same records.
- 🍽️ **Charges from outlets reach the room.** An outlet cashier puts what a guest had on their room, and the Restaurant app can send a bill to a room in one step.
- 🌐 **A booking page for guests.** Dates, party, room types free every night at the online rate, and a link of the guest's own to see or cancel the reservation.
- 🌙 **A real night audit.** It closes the business date, posts each night's room charge once and keeps occupancy, ADR and RevPAR.
- 🏠 **Self-hosted and open source.** AGPL-3.0, your server and your data, with no per-room fee.
- 🧩 **Extendable in Studio.** Add fields and rules to any record, and let AI agents work with the same records over REST and MCP under the same permissions.

## 🎯 At a glance

| | |
|---|---|
| **Version** | 0.3.2 |
| **Kind** | Its own interface at `/hotel/` (front desk and housekeeping) with the records screens for the rest, and a public booking page at `/hotel/book/{workspace}` |
| **What it adds** | 14 entities (9 you work with, 5 that sit inside them), 5 roles, 1 menu module, 1 package. It has no reports or print formats of its own: the Night Audit records are its figures |
| **Depends on** | Ferrum platform `>=0.3.2, <0.4` and [ERP](https://github.com/CBFerrum/ferrum-app-erp) `>=0.1` (selling and accounting) |
| **Works with** | [Restaurant](https://github.com/CBFerrum/ferrum-app-restaurant) (a bill on a guest's room), ERP (customers, items, invoices, payments, journals) |
| **Languages** | English, Arabic (right to left), Hindi |
| **Tests** | 2 Rust tests (one end-to-end walk of a stay), 7 interface tests |
| **Licence** | AGPL-3.0-only |

## 📸 Screenshots

<p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-home-dark.webp"><img alt="Hospitality — home" src="docs/assets/screens/hospitality-home-light.webp"></picture><br><b>🛎️ The front desk</b><br><sub>See who arrives, who is in the house and who leaves on the business date, and check them in or out.</sub></p>

<table>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-room-list-dark.webp"><img alt="Rooms — list" src="docs/assets/screens/hospitality-room-list-light.webp"></picture>
<p><b>🛏️ Rooms</b><br><sub>Every room with its type, floor, housekeeping status and whether a guest is in it.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-room-type-list-dark.webp"><img alt="Room types — list" src="docs/assets/screens/hospitality-room-type-list-light.webp"></picture>
<p><b>🏷️ Room types</b><br><sub>What is sold: capacity, base rate and amenities.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-stay-list-dark.webp"><img alt="Stays — list" src="docs/assets/screens/hospitality-stay-list-light.webp"></picture>
<p><b>📖 Stays</b><br><sub>Search and filter reservations and guests by arrival, status and room.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-room-detail-dark.webp"><img alt="Room — open record" src="docs/assets/screens/hospitality-room-detail-light.webp"></picture>
<p><b>🚪 A room</b><br><sub>Open one room to change its status and see its history.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-housekeeping-dark.webp"><img alt="Hospitality — housekeeping" src="docs/assets/screens/hospitality-housekeeping-light.webp"></picture>
<p><b>🧹 Housekeeping</b><br><sub>Rooms still to clean: mark them cleaned, then have a supervisor inspect them.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-stay-detail-dark.webp"><img alt="Stay — open record" src="docs/assets/screens/hospitality-stay-detail-light.webp"></picture>
<p><b>🧾 A stay</b><br><sub>Check a guest in or out, take a deposit and open their folio.</sub></p>
</td>
</tr>
</table>

<p align="center">
<span><picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-home-phone-dark.webp"><img alt="Hospitality — home" src="docs/assets/screens/hospitality-home-phone-light.webp" width="260"></picture><br><sub><b>🛎️ Front desk on a phone</b></sub></span>&nbsp;
<span><picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/hospitality-room-list-phone-dark.webp"><img alt="Rooms — list" src="docs/assets/screens/hospitality-room-list-phone-light.webp" width="260"></picture><br><sub><b>🛏️ Rooms on a phone</b></sub></span>&nbsp;
</p>

## ✨ Features

### 🏨 Properties, room types and rooms

- **Property.** A hotel with its company, the item room nights are invoiced as, a walk-in customer for stays with no named customer, a takings account, check-in and check-out times, an address and its **business date**. Several properties can live in one workspace.
- **Online switches.** A property chooses whether it books online and which rate plan guests are offered. **Rooms Inspected Before Check-in** makes a property send guests only to rooms a supervisor inspected.
- **Room types.** What is sold: capacity, extra-bed capacity, a base rate and amenities.
- **Rooms.** Number, floor, type and notes, with a housekeeping status (Clean, Inspected, Dirty, Out of Order) and whether a guest is in the room.
- **Amenities** are rows on the room type.

### 💶 Rates and deposits

- **Rate plans by season.** A rate plan prices room types by dated seasons. A night no season covers is the room type's base rate.
- **Breakfast and policy.** A rate plan says whether breakfast is included and carries a cancellation policy text.
- **Deposit by plan.** A plan asks a percentage of the stay's total when it is booked and says how many days before arrival a cancellation still gets it back.
- **Take a Deposit.** The front desk records an amount, a way of paying and a reference. The money is in the books at once as a submitted payment entry (the payer's advance until the folio's invoice), and the deposits become the folio's first payments at check-in.
- **Cancelled in time.** A draft journal entry gives the deposit back, for accounts to submit when the money goes out.
- **Cancelled late, or a no-show.** The hotel keeps it, as a draft invoice for a cancellation fee that accounts can apply the deposit to. The property names a cancellation-fee item, or the room night item is used.

### 📖 Stays: from reservation to check-out

| State | Meaning |
|---|---|
| **Reserved** | A booking that holds its nights and a room of its type |
| **Checked In** | The guest is in a room |
| **Checked Out** | The folio is closed and invoiced |
| **Cancelled** | With a reason, by the deposit policy |
| **No Show** | Set by the night audit when the arrival day passed |

- **A stay** holds the guest (name, email, phone, optional ERP customer), arrival and departure, nights, adults and children, room type, an optional room, a rate plan, the total, the source (Front Desk or Online), deposits and notes. Stays of up to 366 nights can be booked at the desk.
- **Availability by night.** Each night is counted against the room type's rooms that are not out of order, and against the chosen room when there is one.
- **Check In** takes a room. It needs a room that is ready (Clean, or Inspected where the property inspects), empty, and the arrival day or later on the hotel's business date. It opens the folio.
- **Check Out** closes the folio, makes the invoice and marks the room for cleaning.
- **Cancel the Reservation** asks for a reason and settles the deposit.
- **A guest moved to another room** puts the room it frees down for cleaning.

### 🧾 Folios and billing

- **A folio per stay,** open while the guest is in the house, with charges (Room, Service or Adjustment), payments and a running balance.
- **Room charges are fixed.** The night audit posts them and the front desk cannot edit them. To correct one, add an adjustment.
- **Check-out invoice.** A draft sales invoice to the stay's customer or the property's walk-in customer. Room nights use the property's room item.
- **Payments at the desk** reach ERP as payment entries.
- **Outlet charges.** An Outlet Cashier posts what a guest had at the restaurant or spa to their room. The charge is checked against the guest in the room (name, open folio), posted to the folio as a service charge with the outlet's item and invoiced with the room. The cashier sees neither the stay nor the folio, and a posted charge does not change.

### 🛎️ The front desk

- **Three lists on the business date.** Arriving, in the house and leaving, each row with its room and dates.
- **Check in and check out from the list.** One button on each row.
- **Rooms at a glance** with their housekeeping status.
- **Set the Business Date** once when a property starts, and **Run the Night Audit** from the desk.
- **Everything else** opens in the records screens, with search, filters, exports and the audit trail.

### 🧹 Housekeeping

- **Housekeeping tasks.** A check-out, or a guest moving rooms, creates a departure task for the room it frees. The night audit creates a stayover task for each room a guest stays on in.
- **Cleaned, then inspected.** A task goes from To Do to Cleaned. A supervisor (or the hotel manager) who did not clean the room marks it Inspected. A property may let guests in only to inspected rooms.
- **The Housekeeping page** lists the rooms still to do. Assignment is a free-text field today.

### 🌙 The night audit

- **Closes the business date.** Each stay in the house is charged its night, once, as a fixed room charge.
- **No-shows.** Reservations whose arrival day passed become no-shows, and a held deposit is kept.
- **Figures kept.** Rooms available, rooms occupied, occupancy percent, room revenue, average daily rate (ADR), revenue per available room (RevPAR), charges posted and no-shows, for each date.
- **The next day opens.** The business date moves on.

### 🌐 The public booking page

- **Search** by dates and party for a property that books online. The page lists the room types free on every night at the online rate plan's prices, with the deposit asked.
- **Book** with a name and email. The reservation holds its nights at once.
- **A link of the guest's own** shows the reservation and lets them cancel it while it is reserved, by the same policy. Only the link's SHA-256 is stored, and it works until 30 days after departure. It is emailed where the server sends mail.
- **Limits.** 20 bookings or cancellations per address per 10 minutes, parties of up to 20, stays of up to 30 nights, up to 500 days ahead. Longer stays are the desk's.
- **Guests never set prices, rooms or statuses.** Every request is checked on the server, through the same rules as the front desk.
- **A property that does not book online,** an unknown link and an expired one all answer not found.
- **The page asks no payment.** The deposit is taken by the front desk.

### 🔐 Roles, privacy and APIs

- **Five roles,** below. Row-level security applies to every table.
- **Personal data.** A request finds a guest by email or phone and erases their name and contact details from stays and folios. The nights and charges stay, as the books need them.
- **Records for free.** Lists, forms, search, imports (rooms and room types), exports, audit trail, REST and MCP.
- **The assistant** knows what the records mean and suggests questions such as who arrives today and what open folios still owe.

### 🌍 Languages

- **English, Arabic and Hindi** for the app's words and the names of its records. Pages mirror right to left. The Arabic and Hindi text was translated by a model and is waiting for review by native speakers.

## 🧭 How it works

```mermaid
flowchart LR
    G["Guest or front desk books"] --> R["Reserved: nights held"]
    R -->|"deposit paid"| D["Deposit in the books"]
    R -->|"arrival day"| CI["Check In to a ready room"]
    CI --> F["Folio: night charges, outlet charges, payments"]
    F --> CO["Check Out: draft invoice"]
    CO --> HK["Room set for cleaning"]
    R -->|"cancel"| X["Cancelled: deposit refunded or kept"]
    R -->|"arrival day passed"| NS["No Show at the night audit"]
```

A reservation holds its nights from the moment it is made. Check-in needs a ready room on the hotel's business date, and check-out invoices the folio through ERP.

```mermaid
flowchart LR
    NA["Night audit"] --> P["Post each stay's room night once"]
    NA --> S["No-shows and kept deposits"]
    NA --> K["Keep occupancy, ADR, RevPAR"]
    NA --> H["Stayover cleaning tasks"]
    NA --> N["Open the next business date"]
```

The night audit is the heartbeat of the property: nothing posts twice, and a closed date stays closed.

## 🧱 Data model

| Entity | What it is | Key fields or states |
|---|---|---|
| **Property** (HotelProperty) | A hotel | Company, room item, walk-in customer, takings account, online booking, online rate plan, inspect rooms, business date |
| **Room Type** | What is sold | Property, capacity, extra beds, base rate, amenities |
| **Room** | What there is | Number, type, floor, housekeeping (Clean, Inspected, Dirty, Out of Order), occupied |
| **Rate Plan** | Seasonal prices and policy | Seasons (room type, dates, rate), breakfast, deposit percent, refundable days, policy |
| **Stay** | A guest's nights | Guest, arrival, departure, room type, room, rate plan, total, source, deposits and their outcome, status |
| **Folio** | A stay's account | Charges, payments, total, balance, Open or Closed |
| **Night Audit** | What a business date closed with | Rooms, occupancy, revenue, ADR, RevPAR, no-shows |
| **Housekeeping Task** | A room to clean | Room, date, kind (Departure or Stayover), status (To Do, Cleaned, Inspected), who cleaned and inspected |
| **Room Charge** | What a guest had at an outlet | Room, guest, outlet, amount, item, folio |
| **Room Amenity, Rate Season, Stay Deposit, Folio Charge, Folio Payment** | Rows inside the records above | |

| Role | What it can do |
|---|---|
| **Hotel Manager** | Runs the property: rooms, rate plans, every stay and folio, the night audit, housekeeping tasks, customers and the payments and invoices it needs |
| **Front Desk** | Takes reservations, checks guests in and out, keeps folios and takes deposits |
| **Housekeeping** | Keeps rooms' housekeeping status and sees who is in which room |
| **Housekeeping Supervisor** | Inspects the rooms housekeeping cleaned |
| **Outlet Cashier** | Puts what a guest had at an outlet on their room, without seeing their stay or folio |

Actions: Check In, Check Out, Cancel the Reservation and Take a Deposit on a stay; Set the Business Date and Run the Night Audit on a property; Cleaned and Inspected on a housekeeping task.

## 🔌 Integrations

| With | How | Needs |
|---|---|---|
| **ERP** | Customers, items, companies, draft sales invoices, payment entries and journal entries | ERP installed (done for you) |
| **Restaurant** | A bill may be put on a guest's room, becoming a room charge on the folio | Both apps installed, the Outlet Cashier right |
| **Email** | The guest's reservation link | The server's mail |
| **Webhooks and the outbox** | Record events from the platform | An endpoint |
| **REST and MCP** | Every record and action, within each person's permissions | An API token |
| **Personal-data requests** | Export and erase a guest's data | None |

Channel managers, online travel agencies, payment gateways on the booking page and door-lock systems are not connected yet.

## 📊 How it compares

Leaders: Oracle OPERA Cloud, Mews, Cloudbeds, Apaleo and QloApps (open source).

| Capability | Ferrum Hospitality | OPERA Cloud | Mews | Cloudbeds | Apaleo | QloApps |
|---|:-:|:-:|:-:|:-:|:-:|:-:|
| Self-hosted, you hold the data | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Open-source licence | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| No per-room or per-property fee | ✅ | ❌ | ❌ | ❌ | ❌ | ✅¹ |
| Invoices and payments in the company's own ERP books | ✅ | 🟡² | 🟡² | 🟡² | 🟡² | 🟡² |
| Room types and rooms, availability by night | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Several properties | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Seasonal rates per room type | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Rate rules: weekday, minimum stay, per person, corporate | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Deposits and cancellation policy by rate plan | 🟡³ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Public booking page | 🟡⁴ | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Card payment or pre-authorisation online | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Channel manager and OTA sync | ❌ | 🟡⁵ | 🟡⁵ | ✅ | 🟡⁵ | 🟡⁵ |
| Front-desk lists: arrivals, in house, departures | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tape chart or room calendar | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Group and multi-room reservations | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Guest profiles with history and preferences | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Folio with charges, payments and check-out invoice | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Split folios and company billing | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Outlet (restaurant, spa) charges on the room | ✅ | ✅ | ✅ | 🟡 | 🟡 | ❌ |
| Housekeeping status, cleaning and inspection | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Housekeeping rotas and an attendant app | ❌ | ✅ | ✅ | ✅ | 🟡 | ❌ |
| Night audit that closes the business date | ✅ | ✅ | 🟡⁶ | 🟡⁶ | 🟡⁶ | ❌ |
| Occupancy, ADR and RevPAR figures | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Forecast, pickup and segment reports | ❌ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Online check-in, registration card, ID capture | ❌ | ✅ | ✅ | ✅ | 🟡 | ❌ |
| Guest messaging by email, SMS and WhatsApp | 🟡⁷ | ✅ | ✅ | ✅ | 🟡 | 🟡 |
| Room tax, city tax and service charge by tariff | 🟡⁸ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Open API and an integration marketplace | 🟡⁹ | ✅ | ✅ | ✅ | ✅ | 🟡 |

<sub>¹ QloApps is free to use; paid modules exist. ² Through accounting integrations, not one data set. ³ One percentage and one refund cut-off by plan; a sliding scale is not built, and the money is returned by a draft journal entry a person submits. ⁴ One online rate plan per property, no payment on the page. ⁵ Through connected channel managers. ⁶ Closing the day is automated or lighter than a night audit. ⁷ Only the booking-link email. ⁸ Invoice lines carry the item and rate; room and city tax by tariff are not built. ⁹ REST, MCP and webhooks from the platform; no marketplace.</sub>

### 🏆 Where Hospitality stands out

- 🔒 No room type is sold past its rooms on any night, checked under a lock.
- 🧾 Deposits, invoices and cancellation fees land as real ERP records: payment entries, draft invoices and draft journals.
- 🌙 A night audit that posts each night once and keeps occupancy, ADR and RevPAR.
- 🍽️ A restaurant bill goes on a guest's room through the Restaurant app, and the cashier sees neither stay nor folio.
- 🏠 Self-hosted, open source, no per-room fee, with Arabic and Hindi.

### 🚧 Where the leaders are still ahead

- 📅 **A tape chart** to see and move stays by room and night.
- 🔗 **Channel managers and travel agencies,** two-way.
- 💳 **Online payment** on the booking page and card pre-authorisation.
- 👨‍👩‍👧 **Groups, multi-room bookings and allotments,** and split or routed folios.
- 💶 **Rate rules** for weekdays, minimum stay, per-person prices and packages, and room and city taxes.
- 🧹 **Housekeeping depth:** rotas, assignment, dated out-of-order blocks and a mobile list.
- 🪪 **Guest profiles,** online check-in, registration cards and ID capture.
- 📊 **Management reports:** daily flash, forecast and pickup.

> [!NOTE]
> Leader columns are from public product information as of November 2026 and may change. Features marked 🟡 for a leader depend on plan, region or add-on.

## 📍 Roadmap

### ✅ Recently shipped

- [x] **0.3.2:** the interface on the tuned design system, with better menu icons and cleaner print.
- [x] **0.3.1:** the redesigned interface, with the front desk on the work-home pattern.
- [x] **0.3.0:** deposits by the rate plan's policy, kept for late cancellations and no-shows; housekeeping tasks cleaned and inspected; outlets' charges on a guest's room; a public booking page with the guest's own link.
- [x] **0.2.11:** names in Arabic and Hindi.
- [x] **0.2.0:** payments taken at the desk reach ERP at check-out; the business date, check-in's room and a cancellation's reason are asked for on their pages.

### 🔜 Next up

**Sales**
- [ ] Card payment and pre-authorisation on the booking page and at the desk, through a payment gateway.
- [ ] Channel manager and travel agency sync of rates, availability and reservations.
- [ ] Rate rules: day of week, minimum stay, closed to arrival, per-person and extra-bed prices, corporate and promo rates, packages.

**Front office**
- [ ] A tape chart: see and move stays by room and night.
- [ ] Group and multi-room reservations, room blocks and allotments.
- [ ] Split folios, routing to a company bill and transfers between folios.
- [ ] A guest profile: repeat guests, preferences, ID, nationality and flags.
- [ ] Registration card, ID capture, signature and pre-arrival online check-in.

**Money**
- [ ] Room tax, city tax and service charge by tariff, with the country's rules added as an add-on, not in the core.
- [ ] Paying deposits back and applying advances in ERP without a person finishing the draft.

### 🔭 Later

- [ ] Housekeeping assignment and rotas, an attendant list on a phone, and dated out-of-order blocks with a reason.
- [ ] Guest messages: confirmation, pre-arrival and post-stay by email, SMS and WhatsApp, from templates.
- [ ] Reports: daily flash, arrivals and departures, forecast and pickup, segment and source mix, cashier shift close, city ledger ageing.
- [ ] Outlet charges posted by the outlet's own till instead of typed.
- [ ] Booking page depth: several properties, add-ons and upsell, promo codes.
- [ ] Reporting of foreign guests to the authorities, as a country add-on.
- [ ] Limits now fixed in code (the longest stay, online party size and window) as property settings.
- [ ] Keycards, door locks, telephony and revenue management.

## 🚀 Install

Hospitality is a native Ferrum app. It ships in the official Ferrum server image, and a workspace administrator installs it from **Apps** (`/store/`). It requires ERP `>=0.1`, and installing it in a workspace without ERP installs ERP whole. Self-hosters who build their own image enable the `hospitality` feature of the server.

First steps after installing:

1. Give people a role: **Hotel Manager**, **Front Desk**, **Housekeeping**, **Housekeeping Supervisor** or **Outlet Cashier**.
2. Create a **Property** with its company, the item room nights are invoiced as and a walk-in customer. Run **Set the Business Date** once.
3. Add **Room Types** and **Rooms**.
4. Add a **Rate Plan** with its seasons, and a deposit percent and refund days if you take deposits.
5. To book online, switch **Booked Online** on the property, choose the online rate plan (empty, the room types' base rates are sold), and share `/hotel/book/{workspace}`.
6. To let guests' restaurant bills go on their room, install Restaurant and give the cashier the **Outlet Cashier** role.

## 🔧 Develop

Clone it next to the platform:

```text
work/
├── ferrum/                  https://github.com/CBFerrum/Ferrum (with Ferrum-Frontend cloned into ferrum/web)
└── ferrum-app-hospitality/
```

Point the platform dependencies at your checkout (this file is git-ignored):

```toml
# .cargo/config.toml
[patch."https://github.com/CBFerrum/Ferrum"]
ferrum-domain = { path = "../ferrum/crates/domain" }
ferrum-kernel = { path = "../ferrum/crates/kernel" }
ferrum-meta = { path = "../ferrum/crates/meta" }
```

The tests need PostgreSQL with an ordinary (non-superuser) role, so row-level security applies:

```bash
createuser ferrum --pwprompt && createdb -O ferrum ferrum_hospitality_test
export FERRUM_HOSPITALITY_TEST_DATABASE_URL=postgres://ferrum:ferrum@localhost:5432/ferrum_hospitality_test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Its tests also build on [ferrum-app-erp](https://github.com/CBFerrum/ferrum-app-erp), which Cargo takes from GitHub. The interface (`ui/`) is built with the platform's packages from the sibling `ferrum/web` checkout, and its built files (`ui/dist`) are committed and embedded in the crate:

```bash
cd ui
npm run build && npm run check && npm run typecheck && npm test
```

The end-to-end test `tests/hospitality_it.rs` walks a stay: reservations refused when full or held, check-in, a minibar charge and a deposit, two night audits, a no-show, check-out with its invoice, a cancellation that frees the night, and who may do what.

## 📂 Project layout

```text
app.yaml          the manifest: id, version, requirements, what it adds to Ferrum
packages/         metadata: entities, roles, menu, a package guide
src/              the logic in Rust: stays, folios, charges, housekeeping, night audit, booking page
ui/               its interface: front desk, housekeeping, the public booking page (React, TypeScript)
tests/            an end-to-end test against a real PostgreSQL
docs/PROVENANCE.md  what the design took from ERPNext 12's Hotels module
CHANGELOG.md      what changed in each version
```

Room types, rooms and per-night pricing by season are adapted as designs from ERPNext 12's Hotels module (GPL-3.0); see [docs/PROVENANCE.md](docs/PROVENANCE.md).

## 📜 Changelog

Every version is in [CHANGELOG.md](CHANGELOG.md).

## 🤝 Contributing

Issues and pull requests are welcome at [github.com/CBFerrum/ferrum-app-hospitality](https://github.com/CBFerrum/ferrum-app-hospitality/issues). Run the checks in Develop first.

## 📄 License

[AGPL-3.0-only](LICENSE), like the platform.

<div align="center">

<sub>Part of <a href="https://github.com/CBFerrum/Ferrum">Ferrum</a>, the open business platform · Made with 🦀 Rust and 🐘 PostgreSQL</sub>

</div>
