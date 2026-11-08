//! Objectives, key results and check-ins.
//!
//! * A key result goes from its start towards its target: up for an increase, down for a decrease,
//!   or done for a milestone. Its value and confidence are its latest check-in's by day; its
//!   progress is how far along that way it is, from 0 to 100.
//! * An objective's progress is the average of its key results' and of the objectives aligned to
//!   it, each counted by its weight (1 unless set; 0 counts for nothing), rolled up the tree
//!   whenever one of them moves. An objective under another falls within its period (the kernel
//!   keeps the tree from looping).
//! * Check-ins fall within their objective's period and not after today, and a closed objective
//!   takes none; closing it keeps its progress as its score, told on the workspace's scale (0 to
//!   100, 0.0 to 1.0, 1 to 5, or a letter) and graded Missed, Progress Made or Achieved by its
//!   settings' bands. Each of its key results is scored and graded with it, by its own progress.
//! * A key result may be measured from the records: a Studio query of one measure, as a metric is
//!   defined, or, with Metrics' connector, a certified metric it names, whose value Metrics gives as
//!   the metric is defined now (composed of others or converted between currencies as it is, and not
//!   once it is no longer certified); either is read with its owner's rights.
//!   Its check-ins then have their values measured, not typed: one each day by itself (a failure
//!   kept on the key result), and one whenever someone checks in, to say how confident they are.
//! * The owners of key results checked in by hand are reminded, in the app, once a check-in is
//!   due, and again each time another is.

use crate::obj;
use async_trait::async_trait;
use chrono::NaiveDate;
use ferrum_domain::util::{b, d, elevated, id, money, round, s, sys};
use ferrum_kernel::{Ctx, Doc, DocHook, Error, HookCtx, Kernel, Result, SiteRequest, Tx, notify};
use ferrum_studio::spec::Spec;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::HashSet;
use uuid::Uuid;

/// The average of `parts` (progress, weight) by their weights, to one decimal; 0 when nothing weighs.
pub(crate) fn weighted(parts: &[(Decimal, Decimal)]) -> Decimal {
    let total: Decimal = parts.iter().map(|(_, w)| *w).sum();
    if total <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    round(parts.iter().map(|(p, w)| *p * *w).sum::<Decimal>() / total, 1)
}

/// A score from 0 to 100 told on `scale`, and graded by the bands from which it is Progress Made
/// and Achieved. 1 to 5 runs evenly from 1 (nothing) to 5 (all); Letters are A from 90, B from 80,
/// C from 70, D from 60 and F below.
pub(crate) fn graded(score: Decimal, scale: &str, progress_from: Decimal, achieved_from: Decimal) -> (String, &'static str) {
    let shown = match scale {
        "0.0 to 1.0" => format!("{} of 1.0", round(score / Decimal::ONE_HUNDRED, 2).normalize()),
        "1 to 5" => format!("{} of 5", round(Decimal::ONE + score / Decimal::from(25), 1).normalize()),
        "Letters" => {
            let from = |n: i64| score >= Decimal::from(n);
            (if from(90) {
                "A"
            } else if from(80) {
                "B"
            } else if from(70) {
                "C"
            } else if from(60) {
                "D"
            } else {
                "F"
            })
            .to_string()
        }
        _ => format!("{} of 100", score.normalize()),
    };
    let grade = if score >= achieved_from {
        "Achieved"
    } else if score >= progress_from {
        "Progress Made"
    } else {
        "Missed"
    };
    (shown, grade)
}

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

/// A key result's measure: a Studio query of one measure, ungrouped (a metric's form), if it has
/// one.
pub(crate) fn measure_of(doc: &Doc) -> Result<Option<Spec>> {
    let raw = match doc.get("measure") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(t)) if t.trim().is_empty() => return Ok(None),
        Some(Value::String(t)) => serde_json::from_str(t).map_err(|e| Error::field("measure", format!("is not a Studio query: {e}")))?,
        Some(v) => v.clone(),
    };
    spec_from(raw, "measure").map(Some)
}

/// A Studio query of one measure, ungrouped, from `raw`; refused on `field`.
fn spec_from(raw: Value, field: &str) -> Result<Spec> {
    let spec: Spec = serde_json::from_value(raw).map_err(|e| Error::field(field, format!("is not a Studio query: {e}")))?;
    if spec.measures.len() != 1 || !spec.group_by.is_empty() || spec.pivot.is_some() || spec.time_series.is_some() {
        return Err(Error::field(field, "is one measure, ungrouped"));
    }
    Ok(spec)
}

/// What measures a key result: a query of its own, or the certified metric it names (Metrics' connector).
pub(crate) enum Source {
    Query(Box<Spec>),
    /// A metric by name, its value Metrics' own: one composed of others, or converted between
    /// currencies, reads here exactly as it does there. Its definition is never run here.
    Metric(String),
}

/// What measures a key result: the certified metric it names, only while it is certified, or else a
/// query of its own.
pub(crate) async fn source_of(tx: &mut Tx, tenant: Uuid, doc: &Doc) -> Result<Option<Source>> {
    let Some(metric) = id(doc, "metric") else { return Ok(measure_of(doc)?.map(|spec| Source::Query(Box::new(spec)))) };
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("select metric_name, status from e_metric where tenant_id = $1 and id = $2").bind(tenant).bind(metric).fetch_optional(&mut **tx).await?;
    let (name, status) = row.ok_or_else(|| Error::field("metric", "is no metric here"))?;
    if status.as_deref() != Some("Certified") {
        return Err(Error::field("metric", format!("{name} is not certified now")));
    }
    Ok(Some(Source::Metric(name)))
}

/// A certified metric's value as Metrics gives it to `ctx`: the number that person reads there.
async fn metric_value(k: &Kernel, ctx: &Ctx, name: &str) -> Result<Decimal> {
    let site = k.hooks.site("metrics").ok_or_else(|| Error::invalid("Metrics is not installed here"))?.clone();
    let req = SiteRequest {
        tenant: ctx.tenant,
        user: Some(ctx.clone()),
        method: "GET".into(),
        path: "/value".into(),
        query: json!({"metric": name}).as_object().cloned().unwrap_or_default(),
        headers: Default::default(),
        body: vec![],
        client_ip: "127.0.0.1".into(),
    };
    let r = site.handle(k, req).await?;
    let answer: Value = serde_json::from_slice(&r.body).map_err(|e| Error::internal(format!("Metrics answered with something that is not JSON: {e}")))?;
    answer["value"].as_str().and_then(|v| v.parse().ok()).ok_or_else(|| Error::internal(format!("Metrics gave no value for {name}")))
}

/// What the records say for `spec`, read with the rights of `owner`, who must be an active person
/// here.
async fn measured(k: &Kernel, tx: &mut Tx, tenant: Uuid, owner: &str, source: &Source) -> Result<Decimal> {
    let user: Option<Uuid> = sqlx::query_scalar("select id from app_user where tenant_id = $1 and is_active and lower(email::text) = lower($2)")
        .bind(tenant)
        .bind(owner)
        .fetch_optional(&mut **tx)
        .await?;
    let user = user.ok_or_else(|| Error::field("owner", format!("{owner} is not an active person here, so nothing is measured with their rights")))?;
    let ctx = k.load_ctx(tenant, user, None, "objectives.measure".into()).await?;
    let spec = match source {
        Source::Query(spec) => spec,
        Source::Metric(name) => {
            return metric_value(k, &ctx, name).await.map_err(|e| Error::field("metric", format!("{name} has no value for its owner: {}", said(&e))));
        }
    };
    let out = ferrum_studio::run::run(k, &ctx, spec).await.map_err(|e| Error::field("measure", format!("its owner cannot read it: {}", said(&e))))?;
    let key = out["columns"].as_array().and_then(|c| c.last()).and_then(|c| c["name"].as_str()).unwrap_or_default().to_string();
    let cell = out["rows"].as_array().and_then(|r| r.first()).map(|r| r[&key].clone()).unwrap_or(Value::Null);
    Ok(match cell {
        Value::String(v) => v.parse().unwrap_or_default(),
        Value::Number(n) => n.to_string().parse().unwrap_or_default(),
        _ => Decimal::ZERO,
    })
}

/// An error as a person reads it, a refused field's message included.
fn said(e: &Error) -> String {
    match e {
        Error::Validation(fields) => fields.iter().map(|f| format!("{}: {}", f.field, f.message)).collect::<Vec<_>>().join("; "),
        other => other.to_string(),
    }
}

/// Today's check-in measured from the records, when the key result's objective is active and in
/// its period; its confidence stays the last check-in's. Whether one was made.
async fn measure_today(k: &Kernel, ctx: &Ctx, tx: &mut Tx, kr: Uuid) -> Result<bool> {
    let today = ctx.today();
    let open: Option<Option<String>> = sqlx::query_scalar(
        "select kr.confidence from e_key_result kr join e_objective o on o.tenant_id = kr.tenant_id and o.id = kr.objective \
         where kr.tenant_id = $1 and kr.id = $2 and o.status = 'Active' and $3 between o.starts_on and o.ends_on",
    )
    .bind(ctx.tenant)
    .bind(kr)
    .bind(today)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(confidence) = open else { return Ok(false) };
    let confidence = confidence.unwrap_or_else(|| "On Track".into());
    k.insert_tx(ctx, tx, "CheckIn", obj(json!({"key_result": kr.to_string(), "date": today.to_string(), "confidence": confidence}))).await?;
    Ok(true)
}

/// A workspace's upkeep: today's measure of each key result measured from the records (tried once
/// a day, its failure kept on the key result), then a reminder to the owner of each key result
/// checked in by hand whose check-in is due, once until another is.
pub(crate) async fn upkeep(k: &Kernel, tenant: Uuid) -> Result<()> {
    let schema = k.registry.schema(tenant).await?;
    if !schema.packages.iter().any(|p| p == "objectives") {
        return Ok(());
    }
    // Measured from the records: by a query of its own, or by a metric with Metrics' connector.
    let measured = if schema.packages.iter().any(|p| p == crate::CONNECTOR_PACKAGE) {
        "(jsonb_typeof(kr.measure) is not distinct from 'object' or kr.metric is not null)"
    } else {
        "jsonb_typeof(kr.measure) is not distinct from 'object'"
    };
    let ctx = Ctx::system(tenant, schema);
    // The workspace's day, not UTC's.
    let today = ctx.today();
    let zone = Some(ctx.schema.timezone.as_str()).filter(|z| !z.is_empty()).unwrap_or("UTC").to_string();
    let mut tx = k.db.tenant_tx(tenant).await?;
    let due: Vec<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select kr.id from e_key_result kr join e_objective o on o.tenant_id = kr.tenant_id and o.id = kr.objective \
         where kr.tenant_id = $1 and {measured} and o.status = 'Active' and $2 between o.starts_on and o.ends_on \
         and kr.last_check_in is distinct from $2 and (kr.measured_on is null or kr.measured_on < $2) order by kr.id",
    )))
    .bind(tenant)
    .bind(today)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    for kr in due {
        // Each in a transaction of its own: one that cannot be measured leaves the others measured.
        let mut tx = k.db.tenant_tx(tenant).await?;
        let error = match measure_today(k, &ctx, &mut tx, kr).await {
            Ok(_) => {
                tx.commit().await?;
                None
            }
            Err(e) => {
                tx.rollback().await?;
                Some(said(&e))
            }
        };
        let mut tx = k.db.tenant_tx(tenant).await?;
        k.set_values_tx(&ctx, &mut tx, "KeyResult", kr, obj(json!({"measured_on": today.to_string(), "measure_error": error}))).await?;
        tx.commit().await?;
    }
    let mut tx = k.db.tenant_tx(tenant).await?;
    let late: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select kr.id, kr.title, coalesce(kr.owner, o.owner)::text from e_key_result kr join e_objective o on o.tenant_id = kr.tenant_id and o.id = kr.objective \
         where kr.tenant_id = $1 and not {measured} and o.status = 'Active' and $2 between o.starts_on and o.ends_on \
         and coalesce(kr.check_in_every, 7) > 0 \
         and coalesce(kr.last_check_in, (select max(c.\"date\") from e_check_in c where c.tenant_id = kr.tenant_id and c.key_result = kr.id), \
                      greatest((kr.created_at at time zone $3)::date, o.starts_on)) <= $2 - coalesce(kr.check_in_every, 7)::int \
         and (kr.reminded_on is null or kr.reminded_on <= $2 - coalesce(kr.check_in_every, 7)::int) order by kr.id",
    )))
    .bind(tenant)
    .bind(today)
    .bind(&zone)
    .fetch_all(&mut *tx)
    .await?;
    for (kr, title, owner) in late {
        let user: Option<Uuid> = sqlx::query_scalar("select id from app_user where tenant_id = $1 and is_active and lower(email::text) = lower($2)")
            .bind(tenant)
            .bind(owner.unwrap_or_default())
            .fetch_optional(&mut *tx)
            .await?;
        if let Some(user) = user {
            notify::send_kind(
                &mut tx,
                tenant,
                user,
                "objectives.check_in",
                "inapp",
                &format!("Time to check in on {title}"),
                None,
                Some("KeyResult"),
                Some(kr),
            )
            .await?;
        }
        k.set_values_tx(&ctx, &mut tx, "KeyResult", kr, obj(json!({"reminded_on": today.to_string()}))).await?;
    }
    tx.commit().await?;
    Ok(())
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
        let parts: Vec<(Option<Decimal>, Option<Decimal>)> = sqlx::query_as(
            "select progress, weight from e_key_result where tenant_id = $1 and objective = $2 \
             union all select progress, weight from e_objective where tenant_id = $1 and parent = $2",
        )
        .bind(ctx.tenant)
        .bind(o)
        .fetch_all(&mut **tx)
        .await?;
        let parts: Vec<(Decimal, Decimal)> = parts.into_iter().map(|(p, w)| (p.unwrap_or_default(), w.unwrap_or(Decimal::ONE))).collect();
        let p = weighted(&parts);
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
        if d(doc, "weight") < Decimal::ZERO {
            return Err(Error::field("weight", "is 0 or more"));
        }
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
        let set: Option<(Option<String>, Option<Decimal>, Option<Decimal>)> =
            sqlx::query_as("select scale, progress_from, achieved_from from e_objectives_settings where tenant_id = $1 limit 1")
                .bind(hc.ctx.tenant)
                .fetch_optional(&mut **hc.tx)
                .await?;
        let (scale, progress_from, achieved_from) = set.unwrap_or_default();
        let (scale, progress_from, achieved_from) =
            (scale.as_deref().unwrap_or("0 to 100"), progress_from.unwrap_or(Decimal::from(40)), achieved_from.unwrap_or(Decimal::from(70)));
        let (shown, grade) = graded(score, scale, progress_from, achieved_from);
        let values = json!({"status": "Closed", "score": money(score), "score_on_scale": shown, "grade": grade});
        hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "Objective", objective, obj(values.clone())).await?;
        // Its key results are scored and graded with it, each by its own progress.
        let results: Vec<(Uuid, Option<Decimal>)> = sqlx::query_as("select id, progress from e_key_result where tenant_id = $1 and objective = $2")
            .bind(hc.ctx.tenant)
            .bind(objective)
            .fetch_all(&mut **hc.tx)
            .await?;
        for (result, progress) in results {
            let score = progress.unwrap_or_default();
            let (shown, grade) = graded(score, scale, progress_from, achieved_from);
            let graded_values = json!({"score": money(score), "score_on_scale": shown, "grade": grade});
            hc.kernel.set_values_tx(&elevated(hc.ctx), hc.tx, "KeyResult", result, obj(graded_values)).await?;
        }
        Ok(Some(values))
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
        &["current_value", "progress", "owner", "measure"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        // A measure sent as text is kept as the query it says, as the form sends it.
        if let Some(Value::String(text)) = doc.get("measure") {
            let query = if text.trim().is_empty() {
                Value::Null
            } else {
                serde_json::from_str(text).map_err(|e| Error::field("measure", format!("is not a Studio query: {e}")))?
            };
            doc.insert("measure".into(), query);
        }
        if s(doc, "owner").is_none()
            && let Some(objective) = id(doc, "objective")
        {
            let owner: Option<String> = sqlx::query_scalar("select owner::text from e_objective where tenant_id = $1 and id = $2")
                .bind(hc.ctx.tenant)
                .bind(objective)
                .fetch_optional(&mut **hc.tx)
                .await?
                .flatten();
            if let Some(owner) = owner {
                doc.insert("owner".into(), json!(owner.to_lowercase()));
            }
        }
        if doc.get("current_value").is_none_or(Value::is_null) {
            let start = if s(doc, "kind") == Some("Milestone") { Decimal::ZERO } else { d(doc, "start_value") };
            doc.insert("current_value".into(), money(start));
        }
        let p = progress(s(doc, "kind").unwrap_or("Increase"), d(doc, "start_value"), d(doc, "target_value"), d(doc, "current_value"));
        doc.insert("progress".into(), money(p));
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if d(doc, "weight") < Decimal::ZERO {
            return Err(Error::field("weight", "is 0 or more"));
        }
        let (start, target) = (d(doc, "start_value"), d(doc, "target_value"));
        match s(doc, "kind") {
            Some("Increase") if target <= start => return Err(Error::field("target_value", "an increase aims above where it starts")),
            Some("Decrease") if target >= start => return Err(Error::field("target_value", "a decrease aims below where it starts")),
            _ => {}
        }
        if !hc.ctx.is_system && closed(hc.tx, hc.ctx.tenant, id(doc, "objective")).await? {
            return Err(Error::field("objective", "is closed: it takes no more key results"));
        }
        if id(doc, "metric").is_some() && measure_of(doc)?.is_some() {
            return Err(Error::field("measure", "is the certified metric's: leave it empty"));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        // Whoever sets a measure, or a metric, can read what it counts; its owner's rights measure it.
        let changed = |k: &str| old.is_none_or(|o| o.get(k) != doc.get(k));
        if (changed("measure") || changed("metric"))
            && let Some(source) = source_of(hc.tx, hc.ctx.tenant, doc).await?
            && !hc.ctx.is_system
        {
            match &source {
                Source::Query(spec) => {
                    ferrum_studio::query::compile(hc.ctx, spec).map_err(|e| Error::field("measure", format!("does not run: {}", said(&e))))?;
                }
                Source::Metric(name) => {
                    metric_value(hc.kernel, hc.ctx, name).await.map_err(|e| Error::field("metric", format!("{name} has no value for you: {}", said(&e))))?;
                }
            }
        }
        Ok(())
    }

    async fn after_save(&self, hc: &mut HookCtx<'_>, doc: &Doc, old: Option<&Doc>) -> Result<()> {
        let system = elevated(hc.ctx);
        // A new measure, or a new owner whose rights read it, is measured now.
        let changed = |k: &str| old.is_none_or(|o| o.get(k) != doc.get(k));
        // A metric no longer certified is not measured here; the day's upkeep keeps why.
        if (changed("measure") || changed("metric") || changed("owner"))
            && matches!(source_of(hc.tx, hc.ctx.tenant, doc).await, Ok(Some(_)))
            && let Some(kr) = id(doc, "id")
        {
            measure_today(hc.kernel, &system, hc.tx, kr).await?;
        }
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
        &["author", "value", "measured"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        // A new check-in on a key result measured from the records has its value measured, with the
        // key result's owner's rights; a check-in already kept is not measured again.
        let kept = match id(doc, "id") {
            Some(me) => sqlx::query_scalar::<_, i32>("select 1 from e_check_in where tenant_id = $1 and id = $2")
                .bind(hc.ctx.tenant)
                .bind(me)
                .fetch_optional(&mut **hc.tx)
                .await?
                .is_some(),
            None => false,
        };
        if !kept && let Some(kr) = id(doc, "key_result") {
            let result = hc.kernel.get_tx(&sys(hc.ctx), hc.tx, "KeyResult", kr, false).await?;
            let source = source_of(hc.tx, hc.ctx.tenant, &result).await?;
            doc.insert("measured".into(), json!(source.is_some()));
            if let Some(source) = source {
                if doc.get("value").is_some_and(|v| !v.is_null()) {
                    return Err(Error::field("value", "is measured from the records: leave it empty"));
                }
                let owner = s(&result, "owner").unwrap_or_default().to_string();
                let v = measured(hc.kernel, hc.tx, hc.ctx.tenant, &owner, &source).await?;
                doc.insert("value".into(), money(v));
                // The day's own measure is its owner's: theirs are the rights that read it.
                if hc.ctx.is_system && s(doc, "author").is_none() && !owner.is_empty() {
                    doc.insert("author".into(), json!(owner));
                }
            }
        }
        if s(doc, "author").is_none() && !hc.ctx.email.is_empty() {
            doc.insert("author".into(), json!(hc.ctx.email));
        }
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if doc.get("value").is_none_or(Value::is_null) && !b(doc, "measured") {
            return Err(Error::field("value", "is required"));
        }
        let on = day(doc, "date").ok_or_else(|| Error::field("date", "is required"))?;
        if on > hc.ctx.today() {
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
        let latest: Option<(Option<Decimal>, String, NaiveDate)> = sqlx::query_as(
            "select value, confidence, \"date\" from e_check_in where tenant_id = $1 and key_result = $2 order by \"date\" desc, created_at desc limit 1",
        )
        .bind(hc.ctx.tenant)
        .bind(kr)
        .fetch_optional(&mut **hc.tx)
        .await?;
        let Some((value, confidence, on)) = latest else { return Ok(()) };
        let value = value.unwrap_or_default();
        let result = hc.kernel.get_tx(&system, hc.tx, "KeyResult", kr, true).await?;
        let p = progress(s(&result, "kind").unwrap_or("Increase"), d(&result, "start_value"), d(&result, "target_value"), value);
        hc.kernel
            .set_values_tx(
                &system,
                hc.tx,
                "KeyResult",
                kr,
                obj(json!({"current_value": money(value), "confidence": confidence, "progress": money(p), "last_check_in": on.to_string()})),
            )
            .await?;
        if let Some(o) = id(&result, "objective") {
            roll_up(hc.kernel, &system, hc.tx, o).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{graded, measure_of, progress, weighted};
    use rust_decimal::Decimal;
    use serde_json::json;

    #[test]
    fn a_measure_is_one_measure_ungrouped() {
        let doc = |v: serde_json::Value| v.as_object().cloned().unwrap();
        assert!(measure_of(&doc(json!({}))).unwrap().is_none());
        assert!(measure_of(&doc(json!({"measure": null}))).unwrap().is_none());
        assert!(measure_of(&doc(json!({"measure": " "}))).unwrap().is_none());
        let won = measure_of(&doc(json!({"measure": {"entity": "Opportunity", "filters": {"status": "Won"}, "measures": [{"fn": "count"}]}}))).unwrap();
        assert_eq!(won.unwrap().entity, "Opportunity");
        let typed = measure_of(&doc(json!({"measure": r#"{"entity": "Project", "measures": [{"fn": "count"}]}"#}))).unwrap();
        assert_eq!(typed.unwrap().entity, "Project", "as the form sends it, as text");
        assert!(measure_of(&doc(json!({"measure": "not a query"}))).is_err());
        assert!(measure_of(&doc(json!({"measure": {"entity": "Project", "measures": [{"fn": "count"}, {"fn": "count"}]}}))).is_err());
        assert!(measure_of(&doc(json!({"measure": {"entity": "Project", "measures": []}}))).is_err());
        assert!(measure_of(&doc(json!({"measure": {"entity": "Project", "group_by": [{"field": "status"}], "measures": [{"fn": "count"}]}}))).is_err());
    }

    #[test]
    fn how_far_a_key_result_has_come() {
        let v = |s: &str| s.parse::<Decimal>().unwrap();
        assert_eq!(progress("Increase", v("100"), v("200"), v("150")), v("50.0"));
        assert_eq!(progress("Increase", v("100"), v("200"), v("250")), v("100.0"), "past the target is done");
        assert_eq!(progress("Increase", v("100"), v("200"), v("90")), v("0.0"), "below the start is nothing");
        assert_eq!(progress("Decrease", v("30"), v("10"), v("25")), v("25.0"));
        assert_eq!((progress("Milestone", v("0"), v("1"), v("0")), progress("Milestone", v("0"), v("1"), v("1"))), (v("0.0"), v("100.0")));
    }

    #[test]
    fn weights_and_grades() {
        let v = |s: &str| s.parse::<rust_decimal::Decimal>().unwrap();
        assert_eq!(weighted(&[(v("100"), v("1")), (v("0"), v("1"))]), v("50.0"), "equal weights are the plain average");
        assert_eq!(weighted(&[(v("100"), v("3")), (v("0"), v("1"))]), v("75.0"));
        assert_eq!(weighted(&[(v("100"), v("0")), (v("20"), v("1"))]), v("20.0"), "a weight of 0 counts for nothing");
        assert_eq!(weighted(&[(v("80"), v("0"))]), v("0"), "nothing weighs");
        assert_eq!(weighted(&[]), v("0"));
        assert_eq!(graded(v("72.5"), "0 to 100", v("40"), v("70")), ("72.5 of 100".to_string(), "Achieved"));
        assert_eq!(graded(v("65"), "0.0 to 1.0", v("40"), v("70")), ("0.65 of 1.0".to_string(), "Progress Made"));
        assert_eq!(graded(v("30"), "0.0 to 1.0", v("40"), v("70")), ("0.3 of 1.0".to_string(), "Missed"));
        assert_eq!(graded(v("100"), "0.0 to 1.0", v("40"), v("70")).0, "1 of 1.0");
        // 1 to 5, evenly; letters by tens from 60.
        assert_eq!(graded(v("0"), "1 to 5", v("40"), v("70")), ("1 of 5".to_string(), "Missed"));
        assert_eq!(graded(v("72.5"), "1 to 5", v("40"), v("70")), ("3.9 of 5".to_string(), "Achieved"));
        assert_eq!(graded(v("100"), "1 to 5", v("40"), v("70")).0, "5 of 5");
        let letters: Vec<String> = ["95", "80", "79.9", "65", "59"].iter().map(|s| graded(v(s), "Letters", v("40"), v("70")).0).collect();
        assert_eq!(letters, ["A", "B", "C", "D", "F"]);
    }
}
