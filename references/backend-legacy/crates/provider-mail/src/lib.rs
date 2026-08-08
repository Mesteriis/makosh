pub mod gmail;

pub fn manifest() -> makosh_provider_api::ProviderManifest {
    makosh_provider_api::ProviderManifest::new(
        makosh_provider_api::ProviderId::parse("mail").expect("static provider id"),
        1,
        ["mail.read", "mail.write", "attachments.read"],
        [
            makosh_provider_api::RuntimeTopology::InProcess,
            makosh_provider_api::RuntimeTopology::SharedConnector,
        ],
    )
    .expect("static Mail manifest")
}
