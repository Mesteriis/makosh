use serde::Serialize;

use super::errors::AiError;

#[derive(Clone, Debug, Serialize)]
pub struct AiAgentListResponse {
    pub items: Vec<AiAgentDescriptor>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiAgentDescriptor {
    pub agent_id: &'static str,
    pub display_name: &'static str,
    pub role: &'static str,
    pub default_model: String,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persona_email: Option<String>,
}

pub fn v3_agents(chat_model: &str) -> Vec<AiAgentDescriptor> {
    vec![
        AiAgentDescriptor {
            agent_id: "HESTIA",
            display_name: "hestia@sh-inc.ru",
            role: "meeting prep and home context briefing",
            default_model: chat_model.to_owned(),
            status: "available",
            persona_id: None,
            persona_type: None,
            persona_email: None,
        },
        AiAgentDescriptor {
            agent_id: "MAKOSH",
            display_name: "makosh@sh-inc.ru",
            role: "workflow coordination and task candidate extraction",
            default_model: chat_model.to_owned(),
            status: "available",
            persona_id: None,
            persona_type: None,
            persona_email: None,
        },
        AiAgentDescriptor {
            agent_id: "MNEMOSYNE",
            display_name: "mnemosyne@sh-inc.ru",
            role: "source-backed memory answers",
            default_model: chat_model.to_owned(),
            status: "available",
            persona_id: None,
            persona_type: None,
            persona_email: None,
        },
        AiAgentDescriptor {
            agent_id: "ATHENA",
            display_name: "athena@sh-inc.ru",
            role: "planning review and decision support",
            default_model: chat_model.to_owned(),
            status: "available",
            persona_id: None,
            persona_type: None,
            persona_email: None,
        },
        AiAgentDescriptor {
            agent_id: "HEPHAESTUS",
            display_name: "hephaestus@sh-inc.ru",
            role: "development, maintenance and tool automation",
            default_model: chat_model.to_owned(),
            status: "available",
            persona_id: None,
            persona_type: None,
            persona_email: None,
        },
    ]
}

pub(super) fn validate_agent(agent_id: &str) -> Result<(), AiError> {
    match agent_id {
        "HESTIA" | "MAKOSH" | "MNEMOSYNE" | "ATHENA" | "HEPHAESTUS" => Ok(()),
        _ => Err(AiError::UnknownAgent(agent_id.to_owned())),
    }
}

pub(super) fn ai_agent_display_name(agent_id: &str) -> Result<&'static str, AiError> {
    match agent_id {
        "HESTIA" => Ok("hestia@sh-inc.ru"),
        "MAKOSH" => Ok("makosh@sh-inc.ru"),
        "MNEMOSYNE" => Ok("mnemosyne@sh-inc.ru"),
        "ATHENA" => Ok("athena@sh-inc.ru"),
        "HEPHAESTUS" => Ok("hephaestus@sh-inc.ru"),
        _ => Err(AiError::UnknownAgent(agent_id.to_owned())),
    }
}
