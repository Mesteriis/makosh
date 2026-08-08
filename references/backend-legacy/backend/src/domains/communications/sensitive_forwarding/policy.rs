use super::*;

pub(super) fn stored_policy_from_row(
    row: PgRow,
) -> Result<StoredSensitiveForwardingPolicy, SensitiveForwardingError> {
    let policy = StoredSensitiveForwardingPolicy {
        policy_id: row.try_get("policy_id")?,
        source_account_id: row.try_get("source_account_id")?,
        delivery_account_id: row.try_get("delivery_account_id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        include_message_body: row.try_get("include_message_body")?,
        include_attachments: row.try_get("include_attachments")?,
        fixed_recipients: serde_json::from_value(row.try_get("fixed_recipients")?)?,
        minimum_severity: row.try_get("minimum_severity")?,
        subject_template: row.try_get("subject_template")?,
        body_template: row.try_get("body_template")?,
        max_sends_per_hour: row.try_get("max_sends_per_hour")?,
        quiet_hours: row.try_get("quiet_hours")?,
        expires_at: row.try_get("expires_at")?,
        updated_at: row.try_get("updated_at")?,
    };
    let candidate = NewSensitiveForwardingPolicy {
        policy_id: policy.policy_id.clone(),
        source_account_id: policy.source_account_id.clone(),
        delivery_account_id: policy.delivery_account_id.clone(),
        name: policy.name.clone(),
        enabled: policy.enabled,
        include_message_body: policy.include_message_body,
        include_attachments: policy.include_attachments,
        fixed_recipients: policy.fixed_recipients.clone(),
        minimum_severity: policy.minimum_severity.clone(),
        subject_template: policy.subject_template.clone(),
        body_template: policy.body_template.clone(),
        max_sends_per_hour: policy.max_sends_per_hour,
        quiet_hours: policy.quiet_hours.clone(),
        expires_at: policy.expires_at,
    };
    validate_sensitive_forwarding_policy(&candidate)?;
    Ok(policy)
}

pub(super) fn policy_suppression(
    policy: &SensitiveForwardingPolicy,
    request: &SensitiveForwardingRequest,
    now: DateTime<Utc>,
) -> Result<Option<SensitiveForwardingSuppression>, SensitiveForwardingError> {
    if !policy.enabled {
        return Ok(Some(SensitiveForwardingSuppression::Disabled));
    }
    if policy
        .expires_at
        .is_some_and(|expires_at| expires_at <= now)
    {
        return Ok(Some(SensitiveForwardingSuppression::Expired));
    }
    if severity_rank(&request.severity) < severity_rank(&policy.minimum_severity) {
        return Ok(Some(SensitiveForwardingSuppression::BelowMinimumSeverity));
    }
    if parse_quiet_hours(&policy.quiet_hours)?.is_some_and(|quiet_hours| quiet_hours.contains(now))
    {
        return Ok(Some(SensitiveForwardingSuppression::QuietHours));
    }
    Ok(None)
}

pub(super) fn validate_sensitive_forwarding_request(
    request: &SensitiveForwardingRequest,
) -> Result<(), SensitiveForwardingError> {
    if [
        &request.dispatch_id,
        &request.policy_id,
        &request.source_account_id,
        &request.message_id,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || severity_rank(&request.severity).is_none()
    {
        return Err(SensitiveForwardingError::Invalid);
    }
    Ok(())
}

pub(super) fn validate_sensitive_forwarding_policy(
    policy: &NewSensitiveForwardingPolicy,
) -> Result<(), SensitiveForwardingError> {
    for value in [
        &policy.policy_id,
        &policy.source_account_id,
        &policy.delivery_account_id,
        &policy.name,
        &policy.subject_template,
        &policy.body_template,
    ] {
        if value.trim().is_empty() {
            return Err(SensitiveForwardingError::Invalid);
        }
    }
    if policy.fixed_recipients.is_empty()
        || policy
            .fixed_recipients
            .iter()
            .any(|recipient| !is_valid_recipient(recipient))
        || severity_rank(&policy.minimum_severity).is_none()
        || policy.max_sends_per_hour <= 0
    {
        return Err(SensitiveForwardingError::Invalid);
    }
    parse_quiet_hours(&policy.quiet_hours)?;
    Ok(())
}

impl QuietHours {
    pub(super) fn contains(self, now: DateTime<Utc>) -> bool {
        let time = now.time();
        if self.start < self.end {
            time >= self.start && time < self.end
        } else {
            time >= self.start || time < self.end
        }
    }
}

pub(super) fn parse_quiet_hours(
    value: &Value,
) -> Result<Option<QuietHours>, SensitiveForwardingError> {
    let Some(object) = value.as_object() else {
        return Err(SensitiveForwardingError::Invalid);
    };
    if object.is_empty() {
        return Ok(None);
    }
    if object
        .get("timezone")
        .and_then(Value::as_str)
        .is_some_and(|timezone| timezone != "UTC")
    {
        return Err(SensitiveForwardingError::Invalid);
    }
    let start = object
        .get("start")
        .and_then(Value::as_str)
        .and_then(parse_hhmm)
        .ok_or(SensitiveForwardingError::Invalid)?;
    let end = object
        .get("end")
        .and_then(Value::as_str)
        .and_then(parse_hhmm)
        .ok_or(SensitiveForwardingError::Invalid)?;
    if start == end {
        return Err(SensitiveForwardingError::Invalid);
    }
    Ok(Some(QuietHours { start, end }))
}

pub(super) fn parse_hhmm(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M").ok()
}

pub(super) fn severity_rank(value: &str) -> Option<u8> {
    match value.trim() {
        "low" => Some(1),
        "medium" => Some(2),
        "high" => Some(3),
        "critical" => Some(4),
        _ => None,
    }
}

pub(super) fn is_valid_recipient(value: &str) -> bool {
    let value = value.trim();
    value.contains('@') && !value.contains(['\r', '\n', '\0'])
}

pub(super) fn render_template(
    template: &str,
    request: &SensitiveForwardingRequest,
    attachment_notice: &str,
) -> String {
    template
        .replace("{{message_id}}", request.message_id.trim())
        .replace("{{severity}}", request.severity.trim())
        .replace("{{attachment_notice}}", attachment_notice)
}

pub(super) fn render_notification_body(
    template: &str,
    request: &SensitiveForwardingRequest,
    source_message: &crate::domains::communications::messages::models::ProjectedMessage,
    body_transferred: bool,
    attachment_transfer_permitted: bool,
    attachment_plan: &AttachmentForwardingPlan,
) -> String {
    let attachment_notice =
        attachment_notice(request, attachment_transfer_permitted, attachment_plan);
    let mut body = render_template(template, request, &attachment_notice);
    if body_transferred {
        const MAX_FORWARDED_BODY_CHARS: usize = 200_000;
        let source_body: String = source_message
            .body_text
            .chars()
            .take(MAX_FORWARDED_BODY_CHARS)
            .collect();
        body.push_str("\n\n--- Forwarded message ---\nFrom: ");
        body.push_str(&source_message.sender);
        body.push_str("\nSubject: ");
        body.push_str(&source_message.subject);
        body.push_str("\n\n");
        body.push_str(&source_body);
        if source_message.body_text.chars().count() > MAX_FORWARDED_BODY_CHARS {
            body.push_str("\n\n[Message body truncated by Макошь safety limit]");
        }
    }
    body
}

pub(super) fn attachment_notice(
    request: &SensitiveForwardingRequest,
    attachment_transfer_permitted: bool,
    plan: &AttachmentForwardingPlan,
) -> String {
    if !attachment_transfer_permitted {
        return if request.has_unsafe_attachments {
            "Attachments were withheld because they are not safe to forward.".to_owned()
        } else {
            "Attachments are not included in this notification.".to_owned()
        };
    }

    let mut parts = Vec::new();
    if plan.copied_count() > 0 {
        parts.push(format!(
            "{} clean attachment(s) included.",
            plan.copied_count()
        ));
    }
    if plan.unsafe_withheld > 0 {
        parts.push(format!(
            "{} unsafe or unverified attachment(s) withheld.",
            plan.unsafe_withheld
        ));
    }
    if plan.delivery_limit_withheld > 0 {
        parts.push(format!(
            "{} clean attachment(s) withheld by delivery limits.",
            plan.delivery_limit_withheld
        ));
    }
    if parts.is_empty() {
        "No source attachments were available for forwarding.".to_owned()
    } else {
        parts.join(" ")
    }
}
