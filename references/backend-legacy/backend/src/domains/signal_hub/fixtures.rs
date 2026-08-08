use std::sync::OnceLock;

use serde::Deserialize;

use makosh_signal_hub_api::policies::{SignalPolicyMode, SignalPolicyScope};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemSourceFixture {
    pub code: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    pub source_kind: &'static str,
    pub default_enabled: bool,
    pub supports_connections: bool,
    pub supports_runtime: bool,
    pub supports_replay: bool,
    pub supports_pause: bool,
    pub supports_mute: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemProfilePolicyFixture {
    pub scope: SignalPolicyScope,
    pub source_code: Option<&'static str>,
    pub event_pattern: Option<&'static str>,
    pub mode: SignalPolicyMode,
    pub reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemProfileFixture {
    pub code: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub is_system: bool,
    pub policies: &'static [SystemProfilePolicyFixture],
}

pub fn system_source_fixtures() -> &'static [SystemSourceFixture] {
    SYSTEM_SOURCE_FIXTURES
        .get_or_init(load_system_source_fixtures)
        .as_slice()
}

pub fn system_profile_fixtures() -> &'static [SystemProfileFixture] {
    &SYSTEM_PROFILE_FIXTURES
}

static SYSTEM_SOURCE_FIXTURES: OnceLock<Vec<SystemSourceFixture>> = OnceLock::new();

fn load_system_source_fixtures() -> Vec<SystemSourceFixture> {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/signal_hub/system_sources.toml"
    ));
    let catalog: RawSystemSourceCatalog =
        toml::from_str(raw).expect("signal_hub system_sources.toml must parse");

    catalog
        .sources
        .into_iter()
        .map(|source| SystemSourceFixture {
            code: leak_string(source.code),
            display_name: leak_string(source.display_name),
            category: leak_string(source.category),
            source_kind: leak_string(source.source_kind),
            default_enabled: source.default_enabled,
            supports_connections: source.supports_connections,
            supports_runtime: source.supports_runtime,
            supports_replay: source.supports_replay,
            supports_pause: source.supports_pause,
            supports_mute: source.supports_mute,
        })
        .collect()
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

#[derive(Debug, Deserialize)]
struct RawSystemSourceCatalog {
    sources: Vec<RawSystemSourceFixture>,
}

#[derive(Debug, Deserialize)]
struct RawSystemSourceFixture {
    code: String,
    display_name: String,
    category: String,
    source_kind: String,
    default_enabled: bool,
    supports_connections: bool,
    supports_runtime: bool,
    supports_replay: bool,
    supports_pause: bool,
    supports_mute: bool,
}

const DEVELOPMENT_PROFILE_POLICIES: [SystemProfilePolicyFixture; 2] = [
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("rss"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "development profile mutes noisy RSS capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("browser"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "development profile mutes browser capture by default",
    },
];

const TESTING_PROFILE_POLICIES: [SystemProfilePolicyFixture; 13] = [
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("ai"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes AI runtime signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("browser"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes browser capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("calendar"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes calendar provider signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("filesystem"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes filesystem capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("github"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes GitHub provider signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("home_assistant"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Home Assistant signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("mail"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes mail provider signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("rss"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes RSS signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("telegram"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Telegram signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("voice"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes voice capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("whatsapp"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes WhatsApp signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("zulip"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Zulip signals",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("zoom"),
        event_pattern: None,
        mode: SignalPolicyMode::Muted,
        reason: "testing profile mutes Zoom signals",
    },
];

const MAINTENANCE_PROFILE_POLICIES: [SystemProfilePolicyFixture; 5] = [
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("mail"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses mail capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("telegram"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses Telegram capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("whatsapp"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses WhatsApp capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("zulip"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses Zulip capture",
    },
    SystemProfilePolicyFixture {
        scope: SignalPolicyScope::Source,
        source_code: Some("zoom"),
        event_pattern: None,
        mode: SignalPolicyMode::Paused,
        reason: "maintenance profile pauses Zoom capture",
    },
];

const PRODUCTION_PROFILE_POLICIES: [SystemProfilePolicyFixture; 0] = [];

const SYSTEM_PROFILE_FIXTURES: [SystemProfileFixture; 4] = [
    SystemProfileFixture {
        code: "production",
        display_name: "Production",
        description: "All configured real sources run according to owner settings.",
        is_system: true,
        policies: &PRODUCTION_PROFILE_POLICIES,
    },
    SystemProfileFixture {
        code: "development",
        display_name: "Development",
        description: "Selected noisy sources stay muted during local development.",
        is_system: true,
        policies: &DEVELOPMENT_PROFILE_POLICIES,
    },
    SystemProfileFixture {
        code: "testing",
        display_name: "Testing",
        description: "Real sources are muted while deterministic fixture signals stay available.",
        is_system: true,
        policies: &TESTING_PROFILE_POLICIES,
    },
    SystemProfileFixture {
        code: "maintenance",
        display_name: "Maintenance",
        description: "Capture pauses while replay and recovery operations remain available.",
        is_system: true,
        policies: &MAINTENANCE_PROFILE_POLICIES,
    },
];

#[cfg(test)]
mod tests {
    use super::system_source_fixtures;

    #[test]
    fn system_source_fixtures_are_loaded_from_canonical_catalog() {
        let codes: Vec<_> = system_source_fixtures()
            .iter()
            .map(|fixture| fixture.code)
            .collect();

        assert_eq!(
            codes,
            vec![
                "system",
                "ai",
                "mail",
                "telegram",
                "whatsapp",
                "zulip",
                "zoom",
                "yandex_telemost",
                "github",
                "browser",
                "rss",
                "calendar",
                "filesystem",
                "home_assistant",
                "voice",
                "fixture",
            ]
        );
    }
}
