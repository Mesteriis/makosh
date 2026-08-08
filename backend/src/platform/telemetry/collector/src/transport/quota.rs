//! Fixed per-source telemetry admission budget.

use std::collections::BTreeMap;

use makosh_telemetry_protocol::{TelemetryPriorityV1, TelemetrySignalV1};

const MAX_SOURCES: usize = 64;
const WINDOW_MILLIS: i64 = 1_000;
const NORMAL_LIMIT: u16 = 100;
const ERROR_RESERVE: u16 = 10;

#[derive(Default)]
pub struct TelemetryQuotaV1 {
    sources: BTreeMap<String, SourceWindow>,
}

#[derive(Clone, Copy, Default)]
struct SourceWindow {
    opened_at_millis: i64,
    normal: u16,
    reserve: u16,
}

impl TelemetryQuotaV1 {
    pub fn admit(&mut self, signal: &TelemetrySignalV1) -> bool {
        let source = signal.source().runtime_id().to_owned();
        if !self.sources.contains_key(&source) && self.sources.len() == MAX_SOURCES {
            return false;
        }
        let window = self.sources.entry(source).or_default();
        if signal
            .observed_at_utc_millis()
            .saturating_sub(window.opened_at_millis)
            >= WINDOW_MILLIS
        {
            *window = SourceWindow {
                opened_at_millis: signal.observed_at_utc_millis(),
                ..SourceWindow::default()
            };
        }
        if matches!(
            signal.priority(),
            TelemetryPriorityV1::Error | TelemetryPriorityV1::Crash
        ) {
            if window.reserve == ERROR_RESERVE {
                return false;
            }
            window.reserve += 1;
            return true;
        }
        if window.normal == NORMAL_LIMIT {
            return false;
        }
        window.normal += 1;
        true
    }
}
