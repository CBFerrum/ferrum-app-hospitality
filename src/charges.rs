//! Room charges: what a guest had at an outlet (the restaurant, the spa, the minibar) put on their
//! room. It is checked against the stay in that room (a guest is in it, its folio is open, and the
//! name given is that guest's), posted to the folio as a service charge at the property's business
//! date, and invoiced with the room at check-out. An outlet's cashier puts it there without seeing
//! the stay or the folio; once posted it does not change (the front desk adjusts the folio).

use crate::obj;
use crate::stay::business_date;
use async_trait::async_trait;
use ferrum_domain::util::{d, elevated, id, money, rows, s};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result, Tx};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

/// The stay in `room` now, its open folio and its guest's name.
async fn in_room(tx: &mut Tx, tenant: Uuid, room: Uuid) -> Result<Option<(Uuid, Option<Uuid>, String, Option<String>)>> {
    Ok(sqlx::query_as(
        "select s.id, s.folio, s.guest_name, f.status from e_stay s left join e_folio f on f.tenant_id = s.tenant_id and f.id = s.folio \
         where s.tenant_id = $1 and s.room = $2 and s.status = 'Checked In' order by s.arrival desc limit 1",
    )
    .bind(tenant)
    .bind(room)
    .fetch_optional(&mut **tx)
    .await?)
}

/// The words of a name, to compare: "Ravi Rao" and "rao" share "rao".
fn words(name: &str) -> Vec<String> {
    name.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_lowercase).collect()
}

/// Whether the name given is the guest's: every word of it is one of theirs.
pub(crate) fn same_guest(given: &str, guest: &str) -> bool {
    let theirs = words(guest);
    let given = words(given);
    !given.is_empty() && given.iter().all(|w| theirs.contains(w))
}

pub struct RoomChargeHook;

#[async_trait]
impl DocHook for RoomChargeHook {
    fn name(&self) -> &'static str {
        "hospitality.room_charge"
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if old.is_some() {
            return if hc.ctx.is_system { Ok(()) } else { Err(Error::invalid("a posted charge does not change: the front desk adjusts the folio")) };
        }
        if d(doc, "amount") <= Decimal::ZERO {
            return Err(Error::field("amount", "is above zero"));
        }
        let tenant = hc.ctx.tenant;
        let room = id(doc, "room").ok_or_else(|| Error::field("room", "is required"))?;
        let of: Option<(Option<Uuid>, String)> = sqlx::query_as("select property, room_number from e_room where tenant_id = $1 and id = $2")
            .bind(tenant)
            .bind(room)
            .fetch_optional(&mut **hc.tx)
            .await?;
        let (property, number) = of.ok_or_else(|| Error::field("room", "is not a room here"))?;
        if property != id(doc, "property") {
            return Err(Error::field("room", "is another property's"));
        }
        let (_, _, guest, folio) = in_room(hc.tx, tenant, room).await?.ok_or_else(|| Error::field("room", format!("room {number} has no guest checked in")))?;
        if folio.as_deref() != Some("Open") {
            return Err(Error::field("room", format!("room {number}'s folio is not open")));
        }
        if !same_guest(s(doc, "guest_name").unwrap_or_default(), &guest) {
            return Err(Error::field("guest_name", format!("is not the name of the guest in room {number}")));
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, old: Option<&Doc>) -> Result<()> {
        if old.is_some() {
            return Ok(());
        }
        let tenant = hc.ctx.tenant;
        let charge = id(doc, "id").ok_or_else(|| Error::internal("a charge without an id"))?;
        let room = id(doc, "room").ok_or_else(|| Error::field("room", "is required"))?;
        let property = id(doc, "property").ok_or_else(|| Error::field("property", "is required"))?;
        let (stay, folio, _, _) = in_room(hc.tx, tenant, room).await?.ok_or_else(|| Error::field("room", "has no guest checked in"))?;
        let folio = folio.ok_or_else(|| Error::invalid("the stay has no folio"))?;
        let system = elevated(hc.ctx);
        let day = business_date(hc.tx, tenant, property).await?;
        let current = hc.kernel.get_tx(&system, hc.tx, "Folio", folio, true).await?;
        let mut charges = rows(&current, "charges");
        charges.push(obj(json!({
            "charge_date": day.to_string(), "kind": "Service",
            "description": format!("{}: {}", s(doc, "outlet").unwrap_or("Outlet"), s(doc, "description").unwrap_or_default()),
            "item": s(doc, "item"), "qty": "1", "rate": money(d(doc, "amount")), "room_charge": charge.to_string(),
        })));
        hc.kernel.update_tx(&system, hc.tx, "Folio", folio, obj(json!({"charges": charges})), None).await?;
        hc.kernel.set_values_tx(&system, hc.tx, "RoomCharge", charge, obj(json!({"stay": stay.to_string(), "folio": folio.to_string()}))).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::same_guest;

    #[test]
    fn a_guest_is_known_by_their_name_or_part_of_it() {
        assert!(same_guest("Rao", "Ravi Rao"));
        assert!(same_guest("ravi  RAO", "Ravi Rao"));
        assert!(!same_guest("Ravi Rai", "Ravi Rao"));
        assert!(!same_guest("  ", "Ravi Rao"));
    }
}
