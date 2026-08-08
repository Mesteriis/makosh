use std::net::IpAddr;
use url::Url;

const COOKIE_NAME: &str = "__Host-makosh-session";

pub struct BrowserSameOriginSessionV1;

impl BrowserSameOriginSessionV1 {
    pub fn issue_cookie(session_id: &str) -> Result<String, String> {
        valid_session_id(session_id)
            .then(|| {
                format!("{COOKIE_NAME}={session_id}; Path=/; Secure; HttpOnly; SameSite=Strict")
            })
            .ok_or_else(|| "browser session cookie is invalid".to_owned())
    }

    pub fn session_id_from_cookie(cookie_header: &str) -> Result<String, String> {
        let mut session_id = None;
        for item in cookie_header.split(';') {
            let Some((name, value)) = item.trim().split_once('=') else {
                continue;
            };
            if name == COOKIE_NAME
                && (session_id.replace(value).is_some() || !valid_session_id(value))
            {
                return Err("browser session cookie is invalid".to_owned());
            }
        }
        session_id
            .map(str::to_owned)
            .ok_or_else(|| "browser session cookie is unavailable".to_owned())
    }

    pub fn require_mutation_origin(origin: &str, exact_https_origin: &str) -> Result<(), String> {
        (valid_exact_https_origin(exact_https_origin)
            && origin == exact_https_origin
            && valid_exact_https_origin(origin))
        .then_some(())
        .ok_or_else(|| "browser mutation origin is invalid".to_owned())
    }

    pub fn require_lan_development_origin(origin: &str, exact_origin: &str) -> Result<(), String> {
        (origin == exact_origin && valid_exact_private_lan_http_origin(origin))
            .then_some(())
            .ok_or_else(|| "browser development origin is invalid".to_owned())
    }

    pub fn require_loopback_development_origin(
        origin: &str,
        exact_origin: &str,
    ) -> Result<(), String> {
        (origin == exact_origin && valid_exact_loopback_http_origin(origin))
            .then_some(())
            .ok_or_else(|| "browser loopback development origin is invalid".to_owned())
    }
}

fn valid_exact_private_lan_http_origin(value: &str) -> bool {
    let Ok(origin) = Url::parse(value) else {
        return false;
    };
    let private_ip = origin
        .host_str()
        .and_then(|host| host.parse::<IpAddr>().ok())
        .is_some_and(|ip| match ip {
            IpAddr::V4(ip) => ip.is_private() || ip.is_link_local(),
            IpAddr::V6(ip) => {
                let first = ip.segments()[0];
                (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
            }
        });
    origin.scheme() == "http"
        && private_ip
        && origin.port().is_some()
        && origin.username().is_empty()
        && origin.password().is_none()
        && origin.path() == "/"
        && origin.query().is_none()
        && origin.fragment().is_none()
        && origin
            .as_str()
            .strip_suffix('/')
            .is_some_and(|raw| raw == value)
}

fn valid_exact_loopback_http_origin(value: &str) -> bool {
    let Ok(origin) = Url::parse(value) else {
        return false;
    };
    origin.scheme() == "http"
        && origin.host_str() == Some("127.0.0.1")
        && origin.port().is_some()
        && origin.username().is_empty()
        && origin.password().is_none()
        && origin.path() == "/"
        && origin.query().is_none()
        && origin.fragment().is_none()
        && origin
            .as_str()
            .strip_suffix('/')
            .is_some_and(|raw| raw == value)
}

fn valid_session_id(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_exact_https_origin(value: &str) -> bool {
    let Ok(origin) = Url::parse(value) else {
        return false;
    };
    origin.scheme() == "https"
        && origin.username().is_empty()
        && origin.password().is_none()
        && origin.path() == "/"
        && origin.query().is_none()
        && origin.fragment().is_none()
        && origin
            .as_str()
            .strip_suffix('/')
            .is_some_and(|raw| raw == value)
}
