use makosh_events_protocol::{
    envelope::{EnvelopeValidationError, validate_envelope_v1},
    v1::{
        AckDispositionV1, AckMetadataV1, AckStageV1, ActorKindV1, ActorRefV1, CommandMetadataV1,
        ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1, ObservationMetadataV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1, TraceContextV1,
        durable_envelope_v1::Semantics,
    },
};
use makosh_runtime_protocol::{
    v1::{
        BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
        CapabilityRequestV1, ClockTimerRequestV1, ContractReferenceV1, DurableEnvelopeKindV1,
        EventRouteDirectionV1, EventRouteRequestV1, EventSubscriptionRequirementV1,
        HostCapabilityRequestV1, ModuleDescriptorV1, ModuleKindV1, SchedulerJobRequestV1,
        StorageNamespaceRequestV1, TelemetrySignalRequestV1, VaultActionV1, VaultPurposeRequestV1,
        VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
    },
    validation::descriptor::{
        DescriptorValidationError, MAX_BLOB_QUOTA_BYTES, validate_descriptor_v1,
    },
};
use prost::Message;
use prost_types::Timestamp;

fn timestamp() -> Timestamp {
    Timestamp {
        seconds: 1,
        nanos: 0,
    }
}

fn envelope(semantics: Semantics) -> DurableEnvelopeV1 {
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: vec![1; 16],
        correlation_id: vec![2; 16],
        partition_key: b"owner:test".to_vec(),
        contract: Some(ContractRefV1 {
            owner: "test".into(),
            name: "contract".into(),
            major: 1,
            revision: 1,
            schema_sha256: vec![3; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "module".into(),
            runtime_instance_id: vec![4; 16],
            runtime_generation: 1,
        }),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: vec![5],
        }),
        recorded_at: Some(timestamp()),
        semantics: Some(semantics),
        ..Default::default()
    }
}

fn valid_semantics() -> Vec<Semantics> {
    vec![
        Semantics::Command(CommandMetadataV1 {
            command_id: vec![6; 16],
            target_capability: "capability.test".into(),
            idempotency_key: vec![7],
            deadline: Some(timestamp()),
            logical_attempt: 1,
        }),
        Semantics::Event(EventMetadataV1 {
            occurred_at: Some(timestamp()),
        }),
        Semantics::Observation(ObservationMetadataV1 {
            observation_id: vec![8; 16],
            observed_at: Some(timestamp()),
            occurred_at: Some(timestamp()),
            source_cursor_sha256: vec![9; 32],
            source_sequence: Some(1),
        }),
        Semantics::Result(ResultMetadataV1 {
            command_id: vec![10; 16],
            command_message_id: vec![11; 16],
            outcome: ResultOutcomeV1::Succeeded as i32,
            completed_at: Some(timestamp()),
            execution_attempt: 1,
        }),
        Semantics::Ack(AckMetadataV1 {
            acknowledged_message_id: vec![12; 16],
            stage: AckStageV1::DurableAcceptance as i32,
            disposition: AckDispositionV1::Applied as i32,
            acknowledged_at: Some(timestamp()),
        }),
    ]
}

#[test]
fn durable_envelope_accepts_every_complete_metadata_contract() {
    for semantics in valid_semantics() {
        assert_eq!(validate_envelope_v1(&envelope(semantics)), Ok(()));
    }
}

#[test]
fn durable_envelope_rejects_incomplete_metadata_for_every_kind() {
    let invalid = vec![
        Semantics::Command(CommandMetadataV1::default()),
        Semantics::Event(EventMetadataV1::default()),
        Semantics::Observation(ObservationMetadataV1::default()),
        Semantics::Result(ResultMetadataV1::default()),
        Semantics::Ack(AckMetadataV1::default()),
    ];
    for semantics in invalid {
        assert_eq!(
            validate_envelope_v1(&envelope(semantics)),
            Err(EnvelopeValidationError::InvalidMetadata),
        );
    }
}

#[test]
fn durable_envelope_enforces_partition_trace_and_fence_limits() {
    let semantics = Semantics::Event(EventMetadataV1 {
        occurred_at: Some(timestamp()),
    });
    let mut invalid_partition = envelope(semantics.clone());
    invalid_partition.partition_key = vec![0; 257];
    assert_eq!(
        validate_envelope_v1(&invalid_partition),
        Err(EnvelopeValidationError::InvalidPartition)
    );
    let mut invalid_trace = envelope(semantics.clone());
    invalid_trace.trace = Some(TraceContextV1 {
        trace_id: vec![1; 15],
        parent_span_id: vec![2; 8],
        trace_flags: 0,
    });
    assert_eq!(
        validate_envelope_v1(&invalid_trace),
        Err(EnvelopeValidationError::InvalidTrace)
    );
    let mut invalid_fence = envelope(semantics);
    invalid_fence.source_fence = Some(SourceFenceV1 {
        kind: FenceKindV1::GrantEpoch as i32,
        scope_id: vec![3; 65],
        epoch: 1,
    });
    assert_eq!(
        validate_envelope_v1(&invalid_fence),
        Err(EnvelopeValidationError::InvalidFence)
    );
}

fn contract_reference() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: "scheduler".into(),
        name: "job".into(),
        major: 1,
        revision: 1,
        schema_sha256: vec![1; 32],
    }
}

fn valid_requests() -> Vec<Request> {
    vec![
        Request::StorageNamespace(StorageNamespaceRequestV1 {
            owner_id: "owner".into(),
            connection_budget: 1,
            timeout_millis: 1,
        }),
        Request::VaultPurpose(VaultPurposeRequestV1 {
            purpose_id: "provider".into(),
            requested_lease_ttl_seconds: 60,
            allowed_secret_classes: vec![VaultSecretClassV1::ProviderCredential as i32],
            actions: vec![VaultActionV1::Resolve as i32],
            target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
            key_schema_revision: 0,
        }),
        Request::BlobQuota(BlobQuotaRequestV1 {
            max_bytes: 1,
            custody_scope_id: "owner.content.v1".into(),
            allowed_operations: vec![BlobQuotaOperationV1::ReadRange as i32],
        }),
        Request::ClockTimer(ClockTimerRequestV1 {
            requires_wall_clock: true,
        }),
        Request::SchedulerJob(SchedulerJobRequestV1 {
            job_kind: Some(contract_reference()),
        }),
        Request::TelemetrySignal(TelemetrySignalRequestV1 {
            signal_class: "health".into(),
            quota_per_minute: 1,
        }),
        Request::HostCapability(HostCapabilityRequestV1 {
            capability_id: "host.notification".into(),
        }),
        Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(contract_reference()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        }),
    ]
}

fn descriptor(request: CapabilityRequestV1) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module".into(),
        owner_id: "owner".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: "capability.test".into(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Optional as i32,
            requests: vec![request],
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn descriptor_accepts_every_complete_capability_request_variant() {
    for request in valid_requests() {
        let descriptor = descriptor(CapabilityRequestV1 {
            request: Some(request),
        });
        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
    }
}

#[test]
fn descriptor_rejects_blob_quota_above_the_kernel_bound() {
    let descriptor = descriptor(CapabilityRequestV1 {
        request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
            max_bytes: MAX_BLOB_QUOTA_BYTES + 1,
            custody_scope_id: "owner.content.v1".into(),
            allowed_operations: vec![BlobQuotaOperationV1::ReadRange as i32],
        })),
    });
    assert_eq!(
        validate_descriptor_v1(&descriptor),
        Err(DescriptorValidationError::InvalidCapability)
    );
}

#[test]
fn descriptor_rejects_blob_quota_without_exact_custody_and_operation_scope() {
    for request in [
        BlobQuotaRequestV1 {
            max_bytes: 1,
            custody_scope_id: String::new(),
            allowed_operations: vec![BlobQuotaOperationV1::ReadRange as i32],
        },
        BlobQuotaRequestV1 {
            max_bytes: 1,
            custody_scope_id: "owner.content.v1".into(),
            allowed_operations: Vec::new(),
        },
        BlobQuotaRequestV1 {
            max_bytes: 1,
            custody_scope_id: "owner.content.v1".into(),
            allowed_operations: vec![
                BlobQuotaOperationV1::ReadRange as i32,
                BlobQuotaOperationV1::ReadRange as i32,
            ],
        },
        BlobQuotaRequestV1 {
            max_bytes: 1,
            custody_scope_id: "owner.content.v1".into(),
            allowed_operations: vec![BlobQuotaOperationV1::Unspecified as i32],
        },
    ] {
        assert_eq!(
            validate_descriptor_v1(&descriptor(CapabilityRequestV1 {
                request: Some(Request::BlobQuota(request)),
            })),
            Err(DescriptorValidationError::InvalidCapability),
        );
    }
}

#[test]
fn descriptor_rejects_missing_and_unknown_capability_request_variants() {
    let missing = descriptor(CapabilityRequestV1 { request: None });
    assert_eq!(
        validate_descriptor_v1(&missing),
        Err(DescriptorValidationError::InvalidCapability)
    );
    let unknown =
        CapabilityRequestV1::decode([0x7a, 0x00].as_slice()).expect("decode unknown field");
    assert!(unknown.request.is_none());
    assert_eq!(
        validate_descriptor_v1(&descriptor(unknown)),
        Err(DescriptorValidationError::InvalidCapability),
    );
}

#[test]
fn descriptor_rejects_an_incomplete_event_route_request() {
    let route = EventRouteRequestV1 {
        envelope_kind: DurableEnvelopeKindV1::Event as i32,
        contract: Some(contract_reference()),
        direction: EventRouteDirectionV1::Publish as i32,
        max_in_flight: 0,
        subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
        max_deliver: 0,
        ack_wait_millis: 0,
    };
    assert_eq!(
        validate_descriptor_v1(&descriptor(CapabilityRequestV1 {
            request: Some(Request::EventRoute(route)),
        })),
        Err(DescriptorValidationError::InvalidCapability),
    );
}

#[test]
fn descriptor_rejects_event_routes_that_cannot_form_durable_subjects() {
    let mut contract = contract_reference();
    contract.name = "communications.evidence-export.prepare".into();
    let route = EventRouteRequestV1 {
        envelope_kind: DurableEnvelopeKindV1::Event as i32,
        contract: Some(contract),
        direction: EventRouteDirectionV1::Publish as i32,
        max_in_flight: 32,
        subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
        max_deliver: 0,
        ack_wait_millis: 0,
    };
    assert_eq!(
        validate_descriptor_v1(&descriptor(CapabilityRequestV1 {
            request: Some(Request::EventRoute(route)),
        })),
        Err(DescriptorValidationError::InvalidCapability),
    );
}

#[test]
fn descriptor_rejects_consumer_without_explicit_delivery_policy() {
    let route = EventRouteRequestV1 {
        envelope_kind: DurableEnvelopeKindV1::Event as i32,
        contract: Some(contract_reference()),
        direction: EventRouteDirectionV1::Consume as i32,
        max_in_flight: 32,
        subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
        max_deliver: 0,
        ack_wait_millis: 0,
    };
    assert_eq!(
        validate_descriptor_v1(&descriptor(CapabilityRequestV1 {
            request: Some(Request::EventRoute(route)),
        })),
        Err(DescriptorValidationError::InvalidCapability),
    );
}
