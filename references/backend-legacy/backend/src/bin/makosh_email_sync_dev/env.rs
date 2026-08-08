use std::env;

use crate::errors::DevEmailSyncError;

pub(super) fn first_env<const N: usize>(
    names: [&'static str; N],
) -> Result<String, DevEmailSyncError> {
    for name in names {
        if let Some(value) = optional_env(name) {
            return Ok(value);
        }
    }
    Err(DevEmailSyncError::MissingEnv(names.join(" or ")))
}

pub(super) fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub(super) fn parse_port(name: &'static str, value: &str) -> Result<u16, DevEmailSyncError> {
    let port = parse_u16(name, value)?;
    if port == 0 {
        return Err(DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(port)
}

pub(super) fn parse_bool(name: &'static str, value: &str) -> Result<bool, DevEmailSyncError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

pub(super) fn parse_usize(name: &'static str, value: &str) -> Result<usize, DevEmailSyncError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected positive integer",
        })?;
    if parsed == 0 {
        return Err(DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(parsed)
}

fn parse_u16(name: &'static str, value: &str) -> Result<u16, DevEmailSyncError> {
    value
        .parse::<u16>()
        .map_err(|_| DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u16 integer",
        })
}
