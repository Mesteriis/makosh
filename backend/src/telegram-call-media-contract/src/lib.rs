use std::fmt;

use zeroize::Zeroize;

pub const PACKAGE: &str = "makosh-telegram-call-media-contract";
pub const TD_CALL_MIN_LAYER_V1: i32 = 65;
pub const TD_CALL_MAX_LAYER_V1: i32 = 92;
pub const MAX_LIBRARY_VERSION_BYTES: usize = 128;
pub const MAX_LIBRARY_VERSIONS: usize = 16;
pub const MAX_CALL_SESSION_ID_BYTES: usize = 128;
pub const MAX_SERVER_HOST_BYTES: usize = 255;
pub const MAX_SERVER_CREDENTIAL_BYTES: usize = 4 * 1024;
pub const MAX_READY_TEXT_BYTES: usize = 256 * 1024;
pub const MAX_SIGNALING_DATA_BYTES: usize = 256 * 1024;
pub const MAX_CALL_SERVERS: usize = 64;
pub const CALL_ENCRYPTION_KEY_BYTES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallProtocolV1 {
    pub udp_p2p: bool,
    pub udp_reflector: bool,
    pub min_layer: i32,
    pub max_layer: i32,
    pub library_versions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallPeerProtocolV1 {
    pub udp_p2p: bool,
    pub udp_reflector: bool,
    pub min_layer: i32,
    pub max_layer: i32,
    pub library_versions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallMediaContractError {
    InvalidProtocol,
    InvalidPlan,
    Unavailable,
    SessionNotFound,
    InvalidState,
    QueueOverflow,
    NativeFailure,
}

impl TelegramCallProtocolV1 {
    pub fn new(
        udp_p2p: bool,
        udp_reflector: bool,
        library_versions: Vec<String>,
    ) -> Result<Self, TelegramCallMediaContractError> {
        let protocol = Self {
            udp_p2p,
            udp_reflector,
            min_layer: TD_CALL_MIN_LAYER_V1,
            max_layer: TD_CALL_MAX_LAYER_V1,
            library_versions,
        };
        protocol.validate()?;
        Ok(protocol)
    }

    pub fn validate(&self) -> Result<(), TelegramCallMediaContractError> {
        if (!self.udp_p2p && !self.udp_reflector)
            || self.min_layer != TD_CALL_MIN_LAYER_V1
            || self.max_layer != TD_CALL_MAX_LAYER_V1
            || self.library_versions.is_empty()
            || self.library_versions.len() > MAX_LIBRARY_VERSIONS
            || self.library_versions.iter().any(|version| {
                version.trim().is_empty()
                    || version.len() > MAX_LIBRARY_VERSION_BYTES
                    || version.chars().any(char::is_control)
            })
        {
            return Err(TelegramCallMediaContractError::InvalidProtocol);
        }
        Ok(())
    }
}

impl TelegramCallPeerProtocolV1 {
    pub fn select_library(
        &self,
        local: &TelegramCallProtocolV1,
    ) -> Result<String, TelegramCallMediaContractError> {
        local.validate()?;
        if (!self.udp_p2p && !self.udp_reflector)
            || self.min_layer <= 0
            || self.max_layer < self.min_layer
            || self.library_versions.is_empty()
            || self.library_versions.len() > MAX_LIBRARY_VERSIONS
            || self.library_versions.iter().any(|version| {
                version.trim().is_empty()
                    || version.len() > MAX_LIBRARY_VERSION_BYTES
                    || version.chars().any(char::is_control)
            })
            || self.max_layer < local.min_layer
            || self.min_layer > local.max_layer
            || (!self.udp_p2p || !local.udp_p2p) && (!self.udp_reflector || !local.udp_reflector)
        {
            return Err(TelegramCallMediaContractError::InvalidProtocol);
        }
        local
            .library_versions
            .iter()
            .find(|version| self.library_versions.contains(version))
            .cloned()
            .ok_or(TelegramCallMediaContractError::InvalidProtocol)
    }
}

pub struct TelegramCallSecretBytesV1(Vec<u8>);

impl TelegramCallSecretBytesV1 {
    pub fn new(
        bytes: Vec<u8>,
        maximum_bytes: usize,
    ) -> Result<Self, TelegramCallMediaContractError> {
        if bytes.is_empty() || bytes.len() > maximum_bytes {
            return Err(TelegramCallMediaContractError::InvalidPlan);
        }
        Ok(Self(bytes))
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for TelegramCallSecretBytesV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TelegramCallSecretBytesV1([REDACTED])")
    }
}

impl Drop for TelegramCallSecretBytesV1 {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub struct TelegramCallSecretTextV1(String);

impl TelegramCallSecretTextV1 {
    pub fn new(text: String, maximum_bytes: usize) -> Result<Self, TelegramCallMediaContractError> {
        if text.len() > maximum_bytes || text.contains('\0') {
            return Err(TelegramCallMediaContractError::InvalidPlan);
        }
        Ok(Self(text))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for TelegramCallSecretTextV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TelegramCallSecretTextV1([REDACTED])")
    }
}

impl Drop for TelegramCallSecretTextV1 {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug)]
pub enum TelegramCallServerKindV1 {
    TelegramReflector {
        reflector_id: u8,
        peer_tag: [u8; 16],
        is_tcp: bool,
    },
    WebRtc {
        username: TelegramCallSecretTextV1,
        password: TelegramCallSecretTextV1,
        supports_stun: bool,
        supports_turn: bool,
    },
}

#[derive(Debug)]
pub struct TelegramCallServerV1 {
    pub ipv4: String,
    pub ipv6: String,
    pub port: u16,
    pub kind: TelegramCallServerKindV1,
}

impl TelegramCallServerV1 {
    fn validate(&self) -> Result<(), TelegramCallMediaContractError> {
        let valid_host = |host: &str| {
            host.len() <= MAX_SERVER_HOST_BYTES
                && !host.contains('\0')
                && !host.chars().any(char::is_control)
        };
        if self.port == 0
            || (self.ipv4.is_empty() && self.ipv6.is_empty())
            || !valid_host(&self.ipv4)
            || !valid_host(&self.ipv6)
        {
            return Err(TelegramCallMediaContractError::InvalidPlan);
        }
        if let TelegramCallServerKindV1::WebRtc {
            supports_stun,
            supports_turn,
            ..
        } = &self.kind
            && !supports_stun
            && !supports_turn
        {
            return Err(TelegramCallMediaContractError::InvalidPlan);
        }
        Ok(())
    }
}

pub struct TelegramCallReadyPlanV1 {
    pub call_session_id: String,
    pub library_version: String,
    pub servers: Vec<TelegramCallServerV1>,
    pub allow_p2p: bool,
    pub allow_tcp: bool,
    pub call_config: TelegramCallSecretTextV1,
    pub custom_parameters: TelegramCallSecretTextV1,
    pub encryption_key: TelegramCallSecretBytesV1,
    pub is_outgoing: bool,
}

pub struct TelegramCallReadyMaterialV1 {
    pub peer_protocol: TelegramCallPeerProtocolV1,
    pub servers: Vec<TelegramCallServerV1>,
    pub allow_p2p: bool,
    pub allow_tcp: bool,
    pub call_config: TelegramCallSecretTextV1,
    pub custom_parameters: TelegramCallSecretTextV1,
    pub encryption_key: TelegramCallSecretBytesV1,
    pub is_outgoing: bool,
}

impl fmt::Debug for TelegramCallReadyMaterialV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TelegramCallReadyMaterialV1")
            .field("peer_protocol", &self.peer_protocol)
            .field("servers", &self.servers)
            .field("allow_p2p", &self.allow_p2p)
            .field("allow_tcp", &self.allow_tcp)
            .field("call_config", &"[REDACTED]")
            .field("custom_parameters", &"[REDACTED]")
            .field("encryption_key", &"[REDACTED]")
            .field("is_outgoing", &self.is_outgoing)
            .finish()
    }
}

impl TelegramCallReadyMaterialV1 {
    pub fn into_plan(
        self,
        call_session_id: String,
        local_protocol: &TelegramCallProtocolV1,
    ) -> Result<TelegramCallReadyPlanV1, TelegramCallMediaContractError> {
        let library_version = self.peer_protocol.select_library(local_protocol)?;
        let plan = TelegramCallReadyPlanV1 {
            call_session_id,
            library_version,
            servers: self.servers,
            allow_p2p: self.allow_p2p && local_protocol.udp_p2p,
            allow_tcp: self.allow_tcp,
            call_config: self.call_config,
            custom_parameters: self.custom_parameters,
            encryption_key: self.encryption_key,
            is_outgoing: self.is_outgoing,
        };
        plan.validate()?;
        Ok(plan)
    }
}

impl fmt::Debug for TelegramCallReadyPlanV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TelegramCallReadyPlanV1")
            .field("call_session_id", &self.call_session_id)
            .field("library_version", &self.library_version)
            .field("servers", &self.servers)
            .field("allow_p2p", &self.allow_p2p)
            .field("allow_tcp", &self.allow_tcp)
            .field("call_config", &"[REDACTED]")
            .field("custom_parameters", &"[REDACTED]")
            .field("encryption_key", &"[REDACTED]")
            .field("is_outgoing", &self.is_outgoing)
            .finish()
    }
}

impl TelegramCallReadyPlanV1 {
    pub fn validate(&self) -> Result<(), TelegramCallMediaContractError> {
        if self.call_session_id.trim().is_empty()
            || self.call_session_id.len() > MAX_CALL_SESSION_ID_BYTES
            || self.call_session_id.chars().any(char::is_control)
            || self.library_version.trim().is_empty()
            || self.library_version.len() > MAX_LIBRARY_VERSION_BYTES
            || self.library_version.chars().any(char::is_control)
            || self.servers.is_empty()
            || self.servers.len() > MAX_CALL_SERVERS
            || self.encryption_key.expose().len() != CALL_ENCRYPTION_KEY_BYTES
        {
            return Err(TelegramCallMediaContractError::InvalidPlan);
        }
        self.servers
            .iter()
            .try_for_each(TelegramCallServerV1::validate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallMediaStateV1 {
    Connecting,
    Established,
    Reconnecting,
    Failed,
}

#[derive(Debug)]
pub enum TelegramCallMediaEventV1 {
    State(TelegramCallMediaStateV1),
    OutboundSignaling(TelegramCallSecretBytesV1),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelegramCallDiscardContextV1 {
    pub duration_seconds: u32,
    pub connection_id: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelegramCallMediaFinalV1 {
    pub discard_context: TelegramCallDiscardContextV1,
    pub failed: bool,
}

pub trait TelegramCallSignalingMediaPort {
    fn supported_protocol(&self) -> Result<TelegramCallProtocolV1, TelegramCallMediaContractError>;

    fn start_session(
        &mut self,
        plan: TelegramCallReadyPlanV1,
    ) -> Result<(), TelegramCallMediaContractError>;

    fn receive_signaling_data(
        &mut self,
        call_session_id: &str,
        data: TelegramCallSecretBytesV1,
    ) -> Result<(), TelegramCallMediaContractError>;

    fn poll_event(
        &mut self,
        call_session_id: &str,
    ) -> Result<Option<TelegramCallMediaEventV1>, TelegramCallMediaContractError>;

    fn stop_session(
        &mut self,
        call_session_id: &str,
    ) -> Result<TelegramCallMediaFinalV1, TelegramCallMediaContractError>;

    fn discard_context(
        &self,
        call_session_id: &str,
    ) -> Result<TelegramCallDiscardContextV1, TelegramCallMediaContractError>;

    fn set_local_mute(
        &mut self,
        call_session_id: &str,
        muted: bool,
    ) -> Result<(), TelegramCallMediaContractError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_requires_exact_tdlib_layers_and_one_pinned_library_version() {
        let protocol = TelegramCallProtocolV1::new(true, true, vec!["pinned-tgcalls".to_owned()])
            .expect("protocol");

        assert_eq!(protocol.min_layer, 65);
        assert_eq!(protocol.max_layer, 92);
        assert_eq!(
            TelegramCallProtocolV1::new(true, true, Vec::new()),
            Err(TelegramCallMediaContractError::InvalidProtocol)
        );
    }

    #[test]
    fn peer_protocol_selects_only_an_exact_local_library_with_overlapping_layers() {
        let local = TelegramCallProtocolV1::new(true, true, vec!["pinned-tgcalls".to_owned()])
            .expect("local protocol");
        let peer = TelegramCallPeerProtocolV1 {
            udp_p2p: true,
            udp_reflector: true,
            min_layer: 70,
            max_layer: 100,
            library_versions: vec!["other".to_owned(), "pinned-tgcalls".to_owned()],
        };
        assert_eq!(peer.select_library(&local), Ok("pinned-tgcalls".to_owned()));
        assert_eq!(
            TelegramCallPeerProtocolV1 {
                library_versions: vec!["other".to_owned()],
                ..peer
            }
            .select_library(&local),
            Err(TelegramCallMediaContractError::InvalidProtocol)
        );
    }

    #[test]
    fn ready_plan_debug_and_secret_debug_are_redacted() {
        let plan = TelegramCallReadyPlanV1 {
            call_session_id: "call-1".to_owned(),
            library_version: "13.0.0".to_owned(),
            servers: vec![TelegramCallServerV1 {
                ipv4: "127.0.0.1".to_owned(),
                ipv6: String::new(),
                port: 443,
                kind: TelegramCallServerKindV1::WebRtc {
                    username: TelegramCallSecretTextV1::new(
                        "private-user".to_owned(),
                        MAX_SERVER_CREDENTIAL_BYTES,
                    )
                    .expect("username"),
                    password: TelegramCallSecretTextV1::new(
                        "private-password".to_owned(),
                        MAX_SERVER_CREDENTIAL_BYTES,
                    )
                    .expect("password"),
                    supports_stun: true,
                    supports_turn: true,
                },
            }],
            allow_p2p: true,
            allow_tcp: true,
            call_config: TelegramCallSecretTextV1::new(
                "private-config".to_owned(),
                MAX_READY_TEXT_BYTES,
            )
            .expect("config"),
            custom_parameters: TelegramCallSecretTextV1::new(
                "private-parameters".to_owned(),
                MAX_READY_TEXT_BYTES,
            )
            .expect("parameters"),
            encryption_key: TelegramCallSecretBytesV1::new(
                vec![7; CALL_ENCRYPTION_KEY_BYTES],
                CALL_ENCRYPTION_KEY_BYTES,
            )
            .expect("key"),
            is_outgoing: true,
        };

        plan.validate().expect("valid plan");
        let debug = format!("{plan:?}");
        assert!(!debug.contains("private-user"));
        assert!(!debug.contains("private-password"));
        assert!(!debug.contains("private-config"));
        assert!(!debug.contains("private-parameters"));
        assert!(debug.contains("[REDACTED]"));
    }
}
