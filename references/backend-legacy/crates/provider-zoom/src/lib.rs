pub mod protocol;

pub fn manifest() -> makosh_provider_api::ProviderManifest {
    makosh_provider_api::ProviderManifest::new(
        makosh_provider_api::ProviderId::parse("zoom").expect("static provider id"),
        1,
        ["meetings.read", "recordings.read"],
        [makosh_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static Zoom manifest")
}
