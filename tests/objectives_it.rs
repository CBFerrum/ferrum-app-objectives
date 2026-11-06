//! A quarter of objectives (FERRUM-ECOSYSTEM-011) against a real PostgreSQL
//! (`FERRUM_OBJECTIVES_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_objectives_test`;
//! reset first), with Projects installed: a company objective with team objectives aligned under
//! it, inside its period and never in a loop; key results that increase, decrease or are done,
//! moved by their latest check-in by day; progress rolled up the tree; check-ins only within the
//! period and not ahead of today; a closed objective keeping its score and taking nothing more;
//! the projects and portfolio behind them; key results measured from the records with their
//! owner's rights, each day and at each check-in; reminders to check in; and who may do what.

use chrono::{Duration, NaiveDate, Timelike, Utc};
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
    async fn progress(&self, entity: &str, id: Uuid) -> f64 {
        dec(&self.get(entity, id).await["progress"])
    }
    async fn check_ins(&self, kr: Uuid) -> Vec<Doc> {
        let filters = ferrum_kernel::query::parse_filters(&json!({"key_result": kr.to_string()})).unwrap();
        self.k.list(&self.admin, "CheckIn", &ListQuery { filters, ..Default::default() }).await.unwrap().rows
    }
    /// As if the key result's last check-in were the day before `today`: the next day, for its upkeep.
    async fn next_day(&self, kr: Uuid, today: NaiveDate) {
        let yesterday = (today - Duration::days(1)).to_string();
        let mut tx = self.k.db.tenant_tx(self.tenant).await.unwrap();
        let system = ferrum_domain::util::elevated(&self.admin);
        self.k.set_values_tx(&system, &mut tx, "KeyResult", kr, obj(json!({"last_check_in": yesterday}))).await.unwrap();
        tx.commit().await.unwrap();
    }
    /// The reminders to check in on a key result.
    async fn reminders(&self, kr: Uuid) -> i64 {
        let mut tx = self.k.db.tenant_tx(self.tenant).await.unwrap();
        sqlx::query_scalar("select count(*) from notification where tenant_id = $1 and record_id = $2 and kind = 'objectives.check_in'")
            .bind(self.tenant)
            .bind(kr)
            .fetch_one(&mut *tx)
            .await
            .unwrap()
    }
}

fn refused<V: std::fmt::Debug>(r: Result<V, Error>, kind: &str, says: &str) {
    let msg = format!("{r:?}");
    let ok = matches!(
        (&r, kind),
        (Err(Error::Invalid(_) | Error::Validation(_)), "invalid") | (Err(Error::Forbidden(_)), "forbidden") | (Err(Error::NotFound(_)), "not found")
    );
    assert!(ok && msg.contains(says), "expected {kind} saying {says:?}, got {msg}");
}

#[test]
fn a_quarter_of_objectives() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(scenario()).await.unwrap() });
}

async fn scenario() {
    // A workspace whose day is not UTC's, whatever the hour, and an hour or more from its midnight:
    // a day behind until 11:00 UTC, a day ahead from then. Its dates are its own.
    let now = Utc::now();
    let (zone, hours) = if now.hour() < 11 { ("Etc/GMT+12", -12) } else { ("Etc/GMT-14", 14) };
    let today = (now + Duration::hours(hours)).date_naive();
    let url = std::env::var("FERRUM_OBJECTIVES_TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_objectives_test".into());
    let db = Db::connect(DbConfig { url, max_connections: 16, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    // Projects' roles reach ERP's records, so ERP comes too.
    packages.extend(ferrum_app_erp::packages().expect("ERP's packages"));
    packages.extend(ferrum_app_projects::packages().expect("Projects' packages"));
    packages.extend(ferrum_app_objectives::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    ferrum_app_erp::register(&mut hooks);
    ferrum_app_projects::register(&mut hooks);
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
            packages: vec!["objectives".into()],
            settings: Some(json!({"timezone": zone})),
        })
        .await
        .expect("tenant");
    let session = k.login("goals", "admin@goals.test", "Correct-Horse-9").await.expect("login");
    let admin = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let t = T { k: k.clone(), tenant, admin };
    let a = t.admin.clone();
    let owner = t.user("ceo@goals.test", &["Objectives Owner"]).await;
    let viewer = t.user("staff@goals.test", &["Objectives Viewer"]).await;
    let (q_start, q_end) = (today - Duration::days(30), today + Duration::days(60));

    // ───────────── the work behind it ─────────────
    let inr =
        t.k.list(&t.admin, "Currency", &ListQuery { filters: ferrum_kernel::query::parse_filters(&json!({"code": "INR"})).unwrap(), ..Default::default() })
            .await
            .unwrap()
            .rows;
    let company = uid(&t.insert(&a, "Company", json!({"company_name": "Goal Co", "abbr": "GC", "default_currency": inr[0]["id"]})).await["id"]);
    let launch = uid(&t.insert(&a, "Project", json!({"project_name": "Launch the app", "company": company.to_string()})).await["id"]);
    let portfolio = uid(&t.insert(&a, "Portfolio", json!({"portfolio_name": "Growth", "projects": [{"project": launch.to_string()}]})).await["id"]);

    // ───────────── objectives ─────────────
    let objective = |title: &str, parent: Option<Uuid>, from: chrono::NaiveDate, to: chrono::NaiveDate| json!({"title": title, "owner": "ceo@goals.test", "parent": parent.map(|p| p.to_string()), "portfolio": portfolio.to_string(), "starts_on": from.to_string(), "ends_on": to.to_string()});
    refused(t.k.insert(&owner, "Objective", obj(objective("Backwards", None, q_end, q_start))).await, "invalid", "after the objective starts");
    let grow = uid(&t.insert(&owner, "Objective", objective("Grow the business", None, q_start, q_end)).await["id"]);
    let mut unnamed = objective("Whose is it", None, q_start, q_end);
    unnamed.as_object_mut().unwrap().remove("owner");
    let whose = t.insert(&owner, "Objective", unnamed).await;
    assert_eq!(whose["owner"], "ceo@goals.test", "whoever records it, unless they name someone");
    t.k.delete(&t.admin, "Objective", uid(&whose["id"]), None).await.unwrap();
    refused(
        t.k.insert(&owner, "Objective", obj(objective("Too long", Some(grow), q_start, q_end + Duration::days(1)))).await,
        "invalid",
        "falls within the objective it is aligned to",
    );
    let sales = uid(&t.insert(&owner, "Objective", objective("Win customers", Some(grow), q_start, q_end)).await["id"]);
    let ops = uid(&t.insert(&owner, "Objective", objective("Run lean", Some(grow), q_start, q_end)).await["id"]);
    refused(t.k.update(&owner, "Objective", grow, obj(json!({"parent": sales.to_string()})), None).await, "invalid", "cycle");
    refused(t.k.update(&owner, "Objective", grow, obj(json!({"parent": grow.to_string()})), None).await, "invalid", "");

    // ───────────── key results ─────────────
    let kr = |objective: Uuid, title: &str, kind: &str, start: &str, target: &str| json!({"objective": objective.to_string(), "title": title, "kind": kind, "start_value": start, "target_value": target, "projects": [{"project": launch.to_string()}]});
    refused(t.k.insert(&owner, "KeyResult", obj(kr(sales, "Fewer", "Increase", "100", "50"))).await, "invalid", "aims above where it starts");
    refused(t.k.insert(&owner, "KeyResult", obj(kr(ops, "More", "Decrease", "10", "20"))).await, "invalid", "aims below where it starts");
    let customers = t.insert(&owner, "KeyResult", kr(sales, "Customers", "Increase", "100", "200")).await;
    assert_eq!((dec(&customers["current_value"]), dec(&customers["progress"])), (100.0, 0.0), "it starts where it starts");
    let customers = uid(&customers["id"]);
    let cost = uid(&t.insert(&owner, "KeyResult", kr(ops, "Cost per order", "Decrease", "40", "20")).await["id"]);
    let ship = uid(&t.insert(&owner, "KeyResult", kr(ops, "Ship the warehouse move", "Milestone", "0", "1")).await["id"]);

    // ───────────── check-ins ─────────────
    let check = |kr: Uuid, on: chrono::NaiveDate, value: &str, confidence: &str| json!({"key_result": kr.to_string(), "date": on.to_string(), "value": value, "confidence": confidence});
    refused(t.k.insert(&owner, "CheckIn", obj(check(customers, today + Duration::days(1), "150", "On Track"))).await, "invalid", "today or before");
    refused(
        t.k.insert(&owner, "CheckIn", obj(check(customers, q_start - Duration::days(1), "150", "On Track"))).await,
        "invalid",
        "falls within the objective's period",
    );
    let c1 = t.insert(&owner, "CheckIn", check(customers, today - Duration::days(7), "150", "At Risk")).await;
    assert_eq!(c1["author"], "ceo@goals.test");
    let r = t.get("KeyResult", customers).await;
    assert_eq!((dec(&r["current_value"]), dec(&r["progress"]), r["confidence"].as_str()), (150.0, 50.0, Some("At Risk")));
    // A check-in dated earlier, entered later, is not the latest.
    t.insert(&owner, "CheckIn", check(customers, today - Duration::days(14), "120", "Off Track")).await;
    assert_eq!(t.progress("KeyResult", customers).await, 50.0, "the latest by day stands");
    t.insert(&owner, "CheckIn", check(customers, today, "250", "On Track")).await;
    assert_eq!(t.progress("KeyResult", customers).await, 100.0, "past the target is done");
    t.insert(&owner, "CheckIn", check(cost, today, "30", "On Track")).await;
    assert_eq!(t.progress("KeyResult", cost).await, 50.0, "from 40 down to 30 of the way to 20");
    assert_eq!(t.progress("Objective", sales).await, 100.0);
    assert_eq!(t.progress("Objective", ops).await, 25.0, "50 and a milestone not yet done");
    assert_eq!(t.progress("Objective", grow).await, 62.5, "its two objectives, on average");
    t.insert(&owner, "CheckIn", check(ship, today, "1", "On Track")).await;
    assert_eq!((t.progress("Objective", ops).await, t.progress("Objective", grow).await), (75.0, 87.5), "rolled up the tree");
    // Moved under another, an objective takes its progress with it.
    let other = uid(&t.insert(&owner, "Objective", objective("Delight people", None, q_start, q_end)).await["id"]);
    t.k.update(&owner, "Objective", ops, obj(json!({"parent": other.to_string()})), None).await.unwrap();
    assert_eq!((t.progress("Objective", grow).await, t.progress("Objective", other).await), (100.0, 75.0));

    // ───────────── closing ─────────────
    t.k.run_action(&owner, "Objective", sales, "close_objective", Value::Null).await.expect("closed");
    let closed = t.get("Objective", sales).await;
    assert_eq!((closed["status"].as_str(), dec(&closed["score"])), (Some("Closed"), 100.0));
    refused(t.k.insert(&owner, "CheckIn", obj(check(customers, today, "260", "On Track"))).await, "invalid", "takes no more check-ins");
    refused(t.k.insert(&owner, "KeyResult", obj(kr(sales, "Late one", "Increase", "0", "1"))).await, "invalid", "takes no more key results");
    refused(t.k.update(&owner, "Objective", sales, obj(json!({"title": "Renamed"})), None).await, "invalid", "kept as it closed");

    // ───────────── the portfolio behind them ─────────────
    let in_growth =
        t.k.list(
            &t.admin,
            "Objective",
            &ListQuery { filters: ferrum_kernel::query::parse_filters(&json!({"portfolio": portfolio.to_string()})).unwrap(), ..Default::default() },
        )
        .await
        .unwrap()
        .rows;
    assert_eq!(in_growth.len(), 4);
    assert_eq!(t.get("KeyResult", customers).await["projects"][0]["project"], launch.to_string());

    // ───────────── measured from the records ─────────────
    let projects = json!({"entity": "Project", "measures": [{"fn": "count"}]});
    let measured = |title: &str, measure: &Value| json!({"objective": other.to_string(), "title": title, "kind": "Increase", "start_value": "0", "target_value": "4", "measure": measure});
    refused(
        t.k.insert(&owner, "KeyResult", obj(measured("Two numbers", &json!({"entity": "Project", "measures": [{"fn": "count"}, {"fn": "count"}]})))).await,
        "invalid",
        "one measure, ungrouped",
    );
    refused(
        t.k.insert(&owner, "KeyResult", obj(measured("Of nothing", &json!({"entity": "Nothing", "measures": [{"fn": "count"}]})))).await,
        "invalid",
        "does not run",
    );
    let started = t.insert(&owner, "KeyResult", measured("Projects started", &projects)).await;
    assert_eq!(started["owner"], "ceo@goals.test", "its objective's owner, unless someone is named");
    let started = uid(&started["id"]);
    let r = t.get("KeyResult", started).await;
    assert_eq!((dec(&r["current_value"]), dec(&r["progress"])), (1.0, 25.0), "measured as it is saved: one project so far");
    let first = t.check_ins(started).await;
    assert_eq!(
        (first.len(), first[0]["measured"].as_bool(), first[0]["author"].as_str()),
        (1, Some(true), Some("ceo@goals.test")),
        "its owner's rights read it"
    );
    // Someone checking in says how confident they are; the value is measured, not typed.
    t.insert(&a, "Project", json!({"project_name": "Open a second shop", "company": company.to_string()})).await;
    refused(t.k.insert(&owner, "CheckIn", obj(check(started, today, "3", "On Track"))).await, "invalid", "is measured from the records");
    let c = t.insert(&owner, "CheckIn", json!({"key_result": started.to_string(), "date": today.to_string(), "confidence": "At Risk", "note": "Slow"})).await;
    assert_eq!((dec(&c["value"]), c["measured"].as_bool()), (2.0, Some(true)));
    let r = t.get("KeyResult", started).await;
    assert_eq!((dec(&r["progress"]), r["confidence"].as_str(), r["last_check_in"].as_str()), (50.0, Some("At Risk"), Some(today.to_string().as_str())));
    let edited = t.k.update(&owner, "CheckIn", uid(&c["id"]), obj(json!({"note": "Faster now"})), None).await.expect("a kept check-in's note");
    assert_eq!(dec(&edited["value"]), 2.0, "kept as it was measured");
    refused(
        t.k.insert(&owner, "CheckIn", obj(json!({"key_result": cost.to_string(), "date": today.to_string(), "confidence": "On Track"}))).await,
        "invalid",
        "is required",
    );
    // Its owner's rights read it: someone who cannot read projects cannot own it.
    let mut theirs = measured("Staff's count", &projects);
    theirs["owner"] = json!("staff@goals.test");
    refused(t.k.insert(&owner, "KeyResult", obj(theirs)).await, "invalid", "its owner cannot read it");
    // Each day by itself, once, its confidence the last check-in's.
    t.insert(&a, "Project", json!({"project_name": "Open a third shop", "company": company.to_string()})).await;
    t.next_day(started, today).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    let r = t.get("KeyResult", started).await;
    assert_eq!((dec(&r["current_value"]), dec(&r["progress"]), r["confidence"].as_str()), (3.0, 75.0, Some("At Risk")), "the day's measure");
    assert_eq!(t.check_ins(started).await.len(), 3);
    t.next_day(started, today).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    assert_eq!(t.check_ins(started).await.len(), 3, "tried once a day");
    // One that cannot be measured keeps why, and leaves the others measured.
    let analyst = t.user("analyst@goals.test", &["Objectives Owner"]).await;
    let mut theirs = measured("Analyst's count", &projects);
    theirs["owner"] = json!("analyst@goals.test");
    let counted = uid(&t.insert(&owner, "KeyResult", theirs).await["id"]);
    t.k.set_user_active(&t.admin, analyst.user, false).await.expect("deactivated");
    t.next_day(counted, today).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    let r = t.get("KeyResult", counted).await;
    assert!(r["measure_error"].as_str().is_some_and(|e| e.contains("not an active person here")), "{r:?}");
    assert_eq!((r["measured_on"].as_str(), t.check_ins(counted).await.len()), (Some(today.to_string().as_str()), 1));

    // ───────────── reminders to check in ─────────────
    let survey = |title: &str, every: i64| json!({"objective": other.to_string(), "title": title, "kind": "Increase", "start_value": "0", "target_value": "10", "check_in_every": every});
    let nps = uid(&t.insert(&owner, "KeyResult", survey("Promoter score", 7)).await["id"]);
    let quiet = uid(&t.insert(&owner, "KeyResult", survey("Never reminded", 0)).await["id"]);
    let fresh = uid(&t.insert(&owner, "KeyResult", survey("Just set", 7)).await["id"]);
    t.insert(&owner, "CheckIn", check(nps, today - Duration::days(8), "4", "On Track")).await;
    t.insert(&owner, "CheckIn", check(quiet, today - Duration::days(30), "4", "On Track")).await;
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    assert_eq!(t.reminders(nps).await, 1, "eight days since its last check-in, of every seven");
    assert_eq!(t.get("KeyResult", nps).await["reminded_on"].as_str(), Some(today.to_string().as_str()));
    assert_eq!((t.reminders(quiet).await, t.reminders(fresh).await, t.reminders(started).await), (0, 0, 0), "never asked, not yet due, measured");
    ferrum_app_objectives::maintenance(&t.k).await.expect("upkeep");
    assert_eq!(t.reminders(nps).await, 1, "once until another check-in is due");
    t.insert(&owner, "CheckIn", check(nps, today, "5", "On Track")).await;
    assert_eq!(t.get("KeyResult", nps).await["last_check_in"].as_str(), Some(today.to_string().as_str()));

    // ───────────── who may do what ─────────────
    refused(t.k.insert(&viewer, "Objective", obj(objective("Mine", None, q_start, q_end))).await, "forbidden", "");
    refused(t.k.insert(&viewer, "CheckIn", obj(check(cost, today, "25", "On Track"))).await, "forbidden", "");
    assert_eq!(t.k.list(&viewer, "KeyResult", &ListQuery::default()).await.unwrap().rows.len(), 8, "the viewer follows them");
}
