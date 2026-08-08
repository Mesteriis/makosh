pub mod models;
pub mod protocol;

pub fn manifest() -> makosh_provider_api::ProviderManifest {
    makosh_provider_api::ProviderManifest::new(
        makosh_provider_api::ProviderId::parse("telemost").expect("static provider id"),
        1,
        ["meetings.read", "meetings.write"],
        [makosh_provider_api::RuntimeTopology::InProcess],
    )
    .expect("static Telemost manifest")
}
