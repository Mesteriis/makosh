use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATION_TASK_MAX_CANDIDATES_V1, COMMUNICATION_TASK_MAX_HINT_CHARS_V1,
    COMMUNICATION_TASK_MAX_TITLE_CHARS_V1, COMMUNICATION_TASK_SOURCE_MAX_BYTES_V1,
    CommunicationTaskCandidateV1, CommunicationTaskSignalKindV1, CommunicationTaskSourceBasisV1,
    CommunicationTaskSourceContentV1,
};

const EXPLICIT_ACTION_MARKERS: &[&str] = &[
    "action:",
    "action required",
    "task:",
    "todo:",
    "действие:",
    "задача:",
    "acción:",
    "accion:",
    "tarea:",
    "à faire:",
    "a faire:",
    "aufgabe:",
    "aktion:",
];

const DIRECT_REQUEST_MARKERS: &[&str] = &[
    "can you ",
    "could you ",
    "would you ",
    "можешь ",
    "можете ",
    "сможешь ",
    "puedes ",
    "podrías ",
    "podrias ",
    "peux-tu ",
    "pouvez-vous ",
    "kannst du ",
    "könntest du ",
    "koenntest du ",
];

const ACTION_TERMS: &[&str] = &[
    "check",
    "review",
    "prepare",
    "send",
    "schedule",
    "update",
    "confirm",
    "draft",
    "create",
    "fix",
    "investigate",
    "провер",
    "подготов",
    "отправ",
    "запланир",
    "обнов",
    "подтверд",
    "сдела",
    "preparar",
    "revisar",
    "enviar",
    "actualizar",
    "préparer",
    "preparer",
    "vérifier",
    "verifier",
    "envoyer",
    "prüfen",
    "pruefen",
    "vorbereiten",
    "senden",
];

const FOLLOW_UP_MARKERS: &[&str] = &[
    "follow up",
    "follow-up",
    "next step",
    "следующий шаг",
    "нужно ",
    "надо ",
    "seguimiento",
    "siguiente paso",
    "prochaine étape",
    "prochaine etape",
    "nachfassen",
    "nächster schritt",
    "naechster schritt",
];

const DUE_SEPARATORS: &[&str] = &[
    " due: ",
    " deadline: ",
    " до ",
    " by ",
    " para ",
    " avant ",
    " bis ",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskExtractionErrorV1 {
    InvalidSourceEvidenceId,
    InvalidSourceEvidenceRevision,
    SourceLimit,
    InvalidUtf8,
}

struct SourceContext<'a> {
    evidence_id: [u8; 16],
    evidence_revision: u64,
    global_due_hint: Option<&'a str>,
    global_assignee_hint: Option<&'a str>,
}

pub fn extract_communication_task_candidates_v1(
    content: CommunicationTaskSourceContentV1<'_>,
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
) -> Result<Vec<CommunicationTaskCandidateV1>, CommunicationTaskExtractionErrorV1> {
    if source_evidence_id.iter().all(|byte| *byte == 0) {
        return Err(CommunicationTaskExtractionErrorV1::InvalidSourceEvidenceId);
    }
    if source_evidence_revision == 0 {
        return Err(CommunicationTaskExtractionErrorV1::InvalidSourceEvidenceRevision);
    }
    if content
        .subject_utf8
        .len()
        .checked_add(content.body_utf8.len())
        .is_none_or(|bytes| bytes > COMMUNICATION_TASK_SOURCE_MAX_BYTES_V1)
    {
        return Err(CommunicationTaskExtractionErrorV1::SourceLimit);
    }
    let subject = std::str::from_utf8(content.subject_utf8)
        .map_err(|_| CommunicationTaskExtractionErrorV1::InvalidUtf8)?;
    let body = std::str::from_utf8(content.body_utf8)
        .map_err(|_| CommunicationTaskExtractionErrorV1::InvalidUtf8)?;
    let global_due_hint = subject
        .lines()
        .chain(body.lines())
        .find_map(|line| due_hint(&normalize(line)));
    let global_assignee_hint = subject
        .lines()
        .chain(body.lines())
        .find_map(|line| assignee_hint(&normalize(line)));

    let mut candidates = Vec::new();
    let mut positions = BTreeMap::<String, usize>::new();
    let source = SourceContext {
        evidence_id: source_evidence_id,
        evidence_revision: source_evidence_revision,
        global_due_hint: global_due_hint.as_deref(),
        global_assignee_hint: global_assignee_hint.as_deref(),
    };
    collect_lines(
        subject,
        CommunicationTaskSourceBasisV1::Subject,
        &source,
        &mut positions,
        &mut candidates,
    );
    collect_lines(
        body,
        CommunicationTaskSourceBasisV1::Body,
        &source,
        &mut positions,
        &mut candidates,
    );
    candidates.truncate(COMMUNICATION_TASK_MAX_CANDIDATES_V1);
    Ok(candidates)
}

fn collect_lines(
    text: &str,
    basis: CommunicationTaskSourceBasisV1,
    source: &SourceContext<'_>,
    positions: &mut BTreeMap<String, usize>,
    candidates: &mut Vec<CommunicationTaskCandidateV1>,
) {
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let normalized = normalize(line);
        let Some(signal_kind) = signal_kind(&normalized) else {
            continue;
        };
        let title = truncate_chars(line, COMMUNICATION_TASK_MAX_TITLE_CHARS_V1);
        let key = normalize(&title);
        if let Some(position) = positions.get(&key).copied() {
            let candidate = &mut candidates[position];
            if candidate.source_basis != basis {
                candidate.source_basis = CommunicationTaskSourceBasisV1::Combined;
                refresh_identity(candidate);
            }
            continue;
        }
        if candidates.len() == COMMUNICATION_TASK_MAX_CANDIDATES_V1 {
            continue;
        }
        let due_text_hint =
            due_hint(&normalized).or_else(|| source.global_due_hint.map(str::to_owned));
        let assignee_label_hint =
            assignee_hint(&normalized).or_else(|| source.global_assignee_hint.map(str::to_owned));
        let mut candidate = CommunicationTaskCandidateV1 {
            candidate_id: [0; 16],
            candidate_digest: [0; 32],
            title,
            due_text_hint,
            assignee_label_hint,
            source_basis: basis,
            signal_kind,
            confidence_basis_points: confidence(signal_kind),
            source_evidence_id: source.evidence_id,
            source_evidence_revision: source.evidence_revision,
        };
        refresh_identity(&mut candidate);
        positions.insert(key, candidates.len());
        candidates.push(candidate);
    }
}

fn signal_kind(normalized: &str) -> Option<CommunicationTaskSignalKindV1> {
    if contains_any(normalized, EXPLICIT_ACTION_MARKERS) {
        return Some(CommunicationTaskSignalKindV1::ExplicitAction);
    }
    if contains_any(normalized, DIRECT_REQUEST_MARKERS) && contains_any(normalized, ACTION_TERMS) {
        return Some(CommunicationTaskSignalKindV1::DirectRequest);
    }
    contains_any(normalized, FOLLOW_UP_MARKERS).then_some(CommunicationTaskSignalKindV1::FollowUp)
}

fn due_hint(normalized: &str) -> Option<String> {
    if let Some(value) = normalized
        .strip_prefix("due:")
        .or_else(|| normalized.strip_prefix("deadline:"))
    {
        let value = value.trim().trim_end_matches(['.', '?', '!', ':', ';']);
        return (!value.is_empty())
            .then(|| truncate_chars(value, COMMUNICATION_TASK_MAX_HINT_CHARS_V1));
    }
    DUE_SEPARATORS.iter().find_map(|separator| {
        normalized.split_once(separator).and_then(|(_, value)| {
            let value = value.trim().trim_end_matches(['.', '?', '!', ':', ';']);
            (!value.is_empty()).then(|| truncate_chars(value, COMMUNICATION_TASK_MAX_HINT_CHARS_V1))
        })
    })
}

fn assignee_hint(normalized: &str) -> Option<String> {
    ["assignee:", "исполнитель:"]
        .iter()
        .find_map(|prefix| normalized.strip_prefix(prefix))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_chars(value, COMMUNICATION_TASK_MAX_HINT_CHARS_V1))
}

fn refresh_identity(candidate: &mut CommunicationTaskCandidateV1) {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication-task-candidate.digest.v1\0");
    digest.update(candidate.source_evidence_id);
    digest.update(candidate.source_evidence_revision.to_be_bytes());
    digest.update([
        basis_byte(candidate.source_basis),
        signal_byte(candidate.signal_kind),
    ]);
    digest.update(candidate.title.as_bytes());
    digest.update([0]);
    if let Some(value) = &candidate.due_text_hint {
        digest.update(value.as_bytes());
    }
    digest.update([0]);
    if let Some(value) = &candidate.assignee_label_hint {
        digest.update(value.as_bytes());
    }
    candidate.candidate_digest = digest.finalize().into();

    let mut identity = Sha256::new();
    identity.update(b"makosh.communication-task-candidate.id.v1\0");
    identity.update(candidate.candidate_digest);
    let identity: [u8; 32] = identity.finalize().into();
    candidate.candidate_id.copy_from_slice(&identity[..16]);
}

fn basis_byte(value: CommunicationTaskSourceBasisV1) -> u8 {
    match value {
        CommunicationTaskSourceBasisV1::Subject => 1,
        CommunicationTaskSourceBasisV1::Body => 2,
        CommunicationTaskSourceBasisV1::Combined => 3,
    }
}

fn signal_byte(value: CommunicationTaskSignalKindV1) -> u8 {
    match value {
        CommunicationTaskSignalKindV1::ExplicitAction => 1,
        CommunicationTaskSignalKindV1::DirectRequest => 2,
        CommunicationTaskSignalKindV1::FollowUp => 3,
    }
}

fn confidence(value: CommunicationTaskSignalKindV1) -> u32 {
    match value {
        CommunicationTaskSignalKindV1::ExplicitAction => 9_000,
        CommunicationTaskSignalKindV1::DirectRequest => 8_500,
        CommunicationTaskSignalKindV1::FollowUp => 7_800,
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(subject: &str, body: &str) -> Vec<CommunicationTaskCandidateV1> {
        extract_communication_task_candidates_v1(
            CommunicationTaskSourceContentV1 {
                subject_utf8: subject.as_bytes(),
                body_utf8: body.as_bytes(),
            },
            [7; 16],
            4,
        )
        .expect("valid source")
    }

    #[test]
    fn extracts_explicit_direct_and_follow_up_signals_in_source_order() {
        let candidates = extract(
            "Action: prepare the brief by Friday",
            "Можешь проверить backup до понедельника?\nСледующий шаг: отправить отчёт",
        );
        assert_eq!(candidates.len(), 3);
        assert_eq!(
            candidates[0].signal_kind,
            CommunicationTaskSignalKindV1::ExplicitAction
        );
        assert_eq!(
            candidates[0].source_basis,
            CommunicationTaskSourceBasisV1::Subject
        );
        assert_eq!(candidates[0].due_text_hint.as_deref(), Some("friday"));
        assert_eq!(
            candidates[1].signal_kind,
            CommunicationTaskSignalKindV1::DirectRequest
        );
        assert_eq!(candidates[1].due_text_hint.as_deref(), Some("понедельника"));
        assert_eq!(
            candidates[2].signal_kind,
            CommunicationTaskSignalKindV1::FollowUp
        );
    }

    #[test]
    fn empty_source_does_not_fabricate_a_task_candidate() {
        assert!(extract("Weekly update", "Thanks for the information").is_empty());
    }

    #[test]
    fn duplicate_title_across_subject_and_body_becomes_one_combined_candidate() {
        let candidates = extract("Task: send report", "Task: send report");
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].source_basis,
            CommunicationTaskSourceBasisV1::Combined
        );
        assert_ne!(candidates[0].candidate_id, [0; 16]);
        assert_ne!(candidates[0].candidate_digest, [0; 32]);
    }

    #[test]
    fn stable_source_produces_stable_candidate_identity() {
        let first = extract("", "Could you review the plan by Friday?");
        let second = extract("", "Could you review the plan by Friday?");
        assert_eq!(first, second);
    }

    #[test]
    fn associates_separate_due_and_assignee_hint_lines_without_creating_extra_candidates() {
        let candidates = extract("", "Task: prepare release\nDue: Friday\nAssignee: Alice");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].due_text_hint.as_deref(), Some("friday"));
        assert_eq!(candidates[0].assignee_label_hint.as_deref(), Some("alice"));
    }

    #[test]
    fn rejects_invalid_utf8_and_oversized_source() {
        assert_eq!(
            extract_communication_task_candidates_v1(
                CommunicationTaskSourceContentV1 {
                    subject_utf8: &[0xff],
                    body_utf8: &[],
                },
                [1; 16],
                1,
            ),
            Err(CommunicationTaskExtractionErrorV1::InvalidUtf8)
        );
        let oversized = vec![b'a'; COMMUNICATION_TASK_SOURCE_MAX_BYTES_V1 + 1];
        assert_eq!(
            extract_communication_task_candidates_v1(
                CommunicationTaskSourceContentV1 {
                    subject_utf8: &[],
                    body_utf8: &oversized,
                },
                [1; 16],
                1,
            ),
            Err(CommunicationTaskExtractionErrorV1::SourceLimit)
        );
    }
}
