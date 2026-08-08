use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::Url;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use super::super::catalog::DiscoveredModel;
use super::super::errors::AiControlCenterError;
use super::super::models::AiProviderAccount;
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

const OPENAI_COMPATIBLE_MODELS_TIMEOUT_SECS: u64 = 20;

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleModelsResponse {
    data: Vec<OpenAiCompatibleModelRecord>,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleModelRecord {
    id: String,
    #[serde(default)]
    object: Option<String>,
    #[serde(default)]
    owned_by: Option<String>,
    #[serde(default)]
    created: Option<i64>,
    #[serde(flatten)]
    metadata: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelRecord>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelRecord {
    name: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    modified_at: Option<String>,
    #[serde(default)]
    size: Option<i64>,
    #[serde(default)]
    digest: Option<String>,
    #[serde(default)]
    details: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    details: Option<Value>,
    #[serde(default)]
    model_info: Option<Value>,
}

impl AiControlCenterStore {
    pub async fn sync_openai_compatible_provider_models(
        &self,
        provider: &AiProviderAccount,
        api_key: &str,
        actor: &str,
    ) -> Result<usize, AiControlCenterError> {
        if provider.provider_kind != "api" {
            return Err(AiControlCenterError::InvalidRequest(
                "Only OpenAI-compatible API providers can use live model sync".to_owned(),
            ));
        }
        validate_non_empty("api_key", api_key)?;
        let base_url = provider_base_url(provider)?;
        let response = fetch_openai_compatible_models(base_url, api_key).await?;
        let discovered = discovered_models_from_response(provider, response);
        self.upsert_discovered_models_for_provider(provider, &discovered, actor)
            .await
    }

    pub async fn sync_ollama_provider_models(
        &self,
        provider: &AiProviderAccount,
        actor: &str,
    ) -> Result<usize, AiControlCenterError> {
        if provider.provider_kind != "built_in" || provider.provider_key != "ollama" {
            return Err(AiControlCenterError::InvalidRequest(
                "Only built-in Ollama can use Ollama model sync".to_owned(),
            ));
        }
        let base_url = provider_ollama_base_url(provider);
        let response = fetch_ollama_models(base_url).await?;
        let show_by_model = fetch_ollama_model_details(base_url, &response).await?;
        let discovered = discovered_ollama_models_from_response(provider, response, &show_by_model);
        self.upsert_discovered_models_for_provider(provider, &discovered, actor)
            .await
    }

    pub async fn sync_cli_provider_models(
        &self,
        provider: &AiProviderAccount,
        actor: &str,
    ) -> Result<usize, AiControlCenterError> {
        if provider.provider_kind != "cli" {
            return Err(AiControlCenterError::InvalidRequest(
                "Only CLI providers can use CLI settings model sync".to_owned(),
            ));
        }
        let discovered = read_cli_models_from_settings(provider).await?;
        self.upsert_discovered_models_for_provider(provider, &discovered, actor)
            .await
    }
}

async fn read_cli_models_from_settings(
    provider: &AiProviderAccount,
) -> Result<Vec<DiscoveredModel>, AiControlCenterError> {
    let paths = cli_settings_candidate_paths(provider);
    let mut read_paths = Vec::new();
    let mut discovered_models = Vec::new();
    for path in paths {
        let Ok(contents) = tokio::fs::read_to_string(&path).await else {
            continue;
        };
        read_paths.push(path.display().to_string());
        let value: Value = serde_json::from_str(&contents).map_err(|_| {
            AiControlCenterError::ProviderModelSync(format!(
                "CLI settings JSON `{}` is not valid JSON",
                path.display()
            ))
        })?;
        let discovered = discovered_cli_models_from_settings(provider, &path, &value);
        if !discovered.is_empty() {
            discovered_models.extend(discovered);
        }
    }

    let discovered_models = dedupe_discovered_models(discovered_models);
    if !discovered_models.is_empty() {
        return Ok(discovered_models);
    }

    Err(AiControlCenterError::ProviderModelSync(format!(
        "No CLI model definitions were found for `{}`. Checked JSON settings: {}",
        provider.provider_key,
        if read_paths.is_empty() {
            "none readable".to_owned()
        } else {
            read_paths.join(", ")
        }
    )))
}

fn cli_settings_candidate_paths(provider: &AiProviderAccount) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_config_path(&mut paths, provider.config.get("model_catalog_path"));
    push_config_path(&mut paths, provider.config.get("settings_path"));
    push_config_path(&mut paths, provider.config.get("config_path"));
    if let Some(Value::Array(items)) = provider.config.get("model_catalog_paths") {
        for item in items {
            push_config_path(&mut paths, Some(item));
        }
    }

    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        match provider.provider_key.as_str() {
            "codex" => {
                paths.push(home.join(".codex").join("settings.json"));
                paths.push(home.join(".codex").join("config.json"));
                paths.push(home.join(".codex").join(".codex-global-state.json"));
                paths.push(home.join(".codex").join("models_cache.json"));
                paths.push(home.join(".codex").join("models_catalog.json"));
            }
            "claude" => {
                paths.push(home.join(".claude").join("settings.json"));
                paths.push(home.join(".claude.json"));
            }
            other => {
                paths.push(home.join(format!(".{other}")).join("settings.json"));
                paths.push(home.join(format!(".{other}")).join("config.json"));
            }
        }
    }

    dedupe_paths(paths)
}

fn push_config_path(paths: &mut Vec<PathBuf>, value: Option<&Value>) {
    let Some(value) = value else {
        return;
    };
    if let Some(path) = value
        .as_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(expand_home_path)
    {
        paths.push(path);
    }
}

fn expand_home_path(path: &str) -> PathBuf {
    if path == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for path in paths {
        let key = path.to_string_lossy().to_string();
        if seen.insert(key) {
            deduped.push(path);
        }
    }
    deduped
}

async fn fetch_ollama_models(base_url: &str) -> Result<OllamaTagsResponse, AiControlCenterError> {
    let url = ollama_tags_url(base_url)?;
    let url_for_error = url.to_string();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(OPENAI_COMPATIBLE_MODELS_TIMEOUT_SECS))
        .build()
        .map_err(|_| {
            AiControlCenterError::ProviderModelSync(
                "Failed to create Ollama model sync client".to_owned(),
            )
        })?;
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|error| {
            AiControlCenterError::ProviderModelSync(format!(
                "Ollama /api/tags request failed for {url_for_error}: {error}"
            ))
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(AiControlCenterError::ProviderModelSync(format!(
            "Ollama /api/tags returned HTTP {}",
            status.as_u16()
        )));
    }

    response.json().await.map_err(|_| {
        AiControlCenterError::ProviderModelSync(
            "Ollama /api/tags returned an invalid payload".to_owned(),
        )
    })
}

async fn fetch_ollama_model_details(
    base_url: &str,
    response: &OllamaTagsResponse,
) -> Result<HashMap<String, OllamaShowResponse>, AiControlCenterError> {
    let mut seen = HashSet::new();
    let mut details = HashMap::new();
    for record in &response.models {
        let model_key = ollama_record_model_key(record);
        if model_key.is_empty() || !seen.insert(model_key.to_owned()) {
            continue;
        }
        let show = fetch_ollama_model_show(base_url, model_key).await?;
        details.insert(model_key.to_owned(), show);
    }
    Ok(details)
}

async fn fetch_ollama_model_show(
    base_url: &str,
    model_key: &str,
) -> Result<OllamaShowResponse, AiControlCenterError> {
    let url = ollama_show_url(base_url)?;
    let url_for_error = url.to_string();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(OPENAI_COMPATIBLE_MODELS_TIMEOUT_SECS))
        .build()
        .map_err(|_| {
            AiControlCenterError::ProviderModelSync(
                "Failed to create Ollama model details client".to_owned(),
            )
        })?;
    let response = client
        .post(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&json!({ "model": model_key }))
        .send()
        .await
        .map_err(|error| {
            AiControlCenterError::ProviderModelSync(format!(
                "Ollama /api/show request failed for {url_for_error} and model `{model_key}`: {error}"
            ))
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(AiControlCenterError::ProviderModelSync(format!(
            "Ollama /api/show returned HTTP {}",
            status.as_u16()
        )));
    }

    response.json().await.map_err(|_| {
        AiControlCenterError::ProviderModelSync(
            "Ollama /api/show returned an invalid payload".to_owned(),
        )
    })
}

async fn fetch_openai_compatible_models(
    base_url: &str,
    api_key: &str,
) -> Result<OpenAiCompatibleModelsResponse, AiControlCenterError> {
    let url = openai_compatible_models_url(base_url)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(OPENAI_COMPATIBLE_MODELS_TIMEOUT_SECS))
        .build()
        .map_err(|_| {
            AiControlCenterError::ProviderModelSync(
                "Failed to create provider model sync client".to_owned(),
            )
        })?;
    let response = client
        .get(url)
        .bearer_auth(api_key.trim())
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| {
            AiControlCenterError::ProviderModelSync("Provider /models request failed".to_owned())
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(AiControlCenterError::ProviderModelSync(format!(
            "Provider /models returned HTTP {}",
            status.as_u16()
        )));
    }

    response.json().await.map_err(|_| {
        AiControlCenterError::ProviderModelSync(
            "Provider /models returned an invalid OpenAI-compatible payload".to_owned(),
        )
    })
}

fn provider_ollama_base_url(provider: &AiProviderAccount) -> &str {
    provider
        .config
        .get("base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("http://192.168.1.2:11434")
}

fn provider_base_url(provider: &AiProviderAccount) -> Result<&str, AiControlCenterError> {
    provider
        .config
        .get("base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AiControlCenterError::InvalidRequest(
                "OpenAI-compatible provider requires Base URL before model sync".to_owned(),
            )
        })
}

fn ollama_tags_url(base_url: &str) -> Result<Url, AiControlCenterError> {
    ollama_api_url(base_url, "/api/tags")
}

fn ollama_show_url(base_url: &str) -> Result<Url, AiControlCenterError> {
    ollama_api_url(base_url, "/api/show")
}

fn ollama_api_url(base_url: &str, path: &str) -> Result<Url, AiControlCenterError> {
    let mut normalized = base_url.trim().trim_end_matches('/').to_owned();
    if normalized.ends_with("/v1") {
        normalized.truncate(normalized.len() - "/v1".len());
    }
    if normalized.is_empty() {
        return Err(AiControlCenterError::InvalidRequest(
            "Ollama Base URL is empty".to_owned(),
        ));
    }
    if !normalized.ends_with(path) {
        normalized.push_str(path);
    }
    let url = Url::parse(&normalized).map_err(|_| {
        AiControlCenterError::InvalidRequest("Ollama Base URL must be an absolute URL".to_owned())
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AiControlCenterError::InvalidRequest(
            "Ollama Base URL must use http or https".to_owned(),
        ));
    }
    Ok(url)
}

fn openai_compatible_models_url(base_url: &str) -> Result<Url, AiControlCenterError> {
    let mut normalized = base_url.trim().trim_end_matches('/').to_owned();
    if normalized.is_empty() {
        return Err(AiControlCenterError::InvalidRequest(
            "OpenAI-compatible provider Base URL is empty".to_owned(),
        ));
    }
    if !normalized.ends_with("/models") {
        normalized.push_str("/models");
    }
    let url = Url::parse(&normalized).map_err(|_| {
        AiControlCenterError::InvalidRequest(
            "OpenAI-compatible provider Base URL must be an absolute URL".to_owned(),
        )
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AiControlCenterError::InvalidRequest(
            "OpenAI-compatible provider Base URL must use http or https".to_owned(),
        ));
    }
    Ok(url)
}

fn discovered_models_from_response(
    provider: &AiProviderAccount,
    response: OpenAiCompatibleModelsResponse,
) -> Vec<DiscoveredModel> {
    let mut seen = HashSet::new();
    let mut models = Vec::new();
    for record in response.data {
        let model_key = record.id.trim();
        if model_key.is_empty() || !seen.insert(model_key.to_owned()) {
            continue;
        }
        let category = inferred_model_category(model_key).to_owned();
        let reported_capabilities = provider_metadata_capabilities(&record.metadata);
        let (capabilities, capability_source) = if reported_capabilities.is_empty() {
            (
                inferred_model_capabilities(provider, model_key, &category),
                "makosh_model_key_heuristic",
            )
        } else {
            (reported_capabilities, "provider_models_api")
        };
        models.push(DiscoveredModel {
            model_key: model_key.to_owned(),
            display_name: model_key.to_owned(),
            privacy: "remote".to_owned(),
            capabilities,
            context_window: None,
            embedding_dimension: None,
            metadata: discovered_model_metadata(provider, &record, capability_source),
            category,
        });
    }
    models
}

fn discovered_ollama_models_from_response(
    provider: &AiProviderAccount,
    response: OllamaTagsResponse,
    show_by_model: &HashMap<String, OllamaShowResponse>,
) -> Vec<DiscoveredModel> {
    let mut seen = HashSet::new();
    let mut models = Vec::new();
    for record in response.models {
        let model_key = ollama_record_model_key(&record);
        if model_key.is_empty() || !seen.insert(model_key.to_owned()) {
            continue;
        }
        let show = show_by_model.get(model_key);
        let category = ollama_model_category(model_key, show);
        let (capabilities, capability_source) =
            ollama_model_capabilities(model_key, &category, show);
        models.push(DiscoveredModel {
            model_key: model_key.to_owned(),
            display_name: model_key.to_owned(),
            privacy: "local".to_owned(),
            capabilities,
            context_window: ollama_context_window(show),
            embedding_dimension: ollama_embedding_dimension(show),
            metadata: discovered_ollama_model_metadata(provider, &record, show, capability_source),
            category,
        });
    }
    models
}

fn discovered_cli_models_from_settings(
    provider: &AiProviderAccount,
    settings_path: &Path,
    settings: &Value,
) -> Vec<DiscoveredModel> {
    let mut models = Vec::new();
    collect_cli_model_records(provider, settings_path, settings, &mut models, 0);
    dedupe_discovered_models(models)
}

fn collect_cli_model_records(
    provider: &AiProviderAccount,
    settings_path: &Path,
    value: &Value,
    models: &mut Vec<DiscoveredModel>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }

    if let Some(model_name) = value
        .pointer("/model/name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        models.push(discovered_cli_model_from_parts(
            provider,
            settings_path,
            model_name,
            model_name,
            value.pointer("/model").unwrap_or(value),
        ));
    }

    if let Some(items) = value.get("models").and_then(Value::as_array) {
        for item in items {
            if let Some(model) = discovered_cli_model_from_value(provider, settings_path, item) {
                models.push(model);
            }
        }
    }

    if let Some(items) = value
        .get("additionalModelOptionsCache")
        .and_then(Value::as_array)
    {
        for item in items {
            if let Some(model) = discovered_cli_model_from_value(provider, settings_path, item) {
                models.push(model);
            }
        }
    }

    if let Some(Value::Object(providers)) = value.get("modelProviders") {
        for provider_value in providers.values() {
            collect_model_provider_entries(provider, settings_path, provider_value, models);
        }
    }

    match value {
        Value::Object(map) => {
            for child in map.values() {
                collect_cli_model_records(provider, settings_path, child, models, depth + 1);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_cli_model_records(provider, settings_path, child, models, depth + 1);
            }
        }
        _ => {}
    }
}

fn collect_model_provider_entries(
    provider: &AiProviderAccount,
    settings_path: &Path,
    value: &Value,
    models: &mut Vec<DiscoveredModel>,
) {
    match value {
        Value::Array(items) => {
            for item in items {
                if let Some(model) = discovered_cli_model_from_value(provider, settings_path, item)
                {
                    models.push(model);
                }
            }
        }
        Value::Object(map) => {
            if let Some(items) = map.get("models").and_then(Value::as_array) {
                for item in items {
                    if let Some(model) =
                        discovered_cli_model_from_value(provider, settings_path, item)
                    {
                        models.push(model);
                    }
                }
            } else if let Some(model) =
                discovered_cli_model_from_value(provider, settings_path, value)
            {
                models.push(model);
            }
        }
        _ => {}
    }
}

fn discovered_cli_model_from_value(
    provider: &AiProviderAccount,
    settings_path: &Path,
    value: &Value,
) -> Option<DiscoveredModel> {
    if let Some(model_key) = value
        .as_str()
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        return Some(discovered_cli_model_from_parts(
            provider,
            settings_path,
            model_key,
            model_key,
            value,
        ));
    }

    let model_key = first_string_field(
        value,
        &[
            "id",
            "slug",
            "model",
            "model_key",
            "modelKey",
            "value",
            "name",
        ],
    )?;
    let display_name = first_string_field(
        value,
        &["display_name", "displayName", "label", "name", "id", "slug"],
    )
    .unwrap_or(model_key);
    Some(discovered_cli_model_from_parts(
        provider,
        settings_path,
        model_key,
        display_name,
        value,
    ))
}

fn discovered_cli_model_from_parts(
    provider: &AiProviderAccount,
    settings_path: &Path,
    model_key: &str,
    display_name: &str,
    raw_model: &Value,
) -> DiscoveredModel {
    let category = cli_model_category(model_key, raw_model);
    let (capabilities, capability_source) = cli_model_capabilities(provider, model_key, raw_model);
    DiscoveredModel {
        model_key: model_key.trim().to_owned(),
        display_name: display_name.trim().to_owned(),
        category,
        privacy: "cli".to_owned(),
        context_window: cli_context_window(raw_model),
        embedding_dimension: cli_embedding_dimension(raw_model),
        metadata: discovered_cli_model_metadata(
            provider,
            settings_path,
            raw_model,
            capability_source,
        ),
        capabilities,
    }
}

fn first_string_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    let Value::Object(map) = value else {
        return None;
    };
    keys.iter().find_map(|key| {
        map.get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn cli_model_category(model_key: &str, raw_model: &Value) -> String {
    if string_array_fields(
        raw_model,
        &["capabilities", "modalities", "supported_features"],
    )
    .iter()
    .any(|capability| capability.to_ascii_lowercase().contains("embed"))
    {
        return "embeddings".to_owned();
    }
    if cli_model_has_reasoning(raw_model) {
        return "reasoning".to_owned();
    }
    inferred_model_category(model_key).to_owned()
}

fn cli_model_capabilities(
    provider: &AiProviderAccount,
    model_key: &str,
    raw_model: &Value,
) -> (Vec<String>, &'static str) {
    let mut capabilities = Vec::new();
    for capability in string_array_fields(
        raw_model,
        &[
            "capabilities",
            "modalities",
            "supported_features",
            "features",
        ],
    ) {
        push_runtime_capability(&mut capabilities, &capability);
    }
    if cli_model_has_reasoning(raw_model) {
        push_unique_capability(&mut capabilities, "reasoning");
    }
    if !capabilities.is_empty() {
        if !capabilities
            .iter()
            .any(|capability| capability == "embeddings")
        {
            push_unique_capability(&mut capabilities, "chat");
        }
        return (capabilities, "cli_settings_json");
    }

    let category = inferred_model_category(model_key);
    (
        inferred_model_capabilities(provider, model_key, category),
        "makosh_cli_settings_heuristic",
    )
}

fn cli_model_has_reasoning(raw_model: &Value) -> bool {
    raw_bool_path(
        raw_model,
        &["generationConfig", "extra_body", "enable_thinking"],
    ) || raw_bool_path(raw_model, &["reasoning", "enabled"])
        || raw_model
            .get("reasoning")
            .is_some_and(|value| !value.is_null())
        || raw_model
            .get("default_reasoning_level")
            .is_some_and(|value| !value.is_null())
        || raw_model
            .get("supported_reasoning_levels")
            .is_some_and(|value| value.as_array().is_some_and(|items| !items.is_empty()))
        || raw_model
            .get("supports_reasoning_summaries")
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

fn string_array_fields(value: &Value, keys: &[&str]) -> Vec<String> {
    let Value::Object(map) = value else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for key in keys {
        if let Some(value) = map.get(*key) {
            collect_string_values(&mut values, value);
        }
    }
    values
}

fn collect_string_values(values: &mut Vec<String>, value: &Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_string_values(values, item);
            }
        }
        Value::String(value) => {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                values.push(trimmed.to_owned());
            }
        }
        _ => {}
    }
}

fn raw_bool_path(value: &Value, path: &[&str]) -> bool {
    let mut cursor = value;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return false;
        };
        cursor = next;
    }
    cursor.as_bool().unwrap_or(false)
}

fn cli_context_window(raw_model: &Value) -> Option<i32> {
    first_i32_path(
        raw_model,
        &[
            &["context_window"],
            &["contextWindow"],
            &["contextWindowSize"],
            &["generationConfig", "contextWindowSize"],
        ],
    )
}

fn cli_embedding_dimension(raw_model: &Value) -> Option<i32> {
    first_i32_path(
        raw_model,
        &[
            &["embedding_dimension"],
            &["embeddingDimension"],
            &["embeddingDimensionSize"],
        ],
    )
}

fn first_i32_path(value: &Value, paths: &[&[&str]]) -> Option<i32> {
    paths.iter().find_map(|path| {
        let mut cursor = value;
        for segment in *path {
            cursor = cursor.get(*segment)?;
        }
        cursor.as_i64().and_then(|value| i32::try_from(value).ok())
    })
}

fn discovered_cli_model_metadata(
    provider: &AiProviderAccount,
    settings_path: &Path,
    raw_model: &Value,
    capability_source: &str,
) -> Value {
    let mut metadata = Map::new();
    metadata.insert("source".to_owned(), json!("cli_settings_json"));
    metadata.insert("provider_key".to_owned(), json!(&provider.provider_key));
    metadata.insert(
        "settings_path".to_owned(),
        json!(settings_path.display().to_string()),
    );
    metadata.insert("capability_source".to_owned(), json!(capability_source));
    if let Some(description) = first_string_field(raw_model, &["description"]) {
        metadata.insert("description".to_owned(), json!(description));
    }
    if let Some(context_window) = cli_context_window(raw_model) {
        metadata.insert("context_window".to_owned(), json!(context_window));
    }
    if let Some(embedding_dimension) = cli_embedding_dimension(raw_model) {
        metadata.insert("embedding_dimension".to_owned(), json!(embedding_dimension));
    }
    if let Some(reasoning) = sanitized_reasoning_metadata(raw_model) {
        metadata.insert("reasoning".to_owned(), reasoning);
    }
    Value::Object(metadata)
}

fn sanitized_reasoning_metadata(raw_model: &Value) -> Option<Value> {
    if let Some(reasoning) = raw_model.get("reasoning") {
        return match reasoning {
            Value::Bool(enabled) => Some(json!({ "enabled": enabled })),
            Value::Object(map) => {
                let mut sanitized = Map::new();
                for key in ["enabled", "effort", "budget_tokens", "max_tokens"] {
                    if let Some(value) = map.get(key) {
                        sanitized.insert(key.to_owned(), value.clone());
                    }
                }
                Some(Value::Object(sanitized))
            }
            _ => None,
        };
    }
    if raw_bool_path(
        raw_model,
        &["generationConfig", "extra_body", "enable_thinking"],
    ) {
        return Some(
            json!({ "enabled": true, "source": "generationConfig.extra_body.enable_thinking" }),
        );
    }
    None
}

fn dedupe_discovered_models(models: Vec<DiscoveredModel>) -> Vec<DiscoveredModel> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for model in models {
        if model.model_key.trim().is_empty() || !seen.insert(model.model_key.clone()) {
            continue;
        }
        deduped.push(model);
    }
    deduped
}

fn ollama_record_model_key(record: &OllamaModelRecord) -> &str {
    record
        .model
        .as_deref()
        .filter(|model| !model.trim().is_empty())
        .unwrap_or(&record.name)
        .trim()
}

fn ollama_model_category(model_key: &str, show: Option<&OllamaShowResponse>) -> String {
    if let Some(show) = show
        && show
            .capabilities
            .iter()
            .any(|capability| capability.to_ascii_lowercase().contains("embed"))
    {
        return "embeddings".to_owned();
    }
    inferred_model_category(model_key).to_owned()
}

fn inferred_model_category(model_key: &str) -> &'static str {
    let model_key = model_key.to_ascii_lowercase();
    if model_key.contains("embed") {
        return "embeddings";
    }
    if model_key.contains("reason")
        || model_key.starts_with("o1")
        || model_key.starts_with("o3")
        || model_key.starts_with("o4")
        || model_key.starts_with("gpt-5")
    {
        return "reasoning";
    }
    "chat"
}

fn inferred_model_capabilities(
    provider: &AiProviderAccount,
    model_key: &str,
    category: &str,
) -> Vec<String> {
    let mut capabilities = Vec::new();
    if category == "embeddings" {
        push_unique_capability(&mut capabilities, "embeddings");
        return capabilities;
    }

    push_unique_capability(&mut capabilities, "chat");
    for capability in &provider.capabilities {
        if capability == "embeddings" {
            continue;
        }
        push_unique_capability(&mut capabilities, capability);
    }
    if category == "reasoning" || model_key.to_ascii_lowercase().contains("reason") {
        push_unique_capability(&mut capabilities, "reasoning");
    }
    capabilities
}

fn push_unique_capability(capabilities: &mut Vec<String>, capability: &str) {
    if !capabilities.iter().any(|item| item == capability) {
        capabilities.push(capability.to_owned());
    }
}

fn ollama_model_capabilities(
    model_key: &str,
    category: &str,
    show: Option<&OllamaShowResponse>,
) -> (Vec<String>, &'static str) {
    let mut capabilities = Vec::new();
    if let Some(show) = show {
        for capability in &show.capabilities {
            push_runtime_capability(&mut capabilities, capability);
        }
        if ollama_model_info_has_token(show.model_info.as_ref(), "vision") {
            push_unique_capability(&mut capabilities, "vision");
            push_unique_capability(&mut capabilities, "multimodal");
        }
        if ollama_model_info_has_token(show.model_info.as_ref(), "audio") {
            push_unique_capability(&mut capabilities, "audio");
            push_unique_capability(&mut capabilities, "multimodal");
        }
        if !capabilities.is_empty() {
            return (capabilities, "ollama_api_show");
        }
    }

    if category == "embeddings" || model_key.to_ascii_lowercase().contains("embed") {
        push_unique_capability(&mut capabilities, "embeddings");
        return (capabilities, "makosh_model_key_heuristic");
    }

    (capabilities, "runtime_not_reported")
}

fn provider_metadata_capabilities(metadata: &Map<String, Value>) -> Vec<String> {
    let mut capabilities = Vec::new();
    for key in [
        "capabilities",
        "modalities",
        "input_modalities",
        "output_modalities",
        "supported_features",
    ] {
        if let Some(value) = metadata.get(key) {
            collect_capability_values(&mut capabilities, value);
        }
    }
    capabilities
}

fn collect_capability_values(capabilities: &mut Vec<String>, value: &Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_capability_values(capabilities, item);
            }
        }
        Value::String(value) => push_runtime_capability(capabilities, value),
        Value::Object(map) => {
            for key in [
                "capabilities",
                "modalities",
                "input_modalities",
                "output_modalities",
                "supported_features",
            ] {
                if let Some(value) = map.get(key) {
                    collect_capability_values(capabilities, value);
                }
            }
        }
        _ => {}
    }
}

fn push_runtime_capability(capabilities: &mut Vec<String>, capability: &str) {
    let trimmed = capability.trim();
    if trimmed.is_empty() {
        return;
    }
    let normalized = trimmed.replace('_', "-").to_ascii_lowercase();
    match normalized.as_str() {
        "completion" | "completions" | "chat-completion" | "chat-completions"
        | "text-generation" | "text" => push_unique_capability(capabilities, "chat"),
        "embedding" | "embeddings" => push_unique_capability(capabilities, "embeddings"),
        "vision" | "image" | "images" | "image-input" => {
            push_unique_capability(capabilities, "vision");
            push_unique_capability(capabilities, "multimodal");
        }
        "audio" | "audio-input" | "audio-output" => {
            push_unique_capability(capabilities, "audio");
            push_unique_capability(capabilities, "multimodal");
        }
        "tool" | "tools" | "tool-use" | "function-calling" => {
            push_unique_capability(capabilities, "tools");
        }
        _ => push_unique_capability(capabilities, trimmed),
    }
}

fn ollama_context_window(show: Option<&OllamaShowResponse>) -> Option<i32> {
    show.and_then(|show| first_model_info_i32(show.model_info.as_ref(), "context_length"))
}

fn ollama_embedding_dimension(show: Option<&OllamaShowResponse>) -> Option<i32> {
    show.and_then(|show| first_model_info_i32(show.model_info.as_ref(), "embedding_length"))
}

fn first_model_info_i32(model_info: Option<&Value>, suffix: &str) -> Option<i32> {
    let Value::Object(map) = model_info? else {
        return None;
    };
    map.iter().find_map(|(key, value)| {
        if !key.to_ascii_lowercase().ends_with(suffix) {
            return None;
        }
        value.as_i64().and_then(|value| i32::try_from(value).ok())
    })
}

fn ollama_model_info_has_token(model_info: Option<&Value>, token: &str) -> bool {
    let Some(Value::Object(map)) = model_info else {
        return false;
    };
    let token = token.to_ascii_lowercase();
    map.keys()
        .any(|key| key.to_ascii_lowercase().contains(&token))
}

fn summarize_ollama_model_info(model_info: Option<&Value>) -> Value {
    let Some(Value::Object(map)) = model_info else {
        return Value::Object(Map::new());
    };
    let mut summary = Map::new();
    if let Some(context_window) = first_model_info_i32(model_info, "context_length") {
        summary.insert("context_window".to_owned(), json!(context_window));
    }
    if let Some(embedding_dimension) = first_model_info_i32(model_info, "embedding_length") {
        summary.insert("embedding_dimension".to_owned(), json!(embedding_dimension));
    }
    summary.insert(
        "has_vision".to_owned(),
        json!(
            map.keys()
                .any(|key| key.to_ascii_lowercase().contains("vision"))
        ),
    );
    summary.insert(
        "has_audio".to_owned(),
        json!(
            map.keys()
                .any(|key| key.to_ascii_lowercase().contains("audio"))
        ),
    );
    Value::Object(summary)
}

fn discovered_ollama_model_metadata(
    provider: &AiProviderAccount,
    record: &OllamaModelRecord,
    show: Option<&OllamaShowResponse>,
    capability_source: &str,
) -> Value {
    let mut metadata = Map::new();
    metadata.insert("source".to_owned(), json!("ollama_runtime"));
    metadata.insert("inventory_source".to_owned(), json!("ollama_api_tags"));
    metadata.insert("capability_source".to_owned(), json!(capability_source));
    metadata.insert("provider_key".to_owned(), json!(&provider.provider_key));
    metadata.insert("name".to_owned(), json!(&record.name));
    if let Some(model) = &record.model {
        metadata.insert("model".to_owned(), json!(model));
    }
    if let Some(modified_at) = &record.modified_at {
        metadata.insert("modified_at".to_owned(), json!(modified_at));
    }
    if let Some(size) = record.size {
        metadata.insert("size".to_owned(), json!(size));
    }
    if let Some(digest) = &record.digest {
        metadata.insert("digest".to_owned(), json!(digest));
    }
    if let Some(details) = &record.details {
        metadata.insert("details".to_owned(), details.clone());
    }
    if let Some(show) = show {
        metadata.insert("detail_source".to_owned(), json!("ollama_api_show"));
        metadata.insert("runtime_capabilities".to_owned(), json!(&show.capabilities));
        if let Some(details) = &show.details {
            metadata.insert("runtime_details".to_owned(), details.clone());
        }
        metadata.insert(
            "model_info_summary".to_owned(),
            summarize_ollama_model_info(show.model_info.as_ref()),
        );
    }
    Value::Object(metadata)
}

fn discovered_model_metadata(
    provider: &AiProviderAccount,
    record: &OpenAiCompatibleModelRecord,
    capability_source: &str,
) -> Value {
    let mut metadata = Map::new();
    metadata.insert("source".to_owned(), json!("provider_models_api"));
    metadata.insert("capability_source".to_owned(), json!(capability_source));
    metadata.insert("provider_key".to_owned(), json!(&provider.provider_key));
    if let Some(object) = &record.object {
        metadata.insert("object".to_owned(), json!(object));
    }
    if let Some(owned_by) = &record.owned_by {
        metadata.insert("owned_by".to_owned(), json!(owned_by));
    }
    if let Some(created) = record.created {
        metadata.insert("created".to_owned(), json!(created));
    }
    if !record.metadata.is_empty() {
        metadata.insert(
            "provider_metadata".to_owned(),
            Value::Object(record.metadata.clone()),
        );
    }
    Value::Object(metadata)
}

#[cfg(test)]
#[path = "model_sync_tests.rs"]
mod tests;
