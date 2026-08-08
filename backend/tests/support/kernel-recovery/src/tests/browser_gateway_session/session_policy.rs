use super::*;

#[test]
fn browser_session_cookie_is_host_only_secure_and_non_persistent() {
    let session_id = "a".repeat(64);
    let header = BrowserSameOriginSessionV1::issue_cookie(&session_id).expect("session cookie");
    assert_eq!(
        header,
        format!("__Host-makosh-session={session_id}; Path=/; Secure; HttpOnly; SameSite=Strict")
    );
    assert!(!header.contains("Domain="));
    assert!(!header.contains("Max-Age="));
    assert_eq!(
        BrowserSameOriginSessionV1::session_id_from_cookie(&format!("theme=dark; {header}"))
            .expect("extract session cookie"),
        session_id
    );
    assert!(
        BrowserSameOriginSessionV1::session_id_from_cookie("__Host-makosh-session=bad").is_err()
    );
}

#[test]
fn browser_mutation_origin_requires_the_exact_https_origin() {
    BrowserSameOriginSessionV1::require_mutation_origin("https://hub.local", "https://hub.local")
        .expect("exact origin");
    assert!(
        BrowserSameOriginSessionV1::require_mutation_origin(
            "https://other.local",
            "https://hub.local"
        )
        .is_err()
    );
    assert!(
        BrowserSameOriginSessionV1::require_mutation_origin("http://hub.local", "http://hub.local")
            .is_err()
    );
}
