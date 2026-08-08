fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let descriptor_set =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR must be available"))
            .join("makosh.mail.v1.bin");
    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(descriptor_set);
    config.boxed(".makosh.mail.operational.v1.MailOperationalQueryResponseV1.response.message");
    config.boxed(".makosh.mail.portability.v1.MailAccountConfigurationV1.inbound.gmail");
    config
        .compile_protos(
            &[
                "proto/makosh/mail/v1/client.proto",
                "proto/makosh/mail/account/v1/client.proto",
                "proto/makosh/mail/account_lifecycle/v1/client.proto",
                "proto/makosh/mail/composition/v1/client.proto",
                "proto/makosh/mail/message_flags/v1/client.proto",
                "proto/makosh/mail/message_location/v1/client.proto",
                "proto/makosh/mail/message_permanent_delete/v1/client.proto",
                "proto/makosh/mail/operational/v1/client.proto",
                "proto/makosh/mail/sync_health/v1/client.proto",
                "proto/makosh/mail/portability/v1/portability.proto",
            ],
            &["proto"],
        )
        .expect("Mail client protocol must compile");
}
