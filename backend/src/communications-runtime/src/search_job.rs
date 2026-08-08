//! Converts a canonical domain search decision into an owner-local durable job.

use makosh_communications_domain::{
    CommunicationsSearchIndexDecisionV1, CommunicationsSearchIndexRejectionV1,
};
use makosh_communications_persistence::{
    CommunicationsDerivedIndexFailureRecordV1, CommunicationsDerivedIndexFailureV1,
    CommunicationsDerivedIndexJobOperationV1, CommunicationsDerivedIndexJobV1,
    communications_derived_index_job_id_v1,
};

pub struct CommunicationsDerivedIndexWorkV1 {
    pub job: Option<CommunicationsDerivedIndexJobV1>,
    pub failure: Option<CommunicationsDerivedIndexFailureRecordV1>,
}

pub fn derived_index_work_from_decision_v1(
    decision: CommunicationsSearchIndexDecisionV1,
    created_at_unix_seconds: i64,
) -> Option<CommunicationsDerivedIndexWorkV1> {
    match decision {
        CommunicationsSearchIndexDecisionV1::Ignore => None,
        CommunicationsSearchIndexDecisionV1::Index(job) => Some(CommunicationsDerivedIndexWorkV1 {
            job: Some(CommunicationsDerivedIndexJobV1 {
                job_id: communications_derived_index_job_id_v1(
                    job.evidence_id.bytes(),
                    job.message_id.bytes(),
                    job.projection_revision,
                ),
                operation: CommunicationsDerivedIndexJobOperationV1::Index,
                evidence_id: job.evidence_id,
                message_id: job.message_id,
                conversation_id: Some(job.conversation_id),
                blob: Some(job.blob),
                projection_revision: job.projection_revision,
                observed_at_unix_seconds: job.observed_at_unix_seconds,
                created_at_unix_seconds,
            }),
            failure: None,
        }),
        CommunicationsSearchIndexDecisionV1::Remove {
            evidence_id,
            message_id,
            projection_revision,
            observed_at_unix_seconds,
        } => Some(CommunicationsDerivedIndexWorkV1 {
            job: Some(CommunicationsDerivedIndexJobV1 {
                job_id: communications_derived_index_job_id_v1(
                    evidence_id.bytes(),
                    message_id.bytes(),
                    projection_revision,
                ),
                operation: CommunicationsDerivedIndexJobOperationV1::Remove,
                evidence_id,
                message_id,
                conversation_id: None,
                blob: None,
                projection_revision,
                observed_at_unix_seconds,
                created_at_unix_seconds,
            }),
            failure: None,
        }),
        CommunicationsSearchIndexDecisionV1::Reject {
            evidence_id,
            message_id,
            projection_revision,
            observed_at_unix_seconds,
            reason,
        } => Some(CommunicationsDerivedIndexWorkV1 {
            job: Some(CommunicationsDerivedIndexJobV1 {
                job_id: communications_derived_index_job_id_v1(
                    evidence_id.bytes(),
                    message_id.bytes(),
                    projection_revision,
                ),
                operation: CommunicationsDerivedIndexJobOperationV1::Remove,
                evidence_id,
                message_id,
                conversation_id: None,
                blob: None,
                projection_revision,
                observed_at_unix_seconds,
                created_at_unix_seconds,
            }),
            failure: Some(CommunicationsDerivedIndexFailureRecordV1 {
                evidence_id,
                message_id,
                projection_revision,
                observed_at_unix_seconds,
                failure: match reason {
                    CommunicationsSearchIndexRejectionV1::DocumentLimit => {
                        CommunicationsDerivedIndexFailureV1::DocumentLimit
                    }
                },
                recorded_at_unix_seconds: created_at_unix_seconds,
            }),
        }),
    }
}

#[cfg(test)]
mod tests {
    use makosh_communications_api::{CommunicationMessageIdV1, CommunicationObservationIdV1};

    use super::*;

    #[test]
    fn document_limit_rejection_removes_the_prior_projection() {
        let work = derived_index_work_from_decision_v1(
            CommunicationsSearchIndexDecisionV1::Reject {
                evidence_id: CommunicationObservationIdV1::new([1; 16]),
                message_id: CommunicationMessageIdV1::new([2; 16]),
                projection_revision: 1,
                observed_at_unix_seconds: 10,
                reason: CommunicationsSearchIndexRejectionV1::DocumentLimit,
            },
            11,
        )
        .expect("work");
        assert_eq!(
            work.job.as_ref().map(|job| job.operation),
            Some(CommunicationsDerivedIndexJobOperationV1::Remove)
        );
        assert_eq!(
            work.failure.as_ref().map(|failure| failure.failure),
            Some(CommunicationsDerivedIndexFailureV1::DocumentLimit)
        );
    }
}
