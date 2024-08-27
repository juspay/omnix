use predicates::{prelude::*, str::contains};

use super::core::om;

/// `om health` runs, and succeeds.
#[test]
fn om_health() -> anyhow::Result<()> {
    om()?
        .arg("health")
        .assert()
        .success()
        .stderr(contains("All checks passed").or(contains("Required checks passed")));
    Ok(())
}
