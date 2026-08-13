use makosh_projects_api::{
    PROJECTS_MODULE_ID_V1, PROJECTS_OWNER_ID_V1, ProjectsEnvelopeContextV1,
    build_project_changed_outbox_record_v1,
    client_wire::{
        AddProjectOutcomeRequestV1, AddProjectReferenceRequestV1, CreateProjectRequestV1,
        GetProjectRequestV1, ListProjectOutcomesRequestV1, ListProjectOutcomesResultV1,
        ListProjectReferencesRequestV1, ListProjectReferencesResultV1, ListProjectsRequestV1,
        ListProjectsResultV1, ProjectChangedV1, ProjectMutationResultV1,
        ProjectOutcomeStateV1 as WireOutcomeState, ProjectOutcomeV1 as WireOutcome,
        ProjectReferenceKindV1 as WireReferenceKind, ProjectReferenceStateV1 as WireReferenceState,
        ProjectReferenceV1 as WireReference, ProjectStateV1 as WireState, ProjectV1 as WireProject,
        RemoveProjectOutcomeRequestV1, RemoveProjectReferenceRequestV1,
        SetProjectOutcomeStateRequestV1, SetProjectStateRequestV1, TimestampV1,
        UpdateProjectOutcomeRequestV1, UpdateProjectRequestV1,
    },
    projects_client_add_outcome_contract_reference_v1,
    projects_client_add_reference_contract_reference_v1,
    projects_client_create_contract_reference_v1, projects_client_get_contract_reference_v1,
    projects_client_list_contract_reference_v1,
    projects_client_list_outcomes_contract_reference_v1,
    projects_client_list_references_contract_reference_v1,
    projects_client_remove_outcome_contract_reference_v1,
    projects_client_remove_reference_contract_reference_v1,
    projects_client_set_outcome_state_contract_reference_v1,
    projects_client_set_state_contract_reference_v1, projects_client_update_contract_reference_v1,
    projects_client_update_outcome_contract_reference_v1,
};
use makosh_projects_core::{
    ProjectDraftV1, ProjectOutcomeStateV1, ProjectOutcomeV1, ProjectRecordV1,
    ProjectReferenceKindV1, ProjectReferenceStateV1, ProjectReferenceV1, ProjectStateV1,
    ProjectTimestampV1,
};
use makosh_projects_persistence::{
    ProjectLifecycleCommitV1, ProjectLifecycleMutationV1, ProjectLifecycleOperationOutcomeV1,
    ProjectLifecycleOperationV1, ProjectOutboxRecordV1, ProjectsPersistenceErrorV1,
    ProjectsPersistenceV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectsClientRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub async fn dispatch_projects_client_request_v1(
    persistence: &ProjectsPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    context: ProjectsClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == PROJECTS_MODULE_ID_V1
        && request.owner_id == PROJECTS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && nonzero(&context.runtime_instance_id)
        && context.runtime_generation > 0
        && context.now_unix_millis > 0;
    let response = if accepted {
        dispatch(persistence, logical_owner_id, &request, context).await
    } else {
        Err("REJECTED")
    };
    match response {
        Ok(response_payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(error_code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: error_code.to_owned(),
        },
    }
}

async fn dispatch(
    persistence: &ProjectsPersistenceV1,
    owner: &str,
    request: &ModuleClientRequestV1,
    context: ProjectsClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &projects_client_get_contract_reference_v1() {
        return get(persistence, owner, &request.request_payload).await;
    }
    if contract == &projects_client_list_contract_reference_v1() {
        return list(persistence, owner, &request.request_payload).await;
    }
    if contract == &projects_client_list_outcomes_contract_reference_v1() {
        return list_outcomes(persistence, owner, &request.request_payload).await;
    }
    if contract == &projects_client_list_references_contract_reference_v1() {
        return list_references(persistence, owner, &request.request_payload).await;
    }
    let operation_id = operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_operation_replay(
            owner,
            operation_id,
            request_sha256,
            &request.request_payload,
        )
        .await
        .map_err(persistence_error)?
    {
        return Ok(response);
    }
    let mutation = decode_mutation(
        contract,
        owner,
        &request.request_payload,
        context.now_unix_millis,
    )?;
    let envelope_context = ProjectsEnvelopeContextV1 {
        module_id: PROJECTS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: hex16(context.runtime_instance_id),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_lifecycle_operation(
            ProjectLifecycleOperationV1 {
                logical_owner_id: owner.to_owned(),
                operation_id,
                request_sha256,
                request_bytes: request.request_payload.clone(),
                received_at_unix_millis: context.now_unix_millis,
                mutation,
            },
            |project| build_commit(operation_id, project, &envelope_context),
        )
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        ProjectLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | ProjectLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn build_commit(
    operation_id: [u8; 16],
    project: &ProjectRecordV1,
    context: &ProjectsEnvelopeContextV1,
) -> Result<ProjectLifecycleCommitV1, ProjectsPersistenceErrorV1> {
    let response = ProjectMutationResultV1 {
        operation_id: operation_id.to_vec(),
        project: Some(wire_project(project)),
    }
    .encode_to_vec();
    let changed = build_project_changed_outbox_record_v1(
        operation_id,
        ProjectChangedV1 {
            event_id: lifecycle_event_id(operation_id, project).to_vec(),
            project_id: project.project_id.to_vec(),
            logical_owner_id: project.logical_owner_id.clone(),
            project_revision: project.project_revision,
            state: encode_state(project.state),
            start_at: project.start_at.map(wire_timestamp),
            target_at: project.target_at.map(wire_timestamp),
            occurred_at: Some(wire_timestamp(project.updated_at)),
        },
        context,
    )
    .map_err(|_| ProjectsPersistenceErrorV1::InvalidInput)?;
    Ok(ProjectLifecycleCommitV1 {
        response_sha256: Sha256::digest(&response).into(),
        response_bytes: response,
        lifecycle_event: ProjectOutboxRecordV1 {
            message_id: *changed.message_id(),
            envelope_sha256: *changed.envelope_sha256(),
            envelope_bytes: changed.exact_bytes().to_vec(),
        },
    })
}

fn operation_id(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! from {
        ($contract:expr, $ty:ty) => {
            if contract == &$contract {
                return id16(&decode_exact::<$ty>(bytes)?.operation_id);
            }
        };
    }
    from!(
        projects_client_create_contract_reference_v1(),
        CreateProjectRequestV1
    );
    from!(
        projects_client_update_contract_reference_v1(),
        UpdateProjectRequestV1
    );
    from!(
        projects_client_set_state_contract_reference_v1(),
        SetProjectStateRequestV1
    );
    from!(
        projects_client_add_outcome_contract_reference_v1(),
        AddProjectOutcomeRequestV1
    );
    from!(
        projects_client_update_outcome_contract_reference_v1(),
        UpdateProjectOutcomeRequestV1
    );
    from!(
        projects_client_set_outcome_state_contract_reference_v1(),
        SetProjectOutcomeStateRequestV1
    );
    from!(
        projects_client_remove_outcome_contract_reference_v1(),
        RemoveProjectOutcomeRequestV1
    );
    from!(
        projects_client_add_reference_contract_reference_v1(),
        AddProjectReferenceRequestV1
    );
    from!(
        projects_client_remove_reference_contract_reference_v1(),
        RemoveProjectReferenceRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    owner: &str,
    bytes: &[u8],
    now_millis: i64,
) -> Result<ProjectLifecycleMutationV1, &'static str> {
    if contract == &projects_client_create_contract_reference_v1() {
        let value = decode_exact::<CreateProjectRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        let created_at = timestamp(value.created_at.as_ref(), now_millis)?;
        return Ok(ProjectLifecycleMutationV1::Create(ProjectDraftV1 {
            operation_id: id16(&value.operation_id)?,
            logical_owner_id: owner.to_owned(),
            name: value.name,
            description: value.description,
            start_at: optional_domain_timestamp(value.start_at.as_ref())?,
            target_at: optional_domain_timestamp(value.target_at.as_ref())?,
            created_at,
        }));
    }
    if contract == &projects_client_update_contract_reference_v1() {
        let value = decode_exact::<UpdateProjectRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::Update {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            name: value.name,
            description: value.description,
            start_at: optional_domain_timestamp(value.start_at.as_ref())?,
            target_at: optional_domain_timestamp(value.target_at.as_ref())?,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_set_state_contract_reference_v1() {
        let value = decode_exact::<SetProjectStateRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::SetState {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            state: decode_state(value.state)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_add_outcome_contract_reference_v1() {
        let value = decode_exact::<AddProjectOutcomeRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::AddOutcome {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            title: value.title,
            description: value.description,
            target_at: optional_domain_timestamp(value.target_at.as_ref())?,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_update_outcome_contract_reference_v1() {
        let value = decode_exact::<UpdateProjectOutcomeRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::UpdateOutcome {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            outcome_id: id16(&value.outcome_id)?,
            expected_outcome_revision: value.expected_outcome_revision,
            title: value.title,
            description: value.description,
            target_at: optional_domain_timestamp(value.target_at.as_ref())?,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_set_outcome_state_contract_reference_v1() {
        let value = decode_exact::<SetProjectOutcomeStateRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::SetOutcomeState {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            outcome_id: id16(&value.outcome_id)?,
            expected_outcome_revision: value.expected_outcome_revision,
            state: decode_outcome_state(value.state)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_remove_outcome_contract_reference_v1() {
        let value = decode_exact::<RemoveProjectOutcomeRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::RemoveOutcome {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            outcome_id: id16(&value.outcome_id)?,
            expected_outcome_revision: value.expected_outcome_revision,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_add_reference_contract_reference_v1() {
        let value = decode_exact::<AddProjectReferenceRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::AddReference {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            kind: decode_reference_kind(value.kind)?,
            public_id: id16(&value.public_id)?,
            label: value.label,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    if contract == &projects_client_remove_reference_contract_reference_v1() {
        let value = decode_exact::<RemoveProjectReferenceRequestV1>(bytes)?;
        accepted_owner(&value.logical_owner_id, owner)?;
        return Ok(ProjectLifecycleMutationV1::RemoveReference {
            operation_id: id16(&value.operation_id)?,
            project_id: id16(&value.project_id)?,
            expected_revision: value.expected_project_revision,
            reference_id: id16(&value.reference_id)?,
            changed_at: timestamp(value.changed_at.as_ref(), now_millis)?,
        });
    }
    Err("REJECTED")
}

async fn get(
    persistence: &ProjectsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = decode_exact::<GetProjectRequestV1>(bytes)?;
    accepted_owner(&value.logical_owner_id, owner)?;
    persistence
        .get_project(owner, id16(&value.project_id)?)
        .await
        .map_err(persistence_error)?
        .map(|value| wire_project(&value).encode_to_vec())
        .ok_or("NOT_FOUND")
}

async fn list(
    persistence: &ProjectsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = decode_exact::<ListProjectsRequestV1>(bytes)?;
    accepted_owner(&value.logical_owner_id, owner)?;
    let (after, limit) = cursor_limit(&value.after_project_id, value.limit)?;
    let mut projects = persistence
        .list_projects(owner, after, limit + 1)
        .await
        .map_err(persistence_error)?;
    let has_more = projects.len() > usize::from(limit);
    projects.truncate(usize::from(limit));
    let next = if has_more {
        projects
            .last()
            .map(|value| value.project_id.to_vec())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(ListProjectsResultV1 {
        projects: projects.iter().map(wire_project).collect(),
        next_after_project_id: next,
    }
    .encode_to_vec())
}

async fn list_outcomes(
    persistence: &ProjectsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = decode_exact::<ListProjectOutcomesRequestV1>(bytes)?;
    accepted_owner(&value.logical_owner_id, owner)?;
    let (after, limit) = cursor_limit(&value.after_outcome_id, value.limit)?;
    let mut outcomes = persistence
        .list_project_outcomes(owner, id16(&value.project_id)?, after, limit + 1)
        .await
        .map_err(persistence_error)?;
    let has_more = outcomes.len() > usize::from(limit);
    outcomes.truncate(usize::from(limit));
    let next = if has_more {
        outcomes
            .last()
            .map(|value| value.outcome_id.to_vec())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(ListProjectOutcomesResultV1 {
        outcomes: outcomes.iter().map(wire_outcome).collect(),
        next_after_outcome_id: next,
    }
    .encode_to_vec())
}

async fn list_references(
    persistence: &ProjectsPersistenceV1,
    owner: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let value = decode_exact::<ListProjectReferencesRequestV1>(bytes)?;
    accepted_owner(&value.logical_owner_id, owner)?;
    let (after, limit) = cursor_limit(&value.after_reference_id, value.limit)?;
    let mut references = persistence
        .list_project_references(owner, id16(&value.project_id)?, after, limit + 1)
        .await
        .map_err(persistence_error)?;
    let has_more = references.len() > usize::from(limit);
    references.truncate(usize::from(limit));
    let next = if has_more {
        references
            .last()
            .map(|value| value.reference_id.to_vec())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(ListProjectReferencesResultV1 {
        references: references.iter().map(wire_reference).collect(),
        next_after_reference_id: next,
    }
    .encode_to_vec())
}

fn wire_project(value: &ProjectRecordV1) -> WireProject {
    WireProject {
        project_id: value.project_id.to_vec(),
        logical_owner_id: value.logical_owner_id.clone(),
        name: value.name.clone(),
        description: value.description.clone(),
        state: encode_state(value.state),
        start_at: value.start_at.map(wire_timestamp),
        target_at: value.target_at.map(wire_timestamp),
        project_revision: value.project_revision,
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}
fn wire_outcome(value: &ProjectOutcomeV1) -> WireOutcome {
    WireOutcome {
        outcome_id: value.outcome_id.to_vec(),
        project_id: value.project_id.to_vec(),
        title: value.title.clone(),
        description: value.description.clone(),
        state: encode_outcome_state(value.state),
        target_at: value.target_at.map(wire_timestamp),
        outcome_revision: value.outcome_revision,
        updated_at_project_revision: value.updated_at_project_revision,
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}
fn wire_reference(value: &ProjectReferenceV1) -> WireReference {
    WireReference {
        reference_id: value.reference_id.to_vec(),
        kind: encode_reference_kind(value.kind),
        public_id: value.public_id.to_vec(),
        label: value.label.clone(),
        state: match value.state {
            ProjectReferenceStateV1::Active => {
                WireReferenceState::ProjectReferenceStateActive as i32
            }
            ProjectReferenceStateV1::Removed => {
                WireReferenceState::ProjectReferenceStateRemoved as i32
            }
        },
        updated_at_project_revision: value.updated_at_project_revision,
    }
}
fn wire_timestamp(value: ProjectTimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn decode_state(value: i32) -> Result<ProjectStateV1, &'static str> {
    match WireState::try_from(value).map_err(|_| "INVALID_PAYLOAD")? {
        WireState::ProjectStatePlanning => Ok(ProjectStateV1::Planning),
        WireState::ProjectStateActive => Ok(ProjectStateV1::Active),
        WireState::ProjectStateOnHold => Ok(ProjectStateV1::OnHold),
        WireState::ProjectStateCompleted => Ok(ProjectStateV1::Completed),
        WireState::ProjectStateArchived => Ok(ProjectStateV1::Archived),
        WireState::ProjectStateUnspecified => Err("INVALID_PAYLOAD"),
    }
}
fn encode_state(value: ProjectStateV1) -> i32 {
    match value {
        ProjectStateV1::Planning => WireState::ProjectStatePlanning as i32,
        ProjectStateV1::Active => WireState::ProjectStateActive as i32,
        ProjectStateV1::OnHold => WireState::ProjectStateOnHold as i32,
        ProjectStateV1::Completed => WireState::ProjectStateCompleted as i32,
        ProjectStateV1::Archived => WireState::ProjectStateArchived as i32,
    }
}
fn decode_outcome_state(value: i32) -> Result<ProjectOutcomeStateV1, &'static str> {
    match WireOutcomeState::try_from(value).map_err(|_| "INVALID_PAYLOAD")? {
        WireOutcomeState::ProjectOutcomeStatePending => Ok(ProjectOutcomeStateV1::Pending),
        WireOutcomeState::ProjectOutcomeStateAchieved => Ok(ProjectOutcomeStateV1::Achieved),
        WireOutcomeState::ProjectOutcomeStateMissed => Ok(ProjectOutcomeStateV1::Missed),
        WireOutcomeState::ProjectOutcomeStateCancelled => Ok(ProjectOutcomeStateV1::Cancelled),
        WireOutcomeState::ProjectOutcomeStateUnspecified => Err("INVALID_PAYLOAD"),
    }
}
fn encode_outcome_state(value: ProjectOutcomeStateV1) -> i32 {
    match value {
        ProjectOutcomeStateV1::Pending => WireOutcomeState::ProjectOutcomeStatePending as i32,
        ProjectOutcomeStateV1::Achieved => WireOutcomeState::ProjectOutcomeStateAchieved as i32,
        ProjectOutcomeStateV1::Missed => WireOutcomeState::ProjectOutcomeStateMissed as i32,
        ProjectOutcomeStateV1::Cancelled => WireOutcomeState::ProjectOutcomeStateCancelled as i32,
    }
}
fn decode_reference_kind(value: i32) -> Result<ProjectReferenceKindV1, &'static str> {
    match WireReferenceKind::try_from(value).map_err(|_| "INVALID_PAYLOAD")? {
        WireReferenceKind::ProjectReferenceKindPerson => Ok(ProjectReferenceKindV1::Person),
        WireReferenceKind::ProjectReferenceKindOrganization => {
            Ok(ProjectReferenceKindV1::Organization)
        }
        WireReferenceKind::ProjectReferenceKindRelationship => {
            Ok(ProjectReferenceKindV1::Relationship)
        }
        WireReferenceKind::ProjectReferenceKindTask => Ok(ProjectReferenceKindV1::Task),
        WireReferenceKind::ProjectReferenceKindDocument => Ok(ProjectReferenceKindV1::Document),
        WireReferenceKind::ProjectReferenceKindCalendarEvent => {
            Ok(ProjectReferenceKindV1::CalendarEvent)
        }
        WireReferenceKind::ProjectReferenceKindUnspecified => Err("INVALID_PAYLOAD"),
    }
}
fn encode_reference_kind(value: ProjectReferenceKindV1) -> i32 {
    match value {
        ProjectReferenceKindV1::Person => WireReferenceKind::ProjectReferenceKindPerson as i32,
        ProjectReferenceKindV1::Organization => {
            WireReferenceKind::ProjectReferenceKindOrganization as i32
        }
        ProjectReferenceKindV1::Relationship => {
            WireReferenceKind::ProjectReferenceKindRelationship as i32
        }
        ProjectReferenceKindV1::Task => WireReferenceKind::ProjectReferenceKindTask as i32,
        ProjectReferenceKindV1::Document => WireReferenceKind::ProjectReferenceKindDocument as i32,
        ProjectReferenceKindV1::CalendarEvent => {
            WireReferenceKind::ProjectReferenceKindCalendarEvent as i32
        }
    }
}

fn timestamp(
    value: Option<&TimestampV1>,
    now_millis: i64,
) -> Result<ProjectTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_PAYLOAD")?;
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_PAYLOAD");
    }
    let millis = value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|v| v.checked_add(i64::from(value.nanos / 1_000_000)))
        .ok_or("INVALID_PAYLOAD")?;
    if millis > now_millis {
        return Err("INVALID_PAYLOAD");
    }
    Ok(ProjectTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}
fn domain_timestamp(value: &TimestampV1) -> Result<ProjectTimestampV1, &'static str> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_PAYLOAD");
    }
    Ok(ProjectTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}
fn optional_domain_timestamp(
    value: Option<&TimestampV1>,
) -> Result<Option<ProjectTimestampV1>, &'static str> {
    value.map(domain_timestamp).transpose()
}
fn accepted_owner(payload_owner: &str, owner: &str) -> Result<(), &'static str> {
    (payload_owner.is_empty() || payload_owner == owner)
        .then_some(())
        .ok_or("OWNER_MISMATCH")
}
fn cursor_limit(bytes: &[u8], limit: u32) -> Result<(Option<[u8; 16]>, u16), &'static str> {
    let limit = u16::try_from(limit)
        .ok()
        .filter(|value| (1..=200).contains(value))
        .ok_or("INVALID_PAYLOAD")?;
    Ok((
        if bytes.is_empty() {
            None
        } else {
            Some(id16(bytes)?)
        },
        limit,
    ))
}
fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let value: [u8; 16] = value.try_into().map_err(|_| "INVALID_PAYLOAD")?;
    nonzero(&value).then_some(value).ok_or("INVALID_PAYLOAD")
}
fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn decode_exact<M: Message + Default>(bytes: &[u8]) -> Result<M, &'static str> {
    if bytes.is_empty() || bytes.len() > 64 * 1024 {
        return Err("INVALID_PAYLOAD");
    }
    let value = M::decode(bytes).map_err(|_| "INVALID_PAYLOAD")?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or("INVALID_PAYLOAD")
}
fn persistence_error(value: ProjectsPersistenceErrorV1) -> &'static str {
    match value {
        ProjectsPersistenceErrorV1::NotFound => "NOT_FOUND",
        ProjectsPersistenceErrorV1::RevisionConflict => "REVISION_CONFLICT",
        ProjectsPersistenceErrorV1::OperationConflict
        | ProjectsPersistenceErrorV1::OutboxConflict => "CONFLICT",
        ProjectsPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
        ProjectsPersistenceErrorV1::InvalidInput | ProjectsPersistenceErrorV1::InvalidRow => {
            "INVALID_PAYLOAD"
        }
    }
}
fn lifecycle_event_id(operation_id: [u8; 16], project: &ProjectRecordV1) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.projects.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(project.project_id);
    hash.update(project.project_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}
fn hex16(value: [u8; 16]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_and_cursor_boundaries_are_exact() {
        assert_eq!(accepted_owner("", "owner-1"), Ok(()));
        assert_eq!(accepted_owner("owner-1", "owner-1"), Ok(()));
        assert_eq!(accepted_owner("owner-2", "owner-1"), Err("OWNER_MISMATCH"));
        assert_eq!(cursor_limit(&[1; 16], 200), Ok((Some([1; 16]), 200)));
        assert!(cursor_limit(&[], 0).is_err());
    }

    #[test]
    fn project_schedule_timestamps_may_be_in_the_future() {
        let future = TimestampV1 {
            unix_seconds: 2_000,
            nanos: 123_000_000,
        };
        assert_eq!(
            optional_domain_timestamp(Some(&future)),
            Ok(Some(ProjectTimestampV1 {
                unix_seconds: 2_000,
                nanos: 123_000_000,
            }))
        );
        assert_eq!(timestamp(Some(&future), 1_000_000), Err("INVALID_PAYLOAD"));
    }

    #[test]
    fn lifecycle_event_excludes_project_and_outcome_text() {
        let project = ProjectRecordV1 {
            project_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            name: "private-project".to_owned(),
            description: "private-outcome".to_owned(),
            state: ProjectStateV1::Active,
            start_at: None,
            target_at: None,
            project_revision: 2,
            outcomes: vec![],
            references: vec![],
            created_at: ProjectTimestampV1 {
                unix_seconds: 1,
                nanos: 0,
            },
            updated_at: ProjectTimestampV1 {
                unix_seconds: 2,
                nanos: 0,
            },
        };
        let commit = build_commit(
            [2; 16],
            &project,
            &ProjectsEnvelopeContextV1 {
                module_id: PROJECTS_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 2,
                recorded_at_nanos: 0,
            },
        )
        .expect("commit");
        assert!(
            !commit
                .lifecycle_event
                .envelope_bytes
                .windows(b"private-project".len())
                .any(|v| v == b"private-project")
        );
        assert!(
            !commit
                .lifecycle_event
                .envelope_bytes
                .windows(b"private-outcome".len())
                .any(|v| v == b"private-outcome")
        );
    }
}
