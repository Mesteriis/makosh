//! Exact inherited descriptor for a process-specific platform-control channel.

use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::net::UnixStream;

pub const PLATFORM_CONTROL_INHERITED_FD_V1: RawFd = 3;
pub const PLATFORM_CONTROL_INHERITED_FD_ENV_V1: &str = "MAKOSH_PLATFORM_CONTROL_FD";

pub fn open_inherited_platform_control_v1() -> Result<UnixStream, PlatformControlFdErrorV1> {
    let value = std::env::var(PLATFORM_CONTROL_INHERITED_FD_ENV_V1)
        .map_err(|_| PlatformControlFdErrorV1::Missing)?;
    open_inherited_platform_control_from_value_v1(&value)
}

pub fn open_inherited_platform_control_from_value_v1(
    value: &str,
) -> Result<UnixStream, PlatformControlFdErrorV1> {
    if value != PLATFORM_CONTROL_INHERITED_FD_V1.to_string() {
        return Err(PlatformControlFdErrorV1::InvalidDescriptor);
    }
    // Kernel owns this descriptor and has already duplicated the exact child
    // end into FD 3 immediately before exec. The child takes ownership once.
    Ok(unsafe { UnixStream::from_raw_fd(PLATFORM_CONTROL_INHERITED_FD_V1) })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformControlFdErrorV1 {
    Missing,
    InvalidDescriptor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_every_descriptor_except_the_exact_fixed_fd() {
        assert!(matches!(
            open_inherited_platform_control_from_value_v1("4"),
            Err(PlatformControlFdErrorV1::InvalidDescriptor)
        ));
    }
}
