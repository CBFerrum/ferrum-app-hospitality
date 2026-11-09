//! Stays: a guest's nights, held against their room and their room type, from reservation to
//! check-out. Availability follows ERPNext 12's Hotel Room Reservation (each night, the rooms of a
//! type sold must not pass the rooms there are), with a lock on the room type so two reservations
//! for its last room cannot both succeed.

use crate::housekeeping::task;
use crate::{date, obj};
use async_trait::async_trait;
use chrono::{Duration, NaiveDate, Utc};
use ferrum_domain::flow::{create_for_user, created};
use ferrum_domain::util::{d, elevated, id, money, round, rows, s, set};
use ferrum_kernel::{Action, Doc, DocHook, Error, HookCtx, Result, Tx};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use uuid::Uuid;

/// The most nights one stay holds.
const MAX_NIGHTS: i64 = 366;

/// Holding a room: the stay is booked or in the house.
pub(crate) const ACTIVE: &[&str] = &["Reserved", "Checked In"];

/// An amount as people read it.
fn money_text(v: Decimal) -> String {
    format!("{v:.2}")
}

/// One night's price for a room type: the rate plan's season that covers it when the plan is
/// active, else the type's base rate.
pub(crate) async fn night_rate(tx: &mut Tx, tenant: Uuid, room_type: Uuid, plan: Option<Uuid>, night: NaiveDate) -> Result<Decimal> {
    if let Some(plan) = plan {
        let season: Option<Decimal> = sqlx::query_scalar(
            "select s.rate from e_rate_plan__seasons s join e_rate_plan p on p.tenant_id = s.tenant_id and p.id = s.parent_id \
             where s.tenant_id = $1 and s.parent_id = $2 and p.active and s.room_type = $3 and $4 between s.from_date and s.to_date \
             order by s.from_date desc, s.idx limit 1",
        )
        .bind(tenant)
        .bind(plan)
        .bind(room_type)
        .bind(night)
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(rate) = season {
            return Ok(rate);
        }
    }
    let base: Option<Decimal> =
        sqlx::query_scalar("select base_rate from e_room_type where tenant_id = $1 and id = $2").bind(tenant).bind(room_type).fetch_optional(&mut **tx).await?;
    base.ok_or_else(|| Error::field("room_type", "is not a room type here"))
}

fn nights_of(doc: &Doc) -> Result<(NaiveDate, NaiveDate, i64)> {
    let arrival = date(doc, "arrival").ok_or_else(|| Error::field("arrival", "is required"))?;
    let departure = date(doc, "departure").ok_or_else(|| Error::field("departure", "is required"))?;
    let nights = (departure - arrival).num_days();
    if nights < 1 {
        return Err(Error::field("departure", "is after the arrival: a stay is at least one night"));
    }
    if nights > MAX_NIGHTS {
        return Err(Error::field("departure", format!("is at most {MAX_NIGHTS} nights after the arrival")));
    }
    Ok((arrival, departure, nights))
}

/// The property a record belongs to, checked against the stay's.
async fn property_of(tx: &mut Tx, tenant: Uuid, table: &'static str, id: Uuid) -> Result<Option<Uuid>> {
    let sql = match table {
        "room_type" => "select property from e_room_type where tenant_id = $1 and id = $2",
        "room" => "select property from e_room where tenant_id = $1 and id = $2",
        _ => "select property from e_rate_plan where tenant_id = $1 and id = $2",
    };
    Ok(sqlx::query_scalar(sql).bind(tenant).bind(id).fetch_optional(&mut **tx).await?.flatten())
}

/// Whether the stay's nights can be held: its room type has a room free each night, and its room,
/// when it has one, is nobody else's on any of them.
async fn hold(tx: &mut Tx, tenant: Uuid, doc: &Doc) -> Result<()> {
    let (arrival, departure, _) = nights_of(doc)?;
    let me = id(doc, "id").unwrap_or_else(Uuid::nil);
    let room_type = id(doc, "room_type").ok_or_else(|| Error::field("room_type", "is required"))?;
    // One reservation of a type at a time decides whether its last room is free.
    let type_name: Option<String> = sqlx::query_scalar("select room_type_name from e_room_type where tenant_id = $1 and id = $2 for update")
        .bind(tenant)
        .bind(room_type)
        .fetch_optional(&mut **tx)
        .await?;
    let type_name = type_name.ok_or_else(|| Error::field("room_type", "is not a room type here"))?;
    let rooms: i64 = sqlx::query_scalar("select count(*) from e_room where tenant_id = $1 and room_type = $2 and housekeeping <> 'Out of Order'")
        .bind(tenant)
        .bind(room_type)
        .fetch_one(&mut **tx)
        .await?;
    let full: Option<NaiveDate> = sqlx::query_scalar(
        "select n::date from generate_series($3::date, $4::date - 1, interval '1 day') n \
         where (select count(*) from e_stay s where s.tenant_id = $1 and s.room_type = $2 and s.status = any($6) and s.id <> $5 \
                and s.arrival <= n::date and s.departure > n::date) >= $7 \
         order by n limit 1",
    )
    .bind(tenant)
    .bind(room_type)
    .bind(arrival)
    .bind(departure)
    .bind(me)
    .bind(ACTIVE)
    .bind(rooms)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(night) = full {
        return Err(Error::invalid(format!("no {type_name} room is free on the night of {night}: all {rooms} are taken")));
    }
    if let Some(room) = id(doc, "room") {
        let taken: Option<String> = sqlx::query_scalar(
            "select coalesce(s.number, s.guest_name) from e_stay s where s.tenant_id = $1 and s.room = $2 and s.status = any($6) and s.id <> $5 \
             and s.arrival < $4 and s.departure > $3 order by s.arrival limit 1",
        )
        .bind(tenant)
        .bind(room)
        .bind(arrival)
        .bind(departure)
        .bind(me)
        .bind(ACTIVE)
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(other) = taken {
            return Err(Error::field("room", format!("is held by {other} on some of these nights")));
        }
    }
    Ok(())
}

pub struct StayHook;

#[async_trait]
impl DocHook for StayHook {
    fn name(&self) -> &'static str {
        "hospitality.stay"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["nights", "total", "deposit_due"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        let Ok((arrival, _, nights)) = nights_of(doc) else { return Ok(()) };
        set(doc, "nights", json!(nights));
        if let Some(room_type) = id(doc, "room_type") {
            let mut total = Decimal::ZERO;
            for n in 0..nights {
                total += night_rate(hc.tx, hc.ctx.tenant, room_type, id(doc, "rate_plan"), arrival + Duration::days(n)).await?;
            }
            set(doc, "total", money(total));
            // What its rate plan asks before the stay.
            let share = match id(doc, "rate_plan") {
                Some(plan) => policy(hc.tx, hc.ctx.tenant, plan).await?.0,
                None => Decimal::ZERO,
            };
            set(doc, "deposit_due", money(round(total * share / Decimal::ONE_HUNDRED, 2)));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        let tenant = hc.ctx.tenant;
        let property = id(doc, "property").ok_or_else(|| Error::field("property", "is required"))?;
        let status = s(doc, "status").unwrap_or("Reserved");
        let changed = |k: &str| old.is_none_or(|o| o.get(k) != doc.get(k));
        if let Some(o) = old {
            let was = s(o, "status").unwrap_or("Reserved");
            // What a finished stay was stays as it was.
            if !ACTIVE.contains(&was) && ["property", "arrival", "departure", "room_type", "room", "rate_plan", "customer"].iter().any(|k| changed(k)) {
                return Err(Error::invalid(format!("a stay that is {} does not change", was.to_lowercase())));
            }
            if was == "Checked In" && ["property", "arrival", "room_type"].iter().any(|k| changed(k)) {
                return Err(Error::invalid("the guest is in the house: only the departure and the room can change"));
            }
        }
        nights_of(doc)?;
        for (field, table) in [("room_type", "room_type"), ("room", "room"), ("rate_plan", "rate_plan")] {
            if let Some(v) = id(doc, field)
                && property_of(hc.tx, tenant, table, v).await? != Some(property)
            {
                return Err(Error::field(field, "belongs to another property"));
            }
        }
        if let (Some(room), Some(room_type)) = (id(doc, "room"), id(doc, "room_type")) {
            let of: Option<Uuid> = sqlx::query_scalar("select room_type from e_room where tenant_id = $1 and id = $2")
                .bind(tenant)
                .bind(room)
                .fetch_optional(&mut **hc.tx)
                .await?;
            if of != Some(room_type) {
                return Err(Error::field("room", "is not a room of this room type"));
            }
        }
        if ACTIVE.contains(&status) && ["arrival", "departure", "room_type", "room"].iter().any(|k| changed(k)) {
            hold(hc.tx, tenant, doc).await?;
            // A guest in the house moves only into a room that is ready.
            if let Some(room) = id(doc, "room").filter(|_| old.is_some() && status == "Checked In" && changed("room")) {
                ready(hc.tx, tenant, room).await?;
            }
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, old: Option<&Doc>) -> Result<()> {
        // A room move while in the house: the old room needs cleaning, the new one is taken.
        let Some(o) = old else { return Ok(()) };
        if s(doc, "status") != Some("Checked In") || o.get("room") == doc.get("room") {
            return Ok(());
        }
        let system = elevated(hc.ctx);
        if let Some(before) = id(o, "room") {
            hc.kernel.set_values_tx(&system, hc.tx, "Room", before, obj(json!({"occupied": false, "housekeeping": "Dirty"}))).await?;
            if let Some(property) = id(doc, "property") {
                let today = business_date(hc.tx, hc.ctx.tenant, property).await?;
                task(hc, property, before, today, "Departure").await?;
            }
        }
        if let Some(now) = id(doc, "room") {
            hc.kernel.set_values_tx(&system, hc.tx, "Room", now, obj(json!({"occupied": true}))).await?;
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, params: &Value) -> Result<Option<Value>> {
        match name {
            "check_in" => Ok(Some(check_in(hc, doc, params).await?)),
            "check_out" => Ok(Some(check_out(hc, doc).await?)),
            "cancel_stay" => Ok(Some(cancel(hc, doc, params).await?)),
            "take_deposit" => Ok(Some(take_deposit(hc, doc, params).await?)),
            _ => Ok(None),
        }
    }
}

/// A room a guest may go into: clean or inspected (inspected, where its property inspects rooms
/// before check-in), and nobody in it.
async fn ready(tx: &mut Tx, tenant: Uuid, room: Uuid) -> Result<()> {
    let row: Option<(String, String, bool, bool)> = sqlx::query_as(
        "select r.room_number, r.housekeeping, coalesce(r.occupied, false), coalesce(p.inspect_rooms, false) from e_room r \
         left join e_hotel_property p on p.tenant_id = r.tenant_id and p.id = r.property where r.tenant_id = $1 and r.id = $2 for update of r",
    )
    .bind(tenant)
    .bind(room)
    .fetch_optional(&mut **tx)
    .await?;
    let (number, housekeeping, occupied, inspected_only) = row.ok_or_else(|| Error::field("room", "is not a room here"))?;
    if occupied {
        return Err(Error::invalid(format!("room {number} has a guest in it")));
    }
    if inspected_only && housekeeping == "Clean" {
        return Err(Error::invalid(format!("room {number} is clean but not inspected: a supervisor inspects it before a guest goes in")));
    }
    if !matches!(housekeeping.as_str(), "Clean" | "Inspected") {
        return Err(Error::invalid(format!("room {number} is {}: it is not ready for a guest", housekeeping.to_lowercase())));
    }
    Ok(())
}

/// A rate plan's deposit (percent of a stay's total) and refund window (days before arrival).
pub(crate) async fn policy(tx: &mut Tx, tenant: Uuid, plan: Uuid) -> Result<(Decimal, Option<i64>)> {
    let row: Option<(Option<Decimal>, Option<i64>)> = sqlx::query_as("select deposit_percent, refund_days from e_rate_plan where tenant_id = $1 and id = $2")
        .bind(tenant)
        .bind(plan)
        .fetch_optional(&mut **tx)
        .await?;
    let (share, days) = row.unwrap_or_default();
    Ok((share.unwrap_or_default(), days))
}

/// Who pays for a stay: its customer, or its property's walk-in customer.
fn payer(stay: &Doc, prop: &Doc) -> Result<Uuid> {
    id(stay, "customer")
        .or(id(prop, "walk_in_customer"))
        .ok_or_else(|| Error::invalid("give the stay a customer, or the property a walk-in customer, to invoice the folio"))
}

/// A property's company, its currency, and where money taken at the desk goes: the property's
/// takings account, else the company's default cash, else bank account.
async fn books(tx: &mut Tx, tenant: Uuid, prop: &Doc) -> Result<(Uuid, Option<Uuid>, Option<Uuid>)> {
    let company = id(prop, "company").ok_or_else(|| Error::invalid("the property has no company"))?;
    let (currency, cash, bank): (Option<Uuid>, Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select default_currency, default_cash_account, default_bank_account from e_company where tenant_id = $1 and id = $2")
            .bind(tenant)
            .bind(company)
            .fetch_optional(&mut **tx)
            .await?
            .unwrap_or_default();
    Ok((company, currency, id(prop, "takings_account").or(cash).or(bank)))
}

/// A money parameter of an action, when given.
fn amount_param(params: &Value, name: &str) -> Result<Option<Decimal>> {
    let text = match params.get(name) {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(t)) => t.trim().to_string(),
        Some(v) => v.to_string(),
    };
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<Decimal>().map(Some).map_err(|_| Error::field(name, "is an amount"))
}

/// `take_deposit {amount?, mode, reference?}`: money paid before the stay, in the books at once: a
/// submitted payment entry from whoever pays for the stay into the property's takings account,
/// their advance until the folio is invoiced. It goes on the folio as a payment at check-in.
async fn take_deposit(hc: &mut HookCtx<'_>, doc: &Doc, params: &Value) -> Result<Value> {
    hc.ctx.require("PaymentEntry", Action::Create)?;
    let stay = id(doc, "id").ok_or_else(|| Error::internal("a stay without an id"))?;
    let (due, paid) = (d(doc, "deposit_due"), d(doc, "deposit_paid"));
    let amount = match amount_param(params, "amount")? {
        Some(a) => a,
        None if due > paid => due - paid,
        None => return Err(Error::field("amount", "nothing is due: give the amount taken")),
    };
    if amount <= Decimal::ZERO {
        return Err(Error::field("amount", "is above zero"));
    }
    let mode = params.get("mode").and_then(Value::as_str).unwrap_or("Cash").to_string();
    let reference = params.get("reference").and_then(Value::as_str).map(str::trim).filter(|r| !r.is_empty()).map(str::to_string);
    let system = elevated(hc.ctx);
    let prop = hc.kernel.get_tx(&system, hc.tx, "HotelProperty", id(doc, "property").ok_or_else(|| Error::field("property", "is required"))?, false).await?;
    let (company, currency, takings) = books(hc.tx, hc.ctx.tenant, &prop).await?;
    let takings = takings.ok_or_else(|| Error::invalid("set the property's takings account (or the company's default cash account) to take a deposit"))?;
    let customer = payer(doc, &prop)?;
    let number = s(doc, "number").unwrap_or_default().to_string();
    let pe = hc
        .kernel
        .insert_tx(
            &system,
            hc.tx,
            "PaymentEntry",
            obj(json!({"company": company.to_string(), "payment_type": "Receive", "customer": customer.to_string(), "bank_account": takings.to_string(),
                       "currency": currency.map(|c| c.to_string()), "paid_amount": money(amount), "reference_no": reference.clone().unwrap_or_else(|| number.clone()),
                       "remarks": format!("Deposit for stay {number} ({mode})")})),
        )
        .await?;
    let pid = id(&pe, "id").ok_or_else(|| Error::internal("a payment without an id"))?;
    let pe = hc.kernel.submit_tx(&system, hc.tx, "PaymentEntry", pid, None).await?;
    let mut deposits: Vec<Doc> = rows(doc, "deposits")
        .into_iter()
        .map(|mut r| {
            r.retain(|k, _| ["id", "paid_on", "amount", "mode", "reference", "payment_entry"].contains(&k.as_str()));
            r
        })
        .collect();
    deposits
        .push(obj(json!({"paid_on": pe.get("posting_date"), "amount": money(amount), "mode": mode, "reference": reference, "payment_entry": pid.to_string()})));
    hc.kernel.set_child_rows_tx(&system, hc.tx, "Stay", "deposits", stay, deposits).await?;
    hc.kernel.set_values_tx(&system, hc.tx, "Stay", stay, obj(json!({"deposit_paid": money(paid + amount)}))).await?;
    let message = if due > Decimal::ZERO {
        format!("Took a deposit of {}: {} of the {} due is paid", money_text(amount), money_text(paid + amount), money_text(due))
    } else {
        format!("Took a deposit of {}", money_text(amount))
    };
    Ok(json!({"deposit_paid": money(paid + amount), "payment_entry": created("PaymentEntry", &pe), "message": message}))
}

/// What becomes of a deposit when its stay does not come. Cancelled at least its rate plan's
/// refund days before arrival (by the property's business date; any time, when the plan names no
/// days), it is given back: a draft journal entry from the takings account to the payer, for
/// accounts to submit when the money goes out. Cancelled later, or a no-show, the hotel keeps it:
/// a draft invoice for it as a cancellation fee, for accounts to apply the deposit to.
/// `(outcome, values for the stay, words for the message)`, or nothing without a deposit.
pub(crate) async fn deposit_left(hc: &mut HookCtx<'_>, stay: &Doc, no_show: bool) -> Result<Option<(&'static str, Doc, String)>> {
    let paid = d(stay, "deposit_paid");
    if paid <= Decimal::ZERO {
        return Ok(None);
    }
    let tenant = hc.ctx.tenant;
    let system = elevated(hc.ctx);
    let prop = hc.kernel.get_tx(&system, hc.tx, "HotelProperty", id(stay, "property").ok_or_else(|| Error::field("property", "is required"))?, false).await?;
    let (company, currency, takings) = books(hc.tx, tenant, &prop).await?;
    let customer = payer(stay, &prop)?;
    let number = s(stay, "number").unwrap_or_default().to_string();
    let days = match id(stay, "rate_plan") {
        Some(plan) => policy(hc.tx, tenant, plan).await?.1,
        None => None,
    };
    let today = match date(&prop, "business_date") {
        Some(day) => day,
        None => hc.ctx.today(),
    };
    let arrival = date(stay, "arrival").unwrap_or(today);
    let refunded = !no_show && days.is_none_or(|days| (arrival - today).num_days() >= days);
    if refunded {
        let takings =
            takings.ok_or_else(|| Error::invalid("set the property's takings account (or the company's default cash account) to give a deposit back"))?;
        // The payer's receivable account, as ERP's payments find it.
        let theirs = hc.kernel.get_tx(&system, hc.tx, "Customer", customer, false).await?;
        let ours = hc.kernel.get_tx(&system, hc.tx, "Company", company, false).await?;
        let receivable = id(&theirs, "receivable_account")
            .or(id(&ours, "default_receivable_account"))
            .ok_or_else(|| Error::invalid("the company has no default receivable account"))?;
        let je = hc
            .kernel
            .insert_tx(
                &system,
                hc.tx,
                "JournalEntry",
                obj(json!({"company": company.to_string(), "remark": format!("Deposit for stay {number} given back"), "accounts": [
                    {"account": receivable.to_string(), "party_type": "Customer", "customer": customer.to_string(), "debit": money(paid), "credit": "0"},
                    {"account": takings.to_string(), "debit": "0", "credit": money(paid)}]})),
            )
            .await?;
        let values = obj(json!({"deposit_outcome": "Refund Due", "deposit_refund": je.get("id")}));
        return Ok(Some(("Refund Due", values, format!("its deposit of {} is to be given back (a draft journal entry for accounts)", money_text(paid)))));
    }
    let item = id(&prop, "cancellation_item")
        .or(id(&prop, "room_item"))
        .ok_or_else(|| Error::invalid("set the property's cancellation fee item to invoice a deposit kept"))?;
    let why = if no_show { "did not arrive" } else { "was cancelled late" };
    let inv = create_for_user(
        hc,
        "SalesInvoice",
        obj(json!({"company": company.to_string(), "customer": customer.to_string(), "currency": currency.map(|c| c.to_string()),
                   "items": [{"item": item.to_string(), "qty": "1", "price_list_rate": money(paid), "discount_percent": "0",
                              "description": format!("Deposit kept: stay {number} {why}")}],
                   "remarks": format!("Stay {number} {why}: its deposit is kept")})),
    )
    .await?;
    let values = obj(json!({"deposit_outcome": "Kept", "deposit_invoice": inv.get("id")}));
    Ok(Some(("Kept", values, format!("its deposit of {} is kept, invoiced as a cancellation fee (a draft)", money_text(paid)))))
}

/// The property's business date, which check-in and the audit go by.
pub(crate) async fn business_date(tx: &mut Tx, tenant: Uuid, property: Uuid) -> Result<NaiveDate> {
    let bd: Option<Option<NaiveDate>> = sqlx::query_scalar("select business_date from e_hotel_property where tenant_id = $1 and id = $2")
        .bind(tenant)
        .bind(property)
        .fetch_optional(&mut **tx)
        .await?;
    bd.flatten().ok_or_else(|| Error::invalid("set the property's business date first (Set the Business Date)"))
}

/// `check_in {room?}`: the guest arrives. Their room is the one given or the one held, and must be
/// ready; the stay's folio opens.
async fn check_in(hc: &mut HookCtx<'_>, doc: &Doc, params: &Value) -> Result<Value> {
    let tenant = hc.ctx.tenant;
    let stay = id(doc, "id").ok_or_else(|| Error::internal("a stay without an id"))?;
    let property = id(doc, "property").ok_or_else(|| Error::field("property", "is required"))?;
    let (arrival, departure, _) = nights_of(doc)?;
    let today = business_date(hc.tx, tenant, property).await?;
    if today < arrival {
        return Err(Error::invalid(format!("this stay arrives on {arrival}; the hotel's business date is {today}")));
    }
    if today >= departure {
        return Err(Error::invalid(format!("this stay ended on {departure}")));
    }
    let room = params
        .get("room")
        .and_then(Value::as_str)
        .and_then(|r| r.parse().ok())
        .or(id(doc, "room"))
        .ok_or_else(|| Error::field("room", "choose the guest's room"))?;
    ready(hc.tx, tenant, room).await?;
    let mut held = doc.clone();
    set(&mut held, "room", json!(room.to_string()));
    if id(doc, "room") != Some(room) {
        let of: Option<(Option<Uuid>, Option<Uuid>)> = sqlx::query_as("select property, room_type from e_room where tenant_id = $1 and id = $2")
            .bind(tenant)
            .bind(room)
            .fetch_optional(&mut **hc.tx)
            .await?;
        if of != Some((Some(property), id(doc, "room_type"))) {
            return Err(Error::field("room", "is not a room of this stay's type and property"));
        }
        hold(hc.tx, tenant, &held).await?;
    }
    let system = elevated(hc.ctx);
    // Deposits taken before are payments on the folio, already in the books.
    let paid: Vec<Value> = rows(doc, "deposits")
        .iter()
        .map(|p| {
            json!({"paid_on": s(p, "paid_on"), "amount": money(d(p, "amount")), "mode": format!("Deposit, {}", s(p, "mode").unwrap_or("Cash")),
                   "reference": s(p, "reference"), "payment_entry": s(p, "payment_entry")})
        })
        .collect();
    let folio = hc
        .kernel
        .insert_tx(
            &system,
            hc.tx,
            "Folio",
            obj(json!({"stay": stay.to_string(), "property": property.to_string(), "guest_name": s(doc, "guest_name"), "guest_email": s(doc, "guest_email"),
                       "status": "Open", "payments": paid})),
        )
        .await?;
    let folio_id = folio.get("id").cloned().unwrap_or(Value::Null);
    hc.kernel
        .set_values_tx(
            &system,
            hc.tx,
            "Stay",
            stay,
            obj(json!({"status": "Checked In", "room": room.to_string(), "folio": folio_id, "checked_in_at": Utc::now().to_rfc3339()})),
        )
        .await?;
    hc.kernel.set_values_tx(&system, hc.tx, "Room", room, obj(json!({"occupied": true}))).await?;
    let number: Option<String> =
        sqlx::query_scalar("select room_number from e_room where tenant_id = $1 and id = $2").bind(tenant).bind(room).fetch_optional(&mut **hc.tx).await?;
    let message = format!("Checked in to room {}; folio {} is open", number.unwrap_or_default(), s(&folio, "number").unwrap_or_default());
    Ok(json!({"status": "Checked In", "folio": created("Folio", &folio), "message": message}))
}

/// `check_out`: the guest leaves. The folio closes and its charges are invoiced (a draft Sales
/// Invoice) to the stay's customer or the property's walk-in customer; what they paid at the desk
/// goes to ERP too, each payment a draft Payment Entry from that customer into the property's
/// takings account, for accounts to apply to the invoice once it is submitted (until then the
/// books did not know of it). The room needs cleaning.
async fn check_out(hc: &mut HookCtx<'_>, doc: &Doc) -> Result<Value> {
    let tenant = hc.ctx.tenant;
    let stay = id(doc, "id").ok_or_else(|| Error::internal("a stay without an id"))?;
    let property = id(doc, "property").ok_or_else(|| Error::field("property", "is required"))?;
    let folio_id = id(doc, "folio").ok_or_else(|| Error::invalid("this stay has no folio"))?;
    let system = elevated(hc.ctx);
    let folio = hc.kernel.get_tx(&system, hc.tx, "Folio", folio_id, true).await?;
    let prop = hc.kernel.get_tx(&system, hc.tx, "HotelProperty", property, false).await?;
    let charges = rows(&folio, "charges");
    let mut payments = rows(&folio, "payments");
    let customer = || {
        id(doc, "customer")
            .or(id(&prop, "walk_in_customer"))
            .ok_or_else(|| Error::invalid("give the stay a customer, or the property a walk-in customer, to invoice the folio"))
    };
    let (company, currency, takings) = books(hc.tx, tenant, &prop).await?;
    let invoice = if charges.is_empty() {
        None
    } else {
        let customer = customer()?;
        let room_item = id(&prop, "room_item");
        let mut items = Vec::new();
        for c in &charges {
            let item = id(c, "item")
                .or(room_item)
                .ok_or_else(|| Error::invalid(format!("the charge \"{}\" has no item to invoice it as", s(c, "description").unwrap_or("?"))))?;
            items.push(json!({"item": item.to_string(), "qty": d(c, "qty").to_string(), "price_list_rate": money(d(c, "rate")), "discount_percent": "0",
                              "description": format!("{} ({})", s(c, "description").unwrap_or("Charge"), s(c, "charge_date").unwrap_or(""))}));
        }
        let number = s(doc, "number").unwrap_or_default();
        let inv = create_for_user(
            hc,
            "SalesInvoice",
            obj(json!({"company": company.to_string(), "customer": customer.to_string(), "currency": currency.map(|c| c.to_string()), "items": items,
                       "remarks": format!("Stay {number}, folio {}", s(&folio, "number").unwrap_or_default())})),
        )
        .await?;
        Some(inv)
    };
    let mut handed = Decimal::ZERO;
    let advanced: Decimal = payments.iter().filter(|p| id(p, "payment_entry").is_some()).map(|p| d(p, "amount")).sum();
    if payments.iter().any(|p| id(p, "payment_entry").is_none()) {
        let customer = customer()?;
        let takings =
            takings.ok_or_else(|| Error::invalid("set the property's takings account (or the company's default cash account) to hand its payments to ERP"))?;
        for p in payments.iter_mut().filter(|p| id(p, "payment_entry").is_none()) {
            let reference = s(p, "reference").or(s(&folio, "number")).unwrap_or_default().to_string();
            let remarks = format!(
                "Paid at the front desk{} for stay {}, folio {}",
                s(p, "mode").map(|m| format!(" ({m})")).unwrap_or_default(),
                s(doc, "number").unwrap_or_default(),
                s(&folio, "number").unwrap_or_default()
            );
            let pe = hc
                .kernel
                .insert_tx(
                    &system,
                    hc.tx,
                    "PaymentEntry",
                    obj(json!({"company": company.to_string(), "posting_date": s(p, "paid_on"), "payment_type": "Receive", "customer": customer.to_string(),
                               "bank_account": takings.to_string(), "currency": currency.map(|c| c.to_string()), "paid_amount": money(d(p, "amount")),
                               "reference_no": reference, "reference_date": s(p, "paid_on"), "remarks": remarks})),
                )
                .await?;
            handed += d(p, "amount");
            set(p, "payment_entry", pe.get("id").cloned().unwrap_or(Value::Null));
        }
        let patch = obj(json!({"payments": payments.iter().cloned().map(Value::Object).collect::<Vec<_>>()}));
        hc.kernel.update_tx(&system, hc.tx, "Folio", folio_id, patch, None).await?;
    }
    hc.kernel.set_values_tx(&system, hc.tx, "Folio", folio_id, obj(json!({"status": "Closed"}))).await?;
    let mut values = obj(json!({"status": "Checked Out", "checked_out_at": Utc::now().to_rfc3339()}));
    if let Some(inv) = &invoice {
        values.insert("sales_invoice".into(), inv.get("id").cloned().unwrap_or(Value::Null));
    }
    hc.kernel.set_values_tx(&system, hc.tx, "Stay", stay, values).await?;
    if let Some(room) = id(doc, "room") {
        hc.kernel.set_values_tx(&system, hc.tx, "Room", room, obj(json!({"occupied": false, "housekeeping": "Dirty"}))).await?;
        let today = business_date(hc.tx, tenant, property).await?;
        task(hc, property, room, today, "Departure").await?;
    }
    let mut message = match &invoice {
        Some(inv) => format!("Checked out: the folio is invoiced as a draft, {}", money_text(d(inv, "grand_total"))),
        None => "Checked out: the folio had no charges".to_string(),
    };
    if !handed.is_zero() {
        message.push_str(&format!("; {} paid at the desk waits in ERP as draft payments to apply to it", money_text(handed)));
    }
    if !advanced.is_zero() {
        message.push_str(&format!("; the deposit of {} is the guest's advance in ERP, to apply to it", money_text(advanced)));
    }
    Ok(json!({"status": "Checked Out", "sales_invoice": invoice.as_ref().map(|i| created("SalesInvoice", i)), "message": message}))
}

/// `cancel_stay {reason}`: a reservation that will not come; its nights are free again, and its
/// deposit given back or kept by its rate plan's policy (`deposit_left`).
async fn cancel(hc: &mut HookCtx<'_>, doc: &Doc, params: &Value) -> Result<Value> {
    let stay = id(doc, "id").ok_or_else(|| Error::internal("a stay without an id"))?;
    let reason = params.get("reason").and_then(Value::as_str).map(str::trim).filter(|r| !r.is_empty()).ok_or_else(|| Error::field("reason", "say why"))?;
    let mut values = obj(json!({"status": "Cancelled", "cancellation_reason": reason}));
    let mut message = "The reservation is cancelled; its nights are free again".to_string();
    let mut outcome = Value::Null;
    if let Some((what, more, words)) = deposit_left(hc, doc, false).await? {
        values.extend(more);
        message.push_str(&format!("; {words}"));
        outcome = json!(what);
    }
    hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "Stay", stay, values).await?;
    Ok(json!({"status": "Cancelled", "deposit_outcome": outcome, "message": message}))
}

/// A room's type is one of its property's.
pub struct RoomHook;

#[async_trait]
impl DocHook for RoomHook {
    fn name(&self) -> &'static str {
        "hospitality.room"
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if let (Some(room_type), Some(property)) = (id(doc, "room_type"), id(doc, "property"))
            && property_of(hc.tx, hc.ctx.tenant, "room_type", room_type).await? != Some(property)
        {
            return Err(Error::field("room_type", "belongs to another property"));
        }
        Ok(())
    }
}
