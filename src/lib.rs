//! Hospitality: a hotel's front office (FERRUM-INDUSTRY-001) — properties, room types and rooms,
//! rate plans, stays from reservation to check-out, folios and the night audit.
//!
//! * `stay`: reservations held night by night against their room and their room type (no room
//!   is sold twice, no type past its rooms), check-in into a clean room, check-out with the
//!   folio's invoice, cancellation; the per-night price.
//! * `folio`: a stay's charges and payments, its totals, and room nights only the audit posts.
//! * `audit`: the night audit, which closes a business date: each stay in the house is charged
//!   its night, reservations that did not arrive become no-shows, and the day's figures are kept.
//!
//! Hooks are registered per entity, so a workspace without the `hospitality` package never runs them.

mod audit;
mod folio;
mod stay;

use chrono::NaiveDate;
use ferrum_domain::util::s;
use ferrum_kernel::{Doc, HookRegistry};
use serde_json::Value;
use std::sync::Arc;

include!(concat!(env!("OUT_DIR"), "/packages.rs"));
include!(concat!(env!("OUT_DIR"), "/ui.rs"));

/// This app's manifest (`app.yaml`).
pub const MANIFEST: &str = include_str!("../app.yaml");

/// This app's metadata packages, embedded at build time.
pub fn packages() -> std::result::Result<Vec<ferrum_meta::Package>, ferrum_meta::MetaError> {
    ferrum_meta::catalog::load_embedded(EMBEDDED)
}

/// Registers this app's document hooks.
pub fn register(hooks: &mut HookRegistry) {
    hooks
        .register("Stay", Arc::new(stay::StayHook))
        .register("Room", Arc::new(stay::RoomHook))
        .register("Folio", Arc::new(folio::FolioHook))
        .register("HotelProperty", Arc::new(audit::PropertyHook));
}

/// A date field.
pub(crate) fn date(doc: &Doc, k: &str) -> Option<NaiveDate> {
    s(doc, k).and_then(|v| NaiveDate::parse_from_str(v.get(..10).unwrap_or(v), "%Y-%m-%d").ok())
}

/// A document from JSON.
pub(crate) fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap_or_default()
}
