use makosh_mail_api::{
    DEFAULT_WINDOW, MAX_HOST_LEN, MAX_WINDOW, MAX_WINDOWS, valid_host, valid_message_bytes,
    valid_plain_text_bytes, valid_port, valid_window,
};

#[test]
fn host_validation() {
    assert!(valid_host("localhost"));
    assert!(valid_host("mail.example.com"));
    assert!(valid_host("sub.domain.test"));
    assert!(!valid_host(""));
    assert!(!valid_host("local"));
    assert!(!valid_host("bad host"));
    let host_max = format!(
        "{}.{}.{}.{}",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61)
    );
    assert!(valid_host(host_max.as_str()));
    assert_eq!(host_max.len(), MAX_HOST_LEN);
    let host_too_long = format!("{host_max}x");
    assert_eq!(host_too_long.len(), MAX_HOST_LEN + 1);
    assert!(!valid_host(host_too_long.as_str()));
}

#[test]
fn port_validation() {
    assert!(valid_port(993));
    assert!(!valid_port(0));
}

#[test]
fn window_and_payload_limits() {
    assert!(valid_window(MAX_WINDOW / 2, 1));
    assert!(valid_window(MAX_WINDOW, MAX_WINDOWS / 2));
    assert!(valid_window(MAX_WINDOW, MAX_WINDOWS));
    assert!(!valid_window(MAX_WINDOW + 1, MAX_WINDOWS / 2));
    assert!(!valid_window(DEFAULT_WINDOW, 0));
    assert!(!valid_window(DEFAULT_WINDOW, MAX_WINDOWS + 1));
    assert!(!valid_window(0, 1));
    assert!(valid_message_bytes(1024 * 1024));
    assert!(!valid_message_bytes(1024 * 1024 + 1));
    assert!(valid_plain_text_bytes(256 * 1024));
    assert!(!valid_plain_text_bytes(256 * 1024 + 1));
}
