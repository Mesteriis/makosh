pub mod tdlib;

pub fn manifest() -> makosh_provider_api::ProviderManifest {
    makosh_provider_api::ProviderManifest::new(
        makosh_provider_api::ProviderId::parse("telegram").expect("static provider id"),
        1,
        ["messages.read", "messages.write", "attachments.read"],
        [makosh_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static Telegram manifest")
}
