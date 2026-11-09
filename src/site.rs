//! The public booking page's endpoints, `/api/site/{workspace}/hospitality/...`: a guest finds a
//! room type free for their dates at a property that books online, priced by its online rate plan,
//! books it, and opens their reservation with the link they were given (only its SHA-256 is kept on
//! the stay), where they may cancel it while it is reserved, by the rate plan's deposit policy.
//! Guests never set prices, rooms or statuses: every request is checked here and acts with the
//! system context through the same hooks as the front desk (no room type is sold past its rooms).
//! A property that does not book online, an unknown link and an expired one all answer 404.

use crate::obj;
use crate::stay::{ACTIVE, night_rate, policy};
use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, NaiveDate};
use ferrum_domain::util::{id, money, round, s};
use ferrum_kernel::{Ctx, Doc, Error, FieldError, Kernel, Result, Site, SiteRequest, SiteResponse, Tx, with_retry};
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration as StdDuration;
use uuid::Uuid;

const MAX_BODY: usize = 16 * 1024;
/// Bookings and cancellations per address and workspace, per window.
const WRITES_PER_WINDOW: u32 = 20;
const WINDOW: StdDuration = StdDuration::from_secs(600);
/// The longest stay booked online; longer ones are the front desk's.
const MAX_ONLINE_NIGHTS: i64 = 30;
/// How far ahead the page books.
const MAX_AHEAD_DAYS: i64 = 500;
/// How long after departure a guest's link keeps working.
const LINK_DAYS: i32 = 30;
const MAX_PARTY: i64 = 20;

pub struct HotelSite;

#[async_trait]
impl Site for HotelSite {
    fn max_body(&self) -> usize {
        MAX_BODY
    }

    async fn handle(&self, k: &Kernel, req: SiteRequest) -> Result<SiteResponse> {
        let schema = match &req.user {
            Some(u) => u.schema.clone(),
            None => k.registry.schema(req.tenant).await?,
        };
        let mut ctx = Ctx::system(req.tenant, schema);
        ctx.request_id = format!("hospitality-{}", Uuid::now_v7());
        let path: Vec<&str> = req.path.split('/').filter(|p| !p.is_empty()).collect();
        let (status, body) = match (req.method.as_str(), path.as_slice()) {
            ("GET", ["properties"]) => (200, properties(k, &ctx).await?),
            ("GET", ["availability"]) => (200, availability(k, &ctx, &req).await?),
            ("POST", ["book"]) => {
                throttle(k, req.tenant, &req.client_ip).await?;
                book(k, &ctx, &body_of(&req)?).await?
            }
            ("GET", ["manage", tok]) => (200, manage(k, &ctx, tok).await?),
            ("POST", ["manage", tok, "cancel"]) => {
                throttle(k, req.tenant, &req.client_ip).await?;
                (200, cancel(k, &ctx, tok, &body_of(&req)?).await?)
            }
            _ => return Err(Error::NotFound("page".into())),
        };
        let mut resp = SiteResponse::json(status, &body);
        resp.headers.push(("cache-control".into(), "no-store".into()));
        Ok(resp)
    }
}

/// A fixed-window limit per caller address and workspace on the requests that write.
async fn throttle(k: &Kernel, tenant: Uuid, ip: &str) -> Result<()> {
    k.rate_limit(&format!("hospitality|{tenant}|{ip}"), WINDOW, WRITES_PER_WINDOW, "too many booking requests from this address; try again in a few minutes")
        .await
}

fn body_of(req: &SiteRequest) -> Result<Map<String, Value>> {
    if req.body.iter().all(u8::is_ascii_whitespace) {
        return Ok(Map::new());
    }
    match serde_json::from_slice::<Value>(&req.body) {
        Ok(Value::Object(m)) => Ok(m),
        _ => Err(Error::invalid("the request body is a JSON object")),
    }
}

fn query<'a>(req: &'a SiteRequest, key: &str) -> Option<&'a str> {
    req.query.get(key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

/// A text field of a guest's request, at most `max` characters.
fn text(body: &Map<String, Value>, key: &str, max: usize, errors: &mut Vec<FieldError>) -> Option<String> {
    match body.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::String(v)) => {
            let v = v.trim();
            if v.chars().count() > max {
                errors.push(FieldError::new(key, format!("is at most {max} characters")));
            }
            (!v.is_empty()).then(|| v.to_string())
        }
        Some(_) => {
            errors.push(FieldError::new(key, "is text"));
            None
        }
    }
}

fn whole(v: Option<&Value>) -> Option<i64> {
    v.and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|x| x.trim().parse().ok())))
}

/// A new guest link and the hash the stay keeps.
fn new_token() -> (String, String) {
    let token = URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>());
    let hash = hex::encode(Sha256::digest(token.as_bytes()));
    (token, hash)
}

/// The stay a guest link opens, while it works (until a month after departure).
async fn by_token(tx: &mut Tx, tenant: Uuid, token: &str) -> Result<Uuid> {
    let shaped = token.len() == 43 && token.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_');
    let found: Option<Uuid> = if shaped {
        sqlx::query_scalar("select id from e_stay where tenant_id = $1 and guest_token_hash = $2 and departure + $3 > current_date")
            .bind(tenant)
            .bind(hex::encode(Sha256::digest(token.as_bytes())))
            .bind(LINK_DAYS)
            .fetch_optional(&mut **tx)
            .await?
    } else {
        None
    };
    found.ok_or_else(|| Error::NotFound("reservation".into()))
}

struct Property {
    id: Uuid,
    name: String,
    plan: Option<Uuid>,
    /// The day the front office is in: nothing is booked before it.
    today: NaiveDate,
}

/// A property that books online, by its code; 404 otherwise.
async fn open_property(tx: &mut Tx, ctx: &Ctx, code: Option<&str>) -> Result<Property> {
    let code = code.ok_or_else(|| Error::NotFound("property".into()))?;
    let row: Option<(Uuid, String, Option<Uuid>, Option<NaiveDate>)> = sqlx::query_as(
        "select id, property_name, online_rate_plan, business_date from e_hotel_property where tenant_id = $1 and code = $2 and coalesce(book_online, false)",
    )
    .bind(ctx.tenant)
    .bind(code)
    .fetch_optional(&mut **tx)
    .await?;
    let (id, name, plan, day) = row.ok_or_else(|| Error::NotFound("property".into()))?;
    Ok(Property { id, name, plan, today: day.unwrap_or_else(|| ctx.today()) })
}

/// The nights asked for: from the property's day on, at most a year and a half ahead, and at most
/// a month long.
fn nights(arrival: Option<&str>, departure: Option<&str>, today: NaiveDate) -> Result<(NaiveDate, NaiveDate, i64)> {
    let day = |v: Option<&str>, f: &str| {
        v.and_then(|x| NaiveDate::parse_from_str(x, "%Y-%m-%d").ok()).ok_or_else(|| Error::Validation(vec![FieldError::new(f, "is a date, YYYY-MM-DD")]))
    };
    let (arrival, departure) = (day(arrival, "arrival")?, day(departure, "departure")?);
    let bad = |f: &str, m: String| Err(Error::Validation(vec![FieldError::new(f, m)]));
    if arrival < today {
        return bad("arrival", format!("is {today} or later"));
    }
    if (arrival - today).num_days() > MAX_AHEAD_DAYS {
        return bad("arrival", format!("is at most {MAX_AHEAD_DAYS} days ahead"));
    }
    let n = (departure - arrival).num_days();
    if n < 1 {
        return bad("departure", "is after the arrival".into());
    }
    if n > MAX_ONLINE_NIGHTS {
        return bad("departure", format!("is at most {MAX_ONLINE_NIGHTS} nights after the arrival: ask the hotel for a longer stay"));
    }
    Ok((arrival, departure, n))
}

/// Adults and children, as asked.
fn party(adults: Option<i64>, children: Option<i64>) -> Result<(i64, i64)> {
    let (a, c) = (adults.unwrap_or(1), children.unwrap_or(0));
    if !(1..=MAX_PARTY).contains(&a) {
        return Err(Error::Validation(vec![FieldError::new("adults", format!("is from 1 to {MAX_PARTY}"))]));
    }
    if !(0..=MAX_PARTY).contains(&c) {
        return Err(Error::Validation(vec![FieldError::new("children", format!("is from 0 to {MAX_PARTY}"))]));
    }
    Ok((a, c))
}

/// The rooms of a type free on every one of the nights.
async fn free_rooms(tx: &mut Tx, tenant: Uuid, room_type: Uuid, arrival: NaiveDate, departure: NaiveDate) -> Result<i64> {
    let rooms: i64 = sqlx::query_scalar("select count(*) from e_room where tenant_id = $1 and room_type = $2 and housekeeping <> 'Out of Order'")
        .bind(tenant)
        .bind(room_type)
        .fetch_one(&mut **tx)
        .await?;
    let busiest: Option<i64> = sqlx::query_scalar(
        "select max((select count(*) from e_stay s where s.tenant_id = $1 and s.room_type = $2 and s.status = any($5) \
                     and s.arrival <= n::date and s.departure > n::date)) from generate_series($3::date, $4::date - 1, interval '1 day') n",
    )
    .bind(tenant)
    .bind(room_type)
    .bind(arrival)
    .bind(departure)
    .bind(ACTIVE)
    .fetch_one(&mut **tx)
    .await?;
    Ok(rooms - busiest.unwrap_or(0))
}

/// The stay's price at the plan and what its deposit is.
async fn priced(tx: &mut Tx, tenant: Uuid, room_type: Uuid, plan: Option<Uuid>, arrival: NaiveDate, n: i64) -> Result<(Decimal, Decimal)> {
    let mut total = Decimal::ZERO;
    for i in 0..n {
        total += night_rate(tx, tenant, room_type, plan, arrival + Duration::days(i)).await?;
    }
    let share = match plan {
        Some(p) => policy(tx, tenant, p).await?.0,
        None => Decimal::ZERO,
    };
    Ok((total, round(total * share / Decimal::ONE_HUNDRED, 2)))
}

/// `GET /properties`: the properties that book online.
async fn properties(k: &Kernel, ctx: &Ctx) -> Result<Value> {
    let mut tx = k.db.tenant_tx(ctx.tenant).await?;
    let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "select code, property_name, address, check_in_time, check_out_time from e_hotel_property where tenant_id = $1 and coalesce(book_online, false) \
         order by property_name",
    )
    .bind(ctx.tenant)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    if rows.is_empty() {
        return Err(Error::NotFound("page".into()));
    }
    let list: Vec<Value> = rows
        .into_iter()
        .map(|(code, name, address, check_in, check_out)| json!({"code": code, "name": name, "address": address, "check_in_time": check_in, "check_out_time": check_out}))
        .collect();
    Ok(json!({"properties": list}))
}

/// `GET /availability?property=&arrival=&departure=&adults=&children=`: the room types that hold
/// the party and have a room free every night, cheapest first, with the stay's price and deposit.
async fn availability(k: &Kernel, ctx: &Ctx, req: &SiteRequest) -> Result<Value> {
    let mut tx = k.db.tenant_tx(ctx.tenant).await?;
    let p = open_property(&mut tx, ctx, query(req, "property")).await?;
    let (arrival, departure, n) = nights(query(req, "arrival"), query(req, "departure"), p.today)?;
    let (adults, children) = party(query(req, "adults").and_then(|v| v.parse().ok()), query(req, "children").and_then(|v| v.parse().ok()))?;
    let types: Vec<(Uuid, String, Option<i64>, Option<i64>)> = sqlx::query_as(
        "select id, room_type_name, capacity, extra_bed_capacity from e_room_type where tenant_id = $1 and property = $2 order by base_rate, room_type_name",
    )
    .bind(ctx.tenant)
    .bind(p.id)
    .fetch_all(&mut *tx)
    .await?;
    let mut offers = Vec::new();
    for (rt, name, capacity, extra) in types {
        let sleeps = capacity.unwrap_or(2) + extra.unwrap_or(0);
        if sleeps < adults + children {
            continue;
        }
        let free = free_rooms(&mut tx, ctx.tenant, rt, arrival, departure).await?;
        if free < 1 {
            continue;
        }
        let (total, deposit) = priced(&mut tx, ctx.tenant, rt, p.plan, arrival, n).await?;
        let amenities: Vec<String> = sqlx::query_scalar("select amenity from e_room_type__amenities where tenant_id = $1 and parent_id = $2 order by idx")
            .bind(ctx.tenant)
            .bind(rt)
            .fetch_all(&mut *tx)
            .await?;
        offers.push(
            json!({"room_type": rt, "name": name, "sleeps": sleeps, "free": free, "total": money(total), "deposit": money(deposit), "amenities": amenities}),
        );
    }
    let terms = match p.plan {
        Some(plan) => policy(&mut tx, ctx.tenant, plan).await?.1,
        None => None,
    };
    tx.commit().await?;
    offers.sort_by(|a, b| a["total"].as_str().and_then(|x| x.parse::<Decimal>().ok()).cmp(&b["total"].as_str().and_then(|x| x.parse::<Decimal>().ok())));
    Ok(json!({"property": p.name, "arrival": arrival, "departure": departure, "nights": n, "refund_days": terms, "offers": offers}))
}

/// What a guest sees of their reservation.
async fn guest_view(tx: &mut Tx, tenant: Uuid, stay: &Doc) -> Result<Value> {
    let names: Option<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "select p.property_name, t.room_type_name, p.check_in_time, p.check_out_time from e_hotel_property p \
         left join e_room_type t on t.tenant_id = p.tenant_id and t.id = $3 where p.tenant_id = $1 and p.id = $2",
    )
    .bind(tenant)
    .bind(id(stay, "property"))
    .bind(id(stay, "room_type"))
    .fetch_optional(&mut **tx)
    .await?;
    let (property, room_type, check_in, check_out) = names.unwrap_or_default();
    let refund_days = match id(stay, "rate_plan") {
        Some(plan) => policy(tx, tenant, plan).await?.1,
        None => None,
    };
    let status = s(stay, "status").unwrap_or("Reserved");
    Ok(json!({
        "number": s(stay, "number"), "property": property, "room_type": room_type, "guest_name": s(stay, "guest_name"),
        "arrival": s(stay, "arrival"), "departure": s(stay, "departure"), "nights": stay.get("nights"), "adults": stay.get("adults"), "children": stay.get("children"),
        "check_in_time": check_in, "check_out_time": check_out, "total": stay.get("total"), "deposit_due": stay.get("deposit_due"),
        "deposit_paid": stay.get("deposit_paid"), "deposit_outcome": stay.get("deposit_outcome"), "refund_days": refund_days, "status": status,
        "cancellable": status == "Reserved",
    }))
}

/// Text a guest typed, safe in an email.
fn escape(v: &str) -> String {
    v.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/// `POST /book {property, room_type, arrival, departure, adults, children, name, email, phone, notes}`:
/// a reservation held like the front desk's, at the online rate plan, its guest given their link
/// (and emailed it, where the server sends mail).
async fn book(k: &Kernel, ctx: &Ctx, body: &Map<String, Value>) -> Result<(u16, Value)> {
    // A form field people never see: whatever fills it is a program, answered like a person.
    if body.get("website").is_some_and(|v| !v.is_null() && v.as_str() != Some("")) {
        return Ok((201, json!({"status": "received"})));
    }
    let mut errors = Vec::new();
    let name = text(body, "name", 200, &mut errors);
    let email = text(body, "email", 254, &mut errors);
    let phone = text(body, "phone", 40, &mut errors);
    let notes = text(body, "notes", 2000, &mut errors);
    let code = text(body, "property", 20, &mut errors);
    if name.is_none() {
        errors.push(FieldError::new("name", "is required"));
    }
    if !email.as_deref().is_some_and(|e| e.contains('@') && !e.contains(char::is_whitespace)) {
        errors.push(FieldError::new("email", "is an email address, where the reservation is sent"));
    }
    let room_type: Option<Uuid> = body.get("room_type").and_then(Value::as_str).and_then(|v| v.trim().parse().ok());
    if room_type.is_none() {
        errors.push(FieldError::new("room_type", "choose a room"));
    }
    if !errors.is_empty() {
        return Err(Error::Validation(errors));
    }
    let (adults, children) = party(whole(body.get("adults")), whole(body.get("children")))?;
    let arrival = body.get("arrival").and_then(Value::as_str).map(str::to_string);
    let departure = body.get("departure").and_then(Value::as_str).map(str::to_string);
    with_retry(|| async {
        let mut tx = k.db.tenant_tx(ctx.tenant).await?;
        let p = open_property(&mut tx, ctx, code.as_deref()).await?;
        let (arrival, departure, _) = nights(arrival.as_deref(), departure.as_deref(), p.today)?;
        let rt = room_type.ok_or_else(|| Error::NotFound("room type".into()))?;
        let fits: Option<(Option<Uuid>, Option<i64>, Option<i64>)> =
            sqlx::query_as("select property, capacity, extra_bed_capacity from e_room_type where tenant_id = $1 and id = $2")
                .bind(ctx.tenant)
                .bind(rt)
                .fetch_optional(&mut *tx)
                .await?;
        let (of, capacity, extra) = fits.filter(|f| f.0 == Some(p.id)).ok_or_else(|| Error::NotFound("room type".into()))?;
        let _ = of;
        if capacity.unwrap_or(2) + extra.unwrap_or(0) < adults + children {
            return Err(Error::Validation(vec![FieldError::new("room_type", "does not sleep so many: choose a larger room")]));
        }
        let input = json!({"property": p.id.to_string(), "guest_name": name, "guest_email": email, "guest_phone": phone, "arrival": arrival.to_string(),
                           "departure": departure.to_string(), "adults": adults, "children": children, "room_type": rt.to_string(),
                           "rate_plan": p.plan.map(|x| x.to_string()), "notes": notes});
        let stay = k.insert_tx(ctx, &mut tx, "Stay", obj(input)).await?;
        let sid = id(&stay, "id").ok_or_else(|| Error::internal("a stay without an id"))?;
        let (token, hash) = new_token();
        k.set_values_tx(ctx, &mut tx, "Stay", sid, obj(json!({"source": "Online", "guest_token_hash": hash}))).await?;
        let stay = k.get_tx(ctx, &mut tx, "Stay", sid, false).await?;
        let view = guest_view(&mut tx, ctx.tenant, &stay).await?;
        if k.mailer.is_some()
            && let Some(to) = email.as_deref()
        {
            let slug: String = sqlx::query_scalar("select slug from tenant where id = $1").bind(ctx.tenant).fetch_one(&mut *tx).await?;
            let link = format!("{}/hotel/book/{slug}/manage/{token}", k.settings.public_url.as_deref().unwrap_or("").trim_end_matches('/'));
            let subject = format!("Your reservation at {}: {}", p.name, s(&stay, "number").unwrap_or_default());
            let html = format!(
                "<p>Hello {},</p><p>Your reservation {} at {} is booked: {} to {}, {}.</p><p>Total {}; deposit {}.</p><p><a href=\"{}\">View or cancel it</a></p>",
                escape(name.as_deref().unwrap_or("there")),
                escape(s(&stay, "number").unwrap_or_default()),
                escape(&p.name),
                arrival,
                departure,
                escape(view["room_type"].as_str().unwrap_or_default()),
                s(&stay, "total").unwrap_or_default(),
                s(&stay, "deposit_due").unwrap_or("0"),
                escape(&link)
            );
            k.queue_mail(&mut tx, ctx.tenant, to, &subject, &html).await?;
        }
        tx.commit().await?;
        Ok((201, json!({"reservation": view, "token": token})))
    })
    .await
}

/// `GET /manage/{token}`: the guest's reservation.
async fn manage(k: &Kernel, ctx: &Ctx, tok: &str) -> Result<Value> {
    let mut tx = k.db.tenant_tx(ctx.tenant).await?;
    let sid = by_token(&mut tx, ctx.tenant, tok).await?;
    let stay = k.get_tx(ctx, &mut tx, "Stay", sid, false).await?;
    let view = guest_view(&mut tx, ctx.tenant, &stay).await?;
    tx.commit().await?;
    Ok(json!({"reservation": view}))
}

/// `POST /manage/{token}/cancel {reason?}`: the guest cancels while it is reserved; its deposit is
/// given back or kept by the rate plan's policy, as at the front desk.
async fn cancel(k: &Kernel, ctx: &Ctx, tok: &str, body: &Map<String, Value>) -> Result<Value> {
    let mut errors = Vec::new();
    let reason = text(body, "reason", 500, &mut errors).unwrap_or_else(|| "Cancelled by the guest online".into());
    if !errors.is_empty() {
        return Err(Error::Validation(errors));
    }
    with_retry(|| async {
        let mut tx = k.db.tenant_tx(ctx.tenant).await?;
        let sid = by_token(&mut tx, ctx.tenant, tok).await?;
        let stay = k.get_tx(ctx, &mut tx, "Stay", sid, true).await?;
        if s(&stay, "status") != Some("Reserved") {
            return Err(Error::invalid(format!("this reservation is {}: it is not cancelled online", s(&stay, "status").unwrap_or("closed").to_lowercase())));
        }
        let done = k.run_action_tx(ctx, &mut tx, "Stay", sid, "cancel_stay", &json!({"reason": reason})).await?;
        let stay = k.get_tx(ctx, &mut tx, "Stay", sid, false).await?;
        let view = guest_view(&mut tx, ctx.tenant, &stay).await?;
        tx.commit().await?;
        Ok(json!({"reservation": view, "message": done.get("message")}))
    })
    .await
}
