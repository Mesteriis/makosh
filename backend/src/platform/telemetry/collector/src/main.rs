//! Telemetry Collector process entrypoint.

mod cli;
mod control;
mod observability;
mod storage;
mod transport;

fn main() {
    if let Err(error) = observability::initialize() {
        eprintln!("Telemetry observability initialization failed: {error}");
        std::process::exit(1);
    }
    tracing::info!(event = "telemetry.process.started");
    if let Err(error) = cli::run() {
        tracing::error!(
            event = "telemetry.process.failed",
            error.class = "telemetry_collector",
            error.message = %error,
        );
        std::process::exit(1);
    }
    tracing::info!(event = "telemetry.process.stopped");
}
