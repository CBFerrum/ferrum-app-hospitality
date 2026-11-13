# Changelog

## 0.3.2

Rebuilt with Ferrum-Frontend 9e7190c, the design system as tuned after the landed build was reviewed. The left menu's icons are chosen so that none repeats on more than three rows and each fits its row; their drawings load beside the first page instead of inside it, so the page starts lighter; a row that scrolls sideways fades at the edge where there is more, in every browser and with reduced motion; a printed record leaves out empty fields and keeps its totals with the table; menus and popovers sit inside the page's landmarks for assistive technology; a selection bar folds what does not fit into More on a narrow screen. Nothing else changes.

## 0.3.1

The interface follows the upgraded design system, rebuilt with Ferrum-Frontend 76c4fad. Text uses the kit's type roles (nothing under 12px) and controls its height tokens; loading shows skeletons shaped like what is coming; empty, no matches, denied, offline and failed are told apart; status colours come from stored state ids. The interface check now runs the UI lint against an empty baseline. The home is rebuilt on the kit's work-home pattern: one page title, the main action as a button, queues with their counts, status at its natural width; its other own pages follow the same anatomy.

## 0.3.0

The hotel's operating cycle (FEO-045).

- **Deposits.** A rate plan asks a share of the stay (Deposit %) and says how many days before arrival a cancellation still gets it back (Refundable Until). Take a Deposit records it as a submitted payment entry, the payer's advance; it goes on the folio at check-in. Cancelled in time, a draft journal entry gives it back; cancelled late, or a no-show at the night audit, a draft cancellation-fee invoice keeps it (the property's Cancellation Fee Item, else its room night item).
- **Housekeeping.** Housekeeping Task: a departure for each room a check-out or a move frees, a stayover for each room a guest stays on in after the night audit. Cleaned, the room is Clean; inspected by a Housekeeping Supervisor or the Hotel Manager who did not clean it, Inspected. A property may check guests only into inspected rooms. The Housekeeping page lists what is to do.
- **Room charges.** An Outlet Cashier puts what a guest had at an outlet on their room: checked against the guest in it and their open folio, posted to the folio with the outlet's item, invoiced with the room. A posted charge does not change.
- **The booking page** (/hotel/book/{workspace}, public): availability by room type at the online rate plan with the deposit asked, booking with a name and email, and the guest's own link (only its SHA-256 kept on the stay; emailed where the server sends mail) to see or cancel the reservation by the same policy. Writes are limited per address.
- New roles Housekeeping Supervisor and Outlet Cashier; the front desk and the manager may create payment entries (deposits). Names in Arabic and Hindi for the new fields, and Open, Folio and Properties corrected.

## 0.2.11

Rebuilt with Ferrum-Frontend 43d7779: the names of its models, fields and manifest in Arabic and Hindi (149 each in ui/src/locales/names, which the platform serves to every page of the workspace, so a record's labels, the launcher and the bar read in the person's language). Translated by the model, to be reviewed by native speakers.

## 0.2.10

The assistant knows what its records mean in its pages (contributes.shell.assistant): their states, what counts and where figures come from, with questions it suggests asking there.

## 0.2.9

Rebuilt with Ferrum-Frontend 389b2af: its own words carry built-in Arabic and Hindi (23 words each, ui/src/locales, translated by the model and to be reviewed by native speakers), fetched for the page's language only. The labels it keeps in objects, its dialogs, its page titles and the words it keeps in variables go through t() as well, and the names and sizes it shows as they are keep their order right to left. Its pages are drawn once their words are in, and its entry carries only the parts of the kit it uses.

## 0.2.8

Rebuilt with Ferrum-Frontend 1f5981d: an action's link, code or token shown once is kept in a dialog to copy, and on a page that reads right to left, English left untranslated keeps its own order. Its own words go through the kit's t() (17 places), so a workspace's own words for its language reach them, and Admin, Interface words lists them under it (dist/words.json).

## 0.2.7

Rebuilt with Ferrum-Frontend 8a1c2e7: a workspace's own words for its interface (Admin, Interface words) reach its pages as they load; an attachment field that holds a picture shows it; and a value another record already has is said so, where the form said the record had been changed by someone else.

## 0.2.6

Rebuilt with Ferrum-Frontend 9d80f4f: right to left, its pages mirror as they read (icons that point along the line, margins and alignment by the line's start and end, arrow keys along it), and the records pages' words are in Arabic and Hindi.

## 0.2.5

Rebuilt with Ferrum-Frontend 204b713: a new record's box whose default is 1 shows ticked, as it is saved.

## 0.2.4

Rebuilt with Ferrum-Frontend 6cfda39: an action's dialog says every refusal, the errors on what the action made (another record's rows) among them, and an action with parameters and a confirmation asks once.

## 0.2.3

Rebuilt with Ferrum-Frontend b39fffb: its pages show times on a person's own clock when they chose one on their Account page, while today and due dates stay on the workspace's day.

## 0.2.2

Rebuilt with Ferrum-Frontend d350579: its pages show times on the workspace's clock, naming the zone when it is not the browser's, a draft numbered on submit is headed as a draft, and a record of an app that is not installed says so.

## 0.2.1

Rebuilt with Ferrum-Frontend 20d08e5: a record page reads an empty number or check as the server does, so an action whose condition needs one shows, and a workspace's settings open as their one record.

## 0.2.0

What the front desk is paid reaches the books: checking out hands each payment on the folio to ERP as a draft Payment Entry from the guest's customer into the property's new Takings Account (the company's default cash account when empty), with its date and reference, for accounts to apply to the invoice once it is submitted; the folio's payment says which entry it became. Until now a deposit was on the folio alone, and the invoice said the guest owed it all. Set the Business Date, Check In and Cancel the Reservation ask for what they need (the first business date, the room, why), which they read but never declared, so the first and the last could not work from their pages. Checking in, out, the night audit and a cancellation say what they did. Rebuilt with Ferrum-Frontend 22d49b9.

## 0.1.4

Rebuilt with Ferrum-Frontend ae13c69: in a workflow state that names what may change, the other fields show read-only, and the fields the state needs show required.

## 0.1.3

Personal-data requests find stay records by the person's phone number as well as their email, and list the folio, stay records holding their name apart, for an administrator to judge (never erased for the name alone) (Ferrum's FERRUM-PRIVACY-002).

## 0.1.2

Rebuilt with Ferrum-Frontend daeadda: after an action runs, a record page shows what the action said it did rather than only that it is done.

## 0.1.1

Rebuilt with Ferrum-Frontend 89b7cc7: a field's label names its control for assistive technology, and a record whose title is a link shows that record's title, not its id.

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
