use makosh_telegram_call_media_contract::{
    TelegramCallDiscardContextV1, TelegramCallMediaContractError, TelegramCallMediaEventV1,
    TelegramCallMediaFinalV1, TelegramCallMediaStateV1, TelegramCallReadyMaterialV1,
    TelegramCallSecretBytesV1, TelegramCallSignalingMediaPort,
};
use makosh_telegram_calls_core::{
    TelegramCallCommand, TelegramCallFailureCategory, TelegramCallOperation,
    TelegramProviderCallState,
};
use makosh_telegram_calls_persistence::{TelegramCallsPersistence, TelegramCallsPersistenceError};
use makosh_telegram_tdlib::{TdlibError, TdlibRequest, TdlibResponse, TdlibTransport};

use crate::calls_client_port::{TelegramCallsCommandRuntime, TelegramCallsCommandRuntimeError};
use crate::{TelegramRuntime, TelegramRuntimeAdmission};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallExecutionError {
    Persistence(TelegramCallsPersistenceError),
    Admission,
    Provider,
    Media,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelegramActiveCallMediaSession {
    pub account_id: String,
    pub call_session_id: String,
    pub runtime_generation: u64,
    pub provider_revision: u64,
    pub tdlib_call_id: i32,
}

impl<T: TdlibTransport> TelegramRuntime<T> {
    pub fn install_call_media_port(&mut self, port: Box<dyn TelegramCallSignalingMediaPort>) {
        self.call_media = Some(port);
    }

    pub fn has_call_signaling_media(&self) -> bool {
        self.call_media
            .as_ref()
            .is_some_and(|port| port.supported_protocol().is_ok())
    }

    pub fn call_admission(&self) -> Option<&TelegramRuntimeAdmission> {
        self.admission.as_ref()
    }

    pub fn resolve_own_provider_user_id(
        &mut self,
        correlation_id: &str,
    ) -> Result<String, TdlibError> {
        match self.transport.request(TdlibRequest::GetOwnUser {
            correlation_id: correlation_id.to_owned(),
        })? {
            TdlibResponse::OwnUser { provider_user_id } => Ok(provider_user_id),
            _ => Err(TdlibError::Protocol(
                "TDLib getMe returned an unexpected response".to_owned(),
            )),
        }
    }

    pub fn start_call_media_session(
        &mut self,
        call: &makosh_telegram_calls_core::TelegramCallSession,
        material: TelegramCallReadyMaterialV1,
    ) -> Result<(), TelegramCallExecutionError> {
        let port = self
            .call_media
            .as_mut()
            .ok_or(TelegramCallExecutionError::Media)?;
        let protocol = port
            .supported_protocol()
            .map_err(|_| TelegramCallExecutionError::Media)?;
        let plan = material
            .into_plan(call.call_session_id.clone(), &protocol)
            .map_err(|_| TelegramCallExecutionError::Media)?;
        port.start_session(plan)
            .map_err(|_| TelegramCallExecutionError::Media)?;
        self.active_call_media = Some(TelegramActiveCallMediaSession {
            account_id: call.account_id.clone(),
            call_session_id: call.call_session_id.clone(),
            runtime_generation: call.runtime_generation,
            provider_revision: call.revision,
            tdlib_call_id: call.tdlib_call_id,
        });
        Ok(())
    }

    pub fn receive_call_signaling_data(
        &mut self,
        call_session_id: &str,
        data: TelegramCallSecretBytesV1,
    ) -> Result<(), TelegramCallExecutionError> {
        self.call_media
            .as_mut()
            .ok_or(TelegramCallExecutionError::Media)?
            .receive_signaling_data(call_session_id, data)
            .map_err(|_| TelegramCallExecutionError::Media)
    }

    pub fn poll_call_media_event(
        &mut self,
        call_session_id: &str,
        tdlib_call_id: i32,
    ) -> Result<Option<TelegramCallMediaStateV1>, TelegramCallExecutionError> {
        let event = self
            .call_media
            .as_mut()
            .ok_or(TelegramCallExecutionError::Media)?
            .poll_event(call_session_id)
            .map_err(|_| TelegramCallExecutionError::Media)?;
        match event {
            None => Ok(None),
            Some(TelegramCallMediaEventV1::State(state)) => Ok(Some(state)),
            Some(TelegramCallMediaEventV1::OutboundSignaling(data)) => {
                let correlation_id = format!("call-signaling:{call_session_id}");
                match self.transport.request(TdlibRequest::SendCallSignalingData {
                    correlation_id: correlation_id.clone(),
                    tdlib_call_id,
                    data,
                }) {
                    Ok(TdlibResponse::Accepted { operation_id })
                        if operation_id == correlation_id =>
                    {
                        Ok(None)
                    }
                    _ => Err(TelegramCallExecutionError::Provider),
                }
            }
        }
    }

    pub fn stop_call_media_session(
        &mut self,
        call_session_id: &str,
    ) -> Result<Option<TelegramCallMediaFinalV1>, TelegramCallExecutionError> {
        let Some(port) = self.call_media.as_mut() else {
            return Ok(None);
        };
        let result = match port.stop_session(call_session_id) {
            Ok(final_state) => Ok(Some(final_state)),
            Err(TelegramCallMediaContractError::SessionNotFound) => Ok(None),
            Err(_) => Err(TelegramCallExecutionError::Media),
        };
        if result.is_ok()
            && self
                .active_call_media
                .as_ref()
                .is_some_and(|active| active.call_session_id == call_session_id)
        {
            self.active_call_media = None;
        }
        result
    }

    pub(crate) fn take_call_media_for_reconfiguration(
        &mut self,
    ) -> Result<Option<Box<dyn TelegramCallSignalingMediaPort>>, TelegramCallExecutionError> {
        if let Some(active) = self.active_call_media.as_ref() {
            let call_session_id = active.call_session_id.clone();
            self.stop_call_media_session(&call_session_id)?;
        }
        Ok(self.call_media.take())
    }

    pub(crate) fn poll_active_call_media_event(
        &mut self,
    ) -> Result<
        Option<(TelegramActiveCallMediaSession, TelegramCallMediaStateV1)>,
        TelegramCallExecutionError,
    > {
        let Some(active) = self.active_call_media.clone() else {
            return Ok(None);
        };
        self.poll_call_media_event(&active.call_session_id, active.tdlib_call_id)
            .map(|state| state.map(|state| (active, state)))
    }

    pub async fn execute_due_call_operations(
        &mut self,
        persistence: &TelegramCallsPersistence,
        account_id: &str,
        now_unix_seconds: u64,
        limit: u32,
    ) -> Result<Vec<TelegramCallOperation>, TelegramCallExecutionError> {
        let admission = self
            .admission
            .clone()
            .ok_or(TelegramCallExecutionError::Admission)?;
        persistence
            .reconcile_stale_call_operations(
                account_id,
                admission.runtime_generation,
                admission.grant_epoch,
                now_unix_seconds,
            )
            .await
            .map_err(TelegramCallExecutionError::Persistence)?;
        let claimed = persistence
            .claim_accepted_call_operations(
                account_id,
                admission.runtime_generation,
                admission.grant_epoch,
                now_unix_seconds,
                limit,
            )
            .await
            .map_err(TelegramCallExecutionError::Persistence)?;
        let mut results = Vec::with_capacity(claimed.len());

        for operation in claimed {
            let Some(command) = operation.command() else {
                results.push(
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            TelegramCallFailureCategory::Protocol,
                            now_unix_seconds,
                        )
                        .await
                        .map_err(TelegramCallExecutionError::Persistence)?,
                );
                continue;
            };
            if let TelegramCallCommand::SetLocalMute {
                call_session_id,
                muted,
                ..
            } = &command
            {
                let applied = self
                    .call_media
                    .as_mut()
                    .ok_or(TelegramCallMediaContractError::Unavailable)
                    .and_then(|port| port.set_local_mute(call_session_id, *muted));
                let result = if applied.is_ok() {
                    persistence
                        .complete_local_mute_operation(
                            account_id,
                            &operation.operation_id,
                            now_unix_seconds,
                        )
                        .await
                } else {
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            TelegramCallFailureCategory::NotAvailable,
                            now_unix_seconds,
                        )
                        .await
                }
                .map_err(TelegramCallExecutionError::Persistence)?;
                results.push(result);
                continue;
            }

            let request = match self.call_request(persistence, &command).await {
                Ok(request) => request,
                Err(failure_category) => {
                    results.push(
                        persistence
                            .fail_call_operation(
                                account_id,
                                &operation.operation_id,
                                failure_category,
                                now_unix_seconds,
                            )
                            .await
                            .map_err(TelegramCallExecutionError::Persistence)?,
                    );
                    continue;
                }
            };
            let provider_result = self.transport.request(request);
            let saved = match provider_result {
                Ok(TdlibResponse::CallCreated {
                    operation_id,
                    tdlib_call_id,
                }) if operation_id == operation.operation_id => {
                    persistence
                        .mark_call_operation_awaiting_provider(
                            account_id,
                            &operation.operation_id,
                            Some(tdlib_call_id),
                            now_unix_seconds,
                        )
                        .await
                }
                Ok(TdlibResponse::Accepted { operation_id })
                    if operation_id == operation.operation_id =>
                {
                    persistence
                        .mark_call_operation_awaiting_provider(
                            account_id,
                            &operation.operation_id,
                            None,
                            now_unix_seconds,
                        )
                        .await
                }
                Ok(_) => {
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            TelegramCallFailureCategory::Protocol,
                            now_unix_seconds,
                        )
                        .await
                }
                Err(error) => {
                    persistence
                        .fail_call_operation(
                            account_id,
                            &operation.operation_id,
                            provider_failure_category(&error),
                            now_unix_seconds,
                        )
                        .await
                }
            }
            .map_err(TelegramCallExecutionError::Persistence)?;
            results.push(saved);
        }
        Ok(results)
    }

    async fn call_request(
        &self,
        persistence: &TelegramCallsPersistence,
        command: &TelegramCallCommand,
    ) -> Result<TdlibRequest, TelegramCallFailureCategory> {
        match command {
            TelegramCallCommand::InitiateAudio {
                operation_id,
                provider_user_id,
                ..
            } => Ok(TdlibRequest::CreateCall {
                operation_id: operation_id.clone(),
                provider_user_id: provider_user_id.clone(),
                protocol: self.call_protocol()?,
            }),
            TelegramCallCommand::AcceptAudio {
                operation_id,
                account_id,
                call_session_id,
            } => {
                let call = required_call(persistence, account_id, call_session_id).await?;
                Ok(TdlibRequest::AcceptCall {
                    operation_id: operation_id.clone(),
                    tdlib_call_id: call.tdlib_call_id,
                    protocol: self.call_protocol()?,
                })
            }
            TelegramCallCommand::Decline {
                operation_id,
                account_id,
                call_session_id,
            } => {
                let call = required_call(persistence, account_id, call_session_id).await?;
                Ok(discard_request(
                    operation_id,
                    call.tdlib_call_id,
                    TelegramCallDiscardContextV1 {
                        duration_seconds: 0,
                        connection_id: 0,
                    },
                ))
            }
            TelegramCallCommand::End {
                operation_id,
                account_id,
                call_session_id,
            } => {
                let call = required_call(persistence, account_id, call_session_id).await?;
                let context = if call.state == TelegramProviderCallState::MediaReady {
                    self.call_media
                        .as_ref()
                        .ok_or(TelegramCallFailureCategory::NotAvailable)?
                        .discard_context(call_session_id)
                        .map_err(|_| TelegramCallFailureCategory::NotAvailable)?
                } else {
                    TelegramCallDiscardContextV1 {
                        duration_seconds: 0,
                        connection_id: 0,
                    }
                };
                Ok(discard_request(operation_id, call.tdlib_call_id, context))
            }
            TelegramCallCommand::SetLocalMute { .. } => Err(TelegramCallFailureCategory::Protocol),
        }
    }

    fn call_protocol(
        &self,
    ) -> Result<
        makosh_telegram_call_media_contract::TelegramCallProtocolV1,
        TelegramCallFailureCategory,
    > {
        self.call_media
            .as_ref()
            .ok_or(TelegramCallFailureCategory::NotAvailable)?
            .supported_protocol()
            .map_err(|_| TelegramCallFailureCategory::NotAvailable)
    }
}

impl<T: TdlibTransport> TelegramCallsCommandRuntime for TelegramRuntime<T> {
    fn calls_media_available(&self) -> bool {
        self.has_call_signaling_media()
    }

    fn calls_fence(&self) -> Option<(u64, u64)> {
        self.call_admission()
            .map(|admission| (admission.runtime_generation, admission.grant_epoch))
    }

    fn owns_calls_account(&self, account_id: &str) -> bool {
        self.account(account_id).is_some()
    }

    fn resolve_call_owner_provider_identity(
        &mut self,
        correlation_id: &str,
    ) -> Result<String, TelegramCallsCommandRuntimeError> {
        self.resolve_own_provider_user_id(correlation_id)
            .map_err(|_| TelegramCallsCommandRuntimeError)
    }
}

async fn required_call(
    persistence: &TelegramCallsPersistence,
    account_id: &str,
    call_session_id: &str,
) -> Result<makosh_telegram_calls_core::TelegramCallSession, TelegramCallFailureCategory> {
    persistence
        .call(account_id, call_session_id)
        .await
        .map_err(persistence_failure_category)?
        .ok_or(TelegramCallFailureCategory::NotAvailable)
}

fn persistence_failure_category(
    _error: TelegramCallsPersistenceError,
) -> TelegramCallFailureCategory {
    TelegramCallFailureCategory::NotAvailable
}

fn discard_request(
    operation_id: &str,
    tdlib_call_id: i32,
    context: TelegramCallDiscardContextV1,
) -> TdlibRequest {
    TdlibRequest::DiscardCall {
        operation_id: operation_id.to_owned(),
        tdlib_call_id,
        is_disconnected: false,
        duration_seconds: context.duration_seconds,
        connection_id: context.connection_id,
    }
}

fn provider_failure_category(error: &TdlibError) -> TelegramCallFailureCategory {
    match error {
        TdlibError::Transport(_) => TelegramCallFailureCategory::Network,
        TdlibError::Protocol(_) => TelegramCallFailureCategory::Protocol,
        TdlibError::AuthenticationRequired => TelegramCallFailureCategory::Permission,
        TdlibError::RuntimeUnavailable => TelegramCallFailureCategory::NotAvailable,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    use makosh_telegram_call_media_contract::{
        CALL_ENCRYPTION_KEY_BYTES, MAX_READY_TEXT_BYTES, MAX_SIGNALING_DATA_BYTES,
        TelegramCallMediaEventV1, TelegramCallMediaFinalV1, TelegramCallMediaStateV1,
        TelegramCallPeerProtocolV1, TelegramCallProtocolV1, TelegramCallReadyMaterialV1,
        TelegramCallReadyPlanV1, TelegramCallSecretBytesV1, TelegramCallSecretTextV1,
        TelegramCallServerKindV1, TelegramCallServerV1,
    };
    use makosh_telegram_tdlib::TdlibProviderUpdate;

    use super::*;

    #[derive(Default)]
    struct MediaEvidence {
        started: bool,
        stopped: bool,
        inbound: Vec<Vec<u8>>,
    }

    struct FakeMedia {
        evidence: Rc<RefCell<MediaEvidence>>,
        events: VecDeque<TelegramCallMediaEventV1>,
    }

    impl TelegramCallSignalingMediaPort for FakeMedia {
        fn supported_protocol(
            &self,
        ) -> Result<TelegramCallProtocolV1, TelegramCallMediaContractError> {
            TelegramCallProtocolV1::new(true, true, vec!["pinned-tgcalls".to_owned()])
        }

        fn start_session(
            &mut self,
            plan: TelegramCallReadyPlanV1,
        ) -> Result<(), TelegramCallMediaContractError> {
            plan.validate()?;
            assert_eq!(plan.library_version, "pinned-tgcalls");
            self.evidence.borrow_mut().started = true;
            Ok(())
        }

        fn receive_signaling_data(
            &mut self,
            _call_session_id: &str,
            data: TelegramCallSecretBytesV1,
        ) -> Result<(), TelegramCallMediaContractError> {
            self.evidence
                .borrow_mut()
                .inbound
                .push(data.expose().to_vec());
            Ok(())
        }

        fn poll_event(
            &mut self,
            _call_session_id: &str,
        ) -> Result<Option<TelegramCallMediaEventV1>, TelegramCallMediaContractError> {
            Ok(self.events.pop_front())
        }

        fn stop_session(
            &mut self,
            _call_session_id: &str,
        ) -> Result<TelegramCallMediaFinalV1, TelegramCallMediaContractError> {
            self.evidence.borrow_mut().stopped = true;
            Ok(TelegramCallMediaFinalV1 {
                discard_context: TelegramCallDiscardContextV1 {
                    duration_seconds: 2,
                    connection_id: 7,
                },
                failed: false,
            })
        }

        fn discard_context(
            &self,
            _call_session_id: &str,
        ) -> Result<TelegramCallDiscardContextV1, TelegramCallMediaContractError> {
            Ok(TelegramCallDiscardContextV1 {
                duration_seconds: 2,
                connection_id: 7,
            })
        }

        fn set_local_mute(
            &mut self,
            _call_session_id: &str,
            _muted: bool,
        ) -> Result<(), TelegramCallMediaContractError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct SignalingTransport {
        sent: Rc<RefCell<Vec<Vec<u8>>>>,
    }

    impl TdlibTransport for SignalingTransport {
        fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
            match request {
                TdlibRequest::SendCallSignalingData {
                    correlation_id,
                    tdlib_call_id: 41,
                    data,
                } => {
                    self.sent.borrow_mut().push(data.expose().to_vec());
                    Ok(TdlibResponse::Accepted {
                        operation_id: correlation_id,
                    })
                }
                _ => Err(TdlibError::Protocol("unexpected test request".to_owned())),
            }
        }

        fn poll_updates(&mut self) -> Result<Vec<TdlibProviderUpdate>, TdlibError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn runtime_wires_ready_inbound_outbound_state_and_teardown_without_persisting_secrets() {
        let evidence = Rc::new(RefCell::new(MediaEvidence::default()));
        let sent = Rc::new(RefCell::new(Vec::new()));
        let mut runtime = TelegramRuntime::new(SignalingTransport { sent: sent.clone() });
        runtime.install_call_media_port(Box::new(FakeMedia {
            evidence: evidence.clone(),
            events: VecDeque::from([
                TelegramCallMediaEventV1::OutboundSignaling(
                    TelegramCallSecretBytesV1::new(
                        b"outbound-private".to_vec(),
                        MAX_SIGNALING_DATA_BYTES,
                    )
                    .expect("outbound signaling"),
                ),
                TelegramCallMediaEventV1::State(TelegramCallMediaStateV1::Established),
            ]),
        }));
        let call = makosh_telegram_calls_core::TelegramCallSession {
            call_session_id: "call-1".to_owned(),
            account_id: "account-1".to_owned(),
            runtime_generation: 7,
            tdlib_call_id: 41,
            provider_call_unique_id: Some(43),
            provider_user_id: "user-1".to_owned(),
            direction: makosh_telegram_calls_core::TelegramCallDirection::Outgoing,
            state: TelegramProviderCallState::MediaReady,
            pending_created: true,
            pending_received: false,
            discard_reason: None,
            failure_category: None,
            revision: 3,
            created_at_unix_seconds: 100,
            updated_at_unix_seconds: 101,
            ended_at_unix_seconds: None,
        };
        runtime
            .start_call_media_session(
                &call,
                TelegramCallReadyMaterialV1 {
                    peer_protocol: TelegramCallPeerProtocolV1 {
                        udp_p2p: true,
                        udp_reflector: true,
                        min_layer: 65,
                        max_layer: 92,
                        library_versions: vec!["pinned-tgcalls".to_owned()],
                    },
                    servers: vec![TelegramCallServerV1 {
                        ipv4: "127.0.0.1".to_owned(),
                        ipv6: String::new(),
                        port: 443,
                        kind: TelegramCallServerKindV1::TelegramReflector {
                            reflector_id: 1,
                            peer_tag: [2; 16],
                            is_tcp: false,
                        },
                    }],
                    allow_p2p: true,
                    allow_tcp: false,
                    call_config: TelegramCallSecretTextV1::new(
                        "private-config".to_owned(),
                        MAX_READY_TEXT_BYTES,
                    )
                    .expect("config"),
                    custom_parameters: TelegramCallSecretTextV1::new(
                        String::new(),
                        MAX_READY_TEXT_BYTES,
                    )
                    .expect("parameters"),
                    encryption_key: TelegramCallSecretBytesV1::new(
                        vec![3; CALL_ENCRYPTION_KEY_BYTES],
                        CALL_ENCRYPTION_KEY_BYTES,
                    )
                    .expect("encryption key"),
                    is_outgoing: true,
                },
            )
            .expect("start media");
        runtime
            .receive_call_signaling_data(
                "call-1",
                TelegramCallSecretBytesV1::new(
                    b"inbound-private".to_vec(),
                    MAX_SIGNALING_DATA_BYTES,
                )
                .expect("inbound signaling"),
            )
            .expect("receive signaling");
        assert_eq!(runtime.poll_call_media_event("call-1", 41), Ok(None));
        assert_eq!(
            runtime.poll_call_media_event("call-1", 41),
            Ok(Some(TelegramCallMediaStateV1::Established))
        );
        runtime
            .stop_call_media_session("call-1")
            .expect("stop media");
        assert_eq!(runtime.poll_active_call_media_event(), Ok(None));

        let evidence = evidence.borrow();
        assert!(evidence.started);
        assert!(evidence.stopped);
        assert_eq!(evidence.inbound, [b"inbound-private".to_vec()]);
        assert_eq!(*sent.borrow(), [b"outbound-private".to_vec()]);
    }
}
