//! A hotel stay end to end (FERRUM-JOURNEY-010) against a real PostgreSQL
//! (`FERRUM_HOSPITALITY_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_hospitality_test`;
//! reset first): rooms and a rate plan; reservations that no room type is sold past and no room
//! twice; check-in only into a ready room and on the arrival day; a folio with a minibar charge and
//! a payment; night audits that charge each night once at its season's rate, make a no-show of a
//! reservation that did not come and keep the night's figures; check-out with the folio invoiced
//! through ERP; cancellation; and who may do what.

use ferrum_kernel::secret::SecretKey;
use ferrum_kernel::tenant::NewTenant;
use ferrum_kernel::{Ctx, Db, DbConfig, Doc, Error, HookRegistry, Kernel, KernelSettings, ListQuery};
use ferrum_meta::Catalog;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap()
}

fn uid(doc: &Doc) -> Uuid {
    doc["id"].as_str().unwrap().parse().unwrap()
}

fn dec(v: &Value) -> f64 {
    v.as_str().map(|s| s.parse().unwrap()).or_else(|| v.as_f64()).unwrap_or_else(|| panic!("not a number: {v}"))
}

struct T {
    k: Arc<Kernel>,
    tenant: Uuid,
    admin: Ctx,
}

impl T {
    async fn insert(&self, ctx: &Ctx, entity: &str, v: Value) -> Doc {
        self.k.insert(ctx, entity, obj(v)).await.unwrap_or_else(|e| panic!("insert {entity}: {e:?}"))
    }
    async fn try_insert(&self, ctx: &Ctx, entity: &str, v: Value) -> Result<Doc, Error> {
        self.k.insert(ctx, entity, obj(v)).await
    }
    async fn get(&self, entity: &str, id: Uuid) -> Doc {
        self.k.get(&self.admin, entity, id, false).await.unwrap_or_else(|e| panic!("get {entity}: {e:?}"))
    }
    async fn act(&self, ctx: &Ctx, entity: &str, id: Uuid, name: &str, params: Value) -> Result<Value, Error> {
        self.k.run_action(ctx, entity, id, name, params).await
    }
    async fn find_one(&self, entity: &str, filters: Value) -> Doc {
        let q = ListQuery { filters: ferrum_kernel::query::parse_filters(&filters).unwrap(), limit: Some(10), ..Default::default() };
        let rows = self.k.list(&self.admin, entity, &q).await.unwrap().rows;
        assert_eq!(rows.len(), 1, "{entity} {filters}");
        rows[0].clone()
    }
    async fn user(&self, email: &str, roles: &[&str]) -> Ctx {
        let roles: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        let id = self.k.create_user(&self.admin, email, email, Some("Correct-Horse-9"), &roles, false).await.expect("user");
        let s = self.k.login("harbor", email, "Correct-Horse-9").await.expect("login");
        self.k.load_ctx(self.tenant, id, Some(s.token_version), "it".into()).await.expect("ctx")
    }
}

/// A refusal of the expected kind, saying `says`.
fn refused<V: std::fmt::Debug>(r: Result<V, Error>, kind: &str, says: &str) {
    let msg = format!("{r:?}");
    let ok = matches!(
        (&r, kind),
        (Err(Error::Invalid(_) | Error::Validation(_)), "invalid") | (Err(Error::Forbidden(_)), "forbidden") | (Err(Error::NotFound(_)), "not found")
    );
    assert!(ok && msg.contains(says), "expected {kind} saying {says:?}, got {msg}");
}

#[test]
fn a_hotel_stay() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(scenario()).await.unwrap() });
}

async fn scenario() {
    let url =
        std::env::var("FERRUM_HOSPITALITY_TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_hospitality_test".into());
    let db = Db::connect(DbConfig { url, max_connections: 16, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    packages.extend(ferrum_app_erp::packages().expect("ERP's packages"));
    packages.extend(ferrum_app_hospitality::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    ferrum_app_erp::register(&mut hooks);
    ferrum_app_hospitality::register(&mut hooks);
    let settings = KernelSettings { secret_key: Some(SecretKey::new([7u8; 32])), ..Default::default() };
    let k = Kernel::boot(db, catalog, hooks, settings, None).await.expect("boot");
    ferrum_domain::maintenance(&k).await.expect("partitions");
    ferrum_app_erp::maintenance(&k).await.expect("partitions");
    let tenant = k
        .create_tenant(NewTenant {
            slug: "harbor".into(),
            name: "Harbor Hotel".into(),
            admin_email: "admin@harbor.test".into(),
            admin_password: "Correct-Horse-9".into(),
            admin_name: None,
            packages: vec!["hospitality".into()],
            settings: Some(json!({"fiscal_year_start_month": 4, "timezone": "Asia/Kolkata"})),
        })
        .await
        .expect("tenant");
    let session = k.login("harbor", "admin@harbor.test", "Correct-Horse-9").await.expect("login");
    let admin = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let t = T { k: k.clone(), tenant, admin };
    let a = t.admin.clone();

    // ───────────── the books, and the hotel ─────────────
    let inr = t.find_one("Currency", json!({"code": "INR"})).await;
    let company = uid(&t.insert(&a, "Company", json!({"company_name": "Harbor Hotel", "abbr": "HH", "default_currency": inr["id"]})).await);
    let services = t.find_one("ItemGroup", json!({"group_name": "Services"})).await;
    let nos = t.find_one("Uom", json!({"uom_name": "Nos"})).await;
    let item = |code: &str| json!({"item_code": code, "item_name": code, "item_group": services["id"], "stock_uom": nos["id"], "is_stock_item": false});
    let room_night = uid(&t.insert(&a, "Item", item("ROOM-NIGHT")).await);
    let minibar = uid(&t.insert(&a, "Item", item("MINIBAR")).await);
    let walk_in = uid(&t.insert(&a, "Customer", json!({"customer_name": "Walk-in Guest", "customer_type": "Individual"})).await);
    let meera = uid(&t.insert(&a, "Customer", json!({"customer_name": "Meera Iyer", "customer_type": "Individual", "email": "meera@guest.test"})).await);
    let manager = t.user("manager@harbor.test", &["Hotel Manager"]).await;
    let desk = t.user("desk@harbor.test", &["Front Desk"]).await;
    let keeper = t.user("rooms@harbor.test", &["Housekeeping"]).await;

    let hotel = t
        .insert(
            &manager,
            "HotelProperty",
            json!({"property_name": "Harbor Hotel", "code": "HH", "company": company.to_string(), "room_item": room_night.to_string(), "walk_in_customer": walk_in.to_string()}),
        )
        .await;
    let hid = uid(&hotel);
    let other = uid(&t
        .insert(
            &manager,
            "HotelProperty",
            json!({"property_name": "Hill Lodge", "code": "HL", "company": company.to_string(), "room_item": room_night.to_string()}),
        )
        .await);
    let deluxe = uid(&t
        .insert(
            &manager,
            "RoomType",
            json!({"room_type_name": "Deluxe", "property": hid.to_string(), "capacity": 2, "base_rate": "4000", "amenities": [{"amenity": "Sea view"}]}),
        )
        .await);
    let suite = uid(&t.insert(&manager, "RoomType", json!({"room_type_name": "Suite", "property": hid.to_string(), "capacity": 3, "base_rate": "9000"})).await);
    let lodge_type = uid(&t.insert(&manager, "RoomType", json!({"room_type_name": "Cabin", "property": other.to_string(), "base_rate": "3000"})).await);
    let room = |n: &str, ty: Uuid| json!({"room_number": n, "property": hid.to_string(), "room_type": ty.to_string()});
    let r101 = uid(&t.insert(&manager, "Room", room("101", deluxe)).await);
    let r102 = uid(&t.insert(&manager, "Room", room("102", deluxe)).await);
    let r201 = uid(&t.insert(&manager, "Room", room("201", suite)).await);
    t.insert(&manager, "Room", room("202", suite)).await;
    refused(
        t.try_insert(&manager, "Room", json!({"room_number": "301", "property": hid.to_string(), "room_type": lodge_type.to_string()})).await,
        "invalid",
        "another property",
    );

    // The season prices a Deluxe night at 5,000 from 1 December; outside it, the base rate.
    let (d0, d1, d2, d3) = ("2026-12-01", "2026-12-02", "2026-12-03", "2026-12-04");
    let bb = uid(&t
        .insert(
            &manager,
            "RatePlan",
            json!({"rate_plan_name": "Bed and breakfast", "property": hid.to_string(), "breakfast_included": true,
                   "seasons": [{"room_type": deluxe.to_string(), "from_date": d0, "to_date": "2026-12-31", "rate": "5000"}]}),
        )
        .await);
    // What the actions read, their pages ask for: the first business date, the room, why a
    // reservation is cancelled.
    let asks = |entity: &str, action: &str| -> Vec<String> {
        let ent = t.admin.schema.entity(entity).unwrap();
        ent.def.actions.iter().find(|a| a.name == action).unwrap().params.iter().map(|p| p.name.clone()).collect()
    };
    assert_eq!(
        (asks("HotelProperty", "open_business_date"), asks("Stay", "check_in"), asks("Stay", "cancel_stay")),
        (vec!["date".to_string()], vec!["room".to_string()], vec!["reason".to_string()])
    );
    refused(t.act(&manager, "HotelProperty", hid, "night_audit", Value::Null).await, "invalid", "business date first");
    let opened = t.act(&manager, "HotelProperty", hid, "open_business_date", json!({"date": d0})).await.expect("the first business date");
    assert_eq!(opened["message"], "The business date is 2026-12-01");
    refused(t.act(&manager, "HotelProperty", hid, "open_business_date", json!({"date": d1})).await, "invalid", "not available");

    // ───────────── reservations ─────────────
    let stay = |guest: &str, ty: Uuid, from: &str, to: &str, extra: Value| {
        let mut v = json!({"property": hid.to_string(), "guest_name": guest, "room_type": ty.to_string(), "arrival": from, "departure": to});
        v.as_object_mut().unwrap().extend(extra.as_object().cloned().unwrap_or_default());
        v
    };
    let ana = t
        .insert(
            &desk,
            "Stay",
            stay("Ana Rao", deluxe, d0, d2, json!({"rate_plan": bb.to_string(), "customer": meera.to_string(), "guest_email": "ana@guest.test"})),
        )
        .await;
    assert_eq!((ana["nights"].as_i64(), dec(&ana["total"])), (Some(2), 10000.0), "two nights at the season's 5,000");
    assert_eq!(ana["status"], "Reserved");
    assert!(ana["number"].as_str().unwrap().starts_with("STAY-"), "{ana:?}");
    let ana_id = uid(&ana);
    let bo = uid(&t.insert(&desk, "Stay", stay("Bo Chen", deluxe, d0, d1, json!({"room": r102.to_string()}))).await);
    refused(
        t.try_insert(&desk, "Stay", stay("Cy Diaz", deluxe, d0, d1, json!({}))).await,
        "invalid",
        "no Deluxe room is free on the night of 2026-12-01: all 2 are taken",
    );
    let base = t.insert(&desk, "Stay", stay("Cy Diaz", deluxe, d2, d3, json!({}))).await;
    assert_eq!(dec(&base["total"]), 4000.0, "without a rate plan, a night is the room type's base rate");
    let cy = uid(&base);
    let dee = uid(&t.insert(&desk, "Stay", stay("Dee Ito", suite, d1, d3, json!({"room": r201.to_string()}))).await);
    refused(t.try_insert(&desk, "Stay", stay("Eli Fox", suite, d2, d3, json!({"room": r201.to_string()}))).await, "invalid", "held by");
    refused(t.try_insert(&desk, "Stay", stay("Eli Fox", deluxe, d1, d1, json!({}))).await, "invalid", "at least one night");
    refused(t.try_insert(&desk, "Stay", stay("Eli Fox", deluxe, d1, d2, json!({"room": r201.to_string()}))).await, "invalid", "not a room of this room type");
    refused(t.try_insert(&desk, "Stay", stay("Eli Fox", lodge_type, d1, d2, json!({}))).await, "invalid", "another property");
    refused(t.try_insert(&keeper, "Stay", stay("Eli Fox", suite, d0, d1, json!({}))).await, "forbidden", "");
    // A suite night with no season is its base rate.
    assert_eq!(dec(&t.get("Stay", dee).await["total"]), 18000.0);

    // ───────────── check-in ─────────────
    t.k.update(&keeper, "Room", r101, obj(json!({"housekeeping": "Dirty"})), None).await.expect("housekeeping marks a room");
    refused(t.act(&desk, "Stay", ana_id, "check_in", json!({"room": r101.to_string()})).await, "invalid", "room 101 is dirty");
    refused(t.act(&desk, "Stay", dee, "check_in", Value::Null).await, "invalid", "arrives on 2026-12-02");
    t.k.update(&keeper, "Room", r101, obj(json!({"housekeeping": "Inspected"})), None).await.unwrap();
    let inn = t.act(&desk, "Stay", ana_id, "check_in", json!({"room": r101.to_string()})).await.expect("checked in");
    assert_eq!(inn["status"], "Checked In");
    assert!(inn["message"].as_str().unwrap().starts_with("Checked in to room 101; folio FOL-"), "{inn}");
    let ana = t.get("Stay", ana_id).await;
    let folio = uid(&obj(json!({"id": ana["folio"]})));
    assert!(t.get("Room", r101).await["occupied"].as_bool().unwrap(), "the room is taken");
    refused(t.act(&desk, "Stay", ana_id, "check_in", Value::Null).await, "invalid", "");
    refused(t.k.update(&desk, "Stay", ana_id, obj(json!({"arrival": d1})), None).await, "invalid", "in the house");

    // The guest's folio: a minibar charge and a deposit; room nights are the audit's.
    let f = t.get("Folio", folio).await;
    assert_eq!((f["status"].as_str(), f["guest_name"].as_str()), (Some("Open"), Some("Ana Rao")));
    refused(t.try_insert(&desk, "Folio", json!({"stay": ana_id.to_string(), "guest_name": "Ana Rao"})).await, "forbidden", "");
    let f =
        t.k.update(
            &desk,
            "Folio",
            folio,
            obj(json!({"charges": [{"charge_date": d0, "kind": "Service", "description": "Minibar", "item": minibar.to_string(), "qty": "2", "rate": "175"}],
                       "payments": [{"paid_on": d0, "amount": "2000", "mode": "Card", "reference": "AUTH-1"}]})),
            None,
        )
        .await
        .expect("a charge and a payment");
    assert_eq!((dec(&f["total_charges"]), dec(&f["total_payments"]), dec(&f["balance"])), (350.0, 2000.0, -1650.0));
    refused(
        t.k.update(&desk, "Folio", folio, obj(json!({"charges": [{"charge_date": d0, "kind": "Room", "description": "A night", "rate": "1"}]})), None).await,
        "invalid",
        "posted by the night audit",
    );

    // ───────────── the night of 1 December ─────────────
    refused(t.act(&desk, "HotelProperty", hid, "night_audit", Value::Null).await, "forbidden", "");
    let night = t.act(&manager, "HotelProperty", hid, "night_audit", Value::Null).await.expect("the night audit");
    assert_eq!((night["charges_posted"].as_i64(), night["no_shows"].as_i64(), night["next"].as_str()), (Some(1), Some(1), Some(d1)), "{night}");
    assert_eq!(night["message"], "The night of 2026-12-01 is closed: 1 of 4 rooms occupied, 1 room charge posted, 1 no-show; the business date is 2026-12-02");
    assert_eq!(t.get("Stay", bo).await["status"], "No Show", "Bo never arrived");
    let audit = t.find_one("NightAudit", json!({"business_date": d0})).await;
    assert_eq!(
        (
            audit["rooms_available"].as_i64(),
            audit["rooms_occupied"].as_i64(),
            dec(&audit["occupancy_percent"]),
            dec(&audit["room_revenue"]),
            dec(&audit["average_daily_rate"]),
            dec(&audit["revpar"])
        ),
        (Some(4), Some(1), 25.0, 5000.0, 5000.0, 1250.0)
    );
    let f = t.get("Folio", folio).await;
    let nights: Vec<f64> = f["charges"].as_array().unwrap().iter().filter(|c| c["kind"] == "Room").map(|c| dec(&c["amount"])).collect();
    assert_eq!(nights, [5000.0], "the night at its season's rate");
    assert_eq!(dec(&f["balance"]), 3350.0);
    // The front desk cannot edit the night it was charged.
    let mut charges = f["charges"].as_array().unwrap().clone();
    for c in charges.iter_mut().filter(|c| c["kind"] == "Room") {
        c["rate"] = json!("100");
    }
    refused(t.k.update(&desk, "Folio", folio, obj(json!({"charges": charges})), None).await, "invalid", "posted by the night audit");

    // ───────────── the night of 2 December ─────────────
    t.act(&desk, "Stay", dee, "check_in", Value::Null).await.expect("Dee into the suite held for her");
    let night = t.act(&manager, "HotelProperty", hid, "night_audit", Value::Null).await.unwrap();
    assert_eq!((night["charges_posted"].as_i64(), dec(&night["room_revenue"])), (Some(2), 14000.0), "Ana's second night and Dee's first: {night}");
    assert_eq!(t.get("HotelProperty", hid).await["business_date"], d2);
    let f = t.get("Folio", folio).await;
    assert_eq!(f["charges"].as_array().unwrap().iter().filter(|c| c["kind"] == "Room").count(), 2, "one charge a night");
    // A business date wound back (an operator's mistake) is not audited again.
    let mut tx = t.k.db.tenant_tx(t.tenant).await.unwrap();
    sqlx::query("update e_hotel_property set business_date = $2 where tenant_id = $1 and id = $3")
        .bind(t.tenant)
        .bind(chrono::NaiveDate::parse_from_str(d1, "%Y-%m-%d").unwrap())
        .bind(hid)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    refused(t.act(&manager, "HotelProperty", hid, "night_audit", Value::Null).await, "invalid", "the night of 2026-12-02 was audited already");
    let mut tx = t.k.db.tenant_tx(t.tenant).await.unwrap();
    sqlx::query("update e_hotel_property set business_date = $2 where tenant_id = $1 and id = $3")
        .bind(t.tenant)
        .bind(chrono::NaiveDate::parse_from_str(d2, "%Y-%m-%d").unwrap())
        .bind(hid)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // ───────────── check-out ─────────────
    let out = t.act(&desk, "Stay", ana_id, "check_out", Value::Null).await.expect("checked out");
    let invoice = uid(&obj(json!({"id": out["sales_invoice"]["id"]})));
    let inv = t.get("SalesInvoice", invoice).await;
    assert_eq!(inv["customer"].as_str(), Some(meera.to_string().as_str()), "the stay's customer");
    let lines: Vec<(String, f64, f64)> =
        inv["items"].as_array().unwrap().iter().map(|l| (l["item"].as_str().unwrap().to_string(), dec(&l["qty"]), dec(&l["price_list_rate"]))).collect();
    assert_eq!(
        lines,
        [(minibar.to_string(), 2.0, 175.0), (room_night.to_string(), 1.0, 5000.0), (room_night.to_string(), 1.0, 5000.0)],
        "the folio's charges, room nights as the property's room item"
    );
    let ana = t.get("Stay", ana_id).await;
    assert_eq!((ana["status"].as_str(), ana["sales_invoice"].as_str()), (Some("Checked Out"), Some(invoice.to_string().as_str())));
    // What Ana paid at the desk goes to ERP: a draft payment from her customer into the takings
    // account (the company's cash account here), for accounts to apply to the invoice.
    assert!(out["message"].as_str().unwrap().ends_with("; 2000.00 paid at the desk waits in ERP as draft payments to apply to it"), "{out}");
    let paid = &t.get("Folio", folio).await["payments"][0];
    let pe = t.get("PaymentEntry", uid(&obj(json!({"id": paid["payment_entry"]})))).await;
    assert_eq!(
        (
            pe["docstatus"].as_i64(),
            pe["payment_type"].as_str(),
            pe["customer"].as_str(),
            dec(&pe["paid_amount"]),
            pe["reference_no"].as_str(),
            pe["posting_date"].as_str()
        ),
        (Some(0), Some("Receive"), Some(meera.to_string().as_str()), 2000.0, Some("AUTH-1"), Some(d0))
    );
    let cash = t.get("Company", company).await["default_cash_account"].clone();
    assert_eq!(pe["bank_account"], cash);
    assert_eq!(t.get("Folio", folio).await["status"], "Closed");
    let r = t.get("Room", r101).await;
    assert_eq!((r["occupied"].as_bool(), r["housekeeping"].as_str()), (Some(false), Some("Dirty")), "it needs cleaning");
    refused(t.k.update(&desk, "Folio", folio, obj(json!({"payments": []})), None).await, "invalid", "closed");
    refused(t.k.update(&desk, "Stay", ana_id, obj(json!({"departure": d3})), None).await, "invalid", "checked out does not change");
    // Dee had no customer: her folio goes to the walk-in customer.
    let out = t.act(&desk, "Stay", dee, "check_out", Value::Null).await.expect("Dee leaves early");
    assert!(!out["message"].as_str().unwrap().contains("paid at the desk"), "nothing was paid: {out}");
    let inv = t.get("SalesInvoice", uid(&obj(json!({"id": out["sales_invoice"]["id"]})))).await;
    assert_eq!(inv["customer"].as_str(), Some(walk_in.to_string().as_str()));

    // ───────────── cancellation frees the night ─────────────
    refused(t.act(&desk, "Stay", cy, "cancel_stay", json!({})).await, "invalid", "say why");
    let cancelled = t.act(&desk, "Stay", cy, "cancel_stay", json!({"reason": "plans changed"})).await.expect("cancelled");
    assert_eq!(cancelled["message"], "The reservation is cancelled; its nights are free again");
    assert_eq!(t.get("Stay", cy).await["status"], "Cancelled");
    let two = t.insert(&desk, "Stay", stay("Fay Gill", deluxe, d2, d3, json!({}))).await;
    t.insert(&desk, "Stay", stay("Gus Hale", deluxe, d2, d3, json!({}))).await;
    refused(t.try_insert(&desk, "Stay", stay("Hal Ives", deluxe, d2, d3, json!({}))).await, "invalid", "no Deluxe room is free");
    // Out of order, a room is not sold.
    t.k.update(&manager, "Room", r102, obj(json!({"housekeeping": "Out of Order"})), None).await.unwrap();
    refused(t.k.update(&desk, "Stay", uid(&two), obj(json!({"departure": "2026-12-05"})), None).await, "invalid", "all 1 are taken");
}
