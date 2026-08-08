mod backfill;

pub use backfill::*;

pub const PACKAGE: &str = "makosh-telegram-calls-core";
pub const MAX_CALL_ID_BYTES: usize = 256;

use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramProviderCallState {
    Pending,
    ExchangingKeys,
    MediaReady,
    HangingUp,
    Discarded,
    Error,
}

impl TelegramProviderCallState {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::ExchangingKeys => "exchanging_keys",
            Self::MediaReady => "media_ready",
            Self::HangingUp => "hanging_up",
            Self::Discarded => "discarded",
            Self::Error => "error",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "exchanging_keys" => Some(Self::ExchangingKeys),
            "media_ready" => Some(Self::MediaReady),
            "hanging_up" => Some(Self::HangingUp),
            "discarded" => Some(Self::Discarded),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Discarded | Self::Error)
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Pending => 1,
            Self::ExchangingKeys => 2,
            Self::MediaReady => 3,
            Self::HangingUp => 4,
            Self::Discarded | Self::Error => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallMediaState {
    Connecting,
    Active,
    Reconnecting,
    Failed,
}

impl TelegramCallMediaState {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Connecting => "connecting",
            Self::Active => "active",
            Self::Reconnecting => "reconnecting",
            Self::Failed => "failed",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "connecting" => Some(Self::Connecting),
            "active" => Some(Self::Active),
            "reconnecting" => Some(Self::Reconnecting),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallDiscardReason {
    Empty,
    Missed,
    Declined,
    Disconnected,
    HungUp,
}

impl TelegramCallDiscardReason {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Missed => "missed",
            Self::Declined => "declined",
            Self::Disconnected => "disconnected",
            Self::HungUp => "hung_up",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "missed" => Some(Self::Missed),
            "declined" => Some(Self::Declined),
            "disconnected" => Some(Self::Disconnected),
            "hung_up" => Some(Self::HungUp),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallFailureCategory {
    Network,
    NotAvailable,
    Permission,
    Protocol,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallOperationKind {
    InitiateAudio,
    AcceptAudio,
    Decline,
    End,
    SetLocalMute,
}

impl TelegramCallOperationKind {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::InitiateAudio => "initiate_audio",
            Self::AcceptAudio => "accept_audio",
            Self::Decline => "decline",
            Self::End => "end",
            Self::SetLocalMute => "set_local_mute",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "initiate_audio" => Some(Self::InitiateAudio),
            "accept_audio" => Some(Self::AcceptAudio),
            "decline" => Some(Self::Decline),
            "end" => Some(Self::End),
            "set_local_mute" => Some(Self::SetLocalMute),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallOperationState {
    Accepted,
    Dispatching,
    AwaitingProvider,
    Completed,
    Failed,
}

impl TelegramCallOperationState {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Dispatching => "dispatching",
            Self::AwaitingProvider => "awaiting_provider",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            "dispatching" => Some(Self::Dispatching),
            "awaiting_provider" => Some(Self::AwaitingProvider),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TelegramCallCommand {
    InitiateAudio {
        operation_id: String,
        account_id: String,
        call_session_id: String,
        provider_user_id: String,
    },
    AcceptAudio {
        operation_id: String,
        account_id: String,
        call_session_id: String,
    },
    Decline {
        operation_id: String,
        account_id: String,
        call_session_id: String,
    },
    End {
        operation_id: String,
        account_id: String,
        call_session_id: String,
    },
    SetLocalMute {
        operation_id: String,
        account_id: String,
        call_session_id: String,
        muted: bool,
    },
}

impl TelegramCallCommand {
    pub const fn kind(&self) -> TelegramCallOperationKind {
        match self {
            Self::InitiateAudio { .. } => TelegramCallOperationKind::InitiateAudio,
            Self::AcceptAudio { .. } => TelegramCallOperationKind::AcceptAudio,
            Self::Decline { .. } => TelegramCallOperationKind::Decline,
            Self::End { .. } => TelegramCallOperationKind::End,
            Self::SetLocalMute { .. } => TelegramCallOperationKind::SetLocalMute,
        }
    }

    pub fn operation_id(&self) -> &str {
        match self {
            Self::InitiateAudio { operation_id, .. }
            | Self::AcceptAudio { operation_id, .. }
            | Self::Decline { operation_id, .. }
            | Self::End { operation_id, .. }
            | Self::SetLocalMute { operation_id, .. } => operation_id,
        }
    }

    pub fn account_id(&self) -> &str {
        match self {
            Self::InitiateAudio { account_id, .. }
            | Self::AcceptAudio { account_id, .. }
            | Self::Decline { account_id, .. }
            | Self::End { account_id, .. }
            | Self::SetLocalMute { account_id, .. } => account_id,
        }
    }

    pub fn call_session_id(&self) -> &str {
        match self {
            Self::InitiateAudio {
                call_session_id, ..
            }
            | Self::AcceptAudio {
                call_session_id, ..
            }
            | Self::Decline {
                call_session_id, ..
            }
            | Self::End {
                call_session_id, ..
            }
            | Self::SetLocalMute {
                call_session_id, ..
            } => call_session_id,
        }
    }

    pub fn provider_user_id(&self) -> Option<&str> {
        match self {
            Self::InitiateAudio {
                provider_user_id, ..
            } => Some(provider_user_id),
            _ => None,
        }
    }

    pub const fn requested_mute(&self) -> Option<bool> {
        match self {
            Self::SetLocalMute { muted, .. } => Some(*muted),
            _ => None,
        }
    }

    pub fn fingerprint_sha256(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(self.kind().storage_name().as_bytes());
        update_fingerprint_field(&mut digest, self.operation_id());
        update_fingerprint_field(&mut digest, self.account_id());
        update_fingerprint_field(&mut digest, self.call_session_id());
        update_fingerprint_field(&mut digest, self.provider_user_id().unwrap_or_default());
        digest.update([u8::from(self.requested_mute().unwrap_or_default())]);
        digest.finalize().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallOperation {
    pub operation_id: String,
    pub account_id: String,
    pub call_session_id: String,
    pub kind: TelegramCallOperationKind,
    pub state: TelegramCallOperationState,
    pub request_fingerprint_sha256: [u8; 32],
    pub provider_user_id: Option<String>,
    pub requested_mute: Option<bool>,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub tdlib_call_id: Option<i32>,
    pub revision: u64,
    pub accepted_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
    pub completed_at_unix_seconds: Option<u64>,
    pub failure_category: Option<TelegramCallFailureCategory>,
}

impl TelegramCallOperation {
    pub fn accepted(
        command: &TelegramCallCommand,
        runtime_generation: u64,
        grant_epoch: u64,
        accepted_at_unix_seconds: u64,
    ) -> Self {
        Self {
            operation_id: command.operation_id().to_owned(),
            account_id: command.account_id().to_owned(),
            call_session_id: command.call_session_id().to_owned(),
            kind: command.kind(),
            state: TelegramCallOperationState::Accepted,
            request_fingerprint_sha256: command.fingerprint_sha256(),
            provider_user_id: command.provider_user_id().map(str::to_owned),
            requested_mute: command.requested_mute(),
            runtime_generation,
            grant_epoch,
            tdlib_call_id: None,
            revision: 1,
            accepted_at_unix_seconds,
            updated_at_unix_seconds: accepted_at_unix_seconds,
            completed_at_unix_seconds: None,
            failure_category: None,
        }
    }

    pub fn command(&self) -> Option<TelegramCallCommand> {
        let common = (
            self.operation_id.clone(),
            self.account_id.clone(),
            self.call_session_id.clone(),
        );
        match self.kind {
            TelegramCallOperationKind::InitiateAudio => Some(TelegramCallCommand::InitiateAudio {
                operation_id: common.0,
                account_id: common.1,
                call_session_id: common.2,
                provider_user_id: self.provider_user_id.clone()?,
            }),
            TelegramCallOperationKind::AcceptAudio => Some(TelegramCallCommand::AcceptAudio {
                operation_id: common.0,
                account_id: common.1,
                call_session_id: common.2,
            }),
            TelegramCallOperationKind::Decline => Some(TelegramCallCommand::Decline {
                operation_id: common.0,
                account_id: common.1,
                call_session_id: common.2,
            }),
            TelegramCallOperationKind::End => Some(TelegramCallCommand::End {
                operation_id: common.0,
                account_id: common.1,
                call_session_id: common.2,
            }),
            TelegramCallOperationKind::SetLocalMute => Some(TelegramCallCommand::SetLocalMute {
                operation_id: common.0,
                account_id: common.1,
                call_session_id: common.2,
                muted: self.requested_mute?,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallCommandError {
    InvalidRequest(&'static str),
    Conflict(&'static str),
}

pub fn validate_call_command(
    command: &TelegramCallCommand,
    current_call: Option<&TelegramCallSession>,
    own_provider_user_id: Option<&str>,
) -> Result<(), TelegramCallCommandError> {
    validate_command_id("operation_id", command.operation_id())?;
    validate_command_id("account_id", command.account_id())?;
    validate_command_id("call_session_id", command.call_session_id())?;

    match command {
        TelegramCallCommand::InitiateAudio {
            provider_user_id, ..
        } => {
            validate_provider_user_id(provider_user_id)?;
            let own_provider_user_id = own_provider_user_id.ok_or(
                TelegramCallCommandError::InvalidRequest("own_provider_user_id"),
            )?;
            validate_provider_user_id(own_provider_user_id)?;
            if provider_user_id == own_provider_user_id {
                return Err(TelegramCallCommandError::Conflict("provider_user_id"));
            }
            if current_call.is_some() {
                return Err(TelegramCallCommandError::Conflict("active_call"));
            }
        }
        TelegramCallCommand::AcceptAudio { .. } => {
            let call = validated_command_call(command, current_call)?;
            if call.direction != TelegramCallDirection::Incoming
                || call.state != TelegramProviderCallState::Pending
                || !call.pending_received
            {
                return Err(TelegramCallCommandError::Conflict("call_state"));
            }
        }
        TelegramCallCommand::Decline { .. } => {
            let call = validated_command_call(command, current_call)?;
            if call.direction != TelegramCallDirection::Incoming
                || call.state != TelegramProviderCallState::Pending
                || !call.pending_received
            {
                return Err(TelegramCallCommandError::Conflict("call_state"));
            }
        }
        TelegramCallCommand::End { .. } => {
            let call = validated_command_call(command, current_call)?;
            if call.state.is_terminal() {
                return Err(TelegramCallCommandError::Conflict("call_state"));
            }
        }
        TelegramCallCommand::SetLocalMute { .. } => {
            let call = validated_command_call(command, current_call)?;
            if call.state != TelegramProviderCallState::MediaReady {
                return Err(TelegramCallCommandError::Conflict("call_state"));
            }
        }
    }
    Ok(())
}

fn validated_command_call<'a>(
    command: &TelegramCallCommand,
    current_call: Option<&'a TelegramCallSession>,
) -> Result<&'a TelegramCallSession, TelegramCallCommandError> {
    let call = current_call.ok_or(TelegramCallCommandError::Conflict("call_session_id"))?;
    if call.account_id != command.account_id() || call.call_session_id != command.call_session_id()
    {
        return Err(TelegramCallCommandError::Conflict("call_session_id"));
    }
    Ok(call)
}

fn validate_provider_user_id(value: &str) -> Result<(), TelegramCallCommandError> {
    validate_command_id("provider_user_id", value)?;
    if value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .is_none()
    {
        return Err(TelegramCallCommandError::InvalidRequest("provider_user_id"));
    }
    Ok(())
}

fn validate_command_id(field: &'static str, value: &str) -> Result<(), TelegramCallCommandError> {
    if value.trim().is_empty()
        || value.len() > MAX_CALL_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(TelegramCallCommandError::InvalidRequest(field));
    }
    Ok(())
}

fn update_fingerprint_field(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value.as_bytes());
}

impl TelegramCallFailureCategory {
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::NotAvailable => "not_available",
            Self::Permission => "permission",
            Self::Protocol => "protocol",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "network" => Some(Self::Network),
            "not_available" => Some(Self::NotAvailable),
            "permission" => Some(Self::Permission),
            "protocol" => Some(Self::Protocol),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramProviderCallUpdate {
    pub account_id: String,
    pub runtime_generation: u64,
    pub tdlib_call_id: i32,
    pub provider_call_unique_id: Option<i64>,
    pub provider_user_id: String,
    pub direction: TelegramCallDirection,
    pub state: TelegramProviderCallState,
    pub pending_created: bool,
    pub pending_received: bool,
    pub discard_reason: Option<TelegramCallDiscardReason>,
    pub failure_category: Option<TelegramCallFailureCategory>,
    pub observed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallSession {
    pub call_session_id: String,
    pub account_id: String,
    pub runtime_generation: u64,
    pub tdlib_call_id: i32,
    pub provider_call_unique_id: Option<i64>,
    pub provider_user_id: String,
    pub direction: TelegramCallDirection,
    pub state: TelegramProviderCallState,
    pub pending_created: bool,
    pub pending_received: bool,
    pub discard_reason: Option<TelegramCallDiscardReason>,
    pub failure_category: Option<TelegramCallFailureCategory>,
    pub revision: u64,
    pub created_at_unix_seconds: u64,
    pub updated_at_unix_seconds: u64,
    pub ended_at_unix_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallMediaUpdate {
    pub account_id: String,
    pub call_session_id: String,
    pub runtime_generation: u64,
    pub provider_revision: u64,
    pub state: TelegramCallMediaState,
    pub observed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallMediaProjection {
    pub account_id: String,
    pub call_session_id: String,
    pub runtime_generation: u64,
    pub provider_revision: u64,
    pub state: TelegramCallMediaState,
    pub revision: u64,
    pub connected_at_unix_seconds: Option<u64>,
    pub updated_at_unix_seconds: u64,
    pub failed_at_unix_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectedCallMediaUpdate {
    pub projection: TelegramCallMediaProjection,
    pub changed: bool,
}

pub fn project_call_media_update(
    current: Option<&TelegramCallMediaProjection>,
    update: &TelegramCallMediaUpdate,
) -> Result<ProjectedCallMediaUpdate, TelegramCallProjectionError> {
    validate_id("account_id", &update.account_id)?;
    validate_id("call_session_id", &update.call_session_id)?;
    if update.runtime_generation == 0
        || update.provider_revision == 0
        || update.observed_at_unix_seconds == 0
    {
        return Err(TelegramCallProjectionError::InvalidRequest("media_update"));
    }
    let Some(current) = current else {
        if update.state == TelegramCallMediaState::Reconnecting {
            return Err(TelegramCallProjectionError::StateRegression);
        }
        return Ok(ProjectedCallMediaUpdate {
            projection: TelegramCallMediaProjection {
                account_id: update.account_id.clone(),
                call_session_id: update.call_session_id.clone(),
                runtime_generation: update.runtime_generation,
                provider_revision: update.provider_revision,
                state: update.state,
                revision: 1,
                connected_at_unix_seconds: (update.state == TelegramCallMediaState::Active)
                    .then_some(update.observed_at_unix_seconds),
                updated_at_unix_seconds: update.observed_at_unix_seconds,
                failed_at_unix_seconds: (update.state == TelegramCallMediaState::Failed)
                    .then_some(update.observed_at_unix_seconds),
            },
            changed: true,
        });
    };
    if current.account_id != update.account_id
        || current.call_session_id != update.call_session_id
        || update.provider_revision < current.provider_revision
    {
        return Err(TelegramCallProjectionError::IdentityConflict);
    }
    if current.runtime_generation != update.runtime_generation {
        if update.state != TelegramCallMediaState::Connecting {
            return Err(TelegramCallProjectionError::StateRegression);
        }
    } else {
        if current.state == update.state {
            return Ok(ProjectedCallMediaUpdate {
                projection: current.clone(),
                changed: false,
            });
        }
        let valid_transition = matches!(
            (current.state, update.state),
            (
                TelegramCallMediaState::Connecting,
                TelegramCallMediaState::Active | TelegramCallMediaState::Failed
            ) | (
                TelegramCallMediaState::Active,
                TelegramCallMediaState::Reconnecting | TelegramCallMediaState::Failed
            ) | (
                TelegramCallMediaState::Reconnecting,
                TelegramCallMediaState::Active | TelegramCallMediaState::Failed
            )
        );
        if !valid_transition {
            return Err(if current.state == TelegramCallMediaState::Failed {
                TelegramCallProjectionError::TerminalConflict
            } else {
                TelegramCallProjectionError::StateRegression
            });
        }
    }
    Ok(ProjectedCallMediaUpdate {
        projection: TelegramCallMediaProjection {
            account_id: current.account_id.clone(),
            call_session_id: current.call_session_id.clone(),
            runtime_generation: update.runtime_generation,
            provider_revision: update.provider_revision,
            state: update.state,
            revision: current.revision + 1,
            connected_at_unix_seconds: if current.runtime_generation != update.runtime_generation {
                None
            } else {
                current.connected_at_unix_seconds
            }
            .or_else(|| {
                (update.state == TelegramCallMediaState::Active)
                    .then_some(update.observed_at_unix_seconds)
            }),
            updated_at_unix_seconds: update.observed_at_unix_seconds,
            failed_at_unix_seconds: (update.state == TelegramCallMediaState::Failed)
                .then_some(update.observed_at_unix_seconds),
        },
        changed: true,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectedCallUpdate {
    pub session: TelegramCallSession,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallProjectionError {
    InvalidRequest(&'static str),
    IdentityConflict,
    StateRegression,
    TerminalConflict,
}

pub fn project_provider_call_update(
    existing: Option<&TelegramCallSession>,
    new_call_session_id: &str,
    update: &TelegramProviderCallUpdate,
) -> Result<ProjectedCallUpdate, TelegramCallProjectionError> {
    validate_update(new_call_session_id, update)?;

    let Some(current) = existing else {
        return Ok(ProjectedCallUpdate {
            session: TelegramCallSession {
                call_session_id: new_call_session_id.to_owned(),
                account_id: update.account_id.clone(),
                runtime_generation: update.runtime_generation,
                tdlib_call_id: update.tdlib_call_id,
                provider_call_unique_id: update.provider_call_unique_id,
                provider_user_id: update.provider_user_id.clone(),
                direction: update.direction,
                state: update.state,
                pending_created: update.pending_created,
                pending_received: update.pending_received,
                discard_reason: update.discard_reason,
                failure_category: update.failure_category,
                revision: 1,
                created_at_unix_seconds: update.observed_at_unix_seconds,
                updated_at_unix_seconds: update.observed_at_unix_seconds,
                ended_at_unix_seconds: update
                    .state
                    .is_terminal()
                    .then_some(update.observed_at_unix_seconds),
            },
            changed: true,
        });
    };

    validate_identity(current, update)?;
    if update.state.rank() < current.state.rank() {
        return Ok(ProjectedCallUpdate {
            session: current.clone(),
            changed: false,
        });
    }
    validate_transition(current, update)?;

    let provider_call_unique_id = current
        .provider_call_unique_id
        .or(update.provider_call_unique_id);
    let changed = provider_call_unique_id != current.provider_call_unique_id
        || update.state != current.state
        || update.pending_created != current.pending_created
        || update.pending_received != current.pending_received
        || update.discard_reason != current.discard_reason
        || update.failure_category != current.failure_category;

    if !changed {
        return Ok(ProjectedCallUpdate {
            session: current.clone(),
            changed: false,
        });
    }

    Ok(ProjectedCallUpdate {
        session: TelegramCallSession {
            call_session_id: current.call_session_id.clone(),
            account_id: current.account_id.clone(),
            runtime_generation: current.runtime_generation,
            tdlib_call_id: current.tdlib_call_id,
            provider_call_unique_id,
            provider_user_id: current.provider_user_id.clone(),
            direction: current.direction,
            state: update.state,
            pending_created: update.pending_created,
            pending_received: update.pending_received,
            discard_reason: update.discard_reason,
            failure_category: update.failure_category,
            revision: current.revision.saturating_add(1),
            created_at_unix_seconds: current.created_at_unix_seconds,
            updated_at_unix_seconds: update.observed_at_unix_seconds,
            ended_at_unix_seconds: if update.state.is_terminal() {
                current
                    .ended_at_unix_seconds
                    .or(Some(update.observed_at_unix_seconds))
            } else {
                None
            },
        },
        changed: true,
    })
}

fn validate_update(
    new_call_session_id: &str,
    update: &TelegramProviderCallUpdate,
) -> Result<(), TelegramCallProjectionError> {
    validate_id("call_session_id", new_call_session_id)?;
    validate_id("account_id", &update.account_id)?;
    validate_id("provider_user_id", &update.provider_user_id)?;
    if update.runtime_generation == 0 {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "runtime_generation",
        ));
    }
    if update.tdlib_call_id <= 0 {
        return Err(TelegramCallProjectionError::InvalidRequest("tdlib_call_id"));
    }
    if update
        .provider_call_unique_id
        .is_some_and(|value| value <= 0)
    {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "provider_call_unique_id",
        ));
    }
    if update.observed_at_unix_seconds == 0 {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "observed_at_unix_seconds",
        ));
    }
    if update.state != TelegramProviderCallState::Pending
        && (update.pending_created || update.pending_received)
    {
        return Err(TelegramCallProjectionError::InvalidRequest("pending_state"));
    }
    if (update.state == TelegramProviderCallState::Discarded) != update.discard_reason.is_some() {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "discard_reason",
        ));
    }
    if (update.state == TelegramProviderCallState::Error) != update.failure_category.is_some() {
        return Err(TelegramCallProjectionError::InvalidRequest(
            "failure_category",
        ));
    }
    Ok(())
}

fn validate_identity(
    current: &TelegramCallSession,
    update: &TelegramProviderCallUpdate,
) -> Result<(), TelegramCallProjectionError> {
    let same_persistent_call = matches!(
        (
            current.provider_call_unique_id,
            update.provider_call_unique_id
        ),
        (Some(current_id), Some(update_id)) if current_id == update_id
    );
    if current.account_id != update.account_id
        || current.provider_user_id != update.provider_user_id
        || current.direction != update.direction
        || (!same_persistent_call
            && (current.runtime_generation != update.runtime_generation
                || current.tdlib_call_id != update.tdlib_call_id))
    {
        return Err(TelegramCallProjectionError::IdentityConflict);
    }
    if let (Some(current_id), Some(update_id)) = (
        current.provider_call_unique_id,
        update.provider_call_unique_id,
    ) && current_id != update_id
    {
        return Err(TelegramCallProjectionError::IdentityConflict);
    }
    Ok(())
}

fn validate_transition(
    current: &TelegramCallSession,
    update: &TelegramProviderCallUpdate,
) -> Result<(), TelegramCallProjectionError> {
    if current.state.is_terminal() {
        if current.state != update.state
            || current.discard_reason != update.discard_reason
            || current.failure_category != update.failure_category
        {
            return Err(TelegramCallProjectionError::TerminalConflict);
        }
        return Ok(());
    }
    if update.observed_at_unix_seconds < current.updated_at_unix_seconds {
        return Err(TelegramCallProjectionError::StateRegression);
    }
    Ok(())
}

fn validate_id(field: &'static str, value: &str) -> Result<(), TelegramCallProjectionError> {
    if value.trim().is_empty() || value.len() > MAX_CALL_ID_BYTES {
        return Err(TelegramCallProjectionError::InvalidRequest(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(state: TelegramProviderCallState, observed_at: u64) -> TelegramProviderCallUpdate {
        TelegramProviderCallUpdate {
            account_id: "account-1".to_owned(),
            runtime_generation: 7,
            tdlib_call_id: 41,
            provider_call_unique_id: None,
            provider_user_id: "provider-user-9".to_owned(),
            direction: TelegramCallDirection::Incoming,
            state,
            pending_created: state == TelegramProviderCallState::Pending,
            pending_received: false,
            discard_reason: None,
            failure_category: None,
            observed_at_unix_seconds: observed_at,
        }
    }

    #[test]
    fn binds_persistent_provider_identity_without_replacing_local_session() {
        let first = project_provider_call_update(
            None,
            "call-session-1",
            &update(TelegramProviderCallState::Pending, 10),
        )
        .expect("first update");
        let mut bound = update(TelegramProviderCallState::Pending, 11);
        bound.provider_call_unique_id = Some(9001);
        let projected =
            project_provider_call_update(Some(&first.session), "ignored-session", &bound)
                .expect("identity binding");

        assert_eq!(projected.session.call_session_id, "call-session-1");
        assert_eq!(projected.session.provider_call_unique_id, Some(9001));
        assert_eq!(projected.session.revision, 2);
    }

    #[test]
    fn duplicate_update_is_replayed_without_revision_or_timestamp_drift() {
        let first = project_provider_call_update(
            None,
            "call-session-1",
            &update(TelegramProviderCallState::Pending, 10),
        )
        .expect("first update");
        let replay = project_provider_call_update(
            Some(&first.session),
            "ignored-session",
            &update(TelegramProviderCallState::Pending, 10),
        )
        .expect("duplicate update");

        assert!(!replay.changed);
        assert_eq!(replay.session, first.session);
    }

    #[test]
    fn media_projection_is_fenced_restartable_and_terminal() {
        let update = TelegramCallMediaUpdate {
            account_id: "account-1".to_owned(),
            call_session_id: "call-session-1".to_owned(),
            runtime_generation: 1,
            provider_revision: 2,
            state: TelegramCallMediaState::Connecting,
            observed_at_unix_seconds: 20,
        };
        let connecting =
            project_call_media_update(None, &update).expect("connecting media projection");
        let active = project_call_media_update(
            Some(&connecting.projection),
            &TelegramCallMediaUpdate {
                state: TelegramCallMediaState::Active,
                observed_at_unix_seconds: 21,
                ..update.clone()
            },
        )
        .expect("active media projection");
        assert_eq!(active.projection.connected_at_unix_seconds, Some(21));
        assert_eq!(
            project_call_media_update(
                Some(&active.projection),
                &TelegramCallMediaUpdate {
                    runtime_generation: 2,
                    state: TelegramCallMediaState::Active,
                    observed_at_unix_seconds: 22,
                    ..update.clone()
                },
            ),
            Err(TelegramCallProjectionError::StateRegression)
        );
        let restarted = project_call_media_update(
            Some(&active.projection),
            &TelegramCallMediaUpdate {
                runtime_generation: 2,
                state: TelegramCallMediaState::Connecting,
                observed_at_unix_seconds: 23,
                ..update
            },
        )
        .expect("new generation starts a new media session");
        assert_eq!(restarted.projection.connected_at_unix_seconds, None);
    }

    #[test]
    fn persistent_call_identity_survives_runtime_generation_change() {
        let mut first_update = update(TelegramProviderCallState::Discarded, 20);
        first_update.provider_call_unique_id = Some(5001);
        first_update.discard_reason = Some(TelegramCallDiscardReason::Missed);
        let first = project_provider_call_update(None, "call-session-1", &first_update)
            .expect("first call");

        let mut replay_update = first_update.clone();
        replay_update.runtime_generation += 1;
        replay_update.tdlib_call_id += 10;
        let replay =
            project_provider_call_update(Some(&first.session), "ignored-session", &replay_update)
                .expect("cross-generation replay");

        assert!(!replay.changed);
        assert_eq!(replay.session, first.session);
    }

    #[test]
    fn terminal_state_is_immutable_and_rejects_provider_identity_conflict() {
        let mut discarded = update(TelegramProviderCallState::Discarded, 12);
        discarded.discard_reason = Some(TelegramCallDiscardReason::Missed);
        discarded.pending_created = false;
        let terminal =
            project_provider_call_update(None, "call-session-1", &discarded).expect("terminal");
        let mut conflict = discarded.clone();
        conflict.provider_call_unique_id = Some(12);
        let bound =
            project_provider_call_update(Some(&terminal.session), "ignored-session", &conflict)
                .expect("late persistent identity");
        conflict.provider_call_unique_id = Some(13);

        assert_eq!(
            project_provider_call_update(Some(&bound.session), "ignored-session", &conflict),
            Err(TelegramCallProjectionError::IdentityConflict)
        );
    }

    #[test]
    fn stale_state_replay_is_ignored_and_untyped_terminal_details_fail_closed() {
        let ready = project_provider_call_update(
            None,
            "call-session-1",
            &update(TelegramProviderCallState::MediaReady, 12),
        )
        .expect("ready");

        let replay = project_provider_call_update(
            Some(&ready.session),
            "ignored-session",
            &update(TelegramProviderCallState::Pending, 13),
        )
        .expect("stale replay");
        assert!(!replay.changed);
        assert_eq!(replay.session, ready.session);
        assert_eq!(
            project_provider_call_update(
                None,
                "call-session-2",
                &update(TelegramProviderCallState::Discarded, 14),
            ),
            Err(TelegramCallProjectionError::InvalidRequest(
                "discard_reason"
            ))
        );
    }

    #[test]
    fn call_commands_are_typed_fingerprinted_and_state_validated() {
        let initiate = TelegramCallCommand::InitiateAudio {
            operation_id: "operation-1".to_owned(),
            account_id: "account-1".to_owned(),
            call_session_id: "call-session-outgoing".to_owned(),
            provider_user_id: "9001".to_owned(),
        };
        let replay = initiate.clone();
        let conflicting = TelegramCallCommand::InitiateAudio {
            operation_id: "operation-1".to_owned(),
            account_id: "account-1".to_owned(),
            call_session_id: "call-session-outgoing".to_owned(),
            provider_user_id: "9002".to_owned(),
        };

        assert_eq!(validate_call_command(&initiate, None, Some("42")), Ok(()));
        assert_eq!(initiate.fingerprint_sha256(), replay.fingerprint_sha256());
        assert_ne!(
            initiate.fingerprint_sha256(),
            conflicting.fingerprint_sha256()
        );
        assert_eq!(
            validate_call_command(&initiate, None, Some("9001")),
            Err(TelegramCallCommandError::Conflict("provider_user_id"))
        );
    }

    #[test]
    fn accept_decline_end_and_mute_require_exact_call_state() {
        let mut incoming = project_provider_call_update(
            None,
            "call-session-1",
            &TelegramProviderCallUpdate {
                pending_created: false,
                pending_received: true,
                ..update(TelegramProviderCallState::Pending, 10)
            },
        )
        .expect("incoming")
        .session;
        let accept = TelegramCallCommand::AcceptAudio {
            operation_id: "operation-accept".to_owned(),
            account_id: incoming.account_id.clone(),
            call_session_id: incoming.call_session_id.clone(),
        };
        let decline = TelegramCallCommand::Decline {
            operation_id: "operation-decline".to_owned(),
            account_id: incoming.account_id.clone(),
            call_session_id: incoming.call_session_id.clone(),
        };

        assert_eq!(
            validate_call_command(&accept, Some(&incoming), None),
            Ok(())
        );
        assert_eq!(
            validate_call_command(&decline, Some(&incoming), None),
            Ok(())
        );

        incoming.state = TelegramProviderCallState::MediaReady;
        incoming.pending_received = false;
        let mute = TelegramCallCommand::SetLocalMute {
            operation_id: "operation-mute".to_owned(),
            account_id: incoming.account_id.clone(),
            call_session_id: incoming.call_session_id.clone(),
            muted: true,
        };
        let end = TelegramCallCommand::End {
            operation_id: "operation-end".to_owned(),
            account_id: incoming.account_id.clone(),
            call_session_id: incoming.call_session_id.clone(),
        };

        assert_eq!(validate_call_command(&mute, Some(&incoming), None), Ok(()));
        assert_eq!(validate_call_command(&end, Some(&incoming), None), Ok(()));
        assert_eq!(
            validate_call_command(&accept, Some(&incoming), None),
            Err(TelegramCallCommandError::Conflict("call_state"))
        );
    }
}
