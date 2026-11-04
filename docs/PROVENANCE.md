# Provenance

What this app took from elsewhere, and on what terms.

## ERPNext 12's Hotels module

- Source: [frappe/erpnext](https://github.com/frappe/erpnext), branch `version-12`, commit d6da1a9,
  `erpnext/hotels/doctype/`.
- Licence: GNU General Public License v3. This app is under the GNU Affero General Public License v3,
  which may be combined with GPL-3.0 code (section 13 of each licence).
- What was taken, as designs rather than code:
  - its doctypes Hotel Room, Hotel Room Type (with Hotel Room Amenity) and Hotel Room Pricing (with
    its items), adapted as Room, Room Type and Rate Plan with Rate Seasons;
  - the reservation's two rules (`hotel_room_reservation.py`): a night's rate is the pricing season
    that covers it, and a room type is not sold past its rooms on any night.
- Not taken, because the module has none: rooms assigned to stays, check-in and check-out, folios,
  the night audit, no-shows, occupancy, ADR and RevPAR. Those, and all the code, are this app's own.

The copy read is in `ERP/referance/industry/erpnext-v12` on the machine this was written on, sparse
(only `erpnext/hotels` and `erpnext/restaurant`).
