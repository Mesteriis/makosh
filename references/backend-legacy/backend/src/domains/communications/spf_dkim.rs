// Unverified SPF/DKIM/DMARC header assertion parsing without DNS or crypto verification.
use serde::Serialize;

use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::projection::parse_raw_email_message_from_blob;
use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use makosh_communications_api::evidence::StoredRawCommunicationRecord;

#[derive(Clone, Debug, Default, Serialize)]
pub struct AuthResults {
    pub spf: Option<SpfResult>,
    pub dkim: Option<DkimResult>,
    pub dmarc: Option<DmarcResult>,
    pub raw_headers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpfResult {
    pub result: String,
    pub domain: Option<String>,
    pub ip: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DkimResult {
    pub result: String,
    pub domain: Option<String>,
    pub selector: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DmarcResult {
    pub result: String,
    pub domain: Option<String>,
    pub policy: Option<String>,
}

/// Parse Authentication-Results and Received-SPF headers from raw email headers.
pub fn parse_auth_headers(raw_headers: &str) -> AuthResults {
    let mut spf = None;
    let mut dkim = None;
    let mut dmarc = None;
    let mut raw = Vec::new();

    for line in raw_headers.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("authentication-results:") || lower.starts_with("received-spf:") {
            raw.push(line.to_owned());

            if lower.contains("spf=") {
                let result = extract_value(line, "spf=");
                let domain = extract_value(line, "smtp.mailfrom=")
                    .or_else(|| extract_value(line, "envelope-from="));
                if let Some(res) = result {
                    spf = Some(SpfResult {
                        result: res,
                        domain,
                        ip: None,
                    });
                }
            }
            if lower.contains("dkim=") {
                let result = extract_value(line, "dkim=");
                let domain = extract_value(line, "d=");
                let selector = extract_value(line, "s=");
                if let Some(res) = result {
                    dkim = Some(DkimResult {
                        result: res,
                        domain,
                        selector,
                    });
                }
            }
            if lower.contains("dmarc=") {
                let result = extract_value(line, "dmarc=");
                let domain = extract_value(line, "header.from=");
                let policy = extract_value(line, "p=");
                if let Some(res) = result {
                    dmarc = Some(DmarcResult {
                        result: res,
                        domain,
                        policy,
                    });
                }
            }
        }
    }

    AuthResults {
        spf,
        dkim,
        dmarc,
        raw_headers: raw,
    }
}

/// Reads only authentication-related RFC822 fields from retained raw evidence.
///
/// Authentication assertions in a message body are untrusted message content;
/// they must never influence a security result. Older records without a raw
/// RFC822 blob return an empty report instead of falling back to body text.
pub async fn parse_auth_headers_from_raw_record(
    raw: &StoredRawCommunicationRecord,
) -> Result<AuthResults, MessageProjectionError> {
    if raw
        .payload
        .get("raw_blob_storage_kind")
        .and_then(|value| value.as_str())
        != Some("local_fs")
        || raw
            .payload
            .get("raw_blob_storage_path")
            .and_then(|value| value.as_str())
            .is_none_or(|value| value.trim().is_empty())
    {
        return Ok(AuthResults::default());
    }

    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let parsed = parse_raw_email_message_from_blob(&blob_store, raw).await?;
    Ok(parse_auth_header_pairs(&parsed.headers))
}

pub fn parse_auth_header_pairs(headers: &[(String, String)]) -> AuthResults {
    let raw_headers = headers
        .iter()
        .filter(|(name, _)| {
            name.eq_ignore_ascii_case("authentication-results")
                || name.eq_ignore_ascii_case("received-spf")
        })
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    parse_auth_headers(&raw_headers)
}

fn extract_value(line: &str, prefix: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let pos = lower.find(prefix)?;
    let start = pos + prefix.len();
    let rest = &line[start..];
    let end = rest.find([';', ' ', '\r', '\n']).unwrap_or(rest.len());
    let val = rest[..end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_owned())
    }
}

#[derive(Debug, Serialize)]
pub struct SpfDkimReport {
    pub has_spf: bool,
    pub spf_pass: bool,
    pub has_dkim: bool,
    pub dkim_pass: bool,
    pub has_dmarc: bool,
    pub dmarc_pass: bool,
    pub is_spoofed: bool,
    pub risk_summary: String,
}

pub fn assess_auth_risk(auth: &AuthResults) -> SpfDkimReport {
    let spf_pass = auth
        .spf
        .as_ref()
        .map(|s| s.result == "pass")
        .unwrap_or(false);
    let dkim_pass = auth
        .dkim
        .as_ref()
        .map(|d| d.result == "pass")
        .unwrap_or(false);
    let dmarc_pass = auth
        .dmarc
        .as_ref()
        .map(|d| d.result == "pass")
        .unwrap_or(false);
    let has_spf = auth.spf.is_some();
    let has_dkim = auth.dkim.is_some();
    let has_dmarc = auth.dmarc.is_some();
    let is_spoofed =
        (has_spf && !spf_pass) || (has_dkim && !dkim_pass) || (has_dmarc && !dmarc_pass);
    let summary = if is_spoofed {
        "Unverified authentication header assertion indicates possible spoofing".into()
    } else if has_spf || has_dkim || has_dmarc {
        "Unverified authentication header assertions; cryptographic verification is unavailable"
            .into()
    } else {
        "No authentication headers present".into()
    };
    SpfDkimReport {
        has_spf,
        spf_pass,
        has_dkim,
        dkim_pass,
        has_dmarc,
        dmarc_pass,
        is_spoofed,
        risk_summary: summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_spf_pass() {
        let r = parse_auth_headers(
            "Authentication-Results: mx.google.com; spf=pass smtp.mailfrom=alice@example.com",
        );
        assert_eq!(r.spf.as_ref().unwrap().result, "pass");
        assert_eq!(
            r.spf.as_ref().unwrap().domain.as_deref(),
            Some("alice@example.com")
        );
    }
    #[test]
    fn parse_dkim_fail() {
        let r = parse_auth_headers("Authentication-Results: dkim=fail d=evil.com s=default");
        assert_eq!(r.dkim.as_ref().unwrap().result, "fail");
    }
    #[test]
    fn parse_dmarc() {
        let r = parse_auth_headers(
            "Authentication-Results: dmarc=pass header.from=example.com p=reject",
        );
        assert!(r.dmarc.as_ref().unwrap().result == "pass");
    }
    #[test]
    fn spoofed_email_flagged() {
        let auth = AuthResults {
            spf: Some(SpfResult {
                result: "fail".into(),
                domain: None,
                ip: None,
            }),
            dkim: None,
            dmarc: None,
            raw_headers: vec![],
        };
        let risk = assess_auth_risk(&auth);
        assert!(risk.is_spoofed);
    }
    #[test]
    fn clean_email_passes() {
        let auth = AuthResults {
            spf: Some(SpfResult {
                result: "pass".into(),
                domain: None,
                ip: None,
            }),
            dkim: Some(DkimResult {
                result: "pass".into(),
                domain: None,
                selector: None,
            }),
            dmarc: None,
            raw_headers: vec![],
        };
        let risk = assess_auth_risk(&auth);
        assert!(!risk.is_spoofed);
    }

    #[test]
    fn ignores_authentication_looking_text_in_non_authentication_headers() {
        let auth = parse_auth_header_pairs(&[
            (
                "Subject".to_owned(),
                "Please note dkim=pass d=attacker.example".to_owned(),
            ),
            ("From".to_owned(), "sender@example.test".to_owned()),
        ]);

        assert!(auth.spf.is_none());
        assert!(auth.dkim.is_none());
        assert!(auth.dmarc.is_none());
        assert!(auth.raw_headers.is_empty());
    }
}
