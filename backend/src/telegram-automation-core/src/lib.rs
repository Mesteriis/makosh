//! Pure Telegram automation policy validation and deterministic preview rendering.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-telegram-automation-core";
pub const MAX_ID_BYTES: usize = 256;
pub const MAX_NAME_BYTES: usize = 512;
pub const MAX_TEMPLATE_BYTES: usize = 16 * 1024;
pub const MAX_RENDERED_BYTES: usize = 32 * 1024;
pub const MAX_VARIABLES: usize = 32;
pub const MAX_VARIABLE_NAME_BYTES: usize = 64;
pub const MAX_VARIABLE_VALUE_BYTES: usize = 4 * 1024;
pub const MAX_POLICY_CHAT_SCOPES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AutomationError {
    InvalidRequest(&'static str),
    NotFound(&'static str),
    RevisionConflict,
    IdempotencyConflict,
    PolicyDisabled,
    PolicyExpired,
    ScopeDenied,
    MissingVariable,
    UndeclaredVariable,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationTemplateDraft {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
    pub revision: u64,
    pub created_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationPolicyDraft {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub provider_chat_ids: Vec<String>,
    pub expires_at_unix_seconds: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub provider_chat_ids: Vec<String>,
    pub expires_at_unix_seconds: Option<u64>,
    pub revision: u64,
    pub created_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationVariable {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationPreviewRequest {
    pub preview_id: String,
    pub policy_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub variables: Vec<AutomationVariable>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationPreviewReceipt {
    pub preview_id: String,
    pub policy_id: String,
    pub policy_revision: u64,
    pub template_id: String,
    pub template_revision: u64,
    pub account_id: String,
    pub provider_chat_id: String,
    pub rendered_text: String,
    pub rendered_sha256: [u8; 32],
    pub created_at_unix_seconds: u64,
}

impl AutomationTemplateDraft {
    pub fn validate(&self) -> Result<(), AutomationError> {
        validate_id("template_id", &self.template_id)?;
        validate_bounded_non_empty("name", &self.name, MAX_NAME_BYTES)?;
        validate_bounded_non_empty("body_template", &self.body_template, MAX_TEMPLATE_BYTES)?;
        if self.required_variables.len() > MAX_VARIABLES {
            return Err(AutomationError::InvalidRequest("required_variables"));
        }
        let mut declared = BTreeSet::new();
        for variable in &self.required_variables {
            validate_variable_name(variable)?;
            if !declared.insert(variable.as_str()) {
                return Err(AutomationError::InvalidRequest("required_variables"));
            }
        }
        let placeholders = template_placeholders(&self.body_template)?;
        if placeholders != declared {
            return Err(AutomationError::InvalidRequest("body_template"));
        }
        Ok(())
    }
}

impl AutomationPolicyDraft {
    pub fn validate(&self) -> Result<(), AutomationError> {
        validate_id("policy_id", &self.policy_id)?;
        validate_id("template_id", &self.template_id)?;
        validate_bounded_non_empty("name", &self.name, MAX_NAME_BYTES)?;
        validate_id("account_id", &self.account_id)?;
        if self.provider_chat_ids.is_empty()
            || self.provider_chat_ids.len() > MAX_POLICY_CHAT_SCOPES
        {
            return Err(AutomationError::InvalidRequest("provider_chat_ids"));
        }
        let mut scopes = BTreeSet::new();
        for provider_chat_id in &self.provider_chat_ids {
            validate_id("provider_chat_id", provider_chat_id)?;
            if !scopes.insert(provider_chat_id.as_str()) {
                return Err(AutomationError::InvalidRequest("provider_chat_ids"));
            }
        }
        if self.expires_at_unix_seconds == Some(0) {
            return Err(AutomationError::InvalidRequest("expires_at_unix_seconds"));
        }
        Ok(())
    }
}

impl AutomationPreviewRequest {
    pub fn validate(&self) -> Result<(), AutomationError> {
        validate_id("preview_id", &self.preview_id)?;
        validate_id("policy_id", &self.policy_id)?;
        validate_id("account_id", &self.account_id)?;
        validate_id("provider_chat_id", &self.provider_chat_id)?;
        if self.variables.len() > MAX_VARIABLES {
            return Err(AutomationError::InvalidRequest("variables"));
        }
        let mut names = BTreeSet::new();
        for variable in &self.variables {
            validate_variable_name(&variable.name)?;
            validate_bounded_non_empty(
                "variable_value",
                &variable.value,
                MAX_VARIABLE_VALUE_BYTES,
            )?;
            if !names.insert(variable.name.as_str()) {
                return Err(AutomationError::InvalidRequest("variables"));
            }
        }
        Ok(())
    }
}

pub fn render_preview(
    policy: &AutomationPolicy,
    template: &AutomationTemplate,
    request: &AutomationPreviewRequest,
    now_unix_seconds: u64,
) -> Result<AutomationPreviewReceipt, AutomationError> {
    request.validate()?;
    if !policy.enabled {
        return Err(AutomationError::PolicyDisabled);
    }
    if policy
        .expires_at_unix_seconds
        .is_some_and(|expires_at| expires_at <= now_unix_seconds)
    {
        return Err(AutomationError::PolicyExpired);
    }
    if request.policy_id != policy.policy_id
        || request.account_id != policy.account_id
        || !policy
            .provider_chat_ids
            .iter()
            .any(|chat_id| chat_id == &request.provider_chat_id)
    {
        return Err(AutomationError::ScopeDenied);
    }
    if template.template_id != policy.template_id {
        return Err(AutomationError::NotFound("template_id"));
    }

    let variables = request
        .variables
        .iter()
        .map(|variable| (variable.name.as_str(), variable.value.trim()))
        .collect::<BTreeMap<_, _>>();
    if variables
        .keys()
        .any(|name| !template.required_variables.iter().any(|item| item == name))
    {
        return Err(AutomationError::UndeclaredVariable);
    }
    if template
        .required_variables
        .iter()
        .any(|name| !variables.contains_key(name.as_str()))
    {
        return Err(AutomationError::MissingVariable);
    }

    let mut rendered = template.body_template.clone();
    for name in &template.required_variables {
        let value = variables
            .get(name.as_str())
            .copied()
            .ok_or(AutomationError::MissingVariable)?;
        rendered = rendered.replace(&format!("{{{{{name}}}}}"), value);
    }
    if rendered.contains("{{")
        || rendered.contains("}}")
        || rendered.len() > MAX_RENDERED_BYTES
        || rendered.trim().is_empty()
    {
        return Err(AutomationError::InvalidRequest("rendered_text"));
    }

    Ok(AutomationPreviewReceipt {
        preview_id: request.preview_id.clone(),
        policy_id: policy.policy_id.clone(),
        policy_revision: policy.revision,
        template_id: template.template_id.clone(),
        template_revision: template.revision,
        account_id: policy.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        rendered_sha256: Sha256::digest(rendered.as_bytes()).into(),
        rendered_text: rendered,
        created_at_unix_seconds: now_unix_seconds,
    })
}

pub fn validate_identifier(field: &'static str, value: &str) -> Result<(), AutomationError> {
    validate_id(field, value)
}

fn validate_id(field: &'static str, value: &str) -> Result<(), AutomationError> {
    validate_bounded_non_empty(field, value, MAX_ID_BYTES)?;
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
    }) {
        return Err(AutomationError::InvalidRequest(field));
    }
    Ok(())
}

fn validate_variable_name(value: &str) -> Result<(), AutomationError> {
    validate_bounded_non_empty("variable_name", value, MAX_VARIABLE_NAME_BYTES)?;
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(AutomationError::InvalidRequest("variable_name"));
    }
    Ok(())
}

fn validate_bounded_non_empty(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), AutomationError> {
    if value.trim().is_empty() || value.len() > max_bytes {
        return Err(AutomationError::InvalidRequest(field));
    }
    Ok(())
}

fn template_placeholders(body: &str) -> Result<BTreeSet<&str>, AutomationError> {
    let mut remaining = body;
    let mut placeholders = BTreeSet::new();
    loop {
        let Some(start) = remaining.find("{{") else {
            if remaining.contains("}}") {
                return Err(AutomationError::InvalidRequest("body_template"));
            }
            return Ok(placeholders);
        };
        if remaining[..start].contains("}}") {
            return Err(AutomationError::InvalidRequest("body_template"));
        }
        let after_open = &remaining[start + 2..];
        let end = after_open
            .find("}}")
            .ok_or(AutomationError::InvalidRequest("body_template"))?;
        let name = &after_open[..end];
        validate_variable_name(name)?;
        placeholders.insert(name);
        remaining = &after_open[end + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template() -> AutomationTemplate {
        AutomationTemplate {
            template_id: "template-1".to_owned(),
            name: "Greeting".to_owned(),
            body_template: "Hello {{name}}".to_owned(),
            required_variables: vec!["name".to_owned()],
            revision: 3,
            created_at_unix_seconds: 10,
            updated_at_unix_seconds: 20,
        }
    }

    fn policy() -> AutomationPolicy {
        AutomationPolicy {
            policy_id: "policy-1".to_owned(),
            template_id: "template-1".to_owned(),
            name: "Scoped greeting".to_owned(),
            enabled: true,
            account_id: "account-1".to_owned(),
            provider_chat_ids: vec!["chat-1".to_owned()],
            expires_at_unix_seconds: Some(200),
            revision: 4,
            created_at_unix_seconds: 10,
            updated_at_unix_seconds: 20,
        }
    }

    fn request() -> AutomationPreviewRequest {
        AutomationPreviewRequest {
            preview_id: "preview-1".to_owned(),
            policy_id: "policy-1".to_owned(),
            account_id: "account-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            variables: vec![AutomationVariable {
                name: "name".to_owned(),
                value: " Ada ".to_owned(),
            }],
        }
    }

    #[test]
    fn template_requires_exact_declared_placeholders() {
        let draft = AutomationTemplateDraft {
            template_id: "template-1".to_owned(),
            name: "Greeting".to_owned(),
            body_template: "Hello {{name}}".to_owned(),
            required_variables: vec!["name".to_owned()],
        };
        assert_eq!(draft.validate(), Ok(()));

        let invalid = AutomationTemplateDraft {
            required_variables: vec!["other".to_owned()],
            ..draft
        };
        assert_eq!(
            invalid.validate(),
            Err(AutomationError::InvalidRequest("body_template"))
        );
    }

    #[test]
    fn policy_rejects_duplicate_or_empty_chat_scope() {
        let draft = AutomationPolicyDraft {
            policy_id: "policy-1".to_owned(),
            template_id: "template-1".to_owned(),
            name: "Policy".to_owned(),
            enabled: true,
            account_id: "account-1".to_owned(),
            provider_chat_ids: vec!["chat-1".to_owned(), "chat-1".to_owned()],
            expires_at_unix_seconds: None,
        };
        assert_eq!(
            draft.validate(),
            Err(AutomationError::InvalidRequest("provider_chat_ids"))
        );
    }

    #[test]
    fn preview_is_scoped_and_deterministic() {
        let first = render_preview(&policy(), &template(), &request(), 100).expect("preview");
        let second = render_preview(&policy(), &template(), &request(), 100).expect("preview");

        assert_eq!(first, second);
        assert_eq!(first.rendered_text, "Hello Ada");
        assert_eq!(first.policy_revision, 4);
        assert_eq!(first.template_revision, 3);
    }

    #[test]
    fn preview_rejects_disabled_expired_and_foreign_scope() {
        let mut disabled = policy();
        disabled.enabled = false;
        assert_eq!(
            render_preview(&disabled, &template(), &request(), 100),
            Err(AutomationError::PolicyDisabled)
        );

        assert_eq!(
            render_preview(&policy(), &template(), &request(), 200),
            Err(AutomationError::PolicyExpired)
        );

        let mut foreign = request();
        foreign.provider_chat_id = "chat-2".to_owned();
        assert_eq!(
            render_preview(&policy(), &template(), &foreign, 100),
            Err(AutomationError::ScopeDenied)
        );
    }

    #[test]
    fn preview_rejects_missing_and_undeclared_variables() {
        let mut missing = request();
        missing.variables.clear();
        assert_eq!(
            render_preview(&policy(), &template(), &missing, 100),
            Err(AutomationError::MissingVariable)
        );

        let mut undeclared = request();
        undeclared.variables.push(AutomationVariable {
            name: "extra".to_owned(),
            value: "value".to_owned(),
        });
        assert_eq!(
            render_preview(&policy(), &template(), &undeclared, 100),
            Err(AutomationError::UndeclaredVariable)
        );
    }
}
