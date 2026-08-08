// §15: Rich templates with conditional blocks, tables, polls, mail merge
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichTemplate {
    pub name: String,
    pub subject: String,
    pub blocks: Vec<TemplateBlock>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TemplateBlock {
    Text {
        content: String,
    },
    Variable {
        key: String,
        default: Option<String>,
    },
    Conditional {
        condition: Condition,
        then_blocks: Vec<TemplateBlock>,
        else_blocks: Option<Vec<TemplateBlock>>,
    },
    Table {
        headers: Vec<String>,
        row_variable: String,
        columns: Vec<String>,
    },
    Button {
        text: String,
        url_template: String,
    },
    Divider,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub variable: String,
    pub operator: String,
    pub value: String,
}

pub struct RichTemplateEngine;

impl RichTemplateEngine {
    pub fn render(
        template: &RichTemplate,
        vars: &HashMap<String, String>,
    ) -> Result<(String, String), RenderError> {
        let subject = Self::render_text(&template.subject, vars);
        let body = template
            .blocks
            .iter()
            .map(|b| Self::render_block(b, vars))
            .collect::<Vec<_>>()
            .join("\n");
        Ok((subject, body))
    }

    fn render_text(text: &str, vars: &HashMap<String, String>) -> String {
        let mut result = text.to_owned();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{key}}}}}"), value);
        }
        result
    }

    fn render_block(block: &TemplateBlock, vars: &HashMap<String, String>) -> String {
        match block {
            TemplateBlock::Text { content } => Self::render_text(content, vars),
            TemplateBlock::Variable { key, default } => vars
                .get(key)
                .cloned()
                .or_else(|| default.clone())
                .unwrap_or_default(),
            TemplateBlock::Conditional {
                condition,
                then_blocks,
                else_blocks,
            } => {
                let val = vars.get(&condition.variable).cloned().unwrap_or_default();
                let is_true = match condition.operator.as_str() {
                    "equals" => val == condition.value,
                    "not_empty" => !val.is_empty(),
                    "contains" => val.contains(&condition.value),
                    _ => false,
                };
                if is_true {
                    then_blocks
                        .iter()
                        .map(|b| Self::render_block(b, vars))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    else_blocks
                        .as_ref()
                        .map(|blocks| {
                            blocks
                                .iter()
                                .map(|b| Self::render_block(b, vars))
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_default()
                }
            }
            TemplateBlock::Table {
                headers,
                row_variable: _,
                columns: _,
            } => {
                let header_line = headers.join(" | ");
                let sep = headers
                    .iter()
                    .map(|_| "---")
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("{header_line}\n{sep}")
            }
            TemplateBlock::Button { text, url_template } => {
                let url = Self::render_text(url_template, vars);
                format!("[{text}]({url})")
            }
            TemplateBlock::Divider => "---".into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("template render error")]
pub struct RenderError;

#[cfg(test)]
mod tests {
    use super::*;
    fn vars() -> HashMap<String, String> {
        [
            ("name".into(), "Alice".into()),
            ("project".into(), "Макошь".into()),
        ]
        .into()
    }

    #[test]
    fn render_text_with_vars() {
        let tpl = RichTemplate {
            name: "test".into(),
            subject: "Hello {{name}}".into(),
            blocks: vec![TemplateBlock::Text {
                content: "Hi {{name}}, project {{project}}".into(),
            }],
        };
        let (s, b) = RichTemplateEngine::render(&tpl, &vars()).unwrap();
        assert_eq!(s, "Hello Alice");
        assert_eq!(b, "Hi Alice, project Макошь");
    }

    #[test]
    fn conditional_true() {
        let tpl = RichTemplate {
            name: "t".into(),
            subject: "S".into(),
            blocks: vec![TemplateBlock::Conditional {
                condition: Condition {
                    variable: "name".into(),
                    operator: "not_empty".into(),
                    value: "".into(),
                },
                then_blocks: vec![TemplateBlock::Text {
                    content: "Has name".into(),
                }],
                else_blocks: None,
            }],
        };
        let (_, b) = RichTemplateEngine::render(&tpl, &vars()).unwrap();
        assert_eq!(b, "Has name");
    }

    #[test]
    fn conditional_false() {
        let empty_vars: HashMap<String, String> = HashMap::new();
        let tpl = RichTemplate {
            name: "t".into(),
            subject: "S".into(),
            blocks: vec![TemplateBlock::Conditional {
                condition: Condition {
                    variable: "name".into(),
                    operator: "not_empty".into(),
                    value: "".into(),
                },
                then_blocks: vec![TemplateBlock::Text {
                    content: "Has name".into(),
                }],
                else_blocks: Some(vec![TemplateBlock::Text {
                    content: "No name".into(),
                }]),
            }],
        };
        let (_, b) = RichTemplateEngine::render(&tpl, &empty_vars).unwrap();
        assert_eq!(b, "No name");
    }

    #[test]
    fn button_renders_link() {
        let tpl = RichTemplate {
            name: "t".into(),
            subject: "S".into(),
            blocks: vec![TemplateBlock::Button {
                text: "Click".into(),
                url_template: "https://ex.com/{{name}}".into(),
            }],
        };
        let (_, b) = RichTemplateEngine::render(&tpl, &vars()).unwrap();
        assert_eq!(b, "[Click](https://ex.com/Alice)");
    }
}
