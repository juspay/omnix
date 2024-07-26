//! Logging setup for omnix

use clap_verbosity_flag::{InfoLevel, Level, Verbosity};
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    filter::{Directive, LevelFilter},
    fmt::{format, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
    EnvFilter,
};

pub fn setup_logging(verbosity: &Verbosity<InfoLevel>, bare: bool) {
    let builder = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(log_filter(verbosity))
        .compact();
    if bare {
        builder.event_format(BareFormatter).init();
    } else {
        builder.init();
    }
}

/// Return the log filter for CLI flag.
fn log_filter(v: &Verbosity<InfoLevel>) -> EnvFilter {
    log_directives(v)
        .iter()
        .fold(EnvFilter::from_env("OMNIX_LOG"), |filter, directive| {
            filter.add_directive(directive.clone())
        })
}

fn log_directives(v: &Verbosity<InfoLevel>) -> Vec<Directive> {
    // Allow warnings+errors from all crates.
    match v.log_level() {
        None => vec![LevelFilter::WARN.into()],
        Some(Level::Warn) => vec![LevelFilter::WARN.into()],
        Some(Level::Error) => vec![LevelFilter::ERROR.into()],
        // Default
        Some(Level::Info) => vec![
            LevelFilter::WARN.into(),
            "omnix=info".parse().unwrap(),
            "nix_rs=info".parse().unwrap(),
            "nix_health=info".parse().unwrap(),
        ],
        // -v: log app DEBUG level, as well as http requests
        Some(Level::Debug) => vec![
            LevelFilter::WARN.into(),
            "omnix=debug".parse().unwrap(),
            "nix_rs=debug".parse().unwrap(),
            "nix_health=debug".parse().unwrap(),
        ],
        // -vv: log app TRACE level, as well as http requests
        Some(Level::Trace) => vec![
            LevelFilter::WARN.into(),
            "omnix=trace".parse().unwrap(),
            "nix_rs=trace".parse().unwrap(),
            "nix_health=trace".parse().unwrap(),
        ],
        // -vvv: log DEBUG level of app and libraries
        // 3 => vec![LevelFilter::DEBUG.into()],
        // -vvvv: log TRACE level of app and libraries
        // _ => vec![LevelFilter::TRACE.into()],
    }
}

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
        let metadata = event.metadata();
        if metadata.level() != &tracing::Level::INFO {
            write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;
        }
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
