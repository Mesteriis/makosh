use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("whatsapp-delivery-intent-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/whatsapp/delivery_intent/v1/delivery_intent.proto"],
            &["proto"],
        )
        .expect("WhatsApp delivery-intent protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("WhatsApp delivery-intent descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("whatsapp_delivery_intent_schema.rs"),
        format!("pub const WHATSAPP_DELIVERY_INTENT_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("WhatsApp delivery-intent schema digest must be written");
}
