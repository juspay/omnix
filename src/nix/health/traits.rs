use leptos::*;

use super::{
    info,
    report::{Report, WithDetails},
    system,
};

/// Types that implement health check with reports
///
/// Check types can be rendered into views using the [IntoView] trait.
pub trait Check: IntoView {
    /// The type of the report produced by this health check
    type Report = Report<WithDetails>;

    /// Run and create the health check
    fn check(nix_info: &info::NixInfo, sys_info: &system::SysInfo) -> Self
    where
        Self: Sized;

    /// User-facing name for this health check
    fn name(&self) -> &'static str;

    /// Return the health report for this health check
    fn report(&self) -> Self::Report;
}
