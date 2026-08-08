use std::io;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{TimeDelta, Utc};
use getrandom::getrandom;
use url::Url;

use super::errors::AiControlCenterError;
use super::models::{AiProviderAccount, AiProviderAuthPendingGrant, AiProviderAuthStartRequest};
use super::presets::provider_presets;
use super::store::AiControlCenterStore;

const AUTH_SESSION_TTL_MINUTES: i64 = 15;
const CLI_PROBE_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AiProviderAuthProbe {
    pub(crate) authenticated: bool,
    pub(crate) message: String,
}

pub(crate) fn random_ai_provider_auth_token() -> Result<String, AiControlCenterError> {
    let mut bytes = [0_u8; 32];
    getrandom(&mut bytes).map_err(|_| {
        AiControlCenterError::InvalidRequest(
            "failed to generate AI provider authorization state token".to_owned(),
        )
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(crate) fn local_provider_auth_callback_url(
    callback_url: &str,
    setup_id: &str,
    state: &str,
) -> Result<String, AiControlCenterError> {
    let mut url = Url::parse(callback_url.trim()).map_err(|error| {
        AiControlCenterError::InvalidRequest(format!("invalid callback_url: {error}"))
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AiControlCenterError::InvalidRequest(
            "callback_url must use http or https".to_owned(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AiControlCenterError::InvalidRequest(
            "callback_url must not contain credentials".to_owned(),
        ));
    }
    url.query_pairs_mut()
        .append_pair("setup_id", setup_id)
        .append_pair("state", state);
    Ok(url.to_string())
}

pub(crate) async fn start_local_provider_auth(
    request: &AiProviderAuthStartRequest,
) -> Result<AiProviderAuthPendingGrant, AiControlCenterError> {
    request.validate()?;
    let preset = provider_presets()
        .into_iter()
        .find(|preset| {
            preset.provider_kind == request.provider_kind.trim()
                && preset.provider_key == request.provider_key.trim()
                && preset.provider_kind != "api"
        })
        .ok_or_else(|| {
            AiControlCenterError::InvalidRequest(
                "unknown local or CLI AI provider preset".to_owned(),
            )
        })?;

    let setup_id = random_ai_provider_auth_token()?;
    let state = random_ai_provider_auth_token()?;
    let callback_url = local_provider_auth_callback_url(&request.callback_url, &setup_id, &state)?;
    let display_name = request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&preset.display_name)
        .to_owned();
    let provider_id = stable_provider_id(&preset.provider_kind, &preset.provider_key);
    let login_command = login_command_for(&preset.provider_kind, &preset.provider_key);
    let probe = local_provider_auth_probe(&preset.provider_kind, &preset.provider_key).await?;
    let status = if probe.authenticated {
        "ready"
    } else {
        "waiting_for_auth"
    };

    Ok(AiProviderAuthPendingGrant {
        setup_id,
        state,
        provider_id,
        provider_kind: preset.provider_kind,
        provider_key: preset.provider_key,
        display_name,
        callback_url,
        login_command,
        status: status.to_owned(),
        message: probe.message,
        created_at: Utc::now(),
        expires_at: Utc::now() + TimeDelta::minutes(AUTH_SESSION_TTL_MINUTES),
    })
}

pub(crate) async fn connect_pending_ai_provider_auth(
    store: &AiControlCenterStore,
    pending: &mut AiProviderAuthPendingGrant,
) -> Result<Option<AiProviderAccount>, AiControlCenterError> {
    if Utc::now() > pending.expires_at {
        pending.status = "expired".to_owned();
        pending.message = "AI provider authorization callback expired. Start again.".to_owned();
        return Ok(None);
    }

    let probe = local_provider_auth_probe(&pending.provider_kind, &pending.provider_key).await?;
    if !probe.authenticated {
        pending.status = "waiting_for_auth".to_owned();
        pending.message = probe.message;
        return Ok(None);
    }

    pending.status = "ready".to_owned();
    pending.message = probe.message;
    Ok(Some(store.connect_local_or_cli_provider(pending).await?))
}

pub(crate) async fn local_provider_auth_probe(
    provider_kind: &str,
    provider_key: &str,
) -> Result<AiProviderAuthProbe, AiControlCenterError> {
    match (provider_kind.trim(), provider_key.trim()) {
        ("built_in", "ollama") => Ok(AiProviderAuthProbe {
            authenticated: true,
            message: "Built-in Ollama provider is local and does not require CLI authorization."
                .to_owned(),
        }),
        ("cli", "codex") => {
            cli_status_probe(
                "codex",
                &["login", "status"],
                "Codex CLI is authenticated.",
                "Run `codex login` in a trusted terminal, then open the Макошь callback again.",
            )
            .await
        }
        ("cli", "claude") => cli_status_probe(
            "claude",
            &["auth", "status", "--json"],
            "Claude CLI is authenticated.",
            "Run `claude auth login` in a trusted terminal, then open the Макошь callback again.",
        )
        .await,
        _ => Err(AiControlCenterError::InvalidRequest(
            "unsupported local AI provider authorization target".to_owned(),
        )),
    }
}

async fn cli_status_probe(
    command: &'static str,
    args: &'static [&'static str],
    success_message: &'static str,
    auth_required_message: &'static str,
) -> Result<AiProviderAuthProbe, AiControlCenterError> {
    let result = tokio::task::spawn_blocking(move || run_cli_status_command(command, args))
        .await
        .map_err(|error| {
            AiControlCenterError::InvalidRequest(format!("CLI status worker failed: {error}"))
        })?;

    match result {
        Ok(true) => Ok(AiProviderAuthProbe {
            authenticated: true,
            message: success_message.to_owned(),
        }),
        Ok(false) => Ok(AiProviderAuthProbe {
            authenticated: false,
            message: auth_required_message.to_owned(),
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(AiProviderAuthProbe {
            authenticated: false,
            message: format!(
                "`{command}` CLI was not found on PATH. Install it, sign in, then start this callback again."
            ),
        }),
        Err(error) => Ok(AiProviderAuthProbe {
            authenticated: false,
            message: format!("`{command}` CLI status check failed: {error}"),
        }),
    }
}

fn run_cli_status_command(command: &str, args: &[&str]) -> io::Result<bool> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let started_at = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let _output = child.wait_with_output()?;
            return Ok(status.success());
        }
        if started_at.elapsed() >= CLI_PROBE_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn stable_provider_id(provider_kind: &str, provider_key: &str) -> String {
    format!("provider:{}:{}", provider_kind.trim(), provider_key.trim())
}

fn login_command_for(provider_kind: &str, provider_key: &str) -> Option<String> {
    match (provider_kind.trim(), provider_key.trim()) {
        ("cli", "codex") => Some("codex login".to_owned()),
        ("cli", "claude") => Some("claude auth login".to_owned()),
        _ => None,
    }
}
