use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationTemplate {
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    #[serde(default)]
    pub placeholder_variables: Vec<String>,
    #[serde(default)]
    pub undeclared_variables: Vec<String>,
    #[serde(default)]
    pub unused_variables: Vec<String>,
    #[serde(default)]
    pub malformed_placeholders: Vec<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CommunicationTemplateStore {
    pool: PgPool,
}

impl CommunicationTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        tpl: &NewCommunicationTemplate,
    ) -> Result<CommunicationTemplate, CommunicationTemplateError> {
        tpl.validate()?;
        let vars: Value = tpl
            .variables
            .iter()
            .map(|v| Value::String(v.clone()))
            .collect();
        let row = sqlx::query(
            r#"INSERT INTO communication_templates (template_id, name, subject_template, body_template, variables, language)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (template_id) DO UPDATE SET
                name = EXCLUDED.name, subject_template = EXCLUDED.subject_template,
                body_template = EXCLUDED.body_template, variables = EXCLUDED.variables,
                language = EXCLUDED.language, updated_at = now()
            RETURNING template_id, name, subject_template, body_template, variables, language, created_at, updated_at"#,
        )
        .bind(&tpl.template_id).bind(&tpl.name).bind(&tpl.subject_template).bind(&tpl.body_template)
        .bind(&vars).bind(tpl.language.as_deref())
        .fetch_one(&self.pool).await?;
        row_to_template(row)
    }

    pub async fn list(&self) -> Result<Vec<CommunicationTemplate>, CommunicationTemplateError> {
        let rows = sqlx::query(
            r#"SELECT template_id, name, subject_template, body_template, variables, language, created_at, updated_at
            FROM communication_templates ORDER BY name"#,
        ).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_template).collect()
    }

    pub async fn get(
        &self,
        template_id: &str,
    ) -> Result<Option<CommunicationTemplate>, CommunicationTemplateError> {
        let row = sqlx::query(
            r#"SELECT template_id, name, subject_template, body_template, variables, language, created_at, updated_at
            FROM communication_templates WHERE template_id = $1"#,
        ).bind(template_id).fetch_optional(&self.pool).await?;
        row.map(row_to_template).transpose()
    }

    pub async fn delete(&self, template_id: &str) -> Result<bool, CommunicationTemplateError> {
        let result = sqlx::query("DELETE FROM communication_templates WHERE template_id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Render a template with variables.
    pub fn render(
        &self,
        template: &CommunicationTemplate,
        vars: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, CommunicationTemplateError> {
        let missing_variables = template
            .variables
            .iter()
            .filter(|variable| {
                vars.get(variable.as_str())
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        let subject = render_template_text(&template.subject_template, vars);
        let body = render_template_text(&template.body_template, vars);
        let unresolved_variables = unique_strings(
            subject
                .unresolved_variables
                .iter()
                .chain(body.unresolved_variables.iter()),
        );
        let malformed_placeholders = unique_strings(
            subject
                .malformed_placeholders
                .iter()
                .chain(body.malformed_placeholders.iter()),
        );
        Ok(RenderedTemplate {
            subject: subject.text,
            body: body.text,
            missing_variables,
            unresolved_variables,
            malformed_placeholders,
        })
    }

    pub fn render_mail_merge_preview(
        &self,
        template: &CommunicationTemplate,
        rows: Vec<CommunicationMergePreviewRow>,
    ) -> Result<CommunicationMergePreview, CommunicationTemplateError> {
        let template_has_blocking_diagnostics = !template.undeclared_variables.is_empty()
            || !template.malformed_placeholders.is_empty();
        let items = rows
            .into_iter()
            .map(|row| {
                let rendered = self.render(template, &row.variables)?;
                let ready = !template_has_blocking_diagnostics
                    && rendered.missing_variables.is_empty()
                    && rendered.unresolved_variables.is_empty()
                    && rendered.malformed_placeholders.is_empty();
                Ok(CommunicationMergePreviewItem {
                    row_id: row.row_id,
                    ready,
                    rendered,
                })
            })
            .collect::<Result<Vec<_>, CommunicationTemplateError>>()?;
        let ready_count = items.iter().filter(|item| item.ready).count();
        let row_count = items.len();
        Ok(CommunicationMergePreview {
            template_id: template.template_id.clone(),
            row_count,
            ready_count,
            blocked_count: row_count.saturating_sub(ready_count),
            items,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NewCommunicationTemplate {
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub language: Option<String>,
}

impl NewCommunicationTemplate {
    fn validate(&self) -> Result<(), CommunicationTemplateError> {
        if self.template_id.trim().is_empty() {
            return Err(CommunicationTemplateError::InvalidTemplate(
                "template_id empty",
            ));
        }
        if self.name.trim().is_empty() {
            return Err(CommunicationTemplateError::InvalidTemplate("name empty"));
        }
        if self.subject_template.trim().is_empty() {
            return Err(CommunicationTemplateError::InvalidTemplate(
                "subject_template empty",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RenderedTemplate {
    pub subject: String,
    pub body: String,
    pub missing_variables: Vec<String>,
    pub unresolved_variables: Vec<String>,
    pub malformed_placeholders: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CommunicationMergePreviewRow {
    pub row_id: String,
    pub variables: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationMergePreview {
    pub template_id: String,
    pub row_count: usize,
    pub ready_count: usize,
    pub blocked_count: usize,
    pub items: Vec<CommunicationMergePreviewItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationMergePreviewItem {
    pub row_id: String,
    pub ready: bool,
    pub rendered: RenderedTemplate,
}

struct RenderedTemplateText {
    text: String,
    unresolved_variables: Vec<String>,
    malformed_placeholders: Vec<String>,
}

struct TemplateValidation {
    placeholder_variables: Vec<String>,
    undeclared_variables: Vec<String>,
    unused_variables: Vec<String>,
    malformed_placeholders: Vec<String>,
}

struct CommunicationTemplateMetadataInput {
    template_id: String,
    name: String,
    subject_template: String,
    body_template: String,
    variables: Vec<String>,
    language: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn row_to_template(row: PgRow) -> Result<CommunicationTemplate, CommunicationTemplateError> {
    let vars: Value = row.try_get("variables")?;
    let variables: Vec<String> = vars
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Ok(email_template_with_metadata(
        CommunicationTemplateMetadataInput {
            template_id: row.try_get("template_id")?,
            name: row.try_get("name")?,
            subject_template: row.try_get("subject_template")?,
            body_template: row.try_get("body_template")?,
            variables,
            language: row.try_get("language")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        },
    ))
}

fn email_template_with_metadata(
    input: CommunicationTemplateMetadataInput,
) -> CommunicationTemplate {
    let validation = validate_template_content(
        &input.subject_template,
        &input.body_template,
        &input.variables,
    );
    CommunicationTemplate {
        template_id: input.template_id,
        name: input.name,
        subject_template: input.subject_template,
        body_template: input.body_template,
        variables: input.variables,
        placeholder_variables: validation.placeholder_variables,
        undeclared_variables: validation.undeclared_variables,
        unused_variables: validation.unused_variables,
        malformed_placeholders: validation.malformed_placeholders,
        language: input.language,
        created_at: input.created_at,
        updated_at: input.updated_at,
    }
}

fn validate_template_content(
    subject_template: &str,
    body_template: &str,
    variables: &[String],
) -> TemplateValidation {
    let empty_vars = HashMap::new();
    let subject = render_template_text(subject_template, &empty_vars);
    let body = render_template_text(body_template, &empty_vars);
    let placeholder_variables = unique_strings(
        subject
            .unresolved_variables
            .iter()
            .chain(body.unresolved_variables.iter()),
    );
    let malformed_placeholders = unique_strings(
        subject
            .malformed_placeholders
            .iter()
            .chain(body.malformed_placeholders.iter()),
    );
    let undeclared_variables = strings_not_in(&placeholder_variables, variables);
    let unused_variables = strings_not_in(variables, &placeholder_variables);

    TemplateValidation {
        placeholder_variables,
        undeclared_variables,
        unused_variables,
        malformed_placeholders,
    }
}

fn strings_not_in(source: &[String], excluded: &[String]) -> Vec<String> {
    source
        .iter()
        .filter(|value| {
            !excluded
                .iter()
                .any(|excluded_value| excluded_value.as_str() == value.as_str())
        })
        .cloned()
        .collect()
}

fn render_template_text(template: &str, vars: &HashMap<String, String>) -> RenderedTemplateText {
    let mut rendered = String::with_capacity(template.len());
    let mut unresolved_variables = Vec::new();
    let mut malformed_placeholders = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        rendered.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        let Some(end) = after_open.find("}}") else {
            let malformed = &rest[start..];
            rendered.push_str(malformed);
            if !malformed_placeholders
                .iter()
                .any(|existing: &String| existing.as_str() == malformed)
            {
                malformed_placeholders.push(malformed.to_owned());
            }
            return RenderedTemplateText {
                text: rendered,
                unresolved_variables,
                malformed_placeholders,
            };
        };

        let key = after_open[..end].trim();
        if key.is_empty() {
            let malformed = &rest[start..start + 2 + end + 2];
            rendered.push_str(malformed);
            if !malformed_placeholders
                .iter()
                .any(|existing: &String| existing.as_str() == malformed)
            {
                malformed_placeholders.push(malformed.to_owned());
            }
        } else if !is_valid_template_variable_name(key) {
            let malformed = &rest[start..start + 2 + end + 2];
            rendered.push_str(malformed);
            if !malformed_placeholders
                .iter()
                .any(|existing: &String| existing.as_str() == malformed)
            {
                malformed_placeholders.push(malformed.to_owned());
            }
        } else if let Some(value) = vars.get(key).filter(|value| !value.trim().is_empty()) {
            rendered.push_str(value);
        } else {
            rendered.push_str(&rest[start..start + 2 + end + 2]);
            if !key.is_empty()
                && !unresolved_variables
                    .iter()
                    .any(|existing: &String| existing.as_str() == key)
            {
                unresolved_variables.push(key.to_owned());
            }
        }
        rest = &after_open[end + 2..];
    }

    rendered.push_str(rest);
    RenderedTemplateText {
        text: rendered,
        unresolved_variables,
        malformed_placeholders,
    }
}

fn is_valid_template_variable_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

fn unique_strings<'a>(values: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(value) {
            unique.push(value.clone());
        }
    }
    unique
}

#[derive(Debug, Error)]
pub enum CommunicationTemplateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid template: {0}")]
    InvalidTemplate(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_template(
        subject_template: &str,
        body_template: &str,
        variables: Vec<String>,
    ) -> CommunicationTemplate {
        email_template_with_metadata(CommunicationTemplateMetadataInput {
            template_id: "t1".into(),
            name: "Test".into(),
            subject_template: subject_template.into(),
            body_template: body_template.into(),
            variables,
            language: Some("en".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn render_template_substitutes_variables() {
        let tpl = test_template(
            "Hello {{name}}",
            "Hi {{name}},\n\n{{message}}",
            vec!["name".into(), "message".into()],
        );
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "Alice".into());
        vars.insert("message".into(), "How are you?".into());
        let store = CommunicationTemplateStore::new(
            PgPool::connect_lazy("postgres://localhost/makosh_test").unwrap(),
        );
        let rendered = store.render(&tpl, &vars).unwrap();
        assert_eq!(rendered.subject, "Hello Alice");
        assert_eq!(rendered.body, "Hi Alice,\n\nHow are you?");
        assert!(rendered.missing_variables.is_empty());
        assert!(rendered.unresolved_variables.is_empty());
        assert!(rendered.malformed_placeholders.is_empty());
    }

    #[tokio::test]
    async fn render_template_reports_missing_and_unresolved_variables() {
        let tpl = test_template(
            "Hello {{ name }}",
            "Hi {{  name  }},\n\n{{ message }} {{unknown}} {{ blank }}",
            vec!["name".into(), "message".into(), "blank".into()],
        );
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "Alice".into());
        vars.insert("blank".into(), " ".into());
        let store = CommunicationTemplateStore::new(
            PgPool::connect_lazy("postgres://localhost/makosh_test").unwrap(),
        );
        let rendered = store.render(&tpl, &vars).unwrap();
        assert_eq!(rendered.subject, "Hello Alice");
        assert_eq!(
            rendered.body,
            "Hi Alice,\n\n{{ message }} {{unknown}} {{ blank }}"
        );
        assert_eq!(rendered.missing_variables, vec!["message", "blank"]);
        assert_eq!(
            rendered.unresolved_variables,
            vec!["message", "unknown", "blank"]
        );
        assert!(rendered.malformed_placeholders.is_empty());
    }

    #[tokio::test]
    async fn render_template_reports_malformed_placeholders() {
        let tpl = test_template(
            "Hello {{ name",
            "Body {{ }} and {{ project }} {{ first name }}",
            vec!["name".into(), "project".into()],
        );
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "Alice".into());
        vars.insert("project".into(), "Макошь".into());
        let store = CommunicationTemplateStore::new(
            PgPool::connect_lazy("postgres://localhost/makosh_test").unwrap(),
        );
        let rendered = store.render(&tpl, &vars).unwrap();
        assert_eq!(rendered.subject, "Hello {{ name");
        assert_eq!(rendered.body, "Body {{ }} and Макошь {{ first name }}");
        assert!(rendered.missing_variables.is_empty());
        assert!(rendered.unresolved_variables.is_empty());
        assert_eq!(
            rendered.malformed_placeholders,
            vec!["{{ name", "{{ }}", "{{ first name }}"]
        );
    }

    #[test]
    fn template_metadata_reports_undeclared_and_unused_variables() {
        let template = email_template_with_metadata(CommunicationTemplateMetadataInput {
            template_id: "t1".into(),
            name: "Mismatch".into(),
            subject_template: "Hello {{ recipient }}".into(),
            body_template: "Project {{ project }} {{ }} {{ first name }} {{ broken".into(),
            variables: vec!["recipient".into(), "legacy".into()],
            language: Some("en".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        assert_eq!(template.placeholder_variables, vec!["recipient", "project"]);
        assert_eq!(template.undeclared_variables, vec!["project"]);
        assert_eq!(template.unused_variables, vec!["legacy"]);
        assert_eq!(
            template.malformed_placeholders,
            vec!["{{ }}", "{{ first name }}", "{{ broken"]
        );
    }
}
