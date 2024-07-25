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
///
/// Level/target are displayed for non-INFO messages.
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
        let metadata = event.metadata();
        if metadata.level() != &Level::INFO {
            write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;
        }
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn setup_logging(verbose: bool) {
    let env_filter = if verbose {
        "nixci=debug,nix_rs=debug,nix_health=info"
    } else {
        "nixci=info,nix_rs=info,nix_health=info"
    };
    let builder = tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(Level::INFO)
        .with_env_filter(env_filter);

    if !verbose {
        builder.event_format(BareFormatter).init();
    } else {
        builder.init()
    }
}
