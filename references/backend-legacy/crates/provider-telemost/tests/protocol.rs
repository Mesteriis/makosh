use makosh_provider_telemost::protocol::{
    sanitize_yandex_telemost_payload, validate_telemost_join_url,
};
use serde_json::json;

#[test]
fn join_url_rejects_non_telemost_hosts() {
    let error = validate_telemost_join_url("https://evil.example/room").unwrap_err();
    assert!(error.to_string().contains("unsupported Yandex Telemost"));
}

#[test]
fn sanitizer_removes_secret_and_audio_material_recursively() {
    let sanitized = sanitize_yandex_telemost_payload(json!({
        "id": "c1",
        "oauth_token": "secret",
        "nested": { "mp3_bytes": "base64", "speaker": "Alice" }
    }));
    assert_eq!(sanitized["id"], "c1");
    assert!(sanitized.get("oauth_token").is_none());
    assert!(sanitized["nested"].get("mp3_bytes").is_none());
}
