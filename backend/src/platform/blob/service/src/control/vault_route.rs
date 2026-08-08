//! Ciphertext-only Blob-to-Vault relay on the inherited managed channel.

use std::future::Future;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_blob_runtime::vault::{BlobVaultRouteFailureV1, BlobVaultRoutePortV1};
use makosh_runtime_protocol::v1::{
    BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeVaultRouteRequestV1, VaultCiphertextResponseV1,
    VaultCiphertextRouteV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::framing::{read_frame, write_frame};

type NestedControlHandlerV1 =
    Box<dyn FnMut(BlobRuntimeControlRequestV1) -> BlobRuntimeControlResponseV1 + Send>;

pub(super) struct InheritedBlobVaultRouteV1 {
    channel: UnixStream,
    nested_control: NestedControlHandlerV1,
}

impl InheritedBlobVaultRouteV1 {
    pub(super) fn new(
        channel: UnixStream,
        nested_control: NestedControlHandlerV1,
    ) -> Result<Self, ()> {
        channel
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| ())?;
        channel
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| ())?;
        Ok(Self {
            channel,
            nested_control,
        })
    }
}

impl BlobVaultRoutePortV1 for InheritedBlobVaultRouteV1 {
    #[allow(clippy::manual_async_fn)] // The Blob-to-Vault port requires a Send future.
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1>> + Send
    {
        async move { route_once(&mut self.channel, &mut self.nested_control, route) }
    }
}

fn route_once(
    channel: &mut UnixStream,
    nested_control: &mut NestedControlHandlerV1,
    route: VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1> {
    write_frame(
        channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::RouteVaultCiphertext(
                ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
            )),
        }
        .encode_to_vec(),
    )
    .map_err(|_| BlobVaultRouteFailureV1::Unavailable)?;
    loop {
        let frame = read_frame(channel).map_err(|_| BlobVaultRouteFailureV1::Unavailable)?;
        if let Ok(response) = ManagedRuntimeControlResponseV1::decode(frame.as_slice())
            && let Some(ControlResult::VaultRoute(response)) = response.result
        {
            if !response.error_code.is_empty() {
                return Err(BlobVaultRouteFailureV1::Rejected);
            }
            return response.response.ok_or(BlobVaultRouteFailureV1::Rejected);
        }
        let request = BlobRuntimeControlRequestV1::decode(frame.as_slice())
            .map_err(|_| BlobVaultRouteFailureV1::Rejected)?;
        if request.operation.is_none() || request.encode_to_vec() != frame {
            return Err(BlobVaultRouteFailureV1::Rejected);
        }
        let response = nested_control(request);
        write_frame(channel, &response.encode_to_vec())
            .map_err(|_| BlobVaultRouteFailureV1::Unavailable)?;
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use makosh_runtime_protocol::v1::{
        BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1, GetBlobRuntimeStatusRequestV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeVaultRouteResponseV1, VaultCiphertextResponseV1, VaultCiphertextRouteV1,
        blob_runtime_control_request_v1::Operation as BlobOperation,
        managed_runtime_control_request_v1::Operation as ManagedOperation,
        managed_runtime_control_response_v1::Result as ManagedResult,
    };
    use prost::Message;

    use super::{NestedControlHandlerV1, route_once};
    use crate::control::framing::{read_frame, write_frame};

    #[test]
    fn nested_blob_control_request_does_not_steal_vault_response() {
        let (mut runtime, mut kernel) = UnixStream::pair().expect("control channel");
        let kernel_thread = thread::spawn(move || {
            let request =
                ManagedRuntimeControlRequestV1::decode(read_frame(&mut kernel).unwrap().as_slice())
                    .unwrap();
            assert!(matches!(
                request.operation,
                Some(ManagedOperation::RouteVaultCiphertext(_))
            ));
            write_frame(
                &mut kernel,
                &BlobRuntimeControlRequestV1 {
                    operation: Some(BlobOperation::GetStatus(GetBlobRuntimeStatusRequestV1 {})),
                }
                .encode_to_vec(),
            )
            .unwrap();
            let nested_response =
                BlobRuntimeControlResponseV1::decode(read_frame(&mut kernel).unwrap().as_slice())
                    .unwrap();
            assert_eq!(nested_response.error_code, "nested_handled");
            write_frame(
                &mut kernel,
                &ManagedRuntimeControlResponseV1 {
                    result: Some(ManagedResult::VaultRoute(
                        ManagedRuntimeVaultRouteResponseV1 {
                            response: Some(VaultCiphertextResponseV1::default()),
                            error_code: String::new(),
                        },
                    )),
                    error_code: String::new(),
                }
                .encode_to_vec(),
            )
            .unwrap();
        });
        let mut nested_control: NestedControlHandlerV1 =
            Box::new(|_| BlobRuntimeControlResponseV1 {
                result: None,
                error_code: "nested_handled".to_owned(),
            });

        let response = route_once(
            &mut runtime,
            &mut nested_control,
            VaultCiphertextRouteV1::default(),
        );

        assert!(response.is_ok());
        kernel_thread.join().unwrap();
    }
}
