//! A stay's folio: its charges and payments while the guest is in the house. Its totals are worked
//! out on every save; room nights are the night audit's to post, so the front desk corrects one with
//! an adjustment rather than by editing it; a closed folio does not change.

use async_trait::async_trait;
use ferrum_domain::util::{d, money, round, rows, s, set};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::Value;

pub struct FolioHook;

/// The room nights of a folio: (row id, date, amount).
fn nights(doc: &Doc) -> Vec<(String, String, Decimal)> {
    let mut out: Vec<(String, String, Decimal)> = rows(doc, "charges")
        .iter()
        .filter(|c| s(c, "kind") == Some("Room"))
        .map(|c| (s(c, "id").unwrap_or_default().to_string(), s(c, "charge_date").unwrap_or_default().to_string(), round(d(c, "qty") * d(c, "rate"), 2)))
        .collect();
    out.sort();
    out
}

#[async_trait]
impl DocHook for FolioHook {
    fn name(&self) -> &'static str {
        "hospitality.folio"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["total_charges", "total_payments", "balance"]
    }

    async fn before_validate(&self, _hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        let mut charges = Decimal::ZERO;
        let mut lines = rows(doc, "charges");
        for c in lines.iter_mut() {
            let qty = if c.get("qty").is_none_or(Value::is_null) { Decimal::ONE } else { d(c, "qty") };
            let amount = round(qty * d(c, "rate"), 2);
            set(c, "qty", money(qty));
            set(c, "amount", money(amount));
            charges += amount;
        }
        set(doc, "charges", Value::Array(lines.into_iter().map(Value::Object).collect()));
        let payments: Decimal = rows(doc, "payments").iter().map(|p| d(p, "amount")).sum();
        set(doc, "total_charges", money(charges));
        set(doc, "total_payments", money(payments));
        set(doc, "balance", money(charges - payments));
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if hc.ctx.is_system {
            return Ok(());
        }
        let Some(old) = old else {
            return Err(Error::Forbidden("a folio opens when its guest checks in".into()));
        };
        if s(old, "status") == Some("Closed") {
            return Err(Error::invalid("this folio is closed: its stay checked out"));
        }
        if nights(old) != nights(doc) {
            return Err(Error::invalid("room nights are posted by the night audit: correct one with an adjustment"));
        }
        if rows(doc, "payments").iter().any(|p| d(p, "amount") <= Decimal::ZERO) {
            return Err(Error::field("payments", "a payment's amount is above zero"));
        }
        Ok(())
    }
}
