use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1, COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1,
    COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1, CommunicationNoteCandidateV1,
    CommunicationNoteSourceBasisV1, CommunicationNoteSourceContentV1, CommunicationNoteTopicHintV1,
};

const FINANCIAL_MARKERS: &[&str] = &[
    "invoice",
    "payment",
    "amount",
    "счёт",
    "счет",
    "оплата",
    "сумма",
    "factura",
    "pago",
    "importe",
    "paiement",
    "montant",
    "rechnung",
    "zahlung",
    "betrag",
];
const LEGAL_MARKERS: &[&str] = &[
    "contract",
    "agreement",
    "nda",
    "договор",
    "соглашение",
    "contrato",
    "acuerdo",
    "contrat",
    "accord",
    "vertrag",
    "vereinbarung",
];
const DECISION_MARKERS: &[&str] = &[
    "decided",
    "approved",
    "confirmed",
    "решили",
    "одобрено",
    "подтверждено",
    "подтвердили",
    "decidido",
    "aprobado",
    "confirmado",
    "décidé",
    "decide",
    "approuvé",
    "approuve",
    "confirmé",
    "confirme",
    "beschlossen",
    "genehmigt",
    "bestätigt",
    "bestaetigt",
];
const DEADLINE_MARKERS: &[&str] = &[
    "deadline",
    "due date",
    " by ",
    "срок",
    "до ",
    "fecha límite",
    "fecha limite",
    "para el ",
    "échéance",
    "echeance",
    " avant ",
    "frist",
    " bis ",
];
const FALLBACK_TITLE: &str = "Communication note";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteExtractionErrorV1 {
    InvalidSourceEvidenceId,
    InvalidSourceEvidenceRevision,
    SourceLimit,
    InvalidUtf8,
}

pub fn extract_communication_note_candidates_v1(
    content: CommunicationNoteSourceContentV1<'_>,
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
) -> Result<Vec<CommunicationNoteCandidateV1>, CommunicationNoteExtractionErrorV1> {
    validate_source(content, source_evidence_id, source_evidence_revision)?;
    let subject = std::str::from_utf8(content.subject_utf8)
        .map_err(|_| CommunicationNoteExtractionErrorV1::InvalidUtf8)?;
    let body = std::str::from_utf8(content.body_utf8)
        .map_err(|_| CommunicationNoteExtractionErrorV1::InvalidUtf8)?;
    let normalized = normalize(&format!("{subject}\n{body}"));
    let topic_hints = topic_hints(&normalized);
    if topic_hints.is_empty() {
        return Ok(Vec::new());
    }

    let title = subject
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map_or_else(
            || FALLBACK_TITLE.to_owned(),
            |line| truncate_chars(line, COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1),
        );
    let excerpt = excerpt(body);
    let subject_has_marker = contains_marker(&normalize(subject));
    let body_has_marker = contains_marker(&normalize(body));
    let source_basis = match (subject_has_marker, body_has_marker) {
        (true, true) => CommunicationNoteSourceBasisV1::Combined,
        (true, false) => CommunicationNoteSourceBasisV1::Subject,
        (false, true) => CommunicationNoteSourceBasisV1::Body,
        (false, false) => unreachable!("candidate requires at least one marker"),
    };
    let confidence_basis_points = confidence(&topic_hints);
    let mut candidate = CommunicationNoteCandidateV1 {
        candidate_id: [0; 16],
        candidate_digest: [0; 32],
        title,
        excerpt,
        topic_hints,
        source_basis,
        confidence_basis_points,
        source_evidence_id,
        source_evidence_revision,
    };
    refresh_identity(&mut candidate);
    Ok(vec![candidate])
}

fn validate_source(
    content: CommunicationNoteSourceContentV1<'_>,
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
) -> Result<(), CommunicationNoteExtractionErrorV1> {
    if source_evidence_id.iter().all(|byte| *byte == 0) {
        return Err(CommunicationNoteExtractionErrorV1::InvalidSourceEvidenceId);
    }
    if source_evidence_revision == 0 {
        return Err(CommunicationNoteExtractionErrorV1::InvalidSourceEvidenceRevision);
    }
    if content
        .subject_utf8
        .len()
        .checked_add(content.body_utf8.len())
        .is_none_or(|bytes| bytes > COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1)
    {
        return Err(CommunicationNoteExtractionErrorV1::SourceLimit);
    }
    Ok(())
}

fn topic_hints(normalized: &str) -> Vec<CommunicationNoteTopicHintV1> {
    let mut result = Vec::with_capacity(4);
    if contains_any(normalized, FINANCIAL_MARKERS) {
        result.push(CommunicationNoteTopicHintV1::Financial);
    }
    if contains_any(normalized, LEGAL_MARKERS) {
        result.push(CommunicationNoteTopicHintV1::Legal);
    }
    if contains_any(normalized, DECISION_MARKERS) {
        result.push(CommunicationNoteTopicHintV1::DecisionStatement);
    }
    if contains_any(normalized, DEADLINE_MARKERS) {
        result.push(CommunicationNoteTopicHintV1::DeadlineStatement);
    }
    result
}

fn contains_marker(normalized: &str) -> bool {
    !topic_hints(normalized).is_empty()
}

fn excerpt(body: &str) -> String {
    let value = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    truncate_chars(&value, COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1)
}

fn confidence(hints: &[CommunicationNoteTopicHintV1]) -> u32 {
    match hints.len() {
        0 => 0,
        1 => 7_500,
        2 => 8_300,
        3 => 9_000,
        _ => 9_400,
    }
}

fn refresh_identity(candidate: &mut CommunicationNoteCandidateV1) {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication-note-candidate.digest.v1\0");
    digest.update(candidate.source_evidence_id);
    digest.update(candidate.source_evidence_revision.to_be_bytes());
    digest.update([basis_byte(candidate.source_basis)]);
    digest.update(candidate.title.as_bytes());
    digest.update([0]);
    digest.update(candidate.excerpt.as_bytes());
    digest.update([0]);
    for hint in &candidate.topic_hints {
        digest.update([topic_byte(*hint)]);
    }
    candidate.candidate_digest = digest.finalize().into();

    let mut identity = Sha256::new();
    identity.update(b"makosh.communication-note-candidate.id.v1\0");
    identity.update(candidate.candidate_digest);
    let identity: [u8; 32] = identity.finalize().into();
    candidate.candidate_id.copy_from_slice(&identity[..16]);
}

fn basis_byte(value: CommunicationNoteSourceBasisV1) -> u8 {
    match value {
        CommunicationNoteSourceBasisV1::Subject => 1,
        CommunicationNoteSourceBasisV1::Body => 2,
        CommunicationNoteSourceBasisV1::Combined => 3,
    }
}

fn topic_byte(value: CommunicationNoteTopicHintV1) -> u8 {
    match value {
        CommunicationNoteTopicHintV1::Financial => 1,
        CommunicationNoteTopicHintV1::Legal => 2,
        CommunicationNoteTopicHintV1::DecisionStatement => 3,
        CommunicationNoteTopicHintV1::DeadlineStatement => 4,
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn normalize(value: &str) -> String {
    format!(
        " {} ",
        value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    )
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(subject: &str, body: &str) -> Vec<CommunicationNoteCandidateV1> {
        extract_communication_note_candidates_v1(
            CommunicationNoteSourceContentV1 {
                subject_utf8: subject.as_bytes(),
                body_utf8: body.as_bytes(),
            },
            [7; 16],
            4,
        )
        .expect("valid source")
    }

    #[test]
    fn legacy_markers_produce_one_bounded_review_candidate() {
        let candidates = extract(
            "Contract approved",
            "Invoice amount: 42\nPayment by Friday\nLine 3\nLine 4\nLine 5\nLine 6",
        );
        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.title, "Contract approved");
        assert_eq!(candidate.excerpt.lines().count(), 5);
        assert_eq!(
            candidate.topic_hints,
            vec![
                CommunicationNoteTopicHintV1::Financial,
                CommunicationNoteTopicHintV1::Legal,
                CommunicationNoteTopicHintV1::DecisionStatement,
                CommunicationNoteTopicHintV1::DeadlineStatement,
            ]
        );
        assert_eq!(
            candidate.source_basis,
            CommunicationNoteSourceBasisV1::Combined
        );
        assert_ne!(candidate.candidate_id, [0; 16]);
        assert_ne!(candidate.candidate_digest, [0; 32]);
    }

    #[test]
    fn empty_source_does_not_fabricate_a_note_candidate() {
        assert!(extract("Weekly update", "Thanks for the information").is_empty());
    }

    #[test]
    fn body_only_marker_uses_neutral_fallback_and_body_basis() {
        let candidates = extract("", "Решили оставить текущий договор");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].title, FALLBACK_TITLE);
        assert_eq!(
            candidates[0].source_basis,
            CommunicationNoteSourceBasisV1::Body
        );
    }

    #[test]
    fn stable_source_produces_stable_candidate_identity() {
        assert_eq!(
            extract("Payment terms", "Amount confirmed"),
            extract("Payment terms", "Amount confirmed")
        );
    }

    #[test]
    fn invalid_utf8_and_oversized_source_are_rejected() {
        assert_eq!(
            extract_communication_note_candidates_v1(
                CommunicationNoteSourceContentV1 {
                    subject_utf8: &[0xff],
                    body_utf8: &[],
                },
                [1; 16],
                1,
            ),
            Err(CommunicationNoteExtractionErrorV1::InvalidUtf8)
        );
        let oversized = vec![b'a'; COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1 + 1];
        assert_eq!(
            extract_communication_note_candidates_v1(
                CommunicationNoteSourceContentV1 {
                    subject_utf8: &[],
                    body_utf8: &oversized,
                },
                [1; 16],
                1,
            ),
            Err(CommunicationNoteExtractionErrorV1::SourceLimit)
        );
    }
}
