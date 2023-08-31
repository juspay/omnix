use crate::report::{Report, WithDetails};
use nix_rs::info;

/// Types that implement health check with reports
pub trait Check {
    /// The type of the report produced by this health check
    type Report = Report<WithDetails>;

    /// Run and create the health check
    fn check(info: &info::NixInfo) -> Self
    where
        Self: Sized;

    /// User-facing name for this health check
    fn name(&self) -> &'static str;

    /// Return the health report for this health check
    fn report(&self) -> Self::Report;
}
