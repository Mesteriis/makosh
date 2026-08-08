use crate::domains::communications::spf_dkim::{AuthResults, SpfDkimReport};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    MessageAuthReport as ProtoMessageAuthReport, MessageAuthResult as ProtoMessageAuthResult,
    MessageAuthRiskReport as ProtoMessageAuthRiskReport,
};

fn auth_result(
    result: &str,
    domain: Option<String>,
    ip: Option<String>,
    selector: Option<String>,
    policy: Option<String>,
) -> ProtoMessageAuthResult {
    ProtoMessageAuthResult {
        result: result.to_owned(),
        domain,
        ip,
        selector,
        policy,
        ..Default::default()
    }
}
pub(super) fn report(item: AuthResults) -> ProtoMessageAuthReport {
    ProtoMessageAuthReport {
        spf: item
            .spf
            .map(|value| auth_result(&value.result, value.domain, value.ip, None, None))
            .map(|value| Some(value).into())
            .unwrap_or_default(),
        dkim: item
            .dkim
            .map(|value| auth_result(&value.result, value.domain, None, value.selector, None))
            .map(|value| Some(value).into())
            .unwrap_or_default(),
        dmarc: item
            .dmarc
            .map(|value| auth_result(&value.result, value.domain, None, None, value.policy))
            .map(|value| Some(value).into())
            .unwrap_or_default(),
        raw_headers: item.raw_headers,
        ..Default::default()
    }
}
pub(super) fn risk_report(item: SpfDkimReport) -> ProtoMessageAuthRiskReport {
    ProtoMessageAuthRiskReport {
        has_spf: item.has_spf,
        spf_pass: item.spf_pass,
        has_dkim: item.has_dkim,
        dkim_pass: item.dkim_pass,
        has_dmarc: item.has_dmarc,
        dmarc_pass: item.dmarc_pass,
        is_spoofed: item.is_spoofed,
        risk_summary: item.risk_summary,
        ..Default::default()
    }
}
