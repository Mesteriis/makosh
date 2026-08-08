use serde::{Deserialize, Serialize};

use makosh_decisions_api::DecisionRead;

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DecisionListResponse {
    pub(crate) items: Vec<DecisionRead>,
}
