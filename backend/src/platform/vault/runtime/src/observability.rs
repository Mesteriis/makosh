//! Process-local structured diagnostics for the Vault runtime.

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry, filter::filter_fn};

pub(crate) fn initialize() -> Result<(), String> {
    let maximum_level = if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let filter = filter_fn(move |metadata| {
        if metadata.target().starts_with("makosh_") {
            metadata.level() <= &maximum_level
        } else {
            metadata.level() <= &Level::WARN
        }
    });
    let formatter = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .compact()
        .with_filter(filter);
    tracing::subscriber::set_global_default(Registry::default().with(formatter))
        .map_err(|_| "the Vault structured logging subscriber is unavailable".to_owned())
}
