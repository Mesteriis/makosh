//! Scheduler startup and receipt-worker lifecycle on the inherited channel.

use std::os::unix::net::UnixStream;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use makosh_runtime_protocol::{
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeReadyRequestV1,
        SchedulerRuntimeConfigurationV1, SchedulerRuntimeControlRequestV1,
        SchedulerRuntimeControlResponseV1, SchedulerRuntimeStateV1, SchedulerRuntimeStatusV1,
        managed_runtime_control_request_v1::Operation as ManagedOperation,
        scheduler_runtime_control_request_v1::Operation as SchedulerOperation,
        scheduler_runtime_control_response_v1::Result as SchedulerResult,
    },
    validation::scheduler::{
        validate_scheduler_runtime_configuration, validate_scheduler_runtime_control_request,
        validate_scheduler_runtime_status,
    },
};
use makosh_scheduler_jetstream::{
    SchedulerJetStreamDispatchPortV1, SchedulerJetStreamReceiptPortV1,
    SchedulerJetStreamScheduleControlPortV1, request_runtime_credential,
};
use makosh_scheduler_persistence::{
    SchedulerDispatchAdmissionV1, SchedulerMaterializationSourceV1, SchedulerPostgresEndpointV1,
    SchedulerPostgresStoreV1, scheduler_storage_binding_from_runtime,
};
use makosh_scheduler_protocol::SCHEDULER_RUNTIME_MODULE_ID_V1;
use makosh_storage_vault::{StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1};
use prost::Message;

use super::{
    framing::{read_frame, write_frame},
    handshake::{SchedulerRuntimeIdentity, authenticate},
    schedule_control::SchedulerScheduleControlWorkerConfigV1,
    schedules,
    vault_route::InheritedSchedulerVaultRouteV1,
    workers::{SchedulerWorkerLaunchInputV1, launch_workers},
};

const CONTROL_IDLE: Duration = Duration::from_millis(25);
const STORAGE_CONNECT_ATTEMPTS: u8 = 120;
const STORAGE_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);

pub(crate) fn serve_inherited(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: SchedulerRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_scheduler_runtime_configuration(&configuration)
        .map_err(|_| "Scheduler runtime configuration is invalid".to_owned())?;
    let (mut channel, identity) = authenticate(descriptor_bytes, settings_schema_bytes)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| "Scheduler async runtime is unavailable".to_owned())?;
    let dependencies = runtime.block_on(connect_dependencies(
        &mut channel,
        &identity,
        &configuration,
    ))?;
    let control_store = dependencies.store.clone();
    let runtime_instance_id = runtime_instance_id(&configuration.runtime_instance_id)?;
    let schedule_control_configuration = SchedulerScheduleControlWorkerConfigV1::from_runtime(
        SCHEDULER_RUNTIME_MODULE_ID_V1,
        runtime_instance_id,
        identity.runtime_generation(),
        configuration.schedule_control.as_ref(),
        &configuration.schedule_control_grants,
    )?;
    let failures = launch_workers(SchedulerWorkerLaunchInputV1 {
        runtime: &runtime,
        store: dependencies.store,
        dispatch: dependencies.dispatch,
        ports: dependencies.ports,
        schedule_control: dependencies
            .schedule_control
            .zip(schedule_control_configuration),
        dispatch_batch_limit: configuration.dispatch_batch_limit,
        reconcile_interval_millis: configuration.reconcile_interval_millis,
        source: materialization_source(&identity, &configuration)?,
        admission: dispatch_admission(&configuration)?,
    });
    announce_ready(&mut channel, &identity)?;
    serve_control(
        channel,
        identity,
        configuration,
        control_store,
        &runtime,
        failures,
    )
}

struct SchedulerDependenciesV1 {
    store: SchedulerPostgresStoreV1,
    dispatch: SchedulerJetStreamDispatchPortV1,
    ports: Vec<SchedulerJetStreamReceiptPortV1>,
    schedule_control: Option<SchedulerJetStreamScheduleControlPortV1>,
}

async fn connect_dependencies(
    channel: &mut UnixStream,
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<SchedulerDependenciesV1, String> {
    let store = connect_storage(channel, identity, configuration).await?;
    let dispatch = connect_dispatch_port(channel, identity, configuration).await?;
    let ports = connect_receipt_ports(channel, identity, configuration).await?;
    let schedule_control = connect_schedule_control_port(channel, identity, configuration).await?;
    Ok(SchedulerDependenciesV1 {
        store,
        dispatch,
        ports,
        schedule_control,
    })
}

async fn connect_schedule_control_port(
    channel: &mut UnixStream,
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<Option<SchedulerJetStreamScheduleControlPortV1>, String> {
    let Some(binding) = configuration.schedule_control.as_ref() else {
        return Ok(None);
    };
    let credential = request_runtime_credential(
        channel,
        &configuration.logical_owner_id,
        identity.registration_id(),
        &configuration.runtime_instance_id,
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.event_credential_revision,
    )
    .map_err(|_| "Scheduler Event credential is unavailable".to_owned())?;
    SchedulerJetStreamScheduleControlPortV1::connect(
        &configuration.nats_endpoint,
        credential,
        binding,
    )
    .await
    .map(Some)
    .map_err(|_| "Scheduler schedule-control transport is unavailable".to_owned())
}

async fn connect_storage(
    channel: &mut UnixStream,
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<SchedulerPostgresStoreV1, String> {
    let binding = scheduler_storage_binding_from_runtime(
        configuration
            .storage_binding
            .as_ref()
            .expect("validated binding"),
        identity.registration_id().to_owned(),
        configuration.runtime_instance_id.clone(),
        identity.runtime_generation(),
        identity.grant_epoch(),
    )
    .map_err(|_| "Scheduler Storage binding is invalid".to_owned())?;
    let context = StorageVaultRouteContextV1::new(
        configuration.vault_instance_id.clone(),
        configuration.vault_runtime_generation,
        configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| "Scheduler Vault context is invalid".to_owned())?,
    )
    .map_err(|_| "Scheduler Vault context is invalid".to_owned())?;
    let route = InheritedSchedulerVaultRouteV1::new(
        channel
            .try_clone()
            .map_err(|_| "Scheduler inherited control channel is unavailable".to_owned())?,
    )
    .map_err(|_| "Scheduler inherited control channel is unavailable".to_owned())?;
    let mut leases = StorageVaultLeaseAdapterV1::new(route, context);
    let lease_id = leases
        .issue_runtime_credential(&binding)
        .await
        .map_err(|_| "Scheduler Storage credential is unavailable".to_owned())?;
    let password = leases
        .resolve_runtime_credential(&binding, lease_id)
        .await
        .map_err(|_| "Scheduler Storage credential is unavailable".to_owned())?;
    let password = std::str::from_utf8(&password)
        .map_err(|_| "Scheduler Storage credential is unavailable".to_owned())?;
    let storage = configuration
        .storage_binding
        .as_ref()
        .expect("validated binding");
    let endpoint =
        SchedulerPostgresEndpointV1::new(storage.pgbouncer_host.clone(), storage.pgbouncer_port)
            .map_err(|_| "Scheduler Storage endpoint is invalid".to_owned())?;
    for attempt in 1..=STORAGE_CONNECT_ATTEMPTS {
        match SchedulerPostgresStoreV1::connect_runtime(&binding, &endpoint, password).await {
            Ok(store) => return Ok(store),
            Err(_) if attempt < STORAGE_CONNECT_ATTEMPTS => {
                tokio::time::sleep(STORAGE_CONNECT_RETRY_DELAY).await;
            }
            Err(_) => return Err("Scheduler Storage is unavailable".to_owned()),
        }
    }
    Err("Scheduler Storage is unavailable".to_owned())
}

async fn connect_dispatch_port(
    channel: &mut UnixStream,
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<SchedulerJetStreamDispatchPortV1, String> {
    let credential = request_runtime_credential(
        channel,
        &configuration.logical_owner_id,
        identity.registration_id(),
        &configuration.runtime_instance_id,
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.event_credential_revision,
    )
    .map_err(|_| "Scheduler Event credential is unavailable".to_owned())?;
    SchedulerJetStreamDispatchPortV1::connect(
        &configuration.nats_endpoint,
        credential,
        &configuration.dispatch_publishers,
    )
    .await
    .map_err(|_| "Scheduler dispatch publisher is unavailable".to_owned())
}

async fn connect_receipt_ports(
    channel: &mut UnixStream,
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<Vec<SchedulerJetStreamReceiptPortV1>, String> {
    let mut ports = Vec::with_capacity(configuration.receipt_consumers.len());
    for consumer in &configuration.receipt_consumers {
        let credential = request_runtime_credential(
            channel,
            &configuration.logical_owner_id,
            identity.registration_id(),
            &configuration.runtime_instance_id,
            identity.runtime_generation(),
            identity.grant_epoch(),
            configuration.event_credential_revision,
        )
        .map_err(|_| "Scheduler Event credential is unavailable".to_owned())?;
        let port = SchedulerJetStreamReceiptPortV1::connect(
            &configuration.nats_endpoint,
            credential,
            consumer,
        )
        .await
        .map_err(|_| "Scheduler receipt consumer is unavailable".to_owned())?;
        ports.push(port);
    }
    Ok(ports)
}

fn materialization_source(
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<SchedulerMaterializationSourceV1, String> {
    let instance_id = runtime_instance_id(&configuration.runtime_instance_id)?;
    SchedulerMaterializationSourceV1::new(
        SCHEDULER_RUNTIME_MODULE_ID_V1.to_owned(),
        instance_id,
        identity.runtime_generation(),
    )
    .map_err(|_| "Scheduler runtime identity is invalid".to_owned())
}

fn runtime_instance_id(value: &str) -> Result<[u8; 16], String> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|value| u8::from_str_radix(value, 16).ok())
        })
        .collect::<Option<Vec<_>>>()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| "Scheduler runtime instance identity is invalid".to_owned())
}

fn dispatch_admission(
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<SchedulerDispatchAdmissionV1, String> {
    SchedulerDispatchAdmissionV1::new(
        configuration
            .dispatch_publishers
            .iter()
            .map(|binding| binding.subject.clone()),
    )
    .map_err(|_| "Scheduler dispatch admission is invalid".to_owned())
}

fn announce_ready(
    channel: &mut UnixStream,
    identity: &SchedulerRuntimeIdentity,
) -> Result<(), String> {
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
            registration_id: identity.registration_id().to_owned(),
            runtime_generation: identity.runtime_generation(),
            grant_epoch: identity.grant_epoch(),
        })),
    };
    write_frame(channel, &request.encode_to_vec())
}

fn serve_control(
    mut channel: UnixStream,
    identity: SchedulerRuntimeIdentity,
    configuration: SchedulerRuntimeConfigurationV1,
    store: SchedulerPostgresStoreV1,
    runtime: &tokio::runtime::Runtime,
    failures: Receiver<()>,
) -> Result<(), String> {
    channel
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|_| "Scheduler inherited control channel is unavailable".to_owned())?;
    loop {
        if receipt_worker_failed(&failures)? {
            return Err("Scheduler background worker failed".to_owned());
        }
        channel
            .set_read_timeout(Some(CONTROL_IDLE))
            .map_err(|_| "Scheduler inherited control channel is unavailable".to_owned())?;
        let response = match read_frame(&mut channel) {
            Ok(bytes) => SchedulerRuntimeControlRequestV1::decode(bytes.as_slice())
                .map(|request| response_for(request, &identity, &configuration, &store, runtime))
                .unwrap_or_else(|_| error_response("invalid_request")),
            Err(error) if error == "Scheduler inherited control channel is unavailable" => continue,
            Err(_) => error_response("invalid_request"),
        };
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

fn receipt_worker_failed(failures: &Receiver<()>) -> Result<bool, String> {
    match failures.try_recv() {
        Ok(()) => Ok(true),
        Err(TryRecvError::Empty) => Ok(false),
        Err(TryRecvError::Disconnected) => {
            Err("Scheduler background workers are unavailable".to_owned())
        }
    }
}

fn response_for(
    request: SchedulerRuntimeControlRequestV1,
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
    store: &SchedulerPostgresStoreV1,
    runtime: &tokio::runtime::Runtime,
) -> SchedulerRuntimeControlResponseV1 {
    if validate_scheduler_runtime_control_request(&request).is_err() {
        return error_response("operation_not_available");
    }
    match request.operation {
        Some(SchedulerOperation::GetStatus(_)) => status_response(identity, configuration),
        Some(SchedulerOperation::UpsertSchedule(request)) => {
            upsert_schedule(request, store, runtime)
        }
        None => error_response("operation_not_available"),
    }
}

fn upsert_schedule(
    request: makosh_runtime_protocol::v1::UpsertSchedulerScheduleRequestV1,
    store: &SchedulerPostgresStoreV1,
    runtime: &tokio::runtime::Runtime,
) -> SchedulerRuntimeControlResponseV1 {
    let change = match schedules::upsert_from_request(request) {
        Ok(change) => change,
        Err(code) => return error_response(&code),
    };
    let revision = change.spec().revision().value();
    match runtime.block_on(store.upsert_schedule(&change)) {
        Ok(outcome) => SchedulerRuntimeControlResponseV1 {
            result: Some(SchedulerResult::UpsertSchedule(schedules::response(
                outcome, revision,
            ))),
            error_code: String::new(),
        },
        Err(error) => error_response(schedule_store_error(error)),
    }
}

fn schedule_store_error(
    error: makosh_scheduler_persistence::SchedulerScheduleStoreErrorV1,
) -> &'static str {
    use makosh_scheduler_persistence::SchedulerScheduleStoreErrorV1;

    match error {
        SchedulerScheduleStoreErrorV1::StaleRevision => "stale_revision",
        SchedulerScheduleStoreErrorV1::RevisionConflict => "revision_conflict",
        SchedulerScheduleStoreErrorV1::ConcurrencyBusy => "concurrency_busy",
        SchedulerScheduleStoreErrorV1::InvalidLimit
        | SchedulerScheduleStoreErrorV1::CorruptState
        | SchedulerScheduleStoreErrorV1::Unavailable => "schedule_unavailable",
    }
}

fn status_response(
    identity: &SchedulerRuntimeIdentity,
    configuration: &SchedulerRuntimeConfigurationV1,
) -> SchedulerRuntimeControlResponseV1 {
    let status = SchedulerRuntimeStatusV1 {
        state: SchedulerRuntimeStateV1::Ready as i32,
        runtime_generation: identity.runtime_generation(),
        grant_epoch: identity.grant_epoch(),
        storage_generation: configuration
            .storage_binding
            .as_ref()
            .expect("validated binding")
            .storage_generation,
        vault_runtime_generation: configuration.vault_runtime_generation,
        event_credential_revision: configuration.event_credential_revision,
        blocker_code: String::new(),
    };
    validate_scheduler_runtime_status(&status).expect("constant Scheduler runtime status is valid");
    SchedulerRuntimeControlResponseV1 {
        result: Some(SchedulerResult::Status(status)),
        error_code: String::new(),
    }
}

fn error_response(error_code: &str) -> SchedulerRuntimeControlResponseV1 {
    SchedulerRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}
