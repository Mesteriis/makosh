use std::collections::VecDeque;
use std::future::{Future, ready};

use makosh_blob_protocol::{BlobAccessFenceV1, BlobBackupClassV1, BlobCustodyScopeV1, BlobRefV1};
use makosh_blob_runtime::vault::{
    BlobContentKeyFenceV1, BlobVaultKeyLeaseAdapterV1, BlobVaultRouteContextV1,
    BlobVaultRouteFailureV1, BlobVaultRoutePortV1,
};
use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultResponseRecipientV1, VaultTransportBindingV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};

#[tokio::test]
async fn blob_content_key_authority_uses_only_the_encrypted_vault_route() {
    let mut adapter = BlobVaultKeyLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![
            Err(BlobVaultRouteFailureV1::Rejected),
            Ok(lease_id(b'a')),
            Ok(vec![3; 16]),
            Ok(lease_id(b'b')),
            Ok(vec![9; 64]),
        ]),
        context(),
    );
    let _lease = adapter
        .ensure_content_key(&reference(), &fence(), 1)
        .await
        .expect("Vault-backed Blob key lease");
    let route_port = adapter.into_route_port();
    assert_eq!(route_port.routes.len(), 5);
    assert!(route_port.routes.iter().all(|route| {
        route.registration_id == "blob"
            && route.runtime_instance_id == "blob_runtime"
            && route.caller_runtime_generation == 5
            && route.grant_epoch == 6
            && !route.ciphertext.windows(4).any(|value| value == b"blob")
    }));
}

#[tokio::test]
async fn unavailable_blob_vault_route_has_no_local_key_fallback() {
    let mut adapter = BlobVaultKeyLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![Err(BlobVaultRouteFailureV1::Unavailable)]),
        context(),
    );
    assert!(matches!(
        adapter.ensure_content_key(&reference(), &fence(), 1).await,
        Err(makosh_blob_runtime::vault::BlobContentKeyLeaseErrorV1::Unavailable)
    ));
}

fn reference() -> BlobRefV1 {
    BlobRefV1::new([7; 16], "owner_notes", 1, None, BlobBackupClassV1::Required).expect("reference")
}

fn fence() -> BlobContentKeyFenceV1 {
    BlobContentKeyFenceV1::new(
        BlobAccessFenceV1::new(
            "owner_notes",
            "registration_notes",
            "attachments",
            "notes_runtime",
            3,
            4,
        )
        .expect("access fence"),
        BlobCustodyScopeV1::new("owner_notes", "attachments").expect("custody"),
        1,
    )
    .expect("key fence")
}

fn context() -> BlobVaultRouteContextV1 {
    BlobVaultRouteContextV1::new(
        "vault_instance".to_owned(),
        7,
        vault_public_key(),
        LeaseAudienceV1::new("blob".to_owned(), "blob_runtime".to_owned(), 5, 6)
            .expect("Blob route audience"),
    )
    .expect("Vault context")
}

fn vault_public_key() -> [u8; 32] {
    VaultResponseRecipientV1::generate()
        .public_key()
        .as_bytes()
        .to_owned()
}

fn lease_id(value: u8) -> Vec<u8> {
    vec![value; 32]
}

struct ScriptedVaultRoute {
    responses: VecDeque<Result<Vec<u8>, BlobVaultRouteFailureV1>>,
    routes: Vec<VaultCiphertextRouteV1>,
}

impl ScriptedVaultRoute {
    fn new(responses: Vec<Result<Vec<u8>, BlobVaultRouteFailureV1>>) -> Self {
        Self {
            responses: responses.into(),
            routes: Vec::new(),
        }
    }
}

impl BlobVaultRoutePortV1 for ScriptedVaultRoute {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1>> + Send
    {
        let response = self.responses.pop_front().expect("expected Vault response");
        self.routes.push(route.clone());
        ready(response.and_then(|payload| encrypted_response(&route, payload)))
    }
}

fn encrypted_response(
    route: &VaultCiphertextRouteV1,
    payload: Vec<u8>,
) -> Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1> {
    let audience = LeaseAudienceV1::new(
        route.registration_id.clone(),
        route.runtime_instance_id.clone(),
        route.caller_runtime_generation,
        route.grant_epoch,
    )
    .map_err(|_| BlobVaultRouteFailureV1::Rejected)?;
    let recipient = route
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| BlobVaultRouteFailureV1::Rejected)
        .and_then(|value| {
            VaultTransportPublicKey::from_bytes(value)
                .map_err(|_| BlobVaultRouteFailureV1::Rejected)
        })?;
    let binding = VaultTransportBindingV1::new(
        route.vault_runtime_generation,
        audience,
        route
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| BlobVaultRouteFailureV1::Rejected)?,
        route
            .operation_digest_sha256
            .as_slice()
            .try_into()
            .map_err(|_| BlobVaultRouteFailureV1::Rejected)?,
        VaultTransportDirectionV1::FromVault,
        *recipient.as_bytes(),
    )
    .map_err(|_| BlobVaultRouteFailureV1::Rejected)?;
    let frame =
        seal(&recipient, &binding, &payload).map_err(|_| BlobVaultRouteFailureV1::Rejected)?;
    Ok(VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: route.vault_runtime_generation,
        request_id: route.request_id.clone(),
        operation_digest_sha256: route.operation_digest_sha256.clone(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        caller_runtime_generation: route.caller_runtime_generation,
    })
}
