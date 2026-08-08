//! Isolated filesystem fixtures for parallel telemetry conformance tests.

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY_SUFFIX: AtomicU64 = AtomicU64::new(0);

pub(crate) fn unique_directory(kind: &str) -> std::path::PathBuf {
    let suffix = NEXT_DIRECTORY_SUFFIX.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "makosh-telemetry-{kind}-{}-{suffix}",
        std::process::id(),
    ))
}
