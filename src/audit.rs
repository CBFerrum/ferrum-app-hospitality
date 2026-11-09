//! The night audit: what closes a property's business date. Each stay in the house is charged that
//! night at its rate, once; reservations whose arrival day passed without the guest become no-shows;
//! the night's rooms and revenue are kept (occupancy, ADR, RevPAR); and the next day opens. It runs
//! once a night, and the property's row stays locked while it does.

use crate::housekeeping::task;
use crate::stay::{business_date, deposit_left, night_rate};
use crate::{date, obj};
use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use ferrum_domain::util::{elevated, id, money, round, rows};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use uuid::Uuid;

pub struct PropertyHook;

#[async_trait]
impl DocHook for PropertyHook {
    fn name(&self) -> &'static str {
        "hospitality.property"
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, params: &Value) -> Result<Option<Value>> {
        match name {
            "open_business_date" => {
                let property = id(doc, "id").ok_or_else(|| Error::internal("a property without an id"))?;
                if date(doc, "business_date").is_some() {
                    return Err(Error::invalid("the business date is set: the night audit moves it on"));
                }
                let day = params
                    .get("date")
                    .and_then(Value::as_str)
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .ok_or_else(|| Error::field("date", "the hotel's first business date, YYYY-MM-DD"))?;
                hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "HotelProperty", property, obj(json!({"business_date": day.to_string()}))).await?;
                Ok(Some(json!({"business_date": day, "message": format!("The business date is {day}")})))
            }
            "night_audit" => Ok(Some(run(hc, doc).await?)),
            _ => Ok(None),
        }
    }
}

async fn run(hc: &mut HookCtx<'_>, doc: &Doc) -> Result<Value> {
    let tenant = hc.ctx.tenant;
    let property = id(doc, "id").ok_or_else(|| Error::internal("a property without an id"))?;
    let night = business_date(hc.tx, tenant, property).await?;
    let done: bool = sqlx::query_scalar("select exists(select 1 from e_night_audit where tenant_id = $1 and property = $2 and business_date = $3)")
        .bind(tenant)
        .bind(property)
        .bind(night)
        .fetch_one(&mut **hc.tx)
        .await?;
    if done {
        return Err(Error::invalid(format!("the night of {night} was audited already")));
    }
    let system = elevated(hc.ctx);
    let room_item = id(doc, "room_item");
    // Everyone in the house tonight.
    let in_house: Vec<(Uuid, Option<Uuid>, Uuid, Option<Uuid>, Option<String>, Option<Uuid>, NaiveDate)> = sqlx::query_as(
        "select s.id, s.folio, s.room_type, s.rate_plan, r.room_number, s.room, s.departure from e_stay s left join e_room r on r.tenant_id = s.tenant_id and r.id = s.room \
         where s.tenant_id = $1 and s.property = $2 and s.status = 'Checked In' and s.arrival <= $3 and s.departure > $3 order by r.room_number, s.id",
    )
    .bind(tenant)
    .bind(property)
    .bind(night)
    .fetch_all(&mut **hc.tx)
    .await?;
    let mut posted = 0i64;
    let mut revenue = Decimal::ZERO;
    for (_stay, folio, room_type, plan, room, _, _) in &in_house {
        let Some(folio) = folio else { continue };
        let rate = night_rate(hc.tx, tenant, *room_type, *plan, night).await?;
        let current = hc.kernel.get_tx(&system, hc.tx, "Folio", *folio, true).await?;
        // Only the audit posts room nights, once a night (the check above, under the property's lock).
        let mut charges = rows(&current, "charges");
        charges.push(obj(
            json!({"charge_date": night.to_string(), "kind": "Room", "description": format!("Room {}, night of {night}", room.as_deref().unwrap_or("?")),
                                "item": room_item.map(|i| i.to_string()), "qty": "1", "rate": money(rate)}),
        ));
        hc.kernel.update_tx(&system, hc.tx, "Folio", *folio, obj(json!({"charges": charges})), None).await?;
        posted += 1;
        revenue += rate;
    }
    // Those whose arrival day has passed without them.
    let missed: Vec<Uuid> = sqlx::query_scalar("select id from e_stay where tenant_id = $1 and property = $2 and status = 'Reserved' and arrival <= $3")
        .bind(tenant)
        .bind(property)
        .bind(night)
        .fetch_all(&mut **hc.tx)
        .await?;
    // A no-show's deposit is the hotel's to keep.
    for stay in &missed {
        let doc = hc.kernel.get_tx(&system, hc.tx, "Stay", *stay, true).await?;
        let mut values = obj(json!({"status": "No Show"}));
        if let Some((_, more, _)) = deposit_left(hc, &doc, true).await? {
            values.extend(more);
        }
        hc.kernel.set_values_tx(&system, hc.tx, "Stay", *stay, values).await?;
    }
    let no_shows = missed.len() as i64;
    let available: i64 = sqlx::query_scalar("select count(*) from e_room where tenant_id = $1 and property = $2 and housekeeping <> 'Out of Order'")
        .bind(tenant)
        .bind(property)
        .fetch_one(&mut **hc.tx)
        .await?;
    let occupied = in_house.len() as i64;
    let ratio = |a: Decimal, b: i64| if b > 0 { round(a / Decimal::from(b), 2) } else { Decimal::ZERO };
    let audit = hc
        .kernel
        .insert_tx(
            &system,
            hc.tx,
            "NightAudit",
            obj(json!({
                "property": property.to_string(),
                "business_date": night.to_string(),
                "rooms_available": available,
                "rooms_occupied": occupied,
                "occupancy_percent": money(ratio(Decimal::from(occupied * 100), available)),
                "room_revenue": money(revenue),
                "average_daily_rate": money(ratio(revenue, occupied)),
                "revpar": money(ratio(revenue, available)),
                "charges_posted": posted,
                "no_shows": no_shows,
                "run_by": hc.ctx.email,
            })),
        )
        .await?;
    let next = night + Duration::days(1);
    hc.kernel.set_values_tx(&system, hc.tx, "HotelProperty", property, obj(json!({"business_date": next.to_string()}))).await?;
    // Tomorrow's housekeeping: each room a guest stays on in is serviced.
    for (_, _, _, _, _, room, departure) in &in_house {
        if let Some(room) = room.filter(|_| *departure > next) {
            task(hc, property, room, next, "Stayover").await?;
        }
    }
    let message = format!(
        "The night of {night} is closed: {occupied} of {available} rooms occupied, {posted} room charge{} posted, {no_shows} no-show{}; the business date is {next}",
        if posted == 1 { "" } else { "s" },
        if no_shows == 1 { "" } else { "s" }
    );
    Ok(json!({"night": night, "next": next, "charges_posted": posted, "no_shows": no_shows, "rooms_occupied": occupied, "rooms_available": available,
              "room_revenue": money(revenue), "audit": audit.get("id"), "message": message}))
}
