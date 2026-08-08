use makosh_provider_zulip::client::ZulipClientConfig;

#[test]
fn client_configuration_never_exposes_its_api_key_in_debug_output() {
    let config = ZulipClientConfig::new(
        "https://zulip.example.test/",
        "bot@example.test",
        "private-zulip-api-key",
    )
    .expect("client config");

    let debug = format!("{config:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("private-zulip-api-key"));
}
