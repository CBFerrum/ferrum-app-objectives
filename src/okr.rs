//! Objectives, key results and check-ins.
//!
//! * A key result goes from its start towards its target: up for an increase, down for a decrease,
//!   or done for a milestone. Its value and confidence are its latest check-in's by day; its
//!   progress is how far along that way it is, from 0 to 100.
//! * An objective's progress is the average of its key results' and of the objectives aligned to
//!   it, rolled up the tree whenever one of them moves. An objective under another falls within its
//!   period (the kernel keeps the tree from looping).
//! * Check-ins fall within their objective's period and not after today, and a closed objective
//!   takes none; closing it keeps its progress as its score.

use crate::obj;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use ferrum_domain::util::{d, elevated, id, money, round, s};
use ferrum_kernel::{Ctx, Doc, DocHook, Error, HookCtx, Kernel, Result, Tx};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

/// How far a key result of `kind` has come from `start` towards `target`, from 0 to 100.
pub(crate) fn progress(kind: &str, start: Decimal, target: Decimal, current: Decimal) -> Decimal {
    let hundred = Decimal::ONE_HUNDRED;
    let p = match kind {
        "Milestone" => {
            if current >= Decimal::ONE {
                hundred
            } else {
                Decimal::ZERO
            }
        }
        _ if target == start => Decimal::ZERO,
        _ => (current - start) / (target - start) * hundred,
    };
    round(p.clamp(Decimal::ZERO, hundred), 1)
}

fn day(doc: &Doc, k: &str) -> Option<NaiveDate> {
    s(doc, k).and_then(|v| NaiveDate::parse_from_str(v.get(..10).unwrap_or(v), "%Y-%m-%d").ok())
}

/// An objective's progress from its key results and the objectives aligned to it, then its
/// parent's, up the tree.
pub(crate) async fn roll_up(k: &Kernel, ctx: &Ctx, tx: &mut Tx, objective: Uuid) -> Result<()> {
    let mut next = Some(objective);
    let mut seen = HashSet::new();
    while let Some(o) = next {
        if !seen.insert(o) {
            break;
        }
        let parts: Vec<Option<Decimal>> = sqlx::query_scalar(
            "select progress from e_key_result where tenant_id = $1 and objective = $2 union all select progress from e_objective where tenant_id = $1 and parent = $2",
        )
        .bind(ctx.tenant)
        .bind(o)
        .fetch_all(&mut **tx)
        .await?;
        let p =
            if parts.is_empty() { Decimal::ZERO } else { round(parts.iter().map(|p| p.unwrap_or_default()).sum::<Decimal>() / Decimal::from(parts.len()), 1) };
        k.set_values_tx(ctx, tx, "Objective", o, obj(json!({"progress": money(p)}))).await?;
        next = sqlx::query_scalar("select parent from e_objective where tenant_id = $1 and id = $2")
            .bind(ctx.tenant)
            .bind(o)
            .fetch_optional(&mut **tx)
            .await?
            .flatten();
    }
    Ok(())
}

pub struct ObjectiveHook;

#[async_trait]
impl DocHook for ObjectiveHook {
    fn name(&self) -> &'static str {
        "objectives.objective"
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let (from, to) = (day(doc, "starts_on"), day(doc, "ends_on"));
        if let (Some(f), Some(t)) = (from, to)
            && t <= f
        {
            return Err(Error::field("ends_on", "is after the objective starts"));
        }
        let Some(parent) = id(doc, "parent") else { return Ok(()) };
        let period: Option<(NaiveDate, NaiveDate)> = sqlx::query_as("select starts_on, ends_on from e_objective where tenant_id = $1 and id = $2")
            .bind(hc.ctx.tenant)
            .bind(parent)
            .fetch_optional(&mut **hc.tx)
            .await?;
        if let (Some((ps, pe)), Some(f), Some(t)) = (period, from, to)
            && (f < ps || t > pe)
        {
            return Err(Error::field("starts_on", format!("falls within the objective it is aligned to, {ps} to {pe}")));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, _doc: &Doc) -> Result<()> {
        if !hc.ctx.is_system && old.is_some_and(|o| s(o, "status") == Some("Closed")) {
            return Err(Error::invalid("a closed objective is kept as it closed"));
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, old: Option<&Doc>) -> Result<()> {
        let system = elevated(hc.ctx);
        if let Some(me) = id(doc, "id") {
            roll_up(hc.kernel, &system, hc.tx, me).await?;
        }
        if let Some(before) = old.and_then(|o| id(o, "parent"))
            && Some(before) != id(doc, "parent")
        {
            roll_up(hc.kernel, &system, hc.tx, before).await?;
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        if name != "close_objective" {
            return Ok(None);
        }
        let objective = id(doc, "id").ok_or_else(|| Error::internal("an objective without an id"))?;
        let score = d(doc, "progress");
        hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "Objective", objective, obj(json!({"status": "Closed", "score": money(score)}))).await?;
        Ok(Some(json!({"status": "Closed", "score": money(score)})))
    }
}

/// Whether `objective` is closed.
async fn closed(tx: &mut Tx, tenant: Uuid, objective: Option<Uuid>) -> Result<bool> {
    let st: Option<String> = sqlx::query_scalar("select status from e_objective where tenant_id = $1 and id = $2")
        .bind(tenant)
        .bind(objective)
        .fetch_optional(&mut **tx)
        .await?
        .flatten();
    Ok(st.as_deref() == Some("Closed"))
}

pub struct KeyResultHook;

#[async_trait]
impl DocHook for KeyResultHook {
    fn name(&self) -> &'static str {
        "objectives.key_result"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["current_value", "progress"]
    }

    async fn before_validate(&self, _hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if doc.get("current_value").is_none_or(Value::is_null) {
            let start = if s(doc, "kind") == Some("Milestone") { Decimal::ZERO } else { d(doc, "start_value") };
            doc.insert("current_value".into(), money(start));
        }
        let p = progress(s(doc, "kind").unwrap_or("Increase"), d(doc, "start_value"), d(doc, "target_value"), d(doc, "current_value"));
        doc.insert("progress".into(), money(p));
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let (start, target) = (d(doc, "start_value"), d(doc, "target_value"));
        match s(doc, "kind") {
            Some("Increase") if target <= start => return Err(Error::field("target_value", "an increase aims above where it starts")),
            Some("Decrease") if target >= start => return Err(Error::field("target_value", "a decrease aims below where it starts")),
            _ => {}
        }
        if !hc.ctx.is_system && closed(hc.tx, hc.ctx.tenant, id(doc, "objective")).await? {
            return Err(Error::field("objective", "is closed: it takes no more key results"));
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, old: Option<&Doc>) -> Result<()> {
        let system = elevated(hc.ctx);
        if let Some(o) = id(doc, "objective") {
            roll_up(hc.kernel, &system, hc.tx, o).await?;
        }
        if let Some(before) = old.and_then(|o| id(o, "objective"))
            && Some(before) != id(doc, "objective")
        {
            roll_up(hc.kernel, &system, hc.tx, before).await?;
        }
        Ok(())
    }
}

pub struct CheckInHook;

#[async_trait]
impl DocHook for CheckInHook {
    fn name(&self) -> &'static str {
        "objectives.check_in"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["author"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if s(doc, "author").is_none() && !hc.ctx.email.is_empty() {
            doc.insert("author".into(), json!(hc.ctx.email));
        }
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let on = day(doc, "date").ok_or_else(|| Error::field("date", "is required"))?;
        if on > Utc::now().date_naive() {
            return Err(Error::field("date", "is today or before"));
        }
        let row: Option<(Uuid, NaiveDate, NaiveDate, Option<String>)> = sqlx::query_as(
            "select o.id, o.starts_on, o.ends_on, o.status from e_key_result kr join e_objective o on o.tenant_id = kr.tenant_id and o.id = kr.objective \
             where kr.tenant_id = $1 and kr.id = $2",
        )
        .bind(hc.ctx.tenant)
        .bind(id(doc, "key_result"))
        .fetch_optional(&mut **hc.tx)
        .await?;
        let (_, from, to, status) = row.ok_or_else(|| Error::field("key_result", "is required"))?;
        if status.as_deref() == Some("Closed") {
            return Err(Error::field("key_result", "its objective is closed: it takes no more check-ins"));
        }
        if on < from || on > to {
            return Err(Error::field("date", format!("falls within the objective's period, {from} to {to}")));
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, _old: Option<&Doc>) -> Result<()> {
        let Some(kr) = id(doc, "key_result") else { return Ok(()) };
        let system = elevated(hc.ctx);
        let latest: Option<(Decimal, String)> =
            sqlx::query_as("select value, confidence from e_check_in where tenant_id = $1 and key_result = $2 order by \"date\" desc, created_at desc limit 1")
                .bind(hc.ctx.tenant)
                .bind(kr)
                .fetch_optional(&mut **hc.tx)
                .await?;
        let Some((value, confidence)) = latest else { return Ok(()) };
        let result = hc.kernel.get_tx(&system, hc.tx, "KeyResult", kr, true).await?;
        let p = progress(s(&result, "kind").unwrap_or("Increase"), d(&result, "start_value"), d(&result, "target_value"), value);
        hc.kernel
            .set_values_tx(&system, hc.tx, "KeyResult", kr, obj(json!({"current_value": money(value), "confidence": confidence, "progress": money(p)})))
            .await?;
        if let Some(o) = id(&result, "objective") {
            roll_up(hc.kernel, &system, hc.tx, o).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::progress;
    use rust_decimal::Decimal;

    #[test]
    fn how_far_a_key_result_has_come() {
        let v = |s: &str| s.parse::<Decimal>().unwrap();
        assert_eq!(progress("Increase", v("100"), v("200"), v("150")), v("50.0"));
        assert_eq!(progress("Increase", v("100"), v("200"), v("250")), v("100.0"), "past the target is done");
        assert_eq!(progress("Increase", v("100"), v("200"), v("90")), v("0.0"), "below the start is nothing");
        assert_eq!(progress("Decrease", v("30"), v("10"), v("25")), v("25.0"));
        assert_eq!((progress("Milestone", v("0"), v("1"), v("0")), progress("Milestone", v("0"), v("1"), v("1"))), (v("0.0"), v("100.0")));
    }
}
