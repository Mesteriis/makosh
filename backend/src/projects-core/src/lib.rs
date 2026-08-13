#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-projects-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const MAX_OWNER_BYTES_V1: usize = 128;
pub const MAX_NAME_CHARS_V1: usize = 240;
pub const MAX_DESCRIPTION_CHARS_V1: usize = 8_000;
pub const MAX_OUTCOME_TITLE_CHARS_V1: usize = 320;
pub const MAX_REFERENCE_LABEL_CHARS_V1: usize = 320;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectStateV1 {
    Planning,
    Active,
    OnHold,
    Completed,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectOutcomeStateV1 {
    Pending,
    Achieved,
    Missed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectReferenceKindV1 {
    Person,
    Organization,
    Relationship,
    Task,
    Document,
    CalendarEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectReferenceStateV1 {
    Active,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectOutcomeV1 {
    pub outcome_id: [u8; 16],
    pub project_id: [u8; 16],
    pub title: String,
    pub description: String,
    pub state: ProjectOutcomeStateV1,
    pub target_at: Option<ProjectTimestampV1>,
    pub outcome_revision: u64,
    pub updated_at_project_revision: u64,
    pub created_at: ProjectTimestampV1,
    pub updated_at: ProjectTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectReferenceV1 {
    pub reference_id: [u8; 16],
    pub kind: ProjectReferenceKindV1,
    pub public_id: [u8; 16],
    pub label: String,
    pub state: ProjectReferenceStateV1,
    pub updated_at_project_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectDraftV1 {
    pub operation_id: [u8; 16],
    pub logical_owner_id: String,
    pub name: String,
    pub description: String,
    pub start_at: Option<ProjectTimestampV1>,
    pub target_at: Option<ProjectTimestampV1>,
    pub created_at: ProjectTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRecordV1 {
    pub project_id: [u8; 16],
    pub logical_owner_id: String,
    pub name: String,
    pub description: String,
    pub state: ProjectStateV1,
    pub start_at: Option<ProjectTimestampV1>,
    pub target_at: Option<ProjectTimestampV1>,
    pub project_revision: u64,
    pub outcomes: Vec<ProjectOutcomeV1>,
    pub references: Vec<ProjectReferenceV1>,
    pub created_at: ProjectTimestampV1,
    pub updated_at: ProjectTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectLifecycleErrorV1 {
    InvalidInput,
    InvalidOwner,
    InvalidOperationId,
    InvalidProjectId,
    InvalidRevision,
    RevisionOverflow,
    InvalidStateTransition,
    OutcomeConflict,
    OutcomeNotFound,
    ReferenceConflict,
    ReferenceNotFound,
}

pub fn derive_project_id_v1(
    logical_owner_id: &str,
    operation_id: &[u8; 16],
) -> Result<[u8; 16], ProjectLifecycleErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(ProjectLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(operation_id) {
        return Err(ProjectLifecycleErrorV1::InvalidOperationId);
    }
    Ok(derive_id(
        b"makosh.projects.project-id.v1\0",
        &[logical_owner_id.as_bytes(), operation_id],
    ))
}

pub fn derive_project_outcome_id_v1(
    project_id: &[u8; 16],
    operation_id: &[u8; 16],
) -> Result<[u8; 16], ProjectLifecycleErrorV1> {
    if !nonzero(project_id) || !nonzero(operation_id) {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    Ok(derive_id(
        b"makosh.projects.outcome-id.v1\0",
        &[project_id, operation_id],
    ))
}

pub fn derive_project_reference_id_v1(
    project_id: &[u8; 16],
    kind: ProjectReferenceKindV1,
    public_id: &[u8; 16],
) -> Result<[u8; 16], ProjectLifecycleErrorV1> {
    if !nonzero(project_id) || !nonzero(public_id) {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    Ok(derive_id(
        b"makosh.projects.reference-id.v1\0",
        &[project_id, &[reference_kind_code(kind)], public_id],
    ))
}

pub fn create_project_v1(
    draft: ProjectDraftV1,
) -> Result<ProjectRecordV1, ProjectLifecycleErrorV1> {
    validate_text(&draft.name, 1, MAX_NAME_CHARS_V1)?;
    validate_text(&draft.description, 0, MAX_DESCRIPTION_CHARS_V1)?;
    validate_schedule(draft.start_at, draft.target_at)?;
    validate_timestamp(draft.created_at)?;
    let project = ProjectRecordV1 {
        project_id: derive_project_id_v1(&draft.logical_owner_id, &draft.operation_id)?,
        logical_owner_id: draft.logical_owner_id,
        name: draft.name,
        description: draft.description,
        state: ProjectStateV1::Planning,
        start_at: draft.start_at,
        target_at: draft.target_at,
        project_revision: 1,
        outcomes: Vec::new(),
        references: Vec::new(),
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_project_record_v1(&project)?;
    Ok(project)
}

pub fn update_project_v1(
    project: &mut ProjectRecordV1,
    expected_revision: u64,
    name: Option<String>,
    description: Option<String>,
    start_at: Option<ProjectTimestampV1>,
    target_at: Option<ProjectTimestampV1>,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_mutable(project, expected_revision, changed_at)?;
    if name.is_none() && description.is_none() && start_at.is_none() && target_at.is_none() {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    let next_name = name.unwrap_or_else(|| project.name.clone());
    let next_description = description.unwrap_or_else(|| project.description.clone());
    let next_start = start_at.or(project.start_at);
    let next_target = target_at.or(project.target_at);
    validate_text(&next_name, 1, MAX_NAME_CHARS_V1)?;
    validate_text(&next_description, 0, MAX_DESCRIPTION_CHARS_V1)?;
    validate_schedule(next_start, next_target)?;
    project.name = next_name;
    project.description = next_description;
    project.start_at = next_start;
    project.target_at = next_target;
    advance(project, changed_at)
}

pub fn set_project_state_v1(
    project: &mut ProjectRecordV1,
    expected_revision: u64,
    next: ProjectStateV1,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_revision_time(project, expected_revision, changed_at)?;
    let allowed = matches!(
        (project.state, next),
        (ProjectStateV1::Planning, ProjectStateV1::Active)
            | (ProjectStateV1::Active, ProjectStateV1::OnHold)
            | (ProjectStateV1::OnHold, ProjectStateV1::Active)
            | (ProjectStateV1::Active, ProjectStateV1::Completed)
            | (ProjectStateV1::OnHold, ProjectStateV1::Completed)
            | (ProjectStateV1::Completed, ProjectStateV1::Active)
            | (ProjectStateV1::Completed, ProjectStateV1::Archived)
            | (ProjectStateV1::Archived, ProjectStateV1::Active)
    );
    if !allowed {
        return Err(ProjectLifecycleErrorV1::InvalidStateTransition);
    }
    if next == ProjectStateV1::Completed
        && (project.outcomes.is_empty()
            || project
                .outcomes
                .iter()
                .any(|value| value.state == ProjectOutcomeStateV1::Pending))
    {
        return Err(ProjectLifecycleErrorV1::OutcomeConflict);
    }
    project.state = next;
    advance(project, changed_at)
}

#[allow(clippy::too_many_arguments)]
pub fn add_project_outcome_v1(
    project: &mut ProjectRecordV1,
    operation_id: [u8; 16],
    expected_revision: u64,
    title: String,
    description: String,
    target_at: Option<ProjectTimestampV1>,
    changed_at: ProjectTimestampV1,
) -> Result<ProjectOutcomeV1, ProjectLifecycleErrorV1> {
    require_mutable(project, expected_revision, changed_at)?;
    validate_text(&title, 1, MAX_OUTCOME_TITLE_CHARS_V1)?;
    validate_text(&description, 0, MAX_DESCRIPTION_CHARS_V1)?;
    if let Some(value) = target_at {
        validate_timestamp(value)?;
    }
    let outcome_id = derive_project_outcome_id_v1(&project.project_id, &operation_id)?;
    if project
        .outcomes
        .iter()
        .any(|value| value.outcome_id == outcome_id)
    {
        return Err(ProjectLifecycleErrorV1::OutcomeConflict);
    }
    let next_project_revision = checked_next(project.project_revision)?;
    let outcome = ProjectOutcomeV1 {
        outcome_id,
        project_id: project.project_id,
        title,
        description,
        state: ProjectOutcomeStateV1::Pending,
        target_at,
        outcome_revision: 1,
        updated_at_project_revision: next_project_revision,
        created_at: changed_at,
        updated_at: changed_at,
    };
    project.outcomes.push(outcome.clone());
    project.project_revision = next_project_revision;
    project.updated_at = changed_at;
    Ok(outcome)
}

#[allow(clippy::too_many_arguments)]
pub fn update_project_outcome_v1(
    project: &mut ProjectRecordV1,
    expected_project_revision: u64,
    outcome_id: [u8; 16],
    expected_outcome_revision: u64,
    title: Option<String>,
    description: Option<String>,
    target_at: Option<ProjectTimestampV1>,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_mutable(project, expected_project_revision, changed_at)?;
    let next_project_revision = checked_next(project.project_revision)?;
    let outcome = project
        .outcomes
        .iter_mut()
        .find(|value| value.outcome_id == outcome_id)
        .ok_or(ProjectLifecycleErrorV1::OutcomeNotFound)?;
    require_outcome(outcome, expected_outcome_revision, changed_at)?;
    if outcome.state != ProjectOutcomeStateV1::Pending
        || (title.is_none() && description.is_none() && target_at.is_none())
    {
        return Err(ProjectLifecycleErrorV1::OutcomeConflict);
    }
    let next_title = title.unwrap_or_else(|| outcome.title.clone());
    let next_description = description.unwrap_or_else(|| outcome.description.clone());
    validate_text(&next_title, 1, MAX_OUTCOME_TITLE_CHARS_V1)?;
    validate_text(&next_description, 0, MAX_DESCRIPTION_CHARS_V1)?;
    if let Some(value) = target_at {
        validate_timestamp(value)?;
    }
    outcome.title = next_title;
    outcome.description = next_description;
    outcome.target_at = target_at.or(outcome.target_at);
    outcome.outcome_revision = checked_next(outcome.outcome_revision)?;
    outcome.updated_at_project_revision = next_project_revision;
    outcome.updated_at = changed_at;
    project.project_revision = next_project_revision;
    project.updated_at = changed_at;
    Ok(())
}

pub fn set_project_outcome_state_v1(
    project: &mut ProjectRecordV1,
    expected_project_revision: u64,
    outcome_id: [u8; 16],
    expected_outcome_revision: u64,
    next: ProjectOutcomeStateV1,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_mutable(project, expected_project_revision, changed_at)?;
    let next_project_revision = checked_next(project.project_revision)?;
    let outcome = project
        .outcomes
        .iter_mut()
        .find(|value| value.outcome_id == outcome_id)
        .ok_or(ProjectLifecycleErrorV1::OutcomeNotFound)?;
    require_outcome(outcome, expected_outcome_revision, changed_at)?;
    if outcome.state != ProjectOutcomeStateV1::Pending || next == ProjectOutcomeStateV1::Pending {
        return Err(ProjectLifecycleErrorV1::OutcomeConflict);
    }
    outcome.state = next;
    outcome.outcome_revision = checked_next(outcome.outcome_revision)?;
    outcome.updated_at_project_revision = next_project_revision;
    outcome.updated_at = changed_at;
    project.project_revision = next_project_revision;
    project.updated_at = changed_at;
    Ok(())
}

pub fn remove_project_outcome_v1(
    project: &mut ProjectRecordV1,
    expected_project_revision: u64,
    outcome_id: [u8; 16],
    expected_outcome_revision: u64,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_mutable(project, expected_project_revision, changed_at)?;
    let index = project
        .outcomes
        .iter()
        .position(|value| value.outcome_id == outcome_id)
        .ok_or(ProjectLifecycleErrorV1::OutcomeNotFound)?;
    require_outcome(
        &project.outcomes[index],
        expected_outcome_revision,
        changed_at,
    )?;
    if !matches!(
        project.outcomes[index].state,
        ProjectOutcomeStateV1::Pending | ProjectOutcomeStateV1::Cancelled
    ) {
        return Err(ProjectLifecycleErrorV1::OutcomeConflict);
    }
    project.outcomes.remove(index);
    advance(project, changed_at)
}

pub fn add_project_reference_v1(
    project: &mut ProjectRecordV1,
    expected_revision: u64,
    kind: ProjectReferenceKindV1,
    public_id: [u8; 16],
    label: String,
    changed_at: ProjectTimestampV1,
) -> Result<ProjectReferenceV1, ProjectLifecycleErrorV1> {
    require_mutable(project, expected_revision, changed_at)?;
    validate_text(&label, 0, MAX_REFERENCE_LABEL_CHARS_V1)?;
    let reference_id = derive_project_reference_id_v1(&project.project_id, kind, &public_id)?;
    let next_project_revision = checked_next(project.project_revision)?;
    if let Some(existing) = project
        .references
        .iter_mut()
        .find(|value| value.reference_id == reference_id)
    {
        if existing.state == ProjectReferenceStateV1::Active {
            return Err(ProjectLifecycleErrorV1::ReferenceConflict);
        }
        existing.state = ProjectReferenceStateV1::Active;
        existing.label = label;
        existing.updated_at_project_revision = next_project_revision;
        project.project_revision = next_project_revision;
        project.updated_at = changed_at;
        return Ok(existing.clone());
    }
    let reference = ProjectReferenceV1 {
        reference_id,
        kind,
        public_id,
        label,
        state: ProjectReferenceStateV1::Active,
        updated_at_project_revision: next_project_revision,
    };
    project.references.push(reference.clone());
    project.project_revision = next_project_revision;
    project.updated_at = changed_at;
    Ok(reference)
}

pub fn remove_project_reference_v1(
    project: &mut ProjectRecordV1,
    expected_revision: u64,
    reference_id: [u8; 16],
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_mutable(project, expected_revision, changed_at)?;
    let next_project_revision = checked_next(project.project_revision)?;
    let reference = project
        .references
        .iter_mut()
        .find(|value| value.reference_id == reference_id)
        .ok_or(ProjectLifecycleErrorV1::ReferenceNotFound)?;
    if reference.state == ProjectReferenceStateV1::Removed {
        return Err(ProjectLifecycleErrorV1::ReferenceConflict);
    }
    reference.state = ProjectReferenceStateV1::Removed;
    reference.updated_at_project_revision = next_project_revision;
    project.project_revision = next_project_revision;
    project.updated_at = changed_at;
    Ok(())
}

pub fn validate_project_record_v1(value: &ProjectRecordV1) -> Result<(), ProjectLifecycleErrorV1> {
    if !nonzero(&value.project_id)
        || !valid_owner(&value.logical_owner_id)
        || value.project_revision == 0
        || value.updated_at < value.created_at
    {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    validate_text(&value.name, 1, MAX_NAME_CHARS_V1)?;
    validate_text(&value.description, 0, MAX_DESCRIPTION_CHARS_V1)?;
    validate_schedule(value.start_at, value.target_at)?;
    validate_timestamp(value.created_at)?;
    validate_timestamp(value.updated_at)
}

fn require_mutable(
    value: &ProjectRecordV1,
    revision: u64,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    require_revision_time(value, revision, changed_at)?;
    if matches!(
        value.state,
        ProjectStateV1::Completed | ProjectStateV1::Archived
    ) {
        return Err(ProjectLifecycleErrorV1::InvalidStateTransition);
    }
    Ok(())
}

fn require_revision_time(
    value: &ProjectRecordV1,
    revision: u64,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    if revision == 0 || revision != value.project_revision {
        return Err(ProjectLifecycleErrorV1::InvalidRevision);
    }
    validate_timestamp(changed_at)?;
    if changed_at < value.updated_at {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn require_outcome(
    value: &ProjectOutcomeV1,
    revision: u64,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    if revision == 0 || revision != value.outcome_revision || changed_at < value.updated_at {
        return Err(ProjectLifecycleErrorV1::OutcomeConflict);
    }
    Ok(())
}

fn advance(
    value: &mut ProjectRecordV1,
    changed_at: ProjectTimestampV1,
) -> Result<(), ProjectLifecycleErrorV1> {
    value.project_revision = checked_next(value.project_revision)?;
    value.updated_at = changed_at;
    Ok(())
}

fn checked_next(value: u64) -> Result<u64, ProjectLifecycleErrorV1> {
    value
        .checked_add(1)
        .ok_or(ProjectLifecycleErrorV1::RevisionOverflow)
}

fn validate_schedule(
    start_at: Option<ProjectTimestampV1>,
    target_at: Option<ProjectTimestampV1>,
) -> Result<(), ProjectLifecycleErrorV1> {
    if let Some(value) = start_at {
        validate_timestamp(value)?;
    }
    if let Some(value) = target_at {
        validate_timestamp(value)?;
    }
    if start_at
        .zip(target_at)
        .is_some_and(|(start, target)| target <= start)
    {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_timestamp(value: ProjectTimestampV1) -> Result<(), ProjectLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_text(value: &str, min: usize, max: usize) -> Result<(), ProjectLifecycleErrorV1> {
    let count = value.chars().count();
    if count < min || count > max || value.contains('\0') {
        return Err(ProjectLifecycleErrorV1::InvalidInput);
    }
    Ok(())
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_OWNER_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn reference_kind_code(value: ProjectReferenceKindV1) -> u8 {
    match value {
        ProjectReferenceKindV1::Person => 1,
        ProjectReferenceKindV1::Organization => 2,
        ProjectReferenceKindV1::Relationship => 3,
        ProjectReferenceKindV1::Task => 4,
        ProjectReferenceKindV1::Document => 5,
        ProjectReferenceKindV1::CalendarEvent => 6,
    }
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn derive_id(domain: &[u8], parts: &[&[u8]]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize()[..16].try_into().expect("fixed digest")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: i64) -> ProjectTimestampV1 {
        ProjectTimestampV1 {
            unix_seconds: value,
            nanos: 0,
        }
    }

    fn project() -> ProjectRecordV1 {
        create_project_v1(ProjectDraftV1 {
            operation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            name: "Launch".to_owned(),
            description: "Bounded outcome".to_owned(),
            start_at: Some(at(10)),
            target_at: Some(at(20)),
            created_at: at(9),
        })
        .expect("project")
    }

    #[test]
    fn project_identity_is_owner_and_operation_scoped() {
        let one = project();
        let two = project();
        assert_eq!(one.project_id, two.project_id);
        assert_eq!(one.state, ProjectStateV1::Planning);
    }

    #[test]
    fn completion_requires_terminal_expected_outcomes() {
        let mut value = project();
        set_project_state_v1(&mut value, 1, ProjectStateV1::Active, at(10)).expect("active");
        let outcome = add_project_outcome_v1(
            &mut value,
            [2; 16],
            2,
            "Shipped".to_owned(),
            String::new(),
            Some(at(20)),
            at(11),
        )
        .expect("outcome");
        assert_eq!(
            set_project_state_v1(&mut value, 3, ProjectStateV1::Completed, at(12)),
            Err(ProjectLifecycleErrorV1::OutcomeConflict)
        );
        set_project_outcome_state_v1(
            &mut value,
            3,
            outcome.outcome_id,
            1,
            ProjectOutcomeStateV1::Achieved,
            at(12),
        )
        .expect("achieved");
        set_project_state_v1(&mut value, 4, ProjectStateV1::Completed, at(13)).expect("complete");
    }

    #[test]
    fn typed_reference_is_stable_and_removable() {
        let mut value = project();
        let reference = add_project_reference_v1(
            &mut value,
            1,
            ProjectReferenceKindV1::Document,
            [8; 16],
            "Requirements".to_owned(),
            at(10),
        )
        .expect("reference");
        remove_project_reference_v1(&mut value, 2, reference.reference_id, at(11)).expect("remove");
        assert_eq!(value.references[0].state, ProjectReferenceStateV1::Removed);
    }

    #[test]
    fn invalid_schedule_and_overflow_fail_closed() {
        let mut value = project();
        assert_eq!(
            update_project_v1(
                &mut value,
                1,
                None,
                None,
                Some(at(30)),
                Some(at(20)),
                at(10)
            ),
            Err(ProjectLifecycleErrorV1::InvalidInput)
        );
        value.project_revision = u64::MAX;
        assert_eq!(
            set_project_state_v1(&mut value, u64::MAX, ProjectStateV1::Active, at(10)),
            Err(ProjectLifecycleErrorV1::RevisionOverflow)
        );
    }
}
