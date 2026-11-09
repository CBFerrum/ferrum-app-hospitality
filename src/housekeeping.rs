//! Housekeeping: a room to clean on a business date. Check-out (and a guest moving rooms) leaves a
//! Departure task for the room it frees; the night audit a Stayover task for each room a guest stays
//! on in. Cleaned, the room is Clean; a departure inspected by a supervisor other than whoever
//! cleaned it is Inspected, which is what a property that inspects rooms checks guests into.

use crate::obj;
use async_trait::async_trait;
use chrono::NaiveDate;
use ferrum_domain::util::{elevated, id, s};
use ferrum_kernel::{Ctx, Doc, DocHook, Error, HookCtx, Result};
use serde_json::{Value, json};
use uuid::Uuid;

/// A task for `room` on `day`, unless it has one of that kind already.
pub(crate) async fn task(hc: &mut HookCtx<'_>, property: Uuid, room: Uuid, day: NaiveDate, kind: &str) -> Result<()> {
    let tenant = hc.ctx.tenant;
    let has: bool = sqlx::query_scalar("select exists(select 1 from e_housekeeping_task where tenant_id = $1 and room = $2 and task_date = $3 and kind = $4)")
        .bind(tenant)
        .bind(room)
        .bind(day)
        .bind(kind)
        .fetch_one(&mut **hc.tx)
        .await?;
    if has {
        return Ok(());
    }
    let number: Option<String> =
        sqlx::query_scalar("select room_number from e_room where tenant_id = $1 and id = $2").bind(tenant).bind(room).fetch_optional(&mut **hc.tx).await?;
    let input = json!({"property": property.to_string(), "room": room.to_string(), "room_number": number, "task_date": day.to_string(), "kind": kind, "status": "To Do"});
    hc.kernel.insert_tx(&elevated(hc.ctx), hc.tx, "HousekeepingTask", obj(input)).await?;
    Ok(())
}

/// Who may inspect a cleaned room: a supervisor or the hotel's manager.
fn supervises(ctx: &Ctx) -> bool {
    ctx.is_admin || ctx.roles.iter().any(|r| r == "Housekeeping Supervisor" || r == "Hotel Manager")
}

pub struct TaskHook;

#[async_trait]
impl DocHook for TaskHook {
    fn name(&self) -> &'static str {
        "hospitality.housekeeping_task"
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        let task = id(doc, "id").ok_or_else(|| Error::internal("a task without an id"))?;
        let room = id(doc, "room").ok_or_else(|| Error::internal("a task without its room"))?;
        let number = s(doc, "room_number").unwrap_or_default().to_string();
        let system = elevated(hc.ctx);
        match name {
            "cleaned" => {
                hc.kernel.set_values_tx(&system, hc.tx, "HousekeepingTask", task, obj(json!({"status": "Cleaned", "cleaned_by": hc.ctx.email}))).await?;
                // A room someone moved into since keeps what its guest's presence says.
                hc.kernel.set_values_tx(&system, hc.tx, "Room", room, obj(json!({"housekeeping": "Clean"}))).await?;
                Ok(Some(json!({"status": "Cleaned", "message": format!("Room {number} is clean")})))
            }
            "inspect" => {
                if !supervises(hc.ctx) {
                    return Err(Error::Forbidden("a housekeeping supervisor inspects a room".into()));
                }
                if s(doc, "cleaned_by").is_some_and(|by| by.eq_ignore_ascii_case(&hc.ctx.email)) {
                    return Err(Error::Forbidden("someone other than whoever cleaned a room inspects it".into()));
                }
                let now: Option<String> = sqlx::query_scalar("select housekeeping from e_room where tenant_id = $1 and id = $2 for update")
                    .bind(hc.ctx.tenant)
                    .bind(room)
                    .fetch_optional(&mut **hc.tx)
                    .await?;
                if now.as_deref() != Some("Clean") {
                    return Err(Error::invalid(format!("room {number} is {} now: clean it again first", now.unwrap_or_default().to_lowercase())));
                }
                hc.kernel.set_values_tx(&system, hc.tx, "HousekeepingTask", task, obj(json!({"status": "Inspected", "inspected_by": hc.ctx.email}))).await?;
                hc.kernel.set_values_tx(&system, hc.tx, "Room", room, obj(json!({"housekeeping": "Inspected"}))).await?;
                Ok(Some(json!({"status": "Inspected", "message": format!("Room {number} is inspected: ready for a guest")})))
            }
            _ => Ok(None),
        }
    }
}
