use makosh_hub_backend::integrations::mail::accounts::models::GmailOAuthSetupRequest;
use makosh_hub_backend::platform::config::{app_config::AppConfig, google::GoogleOAuthClientType};

#[test]
fn gmail_oauth_setup_defaults_to_mail_send_calendar_and_contacts_scopes() {
    let request = GmailOAuthSetupRequest::new(
        "acct_google_workspace",
        "Google Workspace",
        "",
        "desktop-client-id",
        "http://127.0.0.1:18088/oauth/callback",
    );

    assert_eq!(
        request.scopes,
        [
            "https://www.googleapis.com/auth/gmail.modify",
            "https://www.googleapis.com/auth/gmail.send",
            "https://www.googleapis.com/auth/calendar.readonly",
            "https://www.googleapis.com/auth/contacts.readonly",
        ]
    );
}

#[test]
fn app_config_accepts_google_oauth_client_credentials() {
    let config = AppConfig::from_pairs([
        ("MAKOSH_GOOGLE_OAUTH_CLIENT_ID", "google-client-id"),
        ("MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET", "google-client-secret"),
    ])
    .expect("config");

    assert_eq!(config.google_oauth_client_id(), Some("google-client-id"));
    assert_eq!(
        config
            .google_oauth_client_secret()
            .expect("google client secret")
            .expose_for_runtime(),
        "google-client-secret"
    );
}

#[test]
fn app_config_accepts_google_oauth_installed_client_json() {
    let config = AppConfig::from_pairs([(
        "MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
        r#"{
            "installed": {
                "client_id": "desktop-client-id.apps.googleusercontent.com",
                "project_id": "makosh-local",
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token",
                "client_secret": "desktop-client-secret",
                "redirect_uris": ["http://localhost"]
            }
        }"#,
    )])
    .expect("config");

    let google_client = config
        .google_oauth_client()
        .expect("google oauth client config");
    assert_eq!(
        google_client.client_type(),
        GoogleOAuthClientType::Installed
    );
    assert_eq!(
        google_client.client_id(),
        "desktop-client-id.apps.googleusercontent.com"
    );
    assert_eq!(
        google_client
            .client_secret()
            .expect("desktop client secret")
            .expose_for_runtime(),
        "desktop-client-secret"
    );
    assert_eq!(
        google_client.authorization_endpoint(),
        "https://accounts.google.com/o/oauth2/auth"
    );
    assert_eq!(
        google_client.token_endpoint(),
        "https://oauth2.googleapis.com/token"
    );
    assert_eq!(google_client.redirect_uris(), ["http://localhost"]);
    assert_eq!(
        config.google_oauth_client_id(),
        Some("desktop-client-id.apps.googleusercontent.com")
    );
}
