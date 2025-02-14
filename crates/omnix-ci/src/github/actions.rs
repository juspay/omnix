//! Working with GitHub Actions

use std::future::Future;

/// Group log lines in GitHub Actions
///
/// https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/workflow-commands-for-github-actions#grouping-log-lines
pub async fn in_github_log_group<T, F, Fut>(name: &str, enable: bool, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    if enable {
        eprintln!("::group::{}", name);
    }

    let result = f().await;

    if enable {
        eprintln!("::endgroup::");
    }

    result
}
