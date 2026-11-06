# Changelog

## 0.2.1

Rebuilt with Ferrum-Frontend bb60115: a record page reads an empty number or check as the server does, so an action whose condition needs one shows, and a workspace's settings open as their one record.

## 0.2.0

What the front desk is paid reaches the books: checking out hands each payment on the folio to ERP as a draft Payment Entry from the guest's customer into the property's new Takings Account (the company's default cash account when empty), with its date and reference, for accounts to apply to the invoice once it is submitted; the folio's payment says which entry it became. Until now a deposit was on the folio alone, and the invoice said the guest owed it all. Set the Business Date, Check In and Cancel the Reservation ask for what they need (the first business date, the room, why), which they read but never declared, so the first and the last could not work from their pages. Checking in, out, the night audit and a cancellation say what they did. Rebuilt with Ferrum-Frontend 0f1a544.

## 0.1.4

Rebuilt with Ferrum-Frontend 00dea07: in a workflow state that names what may change, the other fields show read-only, and the fields the state needs show required.

## 0.1.3

Personal-data requests find stay records by the person's phone number as well as their email, and list the folio, stay records holding their name apart, for an administrator to judge (never erased for the name alone) (Ferrum's FERRUM-PRIVACY-002).

## 0.1.2

Rebuilt with Ferrum-Frontend daeadda: after an action runs, a record page shows what the action said it did rather than only that it is done.

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
