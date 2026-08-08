use crate::domains::graph::core::models::GraphReviewState;
use makosh_projects_api::ProjectLinkReviewState;

use super::constants::PROJECT_KEYWORD_CONFIDENCE;

pub(super) fn normalize_email_address(email_address: &str) -> String {
    email_address.trim().to_ascii_lowercase()
}

pub(super) fn project_review_graph_state(review_state: ProjectLinkReviewState) -> GraphReviewState {
    match review_state {
        ProjectLinkReviewState::Suggested => GraphReviewState::Suggested,
        ProjectLinkReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        ProjectLinkReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

pub(super) fn project_review_confidence(review_state: ProjectLinkReviewState) -> f64 {
    match review_state {
        ProjectLinkReviewState::Suggested => PROJECT_KEYWORD_CONFIDENCE,
        ProjectLinkReviewState::UserConfirmed => 1.0,
        ProjectLinkReviewState::UserRejected => 0.0,
    }
}
