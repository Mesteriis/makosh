#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-task-candidate-api";
pub const COMMUNICATION_TASK_CANDIDATE_OWNER_V1: &str = "communication_task_candidate_extraction";
pub const COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1: &str =
    "makosh-communication-task-candidate-runtime";
pub const COMMUNICATION_TASK_CANDIDATE_CAPABILITY_ID_V1: &str =
    "communication.task-candidate-extraction.v1";
pub const COMMUNICATION_TASK_CANDIDATE_COMMAND_CONTRACT_NAME_V1: &str =
    "communication.task-candidate-extraction.command";
pub const COMMUNICATION_TASK_CANDIDATE_QUERY_CONTRACT_NAME_V1: &str =
    "communication.task-candidate-extraction.query";
pub const COMMUNICATION_TASK_CANDIDATE_REALTIME_CONTRACT_NAME_V1: &str =
    "communication.task-candidate-extraction.status_changed";
pub const COMMUNICATION_TASK_CANDIDATE_REALTIME_EVENT_KIND_V1: &str =
    "communication.task-candidate-extraction.status_changed";
pub const COMMUNICATION_TASK_CANDIDATE_COMMAND_CONNECT_PATH_V1: &str =
    "/makosh.communication_task_candidate.v1.CommunicationTaskCandidateCommandService/Start";
pub const COMMUNICATION_TASK_CANDIDATE_QUERY_CONNECT_PATH_V1: &str =
    "/makosh.communication_task_candidate.v1.CommunicationTaskCandidateQueryService/Get";
pub const COMMUNICATION_TASK_CANDIDATE_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_TASK_CANDIDATE_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_TASK_CANDIDATE_MAX_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_TASK_CANDIDATE_MAX_CANDIDATES_V1: usize = 16;

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_task_candidate.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_task_candidate_schema.rs"
));

pub const COMMUNICATION_TASK_CANDIDATE_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-task-candidate-v1.bin"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_contract_is_bounded_typed_and_owner_neutral() {
        assert!(COMMUNICATION_TASK_CANDIDATE_COMMAND_CONNECT_PATH_V1.starts_with('/'));
        assert!(COMMUNICATION_TASK_CANDIDATE_QUERY_CONNECT_PATH_V1.starts_with('/'));
        let source =
            include_str!("../proto/makosh/communication_task_candidate/v1/task_candidate.proto");
        assert!(source.contains("COMMUNICATION_TASK_SIGNAL_KIND_EXPLICIT_ACTION"));
        assert!(source.contains("COMMUNICATION_TASK_SIGNAL_KIND_DIRECT_REQUEST"));
        assert!(source.contains("COMMUNICATION_TASK_SIGNAL_KIND_FOLLOW_UP"));
        assert!(source.contains("candidate_digest"));
        assert!(source.contains("source_evidence_id"));
        for forbidden in [
            "project_id",
            "contact_id",
            "persona_id",
            "organization_id",
            "provider_id",
            "account_id",
            "model_id",
            "prompt",
            "source_body",
            "map<",
            "google",
            "telegram",
            "ollama",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden client field {forbidden}"
            );
        }
    }
}
