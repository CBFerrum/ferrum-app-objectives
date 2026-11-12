//! Portfolio and objectives (FERRUM-ECOSYSTEM-011): objectives aligned to their parents, key
//! results measured by check-ins, progress rolled up the tree, and the projects and portfolios
//! behind them (Projects' records).
//!
//! * `okr`: key results' progress from their latest check-in, objectives' progress as the average
//!   of their key results and the objectives aligned to them, alignment that never loops, periods
//!   within the parent's, and closing with a score; key results measured from the records with
//!   their owner's rights (by a query of their own, or, with Metrics, a certified metric they name),
//!   and reminders to check in on the others.
//!
//! This app's own design; no code was copied (docs/PROVENANCE.md). Hooks are registered per entity,
//! so a workspace without the `objectives` package never runs them.

mod okr;

use ferrum_domain::reports::ReportSource;
use ferrum_kernel::{Doc, Error, HookRegistry, Kernel, Result};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

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
        .register("Objective", Arc::new(okr::ObjectiveHook))
        .register("KeyResult", Arc::new(okr::KeyResultHook))
        .register("CheckIn", Arc::new(okr::CheckInHook))
        .register("OkrCycle", Arc::new(okr::CycleHook));
}

/// The connector with Metrics' package.
pub const CONNECTOR_PACKAGE: &str = "objectives_metrics";

/// The manifest of the connector app with Metrics (`connectors/metrics/app.yaml`): its package is
/// `objectives_metrics`, its code this crate's. It ships with Objectives.
pub const CONNECTOR_MANIFEST: &str = include_str!("../connectors/metrics/app.yaml");

/// This app's reports: none of its own yet.
pub fn reports() -> Vec<Arc<dyn ReportSource>> {
    vec![]
}

/// Upkeep, in each workspace with the package: the day's measure of key results measured from the
/// records, and reminders to check in on the others.
pub async fn maintenance(k: &Kernel) -> Result<()> {
    let mut tx = k.db.system_tx().await?;
    let tenants: Vec<Uuid> = sqlx::query_scalar("select id from tenant where status = 'active' order by id").fetch_all(&mut *tx).await?;
    tx.commit().await?;
    let mut first_error: Option<Error> = None;
    for tenant in tenants {
        if let Err(e) = okr::upkeep(k, tenant).await {
            first_error.get_or_insert(e);
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// A document from JSON.
pub(crate) fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_is_a_release_the_platform_takes() {
        let m = ferrum_apps::AppManifest::parse(MANIFEST).expect("app.yaml");
        assert_eq!(m.routes(), vec![("objectives".to_string(), vec![])]);
        let files = UI.iter().map(|(p, b)| (p.to_string(), b.to_vec())).collect();
        let ui = ferrum_apps::ui::prepare("objectives", files).expect("ui/dist is an interface release");
        assert!(UI.iter().any(|(p, _)| *p == "index.html") && !UI.iter().any(|(p, _)| p.starts_with('.')), "{}", ui.digest);
        let connector = ferrum_apps::AppManifest::parse(CONNECTOR_MANIFEST).expect("connectors/metrics/app.yaml");
        assert_eq!(connector.packages, vec![CONNECTOR_PACKAGE.to_string()]);
    }
}
