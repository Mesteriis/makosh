//! Issues short-lived runtime credentials without disclosing the account signer to Kernel.

use makosh_events_jetstream::{
    NatsAccountSignerFenceV1, NatsAccountSignerLeaseAdapterV1, NatsAccountSignerLeaseErrorV1,
    NatsJwtIssueErrorV1, NatsJwtPermissionSetV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialDeliveryV1, NatsRuntimeCredentialFenceV1, NatsVaultRouteContextV1,
    NatsVaultRoutePortV1, RuntimeNatsJwtCredentialV1, RuntimeNatsJwtIssuerV1,
};
use zeroize::Zeroizing;

/// Private authority that resolves the NATS account signer only for one issuance operation.
pub struct NatsJwtCredentialAuthorityV1<T> {
    account_public_key: String,
    signer_leases: NatsAccountSignerLeaseAdapterV1<T>,
}

impl<T> NatsJwtCredentialAuthorityV1<T>
where
    T: NatsVaultRoutePortV1 + Send,
{
    pub fn new(
        account_public_key: String,
        route_port: T,
        context: NatsVaultRouteContextV1,
    ) -> Result<Self, NatsCredentialAuthorityErrorV1> {
        validate_account_public_key(&account_public_key)?;
        Ok(Self {
            account_public_key,
            signer_leases: NatsAccountSignerLeaseAdapterV1::new(route_port, context),
        })
    }

    /// Enrolls the initial account signer through the encrypted Vault route exactly once.
    pub async fn enroll_account_signer(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
        account_signing_seed: Zeroizing<Vec<u8>>,
    ) -> Result<(), NatsCredentialAuthorityErrorV1> {
        let signing_seed = decode_signing_seed(&account_signing_seed)?;
        RuntimeNatsJwtIssuerV1::from_account_signing_seed(
            self.account_public_key.clone(),
            signing_seed.to_string(),
        )
        .map_err(map_issue_error)?;
        self.signer_leases
            .enroll_once(fence, account_signing_seed)
            .await
            .map_err(map_lease_error)
    }

    /// Resolves the signer through Vault for this one issuance and then drops it.
    pub async fn issue_runtime_credential(
        &mut self,
        signer_fence: &NatsAccountSignerFenceV1,
        runtime_fence: &NatsRuntimeCredentialFenceV1,
        permissions: NatsJwtPermissionSetV1,
        now_unix_seconds: u64,
        ttl_seconds: u64,
    ) -> Result<RuntimeNatsJwtCredentialV1, NatsCredentialAuthorityErrorV1> {
        let account_signing_seed = self
            .signer_leases
            .resolve_once(signer_fence)
            .await
            .map_err(map_lease_error)?;
        let signing_seed = decode_signing_seed(&account_signing_seed)?;
        let issuer = RuntimeNatsJwtIssuerV1::from_account_signing_seed(
            self.account_public_key.clone(),
            signing_seed.to_string(),
        )
        .map_err(map_issue_error)?;
        issuer
            .issue_runtime_credential(runtime_fence, permissions, now_unix_seconds, ttl_seconds)
            .map_err(map_issue_error)
    }

    /// Issues a credential and seals it to the requesting runtime before it leaves authority.
    pub async fn issue_sealed_runtime_credential(
        &mut self,
        signer_fence: &NatsAccountSignerFenceV1,
        runtime_fence: &NatsRuntimeCredentialFenceV1,
        permissions: NatsJwtPermissionSetV1,
        now_unix_seconds: u64,
        ttl_seconds: u64,
        delivery_binding: &NatsRuntimeCredentialDeliveryBindingV1,
    ) -> Result<NatsRuntimeCredentialDeliveryV1, NatsCredentialAuthorityErrorV1> {
        let credential = self
            .issue_runtime_credential(
                signer_fence,
                runtime_fence,
                permissions,
                now_unix_seconds,
                ttl_seconds,
            )
            .await?;
        credential
            .seal_for(delivery_binding)
            .map_err(|_| NatsCredentialAuthorityErrorV1::Rejected)
    }

    /// Confirms that the current Vault record is a usable account signing key.
    pub async fn verify_account_signer(
        &mut self,
        signer_fence: &NatsAccountSignerFenceV1,
    ) -> Result<(), NatsCredentialAuthorityErrorV1> {
        let account_signing_seed = self
            .signer_leases
            .resolve_once(signer_fence)
            .await
            .map_err(map_lease_error)?;
        let signing_seed = decode_signing_seed(&account_signing_seed)?;
        RuntimeNatsJwtIssuerV1::from_account_signing_seed(
            self.account_public_key.clone(),
            signing_seed.to_string(),
        )
        .map(|_| ())
        .map_err(map_issue_error)
    }

    pub async fn revoke_signer_leases(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
    ) -> Result<(), NatsCredentialAuthorityErrorV1> {
        self.signer_leases
            .revoke_leases(fence)
            .await
            .map_err(map_lease_error)
    }

    #[must_use]
    pub fn into_route_port(self) -> T {
        self.signer_leases.into_route_port()
    }
}

fn validate_account_public_key(value: &str) -> Result<(), NatsCredentialAuthorityErrorV1> {
    (value.len() == 56
        && value.starts_with('A')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit()))
    .then_some(())
    .ok_or(NatsCredentialAuthorityErrorV1::Rejected)
}

fn decode_signing_seed(
    value: &Zeroizing<Vec<u8>>,
) -> Result<Zeroizing<String>, NatsCredentialAuthorityErrorV1> {
    String::from_utf8(value.to_vec())
        .map(Zeroizing::new)
        .map_err(|_| NatsCredentialAuthorityErrorV1::Rejected)
}

const fn map_lease_error(error: NatsAccountSignerLeaseErrorV1) -> NatsCredentialAuthorityErrorV1 {
    match error {
        NatsAccountSignerLeaseErrorV1::Rejected => NatsCredentialAuthorityErrorV1::Rejected,
        NatsAccountSignerLeaseErrorV1::Unavailable => NatsCredentialAuthorityErrorV1::Unavailable,
    }
}

const fn map_issue_error(error: NatsJwtIssueErrorV1) -> NatsCredentialAuthorityErrorV1 {
    match error {
        NatsJwtIssueErrorV1::ClockUnavailable => NatsCredentialAuthorityErrorV1::Unavailable,
        NatsJwtIssueErrorV1::InvalidIssuer
        | NatsJwtIssueErrorV1::InvalidPermissions
        | NatsJwtIssueErrorV1::InvalidTtl
        | NatsJwtIssueErrorV1::KeyMaterialUnavailable
        | NatsJwtIssueErrorV1::Expired => NatsCredentialAuthorityErrorV1::Rejected,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NatsCredentialAuthorityErrorV1 {
    Rejected,
    Unavailable,
}
