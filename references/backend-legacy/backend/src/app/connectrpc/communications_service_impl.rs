use super::*;

impl CommunicationsService for CommunicationsConnectService {
    async fn list_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListMessagesRequest>,
    ) -> ServiceResult<ListMessagesResponse> {
        let req = req.to_owned_message();
        let workflow_state = req
            .workflow_state
            .as_deref()
            .map(parse_workflow_state)
            .transpose()?;
        let local_state = req
            .local_state
            .as_deref()
            .map(parse_local_state)
            .transpose()?
            .unwrap_or(LocalMessageState::Active);
        let match_mode = parse_match_mode(req.match_mode.as_deref())?;
        let limit = normalize_limit(req.limit, 100, 500);
        let page = self
            .message_store
            .list_messages_page(ProjectedMessagePageQuery {
                account_id: req.account_id.as_deref(),
                workflow_state,
                is_read: req.is_read,
                channel_kind: req.channel_kind.as_deref(),
                conversation_id: req.conversation_id.as_deref(),
                query: req.query.as_deref(),
                match_mode,
                search: Default::default(),
                local_state,
                cursor: req.cursor.as_deref(),
                limit,
            })
            .await
            .map_err(message_connect_error)?;
        let message_ids = page
            .items
            .iter()
            .map(|summary| summary.message.message_id.clone())
            .collect::<Vec<_>>();
        let read_sync_statuses = self
            .provider_command_store
            .read_sync_statuses(&message_ids)
            .await
            .map_err(|error| ConnectError::new(ErrorCode::Internal, error.to_string()))?;

        Response::ok(ListMessagesResponse {
            items: page
                .items
                .into_iter()
                .map(|summary| {
                    let status = read_sync_statuses
                        .get(&summary.message.message_id)
                        .map(String::as_str)
                        .unwrap_or("synced");
                    super::super::communications_message_proto::summary(summary, status)
                })
                .collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn get_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageRequest>,
    ) -> ServiceResult<GetMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let Some(message) = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
        else {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "communication message was not found",
            ));
        };
        let attachments = self
            .storage_store
            .attachments_for_message(&req.message_id)
            .await
            .map_err(storage_connect_error)?;
        let body_html = crate::app::api_support::communications::rich_body_html_for_message(
            self.pool.clone(),
            &message,
        )
        .await
        .map_err(api_error_connect_error)?;
        let read_sync_status = self
            .provider_command_store
            .read_sync_statuses(std::slice::from_ref(&message.message_id))
            .await
            .map_err(|error| ConnectError::new(ErrorCode::Internal, error.to_string()))?
            .remove(&message.message_id)
            .unwrap_or_else(|| "synced".to_owned());

        Response::ok(GetMessageResponse {
            item: Some(
                super::super::communications_message_proto::message_with_body_html(
                    message,
                    attachments.len() as i64,
                    body_html,
                    &read_sync_status,
                ),
            )
            .into(),
            attachments: attachments
                .into_iter()
                .map(super::super::communications_attachment_proto::from_storage)
                .collect(),
            ..Default::default()
        })
    }

    async fn transition_message_workflow_state(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TransitionMessageWorkflowStateRequest>,
    ) -> ServiceResult<TransitionMessageWorkflowStateResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let new_state = parse_workflow_state(&req.workflow_state)?;
        let actor_id = "makosh-frontend";

        self.audit_log
            .record(&NewApiAuditRecord::message_workflow_state_set(
                actor_id,
                &req.message_id,
            ))
            .await
            .map_err(audit_connect_error)?;

        let result = CommunicationCommandService::new(self.pool.clone())
            .transition_message_workflow_state(&req.message_id, new_state, actor_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(TransitionMessageWorkflowStateResponse {
            message_id: result.updated.message_id,
            workflow_state: result.updated.workflow_state.as_str().to_owned(),
            previous_state: result.previous_state,
            ..Default::default()
        })
    }

    async fn trash_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLocalStateRequest>,
    ) -> ServiceResult<UpdateMessageLocalStateResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .move_message_to_local_trash(&req.message_id, "message_trash", "user_deleted")
            .await
            .map_err(command_connect_error)?;

        Response::ok(UpdateMessageLocalStateResponse {
            message_id: updated.message_id,
            local_state: updated.local_state.as_str().to_owned(),
            provider_deleted: Some(false),
            ..Default::default()
        })
    }

    async fn restore_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLocalStateRequest>,
    ) -> ServiceResult<UpdateMessageLocalStateResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .restore_message_from_local_trash(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(UpdateMessageLocalStateResponse {
            message_id: updated.message_id,
            local_state: updated.local_state.as_str().to_owned(),
            provider_deleted: None,
            ..Default::default()
        })
    }

    async fn mark_message_read(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MarkMessageReadRequest>,
    ) -> ServiceResult<MarkMessageReadResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        // Макошь owns local read state. The provider command is committed with
        // that intent and is executed asynchronously after this response.
        let updated = CommunicationCommandService::new(self.pool.clone())
            .set_message_read_local_with_provider_command(
                &req.message_id,
                true,
                "makosh-local-user",
            )
            .await
            .map_err(command_connect_error)?;

        Response::ok(MarkMessageReadResponse {
            message_id: updated.message_id,
            marked_read: true,
            workflow_state: updated.workflow_state.as_str().to_owned(),
            ..Default::default()
        })
    }

    async fn delete_message_from_provider(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteMessageFromProviderRequest>,
    ) -> ServiceResult<DeleteMessageFromProviderResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let updated = CommunicationCommandService::new(self.pool.clone())
            .move_message_to_local_trash(&req.message_id, "imap_delete_alias", "imap-delete-alias")
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteMessageFromProviderResponse {
            message_id: updated.message_id,
            deleted: true,
            local_state: updated.local_state.as_str().to_owned(),
            provider_deleted: Some(false),
            ..Default::default()
        })
    }

    async fn bulk_message_action(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoBulkMessageActionRequest>,
    ) -> ServiceResult<ProtoBulkMessageActionResponse> {
        let req = req.to_owned_message();
        let action = super::super::communications_request_policy::parse_bulk_action_request(&req)?;
        let outcome = BulkMessageActionStore::new(self.pool.clone())
            .apply(req.message_ids, action)
            .await
            .map_err(bulk_action_connect_error)?;

        Response::ok(
            super::super::communications_result_proto::bulk_message_action_outcome(outcome),
        )
    }

    async fn toggle_message_pin(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MessageToggleRequest>,
    ) -> ServiceResult<ToggleMessagePinResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let pinned = CommunicationCommandService::new(self.pool.clone())
            .toggle_message_pin(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(ToggleMessagePinResponse {
            message_id: req.message_id,
            pinned,
            ..Default::default()
        })
    }

    async fn toggle_message_important(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MessageToggleRequest>,
    ) -> ServiceResult<ToggleMessageImportantResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let important = CommunicationCommandService::new(self.pool.clone())
            .toggle_message_important(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(ToggleMessageImportantResponse {
            message_id: req.message_id,
            important,
            ..Default::default()
        })
    }

    async fn toggle_message_mute(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MessageToggleRequest>,
    ) -> ServiceResult<ToggleMessageMuteResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let muted = CommunicationCommandService::new(self.pool.clone())
            .toggle_message_mute(&req.message_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(ToggleMessageMuteResponse {
            message_id: req.message_id,
            muted,
            ..Default::default()
        })
    }

    async fn snooze_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, SnoozeMessageRequest>,
    ) -> ServiceResult<SnoozeMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let until = super::super::communications_timestamp_policy::parse_timestamp(&req.until)?;
        CommunicationCommandService::new(self.pool.clone())
            .snooze_message(&req.message_id, until)
            .await
            .map_err(command_connect_error)?;

        Response::ok(SnoozeMessageResponse {
            snoozed: true,
            ..Default::default()
        })
    }

    async fn add_message_label(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLabelRequest>,
    ) -> ServiceResult<AddMessageLabelResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if req.label.trim().is_empty() {
            return Err(invalid_argument_error("label must not be empty"));
        }
        CommunicationCommandService::new(self.pool.clone())
            .add_message_label(&req.message_id, &req.label)
            .await
            .map_err(command_connect_error)?;

        Response::ok(AddMessageLabelResponse {
            labeled: true,
            ..Default::default()
        })
    }

    async fn remove_message_label(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateMessageLabelRequest>,
    ) -> ServiceResult<RemoveMessageLabelResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if req.label.trim().is_empty() {
            return Err(invalid_argument_error("label must not be empty"));
        }
        CommunicationCommandService::new(self.pool.clone())
            .remove_message_label(&req.message_id, &req.label)
            .await
            .map_err(command_connect_error)?;

        Response::ok(RemoveMessageLabelResponse {
            removed: true,
            ..Default::default()
        })
    }

    async fn analyze_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, AnalyzeMessageRequest>,
    ) -> ServiceResult<AnalyzeMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }

        let ai_state_store =
            crate::domains::communications::ai_state::CommunicationAiStateStore::new(
                self.pool.clone(),
            );
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;

        let _ = ai_state_store
            .transition(
                &req.message_id,
                crate::domains::communications::ai_state::CommunicationAiStateTransitionRequest {
                    ai_state:
                        crate::domains::communications::ai_state::CommunicationAiState::Processing,
                    review_reason: None,
                    last_error: None,
                },
            )
            .await
            .map_err(ai_state_connect_error)?;

        let heuristic_score =
            crate::workflows::email_intelligence::service::EmailIntelligenceService::heuristic_score(
                &message,
            );
        let heuristic_category =
            crate::workflows::email_intelligence::service::EmailIntelligenceService::heuristic_category(
                &message,
            );
        let summary_contract =
            crate::workflows::email_intelligence::service::EmailIntelligenceService::heuristic_structured_summary(&message);

        self.message_store
            .set_ai_analysis(
                &req.message_id,
                heuristic_category.as_deref(),
                None,
                Some(heuristic_score),
            )
            .await
            .map_err(message_connect_error)?;
        let mut metadata = message.message_metadata.clone();
        metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract)
            .map_err(|_| invalid_argument_error("summary contract serialization failed"))?;
        self.message_store
            .set_message_metadata(&req.message_id, &metadata)
            .await
            .map_err(message_connect_error)?;

        if heuristic_score >= 75 && message.workflow_state.as_str() == "new" {
            let _ = self
                .message_store
                .transition_workflow_state(&req.message_id, WorkflowState::NeedsAction)
                .await;
        }

        let _ = ai_state_store
            .transition(
                &req.message_id,
                crate::domains::communications::ai_state::CommunicationAiStateTransitionRequest {
                    ai_state:
                        crate::domains::communications::ai_state::CommunicationAiState::Processed,
                    review_reason: None,
                    last_error: None,
                },
            )
            .await
            .map_err(ai_state_connect_error)?;

        let updated = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let _ = crate::workflows::review_inbox::refresh_message_knowledge_candidates_into_review(
            &self.pool,
            std::slice::from_ref(&updated),
        )
        .await
        .map_err(|_| invalid_argument_error("message knowledge candidate review sync failed"))?;
        let evidence =
            crate::domains::communications::explain::explain_importance(&updated).reasons;

        Response::ok(AnalyzeMessageResponse {
            message_id: updated.message_id,
            analyzed: true,
            category: updated.ai_category,
            summary: updated.ai_summary,
            summary_contract: Some(
                super::super::communications_summary_proto::summary_contract(summary_contract),
            )
            .into(),
            importance_score: updated.importance_score.map(i32::from),
            workflow_state: updated.workflow_state.as_str().to_owned(),
            source: "local_heuristic".to_owned(),
            confidence: None,
            evidence,
            ..Default::default()
        })
    }

    async fn run_workflow_action(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoWorkflowActionRequest>,
    ) -> ServiceResult<ProtoWorkflowActionResponse> {
        let req = req.to_owned_message();
        let request = super::super::communications_workflow_request_proto::request(req)?;
        let response = execute_workflow_action(&self.pool, "makosh-frontend", request)
            .await
            .map_err(api_error_connect_error)?;
        Response::ok(super::super::communications_workflow_response_proto::response(response))
    }

    async fn get_message_explain(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExplainMessageRequest>,
    ) -> ServiceResult<ExplainMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let context = crate::domains::communications::explain::explain_importance(&message);

        Response::ok(ExplainMessageResponse {
            reasons: context.reasons,
            ..Default::default()
        })
    }

    async fn get_message_smart_cc(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageSmartCcRequest>,
    ) -> ServiceResult<GetMessageSmartCcResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let suggestions = crate::domains::communications::explain::smart_cc_suggestions(&message);

        Response::ok(GetMessageSmartCcResponse {
            suggestions,
            ..Default::default()
        })
    }

    async fn get_message_export(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageExportRequest>,
    ) -> ServiceResult<GetMessageExportResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let format = match req.format.trim() {
            "eml" => crate::domains::communications::export::ExportFormat::Eml,
            "json" => crate::domains::communications::export::ExportFormat::Json,
            "" | "md" | "markdown" => {
                crate::domains::communications::export::ExportFormat::Markdown
            }
            _ => return Err(invalid_argument_error("invalid export format")),
        };
        let export = crate::domains::communications::export::export_message(
            &self.message_store,
            &self.storage_store,
            &req.message_id,
            format,
        )
        .await
        .map_err(super::super::communications_errors::export)?;

        Response::ok(GetMessageExportResponse {
            content_type: export.format.content_type().to_owned(),
            content: export.content,
            filename: format!(
                "message_{}.{}",
                &req.message_id[..8.min(req.message_id.len())],
                export.format.extension()
            ),
            ..Default::default()
        })
    }

    async fn get_message_auth(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageAuthRequest>,
    ) -> ServiceResult<GetMessageAuthResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let auth = match makosh_communications_postgres::store::CommunicationIngestionStore::new(
            self.pool.clone(),
        )
        .raw_record(&message.raw_record_id)
        .await
        .map_err(super::super::communications_errors::raw_evidence)?
        {
            Some(raw) => {
                crate::domains::communications::spf_dkim::parse_auth_headers_from_raw_record(&raw)
                    .await
                    .map_err(message_connect_error)?
            }
            None => crate::domains::communications::spf_dkim::AuthResults::default(),
        };
        let risk = crate::domains::communications::spf_dkim::assess_auth_risk(&auth);

        Response::ok(GetMessageAuthResponse {
            auth: Some(super::super::communications_auth_proto::report(auth)).into(),
            risk: Some(super::super::communications_auth_proto::risk_report(risk)).into(),
            ..Default::default()
        })
    }

    async fn get_message_signature(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMessageSignatureRequest>,
    ) -> ServiceResult<GetMessageSignatureResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let detection =
            crate::domains::communications::signatures::detector::SignatureDetector::detect_in_message(
                &message.body_text,
                "",
            );

        Response::ok(GetMessageSignatureResponse {
            has_signature: detection.has_signature,
            signature_type: detection
                .signature_type
                .map(|value| value.as_str().to_owned()),
            signer_info: detection.signer_info,
            is_valid: detection.is_valid,
            cert_expiry_warning: detection.cert_expiry_warning,
            ..Default::default()
        })
    }

    async fn generate_ai_reply(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoAiReplyRequest>,
    ) -> ServiceResult<ProtoAiReplyResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        super::super::communications_ai_runtime::require_mail_content_egress(
            &self.pool,
            &self.config,
            &message.account_id,
            crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
        )
        .await?;
        let runtime =
            super::super::communications_ai_runtime::hub_optional(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?;
        let service = crate::domains::communications::ai_reply::AiReplyService::new(runtime);
        let opts = crate::domains::communications::ai_reply::AiReplyOptions {
            tone: req.tone,
            language: req.language,
            context: req.context,
        };

        match service.generate_reply(&message, &opts).await {
            Ok(Some(draft)) => {
                crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                    self.pool.clone(),
                    "reply_drafting",
                    &req.message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": req.message_id,
                        "operation": "reply_drafting",
                    }),
                    serde_json::json!({
                        "tone": draft.tone,
                        "language": draft.language,
                    }),
                    serde_json::json!({
                        "source": "communication_message_ai_reply",
                        "message_id": req.message_id,
                    }),
                    None,
                )
                .await;

                Response::ok(ProtoAiReplyResponse {
                    subject: Some(draft.subject),
                    body: Some(draft.body),
                    tone: Some(draft.tone),
                    language: Some(draft.language),
                    generated: Some(true),
                    reason: None,
                    ..Default::default()
                })
            }
            Ok(None) => Response::ok(ProtoAiReplyResponse {
                generated: Some(false),
                reason: Some("no LLM configured".to_owned()),
                ..Default::default()
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %req.message_id,
                    "ai reply generation failed"
                );
                Response::ok(ProtoAiReplyResponse {
                    generated: Some(false),
                    reason: Some("ai reply runtime unavailable".to_owned()),
                    ..Default::default()
                })
            }
        }
    }

    async fn generate_ai_reply_variants(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ProtoAiReplyVariantsRequest>,
    ) -> ServiceResult<ProtoAiReplyVariantsResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        super::super::communications_ai_runtime::require_mail_content_egress(
            &self.pool,
            &self.config,
            &message.account_id,
            crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
        )
        .await?;
        let runtime =
            super::super::communications_ai_runtime::hub_optional(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?;
        let service = crate::domains::communications::ai_reply::AiReplyService::new(runtime);
        let languages = if req.languages.is_empty() {
            vec!["en".to_owned(), "es".to_owned(), "ru".to_owned()]
        } else {
            req.languages
        };
        let tones = if req.tones.is_empty() {
            vec!["professional".to_owned(), "friendly".to_owned()]
        } else {
            req.tones
        };
        let variants = match service
            .generate_reply_variants(&message, &languages, &tones)
            .await
        {
            Ok(variants) => variants,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %req.message_id,
                    "ai reply variants generation failed"
                );
                Vec::new()
            }
        };

        if !variants.is_empty() {
            crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                self.pool.clone(),
                "reply_variant_generation",
                &req.message_id,
                serde_json::json!({
                    "kind": "communication_message",
                    "source_code": "ai",
                    "message_id": req.message_id,
                    "operation": "reply_variant_generation",
                }),
                serde_json::json!({
                    "variant_count": variants.len(),
                    "language_count": languages.len(),
                    "tone_count": tones.len(),
                }),
                serde_json::json!({
                    "source": "communication_message_ai_reply_variants",
                    "message_id": req.message_id,
                }),
                None,
            )
            .await;
        }

        Response::ok(ProtoAiReplyVariantsResponse {
            variants: variants
                .into_iter()
                .map(|draft| ProtoAiReplyResponse {
                    subject: Some(draft.subject),
                    body: Some(draft.body),
                    tone: Some(draft.tone),
                    language: Some(draft.language),
                    generated: Some(true),
                    reason: None,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn list_message_workflow_state_counts(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListMessageWorkflowStateCountsRequest>,
    ) -> ServiceResult<ListMessageWorkflowStateCountsResponse> {
        let req = req.to_owned_message();
        let local_state = req
            .local_state
            .as_deref()
            .unwrap_or("active")
            .parse::<LocalMessageState>()
            .map_err(|_| invalid_argument_error("invalid local_state value"))?;
        let counts = self
            .message_store
            .count_messages_by_state_with_local_state(req.account_id.as_deref(), local_state)
            .await
            .map_err(message_connect_error)?;

        Response::ok(ListMessageWorkflowStateCountsResponse {
            counts: counts
                .into_iter()
                .map(super::super::communications_result_proto::workflow_state_count)
                .collect(),
            ..Default::default()
        })
    }

    async fn list_subscriptions(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListSubscriptionsRequest>,
    ) -> ServiceResult<ListSubscriptionsResponse> {
        let req = req.to_owned_message();
        let page = self
            .subscription_store
            .detect_subscriptions_page(
                req.account_id.as_deref(),
                normalize_limit(req.limit, 50, 100),
                req.cursor.as_deref(),
            )
            .await
            .map_err(subscription_connect_error)?;

        Response::ok(ListSubscriptionsResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_analytics_proto::subscription_source)
                .collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn get_mailbox_health(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetMailboxHealthRequest>,
    ) -> ServiceResult<GetMailboxHealthResponse> {
        let req = req.to_owned_message();
        let item = self
            .analytics_store
            .mailbox_health(req.account_id.as_deref())
            .await
            .map_err(analytics_connect_error)?;

        Response::ok(GetMailboxHealthResponse {
            item: Some(super::super::communications_analytics_proto::mailbox_health(item)).into(),
            ..Default::default()
        })
    }

    async fn list_top_senders(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListTopSendersRequest>,
    ) -> ServiceResult<ListTopSendersResponse> {
        let req = req.to_owned_message();
        let page = self
            .analytics_store
            .top_senders_page(
                req.account_id.as_deref(),
                normalize_limit(req.limit, 20, 50),
                req.cursor.as_deref(),
            )
            .await
            .map_err(analytics_connect_error)?;

        Response::ok(ListTopSendersResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_analytics_proto::sender_stats)
                .collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn list_communication_blockers(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListCommunicationBlockersRequest>,
    ) -> ServiceResult<ListCommunicationBlockersResponse> {
        Response::ok(ListCommunicationBlockersResponse {
            items: crate::domains::communications::blockers::list_blockers()
                .into_iter()
                .map(super::super::communications_blocker_proto::from_domain)
                .collect(),
            ..Default::default()
        })
    }

    async fn list_communication_personas(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListCommunicationPersonasRequest>,
    ) -> ServiceResult<ListCommunicationPersonasResponse> {
        let items = self
            .persona_store
            .list()
            .await
            .map_err(persona_connect_error)?;
        Response::ok(ListCommunicationPersonasResponse {
            items: items
                .into_iter()
                .map(super::super::communications_persona_proto::from_domain)
                .collect(),
            ..Default::default()
        })
    }

    async fn list_rich_templates(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListRichTemplatesRequest>,
    ) -> ServiceResult<ListRichTemplatesResponse> {
        let templates = self
            .template_store
            .list()
            .await
            .map_err(template_connect_error)?;
        Response::ok(ListRichTemplatesResponse {
            templates: templates
                .into_iter()
                .map(super::super::communications_template_proto::rich_template)
                .collect(),
            ..Default::default()
        })
    }

    async fn upsert_rich_template(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpsertRichTemplateRequest>,
    ) -> ServiceResult<UpsertRichTemplateResponse> {
        let req = req.to_owned_message();
        let template = self
            .template_store
            .upsert(&NewCommunicationTemplate {
                template_id: req
                    .template_id
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| {
                        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
                        format!("mail_template:{timestamp}")
                    }),
                name: req.name,
                subject_template: req.subject_template,
                body_template: req.body_template,
                variables: req.variables,
                language: req.language,
            })
            .await
            .map_err(template_connect_error)?;
        Response::ok(UpsertRichTemplateResponse {
            saved: true,
            template: Some(super::super::communications_template_proto::rich_template(
                template,
            ))
            .into(),
            ..Default::default()
        })
    }

    async fn delete_rich_template(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteRichTemplateRequest>,
    ) -> ServiceResult<DeleteRichTemplateResponse> {
        let req = req.to_owned_message();
        let template_id = req.template_id.trim();
        if template_id.is_empty() {
            return Err(invalid_argument_error("template_id is required"));
        }
        let deleted = self
            .template_store
            .delete(template_id)
            .await
            .map_err(template_connect_error)?;
        if !deleted {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "rich template was not found",
            ));
        }
        Response::ok(DeleteRichTemplateResponse {
            template_id: template_id.to_owned(),
            deleted: true,
            ..Default::default()
        })
    }

    async fn render_rich_template(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RichTemplateRenderRequest>,
    ) -> ServiceResult<RichTemplateRenderResponse> {
        let req = req.to_owned_message();
        let template_id = req.template_id.trim();
        if template_id.is_empty() {
            return Err(invalid_argument_error("template_id is required"));
        }
        let Some(template) = self
            .template_store
            .get(template_id)
            .await
            .map_err(template_connect_error)?
        else {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "rich template was not found",
            ));
        };
        let rendered = self
            .template_store
            .render(&template, &req.variables)
            .map_err(template_connect_error)?;
        Response::ok(RichTemplateRenderResponse {
            template_id: template.template_id,
            variables: req.variables,
            rendered: Some(
                super::super::communications_template_proto::rendered_rich_template(rendered),
            )
            .into(),
            ..Default::default()
        })
    }

    async fn preview_rich_template_mail_merge(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RichTemplateMailMergePreviewRequest>,
    ) -> ServiceResult<RichTemplateMailMergePreviewResponse> {
        let req = req.to_owned_message();
        let template_id = req.template_id.trim();
        if template_id.is_empty() {
            return Err(invalid_argument_error("template_id is required"));
        }
        if req.rows.is_empty() {
            return Err(invalid_argument_error(
                "mail merge preview rows are required",
            ));
        }
        let Some(template) = self
            .template_store
            .get(template_id)
            .await
            .map_err(template_connect_error)?
        else {
            return Err(ConnectError::new(
                ErrorCode::NotFound,
                "rich template was not found",
            ));
        };
        let rows = req
            .rows
            .into_iter()
            .map(|row| {
                let row_id = row.row_id.trim().to_owned();
                if row_id.is_empty() {
                    return Err(invalid_argument_error("row_id is required"));
                }
                Ok(CommunicationMergePreviewRow {
                    row_id,
                    variables: row.variables,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let preview = self
            .template_store
            .render_mail_merge_preview(&template, rows)
            .map_err(template_connect_error)?;
        Response::ok(
            super::super::communications_template_proto::rich_template_mail_merge_preview(preview),
        )
    }

    async fn search_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, SearchMessagesRequest>,
    ) -> ServiceResult<SearchMessagesResponse> {
        let req = req.to_owned_message();
        if req.query.trim().is_empty() {
            return Err(invalid_argument_error("query must not be empty"));
        }

        let Some(path) = std::env::var("MAKOSH_SEARCH_INDEX_PATH").ok() else {
            return Response::ok(SearchMessagesResponse {
                results: Vec::new(),
                ..Default::default()
            });
        };

        let index = crate::engines::search::engine::SearchIndex::open_or_create(
            std::path::Path::new(&path),
        )
        .map_err(search_engine_connect_error)?;
        let limit = normalize_limit(req.limit, 20, 100) as usize;
        let _ = crate::domains::communications::search::index_messages(
            &index,
            &self.message_store,
            100,
        )
        .await
        .map_err(search_connect_error)?;
        let results =
            crate::domains::communications::search::search_emails(&index, &req.query, limit)
                .map_err(search_connect_error)?;

        Response::ok(SearchMessagesResponse {
            results: results
                .into_iter()
                .map(super::super::communications_result_proto::search_result)
                .collect(),
            ..Default::default()
        })
    }

    async fn detect_message_language(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DetectMessageLanguageRequest>,
    ) -> ServiceResult<DetectMessageLanguageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );

        Response::ok(DetectMessageLanguageResponse {
            language: detection.language,
            confidence: detection.confidence,
            script: detection.script,
            ..Default::default()
        })
    }

    async fn translate_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TranslateMessageRequest>,
    ) -> ServiceResult<TranslateMessageResponse> {
        let req = req.to_owned_message();
        let message_id = req.message_id.trim();
        let target_language = req.target_language.trim();
        if message_id.is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if target_language.is_empty() {
            return Err(invalid_argument_error("target_language is required"));
        }

        let message = self
            .message_store
            .message(message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        super::super::communications_ai_runtime::require_mail_content_egress(
            &self.pool,
            &self.config,
            &message.account_id,
            crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
        )
        .await?;
        let service =
            super::super::communications_ai_runtime::multilingual_service(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?;
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );

        match service.translate(&message.body_text, target_language).await {
            Ok(Some(translation)) => {
                crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                    self.pool.clone(),
                    "message_translation",
                    message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": message_id,
                        "operation": "translation",
                    }),
                    serde_json::json!({
                        "target_language": translation.target_language,
                        "original_language": detection.language,
                        "model": translation.model,
                    }),
                    serde_json::json!({
                        "source": "communication_message_translation",
                        "message_id": message_id,
                    }),
                    None,
                )
                .await;

                Response::ok(TranslateMessageResponse {
                    translated: true,
                    text: Some(translation.translated_text),
                    target: Some(translation.target_language),
                    model: Some(translation.model),
                    reason: None,
                    ..Default::default()
                })
            }
            Ok(None) => Response::ok(TranslateMessageResponse {
                translated: false,
                text: None,
                target: Some(target_language.to_owned()),
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
                ..Default::default()
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    message_id = %message.message_id,
                    "message translation failed"
                );
                Response::ok(TranslateMessageResponse {
                    translated: false,
                    text: None,
                    target: Some(target_language.to_owned()),
                    model: None,
                    reason: Some("translation runtime unavailable".to_owned()),
                    ..Default::default()
                })
            }
        }
    }

    async fn extract_message_tasks(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExtractMessageTasksRequest>,
    ) -> ServiceResult<ExtractMessageTasksResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        super::super::communications_ai_runtime::require_mail_content_egress(
            &self.pool,
            &self.config,
            &message.account_id,
            crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
        )
        .await?;
        let service = crate::domains::communications::extract::EmailExtractService::new(
            super::super::communications_ai_runtime::hub_optional(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?,
        );
        let tasks = service
            .extract_tasks(&message)
            .await
            .map_err(extract_connect_error)?;
        let external_llm_task_count = tasks
            .iter()
            .filter(|task| task.source == "ai_hub.external_llm")
            .count();
        if external_llm_task_count > 0 {
            crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                self.pool.clone(),
                "message_task_extraction",
                &req.message_id,
                serde_json::json!({
                    "kind": "communication_message",
                    "source_code": "ai",
                    "message_id": req.message_id,
                    "operation": "task_extraction",
                }),
                serde_json::json!({
                    "task_count": tasks.len(),
                    "external_llm_task_count": external_llm_task_count,
                }),
                serde_json::json!({
                    "source": "communication_message_task_extraction",
                    "message_id": req.message_id,
                }),
                None,
            )
            .await;
        }

        Response::ok(ExtractMessageTasksResponse {
            tasks: tasks
                .into_iter()
                .map(super::super::communications_extraction_proto::task)
                .collect(),
            ..Default::default()
        })
    }

    async fn extract_message_notes(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExtractMessageNotesRequest>,
    ) -> ServiceResult<ExtractMessageNotesResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let service = crate::domains::communications::extract::EmailExtractService::new(None);
        let notes = service
            .extract_notes(&message)
            .await
            .map_err(extract_connect_error)?;

        Response::ok(ExtractMessageNotesResponse {
            notes: notes
                .into_iter()
                .map(super::super::communications_extraction_proto::note)
                .collect(),
            ..Default::default()
        })
    }

    async fn list_threads(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListThreadsRequest>,
    ) -> ServiceResult<ListThreadsResponse> {
        let req = req.to_owned_message();
        let page = self
            .thread_store
            .list_threads_page(
                req.account_id.as_deref(),
                req.cursor.as_deref(),
                normalize_limit(req.limit, 50, 100),
            )
            .await
            .map_err(thread_connect_error)?;
        Response::ok(ListThreadsResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_thread_proto::thread)
                .collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            ..Default::default()
        })
    }

    async fn list_thread_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListThreadMessagesRequest>,
    ) -> ServiceResult<ListThreadMessagesResponse> {
        let req = req.to_owned_message();
        let account_id = req.account_id.as_deref().ok_or_else(|| {
            invalid_argument_error("account_id is required for thread message lookup")
        })?;
        if req.subject.trim().is_empty() {
            return Err(invalid_argument_error("subject must not be empty"));
        }
        let items = self
            .thread_store
            .thread_messages(
                account_id,
                &req.subject,
                normalize_limit(req.limit, 50, 100),
            )
            .await
            .map_err(thread_connect_error)?;
        Response::ok(ListThreadMessagesResponse {
            items: items
                .into_iter()
                .map(super::super::communications_thread_proto::message)
                .collect(),
            ..Default::default()
        })
    }

    async fn translate_thread(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TranslateThreadRequest>,
    ) -> ServiceResult<TranslateThreadResponse> {
        let req = req.to_owned_message();
        let account_id = req.account_id.trim();
        let subject = req.subject.trim();
        let target_language = req.target_language.trim();
        if account_id.is_empty() {
            return Err(invalid_argument_error("account_id is required"));
        }
        if subject.is_empty() {
            return Err(invalid_argument_error("subject is required"));
        }
        if target_language.is_empty() {
            return Err(invalid_argument_error("target_language is required"));
        }

        let messages = self
            .thread_store
            .thread_messages(account_id, subject, normalize_limit(req.limit, 50, 100))
            .await
            .map_err(thread_connect_error)?;
        super::super::communications_ai_runtime::require_mail_content_egress(
            &self.pool,
            &self.config,
            account_id,
            crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::Body,
        )
        .await?;
        let service =
            super::super::communications_ai_runtime::multilingual_service(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?;
        let mut items = Vec::with_capacity(messages.len());

        for message in messages {
            let detection =
                crate::domains::communications::multilingual::MultilingualService::detect_language(
                    &message.body_text,
                );
            match service.translate(&message.body_text, target_language).await {
                Ok(Some(translation)) => {
                    crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                        self.pool.clone(),
                        "thread_message_translation",
                        &message.message_id,
                        serde_json::json!({
                            "kind": "communication_message",
                            "source_code": "ai",
                            "message_id": message.message_id,
                            "operation": "thread_message_translation",
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        serde_json::json!({
                            "target_language": translation.target_language,
                            "original_language": detection.language,
                            "model": translation.model,
                        }),
                        serde_json::json!({
                            "source": "communication_thread_message_translation",
                            "message_id": message.message_id,
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        None,
                    )
                    .await;

                    items.push(ProtoThreadTranslationItem {
                        message_id: message.message_id,
                        original_language: detection.language,
                        confidence: detection.confidence,
                        translated: true,
                        text: Some(translation.translated_text),
                        target: translation.target_language,
                        model: Some(translation.model),
                        reason: None,
                        ..Default::default()
                    });
                }
                Ok(None) => items.push(ProtoThreadTranslationItem {
                    message_id: message.message_id,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: false,
                    text: None,
                    target: target_language.to_owned(),
                    model: None,
                    reason: Some("translation runtime unavailable".to_owned()),
                    ..Default::default()
                }),
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        message_id = %message.message_id,
                        "thread message translation failed"
                    );
                    items.push(ProtoThreadTranslationItem {
                        message_id: message.message_id,
                        original_language: detection.language,
                        confidence: detection.confidence,
                        translated: false,
                        text: None,
                        target: target_language.to_owned(),
                        model: None,
                        reason: Some("translation runtime unavailable".to_owned()),
                        ..Default::default()
                    });
                }
            }
        }

        Response::ok(TranslateThreadResponse {
            account_id: account_id.to_owned(),
            subject: subject.to_owned(),
            target_language: target_language.to_owned(),
            items,
            ..Default::default()
        })
    }

    async fn list_drafts(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListDraftsRequest>,
    ) -> ServiceResult<ListDraftsResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let status = req.status.as_deref().map(parse_draft_status).transpose()?;
        let page = self
            .draft_store
            .list_page(
                req.account_id.as_deref(),
                status,
                page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                normalize_limit(page_request.map(|page| page.limit).unwrap_or(100), 100, 100),
            )
            .await
            .map_err(draft_connect_error)?;
        Response::ok(ListDraftsResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_draft_proto::draft)
                .collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn list_saved_searches(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListSavedSearchesRequest>,
    ) -> ServiceResult<ListSavedSearchesResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let page = self
            .saved_search_store
            .list(CommunicationSavedSearchListQuery {
                account_id: req.account_id.as_deref(),
                is_smart_folder: req.smart_folder,
                cursor: page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                limit: normalize_limit(
                    page_request.map(|page| page.limit).unwrap_or(500),
                    500,
                    1000,
                ),
            })
            .await
            .map_err(saved_search_connect_error)?;
        Response::ok(ListSavedSearchesResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_saved_folder_proto::saved_search)
                .collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn list_folders(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListFoldersRequest>,
    ) -> ServiceResult<ListFoldersResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let page = self
            .folder_store
            .list(CommunicationFolderListQuery {
                account_id: req.account_id.as_deref(),
                cursor: page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                limit: normalize_limit(
                    page_request.map(|page| page.limit).unwrap_or(500),
                    500,
                    1000,
                ),
            })
            .await
            .map_err(folder_connect_error)?;
        Response::ok(ListFoldersResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_saved_folder_proto::folder)
                .collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn list_outbox(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListOutboxRequest>,
    ) -> ServiceResult<ListOutboxResponse> {
        let req = req.to_owned_message();
        let page_request = req.page.as_option();
        let status = req.status.as_deref().map(parse_outbox_status).transpose()?;
        let page = self
            .outbox_store
            .list_page(
                req.account_id.as_deref(),
                status,
                page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                normalize_limit(page_request.map(|page| page.limit).unwrap_or(100), 100, 100),
            )
            .await
            .map_err(outbox_connect_error)?;
        Response::ok(ListOutboxResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_outbox_proto::item)
                .collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn create_draft(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateDraftRequest>,
    ) -> ServiceResult<CreateDraftResponse> {
        let req = req.to_owned_message();
        let metadata = super::super::communications_json_policy::parse_json_object(
            req.metadata_json.as_str(),
            "metadata_json",
        )?;
        let scheduled_send_at = req
            .scheduled_send_at
            .as_deref()
            .map(super::super::communications_timestamp_policy::parse_timestamp)
            .transpose()?;
        let draft = CommunicationCommandService::new(self.pool.clone())
            .upsert_draft(CommunicationDraftUpsertCommand {
                draft_id: req.draft_id,
                account_id: req.account_id,
                persona_id: req.persona_id,
                to_recipients: req.to_recipients,
                cc_recipients: Some(req.cc_recipients),
                bcc_recipients: Some(req.bcc_recipients),
                subject: req.subject,
                body_text: req.body_text,
                body_html: req.body_html,
                in_reply_to: req.in_reply_to,
                references: Some(req.references),
                attachment_ids: req.replace_attachments.then_some(req.attachment_ids),
                status: req.status,
                scheduled_send_at,
                metadata: Some(metadata),
            })
            .await
            .map_err(command_connect_error)?;

        Response::ok(CreateDraftResponse {
            item: Some(super::super::communications_draft_proto::draft(draft)).into(),
            ..Default::default()
        })
    }

    async fn delete_draft(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteDraftRequest>,
    ) -> ServiceResult<DeleteDraftResponse> {
        let req = req.to_owned_message();
        if req.draft_id.trim().is_empty() {
            return Err(invalid_argument_error("draft_id must not be empty"));
        }
        let deleted = CommunicationCommandService::new(self.pool.clone())
            .delete_draft(&req.draft_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteDraftResponse {
            deleted,
            ..Default::default()
        })
    }

    async fn create_saved_search(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateSavedSearchRequest>,
    ) -> ServiceResult<CreateSavedSearchResponse> {
        let req = req.to_owned_message();
        let saved_search = CommunicationCommandService::new(self.pool.clone())
            .create_saved_search(NewCommunicationSavedSearch {
                saved_search_id: req.saved_search_id,
                name: req.name,
                description: req.description,
                account_id: req.account_id,
                query: req.query,
                workflow_state: req
                    .workflow_state
                    .as_deref()
                    .map(parse_workflow_state)
                    .transpose()?,
                local_state: req
                    .local_state
                    .as_deref()
                    .map(parse_local_state)
                    .transpose()?,
                channel_kind: req.channel_kind,
                is_smart_folder: req.is_smart_folder,
                sort_order: req.sort_order,
            })
            .await
            .map_err(command_connect_error)?;

        Response::ok(CreateSavedSearchResponse {
            item: Some(super::super::communications_saved_folder_proto::saved_search(saved_search))
                .into(),
            ..Default::default()
        })
    }

    async fn update_saved_search(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateSavedSearchRequest>,
    ) -> ServiceResult<UpdateSavedSearchResponse> {
        let req = req.to_owned_message();
        if req.saved_search_id.trim().is_empty() {
            return Err(invalid_argument_error("saved_search_id must not be empty"));
        }
        let saved_search = CommunicationCommandService::new(self.pool.clone())
            .update_saved_search(
                &req.saved_search_id,
                UpdateCommunicationSavedSearch {
                    name: req.name,
                    description: req.description,
                    account_id: req.account_id,
                    query: req.query,
                    workflow_state: req
                        .workflow_state
                        .as_deref()
                        .map(parse_workflow_state)
                        .transpose()?,
                    local_state: req
                        .local_state
                        .as_deref()
                        .map(parse_local_state)
                        .transpose()?,
                    channel_kind: req.channel_kind,
                    is_smart_folder: req.is_smart_folder,
                    sort_order: req.sort_order,
                },
            )
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "saved search was not found"))?;

        Response::ok(UpdateSavedSearchResponse {
            item: Some(super::super::communications_saved_folder_proto::saved_search(saved_search))
                .into(),
            ..Default::default()
        })
    }

    async fn delete_saved_search(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteSavedSearchRequest>,
    ) -> ServiceResult<DeleteSavedSearchResponse> {
        let req = req.to_owned_message();
        if req.saved_search_id.trim().is_empty() {
            return Err(invalid_argument_error("saved_search_id must not be empty"));
        }
        let deleted = CommunicationCommandService::new(self.pool.clone())
            .delete_saved_search(&req.saved_search_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteSavedSearchResponse {
            deleted,
            ..Default::default()
        })
    }

    async fn create_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateFolderRequest>,
    ) -> ServiceResult<CreateFolderResponse> {
        let req = req.to_owned_message();
        let folder = CommunicationCommandService::new(self.pool.clone())
            .create_folder(NewCommunicationFolder {
                folder_id: req.folder_id,
                account_id: req.account_id,
                name: req.name,
                description: req.description,
                color: req.color,
                sort_order: req.sort_order,
            })
            .await
            .map_err(command_connect_error)?;

        Response::ok(CreateFolderResponse {
            item: Some(super::super::communications_saved_folder_proto::folder(
                folder,
            ))
            .into(),
            ..Default::default()
        })
    }

    async fn update_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateFolderRequest>,
    ) -> ServiceResult<UpdateFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        let folder = CommunicationCommandService::new(self.pool.clone())
            .update_folder(
                &req.folder_id,
                UpdateCommunicationFolder {
                    account_id: req.account_id,
                    name: req.name,
                    description: req.description,
                    color: req.color,
                    sort_order: req.sort_order,
                },
            )
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "folder was not found"))?;

        Response::ok(UpdateFolderResponse {
            item: Some(super::super::communications_saved_folder_proto::folder(
                folder,
            ))
            .into(),
            ..Default::default()
        })
    }

    async fn delete_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DeleteFolderRequest>,
    ) -> ServiceResult<DeleteFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        let deleted = CommunicationCommandService::new(self.pool.clone())
            .delete_folder(&req.folder_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(DeleteFolderResponse {
            deleted,
            ..Default::default()
        })
    }

    async fn list_folder_messages(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListFolderMessagesRequest>,
    ) -> ServiceResult<ListFolderMessagesResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        let page_request = req.page.as_option();
        let page = self
            .folder_store
            .list_messages(FolderMessageListQuery {
                folder_id: &req.folder_id,
                cursor: page_request
                    .and_then(|page| (!page.cursor.is_empty()).then_some(page.cursor.as_str())),
                limit: normalize_limit(
                    page_request.map(|page| page.limit).unwrap_or(100),
                    100,
                    1000,
                ),
            })
            .await
            .map_err(folder_connect_error)?;
        Response::ok(ListFolderMessagesResponse {
            items: page
                .items
                .into_iter()
                .map(super::super::communications_folder_proto::message)
                .collect(),
            page: Some(PageResponse {
                next_cursor: page.next_cursor.unwrap_or_default(),
                has_more: page.has_more,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        })
    }

    async fn copy_message_to_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CopyMessageToFolderRequest>,
    ) -> ServiceResult<CopyMessageToFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let response = CommunicationCommandService::new(self.pool.clone())
            .copy_message_to_folder(&req.folder_id, &req.message_id)
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(
                    ErrorCode::NotFound,
                    "folder message copy target was not found",
                )
            })?;

        Response::ok(CopyMessageToFolderResponse {
            item: Some(super::super::communications_folder_proto::message_action(
                response,
            ))
            .into(),
            ..Default::default()
        })
    }

    async fn move_message_to_folder(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MoveMessageToFolderRequest>,
    ) -> ServiceResult<MoveMessageToFolderResponse> {
        let req = req.to_owned_message();
        if req.folder_id.trim().is_empty() {
            return Err(invalid_argument_error("folder_id must not be empty"));
        }
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        let response = CommunicationCommandService::new(self.pool.clone())
            .move_message_to_folder(&req.folder_id, &req.message_id)
            .await
            .map_err(command_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(
                    ErrorCode::NotFound,
                    "folder message move target was not found",
                )
            })?;

        Response::ok(MoveMessageToFolderResponse {
            item: Some(super::super::communications_folder_proto::message_action(
                response,
            ))
            .into(),
            ..Default::default()
        })
    }

    async fn send_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, SendMessageRequest>,
    ) -> ServiceResult<SendMessageResponse> {
        let req = req.to_owned_message();
        if !req.confirmed_provider_write {
            return Err(invalid_argument_error(
                "provider write confirmation is required",
            ));
        }
        let metadata = super::super::communications_json_policy::parse_json_object(
            req.metadata_json.as_str(),
            "metadata_json",
        )?;
        let scheduled_send_at = req
            .scheduled_send_at
            .as_deref()
            .map(super::super::communications_timestamp_policy::parse_timestamp)
            .transpose()?;
        let result = send_email(
            &CommunicationSendDependencies::new(self.pool.clone(), self.audit_log.clone()),
            CommunicationSendRequest {
                account_id: req.account_id,
                to: req.to_recipients,
                cc: req.cc_recipients,
                bcc: req.bcc_recipients,
                subject: req.subject,
                body_text: req.body_text,
                body_html: req.body_html,
                in_reply_to: req.in_reply_to,
                references: req.references,
                draft_id: req.draft_id,
                scheduled_send_at,
                undo_send_seconds: req.undo_send_seconds,
                metadata,
            },
        )
        .await
        .map_err(send_connect_error)?;
        let outbox_id = result.outbox_id.clone().ok_or_else(|| {
            ConnectError::new(
                ErrorCode::Internal,
                "send_email did not return an outbox item identifier",
            )
        })?;
        let item = self
            .outbox_store
            .get(&outbox_id)
            .await
            .map_err(outbox_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::Internal, "queued outbox item was not found")
            })?;

        Response::ok(SendMessageResponse {
            item: Some(super::super::communications_outbox_proto::item(item)).into(),
            message_id: result.message_id,
            outbox_id: result.outbox_id,
            accepted: result.accepted,
            accepted_recipients: result.accepted_recipients,
            transport: result.transport,
            status: result.status,
            scheduled_send_at: result.scheduled_send_at.map(timestamp_string),
            undo_deadline_at: result.undo_deadline_at.map(timestamp_string),
            failure_reason: result.failure_reason,
            ..Default::default()
        })
    }

    async fn redirect_message(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RedirectMessageRequest>,
    ) -> ServiceResult<RedirectMessageResponse> {
        let req = req.to_owned_message();
        if req.message_id.trim().is_empty() {
            return Err(invalid_argument_error("message_id must not be empty"));
        }
        if !req.confirmed_provider_write {
            return Err(invalid_argument_error(
                "provider write confirmation is required",
            ));
        }

        let to = super::super::communications_request_policy::trim_non_empty_recipients(
            req.to_recipients,
        );
        let cc = super::super::communications_request_policy::trim_non_empty_recipients(
            req.cc_recipients,
        );
        let bcc = super::super::communications_request_policy::trim_non_empty_recipients(
            req.bcc_recipients,
        );
        if to.is_empty() && cc.is_empty() && bcc.is_empty() {
            return Err(invalid_argument_error("at least one recipient is required"));
        }

        let message = self
            .message_store
            .message(&req.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let recipient_count = to.len() + cc.len() + bcc.len();
        let outbox = CommunicationCommandService::new(self.pool.clone())
            .enqueue_redirect_message(&message.message_id, to.clone(), cc, bcc)
            .await
            .map_err(command_connect_error)?;

        self.audit_log
            .record(&NewApiAuditRecord::communication_email_send(
                "makosh-frontend",
                &outbox.account_id,
                recipient_count,
            ))
            .await
            .map_err(audit_connect_error)?;

        Response::ok(RedirectMessageResponse {
            message_id: outbox.outbox_id.clone(),
            outbox_id: Some(outbox.outbox_id),
            accepted: outbox.to_recipients.clone(),
            accepted_recipients: outbox.to_recipients,
            transport: "outbox".to_owned(),
            status: outbox.status.as_str().to_owned(),
            scheduled_send_at: outbox.scheduled_send_at.map(timestamp_string),
            undo_deadline_at: outbox.undo_deadline_at.map(timestamp_string),
            failure_reason: None,
            ..Default::default()
        })
    }

    async fn search_attachments(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, AttachmentSearchRequest>,
    ) -> ServiceResult<AttachmentSearchResponse> {
        let req = req.to_owned_message();
        let page = self
            .attachment_search_store
            .search(AttachmentSearchQuery {
                account_id: req.account_id.as_deref(),
                query: req.query.as_deref(),
                content_type: req.content_type.as_deref(),
                scan_status: req.scan_status.as_deref(),
                cursor: req.cursor.as_deref(),
                limit: normalize_limit(req.limit, 100, 250),
            })
            .await
            .map_err(attachment_search_connect_error)?;

        Response::ok(super::super::communications_attachment_search_proto::page(
            page,
        ))
    }

    async fn get_attachment_preview(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetAttachmentPreviewRequest>,
    ) -> ServiceResult<GetAttachmentPreviewResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let attachment = self
            .storage_store
            .attachment_by_id(&req.attachment_id)
            .await
            .map_err(storage_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "attachment was not found"))?;

        if attachment.storage_kind != "local_fs" {
            return Err(invalid_argument_error(
                "attachment preview requires a local blob",
            ));
        }
        if !super::super::communications_attachment_policy::allowed_by_scan_status(&attachment) {
            return Err(invalid_argument_error(
                "attachment preview is blocked by attachment scan status",
            ));
        }
        if let Some(preview) = crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewService::new(
            self.pool.clone(),
            crate::app::api_support::stores::domain_stores::communication_blob_store(),
        )
        .completed_preview(&req.attachment_id)
        .await
        .map_err(attachment_safe_preview_connect_error)?
        {
            let byte_count = preview.bytes.len();
            return super::super::communications_attachment_preview::image(attachment, preview.bytes, byte_count);
        }
        if super::super::communications_attachment_policy::is_derived_text(&attachment) {
            let derived = crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionService::new(
                self.pool.clone(),
                crate::app::api_support::stores::domain_stores::communication_blob_store(),
            )
            .completed_text(&req.attachment_id)
            .await
            .map_err(attachment_text_extraction_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(
                    ErrorCode::FailedPrecondition,
                    "extract attachment text before preview",
                )
            })?;
            let bytes = derived.text.into_bytes();
            let byte_count = bytes.len();
            return super::super::communications_attachment_preview::text(
                attachment, bytes, byte_count,
            );
        }
        let preview_kind = attachment_preview_kind(&attachment).ok_or_else(|| {
            invalid_argument_error(
                "attachment preview supports text, image, audio and video attachments only",
            )
        })?;

        let bytes = crate::app::api_support::stores::domain_stores::communication_blob_store()
            .read_blob(&attachment.storage_path)
            .await
            .map_err(storage_connect_error)?;
        let byte_count = bytes.len();

        match preview_kind {
            AttachmentPreviewKind::Text => {
                super::super::communications_attachment_preview::text(attachment, bytes, byte_count)
            }
            AttachmentPreviewKind::Image => super::super::communications_attachment_preview::image(
                attachment, bytes, byte_count,
            ),
            AttachmentPreviewKind::Audio => super::super::communications_attachment_preview::audio(
                attachment, bytes, byte_count,
            ),
            AttachmentPreviewKind::Video => super::super::communications_attachment_preview::video(
                attachment, bytes, byte_count,
            ),
        }
    }

    async fn extract_attachment_text(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ExtractAttachmentTextRequest>,
    ) -> ServiceResult<ExtractAttachmentTextResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let service = crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionService::new(
            self.pool.clone(),
            crate::app::api_support::stores::domain_stores::communication_blob_store(),
        );
        let outcome = service
            .extract(&req.attachment_id)
            .await
            .map_err(attachment_text_extraction_connect_error)?;
        match outcome {
            crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionOutcome::Completed {
                attachment_id,
                extracted_size_bytes,
                ..
            } => Response::ok(ExtractAttachmentTextResponse {
                attachment_id,
                status: "completed".to_owned(),
                extracted_size_bytes: Some(extracted_size_bytes as u64),
                ..Default::default()
            }),
            crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionOutcome::Unsupported {
                attachment_id,
            } => Response::ok(ExtractAttachmentTextResponse {
                attachment_id,
                status: "unsupported".to_owned(),
                ..Default::default()
            }),
        }
    }

    async fn get_attachment_extracted_text(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetAttachmentExtractedTextRequest>,
    ) -> ServiceResult<GetAttachmentExtractedTextResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let service = crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionService::new(
            self.pool.clone(),
            crate::app::api_support::stores::domain_stores::communication_blob_store(),
        );
        let content = service
            .completed_text(&req.attachment_id)
            .await
            .map_err(attachment_text_extraction_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "attachment text was not extracted")
            })?;
        Response::ok(GetAttachmentExtractedTextResponse {
            attachment_id: content.attachment_id,
            text: content.text,
            truncated: content.truncated,
            extracted_size_bytes: content.extracted_size_bytes as u64,
            ..Default::default()
        })
    }

    async fn get_attachment_archive_inspection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetAttachmentArchiveInspectionRequest>,
    ) -> ServiceResult<GetAttachmentArchiveInspectionResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let attachment = self
            .storage_store
            .attachment_by_id(&req.attachment_id)
            .await
            .map_err(storage_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "attachment was not found"))?;

        if attachment.storage_kind != "local_fs" {
            return Err(invalid_argument_error(
                "attachment archive inspection requires a local blob",
            ));
        }
        if !super::super::communications_attachment_policy::allowed_by_scan_status(&attachment) {
            return Err(invalid_argument_error(
                "attachment archive inspection is blocked by attachment scan status",
            ));
        }
        if !super::super::communications_attachment_policy::is_zip(&attachment) {
            return Err(invalid_argument_error(
                "attachment archive inspection supports ZIP attachments only",
            ));
        }

        let report = match cached_archive_inspection_report(
            &attachment.attachment.scan_metadata,
            &attachment.attachment.sha256,
        ) {
            Some(report) => report,
            None => {
                let bytes =
                    crate::app::api_support::stores::domain_stores::communication_blob_store()
                        .read_blob(&attachment.storage_path)
                        .await
                        .map_err(storage_connect_error)?;
                let report = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default())
                    .map_err(|error| {
                        tracing::warn!(
                            attachment_id = %attachment.attachment.attachment_id,
                            error = %error,
                            "attachment archive inspection rejected archive"
                        );
                        invalid_argument_error("attachment archive inspection failed")
                    })?;
                self.storage_store
                    .persist_archive_inspection(
                        &attachment.attachment.attachment_id,
                        &attachment.attachment.sha256,
                        &report,
                    )
                    .await
                    .map_err(storage_connect_error)?;
                report
            }
        };

        Response::ok(GetAttachmentArchiveInspectionResponse {
            attachment_id: attachment.attachment.attachment_id,
            message_id: attachment.attachment.message_id,
            filename: attachment.attachment.filename,
            content_type: attachment.attachment.content_type,
            scan_status: attachment.attachment.scan_status.as_str().to_owned(),
            report: Some(super::super::communications_archive_proto::inspection_report(report))
                .into(),
            ..Default::default()
        })
    }

    async fn translate_attachment(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, TranslateAttachmentRequest>,
    ) -> ServiceResult<TranslateAttachmentResponse> {
        let req = req.to_owned_message();
        if req.attachment_id.trim().is_empty() {
            return Err(invalid_argument_error("attachment_id must not be empty"));
        }
        let target_language = req.target_language.trim();
        if target_language.is_empty() {
            return Err(invalid_argument_error("target_language is required"));
        }
        let attachment = self
            .storage_store
            .attachment_by_id(&req.attachment_id)
            .await
            .map_err(storage_connect_error)?
            .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "attachment was not found"))?;
        if attachment.attachment.scan_status
            != crate::domains::communications::storage::scanner::AttachmentSafetyScanStatus::Clean
        {
            return Err(invalid_argument_error(
                "attachment translation requires a clean scan verdict",
            ));
        }
        let message = self
            .message_store
            .message(&attachment.attachment.message_id)
            .await
            .map_err(message_connect_error)?
            .ok_or_else(|| {
                ConnectError::new(ErrorCode::NotFound, "communication message was not found")
            })?;
        let source_text = crate::domains::communications::attachment_text_extraction::AttachmentTextExtractionService::new(
            self.pool.clone(),
            crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore::new(
                crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT,
            ),
        )
        .completed_text(&attachment.attachment.attachment_id)
        .await
        .map_err(attachment_text_extraction_connect_error)?
        .ok_or_else(|| {
            ConnectError::new(
                ErrorCode::FailedPrecondition,
                "extract attachment text before translation",
            )
        })?;
        super::super::communications_ai_runtime::require_mail_content_egress(
            &self.pool,
            &self.config,
            &message.account_id,
            crate::app::api_support::stores::ai_runtime::MailAiContentEgressKind::ExtractedText,
        )
        .await?;
        let service =
            super::super::communications_ai_runtime::multilingual_service(&self.pool, &self.config)
                .await
                .map_err(api_error_connect_error)?;
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &source_text.text,
            );

        match service.translate(&source_text.text, target_language).await {
            Ok(Some(translation)) => {
                crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort(
                    self.pool.clone(),
                    "attachment_translation",
                    &attachment.attachment.message_id,
                    serde_json::json!({
                        "kind": "communication_attachment",
                        "source_code": "ai",
                        "message_id": attachment.attachment.message_id,
                        "attachment_id": attachment.attachment.attachment_id,
                        "operation": "attachment_translation",
                    }),
                    serde_json::json!({
                        "target_language": translation.target_language,
                        "original_language": detection.language,
                        "model": translation.model,
                    }),
                    serde_json::json!({
                        "source": "communication_attachment_translation",
                        "attachment_id": attachment.attachment.attachment_id,
                        "message_id": attachment.attachment.message_id,
                    }),
                    None,
                )
                .await;

                Response::ok(TranslateAttachmentResponse {
                    attachment_id: attachment.attachment.attachment_id,
                    message_id: attachment.attachment.message_id,
                    filename: attachment.attachment.filename,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: true,
                    text: Some(translation.translated_text),
                    target: translation.target_language,
                    model: Some(translation.model),
                    reason: None,
                    source: ATTACHMENT_TRANSLATION_SOURCE.to_owned(),
                    ..Default::default()
                })
            }
            Ok(None) => Response::ok(TranslateAttachmentResponse {
                attachment_id: attachment.attachment.attachment_id,
                message_id: attachment.attachment.message_id,
                filename: attachment.attachment.filename,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: false,
                text: None,
                target: target_language.to_owned(),
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
                source: ATTACHMENT_TRANSLATION_SOURCE.to_owned(),
                ..Default::default()
            }),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    attachment_id = %attachment.attachment.attachment_id,
                    "attachment translation failed"
                );
                Response::ok(TranslateAttachmentResponse {
                    attachment_id: attachment.attachment.attachment_id,
                    message_id: attachment.attachment.message_id,
                    filename: attachment.attachment.filename,
                    original_language: detection.language,
                    confidence: detection.confidence,
                    translated: false,
                    text: None,
                    target: target_language.to_owned(),
                    model: None,
                    reason: Some("translation runtime unavailable".to_owned()),
                    source: ATTACHMENT_TRANSLATION_SOURCE.to_owned(),
                    ..Default::default()
                })
            }
        }
    }

    async fn undo_outbox_item(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UndoOutboxItemRequest>,
    ) -> ServiceResult<UndoOutboxItemResponse> {
        let req = req.to_owned_message();
        if req.outbox_id.trim().is_empty() {
            return Err(invalid_argument_error("outbox_id must not be empty"));
        }
        let item = CommunicationCommandService::new(self.pool.clone())
            .undo_outbox(&req.outbox_id)
            .await
            .map_err(command_connect_error)?;

        Response::ok(UndoOutboxItemResponse {
            item: Some(super::super::communications_outbox_proto::item(item)).into(),
            ..Default::default()
        })
    }
}

pub(super) enum AttachmentPreviewKind {
    Text,
    Image,
    Audio,
    Video,
}
