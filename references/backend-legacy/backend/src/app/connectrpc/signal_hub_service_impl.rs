use super::*;

impl SignalHubService for SignalHubConnectService {
    async fn list_sources(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListSourcesRequest>,
    ) -> ServiceResult<ListSourcesResponse> {
        let items = self
            .store
            .list_sources()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListSourcesResponse {
            items: items.into_iter().map(proto_source).collect(),
            ..Default::default()
        })
    }

    async fn get_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetSourceRequest>,
    ) -> ServiceResult<GetSourceResponse> {
        let item = self
            .store
            .get_source(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(GetSourceResponse {
            item: Some(proto_source(item)).into(),
            ..Default::default()
        })
    }

    async fn list_capabilities(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListCapabilitiesRequest>,
    ) -> ServiceResult<ListCapabilitiesResponse> {
        let items = self
            .capability_service
            .list_capabilities(req.source_code, req.connection_id)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListCapabilitiesResponse {
            items: items.into_iter().map(proto_capability).collect(),
            ..Default::default()
        })
    }

    async fn list_fixture_sources(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListFixtureSourcesRequest>,
    ) -> ServiceResult<ListFixtureSourcesResponse> {
        let items = self
            .fixture_service
            .list_fixture_sources()
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListFixtureSourcesResponse {
            items: items.into_iter().map(proto_fixture_source).collect(),
            ..Default::default()
        })
    }

    async fn list_connections(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListConnectionsRequest>,
    ) -> ServiceResult<ListConnectionsResponse> {
        let items = self
            .store
            .list_connections()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListConnectionsResponse {
            items: items.into_iter().map(proto_connection).collect(),
            ..Default::default()
        })
    }

    async fn list_profiles(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListProfilesRequest>,
    ) -> ServiceResult<ListProfilesResponse> {
        let items = self
            .profile_service
            .list_profiles()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListProfilesResponse {
            items: items.into_iter().map(proto_profile).collect(),
            ..Default::default()
        })
    }

    async fn apply_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ApplyProfileRequest>,
    ) -> ServiceResult<ApplyProfileResponse> {
        let item = self
            .profile_service
            .apply_profile(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ApplyProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn create_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateProfileRequest>,
    ) -> ServiceResult<CreateProfileResponse> {
        let item = self
            .profile_service
            .create_profile(&SignalProfileCreate {
                code: req.code.to_owned(),
                display_name: req.display_name.to_owned(),
                description: req.description.to_owned(),
                source_policies: req
                    .source_policies
                    .iter()
                    .map(|policy| {
                        Ok(SignalProfilePolicy {
                            scope: SignalPolicyScope::parse(policy.scope).ok_or_else(|| {
                                SignalHubError::InvalidPolicyScope(policy.scope.to_string())
                            })?,
                            source_code: policy.source_code.map(str::to_owned),
                            connection_id: policy.connection_id.map(str::to_owned),
                            event_pattern: policy.event_pattern.map(str::to_owned),
                            mode: SignalPolicyMode::parse(policy.mode).ok_or_else(|| {
                                SignalHubError::InvalidPolicyMode(policy.mode.to_string())
                            })?,
                            reason: policy.reason.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(signal_hub_connect_error)?,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreateProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn update_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateProfileRequest>,
    ) -> ServiceResult<UpdateProfileResponse> {
        let source_policies = if req.update_source_policies {
            Some(
                req.source_policies
                    .iter()
                    .map(|policy| {
                        Ok(SignalProfilePolicy {
                            scope: SignalPolicyScope::parse(policy.scope).ok_or_else(|| {
                                SignalHubError::InvalidPolicyScope(policy.scope.to_string())
                            })?,
                            source_code: policy.source_code.map(str::to_owned),
                            connection_id: policy.connection_id.map(str::to_owned),
                            event_pattern: policy.event_pattern.map(str::to_owned),
                            mode: SignalPolicyMode::parse(policy.mode).ok_or_else(|| {
                                SignalHubError::InvalidPolicyMode(policy.mode.to_string())
                            })?,
                            reason: policy.reason.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(signal_hub_connect_error)?,
            )
        } else {
            None
        };

        let item = self
            .profile_service
            .update_profile(&SignalProfileUpdate {
                code: req.code.to_owned(),
                display_name: req.display_name.map(ToOwned::to_owned),
                description: req.description.map(ToOwned::to_owned),
                source_policies,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UpdateProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn remove_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RemoveProfileRequest>,
    ) -> ServiceResult<RemoveProfileResponse> {
        let item = self
            .profile_service
            .remove_profile(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RemoveProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn create_connection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateConnectionRequest>,
    ) -> ServiceResult<CreateConnectionResponse> {
        let item = self
            .connection_service
            .create_connection(&SignalConnectionCreate {
                source_code: req.source_code.to_owned(),
                display_name: req.display_name.to_owned(),
                status: req.status.to_owned(),
                profile: req.profile.map(ToOwned::to_owned),
                settings: parse_metadata_json(req.settings_json).map_err(invalid_argument_error)?,
                secret_ref: req.secret_ref.map(ToOwned::to_owned),
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreateConnectionResponse {
            item: Some(proto_connection(item)).into(),
            ..Default::default()
        })
    }

    async fn update_connection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateConnectionRequest>,
    ) -> ServiceResult<UpdateConnectionResponse> {
        let item = self
            .connection_service
            .update_connection(&SignalConnectionUpdate {
                id: req.id.to_owned(),
                display_name: req.display_name.map(ToOwned::to_owned),
                status: req.status.map(ToOwned::to_owned),
                profile: req.profile.map(ToOwned::to_owned),
                settings: req
                    .settings_json
                    .map(parse_metadata_json)
                    .transpose()
                    .map_err(invalid_argument_error)?,
                secret_ref: req.secret_ref.map(ToOwned::to_owned),
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UpdateConnectionResponse {
            item: Some(proto_connection(item)).into(),
            ..Default::default()
        })
    }

    async fn remove_connection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RemoveConnectionRequest>,
    ) -> ServiceResult<RemoveConnectionResponse> {
        let item = self
            .connection_service
            .remove_connection(req.id)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RemoveConnectionResponse {
            item: Some(proto_connection(item)).into(),
            ..Default::default()
        })
    }

    async fn enable_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, EnableSourceRequest>,
    ) -> ServiceResult<EnableSourceResponse> {
        let item = self
            .control_service
            .enable_source(req.source_code, req.reason)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(EnableSourceResponse {
            source_code: item.source_code.unwrap_or(req.source_code.to_owned()),
            cleared_count: u32::try_from(item.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn disable_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DisableSourceRequest>,
    ) -> ServiceResult<DisableSourceResponse> {
        let item = self
            .control_service
            .disable_source(req.source_code, req.reason)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(DisableSourceResponse {
            source_code: item.source_code.unwrap_or(req.source_code.to_owned()),
            policy_id: item.policy_id.unwrap_or_default(),
            ..Default::default()
        })
    }

    async fn disable_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DisableSignalsRequest>,
    ) -> ServiceResult<DisableSignalsResponse> {
        let result = self
            .control_service
            .disable_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(DisableSignalsResponse {
            policy_id: result.policy_id,
            ..Default::default()
        })
    }

    async fn enable_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, EnableSignalsRequest>,
    ) -> ServiceResult<EnableSignalsResponse> {
        let result = self
            .control_service
            .enable_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(EnableSignalsResponse {
            cleared_count: u32::try_from(result.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn mute_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MuteSignalsRequest>,
    ) -> ServiceResult<MuteSignalsResponse> {
        let item = self
            .control_service
            .mute_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(MuteSignalsResponse {
            policy_id: item.policy_id,
            ..Default::default()
        })
    }

    async fn unmute_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UnmuteSignalsRequest>,
    ) -> ServiceResult<UnmuteSignalsResponse> {
        let item = self
            .control_service
            .unmute_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UnmuteSignalsResponse {
            cleared_count: u32::try_from(item.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn pause_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, PauseSignalsRequest>,
    ) -> ServiceResult<PauseSignalsResponse> {
        let item = self
            .control_service
            .pause_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(PauseSignalsResponse {
            policy_id: item.policy_id,
            ..Default::default()
        })
    }

    async fn resume_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ResumeSignalsRequest>,
    ) -> ServiceResult<ResumeSignalsResponse> {
        let item = self
            .control_service
            .resume_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ResumeSignalsResponse {
            cleared_count: u32::try_from(item.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn list_health(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListHealthRequest>,
    ) -> ServiceResult<ListHealthResponse> {
        let items = self
            .store
            .list_health()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListHealthResponse {
            items: items.into_iter().map(proto_health).collect(),
            ..Default::default()
        })
    }

    async fn run_health_check(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RunHealthCheckRequest>,
    ) -> ServiceResult<RunHealthCheckResponse> {
        let item = run_signal_hub_health_check(
            &self.config,
            self.store.pool().clone(),
            &DomainSignalHealthCheckRequest {
                source_code: req.source_code.to_owned(),
                connection_id: req.connection_id.map(ToOwned::to_owned),
                runtime_kind: None,
            },
        )
        .await
        .map_err(signal_hub_connect_error)?;
        Response::ok(RunHealthCheckResponse {
            item: Some(proto_health(item)).into(),
            ..Default::default()
        })
    }

    async fn list_runtime_states(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListRuntimeStatesRequest>,
    ) -> ServiceResult<ListRuntimeStatesResponse> {
        let items = self
            .store
            .list_runtime_states()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListRuntimeStatesResponse {
            items: items.into_iter().map(proto_runtime_state).collect(),
            ..Default::default()
        })
    }

    async fn update_runtime_state(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateRuntimeStateRequest>,
    ) -> ServiceResult<UpdateRuntimeStateResponse> {
        let metadata = parse_metadata_json(req.metadata_json).map_err(invalid_argument_error)?;
        let item = self
            .store
            .set_runtime_state(&SignalRuntimeStateUpdate {
                source_code: req.source_code.to_owned(),
                runtime_kind: req.runtime_kind.to_owned(),
                state: req.state.to_owned(),
                metadata,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UpdateRuntimeStateResponse {
            item: Some(proto_runtime_state(item)).into(),
            ..Default::default()
        })
    }

    async fn restore_system_fixture(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, RestoreSystemFixtureRequest>,
    ) -> ServiceResult<RestoreSystemFixtureResponse> {
        let report = self
            .store
            .restore_system_sources()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RestoreSystemFixtureResponse {
            sources_created: u32::try_from(report.sources_created).unwrap_or(u32::MAX),
            sources_repaired: u32::try_from(report.sources_repaired).unwrap_or(u32::MAX),
            profiles_created: u32::try_from(report.profiles_created).unwrap_or(u32::MAX),
            profiles_repaired: u32::try_from(report.profiles_repaired).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn list_policies(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListPoliciesRequest>,
    ) -> ServiceResult<ListPoliciesResponse> {
        let items = RawSignalStore::new(self.store.pool().clone())
            .list_active_policies()
            .await
            .map_err(SignalHubError::from)
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListPoliciesResponse {
            items: items.into_iter().map(proto_policy).collect(),
            ..Default::default()
        })
    }

    async fn create_policy(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreatePolicyRequest>,
    ) -> ServiceResult<CreatePolicyResponse> {
        let policy = SignalPolicy {
            scope: parse_policy_scope(req.scope).map_err(invalid_argument_error)?,
            source_code: req.source_code.map(ToOwned::to_owned),
            connection_id: req.connection_id.map(ToOwned::to_owned),
            event_pattern: req.event_pattern.map(ToOwned::to_owned),
            mode: parse_policy_mode(req.mode).map_err(invalid_argument_error)?,
            reason: non_empty_reason(req.reason),
            expires_at: parse_optional_timestamp(req.expires_at, "expires_at")
                .map_err(invalid_argument_error)?,
        };
        let id = self
            .store
            .create_policy(&policy)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreatePolicyResponse {
            id: id.to_string(),
            ..Default::default()
        })
    }

    async fn list_replay_requests(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListReplayRequestsRequest>,
    ) -> ServiceResult<ListReplayRequestsResponse> {
        let items = self
            .store
            .list_replay_requests()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListReplayRequestsResponse {
            items: items.into_iter().map(proto_replay_request).collect(),
            ..Default::default()
        })
    }

    async fn request_replay(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RequestReplayRequest>,
    ) -> ServiceResult<RequestReplayResponse> {
        let metadata = parse_metadata_json(req.metadata_json).map_err(invalid_argument_error)?;
        let item = self
            .replay_service
            .request_replay(&SignalReplayRequestCreate {
                source_code: req.source_code.map(ToOwned::to_owned),
                connection_id: req.connection_id.map(ToOwned::to_owned),
                event_pattern: req.event_pattern.map(ToOwned::to_owned),
                from_position: req.from_position,
                to_position: req.to_position,
                from_time: parse_optional_timestamp(req.from_time, "from_time")
                    .map_err(invalid_argument_error)?,
                to_time: parse_optional_timestamp(req.to_time, "to_time")
                    .map_err(invalid_argument_error)?,
                target_consumer: req.target_consumer.map(ToOwned::to_owned),
                target_projection: req.target_projection.map(ToOwned::to_owned),
                requested_by: "makosh-frontend".to_owned(),
                metadata,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RequestReplayResponse {
            item: Some(proto_replay_request(item)).into(),
            ..Default::default()
        })
    }

    async fn emit_fixture_signal(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, EmitFixtureSignalRequest>,
    ) -> ServiceResult<EmitFixtureSignalResponse> {
        let item = self
            .fixture_service
            .emit_fixture(&SignalFixtureEmitRequest {
                fixture_id: req.fixture_id.to_owned(),
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(EmitFixtureSignalResponse {
            fixture_id: item.fixture_id,
            raw_event_id: item.raw_event_id,
            event_type: item.event_type,
            source_code: item.source_code,
            correlation_id: item.correlation_id,
            ..Default::default()
        })
    }
}
