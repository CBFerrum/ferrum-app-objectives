//! Key results measured by a certified metric (Objectives' connector with Metrics,
//! FERRUM-ECOSYSTEM-011) against a real PostgreSQL (`FERRUM_OBJECTIVES_METRICS_TEST_DATABASE_URL`,
//! default `postgres://ferrum:ferrum@localhost:5432/ferrum_objectives_metrics_test`; reset first):
//! a key result naming a certified metric, measured as the metric is defined now with its owner's
//! rights, as it is saved, each day and at each check-in; one naming a metric not certified, or a
//! query of its own as well, refused; and a metric that goes back to draft measuring nothing more,
//! the day's upkeep keeping why.

use chrono::{Duration, Timelike, Utc};
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

fn uid(v: &Value) -> Uuid {
    v.as_str().unwrap_or_else(|| panic!("not an id: {v}")).parse().unwrap()
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
    async fn get(&self, entity: &str, id: Uuid) -> Doc {
        self.k.get(&self.admin, entity, id, false).await.unwrap_or_else(|e| panic!("get {entity}: {e:?}"))
    }
    async fn user(&self, email: &str, roles: &[&str]) -> Ctx {
        let roles: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        let id = self.k.create_user(&self.admin, email, email, Some("Correct-Horse-9"), &roles, false).await.expect("user");
        let s = self.k.login("goals", email, "Correct-Horse-9").await.expect("login");
        self.k.load_ctx(self.tenant, id, Some(s.token_version), "it".into()).await.expect("ctx")
    }
    async fn check_ins(&self, kr: Uuid) -> usize {
        let filters = ferrum_kernel::query::parse_filters(&json!({"key_result": kr.to_string()})).unwrap();
        self.k.list(&self.admin, "CheckIn", &ListQuery { filters, ..Default::default() }).await.unwrap().rows.len()
    }
    /// As if the key result had last been checked in on yesterday: today is its next day.
    async fn next_day(&self, kr: Uuid) {
        let mut tx = self.k.db.tenant_tx(self.tenant).await.unwrap();
        sqlx::query("update e_key_result set last_check_in = last_check_in - 1, measured_on = measured_on - 1 where tenant_id = $1 and id = $2")
            .bind(self.tenant)
            .bind(kr)
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
}

fn refused<V: std::fmt::Debug>(r: Result<V, Error>, kind: &str, says: &str) {
    let msg = format!("{r:?}");
    let ok = matches!((&r, kind), (Err(Error::Invalid(_) | Error::Validation(_)), "invalid") | (Err(Error::Forbidden(_)), "forbidden"));
    assert!(ok && msg.contains(says), "expected {kind} saying {says:?}, got {msg}");
}

#[test]
fn key_results_by_certified_metrics() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(scenario()).await.unwrap() });
}

async fn scenario() {
    // A workspace whose day is never UTC's: a day behind until 11:00 UTC, a day ahead from then.
    let now = Utc::now();
    let (zone, hours) = if now.hour() < 11 { ("Etc/GMT+12", -12) } else { ("Etc/GMT-14", 14) };
    let today = (now + Duration::hours(hours)).date_naive();
    let url = std::env::var("FERRUM_OBJECTIVES_METRICS_TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_objectives_metrics_test".into());
    let db = Db::connect(DbConfig { url, max_connections: 16, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    // Projects' roles reach ERP's records, so ERP comes too.
    packages.extend(ferrum_app_erp::packages().expect("ERP's packages"));
    packages.extend(ferrum_app_projects::packages().expect("Projects' packages"));
    packages.extend(ferrum_app_metrics::packages().expect("Metrics' packages"));
    packages.extend(ferrum_app_objectives::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    assert_eq!(catalog.links_out_of_reach(ferrum_app_objectives::CONNECTOR_PACKAGE), Vec::<String>::new(), "whoever names a metric can look it up");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    ferrum_app_erp::register(&mut hooks);
    ferrum_app_projects::register(&mut hooks);
    ferrum_app_metrics::register(&mut hooks);
    ferrum_app_objectives::register(&mut hooks);
    let settings = KernelSettings { secret_key: Some(SecretKey::new([7u8; 32])), ..Default::default() };
    let k = Kernel::boot(db, catalog, hooks, settings, None).await.expect("boot");
    ferrum_domain::maintenance(&k).await.expect("partitions");
    ferrum_app_erp::maintenance(&k).await.expect("partitions");
    let tenant = k
        .create_tenant(NewTenant {
            slug: "goals".into(),
            name: "Goal Co".into(),
            admin_email: "admin@goals.test".into(),
            admin_password: "Correct-Horse-9".into(),
            admin_name: None,
            packages: vec![ferrum_app_objectives::CONNECTOR_PACKAGE.into()],
            settings: Some(json!({"timezone": zone})),
        })
        .await
        .expect("tenant");
    let session = k.login("goals", "admin@goals.test", "Correct-Horse-9").await.expect("login");
    let admin = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let t = T { k: k.clone(), tenant, admin };
    let a = t.admin.clone();
    let owner = t.user("ceo@goals.test", &["Objectives Owner"]).await;
    // Who defines a metric reads what it counts.
    let author = t.user("analyst@goals.test", &["Metric Author", "Objectives Owner"]).await;
    let steward = t.user("steward@goals.test", &["Data Steward"]).await;

    // ───────────── the work counted, and a metric certified ─────────────
    let currency = ListQuery { filters: ferrum_kernel::query::parse_filters(&json!({"code": "INR"})).unwrap(), ..Default::default() };
    let inr = t.k.list(&a, "Currency", &currency).await.unwrap().rows;
    let company = uid(&t.insert(&a, "Company", json!({"company_name": "Goal Co", "abbr": "GC", "default_currency": inr[0]["id"]})).await["id"]);
    let project = |name: &str| json!({"project_name": name, "company": company.to_string()});
    t.insert(&a, "Project", project("Launch the app")).await;
    let projects = json!({"entity": "Project", "measures": [{"fn": "count"}]});
    let started = uid(&t.insert(&author, "Metric", json!({"metric_name": "Projects started", "definition": projects})).await["id"]);
    let draft = uid(&t.insert(&author, "Metric", json!({"metric_name": "Projects, not yet certified", "definition": projects})).await["id"]);
    t.k.run_action(&steward, "Metric", started, "certify", Value::Null).await.expect("certified by someone who does not own it");

    // ───────────── a key result measured by it ─────────────
    let (q_start, q_end) = (today - Duration::days(30), today + Duration::days(60));
    let goal = uid(&t
        .insert(&owner, "Objective", json!({"title": "Grow", "owner": "ceo@goals.test", "starts_on": q_start.to_string(), "ends_on": q_end.to_string()}))
        .await["id"]);
    let by = |title: &str, extra: Value| {
        let mut kr = json!({"objective": goal.to_string(), "title": title, "kind": "Increase", "start_value": "0", "target_value": "4"});
        kr.as_object_mut().unwrap().extend(obj(extra));
        kr
    };
    refused(t.k.insert(&a, "KeyResult", obj(by("By a draft", json!({"metric": draft.to_string()})))).await, "invalid", "metric");
    refused(
        t.k.insert(&owner, "KeyResult", obj(by("Both", json!({"metric": started.to_string(), "measure": projects})))).await,
        "invalid",
        "the certified metric's",
    );
    let kr = uid(&t.insert(&owner, "KeyResult", by("Projects started", json!({"metric": started.to_string()}))).await["id"]);
    let r = t.get("KeyResult", kr).await;
    assert_eq!((dec(&r["current_value"]), dec(&r["progress"])), (1.0, 25.0), "measured as it is saved: one project");
    assert_eq!(t.check_ins(kr).await, 1);
    // At a check-in, the value is the metric's, not typed.
    t.insert(&a, "Project", project("Open a shop")).await;
    refused(
        t.k.insert(&owner, "CheckIn", obj(json!({"key_result": kr.to_string(), "date": today.to_string(), "value": "3", "confidence": "On Track"}))).await,
        "invalid",
        "is measured from the records",
    );
    let c = t.insert(&owner, "CheckIn", json!({"key_result": kr.to_string(), "date": today.to_string(), "confidence": "At Risk"})).await;
    assert_eq!((dec(&c["value"]), c["measured"].as_bool()), (2.0, Some(true)));
    // Each day by itself, as the metric is defined now.
    t.insert(&a, "Project", project("Open a second shop")).await;
    t.next_day(kr).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    let r = t.get("KeyResult", kr).await;
    assert_eq!((dec(&r["current_value"]), r["measure_error"].as_str()), (3.0, None), "the day's measure: {r:?}");
    assert_eq!(t.check_ins(kr).await, 3);

    // ───────────── a metric changed goes back to draft, and measures nothing more ─────────────
    let defined_again = json!({"entity": "Project", "filters": {"project_name": "Launch the app"}, "measures": [{"fn": "count"}]});
    t.k.update(&author, "Metric", started, obj(json!({"definition": defined_again})), None).await.expect("the metric changed");
    assert_eq!(t.get("Metric", started).await["status"], "Draft");
    t.next_day(kr).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    let r = t.get("KeyResult", kr).await;
    assert!(r["measure_error"].as_str().is_some_and(|e| e.contains("Projects started is not certified now")), "{r:?}");
    assert_eq!((dec(&r["current_value"]), t.check_ins(kr).await), (3.0, 3), "nothing measured");
    refused(
        t.k.insert(&owner, "CheckIn", obj(json!({"key_result": kr.to_string(), "date": today.to_string(), "confidence": "On Track"}))).await,
        "invalid",
        "not certified now",
    );
    // Certified again, it is measured as it is defined now: one project of that name.
    t.k.run_action(&steward, "Metric", started, "certify", Value::Null).await.expect("certified again");
    t.next_day(kr).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    let r = t.get("KeyResult", kr).await;
    assert_eq!((dec(&r["current_value"]), r["measure_error"].as_str()), (1.0, None), "{r:?}");

    // ───────────── metrics converted between currencies, or composed of others, read as Metrics reads them ─────────────
    // A budget of 8000 rupees, in dollars at the inverse of USD→INR 80: 100. Running the metric's query here
    // instead would sum 8000; half of it, a metric composed of the first, has no query to run at all.
    let usd = match t
        .k
        .list(&a, "Currency", &ListQuery { filters: ferrum_kernel::query::parse_filters(&json!({"code": "USD"})).unwrap(), ..Default::default() })
        .await
        .unwrap()
        .rows
        .first()
    {
        Some(c) => uid(&c["id"]),
        None => uid(&t.insert(&a, "Currency", json!({"code": "USD", "currency_name": "US Dollar", "precision": 2})).await["id"]),
    };
    t.insert(&a, "ExchangeRate", json!({"from_currency": usd.to_string(), "to_currency": inr[0]["id"], "rate": "80", "valid_from": "2000-01-01"})).await;
    t.insert(&a, "Project", json!({"project_name": "Budgeted", "company": company.to_string(), "budget": "8000", "start_date": today.to_string()})).await;
    let budget = json!({"entity": "Project", "filters": [["budget", ">", 0]], "measures": [{"fn": "sum", "field": "budget"}]});
    let in_dollars = uid(&t
        .insert(&author, "Metric", json!({"metric_name": "Budget in USD", "definition": budget, "convert_to": usd.to_string(), "currency_field": "currency", "rate_date_field": "start_date"}))
        .await["id"]);
    t.k.run_action(&steward, "Metric", in_dollars, "certify", Value::Null).await.expect("certified");
    let half = uid(&t.insert(&author, "Metric", json!({"metric_name": "Half the budget", "formula": "[Budget in USD] / 2"})).await["id"]);
    t.k.run_action(&steward, "Metric", half, "certify", Value::Null).await.expect("certified after its part");
    let target = |title: &str, metric: Uuid| json!({"objective": goal.to_string(), "title": title, "kind": "Increase", "start_value": "0", "target_value": "400", "metric": metric.to_string()});
    let raised = uid(&t.insert(&owner, "KeyResult", target("Budget raised", in_dollars)).await["id"]);
    let r = t.get("KeyResult", raised).await;
    assert_eq!((dec(&r["current_value"]), dec(&r["progress"])), (100.0, 25.0), "in dollars, as Metrics converts it: {r:?}");
    let halved = uid(&t.insert(&owner, "KeyResult", target("Half raised", half)).await["id"]);
    assert_eq!(dec(&t.get("KeyResult", halved).await["current_value"]), 50.0, "composed, as Metrics works it out");
    let c = t.insert(&owner, "CheckIn", json!({"key_result": raised.to_string(), "date": today.to_string(), "confidence": "On Track"})).await;
    assert_eq!((dec(&c["value"]), c["measured"].as_bool()), (100.0, Some(true)));
    // Without a rate by its day, Metrics gives no value, and the key result says why rather than a sum of rupees.
    t.insert(&a, "Project", json!({"project_name": "Budgeted before rates", "company": company.to_string(), "budget": "10", "start_date": "1999-12-31"})).await;
    t.next_day(raised).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    let r = t.get("KeyResult", raised).await;
    assert!(r["measure_error"].as_str().is_some_and(|e| e.contains("no exchange rate from INR to USD on or before 1999-12-31")), "{r:?}");
    assert_eq!(dec(&r["current_value"]), 100.0, "nothing measured that day");
}
