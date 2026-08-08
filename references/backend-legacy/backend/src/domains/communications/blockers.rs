//! # Макошь Mail — Explicit Architecture Blockers
//!
//! Sections that are NOT implemented and WHY.
//! This file serves as authoritative documentation of known gaps.

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ArchitectureBlocker {
    pub section: String,
    pub feature: String,
    pub reason: String,
    pub resolution: String,
}

/// Returns all known blockers with explanations.
pub fn list_blockers() -> Vec<ArchitectureBlocker> {
    vec![
        ArchitectureBlocker {
            section: "§8".into(),
            feature: "Безопасность вложений (sandbox, антивирус)".into(),
            reason: "Conservative heuristic attachment scanning flags obvious executable payloads, active-content extensions, macro-enabled office files, MIME/extension mismatch, legacy OLE files and bounded OOXML macro markers as suspicious or malicious. Local attachment imports also stream through the project ClamAV sidecar and stay quarantined when the scanner is unavailable or returns an invalid verdict. A bounded local worker retries not-scanned mail and staged-send blobs after scanner recovery only after size/SHA-256 verification, but sandboxing, CDR and full OLE/macro analysis remain unavailable.".into(),
            resolution: "Keep ClamAV as the mandatory local malware verdict for imported attachment bytes, then add a network-isolated sandbox, CDR and full OLE/macro analysis. Do not mark attachments clean when a required scanner verdict is unavailable.".into(),
        },
        ArchitectureBlocker {
            section: "§12 (крипто-проверка)".into(),
            feature: "Реальная криптографическая верификация подписей (S/MIME, PGP, CAdES, XAdES, ГОСТ)".into(),
            reason: "Требует OpenSSL, GPG, КриптоПро SDK. Это внешние нативные библиотеки, не Rust-крейты. Нужна отдельная интеграционная работа.".into(),
            resolution: "Добавить email_crypto модуль с привязкой к OpenSSL/GPG через FFI или CLI. Сертификаты из Keychain читать через macOS Security framework.".into(),
        },
        ArchitectureBlocker {
            section: "§16-17".into(),
            feature: "Outbox tracking (delivery status, read receipts, bounce detection) и Follow-up engine".into(),
            reason: "Durable outbox tracking, runtime scheduling, account-scoped SMTP sender wiring, Gmail OAuth send scopes, immediate and scheduled Gmail API send, retry/backoff handling, sanitized DSN delivery-status ingestion and MDN read-receipt ingestion exist. Production delivery/read receipt tracking still requires provider callback/webhook wiring and richer delivery UX.".into(),
            resolution: "Connect provider callback/runtime ingestion to the delivery-notification path, and surface delivery status in the user-facing outbox UX without exposing private content in logs or events.".into(),
        },
        ArchitectureBlocker {
            section: "§28-29".into(),
            feature: "Интеграции (Jira, YouTrack, Google Calendar, Apple Notes, Obsidian) и provider-side массовые действия".into(),
            reason: "Каждая интеграция — отдельный коннектор со своим API и аутентификацией. Local bounded bulk actions exist, but provider-side batch mutations, long-running jobs and progress events still require queues.".into(),
            resolution: "Реализовать интеграции как plugin-коннекторы по образцу Telegram/WhatsApp модулей. Provider-side массовые действия — через фоновые задачи projection runner with progress events.".into(),
        },
        ArchitectureBlocker {
            section: "§8.2".into(),
            feature: "Безопасная распаковка архивов (zip slip protection, вложенные архивы, password detection)".into(),
            reason: "Требует потоковой распаковки с защитой от zip bomb/path traversal. Нужна интеграция с zip/rar/7z крейтами и настройка лимитов.".into(),
            resolution: "Добавить email_archive_extractor с лимитами размера/глубины/количества файлов, использовать крейт zip + rar + sevenz-rs.".into(),
        },
        ArchitectureBlocker {
            section: "§9.3".into(),
            feature: "OCR (распознавание текста из PDF-сканов и изображений)".into(),
            reason: "Требует Tesseract OCR или облачного OCR-сервиса. Это тяжёлая зависимость (50+ MB trained data).".into(),
            resolution: "Опциональная фича: добавить tesseract-rs крейт под feature-флагом. Без неё — только текст из PDF/DOCX.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_blockers_have_reason_and_resolution() {
        for b in list_blockers() {
            assert!(!b.reason.is_empty(), "{} has no reason", b.section);
            assert!(!b.resolution.is_empty(), "{} has no resolution", b.section);
        }
    }
    #[test]
    fn blocker_count_is_stable() {
        assert_eq!(
            list_blockers().len(),
            6,
            "Expected exactly 6 architectural blockers"
        );
    }
}
