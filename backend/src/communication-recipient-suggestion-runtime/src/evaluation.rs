use std::os::unix::net::UnixStream;

use makosh_communication_recipient_suggestion_core::{
    CommunicationRecipientSuggestionRejectionCodeV1, CommunicationRecipientSuggestionStateV1,
    CommunicationRecipientSuggestionTransitionV1, evaluate_communication_recipient_candidates_v1,
};
use makosh_communication_recipient_suggestion_persistence::{
    CommunicationRecipientSuggestionPersistenceErrorV1,
    CommunicationRecipientSuggestionPersistenceV1, PersistedCommunicationRecipientSuggestionRunV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};

use crate::blob_materialization::{
    CommunicationRecipientSuggestionBlobErrorV1, read_recipient_source_v1,
    release_recipient_source_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionEvaluationErrorV1 {
    Blob(CommunicationRecipientSuggestionBlobErrorV1),
    InvalidState,
    Persistence(CommunicationRecipientSuggestionPersistenceErrorV1),
}

pub async fn complete_communication_recipient_suggestion_evaluation_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    run: &PersistedCommunicationRecipientSuggestionRunV1,
    body_utf8: &[u8],
    occurred_at_unix_millis: i64,
) -> Result<
    PersistedCommunicationRecipientSuggestionRunV1,
    CommunicationRecipientSuggestionEvaluationErrorV1,
> {
    if run.status.state != CommunicationRecipientSuggestionStateV1::Evaluating {
        return Err(CommunicationRecipientSuggestionEvaluationErrorV1::InvalidState);
    }
    let source_sha256 = run
        .status
        .source_sha256
        .ok_or(CommunicationRecipientSuggestionEvaluationErrorV1::InvalidState)?;
    let transition = match evaluate_communication_recipient_candidates_v1(body_utf8, source_sha256)
    {
        Ok(candidates) => CommunicationRecipientSuggestionTransitionV1::Complete {
            source_sha256,
            candidates,
        },
        Err(_) => CommunicationRecipientSuggestionTransitionV1::Reject(
            CommunicationRecipientSuggestionRejectionCodeV1::EvaluationRejected,
        ),
    };
    persistence
        .persist_evaluation_transition(
            &run.logical_owner_id,
            &run.draft.run_id,
            transition,
            occurred_at_unix_millis,
        )
        .await
        .map_err(CommunicationRecipientSuggestionEvaluationErrorV1::Persistence)
}

pub async fn recover_accepted_communication_recipient_suggestion_once_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, CommunicationRecipientSuggestionEvaluationErrorV1> {
    let Some(run) = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(CommunicationRecipientSuggestionEvaluationErrorV1::Persistence)?
        .into_iter()
        .find(|run| run.status.state != CommunicationRecipientSuggestionStateV1::PreparingSource)
    else {
        return Ok(false);
    };
    match run.status.state {
        CommunicationRecipientSuggestionStateV1::Accepted => {
            persistence
                .begin_source_preparation(
                    logical_owner_id,
                    &run.draft.run_id,
                    occurred_at_unix_millis,
                )
                .await
                .map_err(CommunicationRecipientSuggestionEvaluationErrorV1::Persistence)?;
        }
        CommunicationRecipientSuggestionStateV1::PreparingSource => unreachable!("filtered above"),
        CommunicationRecipientSuggestionStateV1::Evaluating => {
            let cleanup = run
                .source_cleanup
                .as_ref()
                .ok_or(CommunicationRecipientSuggestionEvaluationErrorV1::InvalidState)?;
            let body = read_recipient_source_v1(channel, dispatcher, cleanup)
                .map_err(CommunicationRecipientSuggestionEvaluationErrorV1::Blob)?;
            let terminal = complete_communication_recipient_suggestion_evaluation_v1(
                persistence,
                &run,
                body.as_slice(),
                occurred_at_unix_millis,
            )
            .await?;
            release_recipient_source_v1(
                channel,
                dispatcher,
                run.draft.run_id,
                cleanup,
                terminal.status.state == CommunicationRecipientSuggestionStateV1::Ready,
            )
            .map_err(CommunicationRecipientSuggestionEvaluationErrorV1::Blob)?;
            persistence
                .complete_blob_cleanup(
                    logical_owner_id,
                    &run.draft.run_id,
                    cleanup,
                    occurred_at_unix_millis,
                )
                .await
                .map_err(CommunicationRecipientSuggestionEvaluationErrorV1::Persistence)?;
        }
        CommunicationRecipientSuggestionStateV1::Ready
        | CommunicationRecipientSuggestionStateV1::Rejected => {
            return Err(CommunicationRecipientSuggestionEvaluationErrorV1::InvalidState);
        }
    }
    Ok(true)
}
