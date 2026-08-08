//! Fixed Макошь platform schema bootstrap.

mod password_file;
mod schemas;

pub use password_file::InitdbPasswordFileV1;
pub use schemas::ensure_platform_schemas;
