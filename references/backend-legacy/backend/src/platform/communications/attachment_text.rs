use std::path::{Component, Path};
use std::time::Duration;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const MAX_ATTACHMENT_TEXT_EXTRACTION_BYTES: usize = 1024 * 1024;
const MAX_RICH_WORKER_RESPONSE_BYTES: u64 = 1_500 * 1024;
const MAX_RICH_WORKER_PREVIEW_RESPONSE_BYTES: u64 = 3 * 1024 * 1024;
const MAX_RICH_WORKER_CDR_RESPONSE_BYTES: u64 = 3 * 1024 * 1024;
const MAX_RICH_WORKER_CDR_ARTIFACT_BYTES: u64 = 2 * 1024 * 1024;
const RICH_WORKER_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RichAttachmentExtractionKind {
    Pdf,
    Docx,
    Ocr,
}

impl RichAttachmentExtractionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Ocr => "ocr",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RichAttachmentExtractionResult {
    pub text: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RichAttachmentSafePreview {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RichAttachmentContentDisarm {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum AttachmentTextExtractionError {
    #[error("attachment content type is not supported for local text extraction")]
    UnsupportedContentType,
    #[error("attachment exceeds the local text extraction limit")]
    SizeLimitExceeded,
    #[error("attachment is not valid UTF-8 text")]
    InvalidUtf8,
    #[error("rich attachment extraction worker is not configured")]
    RichWorkerNotConfigured,
    #[error("rich attachment extraction path is invalid")]
    InvalidRichWorkerPath,
    #[error("rich attachment extraction worker is unavailable")]
    RichWorkerUnavailable,
    #[error("rich attachment extraction worker returned an invalid response")]
    InvalidRichWorkerResponse,
    #[error("rich attachment extraction worker rejected the attachment: {0}")]
    RichWorkerRejected(String),
}

#[derive(Serialize)]
struct RichWorkerRequest<'a> {
    operation: &'static str,
    kind: &'static str,
    source_path: &'a str,
}

#[derive(Deserialize)]
struct RichWorkerResponse {
    status: String,
    text_base64: Option<String>,
    truncated: Option<bool>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct RichWorkerPreviewResponse {
    status: String,
    preview_base64: Option<String>,
    content_type: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct RichWorkerContentDisarmResponse {
    status: String,
    artifact_base64: Option<String>,
    content_type: Option<String>,
    error: Option<String>,
}

/// Extracts bounded UTF-8 text from types that require no active-content parser.
///
/// Rich documents, PDFs, images, and archives stay out of this local fast path
/// until they have dedicated sandboxed extractors.
pub fn extract_local_attachment_text(
    content_type: &str,
    filename: Option<&str>,
    bytes: &[u8],
) -> Result<String, AttachmentTextExtractionError> {
    if !is_locally_extractable_text_type(content_type, filename) {
        return Err(AttachmentTextExtractionError::UnsupportedContentType);
    }
    if bytes.len() > MAX_ATTACHMENT_TEXT_EXTRACTION_BYTES {
        return Err(AttachmentTextExtractionError::SizeLimitExceeded);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| AttachmentTextExtractionError::InvalidUtf8)?;
    Ok(text.replace("\r\n", "\n").replace('\r', "\n"))
}

pub fn is_locally_extractable_text_type(content_type: &str, filename: Option<&str>) -> bool {
    let content_type = content_type.trim().to_ascii_lowercase();
    if content_type.starts_with("text/")
        || matches!(
            content_type.as_str(),
            "application/json" | "application/xml" | "application/yaml" | "application/x-yaml"
        )
    {
        return true;
    }

    filename.is_some_and(|filename| {
        let filename = filename.trim().to_ascii_lowercase();
        [".txt", ".md", ".csv", ".json", ".xml", ".yaml", ".yml"]
            .iter()
            .any(|extension| filename.ends_with(extension))
    })
}

pub fn rich_attachment_extraction_kind(
    content_type: &str,
    filename: Option<&str>,
) -> Option<RichAttachmentExtractionKind> {
    let content_type = content_type.trim().to_ascii_lowercase();
    let filename = filename.map(|filename| filename.trim().to_ascii_lowercase());
    if content_type == "application/pdf"
        || filename
            .as_deref()
            .is_some_and(|name| name.ends_with(".pdf"))
    {
        return Some(RichAttachmentExtractionKind::Pdf);
    }
    if content_type == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        || filename
            .as_deref()
            .is_some_and(|name| name.ends_with(".docx"))
    {
        return Some(RichAttachmentExtractionKind::Docx);
    }
    if content_type.starts_with("image/")
        && filename.as_deref().is_some_and(|name| {
            [".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp"]
                .iter()
                .any(|extension| name.ends_with(extension))
        })
    {
        return Some(RichAttachmentExtractionKind::Ocr);
    }
    None
}

pub fn rich_attachment_extractor_address() -> Option<String> {
    let enabled = std::env::var("MAKOSH_ATTACHMENT_EXTRACTOR_ENABLED")
        .ok()
        .is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        });
    if !enabled {
        return None;
    }
    std::env::var("MAKOSH_ATTACHMENT_EXTRACTOR_ADDR")
        .ok()
        .filter(|address| !address.trim().is_empty())
}

pub async fn extract_rich_attachment_text(
    worker_address: &str,
    kind: RichAttachmentExtractionKind,
    storage_path: &str,
) -> Result<RichAttachmentExtractionResult, AttachmentTextExtractionError> {
    validate_rich_worker_storage_path(storage_path)?;
    let mut request = serde_json::to_vec(&RichWorkerRequest {
        operation: "extract",
        kind: kind.as_str(),
        source_path: storage_path,
    })
    .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    request.push(b'\n');
    let response = tokio::time::timeout(RICH_WORKER_TIMEOUT, async {
        let mut stream = TcpStream::connect(worker_address)
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        stream
            .write_all(&request)
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        stream
            .shutdown()
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        let mut bytes = Vec::new();
        let mut response_stream = stream.take(MAX_RICH_WORKER_RESPONSE_BYTES + 1);
        response_stream
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        if bytes.len() as u64 > MAX_RICH_WORKER_RESPONSE_BYTES {
            return Err(AttachmentTextExtractionError::InvalidRichWorkerResponse);
        }
        Ok(bytes)
    })
    .await
    .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)??;
    let response: RichWorkerResponse = serde_json::from_slice(&response)
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    if response.status != "completed" {
        return Err(AttachmentTextExtractionError::RichWorkerRejected(
            response.error.unwrap_or_else(|| "unknown".to_owned()),
        ));
    }
    let text_base64 = response
        .text_base64
        .ok_or(AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    let text_bytes = base64::engine::general_purpose::STANDARD
        .decode(text_base64)
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    let text = String::from_utf8(text_bytes)
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    Ok(RichAttachmentExtractionResult {
        text,
        truncated: response.truncated.unwrap_or(false),
    })
}

pub async fn render_rich_attachment_safe_preview(
    worker_address: &str,
    kind: RichAttachmentExtractionKind,
    storage_path: &str,
) -> Result<RichAttachmentSafePreview, AttachmentTextExtractionError> {
    validate_rich_worker_storage_path(storage_path)?;
    let mut request = serde_json::to_vec(&RichWorkerRequest {
        operation: "render_preview",
        kind: kind.as_str(),
        source_path: storage_path,
    })
    .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    request.push(b'\n');
    let response = read_rich_worker_response(
        worker_address,
        &request,
        MAX_RICH_WORKER_PREVIEW_RESPONSE_BYTES,
    )
    .await?;
    let response: RichWorkerPreviewResponse = serde_json::from_slice(&response)
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    if response.status != "completed" {
        return Err(AttachmentTextExtractionError::RichWorkerRejected(
            response.error.unwrap_or_else(|| "unknown".to_owned()),
        ));
    }
    let content_type = response
        .content_type
        .filter(|content_type| content_type == "image/png")
        .ok_or(AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(
            response
                .preview_base64
                .ok_or(AttachmentTextExtractionError::InvalidRichWorkerResponse)?,
        )
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    if bytes.is_empty() || bytes.len() as u64 > 2 * 1024 * 1024 {
        return Err(AttachmentTextExtractionError::InvalidRichWorkerResponse);
    }
    Ok(RichAttachmentSafePreview {
        bytes,
        content_type,
    })
}

pub async fn disarm_rich_attachment(
    worker_address: &str,
    kind: RichAttachmentExtractionKind,
    storage_path: &str,
) -> Result<RichAttachmentContentDisarm, AttachmentTextExtractionError> {
    if !matches!(
        kind,
        RichAttachmentExtractionKind::Pdf | RichAttachmentExtractionKind::Docx
    ) {
        return Err(AttachmentTextExtractionError::UnsupportedContentType);
    }
    validate_rich_worker_storage_path(storage_path)?;
    let mut request = serde_json::to_vec(&RichWorkerRequest {
        operation: "content_disarm",
        kind: kind.as_str(),
        source_path: storage_path,
    })
    .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    request.push(b'\n');
    let response =
        read_rich_worker_response(worker_address, &request, MAX_RICH_WORKER_CDR_RESPONSE_BYTES)
            .await?;
    let response: RichWorkerContentDisarmResponse = serde_json::from_slice(&response)
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    if response.status != "completed" {
        return Err(AttachmentTextExtractionError::RichWorkerRejected(
            response.error.unwrap_or_else(|| "unknown".to_owned()),
        ));
    }
    let content_type = response
        .content_type
        .filter(|content_type| content_type == "application/pdf")
        .ok_or(AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(
            response
                .artifact_base64
                .ok_or(AttachmentTextExtractionError::InvalidRichWorkerResponse)?,
        )
        .map_err(|_| AttachmentTextExtractionError::InvalidRichWorkerResponse)?;
    if bytes.is_empty()
        || bytes.len() as u64 > MAX_RICH_WORKER_CDR_ARTIFACT_BYTES
        || !bytes.starts_with(b"%PDF-")
        || !bytes[bytes.len().saturating_sub(1024)..]
            .windows(5)
            .any(|window| window == b"%%EOF")
    {
        return Err(AttachmentTextExtractionError::InvalidRichWorkerResponse);
    }
    Ok(RichAttachmentContentDisarm {
        bytes,
        content_type,
    })
}

async fn read_rich_worker_response(
    worker_address: &str,
    request: &[u8],
    maximum_response_bytes: u64,
) -> Result<Vec<u8>, AttachmentTextExtractionError> {
    tokio::time::timeout(RICH_WORKER_TIMEOUT, async {
        let mut stream = TcpStream::connect(worker_address)
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        stream
            .write_all(request)
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        stream
            .shutdown()
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        let mut bytes = Vec::new();
        let mut response_stream = stream.take(maximum_response_bytes + 1);
        response_stream
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?;
        if bytes.len() as u64 > maximum_response_bytes {
            return Err(AttachmentTextExtractionError::InvalidRichWorkerResponse);
        }
        Ok(bytes)
    })
    .await
    .map_err(|_| AttachmentTextExtractionError::RichWorkerUnavailable)?
}

fn validate_rich_worker_storage_path(
    storage_path: &str,
) -> Result<(), AttachmentTextExtractionError> {
    let path = Path::new(storage_path);
    if storage_path.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(AttachmentTextExtractionError::InvalidRichWorkerPath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[test]
    fn extracts_and_normalizes_bounded_utf8_text() {
        let text = extract_local_attachment_text(
            "text/plain",
            Some("notes.txt"),
            b"first\r\nsecond\rthird\n",
        )
        .expect("extract text");

        assert_eq!(text, "first\nsecond\nthird\n");
    }

    #[test]
    fn rejects_active_binary_and_oversized_content() {
        assert_eq!(
            extract_local_attachment_text("application/pdf", Some("invoice.pdf"), b"%PDF"),
            Err(AttachmentTextExtractionError::UnsupportedContentType)
        );
        assert_eq!(
            extract_local_attachment_text("text/plain", Some("notes.txt"), &[0xff]),
            Err(AttachmentTextExtractionError::InvalidUtf8)
        );
        assert_eq!(
            extract_local_attachment_text(
                "text/plain",
                Some("notes.txt"),
                &vec![b'x'; MAX_ATTACHMENT_TEXT_EXTRACTION_BYTES + 1],
            ),
            Err(AttachmentTextExtractionError::SizeLimitExceeded)
        );
    }

    #[test]
    fn selects_only_sandboxed_rich_attachment_types() {
        assert_eq!(
            rich_attachment_extraction_kind("application/pdf", Some("invoice.pdf")),
            Some(RichAttachmentExtractionKind::Pdf)
        );
        assert_eq!(
            rich_attachment_extraction_kind(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                Some("brief.docx")
            ),
            Some(RichAttachmentExtractionKind::Docx)
        );
        assert_eq!(
            rich_attachment_extraction_kind("image/png", Some("scan.png")),
            Some(RichAttachmentExtractionKind::Ocr)
        );
        assert_eq!(
            rich_attachment_extraction_kind("image/png", Some("scan.svg")),
            None
        );
        assert_eq!(
            rich_attachment_extraction_kind("application/zip", Some("archive.zip")),
            None
        );
    }

    #[test]
    fn rejects_worker_path_traversal_before_connecting() {
        assert_eq!(
            validate_rich_worker_storage_path("../outside.pdf"),
            Err(AttachmentTextExtractionError::InvalidRichWorkerPath)
        );
        assert_eq!(
            validate_rich_worker_storage_path("/etc/passwd"),
            Err(AttachmentTextExtractionError::InvalidRichWorkerPath)
        );
    }

    #[tokio::test]
    async fn rich_worker_contract_uses_a_relative_blob_path_and_base64_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind worker");
        let address = listener.local_addr().expect("worker address");
        let worker = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = Vec::new();
            stream
                .read_to_end(&mut request)
                .await
                .expect("read request");
            let request: serde_json::Value =
                serde_json::from_slice(&request).expect("json request");
            assert_eq!(request["operation"], "extract");
            assert_eq!(request["kind"], "docx");
            assert_eq!(request["source_path"], "sha256/ab/source.blob");
            stream
                .write_all(br#"{"status":"completed","text_base64":"c2FmZSByaWNoIHRleHQ=","truncated":false}"#)
                .await
                .expect("write response");
        });

        let result = extract_rich_attachment_text(
            &address.to_string(),
            RichAttachmentExtractionKind::Docx,
            "sha256/ab/source.blob",
        )
        .await
        .expect("extract rich text");

        worker.await.expect("worker completed");
        assert_eq!(result.text, "safe rich text");
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn rich_worker_rejection_is_not_treated_as_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind worker");
        let address = listener.local_addr().expect("worker address");
        let worker = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = Vec::new();
            stream
                .read_to_end(&mut request)
                .await
                .expect("read request");
            stream
                .write_all(br#"{"status":"failed","error":"source_not_found"}"#)
                .await
                .expect("write response");
        });

        let result = extract_rich_attachment_text(
            &address.to_string(),
            RichAttachmentExtractionKind::Pdf,
            "sha256/ab/source.blob",
        )
        .await;

        worker.await.expect("worker completed");
        assert_eq!(
            result,
            Err(AttachmentTextExtractionError::RichWorkerRejected(
                "source_not_found".to_owned()
            ))
        );
    }

    #[tokio::test]
    async fn rich_worker_preview_contract_accepts_only_bounded_png_artifacts() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind worker");
        let address = listener.local_addr().expect("worker address");
        let worker = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = Vec::new();
            stream
                .read_to_end(&mut request)
                .await
                .expect("read request");
            let request: serde_json::Value =
                serde_json::from_slice(&request).expect("json request");
            assert_eq!(request["operation"], "render_preview");
            assert_eq!(request["kind"], "pdf");
            assert_eq!(request["source_path"], "sha256/ab/source.blob");
            stream
                .write_all(br#"{"status":"completed","content_type":"image/png","preview_base64":"iVBORw0KGgo="}"#)
                .await
                .expect("write response");
        });

        let result = render_rich_attachment_safe_preview(
            &address.to_string(),
            RichAttachmentExtractionKind::Pdf,
            "sha256/ab/source.blob",
        )
        .await
        .expect("render safe preview");

        worker.await.expect("worker completed");
        assert_eq!(result.content_type, "image/png");
        assert_eq!(result.bytes, b"\x89PNG\r\n\x1a\n");
    }

    #[tokio::test]
    async fn rich_worker_cdr_contract_accepts_only_bounded_pdf_artifacts() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind worker");
        let address = listener.local_addr().expect("worker address");
        let worker = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = Vec::new();
            stream
                .read_to_end(&mut request)
                .await
                .expect("read request");
            let request: serde_json::Value =
                serde_json::from_slice(&request).expect("json request");
            assert_eq!(request["operation"], "content_disarm");
            assert_eq!(request["kind"], "pdf");
            assert_eq!(request["source_path"], "sha256/ab/source.blob");
            stream
                .write_all(br#"{"status":"completed","content_type":"application/pdf","artifact_base64":"JVBERi0xLjQKJSVFT0Y="}"#)
                .await
                .expect("write response");
        });

        let artifact = disarm_rich_attachment(
            &address.to_string(),
            RichAttachmentExtractionKind::Pdf,
            "sha256/ab/source.blob",
        )
        .await
        .expect("valid CDR artifact");

        worker.await.expect("worker completed");
        assert_eq!(artifact.content_type, "application/pdf");
        assert_eq!(artifact.bytes, b"%PDF-1.4\n%%EOF");
    }

    #[tokio::test]
    async fn rich_worker_cdr_contract_allows_disarmed_docx_pdf_artifacts() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind worker");
        let address = listener.local_addr().expect("worker address");
        let worker = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = Vec::new();
            stream
                .read_to_end(&mut request)
                .await
                .expect("read request");
            let request: serde_json::Value =
                serde_json::from_slice(&request).expect("json request");
            assert_eq!(request["operation"], "content_disarm");
            assert_eq!(request["kind"], "docx");
            stream
                .write_all(br#"{"status":"completed","content_type":"application/pdf","artifact_base64":"JVBERi0xLjQKJSVFT0Y="}"#)
                .await
                .expect("write response");
        });

        let artifact = disarm_rich_attachment(
            &address.to_string(),
            RichAttachmentExtractionKind::Docx,
            "sha256/ab/source.blob",
        )
        .await
        .expect("valid DOCX CDR artifact");

        worker.await.expect("worker completed");
        assert_eq!(artifact.content_type, "application/pdf");
    }
}
