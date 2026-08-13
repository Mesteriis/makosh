#![forbid(unsafe_code)]

mod creation;
mod lifecycle;
mod model;

pub use creation::{TaskCreationErrorV1, create_task_from_reviewed_candidate_v1};
pub use lifecycle::{
    MAX_CHECKLIST_LABEL_CHARS_V1, MAX_DESCRIPTION_CHARS_V1, ManualTaskDraftV1, TaskChecklistItemV1,
    TaskDependencyV1, TaskLifecycleErrorV1, TaskLifecycleStateV1, TaskPriorityV1, TaskRecordV1,
    add_checklist_item_v1, add_task_dependency_v1, create_manual_task_v1, derive_manual_task_id_v1,
    remove_checklist_item_v1, remove_task_dependency_v1, set_task_priority_v1, set_task_state_v1,
    update_checklist_item_v1, update_task_content_v1, validate_task_record_v1,
};
pub use model::{
    ReviewedCandidateTaskDraftV1, TaskProvenanceV1, TaskStatusV1, TaskTimestampV1, TaskV1,
    TasksValidationErrorV1, derive_task_id_v1, task_creation_fingerprint_v1, validate_task_v1,
};

pub const PACKAGE: &str = "makosh-tasks-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_TITLE_CHARS_V1: usize = 240;
pub const MAX_HINT_CHARS_V1: usize = 120;

#[cfg(test)]
mod lifecycle_contract_tests {
    use super::*;

    fn timestamp(seconds: i64) -> TaskTimestampV1 {
        TaskTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }

    #[test]
    fn lifecycle_is_revisioned_closed_and_dependency_safe() {
        let mut task = create_manual_task_v1(ManualTaskDraftV1 {
            operation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            title: "Ship the lifecycle".to_owned(),
            description: Some("Owner-authored detail".to_owned()),
            due_at: None,
            priority: TaskPriorityV1::Normal,
            created_at: timestamp(10),
        })
        .expect("create");
        assert_eq!(task.state, TaskLifecycleStateV1::Open);
        assert_eq!(task.task_revision, 1);

        set_task_state_v1(
            &mut task,
            1,
            TaskLifecycleStateV1::InProgress,
            timestamp(11),
        )
        .expect("state");
        set_task_priority_v1(&mut task, 2, TaskPriorityV1::High, timestamp(12)).expect("priority");
        add_task_dependency_v1(&mut task, 3, [2; 16], [3; 16], false, timestamp(13))
            .expect("dependency");
        assert_eq!(task.task_revision, 4);

        let task_id = task.task_id;
        assert_eq!(
            add_task_dependency_v1(&mut task, 4, [4; 16], task_id, false, timestamp(14)),
            Err(TaskLifecycleErrorV1::SelfDependency)
        );
        assert_eq!(
            add_task_dependency_v1(&mut task, 4, [4; 16], [5; 16], true, timestamp(14)),
            Err(TaskLifecycleErrorV1::DependencyCycle)
        );
        assert_eq!(
            set_task_state_v1(&mut task, 3, TaskLifecycleStateV1::Completed, timestamp(14)),
            Err(TaskLifecycleErrorV1::RevisionConflict)
        );
    }

    #[test]
    fn checklist_has_stable_ids_and_checked_revisions() {
        let mut task = create_manual_task_v1(ManualTaskDraftV1 {
            operation_id: [8; 16],
            logical_owner_id: "owner-1".to_owned(),
            title: "Checklist".to_owned(),
            description: None,
            due_at: None,
            priority: TaskPriorityV1::Low,
            created_at: timestamp(10),
        })
        .expect("create");
        add_checklist_item_v1(&mut task, 1, [9; 16], "First".to_owned(), 10, timestamp(11))
            .expect("add");
        update_checklist_item_v1(
            &mut task,
            2,
            [9; 16],
            None,
            Some(true),
            Some(20),
            timestamp(12),
        )
        .expect("update");
        assert!(task.checklist[0].completed);
        assert_eq!(task.checklist[0].position, 20);
        assert_eq!(task.checklist[0].updated_at_task_revision, 3);
        assert_eq!(
            remove_checklist_item_v1(&mut task, u64::MAX, [9; 16], timestamp(13)),
            Err(TaskLifecycleErrorV1::RevisionConflict)
        );
    }
}
