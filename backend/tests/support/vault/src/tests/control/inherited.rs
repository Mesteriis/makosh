use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use crate::control::inherited::{describe, read_frame, write_frame};

#[test]
fn vault_inherited_control_requires_a_successful_typed_descriptor_response() {
    let (server_stream, client_stream) = UnixStream::pair().expect("inherited pair");
    let server = std::thread::spawn(move || {
        let mut stream = server_stream;
        let request = read_frame(&mut stream).expect("read describe request");
        assert!(!request.is_empty());
        let response = ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::Describe(DescribeManagedRuntimeResponseV1 {
                registration_id: "vault-registration".to_owned(),
                runtime_generation: 1,
                grant_epoch: 1,
            })),
            error_code: String::new(),
        };
        write_frame(&mut stream, &response.encode_to_vec()).expect("write descriptor response");
        let ready = ManagedRuntimeControlRequestV1::decode(
            read_frame(&mut stream)
                .expect("read ready request")
                .as_slice(),
        )
        .expect("decode ready request");
        assert!(matches!(ready.operation, Some(Operation::Ready(_))));
    });
    let channel = describe(client_stream, vec![1], Vec::new()).expect("descriptor accepted");
    drop(channel);
    server.join().expect("server thread");
}
