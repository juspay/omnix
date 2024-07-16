use std::fmt;
use std::io;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format;
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

/// A [tracing_subscriber] event formatter that suppresses everything but the
/// log message.
struct BareFormatter;

impl<S, N> FormatEvent<S, N> for BareFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn setup_logging(quiet: bool) {
    let env_filter = if quiet {
        "nix_health=warn,nix_rs=error"
    } else {
        "nix_health=info,nix_rs=error"
    };
    let builder = tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(Level::INFO)
        .with_env_filter(env_filter);

    builder.event_format(BareFormatter).init();
}
