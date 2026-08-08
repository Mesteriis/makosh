use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const PROVIDER_SHAPE: &str = "yandex_telemost_user";
const RUNTIME_KIND: &str = "yandex_telemost_webview_runtime";
const WINDOW_LABEL_PREFIX: &str = "yandex-telemost";
const TELEMOST_ALLOWED_HOST_RU: &str = "telemost.yandex.ru";
const TELEMOST_ALLOWED_HOST_COM: &str = "telemost.yandex.com";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct YandexTelemostCompanionOpenRequest {
    pub(crate) account_id: String,
    pub(crate) join_url: String,
    #[serde(default)]
    pub(crate) conference_id: Option<String>,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct YandexTelemostCompanionManifest {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) provider_shape: &'static str,
    pub(crate) runtime_kind: &'static str,
    pub(crate) window_label: String,
    pub(crate) opened_window: bool,
    pub(crate) focused_existing_window: bool,
    pub(crate) owner_visible: bool,
    pub(crate) hidden_headless_mode: &'static str,
    pub(crate) allowed_hosts: Vec<&'static str>,
}

#[tauri::command]
pub(crate) async fn yandex_telemost_companion_manifest(
    request: YandexTelemostCompanionOpenRequest,
) -> Result<YandexTelemostCompanionManifest, String> {
    validate_join_url(&request.join_url)?;
    let label = companion_window_label(&request.account_id, request.conference_id.as_deref())?;
    Ok(manifest_for_request(request, label, false, false))
}

#[tauri::command]
pub(crate) async fn open_yandex_telemost_companion(
    app: AppHandle,
    request: YandexTelemostCompanionOpenRequest,
) -> Result<YandexTelemostCompanionManifest, String> {
    validate_join_url(&request.join_url)?;
    let window_label =
        companion_window_label(&request.account_id, request.conference_id.as_deref())?;
    if let Some(window) = app.get_webview_window(&window_label) {
        window
            .show()
            .map_err(|error| format!("failed to show Telemost window: {error}"))?;
        window
            .set_focus()
            .map_err(|error| format!("failed to focus Telemost window: {error}"))?;
        return Ok(manifest_for_request(request, window_label, false, true));
    }

    let url = request
        .join_url
        .parse()
        .map_err(|error| format!("invalid Yandex Telemost join URL: {error}"))?;
    let initialization_script = telemost_initialization_script(&request, &window_label)?;
    let window = WebviewWindowBuilder::new(&app, window_label.clone(), WebviewUrl::External(url))
        .title("Yandex Telemost · Макошь")
        .visible(true)
        .resizable(true)
        .inner_size(1220.0, 820.0)
        .initialization_script(initialization_script)
        .on_navigation(|url| {
            url.scheme() == "https"
                && matches!(
                    url.host_str(),
                    Some(TELEMOST_ALLOWED_HOST_RU) | Some(TELEMOST_ALLOWED_HOST_COM)
                )
        })
        .build()
        .map_err(|error| format!("failed to open Yandex Telemost window: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus Telemost window: {error}"))?;

    Ok(manifest_for_request(request, window_label, true, false))
}

fn manifest_for_request(
    request: YandexTelemostCompanionOpenRequest,
    window_label: String,
    opened_window: bool,
    focused_existing_window: bool,
) -> YandexTelemostCompanionManifest {
    YandexTelemostCompanionManifest {
        account_id: request.account_id,
        conference_id: request.conference_id,
        join_url: request.join_url,
        provider_shape: PROVIDER_SHAPE,
        runtime_kind: RUNTIME_KIND,
        window_label,
        opened_window,
        focused_existing_window,
        owner_visible: true,
        hidden_headless_mode: "forbidden",
        allowed_hosts: vec![TELEMOST_ALLOWED_HOST_RU, TELEMOST_ALLOWED_HOST_COM],
    }
}

fn telemost_initialization_script(
    request: &YandexTelemostCompanionOpenRequest,
    window_label: &str,
) -> Result<String, String> {
    let account_id =
        serde_json::to_string(&request.account_id).map_err(|error| error.to_string())?;
    let conference_id =
        serde_json::to_string(&request.conference_id).map_err(|error| error.to_string())?;
    let window_label = serde_json::to_string(window_label).map_err(|error| error.to_string())?;
    Ok(format!(
        r#"
(() => {{
  const ALLOWED = new Set(['{TELEMOST_ALLOWED_HOST_RU}', '{TELEMOST_ALLOWED_HOST_COM}']);
  if (!ALLOWED.has(window.location.hostname)) return;
  window.__MAKOSH_YANDEX_TELEMOST_COMPANION__ = {{
    accountId: {account_id},
    conferenceId: {conference_id},
    providerShape: '{PROVIDER_SHAPE}',
    runtimeKind: '{RUNTIME_KIND}',
    windowLabel: {window_label}
  }};
}})();
"#
    ))
}

fn companion_window_label(account_id: &str, conference_id: Option<&str>) -> Result<String, String> {
    let account = sanitize_slug(required_slug("account_id", account_id)?);
    if account.is_empty() {
        return Err("account_id must contain at least one slug-safe character".to_owned());
    }
    let conference = conference_id
        .map(sanitize_slug)
        .filter(|value| !value.is_empty());
    Ok(match conference {
        Some(conference) => format!("{WINDOW_LABEL_PREFIX}-{account}-{conference}"),
        None => format!("{WINDOW_LABEL_PREFIX}-{account}"),
    })
}

fn validate_join_url(value: &str) -> Result<(), String> {
    if !value.starts_with("https://") {
        return Err("Yandex Telemost join URL must be HTTPS".to_owned());
    }
    let host = value
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();
    if matches!(host, TELEMOST_ALLOWED_HOST_RU | TELEMOST_ALLOWED_HOST_COM) {
        Ok(())
    } else {
        Err(format!(
            "unsupported Yandex Telemost join URL host `{host}`"
        ))
    }
}

fn required_slug<'a>(field: &'static str, value: &'a str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}

fn sanitize_slug(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_telemost_urls() {
        assert!(validate_join_url("https://example.com/room").is_err());
        assert!(validate_join_url("https://telemost.yandex.ru/j/123").is_ok());
    }

    #[test]
    fn labels_are_stable_and_slugged() {
        assert_eq!(
            companion_window_label("Main Account", Some("Conf 1")).unwrap(),
            "yandex-telemost-main-account-conf-1"
        );
    }

    #[test]
    fn initialization_script_contains_only_companion_identity() {
        let script = telemost_initialization_script(
            &YandexTelemostCompanionOpenRequest {
                account_id: "telemost-main".to_owned(),
                join_url: "https://telemost.yandex.ru/j/conf-1".to_owned(),
                conference_id: Some("conf-1".to_owned()),
                display_name: None,
            },
            "yandex-telemost-main-conf-1",
        )
        .expect("initialization script");

        assert!(script.contains("__MAKOSH_YANDEX_TELEMOST_COMPANION__"));
        assert!(!script.contains("recording"));
        assert!(!script.contains("speaker"));
        assert!(!script.contains("invoke("));
    }
}
