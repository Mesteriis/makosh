fn main() {
    println!("cargo:rerun-if-changed=../../contracts/proto/makosh/common/v1/common.proto");
    println!("cargo:rerun-if-changed=../../contracts/proto/makosh/signal_hub/v1/signal_hub.proto");
    println!(
        "cargo:rerun-if-changed=../../contracts/proto/makosh/communications/v1/communications.proto"
    );
    println!("cargo:rerun-if-changed=../../contracts/proto");

    connectrpc_build::Config::new()
        .files(&[
            "../../contracts/proto/makosh/common/v1/common.proto",
            "../../contracts/proto/makosh/signal_hub/v1/signal_hub.proto",
            "../../contracts/proto/makosh/communications/v1/communications.proto",
        ])
        .includes(&["../../contracts/proto"])
        .include_file("_connectrpc.rs")
        .compile()
        .expect("ConnectRPC contract code generation must succeed");
}
