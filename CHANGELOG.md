# Changelog

## 0.1.1

Rebuilt with Ferrum-Frontend 80ff179: a field's label names its control for assistive technology, and a record whose title is a link shows that record's title, not its id.

## 0.1.0

A hotel's front office (FERRUM-INDUSTRY-001).

- Properties, room types with amenities, rooms with their housekeeping status, and rate plans
  pricing room types by dated seasons.
- Stays: reservations held night by night against their room type and their room, check-in into a
  ready room from the arrival day, check-out with the folio invoiced through ERP, cancellation.
- Folios: the night audit's room charges, the front desk's other charges and adjustments, payments,
  and their totals; closed at check-out.
- The night audit: each stay in the house charged its night once, no-shows, the night's occupancy,
  room revenue, ADR and RevPAR, and the next business date.
- Its own interface at `/hotel/`: the front desk (arriving, in the house, leaving, the rooms) and the
  records.
