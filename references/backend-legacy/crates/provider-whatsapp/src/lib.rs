pub mod ids;

pub fn manifest() -> makosh_provider_api::ProviderManifest {
    makosh_provider_api::ProviderManifest::new(
        makosh_provider_api::ProviderId::parse("whatsapp").expect("static provider id"),
        1,
        ["webview.session", "messages.read", "messages.write"],
        [makosh_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static WhatsApp manifest")
}
