//! The test-only external process: proof, binding discovery and HPKE delivery.

use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use makosh_gateway_protocol::v1::{
    BeginExternalRuntimeSessionRequestV1, CompleteExternalRuntimeSessionRequestV1,
    ExternalRuntimeSessionRequestV1, ExternalRuntimeSessionResponseV1,
    GetExternalRuntimeStorageBindingRequestV1, RouteVaultCiphertextRequestV1,
    external_runtime_session_request_v1::Operation as RequestOperation,
    external_runtime_session_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::v1::{VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1};
use makosh_storage_protocol::v1::StorageBindingV1;
use makosh_vault_protocol::{
    LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1, VaultCiphertextFrameV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultResponseRecipientV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use prost::Message;
use sha2::{Digest, Sha256};

use super::transport;

const CONFIG_ENV: &str = "MAKOSH_EXTERNAL_STORAGE_PROCESS_CONFIG";
const PROOF_DOMAIN: &[u8] = b"makosh.external-runtime-session.v1\0";

#[test]
fn external_runtime_process_fixture() {
    let Some(config_path) = std::env::var_os(CONFIG_ENV) else {
        return;
    };
    let config = ClientConfig::read(Path::new(&config_path)).expect("external process config");
    let session = authenticate(&config).expect("external runtime proof");
    write_private(&config.attested_path, b"attested").expect("attestation marker");
    wait_for(&config.binding_path).expect("owner-issued binding marker");
    let binding = read_binding(&config, &session).expect("current Storage binding");
    let credential = resolve_credential(&config, &session, &binding).expect("Vault credential");
    assert!(
        !credential.is_empty(),
        "Vault returned a non-empty opaque credential"
    );
    write_private(
        &config.initial_result_path,
        encode_hex(&Sha256::digest(credential.as_slice())).as_bytes(),
    )
    .expect("initial credential result marker");
    wait_for(&config.rotation_path).expect("owner-issued Storage rotation");
    assert_stale_binding(&config, &session, &binding).expect("stale binding rejection");
    let rotated_binding = read_binding(&config, &session).expect("rotated Storage binding");
    let rotated =
        resolve_credential(&config, &session, &rotated_binding).expect("rotated Vault credential");
    assert!(
        !rotated.is_empty(),
        "Vault returned a rotated opaque credential"
    );
    write_private(
        &config.result_path,
        format!(
            "{}\n{}\n",
            encode_hex(&Sha256::digest(credential.as_slice())),
            encode_hex(&Sha256::digest(rotated.as_slice())),
        )
        .as_bytes(),
    )
    .expect("rotated credential result marker");
}

struct ClientConfig {
    socket_path: PathBuf,
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    signing_key: SigningKey,
    attested_path: PathBuf,
    binding_path: PathBuf,
    initial_result_path: PathBuf,
    rotation_path: PathBuf,
    result_path: PathBuf,
}

impl ClientConfig {
    fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        let fields = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .collect::<std::collections::HashMap<_, _>>();
        let scalar = decode_hex(value(&fields, "signing_key")?)
            .map_err(|_| "invalid external signing key".to_owned())?;
        let scalar: [u8; 32] = scalar
            .try_into()
            .map_err(|_| "invalid external signing key".to_owned())?;
        Ok(Self {
            socket_path: PathBuf::from(value(&fields, "socket_path")?),
            registration_id: value(&fields, "registration_id")?.to_owned(),
            runtime_id: value(&fields, "runtime_id")?.to_owned(),
            runtime_generation: value(&fields, "runtime_generation")?
                .parse()
                .map_err(|_| "invalid external runtime generation".to_owned())?,
            signing_key: SigningKey::from_bytes((&scalar).into())
                .map_err(|_| "invalid external signing key".to_owned())?,
            attested_path: PathBuf::from(value(&fields, "attested_path")?),
            binding_path: PathBuf::from(value(&fields, "binding_path")?),
            initial_result_path: PathBuf::from(value(&fields, "initial_result_path")?),
            rotation_path: PathBuf::from(value(&fields, "rotation_path")?),
            result_path: PathBuf::from(value(&fields, "result_path")?),
        })
    }
}

fn value<'a>(
    fields: &std::collections::HashMap<&'a str, &'a str>,
    key: &str,
) -> Result<&'a str, String> {
    fields
        .get(key)
        .copied()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing external process {key}"))
}

fn authenticate(config: &ClientConfig) -> Result<String, String> {
    let digest: [u8; 32] = Sha256::digest(b"external-storage-process").into();
    let begun = request(
        config,
        RequestOperation::Begin(BeginExternalRuntimeSessionRequestV1 {
            registration_id: config.registration_id.clone(),
            runtime_id: config.runtime_id.clone(),
            runtime_generation: config.runtime_generation,
            distribution_artifact_sha256: digest.to_vec(),
        }),
    )?;
    let ResponseResult::Begin(challenge) = begun else {
        return Err("external runtime challenge is unavailable".to_owned());
    };
    let signature: Signature = config
        .signing_key
        .sign(&proof_message(config, &challenge, digest)?);
    let completed = request(
        config,
        RequestOperation::Complete(CompleteExternalRuntimeSessionRequestV1 {
            challenge_id: challenge.challenge_id,
            signature_raw: signature.to_bytes().to_vec(),
        }),
    )?;
    match completed {
        ResponseResult::Complete(session) => Ok(session.session_id),
        _ => Err("external runtime proof is unavailable".to_owned()),
    }
}

fn proof_message(
    config: &ClientConfig,
    challenge: &makosh_gateway_protocol::v1::BeginExternalRuntimeSessionResponseV1,
    digest: [u8; 32],
) -> Result<Vec<u8>, String> {
    let mut message = PROOF_DOMAIN.to_vec();
    for field in [
        &challenge.kernel_instance_id,
        &config.registration_id,
        &config.runtime_id,
    ] {
        let length =
            u16::try_from(field.len()).map_err(|_| "proof field is too long".to_owned())?;
        message.extend_from_slice(&length.to_be_bytes());
        message.extend_from_slice(field.as_bytes());
    }
    message.extend_from_slice(&config.runtime_generation.to_be_bytes());
    message.extend_from_slice(&challenge.grant_epoch.to_be_bytes());
    message.extend_from_slice(&digest);
    message.extend_from_slice(&challenge.challenge_bytes);
    Ok(message)
}

fn read_binding(config: &ClientConfig, session_id: &str) -> Result<ExternalBinding, String> {
    let response = request(
        config,
        RequestOperation::GetStorageBinding(GetExternalRuntimeStorageBindingRequestV1 {
            session_id: session_id.to_owned(),
            capability_id: "storage.access".to_owned(),
        }),
    )?;
    let ResponseResult::GetStorageBinding(response) = response else {
        return Err("Storage binding is unavailable".to_owned());
    };
    let binding = StorageBindingV1::decode(response.storage_binding_v1.as_slice())
        .map_err(|_| "Storage binding is invalid".to_owned())?;
    let key: [u8; 32] = response
        .vault_hpke_public_key_x25519
        .try_into()
        .map_err(|_| "Vault key is invalid".to_owned())?;
    Ok(ExternalBinding {
        binding,
        vault_instance_id: response.vault_instance_id,
        vault_generation: response.vault_runtime_generation,
        vault_key: VaultTransportPublicKey::from_bytes(key)
            .map_err(|_| "Vault key is invalid".to_owned())?,
    })
}

struct ExternalBinding {
    binding: StorageBindingV1,
    vault_instance_id: String,
    vault_generation: u64,
    vault_key: VaultTransportPublicKey,
}

fn resolve_credential(
    config: &ClientConfig,
    session_id: &str,
    binding: &ExternalBinding,
) -> Result<Vec<u8>, String> {
    let create = issue(config, session_id, binding, VaultActionV1::Create)?;
    execute(
        config,
        session_id,
        binding,
        VaultTransportCommandV1::GenerateOpaqueToken {
            lease_id: create,
            secret_class: SecretClassV1::PlatformCredential,
        },
    )?;
    let resolve = issue(config, session_id, binding, VaultActionV1::Resolve)?;
    execute(
        config,
        session_id,
        binding,
        VaultTransportCommandV1::ResolveLease {
            lease_id: resolve,
            secret_class: SecretClassV1::PlatformCredential,
        },
    )
}

fn assert_stale_binding(
    config: &ClientConfig,
    session_id: &str,
    binding: &ExternalBinding,
) -> Result<(), String> {
    let error = issue(config, session_id, binding, VaultActionV1::Resolve)
        .expect_err("a replaced Storage binding must not issue another lease");
    (error == "external runtime request failed: runtime_session_stale")
        .then_some(())
        .ok_or_else(|| "replaced Storage binding returned an unexpected error".to_owned())
}

fn issue(
    config: &ClientConfig,
    session_id: &str,
    binding: &ExternalBinding,
    action: VaultActionV1,
) -> Result<LeaseIdV1, String> {
    let request = VaultLeaseIssueRequestV1::new(
        binding.vault_instance_id.clone(),
        binding.vault_generation,
        binding.binding.credential_lease_revision,
        binding.binding.owner.clone(),
        VaultPurposeRequestV1::new(
            "storage.runtime.credential".to_owned(),
            binding.binding.runtime_principal.clone(),
            vec![SecretClassV1::PlatformCredential],
            vec![action],
            60,
        )
        .map_err(|_| "Vault purpose is invalid".to_owned())?,
        audience(&binding.binding)?,
    )
    .map_err(|_| "Vault lease request is invalid".to_owned())?;
    let bytes = execute(
        config,
        session_id,
        binding,
        VaultTransportCommandV1::IssueLease { request },
    )?;
    String::from_utf8(bytes)
        .ok()
        .and_then(|value| LeaseIdV1::new(value).ok())
        .ok_or_else(|| "Vault lease is invalid".to_owned())
}

fn audience(binding: &StorageBindingV1) -> Result<LeaseAudienceV1, String> {
    LeaseAudienceV1::new(
        binding.registration_id.clone(),
        binding.runtime_instance_id.clone(),
        binding.runtime_generation,
        binding.grant_epoch,
    )
    .map_err(|_| "Storage binding audience is invalid".to_owned())
}

fn execute(
    config: &ClientConfig,
    session_id: &str,
    binding: &ExternalBinding,
    command: VaultTransportCommandV1,
) -> Result<Vec<u8>, String> {
    let recipient = VaultResponseRecipientV1::generate();
    let request_id = request_id()?;
    let digest = command.operation_digest();
    let (route, response_binding) =
        encrypted_route(binding, command, &recipient, request_id, digest)?;
    let response = request(
        config,
        RequestOperation::RouteVaultCiphertext(RouteVaultCiphertextRequestV1 {
            session_id: session_id.to_owned(),
            route: Some(route),
        }),
    )?;
    let ResponseResult::RouteVaultCiphertext(response) = response else {
        return Err("Vault route is unavailable".to_owned());
    };
    let response = response
        .response
        .ok_or_else(|| "Vault response is absent".to_owned())?;
    let frame = valid_response(&response, binding, request_id, digest)?;
    recipient
        .open(&response_binding, &frame)
        .map(|value| value.to_vec())
        .map_err(|_| "Vault response cannot be opened".to_owned())
}

fn encrypted_route(
    binding: &ExternalBinding,
    command: VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    request_id: [u8; 16],
    digest: [u8; 32],
) -> Result<(VaultCiphertextRouteV1, VaultTransportBindingV1), String> {
    let audience = audience(&binding.binding)?;
    let request_binding = transport_binding(
        binding,
        audience.clone(),
        request_id,
        digest,
        VaultTransportDirectionV1::ToVault,
        recipient,
    )?;
    let response_binding = transport_binding(
        binding,
        audience,
        request_id,
        digest,
        VaultTransportDirectionV1::FromVault,
        recipient,
    )?;
    let frame = seal(&binding.vault_key, &request_binding, &command.encode())
        .map_err(|_| "Vault request encryption failed".to_owned())?;
    Ok((
        VaultCiphertextRouteV1 {
            major: 1,
            registration_id: binding.binding.registration_id.clone(),
            runtime_instance_id: binding.binding.runtime_instance_id.clone(),
            caller_runtime_generation: binding.binding.runtime_generation,
            vault_runtime_generation: binding.vault_generation,
            grant_epoch: binding.binding.grant_epoch,
            request_id: request_id.to_vec(),
            operation_digest_sha256: digest.to_vec(),
            direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
            hpke_encapped_key: frame.encapped_key().to_vec(),
            ciphertext: frame.ciphertext().to_vec(),
            hpke_authentication_tag: frame.tag().to_vec(),
            response_recipient_hpke_public_key_x25519: recipient.public_key().as_bytes().to_vec(),
            kernel_instance_id: String::new(),
            kernel_authorization_signature_raw: Vec::new(),
            storage_role_epoch: binding.binding.role_epoch,
            storage_credential_lease_revision: binding.binding.credential_lease_revision,
            storage_runtime_principal: binding.binding.runtime_principal.clone(),
            storage_owner_id: binding.binding.owner.clone(),
        },
        response_binding,
    ))
}

fn transport_binding(
    binding: &ExternalBinding,
    audience: LeaseAudienceV1,
    request_id: [u8; 16],
    digest: [u8; 32],
    direction: VaultTransportDirectionV1,
    recipient: &VaultResponseRecipientV1,
) -> Result<VaultTransportBindingV1, String> {
    VaultTransportBindingV1::new(
        binding.vault_generation,
        audience,
        request_id,
        digest,
        direction,
        *recipient.public_key().as_bytes(),
    )
    .map_err(|_| "Vault transport binding is invalid".to_owned())
}

fn valid_response(
    response: &makosh_runtime_protocol::v1::VaultCiphertextResponseV1,
    binding: &ExternalBinding,
    request_id: [u8; 16],
    digest: [u8; 32],
) -> Result<VaultCiphertextFrameV1, String> {
    (response.major == 1
        && response.vault_runtime_generation == binding.vault_generation
        && response.caller_runtime_generation == binding.binding.runtime_generation
        && response.request_id == request_id
        && response.operation_digest_sha256 == digest
        && response.direction == VaultCiphertextRouteDirectionV1::FromVault as i32)
        .then_some(())
        .ok_or_else(|| "Vault response fences are invalid".to_owned())?;
    VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key.clone(),
        response.ciphertext.clone(),
        response.hpke_authentication_tag.clone(),
    )
    .map_err(|_| "Vault response frame is invalid".to_owned())
}

fn request(config: &ClientConfig, operation: RequestOperation) -> Result<ResponseResult, String> {
    let response = transport::call::<_, ExternalRuntimeSessionResponseV1>(
        &config.socket_path,
        &ExternalRuntimeSessionRequestV1 {
            operation: Some(operation),
        },
    )?;
    response
        .result
        .ok_or_else(|| format!("external runtime request failed: {}", response.error_code))
}

fn request_id() -> Result<[u8; 16], String> {
    let mut request_id = [0; 16];
    getrandom::fill(&mut request_id).map_err(|error| error.to_string())?;
    Ok(request_id)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("invalid hexadecimal value".to_owned());
    }
    (0..value.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&value[offset..offset + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "invalid hexadecimal value".to_owned())
}

fn wait_for(path: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if path.is_file() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err("owner did not issue Storage binding".to_owned())
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| error.to_string())?;
    use std::io::Write;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string())
}
